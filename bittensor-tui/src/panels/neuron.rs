//! Neuron View panel — selected neuron details (rank, trust, incentive, bonds, weights).

use crate::panels::network_overview::{COLOR_BG, COLOR_BORDER, COLOR_DIM, COLOR_GOLD, COLOR_TEAL};
use bittensor_core::balance::Balance;
use bittensor_core::types::NeuronInfo;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Neuron display data.
#[derive(Debug, Clone, Default)]
pub struct NeuronDisplay {
    pub uid: u16,
    pub netuid: u16,
    pub hotkey: String,
    pub coldkey: String,
    pub active: bool,
    pub stake: Balance,
    pub rank: u16,
    pub trust: u16,
    pub consensus: u16,
    pub incentive: u16,
    pub dividend: u16,
    pub emission: u64,
    pub validator_trust: u16,
}

impl From<&NeuronInfo> for NeuronDisplay {
    fn from(n: &NeuronInfo) -> Self {
        Self {
            uid: n.uid,
            netuid: n.netuid,
            hotkey: n.hotkey.clone(),
            coldkey: n.coldkey.clone(),
            active: n.active,
            stake: n.stake,
            rank: n.rank,
            trust: n.trust,
            consensus: n.consensus,
            incentive: n.incentive,
            dividend: n.dividend,
            emission: n.emission,
            validator_trust: n.validator_trust,
        }
    }
}

/// Render the neuron view panel.
pub fn render(frame: &mut Frame, area: Rect, neuron: &NeuronDisplay, focused: bool) {
    let border_style = if focused {
        Style::default().fg(COLOR_TEAL).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_BORDER)
    };

    let title = if focused { " ◈ Neuron " } else { " Neuron " };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(COLOR_BG));

    let active_color = if neuron.active { COLOR_TEAL } else { Color::Red };

    let lines = vec![
        Line::from(vec![
            Span::styled(" UID       ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.uid), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Subnet    ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.netuid), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Active    ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.active), Style::default().fg(active_color)),
        ]),
        Line::from(vec![
            Span::styled(" Stake     ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                format!(" {:.4} τ", neuron.stake.to_tao()),
                Style::default().fg(COLOR_GOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Rank      ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.rank), Style::default().fg(COLOR_TEAL)),
        ]),
        Line::from(vec![
            Span::styled(" Trust     ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.trust), Style::default().fg(COLOR_TEAL)),
        ]),
        Line::from(vec![
            Span::styled(" Consensus ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.consensus), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Incentive ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.incentive), Style::default().fg(COLOR_GOLD)),
        ]),
        Line::from(vec![
            Span::styled(" Dividend  ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.dividend), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Emission  ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {} rao", neuron.emission), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" V.Trust   ", Style::default().fg(COLOR_DIM)),
            Span::styled(format!(" {}", neuron.validator_trust), Style::default().fg(COLOR_TEAL)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_neuron_panel_default() {
        let backend = TestBackend::new(40, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        let neuron = NeuronDisplay::default();
        terminal.draw(|frame| render(frame, frame.area(), &neuron, true)).unwrap();
    }

    #[test]
    fn test_neuron_panel_with_data() {
        let backend = TestBackend::new(40, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        let neuron = NeuronDisplay {
            uid: 42,
            netuid: 1,
            hotkey: "5Hk".into(),
            coldkey: "5Ck".into(),
            active: true,
            stake: Balance::from_tao(100.0),
            rank: 85,
            trust: 92,
            consensus: 78,
            incentive: 65,
            dividend: 10,
            emission: 5000,
            validator_trust: 90,
        };
        terminal.draw(|frame| render(frame, frame.area(), &neuron, false)).unwrap();
    }

    #[test]
    fn test_neuron_display_from_neuron_info() {
        let info = NeuronInfo {
            uid: 1,
            netuid: 2,
            active: true,
            stake: Balance::from_tao(50.0),
            rank: 10,
            trust: 5,
            consensus: 3,
            incentive: 7,
            dividend: 2,
            emission: 1000,
            prometheus_info: None,
            axon_info: None,
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            last_update: 0,
            validator_trust: 8,
            weights: vec![],
            bonds: vec![],
            stake_dict: vec![],
        };
        let display = NeuronDisplay::from(&info);
        assert_eq!(display.uid, 1);
        assert_eq!(display.netuid, 2);
    }
}
