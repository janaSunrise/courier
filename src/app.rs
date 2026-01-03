use crate::models::{HttpMethod, Request, RequestState, Response};

/// The currently focused panel in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    #[default]
    Sidebar,
    RequestEditor,
    Response,
}

impl Panel {
    /// Move to the next panel (wrapping around)
    pub fn next(self) -> Self {
        match self {
            Panel::Sidebar => Panel::RequestEditor,
            Panel::RequestEditor => Panel::Response,
            Panel::Response => Panel::Sidebar,
        }
    }

    /// Move to the previous panel (wrapping around)
    pub fn prev(self) -> Self {
        match self {
            Panel::Sidebar => Panel::Response,
            Panel::RequestEditor => Panel::Sidebar,
            Panel::Response => Panel::RequestEditor,
        }
    }
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Currently focused panel
    pub focused_panel: Panel,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Whether to show help overlay
    pub show_help: bool,
    /// List of requests in history
    pub requests: Vec<Request>,
    /// Currently selected request index in sidebar
    pub selected_request: usize,
    /// Whether we're in URL input mode
    pub input_mode: bool,
    /// Current URL being edited
    pub input_url: String,
    /// Current HTTP method
    pub input_method: HttpMethod,
    /// Cursor position in URL input
    pub cursor_position: usize,
    /// Current request/response state
    pub request_state: RequestState,
    /// Scroll position in response body
    pub response_scroll: usize,
    /// Scroll position in help overlay
    pub help_scroll: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            focused_panel: Panel::default(),
            should_quit: false,
            show_help: false,
            requests: vec![],
            selected_request: 0,
            input_mode: false,
            input_url: String::new(),
            input_method: HttpMethod::Get,
            cursor_position: 0,
            request_state: RequestState::default(),
            response_scroll: 0,
            help_scroll: 0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.help_scroll = 0;  // Reset when help is re-opened
        }
    }

    // Panels
    pub fn focus_next(&mut self) {
        self.focused_panel = self.focused_panel.next();
    }

    pub fn focus_prev(&mut self) {
        self.focused_panel = self.focused_panel.prev();
    }

    // Requests UI
    pub fn select_next_request(&mut self) {
        if !self.requests.is_empty() {
            self.selected_request = (self.selected_request + 1) % self.requests.len();
        }
    }

    pub fn select_prev_request(&mut self) {
        if !self.requests.is_empty() {
            self.selected_request = self
                .selected_request
                .checked_sub(1)
                .unwrap_or(self.requests.len() - 1);
        }
    }

    // Requests CRUD
    pub fn current_request(&self) -> Option<&Request> {
        self.requests.get(self.selected_request)
    }

    pub fn add_request(&mut self, request: Request) {
        self.requests.insert(0, request);
        self.selected_request = 0;
    }

    pub fn delete_selected_request(&mut self) {
        if !self.requests.is_empty() {
            self.requests.remove(self.selected_request);
            if self.selected_request >= self.requests.len() && !self.requests.is_empty() {
                self.selected_request = self.requests.len() - 1;
            }
        }
    }

    // Input mode
    pub fn enter_input_mode(&mut self) {
        self.input_mode = true;
    }

    pub fn exit_input_mode(&mut self) {
        self.input_mode = false;
    }

    // URL input editing
    pub fn input_char(&mut self, c: char) {
        self.input_url.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_url.remove(self.cursor_position);
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input_url.len() {
            self.input_url.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_url.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input_url.len();
    }

    /// Move cursor to previous word boundary
    pub fn move_cursor_word_left(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        // Skip any spaces before cursor
        let mut pos = self.cursor_position - 1;
        let chars: Vec<char> = self.input_url.chars().collect();
        while pos > 0 && !chars[pos].is_alphanumeric() {
            pos -= 1;
        }

        // Skip word characters
        while pos > 0 && chars[pos - 1].is_alphanumeric() {
            pos -= 1;
        }
        self.cursor_position = pos;
    }

    /// Move cursor to next word boundary
    pub fn move_cursor_word_right(&mut self) {
        let len = self.input_url.len();
        if self.cursor_position >= len {
            return;
        }

        let chars: Vec<char> = self.input_url.chars().collect();
        let mut pos = self.cursor_position;

        // Skip current word
        while pos < len && chars[pos].is_alphanumeric() {
            pos += 1;
        }
        // Skip spaces/punctuation
        while pos < len && !chars[pos].is_alphanumeric() {
            pos += 1;
        }
        self.cursor_position = pos;
    }

    /// Delete from cursor to start of line (Ctrl+U)
    pub fn delete_to_start(&mut self) {
        if self.cursor_position > 0 {
            self.input_url = self.input_url[self.cursor_position..].to_string();
            self.cursor_position = 0;
        }
    }

    /// Delete from cursor to end of line (Ctrl+K)
    pub fn delete_to_end(&mut self) {
        self.input_url.truncate(self.cursor_position);
    }

    /// Delete word before cursor (Ctrl+W)
    pub fn delete_word_backward(&mut self) {
        if self.cursor_position == 0 {
            return;
        }
        let old_pos = self.cursor_position;
        self.move_cursor_word_left();
        let new_pos = self.cursor_position;
        self.input_url = format!(
            "{}{}",
            &self.input_url[..new_pos],
            &self.input_url[old_pos..]
        );
    }

    pub fn clear_input(&mut self) {
        self.input_url.clear();
        self.cursor_position = 0;
    }

    // Method cycling
    pub fn cycle_method_next(&mut self) {
        self.input_method = self.input_method.next();
    }

    pub fn cycle_method_prev(&mut self) {
        self.input_method = self.input_method.prev();
    }

    // Load selected request into editor
    pub fn load_selected_request(&mut self) {
        if let Some(req) = self.requests.get(self.selected_request) {
            self.input_url = req.url.clone();
            self.input_method = req.method;
            self.cursor_position = self.input_url.len();
        }
    }

    // Request state management
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
    pub fn scroll_response_up(&mut self, lines: usize) {
        self.response_scroll = self.response_scroll.saturating_sub(lines);
    }

    pub fn scroll_response_down(&mut self, lines: usize, max_lines: usize) {
        if max_lines > 0 {
            self.response_scroll = (self.response_scroll + lines).min(max_lines.saturating_sub(1));
        }
    }

    pub fn scroll_response_top(&mut self) {
        self.response_scroll = 0;
    }

    pub fn scroll_response_bottom(&mut self, max_lines: usize) {
        if max_lines > 0 {
            self.response_scroll = max_lines.saturating_sub(1);
        }
    }

    // Help scrolling
    pub fn scroll_help_up(&mut self, lines: usize) {
        self.help_scroll = self.help_scroll.saturating_sub(lines);
    }

    pub fn scroll_help_down(&mut self, lines: usize, max_lines: usize) {
        if max_lines > 0 {
            // Absolute sorcery
            self.help_scroll = (self.help_scroll + lines).min(max_lines.saturating_sub(1));
        }
    }
}
