use crossterm::event;

use super::app_state::App;
use super::rendering::render_command_panel;
use super::rendering::render_status_panel;

const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

type Term = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

/// launches the interactive TUI and blocks until the user quits
pub fn run_tui() -> std::io::Result<()> {
    let mut terminal = enter_tui_mode()?;
    let mut app = App::new();
    let mut next_refresh = std::time::Instant::now();

    while !app.should_quit {
        // render
        terminal.draw(|surface| {
            let regions = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Length(7),
                ratatui::layout::Constraint::Fill(1),
            ])
            .split(surface.area());

            let cmd_content = render_command_panel(&app.feedback);
            let cmd_widget = ratatui::widgets::Paragraph::new(cmd_content).block(
                ratatui::widgets::Block::bordered()
                    .title("  Commands (q to quit)  ")
                    .border_style(ratatui::style::Style::new().fg(ratatui::style::Color::Cyan)),
            );
            surface.render_widget(cmd_widget, regions[0]);

            let status_content = render_status_panel();
            let status_widget = ratatui::widgets::Paragraph::new(status_content).block(
                ratatui::widgets::Block::bordered()
                    .title("  Status  ")
                    .border_style(ratatui::style::Style::new().fg(ratatui::style::Color::Yellow)),
            );
            surface.render_widget(status_widget, regions[1]);
        })?;

        // poll for input or tick
        let wait = REFRESH_INTERVAL.saturating_sub(next_refresh.elapsed());
        if event::poll(wait)?
            && let event::Event::Key(ev) = event::read()?
            && ev.kind == event::KeyEventKind::Press
        {
            app.handle_key(ev.code);
        }

        if next_refresh.elapsed() >= REFRESH_INTERVAL {
            next_refresh = std::time::Instant::now();
        }
    }

    restore_terminal(&mut terminal);
    Ok(())
}

/// Switches the terminal into raw / alternate-screen mode and returns the
/// ratatui terminal handle
///
/// Caller is responsible for calling `restore_terminal` when finished
fn enter_tui_mode() -> std::io::Result<Term> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))
}

/// Undoes everything `enter_tui_mode` did so the users shell is usable again
fn restore_terminal(t: &mut Term) {
    let _ = crossterm::execute!(t.backend_mut(), crossterm::terminal::LeaveAlternateScreen);
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = t.show_cursor();
}
