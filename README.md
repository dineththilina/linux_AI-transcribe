# `README.md`: Native Transcriber (CPU Mode)

### **Cross-Platform Linux Compatibility Note**

**This application is fully compatible with any major Linux distribution (Ubuntu, Debian, Arch, Fedora, etc.)** The "Fedora" naming convention refers only to the environment where this high-performance build was developed and optimized. The core technology uses standard Rust and C++ libraries, ensuring maximum compatibility across the Linux ecosystem.

-----

## 1\. Prerequisites

You must have the following general Linux development tools installed on your system:

| Tool | Purpose | Installation (Common `dnf`/`apt` equivalents) |
| :--- | :--- | :--- |
| **Rust Toolchain** | Core Language | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` |
| **CMake** | Build System | `sudo dnf install cmake` or `sudo apt install cmake` |
| **C/C++ Compiler** | Required by `whisper-rs` | `sudo dnf install gcc-c++` or `sudo apt install build-essential` |
| **GTK Headers** | For the native GUI | `sudo dnf install gtk3-devel` or `sudo apt install libgtk-3-dev` |
| **FFmpeg** | Required for audio extraction | `sudo dnf install ffmpeg` or `sudo apt install ffmpeg` |

## 2\. Model Downloads (GGML Format)

The application requires models in the **GGML (.bin)** format, which is different from the Python CTranslate2 models. The app is configured to automatically scan for these files inside the **`models/`** folder.

### **Setup the `models` Directory**

```bash
mkdir -p models
cd models
```

### **A. Recommended Models (WGET Commands)**

| Model Name | Size | Accuracy | WGET Command |
| :--- | :--- | :--- | :--- |
| **Small** | \~190 MB | Great speed/quality balance | `wget -O ggml-small.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin` |
| **Medium** | \~539 MB | Excellent quality, still fast | `wget -O ggml-medium-q5_0.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin` |
| **Large-v3** | \~1.08 GB | Highest quality, slower | `wget -O ggml-large-v3-q5_0.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin` |
| **Base** | \~148 MB | Fast, good for clear audio | `wget -O ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin` |

***Note:** Your application's Model dropdown will automatically populate with all `.bin` files found in the `models/` directory.*

## 3\. Build and Run the Application

Navigate to your project root folder and build the optimized native binary.

```bash
# Return to the project root
cd .. 

# Build the optimized release binary (uses all available CPU cores)
cargo build --release

# Run the GUI application
cargo run --release
```
## 4\. Prerequisites: System Dependencies

The application relies on several core Linux development packages that must be installed before the Rust build can succeed.

| Component | Fedora/RHEL (`dnf` commands) | Debian/Ubuntu (`apt` commands) | Purpose |
| :--- | :--- | :--- | :--- |
| **Rust Toolchain** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` | Core compiler, build system |
| **C/C++ Build Tools** | `sudo dnf install gcc-c++ clang` | `sudo apt install build-essential clang` | Compiles the C/C++ foundation (`whisper.cpp`) |
| **CMake** | `sudo dnf install cmake` | `sudo apt install cmake` | Manages the C++ build process |
| **GUI/Graphics** | `sudo dnf install gtk3-devel` | `sudo apt install libgtk-3-dev` | Required by the native GUI toolkit (`eframe`) |
| **Audio/Media** | `sudo dnf install alsa-lib-devel ffmpeg` | `sudo apt install libasound2-dev ffmpeg` | Audio I/O and video-to-audio extraction |

