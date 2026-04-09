use std::io;
use std::time::Duration;

use crossterm::event;

use super::app_state::App;
use super::rendering::render_actions_column;
use super::rendering::render_status_panel;
use super::rendering::render_toggles_column;

const REFRESH_INTERVAL: Duration = Duration::from_secs(60);

type Term = ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>;

/// launches the interactive TUI and blocks until the user quits
pub(crate) fn run_tui() -> io::Result<()> {
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

            // Upper panel: bordered block with two inner columns
            let cmd_block = ratatui::widgets::Block::bordered()
                .title("  Commands (q to quit)  ")
                .border_style(ratatui::style::Style::new().fg(ratatui::style::Color::Cyan));
            let cmd_inner = cmd_block.inner(regions[0]);
            surface.render_widget(cmd_block, regions[0]);

            let cols = ratatui::layout::Layout::horizontal([
                ratatui::layout::Constraint::Fill(1),
                ratatui::layout::Constraint::Min(26),
            ])
            .split(cmd_inner);

            let actions = render_actions_column(&app.feedback);
            surface.render_widget(ratatui::widgets::Paragraph::new(actions), cols[0]);

            let toggles = render_toggles_column(app.config.smartcard_active, app.reader_status);
            surface.render_widget(ratatui::widgets::Paragraph::new(toggles), cols[1]);

            // Lower panel
            let status_content = render_status_panel();
            let status_widget = ratatui::widgets::Paragraph::new(status_content).block(
                ratatui::widgets::Block::bordered()
                    .title("  Status  ")
                    .border_style(ratatui::style::Style::new().fg(ratatui::style::Color::Yellow)),
            );
            surface.render_widget(status_widget, regions[1]);
        })?;

        // check for smartcard events coming from the background thread
        app.process_card_events();

        // poll for input or tick
        // Use a shorter wait when smartcard watching is active so the
        // UI reacts quickly to card-inserted / card-removed signals
        let max_wait = if app.config.smartcard_active {
            std::time::Duration::from_millis(250)
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
