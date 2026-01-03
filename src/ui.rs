use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, Panel};
use crate::models::{HttpMethod, RequestState};

pub mod theme {
    use ratatui::style::Color;

    // Background
    pub const BG: Color = Color::Rgb(16, 20, 30);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(30, 36, 50);

    // Borders
    pub const BORDER: Color = Color::Rgb(55, 65, 85);
    pub const BORDER_FOCUSED: Color = Color::Rgb(139, 92, 246);

    // Text
    pub const TEXT: Color = Color::Rgb(226, 232, 240);
    pub const TEXT_DIM: Color = Color::Rgb(100, 116, 139);

    // Accent
    pub const ACCENT: Color = Color::Rgb(139, 92, 246);

    // Methods
    pub const METHOD_GET: Color = Color::Rgb(52, 211, 153);     // emerald
    pub const METHOD_POST: Color = Color::Rgb(251, 191, 36);    // amber
    pub const METHOD_PUT: Color = Color::Rgb(96, 165, 250);     // sky blue
    pub const METHOD_PATCH: Color = Color::Rgb(192, 132, 252);  // purple
    pub const METHOD_DELETE: Color = Color::Rgb(251, 113, 133); // rose
    pub const METHOD_HEAD: Color = Color::Rgb(94, 234, 212);    // teal
    pub const METHOD_OPTIONS: Color = Color::Rgb(156, 163, 175); // gray

    // HTTP Status codes
    pub const STATUS_SUCCESS: Color = Color::Rgb(52, 211, 153);       // 2xx - emerald
    pub const STATUS_REDIRECT: Color = Color::Rgb(96, 165, 250);      // 3xx - sky blue
    pub const STATUS_CLIENT_ERROR: Color = Color::Rgb(251, 191, 36);  // 4xx - amber
    pub const STATUS_SERVER_ERROR: Color = Color::Rgb(251, 113, 133); // 5xx - rose
    pub const STATUS_LOADING: Color = Color::Rgb(139, 92, 246);       // loading - violet
    pub const STATUS_ERROR: Color = Color::Rgb(251, 113, 133);        // error - rose
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Vertical layout: main content + status bar
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Main layout: three columns
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Sidebar
            Constraint::Percentage(40), // Request editor
            Constraint::Percentage(35), // Response
        ])
        .split(outer_layout[0]);

    render_sidebar(frame, app, main_layout[0]);
    render_request_editor(frame, app, main_layout[1]);
    render_response(frame, app, main_layout[2]);
    render_status_bar(frame, app, outer_layout[1]);

    if app.show_help {
        render_help_overlay(frame, app, area);
    }
}

/// Render the sidebar (request list)
fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == Panel::Sidebar;
    let border_color = if is_focused {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER
    };

    let block = Block::default()
        .title(format!(" Requests ({}) ", app.requests.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build request list lines
    let placeholder = "https://api.example.com";
    let lines: Vec<Line> = app
        .requests
        .iter()
        .enumerate()
        .map(|(i, req)| {
            let is_selected = i == app.selected_request;
            let method_color = method_color(req.method);

            // Truncate URL to fit, show placeholder if empty
            let max_url_len = inner.width.saturating_sub(12) as usize;
            let (url_display, url_style) = if req.url.is_empty() {
                (placeholder.to_string(), theme::TEXT_DIM)
            } else if req.url.len() > max_url_len {
                (format!("{}...", &req.url[..max_url_len.saturating_sub(3)]), theme::TEXT)
            } else {
                (req.url.clone(), theme::TEXT)
            };

            let bg = if is_selected {
                theme::BG_HIGHLIGHT
            } else {
                theme::BG
            };

            let prefix = if is_selected { ">" } else { " " };

            Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme::ACCENT).bg(bg)),
                Span::styled(
                    format!("{:5}", req.method.as_str()),
                    Style::default().fg(method_color).bg(bg),
                ),
                Span::styled(url_display, Style::default().fg(url_style).bg(bg)),
                Span::styled(
                    format!(" {:>4}", req.relative_time()),
                    Style::default().fg(theme::TEXT_DIM).bg(bg),
                ),
            ])
        })
        .collect();

    let content = Paragraph::new(Text::from(lines)).style(Style::default().bg(theme::BG));

    frame.render_widget(content, inner);
}

