# `README.md`: Fedora Native Transcriber (CPU Beast Mode)

This project uses **Rust** and **Whisper.cpp (via `whisper-rs`)** to provide a high-performance transcription GUI for Linux. It is optimized to utilize many CPU cores using the efficient GGML model format, achieving fast and stable results on Fedora.

## 1\. Prerequisites

You must have the following tools installed on your Fedora system:

  * **Rust Toolchain:** (Installed via `rustup`)
  * **GCC/C++ Compiler:** (`sudo dnf install gcc-c++`)
  * **CMake:** (`sudo dnf install cmake`)
  * **GTK Headers:** For the GUI (`sudo dnf install gtk3-devel`)
  * **FFmpeg:** Required for audio extraction (`sudo dnf install ffmpeg`)

## 2\. Model Downloads (GGML Format)

The Rust application requires models in the **GGML (.bin)** format, which is different from the Python CTranslate2 models. The app is configured to automatically scan for these files inside the **`models/`** folder.

Run the commands below to download the desired models into the correct directory.

### **Setup the `models` Directory**

```bash
mkdir -p models
cd models
```

### **A. Recommended Models (WGET Commands)**

Quantized models (like `q5_1` or `q5_0`) are highly recommended as they offer near-Large model accuracy at much smaller sizes, maximizing CPU speed.

| Model Name | Size | Accuracy | WGET Command |
| :--- | :--- | :--- | :--- |
| **Small** | \~190 MB | Great speed/quality balance | `wget -O ggml-small.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin` |
| **Medium** | \~539 MB | Excellent quality, still fast | `wget -O ggml-medium-q5_0.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin` |
| **Large-v3** | \~1.08 GB | Highest quality, slower | `wget -O ggml-large-v3-q5_0.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin` |
| **Base** | \~148 MB | Fast, good for clear audio | `wget -O ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin` |

***Note:** Your application's Model dropdown will automatically populate with all `.bin` files found in the `models/` directory.*

## 3\. Build and Run the Application

Once your models are downloaded, you can build the final, optimized native binary.

```bash
# Return to the project root
cd .. 

# Build the optimized release binary (this takes a moment on the first run)
cargo build --release

# Run the GUI application
cargo run --release
```
