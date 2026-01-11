use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph, Tabs},
};

use crate::app::{App, EditFocus, KvField, KvEditor, Panel, RequestTab};
use crate::models::{HttpMethod, KeyValue, Request, RequestState};
use crate::utils::{format_json_if_valid, textarea_value};

pub mod theme {
    use ratatui::style::Color;

    pub const BG: Color = Color::Rgb(16, 20, 30);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(30, 36, 50);
    pub const BORDER: Color = Color::Rgb(55, 65, 85);
    pub const BORDER_FOCUSED: Color = Color::Rgb(139, 92, 246);
    pub const TEXT: Color = Color::Rgb(226, 232, 240);
    pub const TEXT_DIM: Color = Color::Rgb(100, 116, 139);
    pub const ACCENT: Color = Color::Rgb(139, 92, 246);
    pub const ERROR: Color = Color::Rgb(251, 113, 133);

    pub const METHOD_GET: Color = Color::Rgb(52, 211, 153);
    pub const METHOD_POST: Color = Color::Rgb(251, 191, 36);
    pub const METHOD_PUT: Color = Color::Rgb(96, 165, 250);
    pub const METHOD_PATCH: Color = Color::Rgb(192, 132, 252);
    pub const METHOD_DELETE: Color = Color::Rgb(251, 113, 133);
    pub const METHOD_HEAD: Color = Color::Rgb(94, 234, 212);
    pub const METHOD_OPTIONS: Color = Color::Rgb(156, 163, 175);

    pub const STATUS_SUCCESS: Color = Color::Rgb(52, 211, 153);
    pub const STATUS_REDIRECT: Color = Color::Rgb(96, 165, 250);
    pub const STATUS_CLIENT_ERROR: Color = Color::Rgb(251, 191, 36);
    pub const STATUS_SERVER_ERROR: Color = Color::Rgb(251, 113, 133);
    pub const STATUS_LOADING: Color = Color::Rgb(139, 92, 246);
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(40),
            Constraint::Percentage(35),
        ])
        .split(outer[0]);

    render_sidebar(frame, app, main[0]);
    render_request_editor(frame, app, main[1]);
    render_response(frame, app, main[2]);
    render_status_bar(frame, app, outer[1]);

    if app.show_help {
        render_help_overlay(frame, app, area);
    }
}

fn create_request_list_item<'a>(req: &Request, max_url_len: usize) -> ListItem<'a> {
    let placeholder = "https://api.example.com";

    let (url_text, url_color) = if req.url.is_empty() {
        (placeholder.to_string(), theme::TEXT_DIM)
    } else if req.url.len() > max_url_len {
        (format!("{}...", &req.url[..max_url_len.saturating_sub(3)]), theme::TEXT)
    } else {
        (req.url.clone(), theme::TEXT)
    };

    let line = Line::from(vec![
        Span::styled(
            format!("{:6}", req.method.as_str()),
            Style::default().fg(method_color(req.method)),
        ),
        Span::styled(url_text, Style::default().fg(url_color)),
        Span::styled(
            format!(" {:>4}", req.relative_time()),
            Style::default().fg(theme::TEXT_DIM),
        ),
    ]);

    ListItem::new(line)
}

fn render_sidebar(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focused_panel == Panel::Sidebar;
    let border_color = if focused { theme::BORDER_FOCUSED } else { theme::BORDER };

    let block = Block::default()
        .title(format!(" Requests ({}) ", app.requests.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.requests.is_empty() {
        let hint = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("No requests", Style::default().fg(theme::TEXT_DIM))),
            Line::from(""),
            Line::from(Span::styled("Press 'n' to create", Style::default().fg(theme::TEXT_DIM))),
        ]))
        .centered()
        .style(Style::default().bg(theme::BG));
        frame.render_widget(hint, inner);
        return;
    }

    let max_url_len = inner.width.saturating_sub(14) as usize;

    let items: Vec<ListItem> = app.requests
        .iter()
        .map(|req| create_request_list_item(req, max_url_len))
        .collect();

    let list = List::new(items)
        .style(Style::default().bg(theme::BG).fg(theme::TEXT))
        .highlight_style(
            Style::default()
                .bg(theme::BG_HIGHLIGHT)
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, inner, &mut app.sidebar_state);
}

