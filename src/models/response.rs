use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    #[allow(dead_code)]
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub elapsed: Duration,
    pub size_bytes: usize,
}

impl Response {
    /// Format elapsed time for display (e.g., "123ms", "1.2s")
    pub fn elapsed_display(&self) -> String {
        let ms = self.elapsed.as_millis();
        if ms < 1000 {
            format!("{}ms", ms)
        } else {
            format!("{:.1}s", self.elapsed.as_secs_f64())
        }
    }

    /// Format body size for display (e.g., "1.2 KB", "3.4 MB")
    pub fn size_display(&self) -> String {
        let bytes = self.size_bytes;
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }

}

#[derive(Debug, Clone)]
pub enum RequestState {
    /// No request has been made yet
    Idle,
    /// Request is currently in progress
    Loading,
    /// Request completed successfully
    Success(Response),
    /// Request failed with an error
    Error(String),
}

impl Default for RequestState {
    fn default() -> Self {
        Self::Idle
    }
}
