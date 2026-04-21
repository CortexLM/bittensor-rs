//! Delegate Monitor panel — top delegates, take percentages, delegations.

use crate::panels::network_overview::{COLOR_BG, COLOR_BORDER, COLOR_DIM, COLOR_GOLD, COLOR_TEAL};
use bittensor_core::balance::Balance;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

/// Delegate entry for display.
#[derive(Debug, Clone, Default)]
pub struct DelegateEntry {
    pub hotkey: String,
    pub total_stake: Balance,
    pub take: u16,
    pub nominator_count: usize,
}

/// Render the delegate monitor panel.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    delegates: &[DelegateEntry],
    selected: usize,
    focused: bool,
) {
    let border_style = if focused {
        Style::default().fg(COLOR_TEAL).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_BORDER)
    };

    let title = if focused { " ◈ Delegates " } else { " Delegates " };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(COLOR_BG));

    let items: Vec<ListItem> = delegates
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let take_pct = d.take as f64 / 65535.0 * 100.0;
            let hk_display = if d.hotkey.len() > 12 {
                format!("{}…", &d.hotkey[..8])
            } else {
                d.hotkey.clone()
            };
            let line = Line::from(vec![
                Span::styled(format!(" {:>2} ", i + 1), Style::default().fg(COLOR_DIM)),
                Span::styled(format!("{:<12}", hk_display), Style::default().fg(Color::White)),
                Span::styled(
                    format!(" {:>8.2} τ", d.total_stake.to_tao()),
                    Style::default().fg(COLOR_GOLD),
                ),
                Span::styled(format!(" take:{:.1}%", take_pct), Style::default().fg(COLOR_TEAL)),
                Span::styled(
                    format!(" noms:{}", d.nominator_count),
                    Style::default().fg(COLOR_DIM),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(20, 30, 60)).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !delegates.is_empty() {
        state.select(Some(selected.min(delegates.len() - 1)));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_delegate_panel_empty() {
        let backend = TestBackend::new(50, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, frame.area(), &[], 0, true)).unwrap();
    }

    #[test]
    fn test_delegate_panel_with_data() {
        let backend = TestBackend::new(70, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let delegates = vec![DelegateEntry {
            hotkey: "5GrwvaEF5zXb26".into(),
            total_stake: Balance::from_tao(50000.0),
            take: 6553,
            nominator_count: 42,
        }];
        terminal.draw(|frame| render(frame, frame.area(), &delegates, 0, true)).unwrap();
    }
}