fn render_request_editor(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focused_panel == Panel::RequestEditor;
    let border = if focused { theme::BORDER_FOCUSED } else { theme::BORDER };

    let right_title: Line = match app.edit_focus {
        EditFocus::Url => Line::from(Span::styled(" URL ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))),
        EditFocus::KeyValue => {
            let label = match app.active_tab {
                RequestTab::Params => "PARAMS",
                RequestTab::Headers => "HEADERS",
                RequestTab::Body => "BODY",
            };
            Line::from(Span::styled(format!(" {} ", label), Style::default().fg(theme::METHOD_POST).add_modifier(Modifier::BOLD)))
        },
        EditFocus::Body => Line::from(Span::styled(" BODY ", Style::default().fg(theme::METHOD_PUT).add_modifier(Modifier::BOLD))),
        EditFocus::None => Line::from(""),
    };

    let block = Block::default()
        .title(" Request ")
        .title(right_title.alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    render_url_bar(frame, app, layout[0]);
    render_tabs(frame, app, layout[1]);
    render_tab_content(frame, app, layout[2]);
}

fn render_url_bar(frame: &mut Frame, app: &App, area: Rect) {
    let method_color = method_color(app.method);
    let method_text = format!(" {} ", app.method.as_str());
    let method_width = method_text.len() as u16 + 1; // Single space after method

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(method_width), Constraint::Min(0)])
        .split(area);

    let method_span = Span::styled(method_text, Style::default().fg(theme::BG).bg(method_color));
    frame.render_widget(Paragraph::new(Line::from(method_span)).style(Style::default().bg(theme::BG)), chunks[0]);

    if app.edit_focus == EditFocus::Url {
        frame.render_widget(&app.url_input, chunks[1]);
    } else {
        let placeholder = "https://api.example.com";
        let url = app.url();
        let url_text = if url.is_empty() { placeholder } else { url };
        let url_color = if url.is_empty() { theme::TEXT_DIM } else { theme::TEXT };
        let url_para = Paragraph::new(Span::styled(url_text, Style::default().fg(url_color)))
            .style(Style::default().bg(theme::BG));
        frame.render_widget(url_para, chunks[1]);
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let tabs = ["Params", "Headers", "Body"];
    let selected = match app.active_tab {
        RequestTab::Params => 0,
        RequestTab::Headers => 1,
        RequestTab::Body => 2,
    };

    let tab_titles: Vec<Line> = tabs.iter().map(|t| Line::from(*t)).collect();

    let tabs_widget = Tabs::new(tab_titles)
        .select(selected)
        .style(Style::default().fg(theme::TEXT_DIM))
        .highlight_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
        .divider("│");

    frame.render_widget(tabs_widget, area);
}

fn render_tab_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        RequestTab::Params => render_kv_list(frame, app, area, &app.params, &app.params_editor),
        RequestTab::Headers => render_kv_list(frame, app, area, &app.headers, &app.headers_editor),
        RequestTab::Body => render_body_editor(frame, app, area),
    }
}

fn render_kv_list(frame: &mut Frame, app: &App, area: Rect, items: &[KeyValue], editor: &KvEditor) {
    if items.is_empty() {
        let hint = Paragraph::new(Span::styled(
            "Press 'a' to add",
            Style::default().fg(theme::TEXT_DIM),
        ))
        .centered();
        frame.render_widget(hint, area);
        return;
    }

    let is_editing = app.edit_focus == EditFocus::KeyValue;

    for (i, item) in items.iter().enumerate() {
        if i >= area.height as usize {
            break;
        }

        let row_area = Rect {
            x: area.x,
            y: area.y + i as u16,
            width: area.width,
            height: 1,
        };

        let selected = i == editor.selected();
        let bg = if selected { theme::BG_HIGHLIGHT } else { theme::BG };

        frame.render_widget(Paragraph::new("").style(Style::default().bg(bg)), row_area);

        // When editing the selected row, use layout for TextArea widgets
        // When not editing, no extra padding is added between key and value.
        if selected && is_editing {
            render_kv_row_editing(frame, editor, item, row_area, bg);
        } else {
            render_kv_row_static(frame, item, selected, row_area, bg);
        }
    }
}

