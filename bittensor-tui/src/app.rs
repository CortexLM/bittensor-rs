//! Application state and main event loop for the Bittensor TUI.

use crate::event::{Event, EventHandler};
use crate::network::{NetworkData, NetworkFetcher};
use crate::ui;
use ratatui::DefaultTerminal;
use std::sync::mpsc;
use std::time::Duration;

/// Which panel is currently focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    NetworkOverview,
    Wallet,
    Subnet,
    Delegate,
    Neuron,
}

impl Panel {
    /// All panels in navigation order.
    pub const ALL: [Panel; 5] =
        [Panel::NetworkOverview, Panel::Wallet, Panel::Subnet, Panel::Delegate, Panel::Neuron];

    /// Move to the next panel (Tab order).
    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    /// Move to the previous panel.
    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        if idx == 0 { Self::ALL[Self::ALL.len() - 1] } else { Self::ALL[idx - 1] }
    }
}

/// Application state.
pub struct App {
    /// Currently focused panel.
    pub active_panel: Panel,
    /// Whether the app should exit.
    pub should_quit: bool,
    /// Latest data from the network fetcher.
    pub network_data: NetworkData,
    /// Selected index within the active list panel.
    pub selected_index: usize,
    /// Whether a panel is "expanded" (Enter toggled).
    pub expanded: bool,
    /// Terminal size at last render.
    pub term_size: (u16, u16),
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new app with default state.
    pub fn new() -> Self {
        Self {
            active_panel: Panel::NetworkOverview,
            should_quit: false,
            network_data: NetworkData::default(),
            selected_index: 0,
            expanded: false,
            term_size: (80, 24),
        }
    }

    /// Handle a keyboard event, updating app state.
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if !self.expanded {
                    self.should_quit = true;
                } else {
                    self.expanded = false;
                }
            }
            KeyCode::Tab => {
                self.expanded = false;
                self.active_panel = self.active_panel.next();
                self.selected_index = 0;
            }
            KeyCode::BackTab => {
                self.expanded = false;
                self.active_panel = self.active_panel.prev();
                self.selected_index = 0;
            }
            KeyCode::Enter => {
                self.expanded = !self.expanded;
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                self.selected_index = self.selected_index.saturating_add(1);
            }
            KeyCode::Esc => {
                if self.expanded {
                    self.expanded = false;
                }
            }
            _ => {}
        }
    }

    /// Run the main TUI event loop.
    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        network_config: bittensor_core::config::NetworkConfig,
        refresh_secs: u64,
    ) -> std::io::Result<()> {
        // Spawn the async network fetcher
        let (data_tx, data_rx): (mpsc::Sender<NetworkData>, mpsc::Receiver<NetworkData>) =
            mpsc::channel();
        let fetcher = NetworkFetcher::new(network_config, refresh_secs);
        tokio::spawn(async move {
            fetcher.run(data_tx).await;
        });

        // Event handler with 100ms tick rate
        let mut event_handler = EventHandler::new(Duration::from_millis(100));

        loop {
            // Drain any network data updates
            while let Ok(data) = data_rx.try_recv() {
                self.network_data = data;
            }

            // Render
            terminal.draw(|frame| ui::draw(frame, self))?;

            // Handle events
            if let Some(event) = event_handler.try_recv() {
                match event {
                    Event::Key(key_event) => {
                        self.handle_key(key_event.code);
                    }
                    Event::Resize(w, h) => {
                        self.term_size = (w, h);
                    }
                    Event::Quit => {
                        self.should_quit = true;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_navigation() {
        let mut app = App::new();
        assert_eq!(app.active_panel, Panel::NetworkOverview);

        app.handle_key(crossterm::event::KeyCode::Tab);
        assert_eq!(app.active_panel, Panel::Wallet);

        app.handle_key(crossterm::event::KeyCode::BackTab);
        assert_eq!(app.active_panel, Panel::NetworkOverview);
    }

    #[test]
    fn test_quit() {
        let mut app = App::new();
        app.handle_key(crossterm::event::KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn test_expand_toggle() {
        let mut app = App::new();
        assert!(!app.expanded);
        app.handle_key(crossterm::event::KeyCode::Enter);
        assert!(app.expanded);
        app.handle_key(crossterm::event::KeyCode::Enter);
        assert!(!app.expanded);
    }

    #[test]
    fn test_esc_closes_expanded() {
        let mut app = App::new();
        app.expanded = true;
        app.handle_key(crossterm::event::KeyCode::Esc);
        assert!(!app.expanded);
    }

    #[test]
    fn test_arrow_keys() {
        let mut app = App::new();
        assert_eq!(app.selected_index, 0);
        app.handle_key(crossterm::event::KeyCode::Down);
        assert_eq!(app.selected_index, 1);
        app.handle_key(crossterm::event::KeyCode::Up);
        assert_eq!(app.selected_index, 0);
        // Up at 0 stays 0
        app.handle_key(crossterm::event::KeyCode::Up);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_panel_next_prev_cycle() {
        assert_eq!(Panel::NetworkOverview.next(), Panel::Wallet);
        assert_eq!(Panel::Neuron.next(), Panel::NetworkOverview);
        assert_eq!(Panel::NetworkOverview.prev(), Panel::Neuron);
        assert_eq!(Panel::Wallet.prev(), Panel::NetworkOverview);
    }

    #[test]
    fn test_tab_resets_selection() {
        let mut app = App::new();
        app.selected_index = 5;
        app.handle_key(crossterm::event::KeyCode::Tab);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_default_app() {
        let app = App::default();
        assert_eq!(app.active_panel, Panel::NetworkOverview);
        assert!(!app.should_quit);
        assert!(!app.expanded);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.term_size, (80, 24));
    }

    #[test]
    fn test_q_in_expanded_closes_expand() {
        let mut app = App::new();
        app.expanded = true;
        app.handle_key(crossterm::event::KeyCode::Char('q'));
        assert!(!app.expanded);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_uppercase_q_quit() {
        let mut app = App::new();
        app.handle_key(crossterm::event::KeyCode::Char('Q'));
        assert!(app.should_quit);
    }

    #[test]
    fn test_down_does_not_overflow() {
        let mut app = App::new();
        for _ in 0..1000 {
            app.handle_key(crossterm::event::KeyCode::Down);
        }
    }

    #[test]
    fn test_backtab_cycles() {
        let mut app = App::new();
        app.active_panel = Panel::NetworkOverview;
        app.handle_key(crossterm::event::KeyCode::BackTab);
        assert_eq!(app.active_panel, Panel::Neuron);
    }

    #[test]
    fn test_unknown_key_ignored() {
        let mut app = App::new();
        let prev = app.active_panel;
        app.handle_key(crossterm::event::KeyCode::Char('z'));
        assert_eq!(app.active_panel, prev);
        assert!(!app.should_quit);
    }
}
