use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, Panel};
use crate::models::HttpMethod;

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
        render_help_overlay(frame, area);
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
    let lines: Vec<Line> = app
        .requests
        .iter()
        .enumerate()
        .map(|(i, req)| {
            let is_selected = i == app.selected_request;
            let method_color = method_color(req.method);

            // Truncate URL to fit
            let max_url_len = inner.width.saturating_sub(12) as usize;
            let url_display = if req.url.len() > max_url_len {
                format!("{}...", &req.url[..max_url_len.saturating_sub(3)])
            } else {
                req.url.clone()
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
                Span::styled(url_display, Style::default().fg(theme::TEXT).bg(bg)),
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
        _ => theme::TEXT_DIM,
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
        " Request [EDIT] "
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

    // Show input URL bar
    let method_color = method_color(app.input_method);
    let method_text = format!(" {} ", app.input_method.as_str());

    // Build URL display with cursor.
    // Cursor is shown when input mode is active.
    let url_display = if app.input_mode {
        let (before, after) = app.input_url.split_at(app.cursor_position.min(app.input_url.len()));
        vec![
            Span::styled(method_text, Style::default().fg(theme::BG).bg(method_color)),
            Span::styled(" ", Style::default()),
            Span::styled(before.to_string(), Style::default().fg(theme::TEXT)),
            Span::styled("│", Style::default().fg(theme::ACCENT)), // cursor
            Span::styled(after.to_string(), Style::default().fg(theme::TEXT)),
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
            "Press 'i' or Enter to edit",
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
            "Tab: cycle method | Esc: exit edit mode",
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

    let block = Block::default()
        .title(" Response ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let placeholder = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Not sent",
            Style::default()
                .fg(theme::TEXT_DIM)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to send request",
            Style::default().fg(theme::TEXT_DIM),
        )),
    ]))
    .centered()
    .style(Style::default().bg(theme::BG));

    frame.render_widget(placeholder, inner);
}

/// Render the status bar with keybinds
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default().fg(theme::TEXT_DIM);
    let sep_style = Style::default().fg(theme::BORDER);

    // Context aware hints
    let hints = if app.input_mode {
        vec![
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
                Span::styled(" edit  ", sep_style),
                Span::styled("n", key_style),
                Span::styled(" new  ", sep_style),
                Span::styled("d", key_style),
                Span::styled(" del", sep_style),
            ],
            Panel::RequestEditor => vec![
                Span::styled("i", key_style),
                Span::styled(" edit  ", sep_style),
                Span::styled("Tab", key_style),
                Span::styled(" next", sep_style),
            ],
            Panel::Response => vec![
                Span::styled("Tab", key_style),
                Span::styled(" next", sep_style),
            ],
        }
    };

    // Mode indicator
    let mode = if app.input_mode {
        Span::styled(" EDIT ", Style::default().fg(theme::BG).bg(theme::ACCENT))
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
fn render_help_overlay(frame: &mut Frame, area: Rect) {
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

    let help_text = vec![
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
            Span::styled("  Enter         ", key_style),
            Span::styled("Load / Edit request", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  i             ", key_style),
            Span::styled("Edit URL", desc_style),
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
        Line::from(Span::styled("EDIT MODE", section_style)),
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
            Span::styled("  Ctrl+U/K      ", key_style),
            Span::styled("Delete to start/end", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Tab           ", key_style),
            Span::styled("Cycle HTTP method", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Esc           ", key_style),
            Span::styled("Exit edit mode", desc_style),
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

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Courier Help ")
                .title_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .style(Style::default().bg(theme::BG_HIGHLIGHT)),
        );

    frame.render_widget(help, help_area);
}
