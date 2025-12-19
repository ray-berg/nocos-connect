# NOCOS Connect

**NOCOS Connect** is a cross-platform remote desktop application forked from RustDesk, providing secure, self-hosted remote access capabilities with no configuration required.

## Table of Contents

- [Overview](#overview)
- [How The System Works](#how-the-system-works)
- [Architecture](#architecture)
- [Security Review](#security-review)
- [Build Instructions](#build-instructions)
- [Installation](#installation)
- [Configuration](#configuration)

---

## Overview

NOCOS Connect is a feature-rich remote desktop solution written primarily in Rust with a Flutter-based user interface. It supports Windows, macOS, Linux, Android, and iOS platforms.

### Key Features

- **Remote Desktop Control**: Full screen capture and real-time input simulation
- **File Transfer**: Secure bidirectional file transfer between connected devices
- **Clipboard Sharing**: Cross-platform clipboard synchronization (text and files)
- **Audio Streaming**: Real-time audio transmission from remote machine
- **Port Forwarding**: TCP tunneling for accessing remote network services
- **Terminal Access**: Remote shell/terminal access
- **Two-Factor Authentication**: TOTP-based 2FA for enhanced security
- **Self-Hosted Option**: Run your own relay/rendezvous server for full data control

---

## How The System Works

### Connection Architecture

NOCOS Connect uses a rendezvous server architecture for establishing peer-to-peer connections:

```
┌─────────────┐         ┌──────────────────┐         ┌─────────────┐
│   Client    │◄───────►│  Rendezvous      │◄───────►│   Server    │
│  (Viewer)   │         │     Server       │         │  (Host)     │
└─────────────┘         └──────────────────┘         └─────────────┘
       │                                                    │
       │              Direct P2P Connection                 │
       └────────────────────────────────────────────────────┘
```

1. **Registration**: Both client and server register with the rendezvous server using unique IDs
2. **Connection Request**: Client requests connection to server via rendezvous server
3. **NAT Traversal**: System attempts TCP hole punching for direct P2P connection
4. **Relay Fallback**: If direct connection fails, traffic routes through relay server
5. **Encrypted Session**: All communication uses end-to-end encryption

### Protocol Flow

1. **Handshake Phase**
   - Exchange of public keys using Curve25519
   - Symmetric key derivation for session encryption
   - Authentication challenge/response

2. **Session Phase**
   - Screen capture encoding (VP8/VP9/AV1/H264/H265)
   - Input event serialization and transmission
   - Multiplexed channels for video, audio, clipboard, file transfer

3. **Service Communication**
   - Protobuf-based message serialization
   - KCP stream for UDP reliability (optional)
   - TCP with TLS for secure transport

### Core Components

| Component | Location | Purpose |
|-----------|----------|---------|
| Rendezvous Mediator | `src/rendezvous_mediator.rs` | Server registration and connection brokering |
| Connection Handler | `src/server/connection.rs` | Manages active remote sessions |
| Client | `src/client.rs` | Initiates and manages outbound connections |
| Screen Capture | `libs/scrap/` | Platform-specific screen capture |
| Input Simulation | `libs/enigo/` | Cross-platform keyboard/mouse control |
| Clipboard | `libs/clipboard/` | Cross-platform clipboard operations |
| Configuration | `libs/hbb_common/src/config.rs` | All application settings |

### Screen Capture Methods

| Platform | Method |
|----------|--------|
| Windows | GDI, DirectX, DXGI Duplication |
| macOS | CoreGraphics, ScreenCaptureKit |
| Linux | X11 (XShm), Wayland (PipeWire) |

### Video Encoding

Supports multiple codecs with hardware acceleration:
- **VP8/VP9**: Default software codecs
- **AV1**: Modern efficient codec
- **H264/H265**: Hardware-accelerated (with `hwcodec` feature)

---

## Architecture

### Directory Structure

```
nocos-connect/
├── src/                    # Main Rust application
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Library exports
│   ├── client.rs          # Client connection logic
│   ├── server/            # Server services
│   │   ├── connection.rs  # Connection management
│   │   ├── video_service.rs
│   │   ├── audio_service.rs
│   │   ├── input_service.rs
│   │   └── clipboard_service.rs
│   ├── platform/          # OS-specific code
│   │   ├── windows.rs
│   │   ├── linux.rs
│   │   └── macos.rs
│   └── rendezvous_mediator.rs
│
├── flutter/               # Flutter UI
│   ├── lib/
│   │   ├── desktop/      # Desktop UI
│   │   ├── mobile/       # Mobile UI
│   │   ├── common/       # Shared components
│   │   └── models/       # Data models
│   └── pubspec.yaml
│
├── libs/                  # Core libraries
│   ├── hbb_common/       # Shared utilities, config, protocol
│   ├── scrap/            # Screen capture
│   ├── enigo/            # Input simulation
│   └── clipboard/        # Clipboard handling
│
└── res/                   # Resources and packaging
```

### Technology Stack

- **Backend**: Rust (async with Tokio)
- **Frontend**: Flutter/Dart
- **IPC**: Custom protobuf-based protocol
- **Encryption**: libsodium (NaCl box seal, secretbox)
- **Video Codecs**: libvpx, libyuv, opus, aom

---

## Security Review

### Security Assessment Summary

| Category | Status | Risk Level |
|----------|--------|------------|
| Authentication | Good | Low |
| 2FA Implementation | Adequate | Medium |
| Encryption (Nonce Issue) | Concerning | High |
| Key Management | Weak | High |
| Input Validation | Partial | Medium |
| Network Security | Good | Low |
| File Transfer | Good | Low |
| Memory Safety | Good (Rust) | Low |

### Critical Findings

#### 1. Static Zero Nonce in Encryption (HIGH RISK)

**Location**: `libs/hbb_common/src/lib.rs`

The cryptographic key exchange uses a hardcoded zero nonce:
```rust
let nonce = box_::Nonce([0u8; box_::NONCEBYTES]);  // Zero nonce
```

**Risk**: Using a static/zero nonce violates fundamental cryptographic security properties. This could potentially allow:
- Key recovery attacks if the same keypair encrypts multiple messages
- Replay attacks in certain scenarios

**Recommendation**: Generate a random nonce for each encryption operation and transmit it with the ciphertext.

#### 2. Weak Encryption Key Parameter (HIGH RISK)

**Location**: `src/auth_2fa.rs`

2FA secrets and Telegram bot tokens are encrypted using a hardcoded key "00":
```rust
encrypt_vec_or_original(secret, "00", 1024)
```

**Recommendation**: Use a proper key derivation function (KDF) like Argon2 or PBKDF2 with machine-specific entropy.

### Medium Priority Findings

#### 3. No Password Complexity Validation
- System accepts any password including empty strings
- **Recommendation**: Implement password strength requirements

#### 4. Unsafe Code Patterns
- Multiple uses of `unsafe { from_raw_parts() }` in plugin code
- **Recommendation**: Audit unsafe blocks and add safety documentation

#### 5. No Explicit Memory Clearing
- Sensitive data (passwords, keys) not explicitly zeroed after use
- **Recommendation**: Use the `zeroize` crate for sensitive data

### Strengths

1. **Rust Memory Safety**: Core codebase benefits from Rust's memory safety guarantees
2. **Multi-layer Authentication**: Supports passwords, temporary passwords, and 2FA
3. **Rate Limiting**: Built-in brute force protection with IP-based tracking
4. **Permission Model**: Granular permission controls (keyboard, clipboard, file access)
5. **Audit Logging**: File operations and authentication events are logged

---

## Build Instructions

### Prerequisites

1. **Rust toolchain** (1.75+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Flutter SDK** (3.1.0+)
   ```bash
   # Follow https://docs.flutter.dev/get-started/install
   ```

3. **vcpkg** (C++ package manager)
   ```bash
   git clone https://github.com/microsoft/vcpkg
   cd vcpkg && ./bootstrap-vcpkg.sh
   export VCPKG_ROOT=$HOME/vcpkg
   ```

4. **C++ dependencies**
   ```bash
   # Linux/macOS
   vcpkg install libvpx libyuv opus aom

   # Windows
   vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static aom:x64-windows-static
   ```

### Platform-Specific Dependencies

#### Ubuntu/Debian
```bash
sudo apt install -y zip g++ gcc git curl wget nasm yasm libgtk-3-dev clang \
    libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libasound2-dev libpulse-dev cmake make \
    libclang-dev ninja-build libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev libpam0g-dev
```

#### Fedora/CentOS
```bash
sudo yum -y install gcc-c++ git curl wget nasm yasm gcc gtk3-devel clang \
    libxcb-devel libxdo-devel libXfixes-devel pulseaudio-libs-devel cmake \
    alsa-lib-devel gstreamer1-devel gstreamer1-plugins-base-devel pam-devel
```

#### Arch Linux
```bash
sudo pacman -Syu --needed unzip git cmake gcc curl wget yasm nasm zip make \
    pkg-config clang gtk3 xdotool libxcb libxfixes alsa-lib pipewire
```

#### macOS
```bash
brew install nasm yasm cmake
```

### Building

#### Desktop (Flutter)
```bash
# Development build
python3 build.py --flutter

# Release build
python3 build.py --flutter --release

# With hardware codec support
python3 build.py --flutter --hwcodec
```

#### Desktop (Legacy Sciter - Deprecated)
```bash
# Download Sciter library first
wget https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so
mv libsciter-gtk.so target/debug/

# Build and run
cargo run
```

#### Android
```bash
cd flutter
flutter build apk
# or
flutter build appbundle
```

#### iOS
```bash
cd flutter
flutter build ios
```

### Docker Build

```bash
# Build the container
docker build -t "nocos-connect-builder" .

# Build the application
docker run --rm -it \
    -v $PWD:/home/user/nocos-connect \
    -v nocos-git-cache:/home/user/.cargo/git \
    -v nocos-registry-cache:/home/user/.cargo/registry \
    -e PUID="$(id -u)" -e PGID="$(id -g)" \
    nocos-connect-builder
```

---

## Installation

### Linux

#### From Binary
```bash
# Download the appropriate package for your distribution
# .deb for Debian/Ubuntu, .rpm for Fedora/RHEL, .AppImage for universal

# Debian/Ubuntu
sudo dpkg -i nocos-connect*.deb

# Fedora/RHEL
sudo rpm -i nocos-connect*.rpm

# AppImage (universal)
chmod +x nocos-connect*.AppImage
./nocos-connect*.AppImage
```

#### Systemd Service
```bash
sudo cp res/nocos-connect.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable nocos-connect
sudo systemctl start nocos-connect
```

### Windows

1. Run the MSI installer or extract the portable ZIP
2. The application will automatically register as a service

### macOS

1. Mount the DMG file
2. Drag NOCOS Connect to Applications
3. Grant necessary permissions (Screen Recording, Accessibility) in System Preferences

---

## Configuration

### Configuration Files

All configurations are stored in platform-specific locations:

| Platform | Location |
|----------|----------|
| Linux | `~/.config/nocos-connect/` |
| macOS | `~/Library/Application Support/nocos-connect/` |
| Windows | `%APPDATA%\nocos-connect\` |

### Key Configuration Options

Configuration is managed in `libs/hbb_common/src/config.rs` with four categories:

1. **Settings**: Global application preferences
2. **Local**: Machine-specific settings
3. **Display**: UI and display options
4. **Built-in**: Compile-time defaults

### Custom Server Configuration

To use a self-hosted server:

1. Set up [rustdesk-server](https://github.com/rustdesk/rustdesk-server)
2. Configure the client with your server address:
   - Via UI: Settings > ID/Relay Server
   - Via command line: `nocos-connect --config "rendezvous-server=your.server.com"`

### Feature Flags

Enable features at compile time:

| Flag | Description |
|------|-------------|
| `flutter` | Flutter UI support |
| `hwcodec` | Hardware video encoding/decoding |
| `vram` | VRAM optimization (Windows only) |
| `unix-file-copy-paste` | Unix file clipboard support |
| `screencapturekit` | macOS ScreenCaptureKit API |
| `plugin_framework` | Plugin system support |

---

## License

This project is licensed under the GNU Affero General Public License v3 (AGPL-3.0).

---

## Acknowledgments

NOCOS Connect is based on [RustDesk](https://github.com/rustdesk/rustdesk), an open-source remote desktop project by Purslane Ltd.
