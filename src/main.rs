mod app;
mod ui;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use app::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        // Render the UI
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // If help is showing, any key closes it
            if app.show_help {
                app.show_help = false;
                continue;
            }

            match key.code {
                // Quit
                KeyCode::Char('q') => app.quit(),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),

                // Help
                KeyCode::Char('?') => app.toggle_help(),

                // Panel navigation
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app.focus_prev();
                    } else {
                        app.focus_next();
                    }
                }
                KeyCode::BackTab => app.focus_prev(),
                KeyCode::Char('h') | KeyCode::Left => app.focus_prev(),
                KeyCode::Char('l') | KeyCode::Right => app.focus_next(),

                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
