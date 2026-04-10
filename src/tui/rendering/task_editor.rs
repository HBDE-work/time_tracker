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
) -> Vec<Line<'static>> {
    let mut content: Vec<Line<'static>> = Vec::new();

    let rule = TUI.horizontal_rule;
    content.push(Line::from(Span::styled(
        format!(
            "{rule} Task Editor  (press a number to edit, Enter to save, Esc to cancel) {rule}"
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
            let name = config.task_name(slot).unwrap_or("");
            let name_style = if name.is_empty() {
                Style::new().fg(Color::DarkGray)
            } else {
                Style::new().fg(Color::White)
            };
            let display = if name.is_empty() {
                TUI.empty_slot.to_owned()
            } else {
                name.to_owned()
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
        "  Press F1 or Esc to close editor",
        Style::new()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    content
}
