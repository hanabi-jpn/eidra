use crate::event::{RequestEntry, Statistics};

const HISTORY_LIMIT: usize = 72;

pub struct TuiApp {
    pub entries: Vec<RequestEntry>,
    pub stats: Statistics,
    pub selected_index: usize,
    pub should_quit: bool,
    pub scroll_offset: usize,
    pub uptime_secs: u64,
    pub frame_tick: u64,
    pub payload_history: Vec<u64>,
    pub findings_history: Vec<u64>,
    pub risk_history: Vec<u64>,
    pub latency_history: Vec<u64>,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            stats: Statistics::default(),
            selected_index: 0,
            should_quit: false,
            scroll_offset: 0,
            uptime_secs: 0,
            frame_tick: 0,
            payload_history: Vec::new(),
            findings_history: Vec::new(),
            risk_history: Vec::new(),
            latency_history: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: RequestEntry) {
        self.stats.record(&entry);
        self.push_histories(&entry);
        self.entries.push(entry);
        self.scroll_offset = 0;
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max_offset = self.entries.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + 1).min(max_offset);
    }

    pub fn tick(&mut self) {
        self.frame_tick = self.frame_tick.wrapping_add(1);
    }

    fn push_histories(&mut self, entry: &RequestEntry) {
        let payload = (entry.data_size_bytes / 1024).clamp(2, 160);
        let findings = (entry.findings_count as u64)
            .saturating_mul(18)
            .clamp(0, 100);
        let risk = match entry.action {
            crate::event::RequestAction::Allow => 14 + findings / 6,
            crate::event::RequestAction::Route => 58 + findings / 4,
            crate::event::RequestAction::Mask => 72 + findings / 5,
            crate::event::RequestAction::Block => 92,
            crate::event::RequestAction::Escalate => 100,
        }
        .clamp(0, 100);
        let latency = entry.latency_ms.min(2_000);

        Self::push_history(&mut self.payload_history, payload);
        Self::push_history(&mut self.findings_history, findings);
        Self::push_history(&mut self.risk_history, risk);
        Self::push_history(&mut self.latency_history, latency);
    }

    fn push_history(history: &mut Vec<u64>, value: u64) {
        history.push(value);
        if history.len() > HISTORY_LIMIT {
            history.remove(0);
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}
