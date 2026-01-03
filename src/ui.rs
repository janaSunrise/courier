use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, Panel};

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
        .title(" Requests ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("GET  ", Style::default().fg(theme::METHOD_GET)),
            Span::styled("/api/users", Style::default().fg(theme::TEXT)),
            Span::styled("  10m", Style::default().fg(theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("POST ", Style::default().fg(theme::METHOD_POST)),
            Span::styled("/api/auth", Style::default().fg(theme::TEXT)),
            Span::styled("  2h", Style::default().fg(theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("DEL  ", Style::default().fg(theme::METHOD_DELETE)),
            Span::styled("/api/users/1", Style::default().fg(theme::TEXT)),
            Span::styled("  1d", Style::default().fg(theme::TEXT_DIM)),
        ]),
    ]))
    .style(Style::default().bg(theme::BG));

    frame.render_widget(content, inner);
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

    let url_bar = Paragraph::new(Line::from(vec![
        Span::styled(
            " GET ",
            Style::default().fg(theme::BG).bg(theme::METHOD_GET),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            "https://api.example.com/users",
            Style::default().fg(theme::TEXT),
        ),
    ]))
    .style(Style::default().bg(theme::BG));

    frame.render_widget(url_bar, inner);
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
    // Center the help box
    let help_width = 50;
    let help_height = 12;
    let help_area = Rect {
        x: area.width.saturating_sub(help_width) / 2,
        y: area.height.saturating_sub(help_height) / 2,
        width: help_width.min(area.width),
        height: help_height.min(area.height),
    };

    // Clear the area behind the popup
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
