use tui_textarea::TextArea;

pub fn format_json_if_valid(text: &str) -> String {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|json| serde_json::to_string_pretty(&json).ok())
        .unwrap_or_else(|| text.to_string())
}

pub fn scroll_by(pos: &mut usize, delta: isize, max: usize) {
    if delta < 0 {
        *pos = pos.saturating_sub((-delta) as usize);
    } else if max > 0 {
        *pos = (*pos + delta as usize).min(max.saturating_sub(1));
    }
}

pub fn single_line_textarea(initial: &str) -> TextArea<'static> {
    let mut textarea = TextArea::new(vec![initial.to_string()]);
    textarea.set_cursor_line_style(ratatui::style::Style::default());
    textarea
}

pub fn textarea_value<'a>(textarea: &'a TextArea<'a>) -> &'a str {
    textarea.lines().first().map(|s| s.as_str()).unwrap_or("")
}
