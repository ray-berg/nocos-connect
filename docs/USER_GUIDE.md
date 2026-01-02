# NOCOS Connect: Complete Installation and Operation Guide

## What is NOCOS Connect?

NOCOS Connect is a cross-platform remote desktop application built in Rust with a Flutter-based UI. Forked from RustDesk, it provides secure, self-hosted remote access capabilities with end-to-end encryption and no mandatory configuration.

### Key Features

| Feature | Description |
|---------|-------------|
| **Remote Desktop Control** | Full screen capture and real-time input simulation |
| **Cross-Platform** | Windows, macOS, Linux, Android, iOS |
| **File Transfer** | Secure bidirectional file transfer |
| **Clipboard Sharing** | Cross-platform text and file clipboard sync |
| **Audio Streaming** | Real-time audio from remote machine |
| **Port Forwarding** | TCP tunneling for remote services |
| **Terminal Access** | Remote shell/terminal |
| **Two-Factor Auth** | TOTP-based 2FA support |
| **Self-Hosted Option** | Run your own relay/rendezvous server |

---

## System Requirements

### Minimum Hardware

| Component | Requirement |
|-----------|-------------|
| **RAM** | 8GB minimum, 16GB recommended |
| **Disk Space** | 20GB+ (including build dependencies) |
| **Network** | Stable internet connection |

### Software Requirements

| Component | Version |
|-----------|---------|
| **Rust** | 1.75 or later |
| **Flutter** | 3.1.0+ (for Flutter UI) |
| **Python** | 3.8+ (build scripts) |
| **Git** | 2.0+ |
| **CMake** | 3.20+ |

### Platform-Specific Requirements

| Platform | Requirements |
|----------|-------------|
| **Linux** | Ubuntu 20.04+, Debian 11+, Fedora, Arch |
| **Windows** | Windows 10/11, Visual Studio 2019+ |
| **macOS** | macOS 10.14+, Xcode 12+ |
| **Android** | SDK 21+, NDK r22+ |
| **iOS** | iOS 12+, Xcode 12+ |

---

## Installation

### Method 1: Pre-built Binaries (Recommended)

#### Linux

```bash
# Debian/Ubuntu (.deb)
sudo dpkg -i nocos-connect*.deb

# Fedora/RHEL (.rpm)
sudo rpm -i nocos-connect*.rpm

# Universal (AppImage)
chmod +x nocos-connect*.AppImage
./nocos-connect*.AppImage
```

#### Windows

Run the MSI installer or extract the portable ZIP archive.

#### macOS

1. Mount the DMG file
2. Drag NOCOS Connect to Applications
3. Grant permissions in System Preferences:
   - Screen Recording
   - Accessibility

---

### Method 2: Build from Source

#### Step 1: Install System Dependencies

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install -y \
    g++ gcc git curl wget nasm yasm \
    libgtk-3-dev clang libxcb-randr0-dev \
    libxdo-dev libxfixes-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libasound2-dev libpam0g-dev \
    libpulse-dev libssl-dev libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev cmake \
    ninja-build pkg-config unzip zip
```

**Fedora/RHEL:**

```bash
sudo dnf install -y \
    gcc-c++ gcc git curl wget nasm yasm \
    gtk3-devel clang libxcb-devel libxdo-devel \
    libXfixes-devel alsa-lib-devel pam-devel \
    pulseaudio-libs-devel openssl-devel \
    gstreamer1-devel gstreamer1-plugins-base-devel \
    cmake ninja-build pkgconfig unzip zip
```

**Arch Linux:**

```bash
sudo pacman -Syu --needed \
    unzip git cmake gcc curl wget yasm nasm \
    zip make pkg-config clang gtk3 xdotool \
    libxcb libxfixes alsa-lib pipewire
```

**macOS:**

```bash
xcode-select --install
brew install cmake nasm yasm pkg-config
```

#### Step 2: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
```

#### Step 3: Install vcpkg and C++ Dependencies

```bash
# Clone vcpkg
cd /opt
sudo git clone https://github.com/microsoft/vcpkg
cd vcpkg
sudo ./bootstrap-vcpkg.sh -disableMetrics

# Set environment variable
export VCPKG_ROOT=/opt/vcpkg
echo 'export VCPKG_ROOT=/opt/vcpkg' >> ~/.bashrc

# Install dependencies
# Linux/macOS:
$VCPKG_ROOT/vcpkg install libvpx libyuv opus aom

# Windows (PowerShell):
vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static aom:x64-windows-static
```

#### Step 4: Install Flutter (for Flutter UI)

```bash
# Linux
cd /opt
sudo wget https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.19.0-stable.tar.xz
sudo tar xf flutter_linux_3.19.0-stable.tar.xz
export PATH=/opt/flutter/bin:$PATH
echo 'export PATH=/opt/flutter/bin:$PATH' >> ~/.bashrc
flutter doctor
```

#### Step 5: Clone and Build

```bash
git clone --recursive https://github.com/ray-berg/nocos-connect.git
cd nocos-connect

# Development build with Flutter UI
python3 build.py --flutter

# Release build
python3 build.py --flutter --release

# With hardware codec support
python3 build.py --flutter --hwcodec
```

