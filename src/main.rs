mod app;
mod http;
mod models;
mod ui;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;
use tui_input::backend::crossterm::EventHandler;

use app::{App, EditFocus, Panel, RequestTab};
use http::{HttpResult, RequestData};

const HELP_LINES: usize = 24;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let (tx, mut rx) = mpsc::unbounded_channel::<HttpResult>();
    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        // Check for HTTP responses
        if let Ok(result) = rx.try_recv() {
            match result {
                HttpResult::Success(response) => app.set_response(response),
                HttpResult::Error(err) => app.set_error(err),
            }
        }

        if !event::poll(Duration::from_millis(50))? {
            continue;
        }

        let event = event::read()?;
        let Event::Key(key) = event else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Help overlay
        if app.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => app.show_help = false,
                KeyCode::Char('j') | KeyCode::Down => app.help_scroll_down(1, HELP_LINES),
                KeyCode::Char('k') | KeyCode::Up => app.help_scroll_up(1),
                _ => {}
            }
            continue;
        }

        // Global shortcuts
        match key.code {
            KeyCode::Char('s') if ctrl => {
                send_request(&rt, &mut app, tx.clone());
                continue;
            }
            KeyCode::Char('c') if ctrl => {
                app.quit();
                continue;
            }
            _ => {}
        }

        // Handle based on edit focus
        match app.edit_focus {
            EditFocus::None => handle_normal_mode(&mut app, key.code, ctrl),
            EditFocus::Url => handle_url_edit(&mut app, key, ctrl),
            EditFocus::KeyValue => handle_kv_edit(&mut app, key, ctrl),
            EditFocus::Body => handle_body_edit(&mut app, key, ctrl),
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_normal_mode(app: &mut App, code: KeyCode, ctrl: bool) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        KeyCode::Char('?') => app.toggle_help(),

        // Panel navigation
        KeyCode::Tab => app.focus_next_panel(),
        KeyCode::BackTab => app.focus_prev_panel(),
        KeyCode::Char('h') | KeyCode::Left => app.focus_prev_panel(),
        KeyCode::Char('l') | KeyCode::Right => app.focus_next_panel(),

        // Tab switching
        KeyCode::Char('1') => app.active_tab = RequestTab::Params,
        KeyCode::Char('2') => app.active_tab = RequestTab::Headers,
        KeyCode::Char('3') => app.active_tab = RequestTab::Body,

        // Context-specific
        KeyCode::Char('j') | KeyCode::Down => match app.focused_panel {
            Panel::Sidebar => app.select_next_request(),
            Panel::Response => app.scroll_down(1, response_lines(app)),
            Panel::RequestEditor => app.kv_select_next(),
        },
        KeyCode::Char('k') | KeyCode::Up => match app.focused_panel {
            Panel::Sidebar => app.select_prev_request(),
            Panel::Response => app.scroll_up(1),
            Panel::RequestEditor => app.kv_select_prev(),
        },

        // Response scrolling
        KeyCode::Char('g') if app.focused_panel == Panel::Response => app.scroll_top(),
        KeyCode::Char('G') if app.focused_panel == Panel::Response => {
            app.scroll_bottom(response_lines(app))
        }
        KeyCode::Char('d') if ctrl && app.focused_panel == Panel::Response => {
            app.scroll_down(10, response_lines(app))
        }
        KeyCode::Char('u') if ctrl && app.focused_panel == Panel::Response => app.scroll_up(10),

        // Sidebar actions
        KeyCode::Char('n') if app.focused_panel == Panel::Sidebar => {
            app.new_request();
        }
        KeyCode::Char('d') if app.focused_panel == Panel::Sidebar => {
            app.delete_selected_request();
        }
        KeyCode::Enter if app.focused_panel == Panel::Sidebar => {
            app.load_selected_request();
            app.focused_panel = Panel::RequestEditor;
        }

        // Request editor actions
        KeyCode::Char('i') if app.focused_panel == Panel::RequestEditor => {
            app.start_editing(EditFocus::Url);
        }
        KeyCode::Char('a') if app.focused_panel == Panel::RequestEditor => {
            if app.active_tab != RequestTab::Body {
                app.kv_add();
                app.start_editing(EditFocus::KeyValue);
            }
        }
        KeyCode::Char('e')
            if app.focused_panel == Panel::RequestEditor && app.active_tab == RequestTab::Body =>
        {
            app.start_editing(EditFocus::Body);
        }
        KeyCode::Enter if app.focused_panel == Panel::RequestEditor => match app.active_tab {
            RequestTab::Body => app.start_editing(EditFocus::Body),
            _ if !app.current_kv_items().is_empty() => {
                app.start_editing(EditFocus::KeyValue);
            }
            _ => {}
        },
        KeyCode::Char('d')
            if app.focused_panel == Panel::RequestEditor && app.active_tab != RequestTab::Body =>
        {
            app.kv_delete();
        }
        KeyCode::Char(' ')
            if app.focused_panel == Panel::RequestEditor && app.active_tab != RequestTab::Body =>
        {
            app.kv_toggle_enabled();
        }

        _ => {}
    }
}

