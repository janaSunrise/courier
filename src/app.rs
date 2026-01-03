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
#[derive(Debug, Default)]
pub struct App {
    /// Currently focused panel
    pub focused_panel: Panel,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Whether to show help overlay
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Focus the next panel
    pub fn focus_next(&mut self) {
        self.focused_panel = self.focused_panel.next();
    }

    /// Focus the previous panel
    pub fn focus_prev(&mut self) {
        self.focused_panel = self.focused_panel.prev();
    }
}
