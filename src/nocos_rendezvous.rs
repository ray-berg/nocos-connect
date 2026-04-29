//! NOCOS Connect client-side rendezvous (Phases 3.2a-6a, 6b, 6c).
//!
//! Replaces stock RustDesk's UDP-protobuf rendezvous with NOCOS's
//! HTTP+WebSocket control plane. See the design doc in the parent
//! nocos repo: `tasks/3.2-alt-nocos-native-control-plane.md`.
//!
//! # Slice status
//!
//! - **3.2a-6a (shipped)**: heartbeat loop. POSTs
//!   `/connect/peer/heartbeat` every 30 s.
//! - **3.2a-6b (shipped)**: spawns a rendezvous WebSocket task
//!   per pending session surfaced by heartbeat.
//! - **3.2a-6c (this slice)**: NAT-punch attempt.
//!     - Binds a UDP socket and enumerates local interface IPs
//!       as candidates (via `default_net`).
//!     - Sends `{"type":"candidates","addrs":[...]}` over the WS.
//!     - When the peer sends its own candidates, fires small UDP
//!       probes at each and listens on the bound socket for a
//!       reply. On reply → sends `{"type":"punched"}`.
//!     - On ~5-s punch timeout → sends `{"type":"fallback"}`, POSTs
//!       `/connect/session/{sid}/relay-fallback` to mint the
//!       `aud=hbbr` JWT, and logs the response. The actual hbbr
//!       TCP handoff (feeding the JWT into RustDesk's existing
//!       relay state machine) lands in **3.2a-6d**.
//!
//! # Activation
//!
//! Gated by the `NOCOS_CONNECT_BASE_URL` environment variable. When
//! set (e.g. `https://dashboard.nocos.ai`), [`start`] drives the
//! NOCOS-native loop; when absent, the caller in
//! `rendezvous_mediator::start` falls through to stock RustDesk
//! rendezvous unchanged. Safe to ship in a fork that also serves
//! stock environments.
//!
//! # Identity
//!
//! - `NOCOS_AGENT_ID` — UUID the endpoint was issued by NOCOS. Required.
//! - ed25519 public key — read from the existing `Config::get_key_pair()`
//!   so we don't introduce a parallel key store. Sent on the first
//!   heartbeat only (per the Pydantic schema in
//!   `api/connect/schemas.py`).
//! - Rendezvous WS token — minted per session by NOCOS's
//!   `/connect/peer/heartbeat` and returned in `pending_rendezvous`.
//!   The fork presents it via `?token=<jwt>` query param.

use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use hbb_common::{
    base64,
    config::Config,
    futures_util::{SinkExt, StreamExt},
    log,
    tokio::{
        self,
        net::UdpSocket,
        time::{sleep, timeout, Instant},
    },
    ResultType,
};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

use crate::hbbs_http::create_http_client_async_with_url;

/// Interval between heartbeats. Matches NOCOS's `PENDING_TTL / 4` so a
/// pending rendezvous queued at worst case will still be picked up
/// inside its 120-s freshness window.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
/// Upper bound on a single heartbeat request. NOCOS's handler is
/// cheap (one SELECT + one UPSERT); a 10-s timeout is generous
/// without pinning the client to a slow/dead control plane.
const HEARTBEAT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Overall deadline for a rendezvous WS once opened. Matches NOCOS's
/// `_IDLE_TIMEOUT_SEC` on the server side — sides agree so neither
/// hangs waiting for the other.
const RENDEZVOUS_WS_TIMEOUT: Duration = Duration::from_secs(60);
/// UDP punch attempt budget. Starts once we've sent our candidates;
/// ends when we either see a reply on the bound socket or give up
/// and ask for relay fallback. 5 s is generous — successful punches
/// typically complete in < 500 ms.
const PUNCH_TIMEOUT: Duration = Duration::from_secs(5);
/// Upper bound on a single /relay-fallback HTTP request.
const RELAY_FALLBACK_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Small probe bytes we send during punch attempts. The peer doesn't
/// need to parse it — reception alone proves the NAT mapping opened.
/// We use an ASCII tag for log-readability on the other side's socket
/// if it happens to have packet capture running.
const PUNCH_PROBE: &[u8] = b"nocos-connect-punch";

const ENV_BASE_URL: &str = "NOCOS_CONNECT_BASE_URL";
const ENV_AGENT_ID: &str = "NOCOS_AGENT_ID";

// ---------------------------------------------------------------------------
// HTTP wire shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct HeartbeatRequest<'a> {
    agent_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_key_b64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_addr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reported_nat_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_version: Option<&'a str>,
}

