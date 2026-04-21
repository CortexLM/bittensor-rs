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

    #[tokio::test]
    async fn test_try_recv_returns_none_when_no_events() {
        let mut handler = EventHandler::new(Duration::from_millis(100));
        // No events have been sent (no terminal attached), so try_recv should return None
        let result = handler.try_recv();
        assert!(result.is_none(), "try_recv should return None when no events are pending");
    }

    #[tokio::test]
    async fn test_event_handler_new_with_various_tick_rates() {
        let _h1 = EventHandler::new(Duration::from_millis(1));
        let _h2 = EventHandler::new(Duration::from_millis(500));
        let _h3 = EventHandler::new(Duration::from_secs(1));
        let _h4 = EventHandler::new(Duration::from_secs(10));
    }

    #[test]
    fn test_event_key_variant() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let event = Event::Key(key);
        assert!(matches!(event, Event::Key(k) if k.code == KeyCode::Char('a')));
    }

    #[test]
    fn test_event_resize_variant() {
        let event = Event::Resize(80, 24);
        assert!(matches!(event, Event::Resize(w, h) if w == 80 && h == 24));
    }

    #[test]
    fn test_event_quit_variant() {
        let event = Event::Quit;
        assert!(matches!(event, Event::Quit));
    }

    #[test]
    fn test_event_debug_format() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key_event = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        let debug_str = format!("{key_event:?}");
        assert!(debug_str.contains("Key"));

        let resize_event = Event::Resize(120, 40);
        let debug_str = format!("{resize_event:?}");
        assert!(debug_str.contains("Resize"));

        let quit_event = Event::Quit;
        let debug_str = format!("{quit_event:?}");
        assert!(debug_str.contains("Quit"));
    }
}
