mod app;
mod models;
mod ui;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use app::{App, Panel};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

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
                // Quit (q / ctrl + c)
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
                KeyCode::BackTab => app.focus_prev(),  // Shift + Tab
                KeyCode::Char('h') | KeyCode::Left => app.focus_prev(),
                KeyCode::Char('l') | KeyCode::Right => app.focus_next(),

                // Sidebar navigation (when sidebar is focused)
                KeyCode::Char('j') | KeyCode::Down => {
                    if app.focused_panel == Panel::Sidebar {
                        app.select_next_request();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app.focused_panel == Panel::Sidebar {
                        app.select_prev_request();
                    }
                }

                KeyCode::Char('n') => {
                    if app.focused_panel == Panel::Sidebar {
                        app.add_request(models::Request::default());
                    }
                }
                KeyCode::Char('d') => {
                    if app.focused_panel == Panel::Sidebar {
                        app.delete_selected_request();
                    }
                }

                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