#[derive(Debug, Deserialize, Clone)]
struct PendingRendezvous {
    session_id: String,
    operator_hint: String,
    #[allow(dead_code)] // surfaced in logs; not yet acted on
    expires_at: String,
    /// Endpoint-audience JWT minted per-heartbeat by NOCOS so the
    /// token is guaranteed fresh when we connect the WS.
    token: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HeartbeatResponse {
    acknowledged_at: String,
    #[serde(default)]
    pending_rendezvous: Vec<PendingRendezvous>,
}

// ---------------------------------------------------------------------------
// WS wire shapes — match api/connect/schemas.py::RendezvousMessage
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct RendezvousMessage {
    #[serde(rename = "type")]
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    addrs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

// ---------------------------------------------------------------------------
// /connect/session/{sid}/relay-fallback shape
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct RelayFallbackRequest<'a> {
    reason: &'a str,
}

#[derive(Debug, Deserialize)]
struct RelayFallbackResponse {
    relay_host: String,
    relay_pin: String,
    relay_token: String,
}

// ---------------------------------------------------------------------------
// Config snapshot
// ---------------------------------------------------------------------------

/// Env-derived config. Captured once at `start`; env changes mid-run
/// need a restart (matches how `rendezvous_mediator` treats its config).
#[derive(Debug)]
struct Config4 {
    base_url: String,
    agent_id: String,
}

impl Config4 {
    fn from_env() -> Option<Self> {
        let base_url = std::env::var(ENV_BASE_URL).ok()?;
        if base_url.is_empty() {
            return None;
        }
        let agent_id = std::env::var(ENV_AGENT_ID).ok().filter(|s| !s.is_empty())?;
        Some(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            agent_id,
        })
    }

    fn heartbeat_url(&self) -> String {
        format!("{}/connect/peer/heartbeat", self.base_url)
    }

    /// Build the wss://… URL for a rendezvous WS. Handles https/http
    /// symmetrically — if the control plane is plain http for local
    /// dev, WS is plain ws, same origin.
    fn relay_fallback_url(&self, session_id: &str) -> String {
        format!(
            "{}/connect/session/{}/relay-fallback",
            self.base_url, session_id
        )
    }

    fn rendezvous_ws_url(&self, session_id: &str, token: &str) -> String {
        let base = self.base_url.as_str();
        let ws_base = if let Some(rest) = base.strip_prefix("https://") {
            format!("wss://{}", rest)
        } else if let Some(rest) = base.strip_prefix("http://") {
            format!("ws://{}", rest)
        } else {
            // Non-URL-shaped base — caller's problem; pass through.
            base.to_string()
        };
        format!(
            "{}/connect/rendezvous/{}?token={}",
            ws_base, session_id, token
        )
    }
}

// ---------------------------------------------------------------------------
// Active-session dedup
// ---------------------------------------------------------------------------

/// Tracks session_ids for which a rendezvous WS task is already
/// running. Prevents the heartbeat loop from spawning duplicates when
/// a slow WS is still live and NOCOS keeps surfacing the pending
/// session. Entries are removed when the WS task exits.
#[derive(Default)]
struct ActiveSessions {
    ids: Mutex<HashSet<String>>,
}

impl ActiveSessions {
    fn try_claim(&self, session_id: &str) -> bool {
        let mut g = self.ids.lock().expect("ActiveSessions mutex poisoned");
        g.insert(session_id.to_string())
    }

