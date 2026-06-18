use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tracing::{debug, info, warn};

pub const MDNS_SERVICE_NAME: &str = "_niri-link._tcp.local.";
pub const MDNS_SERVICE_TYPE: &str = "_niri-link._tcp.local.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredPeer {
    pub hostname: String,
    pub addr: String,
    pub fingerprint: String,
}

#[derive(Debug, Default)]
pub struct DiscoveryState {
    peers: Vec<DiscoveredPeer>,
}

impl DiscoveryState {
    pub fn peers(&self) -> &[DiscoveredPeer] {
        &self.peers
    }

    pub fn upsert(&mut self, peer: DiscoveredPeer) {
        if let Some(existing) = self.peers.iter_mut().find(|it| it.addr == peer.addr) {
            *existing = peer;
        } else {
            self.peers.push(peer);
        }
    }

    pub fn remove_by_addr(&mut self, addr: &str) {
        self.peers.retain(|p| p.addr != addr);
    }
}

/// Handle for the mDNS discovery background task.
pub struct DiscoveryHandle {
    daemon: ServiceDaemon,
}

impl std::fmt::Debug for DiscoveryHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscoveryHandle").finish_non_exhaustive()
    }
}

impl DiscoveryHandle {
    /// Stop the mDNS daemon.
    pub fn stop(self) {
        if let Err(e) = self.daemon.shutdown() {
            warn!("niri-link: mDNS shutdown error: {e}");
        }
    }
}

/// Start advertising this node via mDNS and browse for peers.
/// Returns a handle and a shared queue of discovered peers.
///
/// `local_node_id` – UUID string for this node.
/// `fingerprint` – short fingerprint string.
/// `port` – TCP port this node is listening on.
/// `discovered` – shared queue into which discovered peers are pushed.
pub fn start_discovery(
    local_node_id: String,
    fingerprint: &str,
    port: u16,
    discovered: Arc<Mutex<Vec<DiscoveredPeer>>>,
) -> Result<DiscoveryHandle, mdns_sd::Error> {
    let daemon = ServiceDaemon::new()?;

    // Register our own service so peers can find us.
    let hostname = gethostname();
    let service_name = format!("niri-link-{local_node_id}");
    let mut props = std::collections::HashMap::new();
    props.insert("fingerprint".to_string(), fingerprint.to_string());
    props.insert("node_id".to_string(), local_node_id.to_string());

    let service_info = ServiceInfo::new(
        MDNS_SERVICE_TYPE,
        &service_name,
        &format!("{hostname}.local."),
        "",
        port,
        props,
    )?;
    daemon.register(service_info)?;
    info!("niri-link: mDNS registered as {service_name} on port {port}");

    // Browse for peers.
    let receiver = daemon.browse(MDNS_SERVICE_TYPE)?;
    thread::spawn(move || {
        loop {
            match receiver.recv_timeout(Duration::from_secs(5)) {
                Ok(ServiceEvent::ServiceResolved(info)) => {
                    let node_id = info
                        .get_properties()
                        .get("node_id")
                        .map(|p| p.val_str())
                        .unwrap_or("")
                        .to_string();
                    // Skip our own advertisements.
                    if node_id == local_node_id {
                        continue;
                    }
                    let fp = info
                        .get_properties()
                        .get("fingerprint")
                        .map(|p| p.val_str())
                        .unwrap_or("")
                        .to_string();
                    for addr in info.get_addresses() {
                        let peer_addr = format!("{}:{}", addr, info.get_port());
                        debug!("niri-link: discovered peer {node_id} at {peer_addr}");
                        let peer = DiscoveredPeer {
                            hostname: info.get_hostname().to_string(),
                            addr: peer_addr,
                            fingerprint: fp.clone(),
                        };
                        let mut q = discovered.lock().unwrap();
                        // Upsert by hostname+port.
                        if let Some(existing) = q.iter_mut().find(|p| p.hostname == peer.hostname) {
                            *existing = peer;
                        } else {
                            q.push(peer);
                        }
                    }
                }
                Ok(ServiceEvent::ServiceRemoved(_, full_name)) => {
                    debug!("niri-link: peer left: {full_name}");
                }
                Ok(_) => {}
                Err(_) => {
                    // Timeout or closed — exit the loop.
                    break;
                }
            }
        }
        info!("niri-link: mDNS browse loop ended");
    });

    Ok(DiscoveryHandle { daemon })
}

fn gethostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string()))
        .unwrap_or_else(|_| "localhost".to_string())
}
