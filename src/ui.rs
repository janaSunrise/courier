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

    let block = Block::default()
        .title(" Request ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show currently selected request
    if let Some(req) = app.current_request() {
        let method_color = method_color(req.method);
        let url_bar = Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {} ", req.method.as_str()),
                Style::default().fg(theme::BG).bg(method_color),
            ),
            Span::styled(" ", Style::default()),
            Span::styled(&req.url, Style::default().fg(theme::TEXT)),
        ]))
        .style(Style::default().bg(theme::BG));

        frame.render_widget(url_bar, inner);
    } else {
        let empty = Paragraph::new(Span::styled(
            "No request selected",
            Style::default().fg(theme::TEXT_DIM),
        ))
        .style(Style::default().bg(theme::BG));
        frame.render_widget(empty, inner);
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
    let help_width = 50;
    let help_height = 16;
    let help_area = Rect {
        x: area.width.saturating_sub(help_width) / 2,
        y: area.height.saturating_sub(help_height) / 2,
        width: help_width.min(area.width),
        height: help_height.min(area.height),
    };

    frame.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab / Shift+Tab  ", Style::default().fg(theme::ACCENT)),
            Span::styled("Cycle panels", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("h / l            ", Style::default().fg(theme::ACCENT)),
            Span::styled("Previous / Next panel", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("j / k            ", Style::default().fg(theme::ACCENT)),
            Span::styled("Navigate requests", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("n                ", Style::default().fg(theme::ACCENT)),
            Span::styled("New request", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("d                ", Style::default().fg(theme::ACCENT)),
            Span::styled("Delete request", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("?                ", Style::default().fg(theme::ACCENT)),
            Span::styled("Toggle this help", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("q                ", Style::default().fg(theme::ACCENT)),
            Span::styled("Quit", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(theme::TEXT_DIM),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::ACCENT))
                .style(Style::default().bg(theme::BG_HIGHLIGHT)),
        )
        .centered();

    frame.render_widget(help, help_area);
}
