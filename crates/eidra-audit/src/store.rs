use crate::error::AuditError;
use crate::event::AuditEvent;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct AuditStore {
    conn: Mutex<Connection>,
}

impl AuditStore {
    pub fn open(path: &Path) -> Result<Self, AuditError> {
        let conn = Connection::open(path)?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_tables()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, AuditError> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> Result<(), AuditError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AuditError::Lock(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                action TEXT NOT NULL,
                destination TEXT NOT NULL,
                findings_count INTEGER NOT NULL DEFAULT 0,
                findings_summary TEXT,
                data_size_bytes INTEGER NOT NULL DEFAULT 0,
                metadata TEXT
            );",
        )?;
        Ok(())
    }

    pub fn log_event(&self, event: &AuditEvent) -> Result<(), AuditError> {
        let metadata_json = serde_json::to_string(&event.metadata)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| AuditError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT INTO audit_events (id, timestamp, event_type, action, destination, findings_count, findings_summary, data_size_bytes, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                event.id.to_string(),
                event.timestamp.to_rfc3339(),
                event.event_type.to_string(),
                event.action.to_string(),
                event.destination,
                event.findings_count,
                event.findings_summary,
                event.data_size_bytes,
                metadata_json,
            ],
        )?;
        Ok(())
    }

    pub fn query_recent(&self, limit: usize) -> Result<Vec<AuditEvent>, AuditError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AuditError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, action, destination, findings_count, findings_summary, data_size_bytes, metadata
             FROM audit_events ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            let id_str: String = row.get(0)?;
            let ts_str: String = row.get(1)?;
            let event_type_str: String = row.get(2)?;
            let action_str: String = row.get(3)?;
            let destination: String = row.get(4)?;
            let findings_count: u32 = row.get(5)?;
            let findings_summary: String = row.get(6)?;
            let data_size_bytes: u64 = row.get(7)?;
            let metadata_str: String = row.get(8)?;

            Ok((
                id_str,
                ts_str,
                event_type_str,
                action_str,
                destination,
                findings_count,
                findings_summary,
                data_size_bytes,
                metadata_str,
            ))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let (
                id_str,
                ts_str,
                event_type_str,
                action_str,
                destination,
                findings_count,
                findings_summary,
                data_size_bytes,
                metadata_str,
            ) = row?;

            let id = uuid::Uuid::parse_str(&id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
            let timestamp = chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let event_type = match event_type_str.as_str() {
                "ai_request" => crate::event::EventType::AiRequest,
                "scan_finding" => crate::event::EventType::ScanFinding,
                "policy_action" => crate::event::EventType::PolicyAction,
                "agent_message" => crate::event::EventType::AgentMessage,
                "identity_verification" => crate::event::EventType::IdentityVerification,
                other => crate::event::EventType::Custom(other.to_string()),
            };
            let action = match action_str.as_str() {
                "allow" => crate::event::ActionTaken::Allow,
                "mask" => crate::event::ActionTaken::Mask,
                "block" => crate::event::ActionTaken::Block,
                "escalate" => crate::event::ActionTaken::Escalate,
                other => crate::event::ActionTaken::Custom(other.to_string()),
            };
            let metadata: std::collections::HashMap<String, String> =
                serde_json::from_str(&metadata_str).unwrap_or_default();

            events.push(AuditEvent {
                id,
                timestamp,
                event_type,
                action,
                destination,
                findings_count,
                findings_summary,
                data_size_bytes,
                metadata,
            });
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{ActionTaken, AuditEvent, EventType};

    #[test]
    fn test_log_and_query() {
        let store = AuditStore::open_in_memory().unwrap();
        let event = AuditEvent::new(
            EventType::AiRequest,
            ActionTaken::Allow,
            "api.openai.com",
            0,
            "[]",
            1024,
        );
        store.log_event(&event).unwrap();

        let results = store.query_recent(10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].destination, "api.openai.com");
        assert_eq!(results[0].findings_count, 0);
    }

    #[test]
    fn test_log_with_findings() {
        let store = AuditStore::open_in_memory().unwrap();
        let event = AuditEvent::new(
            EventType::ScanFinding,
            ActionTaken::Mask,
            "api.anthropic.com",
            2,
            r#"["api_key","pii"]"#,
            2048,
        );
        store.log_event(&event).unwrap();

        let results = store.query_recent(10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].findings_count, 2);
    }
}
