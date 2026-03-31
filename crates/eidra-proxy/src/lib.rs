pub mod ai_domains;
pub mod error;
pub mod handler;
pub mod server;
pub mod tls;

/// Channel for sending request events to the TUI dashboard.
pub type EventSender = tokio::sync::broadcast::Sender<ProxyEvent>;
pub type EventReceiver = tokio::sync::broadcast::Receiver<ProxyEvent>;

#[derive(Debug, Clone)]
pub struct ProxyEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub provider: String,
    pub findings_count: u32,
    pub categories: Vec<String>,
    pub data_size_bytes: u64,
    pub latency_ms: u64,
    pub status_code: u16,
}

pub fn create_event_channel() -> (EventSender, EventReceiver) {
    tokio::sync::broadcast::channel(256)
}