    fn release(&self, session_id: &str) {
        let mut g = self.ids.lock().expect("ActiveSessions mutex poisoned");
        g.remove(session_id);
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn is_enabled() -> bool {
    Config4::from_env().is_some()
}

pub async fn start(_host: String) -> ResultType<()> {
    let cfg = match Config4::from_env() {
        Some(c) => Arc::new(c),
        None => {
            log::info!(
                "NOCOS Connect: {} unset, leaving stock rendezvous active",
                ENV_BASE_URL
            );
            return Ok(());
        }
    };
    log::info!(
        "NOCOS Connect: rendezvous mediator starting; base={}, agent_id={}",
        cfg.base_url,
        cfg.agent_id,
    );

    let client = create_http_client_async_with_url(&cfg.heartbeat_url()).await;
    let active = Arc::new(ActiveSessions::default());
    let mut first = true;
    let client_version = env!("CARGO_PKG_VERSION");

    loop {
        let body = HeartbeatRequest {
            agent_id: &cfg.agent_id,
            public_key_b64: if first { public_key_b64() } else { None },
            local_addr: None, // populated in 3.2a-6c after NAT probing
            reported_nat_type: None,
            client_version: Some(client_version),
        };

        match heartbeat_once(&client, &cfg, &body).await {
            Ok(resp) => {
                first = false;
                for pending in resp.pending_rendezvous {
                    if active.try_claim(&pending.session_id) {
                        let cfg = Arc::clone(&cfg);
                        let active = Arc::clone(&active);
                        tokio::spawn(async move {
                            run_rendezvous_session(cfg, active, pending).await;
                        });
                    } else {
                        log::debug!(
                            "NOCOS Connect: session {} already has an active WS task",
                            pending.session_id
                        );
                    }
                }
            }
            Err(e) => {
                log::warn!("NOCOS Connect heartbeat failed: {}", e);
            }
        }

        sleep(HEARTBEAT_INTERVAL).await;
    }
}

// ---------------------------------------------------------------------------
// Heartbeat
// ---------------------------------------------------------------------------

async fn heartbeat_once(
    client: &reqwest::Client,
    cfg: &Config4,
    body: &HeartbeatRequest<'_>,
) -> ResultType<HeartbeatResponse> {
    let resp = client
        .post(cfg.heartbeat_url())
        .timeout(HEARTBEAT_REQUEST_TIMEOUT)
        .json(body)
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let snippet = resp.text().await.unwrap_or_default();
        let snippet = &snippet[..snippet.len().min(256)];
        hbb_common::anyhow::bail!("heartbeat returned {}: {}", status, snippet);
    }
    Ok(resp.json::<HeartbeatResponse>().await?)
}

// ---------------------------------------------------------------------------
// Rendezvous WS — 3.2a-6b
// ---------------------------------------------------------------------------

/// One-shot task: open the rendezvous WebSocket for a single session,
/// drive the signaling exchange, exit on a terminator or timeout.
///
/// The task is fire-and-forget from `start()`'s perspective. Errors
/// log at `warn` and the session_id is released so the next heartbeat
/// can reclaim it.
async fn run_rendezvous_session(
    cfg: Arc<Config4>,
    active: Arc<ActiveSessions>,
    pending: PendingRendezvous,
) {
    let url = cfg.rendezvous_ws_url(&pending.session_id, &pending.token);
    log::info!(
        "NOCOS Connect: rendezvous WS starting for session={} operator={}",
        pending.session_id,
        pending.operator_hint,
    );

    match timeout(
        RENDEZVOUS_WS_TIMEOUT,
        drive_rendezvous(&cfg, &pending, &url),
    )
    .await
    {
        Ok(Ok(())) => {
            log::info!(
                "NOCOS Connect: rendezvous WS closed cleanly for session={}",
                pending.session_id,
            );
        }
        Ok(Err(e)) => {
            log::warn!(
                "NOCOS Connect: rendezvous WS error for session={}: {}",
                pending.session_id,
                e,
            );
        }
        Err(_) => {
            log::info!(
                "NOCOS Connect: rendezvous WS timed out for session={} ({}s)",
                pending.session_id,
                RENDEZVOUS_WS_TIMEOUT.as_secs(),
            );
        }
    }
    active.release(&pending.session_id);
}

/// Inner body of the rendezvous task. Binds a UDP punch socket,
/// connects the WS, exchanges candidates, tries to punch, and on
/// failure calls `/relay-fallback`.
///
/// Status update contract mirrors the Python side in
/// `api/connect/rendezvous.py`:
/// - sending `{"type":"punched"}` → server transitions session to
///   `p2p`
/// - sending `{"type":"fallback"}` is advisory; the real state
///   change to `relayed` happens on the `/relay-fallback` POST
async fn drive_rendezvous(
    cfg: &Config4,
    pending: &PendingRendezvous,
    url: &str,
) -> ResultType<()> {
    // Bind first — if this fails (firewall), no point opening the WS.
    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    let local_port = sock.local_addr()?.port();
    let my_candidates = local_udp_candidates(local_port);
    log::info!(
        "NOCOS Connect: UDP socket bound on port {}, {} local candidates",
        local_port,
        my_candidates.len(),
    );

    let (ws, _response) = connect_async(url).await?;
    let (mut ws_sink, mut ws_stream) = ws.split();

    // Announce our candidates + a `ready` so peers or servers that
    // care about the "peer is live" bit both see it.
    send_json(
        &mut ws_sink,
        &RendezvousMessage {
            kind: "candidates".to_string(),
            addrs: Some(my_candidates.clone()),
            value: None,
        },
    )
    .await?;
    send_json(
        &mut ws_sink,
        &RendezvousMessage {
            kind: "ready".to_string(),
            addrs: None,
            value: None,
        },
    )
    .await?;

    // Main loop: race three things per iteration —
    //   (a) an incoming WS message
    //   (b) an incoming UDP datagram on the punch socket
    //   (c) the punch deadline
    let deadline = Instant::now() + PUNCH_TIMEOUT;
    let mut udp_buf = [0u8; 256];
    let mut peer_candidates_fired = false;

    loop {
        tokio::select! {
            // (a) WS inbound
            msg = ws_stream.next() => {
                let msg = match msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => hbb_common::anyhow::bail!("ws recv error: {}", e),
                    None => {
                        log::debug!("NOCOS Connect: rendezvous WS closed by server");
                        return Ok(());
                    }
                };
                match msg {
                    WsMessage::Text(text) => {
                        if let Some(terminator) = handle_ws_text(
                            &text,
                            &sock,
                            &mut peer_candidates_fired,
                        )
                        .await
                        {
                            // handle_ws_text returns Some(_) when the
                            // peer reported a terminal state
                            // (`punched` or `fallback`) — honor it.
                            log::info!(
                                "NOCOS Connect: peer signalled {}; session={}",
                                terminator,
                                pending.session_id,
                            );
                            // If peer said they punched, we implicitly did too.
                            // If peer said fallback, fall through to our own
                            // fallback path — we still need the hbbr JWT.
                            if terminator == "fallback" {
                                return do_relay_fallback(cfg, pending, &mut ws_sink).await;
                            }
                            return Ok(());
                        }
                    }
                    WsMessage::Close(frame) => {
                        log::info!(
                            "NOCOS Connect: rendezvous WS close frame: {:?}",
                            frame
                        );
                        return Ok(());
                    }
                    WsMessage::Binary(_)
                    | WsMessage::Ping(_)
                    | WsMessage::Pong(_)
                    | WsMessage::Frame(_) => {
                        // tungstenite handles ping/pong; binary frames
                        // aren't part of our protocol.
                    }
                }
            }

            // (b) UDP recv — a reply from our probes = punch succeeded
            recv = sock.recv_from(&mut udp_buf) => {
                match recv {
                    Ok((n, peer_addr)) => {
                        log::info!(
                            "NOCOS Connect: UDP punch reply received from {} ({} bytes); session={}",
                            peer_addr,
                            n,
                            pending.session_id,
                        );
                        send_json(
                            &mut ws_sink,
                            &RendezvousMessage {
                                kind: "punched".to_string(),
                                addrs: None,
                                value: None,
                            },
                        )
                        .await?;
                        return Ok(());
                    }
                    Err(e) => {
                        // Unusual — bound socket failed to recv.
                        // Abandon the punch, let deadline fire.
                        log::warn!("NOCOS Connect: UDP recv_from error: {}", e);
                    }
                }
            }

            // (c) deadline — time to fall back to hbbr
            _ = tokio::time::sleep_until(deadline) => {
                log::info!(
                    "NOCOS Connect: punch timeout for session={}; falling back to hbbr",
                    pending.session_id,
                );
                send_json(
                    &mut ws_sink,
                    &RendezvousMessage {
                        kind: "fallback".to_string(),
                        addrs: None,
                        value: None,
                    },
                )
                .await?;
                return do_relay_fallback(cfg, pending, &mut ws_sink).await;
            }
        }
    }
}

