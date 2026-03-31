use chrono::{DateTime, Duration, Utc};
use rand::Rng;

use crate::crypto::{self, KeyPair};
use crate::types::{RoomConfig, RoomId};

/// Characters used to generate room IDs.
const ROOM_ID_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
/// Length of a room ID.
const ROOM_ID_LEN: usize = 4;

/// A secure E2EE room.
pub struct Room {
    /// The room's unique identifier (4-char alphanumeric).
    pub id: RoomId,
    /// When the room was created.
    pub created_at: DateTime<Utc>,
    /// When the room expires.
    pub expires_at: DateTime<Utc>,
    /// Room configuration.
    pub config: RoomConfig,
    /// The room's X25519 key pair for E2EE.
    pub keypair: KeyPair,
}

impl Room {
    /// Create a new secure room with the given configuration.
    pub fn create(config: RoomConfig) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(config.ttl_secs as i64);
        let id = generate_room_id();
        let keypair = crypto::generate_keypair();

        Self {
            id,
            created_at: now,
            expires_at,
            config,
            keypair,
        }
    }

    /// Check if this room has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Generate a random 4-character alphanumeric room ID.
pub fn generate_room_id() -> String {
    let mut rng = rand::thread_rng();
    (0..ROOM_ID_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..ROOM_ID_CHARS.len());
            ROOM_ID_CHARS[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_id_is_4_chars() {
        let id = generate_room_id();
        assert_eq!(id.len(), 4);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn room_create_with_defaults() {
        let room = Room::create(RoomConfig::default());
        assert_eq!(room.id.len(), 4);
        assert!(!room.is_expired());
        assert_eq!(room.config.ttl_secs, 1800);
        assert_eq!(room.config.max_participants, 2);
    }

    #[test]
    fn room_expiry() {
        let config = RoomConfig {
            ttl_secs: 0,
            max_participants: 2,
        };
        let room = Room::create(config);
        // With 0 TTL, room should be expired (or just at the boundary)
        // Sleep is not ideal in tests, so we check the expires_at is <= now
        assert!(room.expires_at <= Utc::now());
    }
}
