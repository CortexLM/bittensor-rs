//! Main UI rendering — composes all panels into the dashboard layout.

use crate::app::{App, Panel};
use crate::panels::delegate::{self, DelegateEntry};
use crate::panels::network_overview;
use crate::panels::neuron::{self, NeuronDisplay};
use crate::panels::subnet::{self, SubnetEntry};
use crate::panels::wallet::{self, WalletData};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Draw the full dashboard.
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Top bar: title + status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),    // main panels
            Constraint::Length(3), // footer
        ])
        .split(size);

    render_header(frame, chunks[0], app);
    render_main(frame, chunks[1], app);
    render_footer(frame, chunks[2], app);
}

/// Render the header bar.
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let connected = if app.network_data.connected { "●" } else { "○" };
    let conn_color =
        if app.network_data.connected { network_overview::COLOR_TEAL } else { Color::Red };

    let title = Line::from(vec![
        Span::styled(" ◈ ", Style::default().fg(network_overview::COLOR_TEAL)),
        Span::styled("BITTENSOR", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" DASHBOARD ", Style::default().fg(network_overview::COLOR_DIM)),
        Span::styled(connected, Style::default().fg(conn_color)),
        Span::styled(
            format!(" Block #{}", app.network_data.block_height),
            Style::default().fg(network_overview::COLOR_DIM),
        ),
    ]);

    let paragraph =
        Paragraph::new(title).style(Style::default().bg(network_overview::COLOR_BG)).block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(network_overview::COLOR_BG)),
        );

    frame.render_widget(paragraph, area);
}

/// Render the main panel area — a 2×2 grid (or expanded single panel).
fn render_main(frame: &mut Frame, area: Rect, app: &App) {
    if app.expanded {
        // Single panel takes the full area
        match app.active_panel {
            Panel::NetworkOverview => {
                network_overview::render(frame, area, &app.network_data, true);
            }
            Panel::Wallet => {
                let data = WalletData::default();
                wallet::render(frame, area, &data, true);
            }
            Panel::Subnet => {
                let subnets: Vec<SubnetEntry> = Vec::new();
                subnet::render(frame, area, &subnets, app.selected_index, true);
            }
            Panel::Delegate => {
                let delegates: Vec<DelegateEntry> = Vec::new();
                delegate::render(frame, area, &delegates, app.selected_index, true);
            }
            Panel::Neuron => {
                let neuron = NeuronDisplay::default();
                neuron::render(frame, area, &neuron, true);
            }
        }
        return;
    }

    // 2×2 grid layout
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(columns[0]);

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(columns[1]);

    // Top-left: Network Overview
    network_overview::render(
        frame,
        left_rows[0],
        &app.network_data,
        app.active_panel == Panel::NetworkOverview,
    );

    // Bottom-left: Wallet
    let wallet_data = WalletData::default();
    wallet::render(frame, left_rows[1], &wallet_data, app.active_panel == Panel::Wallet);

    // Top-right: Subnet Explorer
    let subnets: Vec<SubnetEntry> = Vec::new();
    subnet::render(
        frame,
        right_rows[0],
        &subnets,
        app.selected_index,
        app.active_panel == Panel::Subnet,
    );

    // Bottom-right split: Delegate + Neuron
    let right_bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(right_rows[1]);

    let delegates: Vec<DelegateEntry> = Vec::new();
    delegate::render(
        frame,
        right_bottom[0],
        &delegates,
        app.selected_index,
        app.active_panel == Panel::Delegate,
    );

    let neuron = NeuronDisplay::default();
    neuron::render(frame, right_bottom[1], &neuron, app.active_panel == Panel::Neuron);
}

/// Render the footer bar with keyboard hints.
fn render_footer(frame: &mut Frame, area: Rect, _app: &App) {
    let hints = Line::from(vec![
        Span::styled(
            " Tab",
            Style::default().fg(network_overview::COLOR_TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Next ", Style::default().fg(network_overview::COLOR_DIM)),
        Span::styled(
            " ↑↓",
            Style::default().fg(network_overview::COLOR_TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Nav ", Style::default().fg(network_overview::COLOR_DIM)),
        Span::styled(
            " Enter",
            Style::default().fg(network_overview::COLOR_TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Expand ", Style::default().fg(network_overview::COLOR_DIM)),
        Span::styled(
            " Esc",
            Style::default().fg(network_overview::COLOR_TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Collapse ", Style::default().fg(network_overview::COLOR_DIM)),
        Span::styled(
            " q",
            Style::default().fg(network_overview::COLOR_TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Quit ", Style::default().fg(network_overview::COLOR_DIM)),
        Span::styled(
            " Ctrl+C",
            Style::default().fg(network_overview::COLOR_TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Force Quit", Style::default().fg(network_overview::COLOR_DIM)),
    ]);

    let paragraph =
        Paragraph::new(hints).style(Style::default().bg(network_overview::COLOR_BG)).block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(network_overview::COLOR_BG)),
        );

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::network::NetworkData;
    use bittensor_core::balance::Balance;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn draw_app(app: &App) -> Terminal<TestBackend> {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        terminal
    }

    #[test]
    fn test_draw_default_app() {
        let app = App::new();
        let _terminal = draw_app(&app);
    }

    #[test]
    fn test_draw_expanded_network() {
        let mut app = App::new();
        app.expanded = true;
        app.active_panel = Panel::NetworkOverview;
        app.network_data = NetworkData {
            block_height: 99999,
            total_stake: Balance::from_tao(12345.0),
            total_issuance: Balance::from_tao(6789.0),
            connected: true,
            ..Default::default()
        };
        let _terminal = draw_app(&app);
    }

    #[test]
    fn test_draw_disconnected() {
        let mut app = App::new();
        app.network_data.connected = false;
        app.network_data.last_error = Some("connection refused".into());
        let _terminal = draw_app(&app);
    }

    #[test]
    fn test_draw_each_panel_focused() {
        for panel in Panel::ALL {
            let mut app = App::new();
            app.active_panel = panel;
            let _terminal = draw_app(&app);
        }
    }

    #[test]
    fn test_draw_expanded_wallet() {
        let mut app = App::new();
        app.expanded = true;
        app.active_panel = Panel::Wallet;
        let _terminal = draw_app(&app);
    }

    #[test]
    fn test_draw_expanded_subnet() {
        let mut app = App::new();
        app.expanded = true;
        app.active_panel = Panel::Subnet;
        let _terminal = draw_app(&app);
    }

    #[test]
    fn test_draw_expanded_delegate() {
        let mut app = App::new();
        app.expanded = true;
        app.active_panel = Panel::Delegate;
        let _terminal = draw_app(&app);
    }

    #[test]
    fn test_draw_expanded_neuron() {
        let mut app = App::new();
        app.expanded = true;
        app.active_panel = Panel::Neuron;
        let _terminal = draw_app(&app);
    }

    #[test]
    fn test_draw_small_terminal() {
        let app = App::new();
        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_disconnected_with_error() {
        let mut app = App::new();
        app.network_data.connected = false;
        app.network_data.last_error = Some("timeout".into());
        let _terminal = draw_app(&app);
    }
}
