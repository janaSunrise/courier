mod app;
mod http;
mod models;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use app::{App, Panel};
use http::HttpResult;

/// Total number of lines in the help overlay (for scrolling calculation)
const HELP_TOTAL_LINES: usize = 28;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    // Channel for receiving HTTP results
    let (tx, mut rx) = mpsc::unbounded_channel::<HttpResult>();

    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        if let Ok(result) = rx.try_recv() {
            match result {
                HttpResult::Success(response) => app.set_response(response),
                HttpResult::Error(err) => app.set_error(err),
            }
        }

        // Poll for keyboard events with timeout to allow checking HTTP results
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.show_help {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                            app.show_help = false;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.scroll_help_down(1, HELP_TOTAL_LINES);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.scroll_help_up(1);
                        }
                        KeyCode::Char('g') => {
                            app.help_scroll = 0;
                        }
                        KeyCode::Char('G') => {
                            app.scroll_help_down(HELP_TOTAL_LINES, HELP_TOTAL_LINES);
                        }
                        _ => {}
                    }
                    continue;
                }

                if app.input_mode {
                    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                    let alt = key.modifiers.contains(KeyModifiers::ALT);

                    match key.code {
                        KeyCode::Esc => app.exit_input_mode(),

                        // Send request with Ctrl+S
                        KeyCode::Char('s') if ctrl => {
                            send_request(&rt, &mut app, tx.clone());
                        }

                        // Text Navigation
                        KeyCode::Left if ctrl || alt => app.move_cursor_word_left(),
                        KeyCode::Right if ctrl || alt => app.move_cursor_word_right(),
                        KeyCode::Left => app.move_cursor_left(),
                        KeyCode::Right => app.move_cursor_right(),
                        KeyCode::Home | KeyCode::Char('a') if ctrl => app.move_cursor_start(),
                        KeyCode::End | KeyCode::Char('e') if ctrl => app.move_cursor_end(),

                        // Deletion
                        KeyCode::Backspace if ctrl || alt => app.delete_word_backward(),
                        KeyCode::Backspace => app.delete_char(),
                        KeyCode::Delete => app.delete_char_forward(),
                        KeyCode::Char('u') if ctrl => app.delete_to_start(),
                        KeyCode::Char('k') if ctrl => app.delete_to_end(),
                        KeyCode::Char('w') if ctrl => app.delete_word_backward(),

                        // Clear all
                        KeyCode::Char('l') if ctrl => app.clear_input(),

                        // Method cycling
                        KeyCode::Tab => app.cycle_method_next(),
                        KeyCode::BackTab => app.cycle_method_prev(),

                        // Regular character input
                        KeyCode::Char(c) => app.input_char(c),

                        _ => {}
                    }
                    continue;
                }

                // Normal mode handling
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

                match key.code {
                    // Quit
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Char('c') if ctrl => app.quit(),
                    KeyCode::Esc => app.quit(),

                    // Help
                    KeyCode::Char('?') => app.toggle_help(),

                    // Send request with Ctrl+S
                    KeyCode::Char('s') if ctrl => {
                        send_request(&rt, &mut app, tx.clone());
                    }

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

                    // Sidebar navigation / Response scrolling
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app.focused_panel == Panel::Sidebar {
                            app.select_next_request();
                        } else if app.focused_panel == Panel::Response {
                            app.scroll_response_down(1, get_response_line_count(&app));
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app.focused_panel == Panel::Sidebar {
                            app.select_prev_request();
                        } else if app.focused_panel == Panel::Response {
                            app.scroll_response_up(1);
                        }
                    }

                    // Page up/down for response
                    KeyCode::PageDown | KeyCode::Char('d') if ctrl => {
                        if app.focused_panel == Panel::Response {
                            app.scroll_response_down(10, get_response_line_count(&app));
                        }
                    }
                    KeyCode::PageUp | KeyCode::Char('u') if ctrl => {
                        if app.focused_panel == Panel::Response {
                            app.scroll_response_up(10);
                        }
                    }

                    // Home/End for response
                    KeyCode::Char('g') => {
                        if app.focused_panel == Panel::Response {
                            app.scroll_response_top();
                        }
                    }
                    KeyCode::Char('G') => {
                        if app.focused_panel == Panel::Response {
                            app.scroll_response_bottom(get_response_line_count(&app));
                        }
                    }

                    // Request management
                    KeyCode::Char('n') => {
                        if app.focused_panel == Panel::Sidebar {
                            app.add_request(models::Request::default());
                        }
                    }
                    KeyCode::Char('d') if !ctrl => {
                        if app.focused_panel == Panel::Sidebar {
                            app.delete_selected_request();
                        }
                    }

                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn send_request(
    rt: &tokio::runtime::Runtime,
    app: &mut App,
    tx: mpsc::UnboundedSender<HttpResult>,
) {
    if app.is_loading() {
        return; // Don't send if already loading
    }

    let url = app.input_url.trim().to_string();
    if url.is_empty() {
        app.set_error("URL is empty".to_string());
        return;
    }

    // Auto-prepend https:// if no scheme is present
    let url = if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url
    };

    let method = app.input_method;
    app.set_loading();

    rt.spawn(async move {
        http::send_request(method, url, tx).await;
    });
}

/// Get the number of lines in the response body for scrolling
fn get_response_line_count(app: &App) -> usize {
    if let models::RequestState::Success(ref resp) = app.request_state {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp.body) {
            if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                return pretty.lines().count();
            }
        }
        resp.body.lines().count()
    } else {
        0
    }
}