/// Handle a single WS Text frame. Returns `Some(terminator_kind)` if
/// the message was a terminator (`punched` / `fallback`), else `None`.
/// Side-effect: on receiving peer `candidates`, fires UDP probes at
/// each parseable addr — the other side's `recv_from` noticing that
/// traffic is what eventually sets off the punch detection on their
/// end.
async fn handle_ws_text(
    text: &str,
    sock: &UdpSocket,
    peer_candidates_fired: &mut bool,
) -> Option<&'static str> {
    log::debug!("NOCOS Connect: rendezvous WS recv: {}", text);
    let rm = match serde_json::from_str::<RendezvousMessage>(text) {
        Ok(rm) => rm,
        Err(_) => return None,
    };
    match rm.kind.as_str() {
        "candidates" => {
            if *peer_candidates_fired {
                // Already fired probes for a prior candidates message
                // this session. Avoid duplicate bursts if the peer
                // re-announces.
                return None;
            }
            *peer_candidates_fired = true;
            if let Some(addrs) = rm.addrs {
                let count = fire_punch_probes(sock, &addrs).await;
                log::info!(
                    "NOCOS Connect: fired {} UDP probes at peer candidates",
                    count,
                );
            }
            None
        }
        "punched" => Some("punched"),
        "fallback" => Some("fallback"),
        _ => None, // nat_type, ready, error — log only
    }
}