fn render_kv_row_static(frame: &mut Frame, item: &KeyValue, selected: bool, area: Rect, bg: ratatui::style::Color) {
    let prefix = if selected { "› " } else { "  " };
    let checkbox = if item.enabled { "[✓] " } else { "[ ] " };
    let checkbox_color = if item.enabled { theme::METHOD_GET } else { theme::TEXT_DIM };
    let key_color = if selected { theme::ACCENT } else { theme::TEXT };

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(theme::ACCENT).bg(bg)),
        Span::styled(checkbox, Style::default().fg(checkbox_color).bg(bg)),
        Span::styled(&item.key, Style::default().fg(key_color).bg(bg)),
        Span::styled(": ", Style::default().fg(theme::TEXT_DIM).bg(bg)),
        Span::styled(&item.value, Style::default().fg(theme::TEXT).bg(bg)),
    ]);

    frame.render_widget(Paragraph::new(line).style(Style::default().bg(bg)), area);
}

fn render_kv_row_editing(frame: &mut Frame, editor: &KvEditor, item: &KeyValue, area: Rect, bg: ratatui::style::Color) {
    // Layout: prefix + checkbox (6) | key input | colon (3) | value input
    let prefix_width = 6u16; // "› [✓] "
    let colon_width = 3u16;
    let remaining = area.width.saturating_sub(prefix_width + colon_width);
    let input_width = remaining / 2;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(prefix_width),
            Constraint::Length(input_width),
            Constraint::Length(colon_width),
            Constraint::Min(0),
        ])
        .split(area);

    let checkbox = if item.enabled { "[✓] " } else { "[ ] " };
    let checkbox_color = if item.enabled { theme::METHOD_GET } else { theme::TEXT_DIM };
    let prefix_line = Line::from(vec![
        Span::styled("› ", Style::default().fg(theme::ACCENT).bg(bg)),
        Span::styled(checkbox, Style::default().fg(checkbox_color).bg(bg)),
    ]);
    frame.render_widget(Paragraph::new(prefix_line).style(Style::default().bg(bg)), chunks[0]);

    match editor.field {
        KvField::Key => {
            frame.render_widget(&editor.key_input, chunks[1]);
            let val = textarea_value(&editor.value_input);
            frame.render_widget(Paragraph::new(val).style(Style::default().fg(theme::TEXT).bg(bg)), chunks[3]);
        }
        KvField::Value => {
            let key = textarea_value(&editor.key_input);
            frame.render_widget(Paragraph::new(key).style(Style::default().fg(theme::TEXT).bg(bg)), chunks[1]);
            frame.render_widget(&editor.value_input, chunks[3]);
        }
    }

    frame.render_widget(Paragraph::new(" : ").style(Style::default().fg(theme::TEXT_DIM).bg(bg)), chunks[2]);
}

fn render_body_editor(frame: &mut Frame, app: &App, area: Rect) {
    let is_editing = app.edit_focus == EditFocus::Body;
    let body_text = app.body();

    if let Some(ref err) = app.json_error {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        render_body_content(frame, app, layout[0], is_editing, &body_text);

        let error = Paragraph::new(Span::styled(err, Style::default().fg(theme::ERROR)))
            .style(Style::default().bg(theme::BG));
        frame.render_widget(error, layout[1]);
    } else {
        render_body_content(frame, app, area, is_editing, &body_text);
    }
}

fn render_body_content(frame: &mut Frame, app: &App, area: Rect, is_editing: bool, body_text: &str) {
    if body_text.is_empty() && !is_editing {
        let hint = Paragraph::new(Span::styled(
            "Press 'e' to edit body (Ctrl+F to format JSON)",
            Style::default().fg(theme::TEXT_DIM),
        ))
        .centered();
        frame.render_widget(hint, area);
        return;
    }

    if is_editing {
        frame.render_widget(&app.body_editor, area);
    } else {
        let content = format_json_if_valid(body_text);
        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(theme::TEXT).bg(theme::BG));
        frame.render_widget(paragraph, area);
    }
}

