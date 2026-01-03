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

            // Input mode handling
            if app.input_mode {
                match key.code {
                    KeyCode::Esc => app.exit_input_mode(),
                    KeyCode::Char(c) => app.input_char(c),
                    KeyCode::Backspace => app.delete_char(),
                    KeyCode::Delete => app.delete_char_forward(),
                    KeyCode::Left => app.move_cursor_left(),
                    KeyCode::Right => app.move_cursor_right(),
                    KeyCode::Home => app.move_cursor_start(),
                    KeyCode::End => app.move_cursor_end(),
                    // Cycle method with Ctrl + m or Tab
                    KeyCode::Tab => app.cycle_method_next(),
                    KeyCode::BackTab => app.cycle_method_prev(),
                    _ => {}
                }
                continue;
            }

            // Normal mode handling
            match key.code {
                // Quit
                KeyCode::Char('q') => app.quit(),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
                KeyCode::Esc => app.quit(),

                // Help
                KeyCode::Char('?') => app.toggle_help(),

                // Enter input mode
                KeyCode::Char('i') | KeyCode::Enter => {
                    if app.focused_panel == Panel::RequestEditor {
                        app.enter_input_mode();
                    } else if app.focused_panel == Panel::Sidebar {
                        // Load selected request and switch to editor
                        app.load_selected_request();
                        app.focused_panel = Panel::RequestEditor;
                    }
                }

                // Panel navigation
                KeyCode::Tab => app.focus_next(),
                KeyCode::BackTab => app.focus_prev(),
                KeyCode::Char('h') | KeyCode::Left => app.focus_prev(),
                KeyCode::Char('l') | KeyCode::Right => app.focus_next(),

                // Sidebar navigation
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

                // Request management
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
