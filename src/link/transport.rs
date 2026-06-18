use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tracing::{debug, error, info, warn};

use crate::link::protocol::Envelope;

/// Low-level transport kind preference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportKind {
    Quic,
    TcpTls,
}

/// A message pending delivery.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMessage {
    pub addr: String,
    pub envelope: Envelope,
}

/// Incoming decoded message from a peer.
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub peer_addr: String,
    pub envelope: Envelope,
}

/// Shared queue for incoming messages, used to bridge the network thread to the compositor loop.
type IncomingQueue = Arc<Mutex<VecDeque<IncomingMessage>>>;

/// State for the link transport layer.
#[derive(Debug, Default)]
pub struct TransportState {
    outgoing: VecDeque<PendingMessage>,
    incoming: IncomingQueue,
    listener_running: bool,
}

impl TransportState {
    /// Enqueue a message to be sent to a remote peer.
    pub fn enqueue(&mut self, addr: String, envelope: Envelope) {
        self.outgoing.push_back(PendingMessage { addr, envelope });
    }

    /// Drain all outgoing messages and attempt delivery.
    /// Each message is sent in its own short-lived TCP connection (fire-and-forget style).
    /// A production implementation would maintain persistent connections per peer.
    pub fn drain(&mut self) -> Vec<PendingMessage> {
        let messages: Vec<_> = self.outgoing.drain(..).collect();
        let incoming = Arc::clone(&self.incoming);
        for msg in &messages {
            let addr = msg.addr.clone();
            let envelope = msg.envelope.clone();
            let incoming = Arc::clone(&incoming);
            thread::spawn(move || {
                send_and_maybe_recv(addr, envelope, incoming);
            });
        }
        messages
    }

    /// Drain all received messages.
    pub fn drain_incoming(&mut self) -> Vec<IncomingMessage> {
        let mut q = self.incoming.lock().unwrap();
        q.drain(..).collect()
    }

    /// Start a TCP listener on `bind_addr` in a background thread.
    /// Incoming envelopes are pushed into the shared incoming queue.
    pub fn start_listener(&mut self, bind_addr: &str) -> io::Result<u16> {
        if self.listener_running {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "listener already running",
            ));
        }
        let listener = TcpListener::bind(bind_addr)?;
        let port = listener.local_addr()?.port();
        let incoming = Arc::clone(&self.incoming);
        self.listener_running = true;
        info!("niri-link: TCP listener started on port {}", port);
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let incoming = Arc::clone(&incoming);
                        thread::spawn(move || {
                            handle_incoming_stream(stream, incoming);
                        });
                    }
                    Err(e) => {
                        warn!("niri-link: TCP accept error: {e}");
                        break;
                    }
                }
            }
        });
        Ok(port)
    }
}

/// Send an Envelope to `addr` over TCP, then optionally receive a response.
fn send_and_maybe_recv(addr: String, envelope: Envelope, incoming: IncomingQueue) {
    let encoded = match bincode::serialize(&envelope) {
        Ok(b) => b,
        Err(e) => {
            error!("niri-link: failed to serialize envelope: {e}");
            return;
        }
    };

    let mut stream = match TcpStream::connect_timeout(
        &addr
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:0".parse().unwrap()),
        Duration::from_millis(2000),
    ) {
        Ok(s) => s,
        Err(e) => {
            warn!("niri-link: TCP connect to {addr} failed: {e}");
            return;
        }
    };

    // Write length-prefixed envelope.
    let len = (encoded.len() as u32).to_be_bytes();
    if let Err(e) = stream
        .write_all(&len)
        .and_then(|_| stream.write_all(&encoded))
    {
        warn!("niri-link: TCP write to {addr} failed: {e}");
        return;
    }

    debug!("niri-link: sent {} bytes to {addr}", encoded.len());

    // Try to read one response envelope (optional).
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    if let Some(msg) = read_envelope(&mut stream, &addr) {
        let mut q = incoming.lock().unwrap();
        q.push_back(IncomingMessage {
            peer_addr: addr,
            envelope: msg,
        });
    }
}

/// Handle an incoming TCP connection, reading length-prefixed envelopes.
fn handle_incoming_stream(mut stream: TcpStream, incoming: IncomingQueue) {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    debug!("niri-link: incoming connection from {peer}");
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));

    loop {
        match read_envelope(&mut stream, &peer) {
            Some(envelope) => {
                let mut q = incoming.lock().unwrap();
                q.push_back(IncomingMessage {
                    peer_addr: peer.clone(),
                    envelope,
                });
            }
            None => break,
        }
    }
    debug!("niri-link: connection from {peer} closed");
}

/// Read a single length-prefixed `Envelope` from the stream.
/// Returns `None` on EOF or error.
fn read_envelope(stream: &mut TcpStream, peer: &str) -> Option<Envelope> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).ok()?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > 64 * 1024 * 1024 {
        warn!("niri-link: oversized message ({len} bytes) from {peer}, dropping");
        return None;
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).ok()?;
    match bincode::deserialize::<Envelope>(&buf) {
        Ok(envelope) => Some(envelope),
        Err(e) => {
            warn!("niri-link: failed to deserialize envelope from {peer}: {e}");
            None
        }
    }
}
