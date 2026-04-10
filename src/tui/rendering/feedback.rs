use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

/// Single-line feedback styled as italic white text
pub(crate) fn render_feedback_line(feedback: &str) -> Line<'_> {
    Line::from(Span::styled(
        format!(" {feedback}"),
        Style::new().fg(Color::White).add_modifier(Modifier::ITALIC),
    ))
}
