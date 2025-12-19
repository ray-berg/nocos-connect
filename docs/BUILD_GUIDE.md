# NOCOS Connect Build and Installation Guide

This document provides comprehensive instructions for building NOCOS Connect from source on all supported platforms.

## Table of Contents

1. [System Requirements](#1-system-requirements)
2. [Dependencies Overview](#2-dependencies-overview)
3. [Linux Build](#3-linux-build)
4. [Windows Build](#4-windows-build)
5. [macOS Build](#5-macos-build)
6. [Android Build](#6-android-build)
7. [iOS Build](#7-ios-build)
8. [Docker Build](#8-docker-build)
9. [Feature Flags](#9-feature-flags)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. System Requirements

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| **Rust** | 1.75 or later |
| **Flutter** | 3.1.0 or later (Dart SDK 3.1.0+) |
| **Python** | 3.8+ (for build scripts) |
| **Git** | 2.0+ |
| **Disk Space** | 20GB+ (including dependencies) |
| **RAM** | 8GB minimum, 16GB recommended |

### Platform-Specific Requirements

| Platform | Requirements |
|----------|-------------|
| **Linux** | Ubuntu 20.04+, Debian 11+, or equivalent |
| **Windows** | Windows 10/11, Visual Studio 2019+ |
| **macOS** | macOS 10.14+, Xcode 12+ |
| **Android** | Android SDK 21+, NDK r22+ |
| **iOS** | iOS 12+, Xcode 12+ |

---

## 2. Dependencies Overview

### 2.1 Core Native Dependencies (vcpkg)

These libraries are installed via vcpkg:

| Library | Version | Purpose |
|---------|---------|---------|
| **libvpx** | latest | VP8/VP9 video codec |
| **libyuv** | latest | YUV image processing |
| **opus** | latest | Audio codec |
| **aom** | latest | AV1 video codec |
| **libjpeg-turbo** | latest | JPEG image processing |

#### Optional vcpkg Dependencies

| Library | Platform | Purpose |
|---------|----------|---------|
| **ffmpeg** | Windows/Linux/macOS | Hardware codec support |
| **mfx-dispatch** | Windows/Linux | Intel Quick Sync Video |
| **oboe** | Android | Low-latency audio |

### 2.2 Rust Crate Dependencies

Key Rust dependencies from `Cargo.toml`:

#### Core Libraries
```toml
hbb_common = { path = "libs/hbb_common" }    # Protocol, config, utilities
scrap = { path = "libs/scrap" }               # Screen capture
enigo = { path = "libs/enigo" }               # Input simulation
clipboard = { path = "libs/clipboard" }       # Clipboard handling
```

#### Networking
```toml
tokio                    # Async runtime
reqwest                  # HTTP client
parity-tokio-ipc        # IPC communication
```

#### Audio/Video
```toml
magnum-opus             # Opus audio codec
cpal                    # Cross-platform audio
```

#### Cryptography
```toml
sodiumoxide             # NaCl crypto (via hbb_common)
sha2                    # SHA-256 hashing
```

#### UI (Desktop)
```toml
sciter-rs               # Legacy Sciter UI
flutter_rust_bridge     # Flutter FFI bindings
```

### 2.3 Flutter Dependencies

Key Flutter packages from `flutter/pubspec.yaml`:

```yaml
flutter_rust_bridge: "1.80.1"    # Rust-Flutter bridge
window_manager                    # Window management
desktop_multi_window             # Multi-window support
texture_rgba_renderer            # Video rendering
```

---

## 3. Linux Build

### 3.1 Install System Dependencies

#### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y \
    g++ \
    gcc \
    git \
    curl \
    wget \
    nasm \
    yasm \
    libgtk-3-dev \
    clang \
    libxcb-randr0-dev \
    libxdo-dev \
    libxfixes-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libasound2-dev \
    libpam0g-dev \
    libpulse-dev \
    libssl-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    cmake \
    ninja-build \
    pkg-config \
    unzip \
    zip
```

#### Fedora/RHEL

```bash
sudo dnf install -y \
    gcc-c++ \
    gcc \
    git \
    curl \
    wget \
    nasm \
    yasm \
    gtk3-devel \
    clang \
    libxcb-devel \
    libxdo-devel \
    libXfixes-devel \
    alsa-lib-devel \
    pam-devel \
    pulseaudio-libs-devel \
    openssl-devel \
    gstreamer1-devel \
    gstreamer1-plugins-base-devel \
    cmake \
    ninja-build \
    pkgconfig \
    unzip \
    zip
```

### 3.2 Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
```

### 3.3 Install vcpkg

```bash
cd /opt
sudo git clone https://github.com/microsoft/vcpkg
cd vcpkg
sudo ./bootstrap-vcpkg.sh -disableMetrics
export VCPKG_ROOT=/opt/vcpkg
echo 'export VCPKG_ROOT=/opt/vcpkg' >> ~/.bashrc
```

### 3.4 Install vcpkg Dependencies

```bash
$VCPKG_ROOT/vcpkg install libvpx libyuv opus aom
```

### 3.5 Install Flutter (for Flutter UI)

```bash
cd /opt
sudo wget https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.19.0-stable.tar.xz
sudo tar xf flutter_linux_3.19.0-stable.tar.xz
export PATH=/opt/flutter/bin:$PATH
echo 'export PATH=/opt/flutter/bin:$PATH' >> ~/.bashrc
flutter doctor
```

### 3.6 Download Sciter Library (for Legacy UI)

```bash
mkdir -p ~/lib
curl -L https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so -o ~/lib/libsciter-gtk.so
export LD_LIBRARY_PATH=$HOME/lib:$LD_LIBRARY_PATH
```

### 3.7 Clone and Build

```bash
git clone --recursive https://github.com/ray-berg/nocos-connect.git
cd nocos-connect

# Build with legacy Sciter UI
cargo build --release

# OR build with Flutter UI
python3 build.py --flutter
```

### 3.8 Output Locations

| Build Type | Output |
|------------|--------|
| Sciter | `target/release/nocos-connect` |
| Flutter | `flutter/build/linux/x64/release/bundle/` |

---

## 4. Windows Build

### 4.1 Install Prerequisites

1. **Visual Studio 2019/2022** with C++ workload
2. **Git for Windows**: https://git-scm.com/download/win
3. **Python 3**: https://www.python.org/downloads/

### 4.2 Install Rust

```powershell
# Download and run rustup-init.exe from https://rustup.rs
rustup default stable
```

### 4.3 Install vcpkg

```powershell
cd C:\
git clone https://github.com/microsoft/vcpkg
cd vcpkg
.\bootstrap-vcpkg.bat -disableMetrics
[Environment]::SetEnvironmentVariable("VCPKG_ROOT", "C:\vcpkg", "User")
$env:VCPKG_ROOT = "C:\vcpkg"
```

### 4.4 Install vcpkg Dependencies

```powershell
vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static aom:x64-windows-static
```

### 4.5 Install Flutter

```powershell
# Download Flutter SDK from https://flutter.dev/docs/get-started/install/windows
# Extract to C:\flutter
$env:PATH += ";C:\flutter\bin"
flutter doctor
```

### 4.6 Download Sciter Library

```powershell
# Download from https://github.com/nicecpp/nicecpp-assets/releases/tag/latest
# Place sciter.dll in project root or system PATH
```

### 4.7 Clone and Build

```powershell
git clone --recursive https://github.com/ray-berg/nocos-connect.git
cd nocos-connect

# Build with legacy Sciter UI
cargo build --release

# OR build with Flutter UI
python build.py --flutter

# Build portable version
python build.py --flutter --portable
```

### 4.8 Output Locations

| Build Type | Output |
|------------|--------|
| Sciter | `target\release\nocos-connect.exe` |
| Flutter | `flutter\build\windows\x64\runner\Release\` |

---

## 5. macOS Build

### 5.1 Install Prerequisites

```bash
# Install Xcode from App Store, then:
xcode-select --install

# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install cmake nasm yasm pkg-config
```

### 5.2 Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 5.3 Install vcpkg

```bash
cd /opt
sudo git clone https://github.com/microsoft/vcpkg
cd vcpkg
sudo ./bootstrap-vcpkg.sh -disableMetrics
export VCPKG_ROOT=/opt/vcpkg
```

### 5.4 Install vcpkg Dependencies

```bash
$VCPKG_ROOT/vcpkg install libvpx libyuv opus aom
```

### 5.5 Install Flutter

```bash
cd ~/development
curl -LO https://storage.googleapis.com/flutter_infra_release/releases/stable/macos/flutter_macos_3.19.0-stable.zip
unzip flutter_macos_3.19.0-stable.zip
export PATH="$HOME/development/flutter/bin:$PATH"
flutter doctor
```

### 5.6 Download Sciter Library

```bash
curl -L https://raw.githubusercontent.com/nicecpp/nicecpp-assets/releases/download/latest/libsciter.dylib -o /usr/local/lib/libsciter.dylib
```

### 5.7 Clone and Build

```bash
git clone --recursive https://github.com/ray-berg/nocos-connect.git
cd nocos-connect

# Build with Flutter UI
python3 build.py --flutter

# With ScreenCaptureKit (macOS 12.3+)
python3 build.py --flutter --screencapturekit
```

### 5.8 Output Locations

| Build Type | Output |
|------------|--------|
| Sciter | `target/release/nocos-connect` |
| Flutter | `flutter/build/macos/Build/Products/Release/nocos_connect.app` |

---

## 6. Android Build

### 6.1 Install Android SDK and NDK

1. Install Android Studio: https://developer.android.com/studio
2. Open SDK Manager and install:
   - Android SDK Platform 33
   - Android NDK (r22+)
   - Android SDK Build-Tools
   - CMake

### 6.2 Set Environment Variables

```bash
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
export PATH=$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools
```

### 6.3 Install Rust Android Targets

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
```

### 6.4 Install Flutter

```bash
# Follow platform-specific Flutter installation above
flutter config --android-sdk $ANDROID_HOME
```

### 6.5 Build

```bash
cd nocos-connect

# Build APK
cd flutter
flutter build apk

# Build with specific NDK script
./build_android.sh
```

### 6.6 Output Location

```
flutter/build/app/outputs/flutter-apk/app-release.apk
```

---

## 7. iOS Build

### 7.1 Prerequisites

- macOS with Xcode 12+
- Apple Developer Account (for device deployment)
- CocoaPods: `sudo gem install cocoapods`

### 7.2 Install Rust iOS Targets

```bash
rustup target add aarch64-apple-ios x86_64-apple-ios
```

### 7.3 Build

```bash
cd nocos-connect/flutter

# Install dependencies
flutter pub get
cd ios && pod install && cd ..

# Build
flutter build ios --release
```

### 7.4 Output Location

```
flutter/build/ios/ipa/
```

---

## 8. Docker Build

### 8.1 Using Dockerfile

```bash
# Build the Docker image
docker build -t nocos-connect-builder .

# Run build
docker run -v $(pwd):/home/user/rustdesk nocos-connect-builder
```

### 8.2 Dockerfile Contents

The Dockerfile installs:
- Debian Bullseye base
- Build tools (gcc, g++, cmake, ninja)
- System libraries (GTK3, X11, PulseAudio, PAM)
- vcpkg with required libraries
- Rust toolchain

---

## 9. Feature Flags

### 9.1 Cargo Features

| Feature | Description | Command |
|---------|-------------|---------|
| `flutter` | Enable Flutter UI | `cargo build --features flutter` |
| `hwcodec` | Hardware video encoding | `cargo build --features hwcodec` |
| `vram` | VRAM optimization (Windows) | `cargo build --features vram` |
| `unix-file-copy-paste` | Unix file clipboard | `cargo build --features unix-file-copy-paste` |
| `screencapturekit` | macOS ScreenCaptureKit | `cargo build --features screencapturekit` |
| `plugin_framework` | Plugin support | `cargo build --features plugin_framework` |

### 9.2 Build Script Options

```bash
python3 build.py [OPTIONS]

Options:
  --flutter                Build Flutter version
  --hwcodec                Enable hardware codec
  --vram                   Enable VRAM (Windows only)
  --portable               Build portable version (Windows)
  --unix-file-copy-paste   Unix file copy/paste
  --screencapturekit       macOS ScreenCaptureKit
  --skip-cargo             Skip Rust build (Linux only)
```

### 9.3 Hardware Codec Requirements

#### NVIDIA (nvcodec)
- NVIDIA GPU with NVENC/NVDEC support
- NVIDIA drivers 470+
- Linux: Install `libnvidia-encode` and `libnvidia-decode`

#### AMD (AMF)
- AMD GPU with VCE/VCN support
- AMD drivers with AMF support

#### Intel (QSV)
- Intel CPU/GPU with Quick Sync Video
- Linux: Install `intel-media-va-driver`

---

## 10. Troubleshooting

### 10.1 Common Issues

#### vcpkg not found

```bash
# Ensure VCPKG_ROOT is set
export VCPKG_ROOT=/path/to/vcpkg
```

#### Sciter library not found

```bash
# Linux
export LD_LIBRARY_PATH=/path/to/libsciter-gtk.so:$LD_LIBRARY_PATH

# macOS
export DYLD_LIBRARY_PATH=/path/to/libsciter.dylib:$DYLD_LIBRARY_PATH

# Windows - place sciter.dll in same directory as executable
```

#### Flutter build fails

```bash
# Clean and rebuild
cd flutter
flutter clean
flutter pub get
flutter build linux  # or windows/macos/android/ios
```

#### Rust compilation errors

```bash
# Update Rust
rustup update stable

# Clean build
cargo clean
cargo build --release
```

### 10.2 Linux-Specific Issues

#### PulseAudio not found

```bash
sudo apt install libpulse-dev pulseaudio
```

#### X11/XCB errors

```bash
sudo apt install libxcb-randr0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

#### PAM authentication issues

```bash
sudo apt install libpam0g-dev
```

### 10.3 Windows-Specific Issues

#### MSVC linker errors

- Ensure Visual Studio C++ workload is installed
- Run from "Developer Command Prompt for VS"

#### DLL not found

- Copy required DLLs to output directory
- Or add to system PATH

### 10.4 macOS-Specific Issues

#### Code signing

```bash
# For development
codesign --force --deep --sign - target/release/nocos-connect

# For distribution, use proper signing certificate
```

#### Screen recording permission

- Grant permission in System Preferences → Security & Privacy → Privacy → Screen Recording

---

## Appendix A: Directory Structure

```
nocos-connect/
├── src/                    # Main Rust source
├── libs/                   # Rust libraries
│   ├── hbb_common/         # Common utilities (submodule)
│   ├── scrap/              # Screen capture
│   ├── enigo/              # Input simulation
│   ├── clipboard/          # Clipboard handling
│   └── virtual_display/    # Virtual display (Windows)
├── flutter/                # Flutter UI
│   ├── lib/                # Dart source
│   ├── android/            # Android config
│   ├── ios/                # iOS config
│   ├── linux/              # Linux config
│   ├── macos/              # macOS config
│   └── windows/            # Windows config
├── res/                    # Resources
│   └── vcpkg/              # vcpkg overlay ports
├── docs/                   # Documentation
└── build.py                # Build script
```

---

## Appendix B: Version Compatibility Matrix

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| Rust | 1.75 | Latest stable |
| Flutter | 3.1.0 | 3.19+ |
| vcpkg | 2023.04.15 | Latest |
| CMake | 3.20 | 3.30+ |
| Python | 3.8 | 3.10+ |

---

*Last updated: December 2024*
*Version: 1.4.4*
