use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use eidra_proxy::EventReceiver;
use eidra_tui::app::TuiApp;
use eidra_tui::event::{RequestAction, RequestEntry};
use eidra_tui::ui;

pub async fn run_tui(mut event_rx: EventReceiver) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new();
    let start_time = std::time::Instant::now();

    loop {
        app.uptime_secs = start_time.elapsed().as_secs();
        app.tick();
        // Draw
        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        // Poll for proxy events (non-blocking)
        while let Ok(proxy_event) = event_rx.try_recv() {
            let action = match proxy_event.action.as_str() {
                "block" => RequestAction::Block,
                "mask" => RequestAction::Mask,
                "route" => RequestAction::Route,
                "escalate" => RequestAction::Escalate,
                "allow" => RequestAction::Allow,
                _ => RequestAction::Allow,
            };
            app.add_entry(RequestEntry {
                timestamp: proxy_event.timestamp,
                action,
                provider: proxy_event.provider,
                findings_count: proxy_event.findings_count,
                categories: proxy_event.categories,
                data_size_bytes: proxy_event.data_size_bytes,
                latency_ms: proxy_event.latency_ms,
                status_code: proxy_event.status_code,
            });
        }

        // Poll for keyboard events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                            break;
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
                        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
