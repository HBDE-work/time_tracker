use std::io;
use std::time::Duration;
use std::time::Instant;

use crossterm::event;

use super::app_state::App;
use super::rendering::render_actions_column;
use super::rendering::render_feedback_line;
use super::rendering::render_status_panel;
use super::rendering::render_task_editor_panel;
use super::rendering::render_task_indicators;
use super::rendering::render_toggles_column;

const REFRESH_INTERVAL: Duration = Duration::from_secs(60);

type Term = ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>;

/// launches the interactive TUI and blocks until the user quits
pub(crate) fn run_tui() -> io::Result<()> {
    let mut terminal = enter_tui_mode()?;
    let mut app = App::new();
    let mut next_refresh = Instant::now();

    while !app.should_quit {
        terminal.draw(|surface| {
            let regions = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Length(8),
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Fill(1),
            ])
            .split(surface.area());

            let commands_block = ratatui::widgets::Block::bordered()
                .title("  Commands (q to quit)  ")
                .border_style(ratatui::style::Style::new().fg(ratatui::style::Color::Cyan));
            let commands_area = commands_block.inner(regions[0]);
            surface.render_widget(commands_block, regions[0]);

            let command_columns = ratatui::layout::Layout::horizontal([
                ratatui::layout::Constraint::Fill(1),
                ratatui::layout::Constraint::Fill(2),
                ratatui::layout::Constraint::Min(26),
            ])
            .split(commands_area);

            let actions = render_actions_column();
            surface.render_widget(
                ratatui::widgets::Paragraph::new(actions),
                command_columns[0],
            );

            let tasks = render_task_indicators(&app.config, app.active_task);
            surface.render_widget(ratatui::widgets::Paragraph::new(tasks), command_columns[1]);

            let toggles = render_toggles_column(
                app.config.smartcard_active(),
                app.reader_status,
                app.task_editor_open,
                app.decimal_time_format,
                app.history_mode,
            );
            surface.render_widget(
                ratatui::widgets::Paragraph::new(toggles)
                    .alignment(ratatui::layout::Alignment::Right),
                command_columns[2],
            );

            let feedback_line = render_feedback_line(&app.feedback);
            surface.render_widget(ratatui::widgets::Paragraph::new(feedback_line), regions[1]);

            if app.task_editor_open {
                let editor_content = render_task_editor_panel(
                    &app.config,
                    app.editing_slot,
                    &app.editing_buffer,
                    app.editing_max_hours,
                );
                let editor_widget = ratatui::widgets::Paragraph::new(editor_content).block(
                    ratatui::widgets::Block::bordered()
                        .title("  Task Editor [F1]  ")
                        .border_style(
                            ratatui::style::Style::new().fg(ratatui::style::Color::Yellow),
                        ),
                );
                surface.render_widget(editor_widget, regions[2]);
            } else {
                let viewed_date = if app.history_mode {
                    Some(app.get_viewed_date())
                } else {
                    None
                };
                let status_content = render_status_panel(app.decimal_time_format, viewed_date);
                let status_widget = ratatui::widgets::Paragraph::new(status_content).block(
                    ratatui::widgets::Block::bordered()
                        .title("  Status  ")
                        .border_style(
                            ratatui::style::Style::new().fg(ratatui::style::Color::Yellow),
                        ),
                );
                surface.render_widget(status_widget, regions[2]);
            }
        })?;

        app.process_card_events();

        let max_wait = if app.config.smartcard_active() {
            Duration::from_millis(250)
        } else {
            REFRESH_INTERVAL
        };
        let wait = max_wait.min(REFRESH_INTERVAL.saturating_sub(next_refresh.elapsed()));
        if event::poll(wait)?
            && let event::Event::Key(ev) = event::read()?
            && ev.kind == event::KeyEventKind::Press
        {
            app.handle_key(ev.code);
        }

        if next_refresh.elapsed() >= REFRESH_INTERVAL {
            next_refresh = Instant::now();
        }
    }

    restore_terminal(&mut terminal);
    Ok(())
}

/// Switches the terminal into raw / alternate-screen mode and returns the
/// ratatui terminal handle
///
/// Caller is responsible for calling `restore_terminal` when finished
fn enter_tui_mode() -> io::Result<Term> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))
}

/// Undoes everything `enter_tui_mode` did so the users shell is usable again
fn restore_terminal(t: &mut Term) {
    let _ = crossterm::execute!(t.backend_mut(), crossterm::terminal::LeaveAlternateScreen);
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = t.show_cursor();
}
