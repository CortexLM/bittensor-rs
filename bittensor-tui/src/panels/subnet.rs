//! Subnet Explorer panel — list subnets, view details.

use crate::panels::network_overview::{COLOR_BG, COLOR_BORDER, COLOR_DIM, COLOR_GOLD, COLOR_TEAL};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

/// Subnet list entry for display.
#[derive(Debug, Clone, Default)]
pub struct SubnetEntry {
    pub netuid: u16,
    pub name: String,
    pub owner: String,
    pub neuron_count: u16,
    pub total_stake: f64,
}

/// Render the subnet explorer panel.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    subnets: &[SubnetEntry],
    selected: usize,
    focused: bool,
) {
    let border_style = if focused {
        Style::default().fg(COLOR_TEAL).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_BORDER)
    };

    let title = if focused { " ◈ Subnets " } else { " Subnets " };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(COLOR_BG));

    let items: Vec<ListItem> = subnets
        .iter()
        .map(|s| {
            let line = Line::from(vec![
                Span::styled(format!(" {:>3} ", s.netuid), Style::default().fg(COLOR_DIM)),
                Span::styled(format!("{:<16}", s.name), Style::default().fg(Color::White)),
                Span::styled(format!(" {:>8.2} τ", s.total_stake), Style::default().fg(COLOR_GOLD)),
                Span::styled(format!(" N:{}", s.neuron_count), Style::default().fg(COLOR_TEAL)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(20, 30, 60)).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !subnets.is_empty() {
        state.select(Some(selected.min(subnets.len() - 1)));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_subnet_panel_empty() {
        let backend = TestBackend::new(50, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, frame.area(), &[], 0, true)).unwrap();
    }

    #[test]
    fn test_subnet_panel_with_data() {
        let backend = TestBackend::new(60, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let subnets = vec![
            SubnetEntry {
                netuid: 1,
                name: "root".into(),
                owner: "5Owner1".into(),
                neuron_count: 64,
                total_stake: 5000.0,
            },
            SubnetEntry {
                netuid: 3,
                name: "compute".into(),
                owner: "5Owner2".into(),
                neuron_count: 256,
                total_stake: 12000.0,
            },
        ];
        terminal.draw(|frame| render(frame, frame.area(), &subnets, 0, true)).unwrap();
    }
}
