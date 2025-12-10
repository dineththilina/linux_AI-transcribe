use eframe::egui;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::fs;
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use walkdir::WalkDir;

// --- CONFIGURATION ---
const MODELS_DIR: &str = "./models"; // Generic relative path

// --- MESSAGES & SIGNALS ---

struct CancellationToken {
    receiver: Receiver<()>,
}
impl CancellationToken {
    fn new(receiver: Receiver<()>) -> Self {
        CancellationToken { receiver }
    }
    fn is_cancelled(&self) -> bool {
        match self.receiver.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => true,
            Err(TryRecvError::Empty) => false,
        }
    }
}
enum AppMessage {
    Log(String),
    Finished(String, String),
    Error(String),
    Cancellation(Sender<()>),
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Fedora Native Transcriber (Rust)"),
        ..Default::default()
    };
    
    eframe::run_native(
        "fedora_whisper_gui",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

struct MyApp {
    video_path: Option<PathBuf>,
    available_models: Vec<PathBuf>,
    selected_model_idx: usize,
    is_transcribing: bool,
    transcript: String,
    logs: String,
    duration_str: String,
    receiver: Receiver<AppMessage>,
    sender: Sender<AppMessage>,
    cancellation_sender: Option<Sender<()>>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        
        // 1. AUTO-SCAN MODELS
        let mut models = Vec::new();
        let models_path = Path::new(MODELS_DIR);
        if models_path.exists() {
            // Use canonicalize to resolve the relative path
            if let Ok(path) = models_path.canonicalize() {
                for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
                    if let Ok(e) = entry {
                        let path = e.path().to_path_buf();
                        if let Some(ext) = path.extension() {
                            if ext == "bin" {
                                models.push(path);
                            }
                        }
                    }
                }
            }
        }
        models.sort();

        Self {
            video_path: None,
            available_models: models,
            selected_model_idx: 0,
            is_transcribing: false,
            transcript: String::new(),
            logs: format!("Ready. Place models in '{}' and select a video.", MODELS_DIR).to_owned(),
            duration_str: String::new(),
            receiver,
            sender,
            cancellation_sender: None,
        }
    }
}

