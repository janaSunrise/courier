use crate::models::{HttpMethod, Request};

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
}

impl Default for App {
    fn default() -> Self {
        Self {
            focused_panel: Panel::default(),
            should_quit: false,
            show_help: false,
            requests: vec![
                Request::new(HttpMethod::Get, "https://api.example.com/users"),
                Request::new(HttpMethod::Post, "https://api.example.com/auth/login"),
                Request::new(HttpMethod::Put, "https://api.example.com/users/42"),
                Request::new(HttpMethod::Delete, "https://api.example.com/sessions"),
                Request::new(HttpMethod::Get, "https://api.example.com/products"),
            ],
            selected_request: 0,
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
}
