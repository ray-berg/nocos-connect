# NOCOS Connect API Specification

This document describes all APIs exposed by NOCOS Connect, a cross-platform remote desktop application.

## Table of Contents

1. [Protocol Buffer Wire Protocol](#1-protocol-buffer-wire-protocol)
2. [HTTP/REST APIs](#2-httprest-apis)
3. [IPC API](#3-ipc-api)
4. [Flutter FFI API](#4-flutter-ffi-api)
5. [WebSocket API](#5-websocket-api)
6. [Network Flows](#6-network-flows)

---

## 1. Protocol Buffer Wire Protocol

The core peer-to-peer communication uses Protocol Buffers v3 over TCP, UDP, or WebSocket.

### 1.1 Peer Message Protocol

**Source:** `libs/hbb_common/protos/message.proto`

#### Main Message Container

```protobuf
message Message {
  oneof union {
    SignedId signed_id = 3;
    PublicKey public_key = 4;
    TestDelay test_delay = 5;
    VideoFrame video_frame = 6;
    LoginRequest login_request = 7;
    LoginResponse login_response = 8;
    Hash hash = 9;
    MouseEvent mouse_event = 10;
    AudioFrame audio_frame = 11;
    CursorData cursor_data = 12;
    CursorPosition cursor_position = 13;
    uint64 cursor_id = 14;
    KeyEvent key_event = 15;
    Clipboard clipboard = 16;
    FileAction file_action = 17;
    FileResponse file_response = 18;
    Misc misc = 19;
    Cliprdr cliprdr = 20;
    MessageBox message_box = 21;
    SwitchDisplay switch_display = 22;
    PointerDeviceEvent pointer_device_event = 23;
    PortForward port_forward = 24;
    Auth2FA auth_2fa = 25;
    MultiClipboards multi_clipboards = 27;
    VoiceCallRequest voice_call_request = 28;
    VoiceCallResponse voice_call_response = 29;
    PluginRequest plugin_request = 30;
    PluginResponse plugin_response = 31;
    AuthExtra auth_extra = 32;
  }
}
```

#### Authentication Messages

| Message | Fields | Description |
|---------|--------|-------------|
| `LoginRequest` | `username`, `password`, `my_id`, `my_name`, `session_id`, `hwid`, `options` | Initiates connection |
| `LoginResponse` | `peer_info`, `error` | Returns peer info or error |
| `Hash` | `salt`, `challenge` | Authentication challenge |
| `Auth2FA` | `code`, `hwid` | Two-factor authentication |

**LoginRequest Structure:**
```protobuf
message LoginRequest {
  string username = 1;
  bytes password = 2;
  string my_id = 4;
  string my_name = 5;
  OptionMessage option = 6;
  oneof union {
    FileTransfer file_transfer = 7;
    PortForward port_forward = 8;
    ViewCamera view_camera = 14;
    Terminal terminal = 15;
  }
  bytes session_id = 9;
  string version = 10;
  string peer_id = 11;
  bytes hwid = 12;
  OSLogin os_login = 13;
}
```

**LoginResponse Structure:**
```protobuf
message LoginResponse {
  oneof union {
    string error = 1;
    PeerInfo peer_info = 2;
  }
}

message PeerInfo {
  string username = 1;
  string hostname = 2;
  string platform = 3;
  repeated DisplayInfo displays = 4;
  SupportedEncoding encoding = 6;
  SupportedResolutions resolutions = 7;
  repeated WindowsSession windows_sessions = 8;
  CodecAbility codec_abilities = 9;
  bool keyboard = 10;
  bool clipboard = 11;
  bool audio = 12;
  bool file = 13;
  bool restart = 14;
  bool recording = 15;
  bool block_input = 16;
  Features features = 17;
}
```

#### Input Events

**Mouse Event:**
```protobuf
message MouseEvent {
  int32 mask = 1;      // Button state mask
  int32 x = 2;         // X coordinate
  int32 y = 3;         // Y coordinate
  int32 modifiers = 4; // Modifier keys
  int32 buttons = 5;   // Button flags
}
```

**Keyboard Event:**
```protobuf
message KeyEvent {
  bool down = 1;
  bool press = 2;
  oneof union {
    ControlKey control_key = 3;
    uint32 chr = 4;
    uint32 unicode = 5;
    string seq = 6;
  }
  int32 modifiers = 8;
  KeyboardMode keyboard_mode = 9;
}

enum ControlKey {
  Unknown = 0;
  Alt = 1;
  Backspace = 2;
  CapsLock = 3;
  Control = 4;
  Delete = 5;
  // ... (full list in proto file)
}
```

**Pointer/Touch Event:**
```protobuf
message PointerDeviceEvent {
  oneof union {
    TouchEvent touch_event = 1;
  }
  int32 modifiers = 2;
}

message TouchEvent {
  repeated TouchFrame frames = 1;
}
```

#### Display & Video

**Video Frame:**
```protobuf
message VideoFrame {
  oneof union {
    VP9s vp9s = 6;
    H264s h264s = 7;
    H265s h265s = 8;
    VP8s vp8s = 9;
    Av1s av1s = 10;
    RGB rgb = 11;
    YUV yuv = 12;
  }
  int64 timestamp = 2;
  Resolution resolution = 3;
  int32 display = 4;
}
```

**Display Information:**
```protobuf
message DisplayInfo {
  int32 x = 1;
  int32 y = 2;
  int32 width = 3;
  int32 height = 4;
  string name = 5;
  bool online = 6;
  bool cursor_embedded = 7;
  Resolution original_resolution = 8;
  float scale = 9;
}
```

#### File Transfer

**File Action:**
```protobuf
message FileAction {
  oneof union {
    ReadDir read_dir = 1;
    FileTransferSendRequest send = 2;
    FileTransferReceiveRequest receive = 3;
    FileDirCreate create = 4;
    FileRemoveDir remove_dir = 5;
    FileRemoveFile remove_file = 6;
    FileRename rename = 7;
    FileTransferCancel cancel = 8;
    ReadEmptyDirs read_empty_dirs = 12;
    FileTransferSendConfirmRequest send_confirm = 10;
  }
}
```

**File Response:**
```protobuf
message FileResponse {
  oneof union {
    FileDirectory dir = 1;
    FileTransferBlock block = 2;
    FileTransferError error = 3;
    FileTransferDone done = 4;
    FileTransferDigest digest = 5;
    FileTransferBlockV2 block_v2 = 6;
  }
}
```

#### Terminal

```protobuf
message TerminalAction {
  oneof union {
    TerminalOpen open_terminal = 1;
    TerminalData terminal_data = 2;
    TerminalResize resize_terminal = 3;
    TerminalClose close_terminal = 4;
  }
}

message TerminalOpen {
  uint32 terminal_id = 1;
  uint32 rows = 2;
  uint32 cols = 3;
}

message TerminalData {
  uint32 terminal_id = 1;
  bytes data = 2;
  bool compressed = 3;
}
```

#### Audio

```protobuf
message AudioFrame {
  bytes data = 1;
  int64 timestamp = 2;
}

message AudioFormat {
  uint32 sample_rate = 1;
  uint32 channels = 2;
}
```

#### Clipboard

```protobuf
message Clipboard {
  bool compress = 1;
  bytes content = 2;
  int32 width = 3;
  int32 height = 4;
  ClipboardFormat format = 5;
}

enum ClipboardFormat {
  Text = 0;
  Rtf = 1;
  Html = 2;
  File = 3;
  Special = 4;
}
```

### 1.2 Rendezvous Protocol

**Source:** `libs/hbb_common/protos/rendezvous.proto`

#### Registration

```protobuf
message RegisterPeer {
  string id = 1;
  int32 serial = 2;
}

message RegisterPeerResponse {
  bool request_pk = 2;
}

message RegisterPk {
  string id = 1;
  bytes uuid = 2;
  bytes pk = 3;
  string old_id = 4;
  bool no_register_device = 5;
}

message RegisterPkResponse {
  enum Result {
    OK = 0;
    UUID_MISMATCH = 2;
    ID_EXISTS = 3;
    TOO_FREQUENT = 4;
    INVALID_ID_FORMAT = 5;
    NOT_SUPPORT = 6;
    SERVER_ERROR = 7;
  }
  Result result = 1;
}
```

#### NAT Traversal

```protobuf
message PunchHoleRequest {
  string id = 1;
  NatType nat_type = 2;
  string licence_key = 3;
  ConnType conn_type = 4;
  string token = 5;
  int32 udp_port = 6;
  bool force_relay = 7;
}

message PunchHole {
  bytes socket_addr = 1;
  string relay_server = 2;
  NatType nat_type = 3;
  int32 udp_port = 4;
  int32 upnp_port = 5;
}

message PunchHoleResponse {
  bytes socket_addr = 1;
  bytes pk = 2;
  enum Failure {
    ID_NOT_EXIST = 0;
    OFFLINE = 2;
    LICENSE_MISMATCH = 3;
    LICENSE_OVERUSE = 4;
  }
  Failure failure = 3;
  string relay_server = 4;
  oneof union {
    NatType nat_type = 5;
    bool is_local = 6;
  }
  string other_failure = 7;
}
```

#### Connection Types

```protobuf
enum ConnType {
  DEFAULT_CONN = 0;
  FILE_TRANSFER = 1;
  PORT_FORWARD = 2;
  RDP = 3;
  VIEW_CAMERA = 4;
  TERMINAL = 5;
}
```

---

## 2. HTTP/REST APIs

### 2.1 Account API

**Base URL:** Configured via `api-server` option

#### Get Login Options

```
POST /api/login-options
Content-Type: application/json

Response:
{
  "oidc": [
    {
      "name": "provider_name",
      "op": "operation_code"
    }
  ]
}
```

#### Initiate OIDC Authentication

```
POST /api/oidc/auth
Content-Type: application/json

Request:
{
  "op": "provider_operation",
  "id": "device_id",
  "uuid": "device_uuid",
  "deviceInfo": {
    "os": "platform",
    "type": "device_type",
    "name": "device_name"
  }
}

Response:
{
  "code": "auth_code",
  "url": "https://provider.com/auth?..."
}
```

#### Query OIDC Status

```
GET /api/oidc/auth-query?code={code}&id={id}&uuid={uuid}

Response:
{
  "access_token": "jwt_token",
  "type": "token_type",
  "tfa_type": "totp|telegram|none",
  "secret": "2fa_secret (if new setup)",
  "user": {
    "name": "username",
    "email": "user@example.com",
    "status": 1,
    "info": {
      "email_verification": true,
      "email_alarm_notification": true,
      "login_device_whitelist": []
    }
  }
}
```

### 2.2 Sync API

#### Heartbeat

```
POST /api/heartbeat
Content-Type: application/json

Request:
{
  "id": "device_id",
  "uuid": "device_uuid",
  "ver": 1234567890,
  "conns": ["peer_id_1", "peer_id_2"],
  "modified_at": 1234567890
}

Response:
{
  "sysinfo": true,
  "disconnect": ["peer_id_to_disconnect"],
  "modified_at": 1234567890,
  "strategy": {
    "option_key": "option_value"
  }
}
```

#### Upload System Info

```
POST /api/sysinfo
Content-Type: application/json

Request:
{
  "id": "device_id",
  "uuid": "device_uuid",
  "username": "system_username",
  "hostname": "hostname",
  "platform": "windows|linux|macos",
  "version": "1.4.4"
}

Response: "SYSINFO_UPDATED" | "ID_NOT_FOUND" | error
```

### 2.3 Recording API

#### Upload Recording

```
POST /api/record?type={type}&file={filename}&offset={offset}&length={length}
Content-Type: application/octet-stream

Query Parameters:
- type: "new" | "part" | "tail" | "remove"
- file: filename for the recording
- offset: byte offset for resume
- length: total file length

Body: Binary file data (for new/part/tail)
```

---

## 3. IPC API

Inter-process communication via Unix domain sockets.

**Socket Path:** `Config::ipc_path(postfix)`

### Message Types

```rust
pub enum Data {
    // Configuration
    Config((String, Option<String>)),
    Options(Option<HashMap<String, String>>),
    SyncConfig(Option<(Config, Config2)>),

    // Session
    Login { ... },
    Authorize,
    Close,
    OnlineStatus(Option<(i64, bool)>),

    // System
    SystemInfo(Option<String>),
    NatType(Option<i32>),

    // Input
    Keyboard(KeyboardData),
    Mouse(MouseData),

    // File System
    FS(FsData),

    // Clipboard
    ClipboardFile(ClipboardFileData),
    ClipboardNonFile((Vec<ClipboardData>, i64)),

    // Permissions
    SwitchPermission {
        name: String,
        enabled: bool,
    },

    // Chat
    ChatMessage { text: String },

    // Voice Call
    VoiceCallIncoming,
    StartVoiceCall,
    VoiceCallResponse(bool),
    CloseVoiceCall(String),

    // Privacy
    PrivacyModeState((i32, PrivacyModeState, String)),

    // Plugin
    Plugin(PluginData),

    // Status
    Empty,
    Disconnected,
}
```

### Keyboard Data

```rust
pub enum KeyboardData {
    Sequence(String),
    KeyDown(RdevKey),
    KeyUp(RdevKey),
    KeyClick(RdevKey),
    GetKeyState(RdevKey),
}
```

### Mouse Data

```rust
pub enum MouseData {
    MoveTo(i32, i32),
    MoveRelative(i32, i32),
    Down(MouseButton),
    Up(MouseButton),
    Click(MouseButton),
    ScrollX(i32),
    ScrollY(i32),
    Refresh,
}
```

### File System Data

```rust
pub enum FsData {
    ReadDir { path: String, include_hidden: bool },
    RemoveDir { path: String, recursive: bool },
    RemoveFile { path: String, file_num: i32 },
    CreateDir { path: String },
    NewWrite { path: String, id: i32, file_num: i32, ... },
    WriteBlock { id: i32, file_num: i32, data: Vec<u8>, ... },
    WriteDone { id: i32, file_num: i32 },
    WriteError { id: i32, file_num: i32, err: String },
    Rename { path: String, new_name: String },
}
```

---

## 4. Flutter FFI API

Direct Rust bindings for Flutter UI.

**Source:** `src/flutter_ffi.rs`

### Session Management

```rust
// Create and manage sessions
fn session_add_sync(
    session_id: String,
    id: String,
    is_file_transfer: bool,
    is_port_forward: bool,
    is_rdp: bool,
    switch_uuid: String,
    force_relay: bool,
    password: String,
    is_shared_password: bool,
) -> SyncReturn<String>;

fn session_start(session_id: String, id: String) -> ResultType<()>;
fn session_close(session_id: String);
fn session_reconnect(session_id: String, force_relay: bool);
```

### Input Control

```rust
fn session_input_key(
    session_id: String,
    name: String,
    down: bool,
    press: bool,
    alt: bool,
    ctrl: bool,
    shift: bool,
    command: bool,
);

fn session_input_string(session_id: String, value: String);

fn session_send_mouse(
    session_id: String,
    msg: String,  // JSON: {"x": 100, "y": 200, "buttons": 1, ...}
);
```

### Display Control

```rust
fn session_refresh(session_id: String, display: i32);
fn session_switch_display(is_desktop: bool, session_id: String, value: i32);
fn session_set_size(session_id: String, display: usize, width: usize, height: usize);
fn session_set_image_quality(session_id: String, value: String);
fn session_set_custom_image_quality(session_id: String, value: i32);
fn session_set_custom_fps(session_id: String, fps: i32);
fn session_take_screenshot(session_id: String, display: i32) -> String;
```

### File Transfer

```rust
fn session_read_remote_dir(session_id: String, path: String, include_hidden: bool);
fn session_send_files(
    session_id: String,
    act_id: i32,
    path: String,
    to: String,
    file_num: i32,
    include_hidden: bool,
    is_remote: bool,
);
fn session_remove_file(
    session_id: String,
    act_id: i32,
    path: String,
    file_num: i32,
    is_remote: bool,
);
fn session_create_dir(session_id: String, act_id: i32, path: String, is_remote: bool);
fn session_rename_file(
    session_id: String,
    act_id: i32,
    path: String,
    new_name: String,
    is_remote: bool,
);
```

### Terminal

```rust
fn session_open_terminal(session_id: String, terminal_id: i32, rows: i32, cols: i32);
fn session_send_terminal_input(session_id: String, terminal_id: i32, data: String);
fn session_resize_terminal(session_id: String, terminal_id: i32, rows: i32, cols: i32);
fn session_close_terminal(session_id: String, terminal_id: i32);
```

### Configuration

```rust
fn main_get_option(key: String) -> String;
fn main_set_option(key: String, value: String);
fn main_get_options() -> String;  // JSON
fn main_set_options(json: String);

fn get_local_flutter_option(key: String) -> String;
fn set_local_flutter_option(key: String, value: String);
```

### Security & Permissions

```rust
fn session_toggle_option(session_id: String, value: String);
fn session_get_toggle_option(session_id: String, arg: String) -> bool;
fn session_toggle_privacy_mode(session_id: String, on: bool);
fn session_lock_screen(session_id: String);
fn session_ctrl_alt_del(session_id: String);
fn session_elevate_direct(session_id: String);
fn session_elevate_with_logon(session_id: String, username: String, password: String);
```

### Voice Call

```rust
fn session_request_voice_call(session_id: String);
fn session_close_voice_call(session_id: String);
```

---

## 5. WebSocket API

WebSocket transport for relay connections.

**Protocol:** `ws://` or `wss://`

### Configuration

```rust
const OPTION_ALLOW_WEBSOCKET: &str = "allow-websocket";
```

### Connection Flow

1. Check if WebSocket is allowed via configuration
2. Connect to relay server via WebSocket
3. Use same protobuf message format as TCP
4. TLS negotiation (if wss://)
5. SOCKS5 proxy support if configured

### Message Format

Same as TCP protocol - protobuf-encoded `Message` wrapped in WebSocket frames.

---

## 6. Network Flows

### 6.1 Connection Establishment

```
┌────────┐         ┌────────────────┐         ┌────────┐
│ Client │         │ Rendezvous Srv │         │ Server │
└───┬────┘         └───────┬────────┘         └───┬────┘
    │                      │                      │
    │ RegisterPeer ────────┼──────────────────────►
    │                      │                      │
    │ PunchHoleRequest ───►│                      │
    │                      │ PunchHole ──────────►│
    │                      │                      │
    │◄─── PunchHoleResponse│                      │
    │                      │                      │
    │◄─────────────── Direct P2P Connection ─────►│
    │              (or via Relay if NAT fails)    │
```

### 6.2 Authentication Flow

```
┌────────┐                              ┌────────┐
│ Client │                              │ Server │
└───┬────┘                              └───┬────┘
    │                                       │
    │ LoginRequest (my_id, my_name, etc.) ─►│
    │                                       │
    │◄─────────────── Hash (salt, challenge)│
    │                                       │
    │ LoginRequest (password hash) ────────►│
    │                                       │
    │◄─────── LoginResponse (PeerInfo/error)│
    │                                       │
    │ Auth2FA (code, hwid) ───────────────►│ (if 2FA enabled)
    │                                       │
    │◄─────────────────── Session Established
```

### 6.3 File Transfer Flow

```
┌────────┐                              ┌────────┐
│ Sender │                              │Receiver│
└───┬────┘                              └───┬────┘
    │                                       │
    │ FileAction::Send (path, files) ──────►│
    │                                       │
    │◄────── FileResponse::Digest (size, hash)
    │                                       │
    │ FileTransferBlock (data chunks) ─────►│
    │        ... (repeat for all chunks)    │
    │                                       │
    │◄────────────────── FileResponse::Done │
```

### 6.4 Heartbeat Sync Flow

```
┌────────┐                    ┌────────────┐
│ Client │                    │ API Server │
└───┬────┘                    └─────┬──────┘
    │                               │
    │ POST /api/heartbeat ─────────►│
    │ {id, uuid, ver, conns}        │
    │                               │
    │◄──────── {sysinfo, strategy}  │
    │                               │
    │ POST /api/sysinfo ───────────►│ (if sysinfo=true)
    │ {system details}              │
    │                               │
    │◄──────── "SYSINFO_UPDATED"    │
```

---

## 7. Configuration Options

### Connection Options

| Key | Type | Description |
|-----|------|-------------|
| `direct-server` | string | Direct connection server |
| `rendezvous-server` | string | Rendezvous server address |
| `relay-server` | string | Relay server address |
| `api-server` | string | HTTP API server |
| `allow-websocket` | bool | Enable WebSocket transport |

### Security Options

| Key | Type | Description |
|-----|------|-------------|
| `verification-method` | string | `use-temporary-password`, `use-permanent-password`, or both |
| `temporary-password-length` | string | `6`, `8`, or `10` |
| `approve-mode` | string | `password`, `click`, or `both` |
| `2fa` | string | Encrypted 2FA secret |

### Permission Options

| Key | Type | Description |
|-----|------|-------------|
| `enable-keyboard` | bool | Allow keyboard input |
| `enable-clipboard` | bool | Allow clipboard sync |
| `enable-file-transfer` | bool | Allow file transfer |
| `enable-audio` | bool | Allow audio streaming |
| `enable-remote-restart` | bool | Allow remote restart |
| `allow-remote-config-modification` | bool | Allow remote config changes |

---

## 8. Error Codes

### Login Errors

| Error | Description |
|-------|-------------|
| `Wrong Password` | Invalid password |
| `Too many login failures` | Rate limited |
| `2FA Required` | Two-factor authentication needed |
| `OS Login Required` | OS-level authentication required |
| `Desktop Session Not Ready` | Remote session not available |

### File Transfer Errors

| Error | Description |
|-------|-------------|
| `No permission` | Insufficient permissions |
| `Path not found` | File/directory doesn't exist |
| `Disk full` | No space on destination |
| `Cancelled` | Transfer cancelled by user |

### Connection Errors

| Error | Description |
|-------|-------------|
| `ID_NOT_EXIST` | Peer ID not registered |
| `OFFLINE` | Peer is offline |
| `LICENSE_MISMATCH` | License key mismatch |
| `LICENSE_OVERUSE` | License limit exceeded |

---

*Last updated: December 2024*
*Version: 1.4.4*
