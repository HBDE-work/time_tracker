use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::glyphs::TUI;
use crate::storage::TrackerConfig;

pub(crate) fn render_autopause_editor_panel(
    config: &TrackerConfig,
    editing_autopause_index: Option<usize>,
    editing_autopause_buffer: &str,
) -> Vec<Line<'static>> {
    let mut content: Vec<Line<'static>> = Vec::new();

    let rule = TUI.horizontal_rule;
    content.push(Line::from(Span::styled(
        format!("{rule} Pause Editor  (N=new  1-9=edit  D+digit=delete  Esc/F11=close) {rule}"),
        Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD),
    )));
    content.push(Line::raw(""));

    let rules = config.auto_pause_rules();

    if rules.is_empty() {
        content.push(Line::from(Span::styled(
            "  No rules configured. Press N to add one.",
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));
    } else {
        for (idx, rule_entry) in rules.iter().enumerate() {
            let slot_number = idx + 1;
            let is_editing = editing_autopause_index == Some(idx);

            if is_editing {
                content.push(Line::from(vec![
                    Span::styled(
                        format!("  [{slot_number}] "),
                        Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        editing_autopause_buffer.to_owned(),
                        Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        TUI.cursor_block,
                        Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else {
                content.push(Line::from(vec![
                    Span::styled(
                        format!("  [{slot_number}] "),
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "after {}h  ->  {}min minimum pause",
                            rule_entry.after_hours, rule_entry.minimum_pause_minutes
                        ),
                        Style::new().fg(Color::White),
                    ),
                ]));
            }
        }
    }

    // show input line when adding a new rule
    if editing_autopause_index == Some(usize::MAX) {
        content.push(Line::raw(""));
        content.push(Line::from(vec![
            Span::styled(
                "  [new] ",
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                editing_autopause_buffer.to_owned(),
                Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                TUI.cursor_block,
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    content.push(Line::raw(""));
    content.push(Line::from(Span::styled(
        "  ─────────────────────────────────────",
        Style::new().fg(Color::DarkGray),
    )));
    content.push(Line::from(Span::styled(
        "  Format: HOURS:MINUTES  e.g.  8:30  or  9:45",
        Style::new()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));
    content.push(Line::from(Span::styled(
        "  Press F11 or Esc to close editor",
        Style::new()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    content
}