fn method_color(method: HttpMethod) -> ratatui::style::Color {
    match method {
        HttpMethod::Get => theme::METHOD_GET,
        HttpMethod::Post => theme::METHOD_POST,
        HttpMethod::Put => theme::METHOD_PUT,
        HttpMethod::Patch => theme::METHOD_PATCH,
        HttpMethod::Delete => theme::METHOD_DELETE,
        HttpMethod::Head => theme::METHOD_HEAD,
        HttpMethod::Options => theme::METHOD_OPTIONS,
    }
}

/// Render the request editor panel
fn render_request_editor(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == Panel::RequestEditor;
    let border_color = if is_focused {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER
    };

    let title = if app.input_mode {
        " Request | [INSERT] "
    } else {
        " Request "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let method_color = method_color(app.input_method);
    let method_text = format!(" {} ", app.input_method.as_str());

    let placeholder = "https://api.example.com";
    let url_display = if app.input_mode {
        let (before, after) = app.input_url.split_at(app.cursor_position.min(app.input_url.len()));
        if app.input_url.is_empty() {
            vec![
                Span::styled(method_text, Style::default().fg(theme::BG).bg(method_color)),
                Span::styled(" ", Style::default()),
                Span::styled("│", Style::default().fg(theme::ACCENT)), // cursor
                Span::styled(placeholder, Style::default().fg(theme::TEXT_DIM)),
            ]
        } else {
            vec![
                Span::styled(method_text, Style::default().fg(theme::BG).bg(method_color)),
                Span::styled(" ", Style::default()),
                Span::styled(before.to_string(), Style::default().fg(theme::TEXT)),
                Span::styled("│", Style::default().fg(theme::ACCENT)), // cursor
                Span::styled(after.to_string(), Style::default().fg(theme::TEXT)),
            ]
        }
    } else if app.input_url.is_empty() {
        vec![
            Span::styled(method_text, Style::default().fg(theme::BG).bg(method_color)),
            Span::styled(" ", Style::default()),
            Span::styled(placeholder, Style::default().fg(theme::TEXT_DIM)),
        ]
    } else {
        vec![
            Span::styled(method_text, Style::default().fg(theme::BG).bg(method_color)),
            Span::styled(" ", Style::default()),
            Span::styled(&app.input_url, Style::default().fg(theme::TEXT)),
        ]
    };

    let url_bar = Paragraph::new(Line::from(url_display)).style(Style::default().bg(theme::BG));
    frame.render_widget(url_bar, inner);

    if is_focused && !app.input_mode {
        let hint_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        };
        let hint = Paragraph::new(Span::styled(
            "Press 'i' or Enter to insert",
            Style::default().fg(theme::TEXT_DIM),
        ));
        frame.render_widget(hint, hint_area);
    } else if app.input_mode {
        let hint_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        };
        let hint = Paragraph::new(Span::styled(
            "Tab: cycle method | Esc: exit insert mode",
            Style::default().fg(theme::TEXT_DIM),
        ));
        frame.render_widget(hint, hint_area);
    }
}

