use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::glyphs::TUI;
use crate::storage::TrackerConfig;

pub(crate) fn render_task_editor_panel(
    config: &TrackerConfig,
    editing_slot: Option<u8>,
    editing_buffer: &str,
    editing_max_hours: bool,
) -> Vec<Line<'static>> {
    let mut content: Vec<Line<'static>> = Vec::new();

    let rule = TUI.horizontal_rule;
    content.push(Line::from(Span::styled(
        format!(
            "{rule} Task Editor  (press 0-9 to edit task, H for max hours, Enter to save, Esc to cancel) {rule}"
        ),
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    content.push(Line::raw(""));

    for slot in 0u8..=9 {
        let is_editing = editing_slot == Some(slot);

        if is_editing {
            content.push(Line::from(vec![
                Span::styled(
                    format!("  [{slot}] "),
                    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    editing_buffer.to_owned(),
                    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    TUI.cursor_block,
                    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            let task_info = config.task_info(slot);
            let name_style = if task_info.is_some() {
                Style::new().fg(Color::White)
            } else {
                Style::new().fg(Color::DarkGray)
            };
            let display = if let Some(info) = task_info {
                if let Some(max_hours) = info.max_hours {
                    format!("{} ({:.2}h)", info.name, max_hours)
                } else {
                    info.name.clone()
                }
            } else {
                TUI.empty_slot.to_owned()
            };
            content.push(Line::from(vec![
                Span::styled(
                    format!("  [{slot}] "),
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(display, name_style),
            ]));
        }
    }

    content.push(Line::raw(""));
    content.push(Line::from(Span::styled(
        "  ─────────────────────",
        Style::new().fg(Color::DarkGray),
    )));

    // Max hours per day setting
    if editing_max_hours {
        content.push(Line::from(vec![
            Span::styled(
                "  [H] Max hours/day: ",
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                editing_buffer.to_owned(),
                Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                TUI.cursor_block,
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));
    } else {
        let max_display = config
            .max_hours_per_day()
            .map(|h| format!("{:.2}h", h))
            .unwrap_or_else(|| "not set".to_owned());
        let max_style = if config.max_hours_per_day().is_some() {
            Style::new().fg(Color::White)
        } else {
            Style::new().fg(Color::DarkGray)
        };
        content.push(Line::from(vec![
            Span::styled(
                "  [H] Max hours/day: ",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(max_display, max_style),
        ]));
    }

    content.push(Line::raw(""));
    content.push(Line::from(Span::styled(
        "  Press F1 or Esc to close editor",
        Style::new()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    content
}