impl eframe::App for MyApp { // <-- The impl block that needed a closing brace
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process Messages
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                AppMessage::Log(text) => self.logs = text,
                AppMessage::Finished(status, text) => {
                    self.is_transcribing = false;
                    self.logs = status;
                    self.transcript = text;
                    self.cancellation_sender = None; // Clear on finish
                }
                AppMessage::Error(e) => {
                    self.is_transcribing = false;
                    self.logs = format!("❌ Error: {}", e);
                    self.cancellation_sender = None; // Clear on error
                }
                AppMessage::Cancellation(tx) => {
                    self.cancellation_sender = Some(tx); // Store the sender for the Stop button
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Fedora Native Transcriber");
            ui.separator();

            // VIDEO SELECTION
            ui.horizontal(|ui| {
                if ui.button("📂 Select Video").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.video_path = Some(path.clone());
                        self.logs = "Video selected.".to_string();
                        if let Ok(d) = get_duration_str(&path) {
                            self.duration_str = d;
                        }
                    }
                }
                if let Some(path) = &self.video_path {
                    ui.strong(path.file_name().unwrap_or_default().to_string_lossy());
                    if !self.duration_str.is_empty() {
                        ui.label(format!("({})", self.duration_str));
                    }
                } else {
                    ui.label("No file selected");
                }
            });

            // MODEL DROPDOWN
            ui.horizontal(|ui| {
                ui.label("🧠 Model:");
                if self.available_models.is_empty() {
                    ui.colored_label(egui::Color32::RED, format!("No .bin models found in {}", MODELS_DIR));
                } else {
                    egui::ComboBox::from_id_salt("model_combo")
                        .width(300.0)
                        .show_index(
                            ui,
                            &mut self.selected_model_idx,
                            self.available_models.len(),
                            |i| self.available_models[i].file_name().unwrap().to_string_lossy().to_string()
                        );
                }
            });

            ui.separator();

            // TRANSCRIPTION CONTROL
            ui.horizontal(|ui| {
                if self.is_transcribing {
                    // STOP BUTTON LOGIC
                    // FIX E0283: Explicitly use egui::Vec2 for the size.
                    let stop_button = ui.add_sized(egui::Vec2::new(120.0, 30.0), egui::Button::new("◼ Stop Transcribing"));
                    
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Spinner::new().size(20.0));
                        ui.label(format!("Processing: {}", self.logs));
                    });

                    if stop_button.clicked() {
                        if let Some(tx_cancel) = &self.cancellation_sender {
                            // Signal the transcription thread to stop
                            let _ = tx_cancel.send(()); 
                            self.is_transcribing = false;
                            self.logs = "Stopping transcription...".to_string();
                            self.cancellation_sender = None; // Clear immediately
                        }
                    }
                } else {
                    // START BUTTON LOGIC
                    let enabled = self.video_path.is_some() && !self.available_models.is_empty();
                    if ui.add_enabled(enabled, egui::Button::new("▶ Start Transcribing").min_size([120.0, 30.0].into())).clicked() {
                        self.start_transcription();
                    }
                }
            });
            
            // ACTIVITY INDICATOR
            if self.is_transcribing {
                ui.add(egui::ProgressBar::new(0.5).animate(true).text("CPU Crunching..."));
                ui.small("(Check your black Terminal window to see live text segments!)");
            }

            ui.separator();

            // TRANSCRIPT & ACTIONS
            ui.horizontal(|ui| {
                ui.label("Transcript:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    // EXPORT BUTTON LOGIC
                    if ui.button("💾 Export .txt").clicked() {
                        self.export_transcript();
                    }
                    // COPY BUTTON LOGIC
                    if ui.button("📋 Copy All Text").clicked() {
                        ctx.output_mut(|o| o.copied_text = self.transcript.clone());
                        self.logs = "Transcript copied to clipboard!".to_string();
                    }
                });
            });


            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut self.transcript)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .lock_focus(true)
                );
            });
        });
    } // <-- Closing brace for fn update
} // <-- Closing brace for impl eframe::App for MyApp

impl MyApp {
    fn start_transcription(&mut self) {
        self.is_transcribing = true;
        self.transcript.clear();
        self.logs = "Initializing...".to_string();

        let video = self.video_path.clone().unwrap();
        let model = self.available_models[self.selected_model_idx].clone();
        let tx_msg = self.sender.clone();

        // FIX E0382: Clone 'video' here to maintain ownership for cleanup later.
        let video_for_cleanup = video.clone(); 

        // Channels for the cancellation token
        let (tx_cancel, rx_cancel) = unbounded::<()>();
        let cancel_token = CancellationToken::new(rx_cancel);

        // Send the cancellation sender back to the main thread immediately
        tx_msg.send(AppMessage::Cancellation(tx_cancel)).unwrap();


        thread::spawn(move || {
            // PASS ORIGINAL VIDEO: Pass the original pathbuf (which is moved) to the logic function.
            let result = run_whisper_logic(video, model, tx_msg.clone(), cancel_token);
            
            // Clean up the temporary audio file regardless of success/failure
            if let Some(file_stem) = video_for_cleanup.file_stem().and_then(|s| s.to_str()) {
                let audio_out = format!("/dev/shm/{}_temp.wav", file_stem);
                let _ = fs::remove_file(&audio_out); 
            }

            if let Err(e) = result {
                let msg = e.to_string();
                if msg.contains("Transcription cancelled") {
                    tx_msg.send(AppMessage::Finished("Transcription Stopped by User".into(), String::new())).unwrap();
                } else {
                    tx_msg.send(AppMessage::Error(msg)).unwrap();
                }
            }
        });
    }