fn handle_url_edit(app: &mut App, key: KeyEvent, _ctrl: bool) {
    match key.code {
        KeyCode::Esc => app.stop_editing(),
        KeyCode::Tab => app.cycle_method_next(),
        KeyCode::BackTab => app.cycle_method_prev(),
        _ => {
            // Let tui-input handle the rest
            app.url_input.handle_event(&Event::Key(key));
        }
    }
}

fn handle_kv_edit(app: &mut App, key: KeyEvent, ctrl: bool) {
    match key.code {
        KeyCode::Esc => app.stop_editing(),
        KeyCode::Tab => app.kv_toggle_field(),
        KeyCode::BackTab => app.kv_toggle_field(),
        KeyCode::Up if ctrl => app.kv_select_prev(),
        KeyCode::Down if ctrl => app.kv_select_next(),
        KeyCode::Char('k') if ctrl => app.kv_select_prev(),
        KeyCode::Char('j') if ctrl => app.kv_select_next(),
        KeyCode::Char(' ') if ctrl => app.kv_toggle_enabled(),
        KeyCode::Char('n') if ctrl => app.kv_add(),
        KeyCode::Char('d') if ctrl => {
            app.kv_delete();
            if app.current_kv_items().is_empty() {
                app.stop_editing();
            }
        }
        _ => {
            // Let tui-input handle the rest
            app.current_kv_editor_mut()
                .current_input_mut()
                .handle_event(&Event::Key(key));
        }
    }
}

fn handle_body_edit(app: &mut App, key: KeyEvent, ctrl: bool) {
    match key.code {
        KeyCode::Esc => app.stop_editing(),
        KeyCode::Char('f') if ctrl => app.format_json(),
        _ => {
            app.body_editor.input(key);
        }
    }
}

fn send_request(rt: &tokio::runtime::Runtime, app: &mut App, tx: mpsc::UnboundedSender<HttpResult>) {
    if app.is_loading() {
        return;
    }

    let url = app.url().trim().to_string();
    if url.is_empty() {
        app.set_error("URL is empty".to_string());
        return;
    }

    // Save or update request
    let body = app.body();
    let mut request = models::Request::new(app.method, url.clone());
    request.params = app.params.clone();
    request.headers = app.headers.clone();
    request.body = body.clone();

    if let Some(idx) = app.editing_request_idx {
        app.update_request(idx, request);
    } else {
        app.add_request(request);
    }

    let data = RequestData {
        method: app.method,
        url,
        params: app.params.clone(),
        headers: app.headers.clone(),
        body,
    };

    app.set_loading();

    rt.spawn(async move {
        http::send_request(data, tx).await;
    });
}

fn response_lines(app: &App) -> usize {
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
