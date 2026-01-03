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

    pub const BG: Color = Color::Rgb(26, 26, 46);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(40, 40, 60);
    pub const BORDER: Color = Color::Rgb(58, 58, 74);
    pub const BORDER_FOCUSED: Color = Color::Rgb(115, 210, 22);
    pub const TEXT: Color = Color::Rgb(224, 224, 224);
    pub const TEXT_DIM: Color = Color::Rgb(128, 128, 140);
    pub const ACCENT: Color = Color::Rgb(115, 210, 22);
    pub const METHOD_GET: Color = Color::Rgb(115, 210, 22);
    pub const METHOD_POST: Color = Color::Rgb(252, 186, 3);
    pub const METHOD_PUT: Color = Color::Rgb(88, 166, 255);
    pub const METHOD_PATCH: Color = Color::Rgb(163, 113, 247);
    pub const METHOD_DELETE: Color = Color::Rgb(252, 78, 78);
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
    // Main layout: end
    render_status_bar(frame, outer_layout[1]);
    // Vertical layout: end

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
fn render_status_bar(frame: &mut Frame, area: Rect) {
    let status = Line::from(vec![
        Span::styled(" q ", Style::default().fg(theme::BG).bg(theme::TEXT_DIM)),
        Span::styled(" Quit ", Style::default().fg(theme::TEXT_DIM)),
        Span::styled(" ? ", Style::default().fg(theme::BG).bg(theme::TEXT_DIM)),
        Span::styled(" Help ", Style::default().fg(theme::TEXT_DIM)),
        Span::styled(" Tab ", Style::default().fg(theme::BG).bg(theme::TEXT_DIM)),
        Span::styled(" Switch Panel ", Style::default().fg(theme::TEXT_DIM)),
    ]);

    let status_bar = Paragraph::new(status).style(Style::default().bg(theme::BG));

    frame.render_widget(status_bar, area);
}

/// Render the help overlay with keybinds
fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let help_width = 44;
    let help_height = 20;
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