    fn export_transcript(&mut self) {
        if self.transcript.is_empty() {
            self.logs = "❌ Nothing to export!".to_string();
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("transcript.txt")
            .save_file() 
        {
            match fs::write(&path, &self.transcript) {
                Ok(_) => self.logs = format!("✅ Exported to: {}", path.display()),
                Err(e) => self.logs = format!("❌ Export failed: {}", e),
            }
        }
    }
}

fn get_duration_str(path: &PathBuf) -> anyhow::Result<String> {
    let output = Command::new("ffprobe")
        .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", path.to_str().unwrap()])
        .output()?;
    let val = String::from_utf8(output.stdout)?.trim().parse::<f64>().unwrap_or(0.0);
    let mins = (val / 60.0).floor();
    let secs = val % 60.0;
    Ok(format!("{}m {:.0}s", mins, secs))
}

fn run_whisper_logic(video: PathBuf, model: PathBuf, tx: Sender<AppMessage>, cancel_token: CancellationToken) -> anyhow::Result<()> {
    
    // --- AUDIO EXTRACTION ---
    tx.send(AppMessage::Log("Extracting audio to RAM...".into()))?;

    let file_stem = video.file_stem().unwrap().to_str().unwrap();
    let audio_out = format!("/dev/shm/{}_temp.wav", file_stem);

    let status = Command::new("ffmpeg")
        .args(&["-y", "-i", video.to_str().unwrap(), "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le", &audio_out])
        .output()?;

    if !status.status.success() {
        return Err(anyhow::anyhow!("FFmpeg failed: {}", String::from_utf8_lossy(&status.stderr)));
    }
    
    // Check cancellation after a potentially long operation (like FFmpeg)
    if cancel_token.is_cancelled() {
        return Err(anyhow::anyhow!("Transcription cancelled after audio extraction"));
    }


    // --- MODEL LOADING & PARAMETERS ---
    tx.send(AppMessage::Log("Loading Model...".into()))?;
    let ctx = WhisperContext::new_with_params(
        model.to_str().unwrap(), 
        WhisperContextParameters::default()
    ).map_err(|_| anyhow::anyhow!("Failed to load model"))?;

    let mut state = ctx.create_state().expect("failed to create state");

    let mut reader = hound::WavReader::open(&audio_out)?;
    let audio_data: Vec<f32> = reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / 32768.0)
        .collect();

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(16);
    params.set_print_special(false);
    params.set_language(Some("en")); 
    
    // Anti-Loop / Hallucination fix
    params.set_no_speech_thold(0.6); 
    params.set_temperature(0.0);
    
    // Enable live text in terminal
    params.set_print_realtime(true); 

    // FIX E0308: set_abort_callback removed because it requires a C-compatible function pointer,
    // and our closure captures variables.

    // --- TRANSCRIPTION ---
    tx.send(AppMessage::Log("Transcribing (See Terminal)...".into()))?;

    // The state.full() call runs
    state.full(params, &audio_data[..]).expect("failed to run model");

    
    // --- RESULT PROCESSING ---
    
    // Check if we were cancelled *before* processing segments
    if cancel_token.is_cancelled() {
        return Err(anyhow::anyhow!("Transcription cancelled"));
    }

    let num_segments = state.full_n_segments().expect("failed to get segments");
    let mut full_text = String::new();

    for i in 0..num_segments {
        let text = state.full_get_segment_text(i).expect("failed text");
        let start = state.full_get_segment_t0(i).unwrap_or(0) / 100;
        let end = state.full_get_segment_t1(i).unwrap_or(0) / 100;
        
        // Format time as M:SS
        let line = format!("[{:>02}:{:02} - {:>02}:{:02}] {}\n", start / 60, start % 60, end / 60, end % 60, text.trim());
        full_text.push_str(&line);
    }

    tx.send(AppMessage::Finished("Transcription Complete".into(), full_text))?;
    
    Ok(())
}
