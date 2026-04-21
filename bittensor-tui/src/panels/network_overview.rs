//! Network Overview panel — block height, total stake, issuance, hash rate.

use crate::network::NetworkData;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Brand colors for the Bittensor TUI.
pub const COLOR_NAVY: Color = Color::Rgb(10, 14, 39);
pub const COLOR_TEAL: Color = Color::Rgb(0, 212, 170);
pub const COLOR_GOLD: Color = Color::Rgb(255, 215, 0);
pub const COLOR_DIM: Color = Color::Rgb(80, 90, 120);
pub const COLOR_BG: Color = Color::Rgb(12, 16, 42);
pub const COLOR_BORDER: Color = Color::Rgb(30, 40, 80);

/// Render the network overview panel.
pub fn render(frame: &mut Frame, area: Rect, data: &NetworkData, focused: bool) {
    let border_style = if focused {
        Style::default().fg(COLOR_TEAL).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_BORDER)
    };

    let title = if focused { " ◈ Network Overview " } else { " Network Overview " };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(COLOR_BG));

    let status_indicator = if data.connected {
        Span::styled("●", Style::default().fg(COLOR_TEAL))
    } else {
        Span::styled("●", Style::default().fg(Color::Red))
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(" Status       ", Style::default().fg(COLOR_DIM)),
            status_indicator,
            if data.connected {
                Span::styled("  Connected", Style::default().fg(COLOR_TEAL))
            } else {
                Span::styled("  Disconnected", Style::default().fg(Color::Red))
            },
        ]),
        Line::from(vec![
            Span::styled(" Block Height ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!("  {}", data.block_height), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Total Stake  ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                format!("  {:.4} τ", data.total_stake.to_tao()),
                Style::default().fg(COLOR_GOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Issuance     ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                format!("  {:.4} τ", data.total_issuance.to_tao()),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Hash Rate    ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                format!("  {} H/s", data.network_hash_rate),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bittensor_core::balance::Balance;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn draw_network_overview(focused: bool) -> Terminal<TestBackend> {
        let backend = TestBackend::new(50, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let data = NetworkData {
            block_height: 12345,
            total_stake: Balance::from_tao(1000.0),
            total_issuance: Balance::from_tao(500.0),
            network_hash_rate: 42,
            connected: true,
            subnet_ids: vec![],
            last_error: None,
        };
        terminal.draw(|frame| render(frame, frame.area(), &data, focused)).unwrap();
        terminal
    }

    #[test]
    fn test_network_overview_renders_focused() {
        let terminal = draw_network_overview(true);
        let _buffer = terminal.backend().buffer();
        // Verify it rendered without panic
    }

    #[test]
    fn test_network_overview_renders_unfocused() {
        let terminal = draw_network_overview(false);
        let _buffer = terminal.backend().buffer();
    }

    #[test]
    fn test_brand_colors() {
        assert!(matches!(COLOR_TEAL, Color::Rgb(0, 212, 170)));
        assert!(matches!(COLOR_GOLD, Color::Rgb(255, 215, 0)));
        assert!(matches!(COLOR_NAVY, Color::Rgb(10, 14, 39)));
    }
}
