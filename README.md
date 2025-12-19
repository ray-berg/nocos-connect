<p align="center">
  <img src="res/logo-header.svg" alt="NOCOS Connect - Your remote desktop"><br>
  <a href="#build-instructions">Build</a> •
  <a href="#docker-build">Docker</a> •
  <a href="#file-structure">Structure</a> •
  <a href="#screenshots">Screenshots</a>
</p>

# NOCOS Connect

A cross-platform remote desktop application providing secure, self-hosted remote access with no configuration required. Forked from [RustDesk](https://github.com/rustdesk/rustdesk).

> [!Caution]
> **Misuse Disclaimer:** <br>
> The developers do not condone or support any unethical or illegal use of this software. Misuse, such as unauthorized access, control or invasion of privacy, is strictly against our guidelines. The authors are not responsible for any misuse of the application.

## Features

- **Cross-Platform**: Windows, macOS, Linux, Android, iOS
- **Secure**: End-to-end encryption with no configuration required
- **Self-Hosted**: Full control of your data with optional self-hosted servers
- **File Transfer**: Secure bidirectional file transfer
- **Clipboard Sharing**: Cross-platform clipboard synchronization
- **Audio Streaming**: Real-time audio transmission
- **Two-Factor Authentication**: TOTP-based 2FA support

See [NOCOS_CONNECT.md](NOCOS_CONNECT.md) for comprehensive documentation including security review, architecture details, and advanced configuration.

## Quick Start

### Dependencies

- Rust development environment (1.75+)
- [vcpkg](https://github.com/microsoft/vcpkg) with `VCPKG_ROOT` environment variable set
- C++ dependencies: `libvpx`, `libyuv`, `opus`, `aom`

```bash
# Install vcpkg dependencies
vcpkg install libvpx libyuv opus aom  # Linux/macOS
vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static aom:x64-windows-static  # Windows
```

## Build Instructions

### Ubuntu/Debian

```bash
# Install system dependencies
sudo apt install -y zip g++ gcc git curl wget nasm yasm libgtk-3-dev clang \
    libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libasound2-dev libpulse-dev cmake make \
    libclang-dev ninja-build libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev libpam0g-dev

# Install vcpkg
git clone https://github.com/microsoft/vcpkg
cd vcpkg && git checkout 2023.04.15 && ./bootstrap-vcpkg.sh
export VCPKG_ROOT=$HOME/vcpkg
vcpkg/vcpkg install libvpx libyuv opus aom

# Build
python3 build.py --flutter
```

### Fedora/CentOS

```bash
sudo yum -y install gcc-c++ git curl wget nasm yasm gcc gtk3-devel clang \
    libxcb-devel libxdo-devel libXfixes-devel pulseaudio-libs-devel cmake \
    alsa-lib-devel gstreamer1-devel gstreamer1-plugins-base-devel pam-devel
```

### Arch Linux

```bash
sudo pacman -Syu --needed unzip git cmake gcc curl wget yasm nasm zip make \
    pkg-config clang gtk3 xdotool libxcb libxfixes alsa-lib pipewire
```

### Build Commands

```bash
# Flutter desktop build
python3 build.py --flutter

# Release build
python3 build.py --flutter --release

# With hardware codec support
python3 build.py --hwcodec
```

## Docker Build

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

# Run the built application
target/debug/nocos-connect    # Development
target/release/nocos-connect  # Release
```

## File Structure

- **[libs/hbb_common](libs/hbb_common)**: Video codec, config, tcp/udp wrapper, protobuf, file transfer utilities
- **[libs/scrap](libs/scrap)**: Screen capture
- **[libs/enigo](libs/enigo)**: Platform-specific keyboard/mouse control
- **[libs/clipboard](libs/clipboard)**: File copy and paste implementation
- **[src/server](src/server)**: Audio/clipboard/input/video services, and network connections
- **[src/client.rs](src/client.rs)**: Start a peer connection
- **[src/rendezvous_mediator.rs](src/rendezvous_mediator.rs)**: Rendezvous server communication
- **[src/platform](src/platform)**: Platform-specific code
- **[flutter](flutter)**: Flutter code for desktop and mobile

## Screenshots

![Connection Manager](https://github.com/rustdesk/rustdesk/assets/28412477/db82d4e7-c4bc-4823-8e6f-6af7eadf7651)

![Connected to a Windows PC](https://github.com/rustdesk/rustdesk/assets/28412477/9baa91e9-3362-4d06-aa1a-7518edcbd7ea)

![File Transfer](https://github.com/rustdesk/rustdesk/assets/28412477/39511ad3-aa9a-4f8c-8947-1cce286a46ad)

![TCP Tunneling](https://github.com/rustdesk/rustdesk/assets/28412477/78e8708f-e87e-4570-8373-1360033ea6c5)

## License

GNU Affero General Public License v3 (AGPL-3.0)

## Acknowledgments

Based on [RustDesk](https://github.com/rustdesk/rustdesk) by Purslane Ltd.