/// Render the response panel
fn render_response(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == Panel::Response;
    let border_color = if is_focused {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER
    };

    let title = match &app.request_state {
        RequestState::Idle => " Response ".to_string(),
        RequestState::Loading => " Response [Loading...] ".to_string(),
        RequestState::Success(resp) => {
            format!(" {} {} | {} | {} ", resp.status, resp.status_text, resp.elapsed_display(), resp.size_display())
        }
        RequestState::Error(_) => " Response [Error] ".to_string(),
    };

    let title_style = match &app.request_state {
        RequestState::Loading => Style::default().fg(theme::STATUS_LOADING),
        RequestState::Success(resp) => {
            let color = status_color(resp.status);
            Style::default().fg(color)
        }
        RequestState::Error(_) => Style::default().fg(theme::STATUS_ERROR),
        _ => Style::default().fg(theme::TEXT),
    };

    let block = Block::default()
        .title(title)
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &app.request_state {
        RequestState::Idle => {
            let placeholder = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    "No request sent",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::ITALIC),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press Ctrl+S to send",
                    Style::default().fg(theme::TEXT_DIM),
                )),
            ]))
            .centered()
            .style(Style::default().bg(theme::BG));
            frame.render_widget(placeholder, inner);
        }
        RequestState::Loading => {
            let loading = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    "Sending request...",
                    Style::default()
                        .fg(theme::STATUS_LOADING)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press Esc to cancel",
                    Style::default().fg(theme::TEXT_DIM),
                )),
            ]))
            .centered()
            .style(Style::default().bg(theme::BG));
            frame.render_widget(loading, inner);
        }
        RequestState::Success(resp) => {
            render_response_body(frame, app, inner, &resp.body);
        }
        RequestState::Error(err) => {
            let error_text = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Request Failed",
                    Style::default()
                        .fg(theme::STATUS_ERROR)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    err.as_str(),
                    Style::default().fg(theme::TEXT),
                )),
            ]))
            .centered()
            .style(Style::default().bg(theme::BG));
            frame.render_widget(error_text, inner);
        }
    }
}

/// Render response body with scrolling
fn render_response_body(frame: &mut Frame, app: &App, area: Rect, body: &str) {
    let formatted_body = if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string())
    } else {
        body.to_string()
    };

    let lines: Vec<Line> = formatted_body
        .lines()
        .skip(app.response_scroll)
        .take(area.height as usize)
        .map(|line| Line::from(Span::styled(line, Style::default().fg(theme::TEXT))))
        .collect();

    let content = Paragraph::new(Text::from(lines)).style(Style::default().bg(theme::BG));
    frame.render_widget(content, area);
}

/// Get color for HTTP status code
fn status_color(status: u16) -> ratatui::style::Color {
    match status {
        200..=299 => theme::STATUS_SUCCESS,
        300..=399 => theme::STATUS_REDIRECT,
        400..=499 => theme::STATUS_CLIENT_ERROR,
        500..=599 => theme::STATUS_SERVER_ERROR,
        _ => theme::TEXT_DIM,
    }
}

/// Render the status bar with keybinds
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default().fg(theme::TEXT_DIM);
    let sep_style = Style::default().fg(theme::BORDER);

    // Context aware hints
    let hints = if app.input_mode {
        vec![
            Span::styled("C-s", key_style),
            Span::styled(" send  ", sep_style),
            Span::styled("Tab", key_style),
            Span::styled(" method  ", sep_style),
            Span::styled("Esc", key_style),
            Span::styled(" done", sep_style),
        ]
    } else {
        match app.focused_panel {
            Panel::Sidebar => vec![
                Span::styled("j/k", key_style),
                Span::styled(" nav  ", sep_style),
                Span::styled("Enter", key_style),
                Span::styled(" insert  ", sep_style),
                Span::styled("n", key_style),
                Span::styled(" new  ", sep_style),
                Span::styled("d", key_style),
                Span::styled(" del", sep_style),
            ],
            Panel::RequestEditor => vec![
                Span::styled("C-s", key_style),
                Span::styled(" send  ", sep_style),
                Span::styled("i", key_style),
                Span::styled(" insert  ", sep_style),
                Span::styled("Tab", key_style),
                Span::styled(" next", sep_style),
            ],
            Panel::Response => vec![
                Span::styled("j/k", key_style),
                Span::styled(" scroll  ", sep_style),
                Span::styled("g/G", key_style),
                Span::styled(" top/end  ", sep_style),
                Span::styled("Tab", key_style),
                Span::styled(" next", sep_style),
            ],
        }
    };

    // Mode indicator
    let mode = if app.input_mode {
        Span::styled(" INSERT ", Style::default().fg(theme::BG).bg(theme::ACCENT))
    } else {
        Span::styled(" NORMAL ", Style::default().fg(theme::BG).bg(theme::TEXT_DIM))
    };

    // Left side: mode + hints
    let mut left_spans = vec![mode, Span::styled("  ", sep_style)];
    left_spans.extend(hints);

    // Right side: help + app name
    let right_spans = vec![
        Span::styled("?", key_style),
        Span::styled(" help  ", sep_style),
        Span::styled("q", key_style),
        Span::styled(" quit  ", sep_style),
        Span::styled("courier", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(" ", sep_style),
    ];
    let left_len: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_len: usize = right_spans.iter().map(|s| s.width()).sum();
    let padding = area.width.saturating_sub(left_len as u16 + right_len as u16) as usize;

    let mut all_spans = left_spans;
    all_spans.push(Span::styled(" ".repeat(padding), sep_style));
    all_spans.extend(right_spans);

    let status_bar = Paragraph::new(Line::from(all_spans))
        .style(Style::default().bg(theme::BG));

    frame.render_widget(status_bar, area);
}

