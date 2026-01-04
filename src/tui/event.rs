use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use tokio::sync::mpsc;
use crate::tui::app::Action;

pub fn start_event_listener(tx: mpsc::Sender<Action>) {
    tokio::task::spawn_blocking(move || {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = std::time::Instant::now();

        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                let _ = tx.blocking_send(Action::Quit);
                                break;
                            }
                            KeyCode::Char('r') => {
                                let _ = tx.blocking_send(Action::Refresh);
                            }
                            _ => {
                                let _ = tx.blocking_send(Action::Key(key));
                            }
                        }
                    }
                    _ => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if tx.blocking_send(Action::Tick).is_err() {
                    break;
                }
                last_tick = std::time::Instant::now();
            }
        }
    });
}
