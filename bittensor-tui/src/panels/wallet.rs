//! Wallet panel — displays wallet balance, stakes, and delegations.

use crate::panels::network_overview::{COLOR_BG, COLOR_BORDER, COLOR_DIM, COLOR_GOLD, COLOR_TEAL};
use bittensor_core::balance::Balance;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Wallet display data (simplified for V1 — no live wallet integration yet).
#[derive(Debug, Clone, Default)]
pub struct WalletData {
    pub address: String,
    pub free_balance: Balance,
    pub staked_balance: Balance,
    pub delegations: Vec<DelegationEntry>,
}

#[derive(Debug, Clone)]
pub struct DelegationEntry {
    pub delegate_ss58: String,
    pub amount: Balance,
}

/// Render the wallet panel.
pub fn render(frame: &mut Frame, area: Rect, data: &WalletData, focused: bool) {
    let border_style = if focused {
        Style::default().fg(COLOR_TEAL).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_BORDER)
    };

    let title = if focused { " ◈ Wallet " } else { " Wallet " };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(COLOR_BG));

    let addr_display = if data.address.len() > 20 {
        format!("{}…{}", &data.address[..8], &data.address[data.address.len() - 6..])
    } else {
        data.address.clone()
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(" Address  ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", addr_display), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Free     ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                format!(" {:.4} τ", data.free_balance.to_tao()),
                Style::default().fg(COLOR_GOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Staked   ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                format!(" {:.4} τ", data.staked_balance.to_tao()),
                Style::default().fg(COLOR_TEAL),
            ),
        ]),
    ];

    if !data.delegations.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(" Delegations:", Style::default().fg(COLOR_DIM))));
        for (i, del) in data.delegations.iter().take(5).enumerate() {
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", i + 1), Style::default().fg(COLOR_DIM)),
                Span::styled(
                    format!("{:.4} τ → ", del.amount.to_tao()),
                    Style::default().fg(COLOR_TEAL),
                ),
                Span::styled(&del.delegate_ss58, Style::default().fg(Color::White)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_wallet_panel_renders() {
        let backend = TestBackend::new(50, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        let data = WalletData {
            address: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
            free_balance: Balance::from_tao(42.0),
            staked_balance: Balance::from_tao(100.0),
            delegations: vec![DelegationEntry {
                delegate_ss58: "5Delegate".into(),
                amount: Balance::from_tao(50.0),
            }],
        };
        terminal.draw(|frame| render(frame, frame.area(), &data, true)).unwrap();
    }

    #[test]
    fn test_wallet_empty_delegations() {
        let backend = TestBackend::new(50, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let data = WalletData {
            address: "5Test".into(),
            free_balance: Balance::ZERO,
            staked_balance: Balance::ZERO,
            delegations: vec![],
        };
        terminal.draw(|frame| render(frame, frame.area(), &data, false)).unwrap();
    }
}
