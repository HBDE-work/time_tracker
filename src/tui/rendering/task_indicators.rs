use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::glyphs::TUI;
use crate::storage::TrackerConfig;

pub(crate) fn render_task_indicators(
    config: &TrackerConfig,
    active_task: Option<u8>,
) -> Vec<Line<'static>> {
    const COL_WIDTH: usize = 22;

    let slots: Vec<(u8, &str)> = (0u8..=9)
        .filter_map(|s| config.task_name(s).map(|n| (s, n)))
        .collect();

    let mut content: Vec<Line<'static>> = Vec::new();
    content.push(Line::raw(""));

    for row in slots.chunks(3) {
        let mut timespans: Vec<Span<'static>> = Vec::new();

        for (slot, name) in row {
            let is_active = active_task == Some(*slot);

            let (prefix, display_name) = if is_active {
                (format!("[{slot}]{}", TUI.task_playing), *name)
            } else {
                (format!("[{slot}] "), *name)
            };

            let prefix_len = prefix.chars().count();
            let max_name = COL_WIDTH.saturating_sub(prefix_len);
            let truncated: String = display_name.chars().take(max_name).collect();

            let style = if is_active {
                Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::White)
            };

            if is_active {
                timespans.push(Span::styled(
                    format!("[{slot}]"),
                    Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
                ));
                let play = TUI.task_playing;
                let padded_name = format!("{play} {truncated:<pad$}", pad = max_name);
                timespans.push(Span::styled(padded_name, style));
            } else {
                timespans.push(Span::styled(
                    format!("[{slot}]"),
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ));
                let padded_name = format!(" {truncated:<pad$}", pad = max_name);
                timespans.push(Span::styled(padded_name, style));
            }
        }

        content.push(Line::from(timespans));
    }

    content
}
