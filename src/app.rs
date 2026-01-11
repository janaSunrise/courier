use tui_textarea::TextArea;
use ratatui::widgets::ListState;

use crate::models::{HttpMethod, KeyValue, Request, RequestState, Response};
use crate::utils::{scroll_by, single_line_textarea, textarea_value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    #[default]
    Sidebar,
    RequestEditor,
    Response,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Sidebar => Panel::RequestEditor,
            Panel::RequestEditor => Panel::Response,
            Panel::Response => Panel::Sidebar,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Panel::Sidebar => Panel::Response,
            Panel::RequestEditor => Panel::Sidebar,
            Panel::Response => Panel::RequestEditor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RequestTab {
    #[default]
    Params,
    Headers,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditFocus {
    #[default]
    None,
    Url,
    KeyValue,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KvField {
    #[default]
    Key,
    Value,
}

impl KvField {
    pub fn toggle(self) -> Self {
        match self {
            KvField::Key => KvField::Value,
            KvField::Value => KvField::Key,
        }
    }
}

pub struct KvEditor {
    pub state: ListState,
    pub field: KvField,
    pub key_input: TextArea<'static>,
    pub value_input: TextArea<'static>,
}

impl Default for KvEditor {
    fn default() -> Self {
        Self {
            state: ListState::default(),
            field: KvField::Key,
            key_input: single_line_textarea(""),
            value_input: single_line_textarea(""),
        }
    }
}

impl KvEditor {
    pub fn reset(&mut self) {
        self.state = ListState::default();
        self.field = KvField::Key;
        self.key_input = single_line_textarea("");
        self.value_input = single_line_textarea("");
    }

    pub fn selected(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    pub fn select(&mut self, index: usize) {
        self.state.select(Some(index));
    }

    pub fn select_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_prev(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => if i == 0 { len - 1 } else { i - 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn toggle_field(&mut self) {
        self.field = self.field.toggle();
    }

    pub fn current_input_mut(&mut self) -> &mut TextArea<'static> {
        match self.field {
            KvField::Key => &mut self.key_input,
            KvField::Value => &mut self.value_input,
        }
    }

    pub fn sync_from_item(&mut self, item: &KeyValue) {
        self.key_input = single_line_textarea(&item.key);
        self.value_input = single_line_textarea(&item.value);
    }
}

pub struct App<'a> {
    // UI state
    pub focused_panel: Panel,
    pub should_quit: bool,
    pub show_help: bool,
    pub help_scroll: usize,

    // Sidebar
    pub requests: Vec<Request>,
    pub sidebar_state: ListState,
    pub editing_request_idx: Option<usize>,

    // Request editor
    pub active_tab: RequestTab,
    pub edit_focus: EditFocus,

    // URL
    pub url_input: TextArea<'static>,
    pub method: HttpMethod,

    // Params & Headers
    pub params: Vec<KeyValue>,
    pub params_editor: KvEditor,
    pub headers: Vec<KeyValue>,
    pub headers_editor: KvEditor,

    // Body
    pub body_editor: TextArea<'a>,
    pub json_error: Option<String>,

    // Response
    pub request_state: RequestState,
    pub response_scroll: usize,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut body_editor = TextArea::default();
        body_editor.set_cursor_line_style(ratatui::style::Style::default());

        Self {
            focused_panel: Panel::default(),
            should_quit: false,
            show_help: false,
            help_scroll: 0,
            requests: vec![],
            sidebar_state: ListState::default(),
            editing_request_idx: None,
            active_tab: RequestTab::default(),
            edit_focus: EditFocus::None,
            url_input: single_line_textarea(""),
            method: HttpMethod::Get,
            params: vec![],
            params_editor: KvEditor::default(),
            headers: vec![],
            headers_editor: KvEditor::default(),
            body_editor,
            json_error: None,
            request_state: RequestState::default(),
            response_scroll: 0,
        }
    }

    pub fn url(&self) -> &str {
        textarea_value(&self.url_input)
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.help_scroll = 0;
        }
    }

    pub fn is_editing(&self) -> bool {
        self.edit_focus != EditFocus::None
    }

    pub fn body(&self) -> String {
        self.body_editor.lines().join("\n")
    }

    pub fn set_body(&mut self, text: &str) {
        self.body_editor = TextArea::new(text.lines().map(String::from).collect());
        self.body_editor.set_cursor_line_style(ratatui::style::Style::default());
        self.validate_json();
    }

    pub fn format_json(&mut self) {
        let text = self.body();
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
            && let Ok(formatted) = serde_json::to_string_pretty(&value)
        {
            self.set_body(&formatted);
            self.json_error = None;
        }
    }

    pub fn validate_json(&mut self) {
        let text = self.body();
        if text.trim().is_empty() {
            self.json_error = None;
        } else {
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(_) => self.json_error = None,
                Err(e) => self.json_error = Some(format!("JSON: {}", e)),
            }
        }
    }

    // Panel navigation
    pub fn focus_next_panel(&mut self) {
        self.focused_panel = self.focused_panel.next();
    }

    pub fn focus_prev_panel(&mut self) {
        self.focused_panel = self.focused_panel.prev();
    }

    // Sidebar
    pub fn selected_request(&self) -> usize {
        self.sidebar_state.selected().unwrap_or(0)
    }

    pub fn select_next_request(&mut self) {
        if self.requests.is_empty() {
            return;
        }
        let i = match self.sidebar_state.selected() {
            Some(i) => (i + 1) % self.requests.len(),
            None => 0,
        };
        self.sidebar_state.select(Some(i));
    }

    pub fn select_prev_request(&mut self) {
        if self.requests.is_empty() {
            return;
        }
        let i = match self.sidebar_state.selected() {
            Some(i) => if i == 0 { self.requests.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.sidebar_state.select(Some(i));
    }

    pub fn add_request(&mut self, request: Request) {
        self.requests.insert(0, request);
        self.sidebar_state.select(Some(0));
        self.editing_request_idx = Some(0);
    }

    pub fn new_request(&mut self) {
        self.requests.insert(0, Request::default());
        self.sidebar_state.select(Some(0));

        // Clear editor state for new request
        self.editing_request_idx = Some(0);
        self.url_input = single_line_textarea("");
        self.method = HttpMethod::Get;
        self.params = vec![];
        self.headers = vec![];
        self.set_body("");
        self.params_editor.reset();
        self.headers_editor.reset();
        self.request_state = RequestState::default();
    }

    pub fn update_request(&mut self, idx: usize, request: Request) {
        if let Some(existing) = self.requests.get_mut(idx) {
            *existing = request;
        }
    }

    pub fn delete_selected_request(&mut self) {
        if !self.requests.is_empty() {
            let selected = self.selected_request();
            self.requests.remove(selected);

            // Adjust editing index after deletion
            if let Some(idx) = self.editing_request_idx {
                if idx == selected {
                    self.editing_request_idx = None;
                } else if idx > selected {
                    self.editing_request_idx = Some(idx - 1);
                }
            }

            if selected >= self.requests.len() && !self.requests.is_empty() {
                self.sidebar_state.select(Some(self.requests.len() - 1));
            }
        }
    }

    pub fn load_selected_request(&mut self) {
        let idx = self.selected_request();
        let Some(req) = self.requests.get(idx).cloned() else { return };

        self.editing_request_idx = Some(idx);
        self.url_input = single_line_textarea(&req.url);
        self.method = req.method;
        self.params = req.params;
        self.headers = req.headers;
        self.set_body(&req.body);
        self.params_editor.reset();
        self.headers_editor.reset();
    }

    // Editing
    pub fn start_editing(&mut self, focus: EditFocus) {
        self.edit_focus = focus;
        if focus == EditFocus::KeyValue {
            self.sync_kv_editor_from_items();
        }
    }

    pub fn stop_editing(&mut self) {
        if self.edit_focus == EditFocus::KeyValue {
            self.sync_kv_items_from_editor();
        }
        if self.edit_focus == EditFocus::Body {
            self.validate_json();
        }
        self.edit_focus = EditFocus::None;
    }

    pub fn cycle_method_next(&mut self) {
        self.method = self.method.next();
    }

    pub fn cycle_method_prev(&mut self) {
        self.method = self.method.prev();
    }

    // Key-value helpers
    pub fn current_kv_items(&self) -> &Vec<KeyValue> {
        match self.active_tab {
            RequestTab::Params => &self.params,
            RequestTab::Headers | RequestTab::Body => &self.headers,
        }
    }

    fn current_kv_items_mut(&mut self) -> &mut Vec<KeyValue> {
        match self.active_tab {
            RequestTab::Params => &mut self.params,
            RequestTab::Headers | RequestTab::Body => &mut self.headers,
        }
    }

    pub fn current_kv_editor(&self) -> &KvEditor {
        match self.active_tab {
            RequestTab::Params => &self.params_editor,
            RequestTab::Headers | RequestTab::Body => &self.headers_editor,
        }
    }

    pub fn current_kv_editor_mut(&mut self) -> &mut KvEditor {
        match self.active_tab {
            RequestTab::Params => &mut self.params_editor,
            RequestTab::Headers | RequestTab::Body => &mut self.headers_editor,
        }
    }

    fn sync_kv_editor_from_items(&mut self) {
        let selected = self.current_kv_editor().selected();
        if let Some(item) = self.current_kv_items().get(selected).cloned() {
            self.current_kv_editor_mut().sync_from_item(&item);
        }
    }

    fn sync_kv_items_from_editor(&mut self) {
        let selected = self.current_kv_editor().selected();
        let (key, value) = {
            let editor = self.current_kv_editor();
            (
                textarea_value(&editor.key_input).to_string(),
                textarea_value(&editor.value_input).to_string(),
            )
        };
        if let Some(item) = self.current_kv_items_mut().get_mut(selected) {
            item.key = key;
            item.value = value;
        }
    }

    pub fn kv_add(&mut self) {
        self.current_kv_items_mut().push(KeyValue::default());
        let len = self.current_kv_items().len();
        let editor = self.current_kv_editor_mut();
        editor.select(len.saturating_sub(1));
        editor.field = KvField::Key;
        editor.key_input = single_line_textarea("");
        editor.value_input = single_line_textarea("");
    }

    pub fn kv_delete(&mut self) {
        let selected = self.current_kv_editor().selected();
        let items = self.current_kv_items_mut();
        if items.is_empty() {
            return;
        }
        items.remove(selected);
        let new_len = items.len();
        let editor = self.current_kv_editor_mut();
        if editor.selected() >= new_len && new_len > 0 {
            editor.select(new_len - 1);
        }
    }

    pub fn kv_toggle_enabled(&mut self) {
        let selected = self.current_kv_editor().selected();
        if let Some(item) = self.current_kv_items_mut().get_mut(selected) {
            item.enabled = !item.enabled;
        }
    }

    fn kv_navigate(&mut self, forward: bool) {
        if self.edit_focus == EditFocus::KeyValue {
            self.sync_kv_items_from_editor();
        }
        let len = self.current_kv_items().len();
        let editor = self.current_kv_editor_mut();
        if forward {
            editor.select_next(len);
        } else {
            editor.select_prev(len);
        }
        if self.edit_focus == EditFocus::KeyValue {
            self.sync_kv_editor_from_items();
        }
    }

    pub fn kv_select_next(&mut self) {
        self.kv_navigate(true);
    }

    pub fn kv_select_prev(&mut self) {
        self.kv_navigate(false);
    }

    pub fn kv_toggle_field(&mut self) {
        self.current_kv_editor_mut().toggle_field();
    }

    // Request state
    pub fn set_loading(&mut self) {
        self.request_state = RequestState::Loading;
        self.response_scroll = 0;
    }

    pub fn set_response(&mut self, response: Response) {
        self.request_state = RequestState::Success(response);
        self.response_scroll = 0;
    }

    pub fn set_error(&mut self, error: String) {
        self.request_state = RequestState::Error(error);
        self.response_scroll = 0;
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.request_state, RequestState::Loading)
    }

    // Response scrolling
    pub fn response_scroll_up(&mut self) {
        scroll_by(&mut self.response_scroll, -1, usize::MAX);
    }

    pub fn response_scroll_down(&mut self) {
        scroll_by(&mut self.response_scroll, 1, usize::MAX);
    }

    pub fn response_scroll_top(&mut self) {
        self.response_scroll = 0;
    }

    pub fn response_scroll_bottom(&mut self, max: usize) {
        if max > 0 {
            self.response_scroll = max.saturating_sub(1);
        }
    }

    pub fn help_scroll_up(&mut self, lines: usize) {
        scroll_by(&mut self.help_scroll, -(lines as isize), usize::MAX);
    }

    pub fn help_scroll_down(&mut self, lines: usize, max: usize) {
        scroll_by(&mut self.help_scroll, lines as isize, max);
    }
}