fn render_response(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focused_panel == Panel::Response;
    let border = if focused { theme::BORDER_FOCUSED } else { theme::BORDER };

    let right_title: Line = match &app.request_state {
        RequestState::Idle => Line::from(""),
        RequestState::Loading => Line::from(Span::styled(" ● Loading ", Style::default().fg(theme::STATUS_LOADING))),
        RequestState::Success(resp) => {
            let status_col = status_color(resp.status);
            Line::from(vec![
                Span::styled(format!(" {} {} ", resp.status, resp.status_text), Style::default().fg(theme::BG).bg(status_col).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  {}  {} ", resp.elapsed_display(), resp.size_display()), Style::default().fg(theme::TEXT_DIM)),
            ])
        },
        RequestState::Error(_) => Line::from(Span::styled(" ✕ Error ", Style::default().fg(theme::BG).bg(theme::STATUS_SERVER_ERROR).add_modifier(Modifier::BOLD))),
    };

    let block = Block::default()
        .title(" Response ")
        .title(right_title.alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &app.request_state {
        RequestState::Idle => {
            let text = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(Span::styled("No request sent", Style::default().fg(theme::TEXT_DIM).add_modifier(Modifier::ITALIC))),
                Line::from(""),
                Line::from(Span::styled("Press Ctrl+S to send", Style::default().fg(theme::TEXT_DIM))),
            ]))
            .centered();
            frame.render_widget(text, inner);
        }
        RequestState::Loading => {
            let text = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(Span::styled("Sending request...", Style::default().fg(theme::STATUS_LOADING).add_modifier(Modifier::BOLD))),
            ]))
            .centered();
            frame.render_widget(text, inner);
        }
        RequestState::Success(resp) => {
            let formatted = resp.formatted_body();
            let lines: Vec<Line> = formatted
                .lines()
                .skip(app.response_scroll)
                .take(inner.height as usize)
                .map(|l| Line::from(Span::styled(l, Style::default().fg(theme::TEXT))))
                .collect();

            frame.render_widget(Paragraph::new(Text::from(lines)).style(Style::default().bg(theme::BG)), inner);
        }
        RequestState::Error(err) => {
            let text = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(Span::styled("Request Failed", Style::default().fg(theme::STATUS_SERVER_ERROR).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled(err.as_str(), Style::default().fg(theme::TEXT))),
            ]))
            .centered();
            frame.render_widget(text, inner);
        }
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let key = Style::default().fg(theme::TEXT);
    let desc = Style::default().fg(theme::TEXT_DIM);
    let dim = Style::default().fg(theme::BORDER);

    let mode = match app.edit_focus {
        EditFocus::None => Span::styled(" NORMAL ", Style::default().fg(theme::BG).bg(theme::TEXT_DIM)),
        EditFocus::Url => Span::styled(" INSERT ", Style::default().fg(theme::BG).bg(theme::ACCENT)),
        EditFocus::KeyValue => Span::styled(" INSERT ", Style::default().fg(theme::BG).bg(theme::METHOD_POST)),
        EditFocus::Body => Span::styled(" INSERT ", Style::default().fg(theme::BG).bg(theme::METHOD_PUT)),
    };

    let hints: Vec<Span> = if app.edit_focus == EditFocus::Body {
        vec![
            Span::styled("esc", key), Span::styled(":done ", desc),
            Span::styled("C-F", key), Span::styled(":fmt ", desc),
            Span::styled("C-S", key), Span::styled(":send", desc),
        ]
    } else if app.is_editing() {
        vec![
            Span::styled("Esc", key), Span::styled(":done ", desc),
            Span::styled("C-S", key), Span::styled(":send", desc),
        ]
    } else {
        match app.focused_panel {
            Panel::Sidebar => vec![
                Span::styled("j/k", key), Span::styled(":nav ", desc),
                Span::styled("enter", key), Span::styled(":select ", desc),
                Span::styled("n", key), Span::styled(":new ", desc),
                Span::styled("d", key), Span::styled(":del", desc),
            ],
            Panel::RequestEditor => vec![
                Span::styled("i", key), Span::styled(":url ", desc),
                Span::styled("1-3", key), Span::styled(":tab ", desc),
                Span::styled("a", key), Span::styled(":add ", desc),
                Span::styled("C-S", key), Span::styled(":send", desc),
            ],
            Panel::Response => vec![
                Span::styled("j/k", key), Span::styled(":scroll ", desc),
                Span::styled("g/G", key), Span::styled(":jump", desc),
            ],
        }
    };

    let right = vec![
        Span::styled("?", key), Span::styled(":help ", desc),
        Span::styled("q", key), Span::styled(":quit ", desc),
        Span::styled("│ ", dim),
        Span::styled("courier", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
    ];

    let mut left: Vec<Span> = vec![mode, Span::styled(" ", desc)];
    left.extend(hints);

    let left_len: usize = left.iter().map(|s| s.width()).sum();
    let right_len: usize = right.iter().map(|s| s.width()).sum();
    let padding = area.width.saturating_sub(left_len as u16 + right_len as u16) as usize;

    let mut all = left;
    all.push(Span::styled(" ".repeat(padding), desc));
    all.extend(right);

    frame.render_widget(Paragraph::new(Line::from(all)).style(Style::default().bg(theme::BG)), area);
}

fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let (w, h) = (50, 26);
    let help_area = Rect {
        x: area.width.saturating_sub(w) / 2,
        y: area.height.saturating_sub(h) / 2,
        width: w.min(area.width),
        height: h.min(area.height),
    };

    frame.render_widget(Clear, help_area);

    let section = Style::default().fg(theme::TEXT_DIM).add_modifier(Modifier::BOLD);
    let k = Style::default().fg(theme::ACCENT);
    let d = Style::default().fg(theme::TEXT);

    let lines = vec![
        Line::from(Span::styled("NAVIGATION", section)),
        Line::from(vec![Span::styled("  Tab/h/l       ", k), Span::styled("Switch panels", d)]),
        Line::from(vec![Span::styled("  j/k           ", k), Span::styled("Navigate list/scroll", d)]),
        Line::from(vec![Span::styled("  1/2/3         ", k), Span::styled("Switch tabs", d)]),
        Line::from(""),
        Line::from(Span::styled("REQUESTS", section)),
        Line::from(vec![Span::styled("  Ctrl+S        ", k), Span::styled("Send request", d)]),
        Line::from(vec![Span::styled("  i             ", k), Span::styled("Edit URL", d)]),
        Line::from(vec![Span::styled("  a             ", k), Span::styled("Add param/header", d)]),
        Line::from(vec![Span::styled("  e             ", k), Span::styled("Edit body", d)]),
        Line::from(vec![Span::styled("  Enter         ", k), Span::styled("Edit selected item", d)]),
        Line::from(vec![Span::styled("  n             ", k), Span::styled("New request", d)]),
        Line::from(vec![Span::styled("  d             ", k), Span::styled("Delete item/request", d)]),
        Line::from(""),
        Line::from(Span::styled("BODY EDITING", section)),
        Line::from(vec![Span::styled("  Ctrl+F        ", k), Span::styled("Format JSON", d)]),
        Line::from(vec![Span::styled("  Esc           ", k), Span::styled("Stop editing", d)]),
        Line::from(""),
        Line::from(Span::styled("GENERAL", section)),
        Line::from(vec![Span::styled("  ?             ", k), Span::styled("Toggle help", d)]),
        Line::from(vec![Span::styled("  q             ", k), Span::styled("Quit", d)]),
    ];

    let help = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .style(Style::default().bg(theme::BG_HIGHLIGHT)),
        )
        .scroll((app.help_scroll as u16, 0));

    frame.render_widget(help, help_area);
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

fn status_color(status: u16) -> ratatui::style::Color {
    match status {
        200..=299 => theme::STATUS_SUCCESS,
        300..=399 => theme::STATUS_REDIRECT,
        400..=499 => theme::STATUS_CLIENT_ERROR,
        _ => theme::STATUS_SERVER_ERROR,
    }
}