#### Build Output Locations

| Build Type | Platform | Location |
|------------|----------|----------|
| Flutter | Linux | `flutter/build/linux/x64/release/bundle/` |
| Flutter | Windows | `flutter/build/windows/x64/runner/Release/` |
| Flutter | macOS | `flutter/build/macos/Build/Products/Release/` |
| Cargo | All | `target/release/nocos-connect` |

---

### Method 3: Docker Build

```bash
# Build the Docker image
docker build -t nocos-connect-builder .

# Build the application
docker run --rm -it \
    -v $PWD:/home/user/nocos-connect \
    -v nocos-git-cache:/home/user/.cargo/git \
    -v nocos-registry-cache:/home/user/.cargo/registry \
    -e PUID="$(id -u)" -e PGID="$(id -g)" \
    nocos-connect-builder

# Run the built application
target/release/nocos-connect
```

---

## Operating NOCOS Connect

### Starting the Application

```bash
# Run directly
./nocos-connect

# Or from Flutter build
./flutter/build/linux/x64/release/bundle/nocos-connect
```

### Running as a Service (Linux)

```bash
sudo cp res/nocos-connect.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable nocos-connect
sudo systemctl start nocos-connect
```

### Command-Line Options

```bash
# Start as server (accepting connections)
nocos-connect -s

# Connect to a remote machine
nocos-connect -c <remote-id> -k <encryption-key>

# Port forwarding
nocos-connect -p <remote-id>:<local-port>:<remote-port>

# Configure custom server
nocos-connect --config "rendezvous-server=your.server.com"
```

### Basic Usage Workflow

1. **Host Machine (Server)**
   - Launch NOCOS Connect
   - Note your unique ID displayed in the main window
   - Set a password via Settings or share the temporary password

2. **Client Machine (Viewer)**
   - Launch NOCOS Connect
   - Enter the remote machine's ID
   - Enter the password when prompted
   - Complete 2FA verification if enabled

3. **Active Session**
   - Control the remote desktop via mouse/keyboard
   - Use toolbar for file transfer, clipboard, audio toggle
   - Access port forwarding and terminal features from menu

### Permission Controls

During a session, the host can control:

- Keyboard/mouse access
- Clipboard sharing
- File transfer permissions
- Audio streaming
- Screen visibility

---

## Configuration

### Configuration File Locations

| Platform | Path |
|----------|------|
| Linux | `~/.config/nocos-connect/` |
| macOS | `~/Library/Application Support/nocos-connect/` |
| Windows | `%APPDATA%\nocos-connect\` |

### Self-Hosted Server Setup

To use your own server instead of public relays:

1. Deploy [rustdesk-server](https://github.com/rustdesk/rustdesk-server)
2. Configure clients:
   - **Via UI**: Settings → ID/Relay Server → Enter server address
   - **Via CLI**: `nocos-connect --config "rendezvous-server=server.example.com"`

### Feature Flags (Compile-Time)

| Flag | Description | Build Command |
|------|-------------|---------------|
| `flutter` | Flutter UI | `cargo build --features flutter` |
| `hwcodec` | Hardware video encoding | `python3 build.py --hwcodec` |
| `vram` | VRAM optimization | Windows only |
| `unix-file-copy-paste` | File clipboard on Unix | Linux/macOS |
| `screencapturekit` | macOS screen capture API | `python3 build.py --screencapturekit` |
| `plugin_framework` | Plugin system | Desktop only |

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| `VCPKG_ROOT not found` | `export VCPKG_ROOT=/opt/vcpkg` |
| Sciter library not found | Set `LD_LIBRARY_PATH` to library location |
| Flutter build fails | Run `flutter clean && flutter pub get` |
| Connection refused | Check firewall rules for ports 21115-21119 |
| No screen capture | Grant Screen Recording permission (macOS) |

### Linux-Specific

```bash
# PulseAudio issues
sudo apt install libpulse-dev pulseaudio

# X11/XCB errors
sudo apt install libxcb-randr0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# PAM authentication
sudo apt install libpam0g-dev
```

### macOS-Specific

```bash
# Code signing for development
codesign --force --deep --sign - target/release/nocos-connect
```

Grant permissions in **System Preferences → Security & Privacy → Privacy**:

- Screen Recording
- Accessibility

---

## Security Considerations

NOCOS Connect uses:

- **Curve25519** key exchange
- **NaCl secretbox** (XSalsa20 + Poly1305) symmetric encryption
- **Ed25519** signatures
- Rate limiting for brute force protection
- Granular permission controls

Previous critical security issues (static nonce in encryption, weak 2FA key derivation) have been resolved. See `NOCOS_CONNECT.md` for the full security review.

---

## License

GNU Affero General Public License v3 (AGPL-3.0)

---

This guide covers the essential aspects of installing and operating NOCOS Connect. For additional platform-specific details, refer to `BUILD_GUIDE.md` and `API_SPECIFICATION.md` in the docs directory.
