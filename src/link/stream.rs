use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::link::protocol::{StreamState, TileId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FramePacket {
    pub tile_id: TileId,
    pub frame_id: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamBuffer {
    pub tile_id: TileId,
    pub state: StreamState,
    pub queue: VecDeque<FramePacket>,
    pub max_queued_frames: usize,
}

impl StreamBuffer {
    pub fn new(tile_id: TileId) -> Self {
        Self {
            tile_id,
            state: StreamState::Pending,
            queue: VecDeque::new(),
            max_queued_frames: 2,
        }
    }

    pub fn push(&mut self, packet: FramePacket) {
        if self.queue.len() >= self.max_queued_frames {
            self.queue.pop_front();
        }
        self.state = StreamState::Streaming;
        self.queue.push_back(packet);
    }

    pub fn latest(&self) -> Option<&FramePacket> {
        self.queue.back()
    }

    pub fn mark_stale(&mut self) {
        self.state = StreamState::Stale;
    }
}
