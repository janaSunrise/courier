use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code (e.g., 200, 404, 500)
    pub status: u16,
    /// Status text (e.g., "OK", "Not Found")
    pub status_text: String,
    /// Response headers as key-value pairs
    pub headers: Vec<(String, String)>,
    /// Response body as string
    pub body: String,
    /// Request duration
    pub elapsed: Duration,
    /// Size of response body in bytes
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

    pub fn content_type(&self) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.as_str())
    }

    pub fn is_json(&self) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
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