/// Fire one small UDP probe at each parseable peer candidate. Returns
/// the number of successful sends (failures are logged at debug and
/// ignored — unreachable candidates are the whole reason for this
/// coordination step).
async fn fire_punch_probes(sock: &UdpSocket, addrs: &[String]) -> usize {
    let mut sent = 0;
    for candidate in addrs {
        match candidate.parse::<SocketAddr>() {
            Ok(addr) => match sock.send_to(PUNCH_PROBE, addr).await {
                Ok(_) => sent += 1,
                Err(e) => {
                    log::debug!(
                        "NOCOS Connect: UDP probe to {} failed: {}",
                        addr,
                        e
                    );
                }
            },
            Err(e) => {
                log::debug!(
                    "NOCOS Connect: unparseable peer candidate {:?}: {}",
                    candidate,
                    e
                );
            }
        }
    }
    sent
}

/// Enumerate IPv4 addrs from local interfaces, skipping loopback.
/// Each string is `ip:port` — the bound UDP port is the same for
/// every candidate since we're advertising one socket.
///
/// IPv6 is omitted for this slice. RustDesk's punch code biases
/// toward v4 today, and NOCOS Connect's own protocol is v4-first
/// on the data plane. v6 can be added once we've seen whether
/// customer networks demand it.
fn local_udp_candidates(port: u16) -> Vec<String> {
    let mut out = Vec::new();
    for iface in default_net::get_interfaces() {
        for v4 in iface.ipv4 {
            let addr = v4.addr;
            if addr.is_loopback() || addr.is_unspecified() {
                continue;
            }
            out.push(format!("{}:{}", addr, port));
        }
    }
    out
}

/// Mint the JWT + return relay coordinates to the operator. Logs the
/// full response so a dev inspecting logs on lab-ws1 can see the
/// control plane completed successfully. 3.2a-6d will take this JWT
/// and feed it into RustDesk's existing hbbr relay state machine.
async fn do_relay_fallback<S>(
    cfg: &Config4,
    pending: &PendingRendezvous,
    ws_sink: &mut S,
) -> ResultType<()>
where
    S: SinkExt<WsMessage, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    // Best effort: drop the WS once we have the fallback response
    // so NOCOS's rendezvous timer can release our slot.
    let url = cfg.relay_fallback_url(&pending.session_id);
    let client = create_http_client_async_with_url(&url).await;
    let resp = client
        .post(&url)
        .bearer_auth(&pending.token)
        .timeout(RELAY_FALLBACK_REQUEST_TIMEOUT)
        .json(&RelayFallbackRequest {
            reason: "punch_timeout",
        })
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let snippet = resp.text().await.unwrap_or_default();
        let snippet = &snippet[..snippet.len().min(256)];
        hbb_common::anyhow::bail!("relay-fallback returned {}: {}", status, snippet);
    }
    let payload: RelayFallbackResponse = resp.json().await?;
    log::info!(
        "NOCOS Connect: relay-fallback OK — relay_host={} relay_pin={} relay_token={}…",
        payload.relay_host,
        payload.relay_pin,
        &payload.relay_token[..payload.relay_token.len().min(24)],
    );
    // 3.2a-6d integrates payload.relay_token with RustDesk's existing
    // hbbr relay flow (via rendezvous_mediator::create_relay or
    // equivalent). Not wired here so 6c keeps its scope small and
    // end-to-end-verifiable: if you see this log line, the control
    // plane delivered everything we need.

    // Ignore close errors — we're exiting anyway.
    let _ = ws_sink.close().await;
    Ok(())
}

/// Convenience: serialize + send a JSON message over the split sink.
async fn send_json<S>(sink: &mut S, msg: &RendezvousMessage) -> ResultType<()>
where
    S: SinkExt<WsMessage, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    let json = serde_json::to_string(msg)?;
    sink.send(WsMessage::Text(json.into())).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn public_key_b64() -> Option<String> {
    let kp = Config::get_key_pair();
    let pk: &[u8] = kp.1.as_ref();
    if pk.is_empty() {
        return None;
    }
    // hbb_common::base64 re-exports the `base64` crate (0.22). The
    // 1-arg `encode(bytes)` uses the STANDARD alphabet with padding,
    // matching Python's default `base64.b64encode` so the round-trip
    // with NOCOS's Pydantic `public_key_b64` validator holds.
    Some(base64::encode(pk))
}
