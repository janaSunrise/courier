use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct KeyValue {
    pub enabled: bool,
    pub key: String,
    pub value: String,
}

impl Default for KeyValue {
    fn default() -> Self {
        Self {
            enabled: true,
            key: String::new(),
            value: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AuthType {
    #[default]
    None,
    Basic { username: String, password: String },
    Bearer { token: String },
    ApiKey { key: String, value: String },
}

impl AuthType {
    pub fn variant_name(&self) -> &'static str {
        match self {
            AuthType::None => "None",
            AuthType::Basic { .. } => "Basic",
            AuthType::Bearer { .. } => "Bearer",
            AuthType::ApiKey { .. } => "API Key",
        }
    }

    pub fn cycle_next(&self) -> AuthType {
        match self {
            AuthType::None => AuthType::Basic { username: String::new(), password: String::new() },
            AuthType::Basic { .. } => AuthType::Bearer { token: String::new() },
            AuthType::Bearer { .. } => AuthType::ApiKey { key: String::new(), value: String::new() },
            AuthType::ApiKey { .. } => AuthType::None,
        }
    }

    pub fn cycle_prev(&self) -> AuthType {
        match self {
            AuthType::None => AuthType::ApiKey { key: String::new(), value: String::new() },
            AuthType::Basic { .. } => AuthType::None,
            AuthType::Bearer { .. } => AuthType::Basic { username: String::new(), password: String::new() },
            AuthType::ApiKey { .. } => AuthType::Bearer { token: String::new() },
        }
    }

    pub fn has_two_fields(&self) -> bool {
        matches!(self, AuthType::Basic { .. } | AuthType::ApiKey { .. })
    }
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DEL",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPT",
        }
    }

    pub fn next(self) -> Self {
        match self {
            HttpMethod::Get => HttpMethod::Post,
            HttpMethod::Post => HttpMethod::Put,
            HttpMethod::Put => HttpMethod::Patch,
            HttpMethod::Patch => HttpMethod::Delete,
            HttpMethod::Delete => HttpMethod::Head,
            HttpMethod::Head => HttpMethod::Options,
            HttpMethod::Options => HttpMethod::Get,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            HttpMethod::Get => HttpMethod::Options,
            HttpMethod::Post => HttpMethod::Get,
            HttpMethod::Put => HttpMethod::Post,
            HttpMethod::Patch => HttpMethod::Put,
            HttpMethod::Delete => HttpMethod::Patch,
            HttpMethod::Head => HttpMethod::Delete,
            HttpMethod::Options => HttpMethod::Head,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub url: String,
    pub params: Vec<KeyValue>,
    pub headers: Vec<KeyValue>,
    pub body: String,
    pub auth: AuthType,
    pub created_at: SystemTime,
}

impl Request {
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            params: vec![],
            headers: vec![],
            body: String::new(),
            auth: AuthType::None,
            created_at: SystemTime::now(),
        }
    }

    pub fn relative_time(&self) -> String {
        let elapsed = self.created_at.elapsed().unwrap_or_default();
        let secs = elapsed.as_secs();

        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else if secs < 86400 {
            format!("{}h", secs / 3600)
        } else {
            format!("{}d", secs / 86400)
        }
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new(HttpMethod::Get, "")
    }
}