/// Render the help overlay with keybinds
fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let help_width = 44;
    let help_height = 24;
    let help_area = Rect {
        x: area.width.saturating_sub(help_width) / 2,
        y: area.height.saturating_sub(help_height) / 2,
        width: help_width.min(area.width),
        height: help_height.min(area.height),
    };

    frame.render_widget(Clear, help_area);

    let section_style = Style::default()
        .fg(theme::TEXT_DIM)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default().fg(theme::ACCENT);
    let desc_style = Style::default().fg(theme::TEXT);

    let help_lines = vec![
        Line::from(Span::styled("NAVIGATION", section_style)),
        Line::from(vec![
            Span::styled("  Tab           ", key_style),
            Span::styled("Next panel", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab     ", key_style),
            Span::styled("Previous panel", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  h l           ", key_style),
            Span::styled("Left / Right panel", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  j k           ", key_style),
            Span::styled("Up / Down in list", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("REQUESTS", section_style)),
        Line::from(vec![
            Span::styled("  Ctrl+S        ", key_style),
            Span::styled("Send request", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Enter         ", key_style),
            Span::styled("Load request", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  i             ", key_style),
            Span::styled("Insert mode", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  n             ", key_style),
            Span::styled("New request", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  d             ", key_style),
            Span::styled("Delete request", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("RESPONSE", section_style)),
        Line::from(vec![
            Span::styled("  j k           ", key_style),
            Span::styled("Scroll up / down", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  g G           ", key_style),
            Span::styled("Jump to top / bottom", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("INSERT MODE", section_style)),
        Line::from(vec![
            Span::styled("  Ctrl+←/→      ", key_style),
            Span::styled("Move by word", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+A/E      ", key_style),
            Span::styled("Start / End of line", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+W        ", key_style),
            Span::styled("Delete word", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Tab           ", key_style),
            Span::styled("Cycle HTTP method", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Esc           ", key_style),
            Span::styled("Exit insert mode", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("GENERAL", section_style)),
        Line::from(vec![
            Span::styled("  ?             ", key_style),
            Span::styled("Toggle help", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  q             ", key_style),
            Span::styled("Quit", desc_style),
        ]),
    ];

    let total_lines = help_lines.len();
    let visible_height = help_area.height.saturating_sub(2) as usize; // Subtract border
    let can_scroll = total_lines > visible_height;

    let title = if can_scroll {
        format!(
            " Help [{}/{}] ",
            app.help_scroll + 1,
            total_lines.saturating_sub(visible_height) + 1
        )
    } else {
        " Courier Help ".to_string()
    };

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .title(title)
                .title_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .style(Style::default().bg(theme::BG_HIGHLIGHT)),
        )
        .scroll((app.help_scroll as u16, 0));

    frame.render_widget(help, help_area);
}
