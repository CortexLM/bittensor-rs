//! Keyboard and terminal event handling using crossterm.

use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

/// Events the TUI loop processes.
#[derive(Debug)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Resize(u16, u16),
    Quit,
}

/// Async event handler that polls crossterm for keyboard and resize events.
pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate for polling.
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                // Poll with a timeout using spawn_blocking so we don't block the tokio runtime
                let has_event = tokio::task::spawn_blocking({
                    let tick = tick_rate;
                    move || event::poll(tick).unwrap_or(false)
                })
                .await
                .unwrap_or(false);

                if has_event {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if key.kind == KeyEventKind::Press {
                                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                    && key.code == crossterm::event::KeyCode::Char('c')
                                {
                                    let _ = tx.send(Event::Quit);
                                    break;
                                }
                                if tx.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            if tx.send(Event::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {
                            let _ = tx.send(Event::Quit);
                            break;
                        }
                    }
                }
            }
        });

        Self { rx }
    }

    /// Try to receive the next event (non-blocking).
    pub fn try_recv(&mut self) -> Option<Event> {
        self.rx.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_handler_creation() {
        let _handler = EventHandler::new(Duration::from_millis(100));
    }
}
