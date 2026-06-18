use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::link::protocol::{GlobalWorkspace, NodeId, SessionId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistedSession {
    pub version: u16,
    pub session_id: SessionId,
    pub generation: u64,
    pub participants: Vec<NodeId>,
    pub trusted_fingerprints: Vec<String>,
    pub global_layout: GlobalWorkspace,
    pub per_node_owned_tile_order: Vec<(NodeId, Vec<uuid::Uuid>)>,
    pub timestamp_millis: u64,
}

pub fn state_root() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("", "", "niri-link") {
        project_dirs
            .state_dir()
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".local/state/niri-link"))
    } else {
        PathBuf::from(".local/state/niri-link")
    }
}

pub fn trusted_nodes_path() -> PathBuf {
    state_root().join("trusted-nodes.json")
}

pub fn local_node_id_path() -> PathBuf {
    state_root().join("local-node-id")
}

pub fn sessions_dir() -> PathBuf {
    state_root().join("sessions")
}

pub fn session_file(session_id: SessionId) -> PathBuf {
    sessions_dir().join(format!("{session_id}.json"))
}

pub fn ensure_dirs() -> io::Result<()> {
    fs::create_dir_all(sessions_dir())
}

pub fn load_session(session_id: SessionId) -> io::Result<PersistedSession> {
    let content = fs::read_to_string(session_file(session_id))?;
    serde_json::from_str(&content).map_err(io::Error::other)
}

pub fn load_sessions() -> io::Result<Vec<PersistedSession>> {
    ensure_dirs()?;
    let mut sessions = Vec::new();
    for entry in fs::read_dir(sessions_dir())? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|it| it.to_str()) != Some("json") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(session) = serde_json::from_str::<PersistedSession>(&content) {
                sessions.push(session);
            }
        }
    }
    sessions.sort_by_key(|it| it.timestamp_millis);
    Ok(sessions)
}

pub fn persist_session(session: &PersistedSession) -> io::Result<PathBuf> {
    ensure_dirs()?;
    let path = session_file(session.session_id);
    let content = serde_json::to_string_pretty(session).map_err(io::Error::other)?;
    fs::write(&path, content)?;
    Ok(path)
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn peer_match_score(expected: &[NodeId], actual: &[NodeId]) -> usize {
    actual.iter().filter(|id| expected.contains(id)).count()
}

pub fn restore_candidate<P: AsRef<Path>>(
    sessions: &[PersistedSession],
    participants: &[NodeId],
    path_selector: P,
) -> Option<PersistedSession> {
    let _ = path_selector.as_ref();
    sessions
        .iter()
        .max_by_key(|session| peer_match_score(&session.participants, participants))
        .cloned()
}
