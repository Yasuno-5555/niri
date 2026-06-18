use std::{fs, io};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::link::persistence::{ensure_dirs, local_node_id_path, trusted_nodes_path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustedNode {
    pub node: String,
    pub fingerprint: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustStore {
    pub nodes: Vec<TrustedNode>,
}

impl TrustStore {
    pub fn load() -> io::Result<Self> {
        let path = trusted_nodes_path();
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).map_err(io::Error::other),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err),
        }
    }

    pub fn persist(&self) -> io::Result<()> {
        ensure_dirs()?;
        let content = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(trusted_nodes_path(), content)
    }

    pub fn trust(&mut self, node: String, fingerprint: String) {
        if let Some(existing) = self.nodes.iter_mut().find(|it| it.node == node) {
            existing.fingerprint = fingerprint;
        } else {
            self.nodes.push(TrustedNode { node, fingerprint });
        }
    }

    pub fn forget(&mut self, node: &str) {
        self.nodes
            .retain(|it| it.node != node && it.fingerprint != node);
    }

    pub fn is_trusted(&self, node: &str, fingerprint: &str) -> bool {
        self.nodes
            .iter()
            .any(|it| (it.node == node || it.fingerprint == node) && it.fingerprint == fingerprint)
    }
}

pub fn load_or_create_local_node_id() -> io::Result<Uuid> {
    ensure_dirs()?;
    let path = local_node_id_path();

    match fs::read_to_string(&path) {
        Ok(content) => {
            let trimmed = content.trim();
            Uuid::parse_str(trimmed).map_err(io::Error::other)
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let node_id = Uuid::new_v4();
            fs::write(&path, node_id.to_string())?;
            Ok(node_id)
        }
        Err(err) => Err(err),
    }
}
