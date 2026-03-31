use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A 4-character alphanumeric room identifier.
pub type RoomId = String;

/// Configuration for a secure room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    /// Time-to-live in seconds. Default: 1800 (30 minutes).
    pub ttl_secs: u64,
    /// Maximum number of participants. Default: 2.
    pub max_participants: usize,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            ttl_secs: 1800,
            max_participants: 2,
        }
    }
}

/// An encrypted message exchanged through a secure room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier.
    pub id: String,
    /// Sender identifier (device hash or room participant id).
    pub sender: String,
    /// Encrypted payload bytes.
    pub payload: Vec<u8>,
    /// Timestamp when the message was created.
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// Create a new message with the given sender and encrypted payload.
    pub fn new(sender: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender: sender.into(),
            payload,
            timestamp: Utc::now(),
        }
    }
}
