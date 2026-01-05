use std::time::Instant;

use reqwest::Client;
use tokio::sync::mpsc;

use crate::models::{HttpMethod, KeyValue, Response};

#[derive(Debug)]
pub enum HttpResult {
    Success(Response),
    Error(String),
}

pub struct RequestData {
    pub method: HttpMethod,
    pub url: String,
    pub params: Vec<KeyValue>,
    pub headers: Vec<KeyValue>,
    pub body: String,
}

pub async fn send_request(client: Client, data: RequestData, tx: mpsc::UnboundedSender<HttpResult>) {
    let result = execute_request(&client, data).await;
    let _ = tx.send(result);
}

async fn execute_request(client: &Client, data: RequestData) -> HttpResult {
    let url = build_url_with_params(&data.url, &data.params);

    let start = Instant::now();

    let mut request = match data.method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
        HttpMethod::Put => client.put(&url),
        HttpMethod::Patch => client.patch(&url),
        HttpMethod::Delete => client.delete(&url),
        HttpMethod::Head => client.head(&url),
        HttpMethod::Options => client.request(reqwest::Method::OPTIONS, &url),
    };

    for header in &data.headers {
        if header.enabled && !header.key.is_empty() {
            request = request.header(&header.key, &header.value);
        }
    }

    if !data.body.is_empty() {
        let has_content_type = data.headers.iter().any(|h| {
            h.enabled && h.key.to_lowercase() == "content-type"
        });

        if !has_content_type {
            // Try to detect if it's JSON
            if data.body.trim().starts_with('{') || data.body.trim().starts_with('[') {
                request = request.header("Content-Type", "application/json");
            } else {
                request = request.header("Content-Type", "text/plain");
            }
        }

        request = request.body(data.body);
    }

    let response = match request.send().await {
        Ok(r) => r,
        Err(e) => {
            let error_msg = if e.is_timeout() {
                "Request timed out".to_string()
            } else if e.is_connect() {
                format!("Connection failed: {}", e)
            } else if e.is_request() {
                format!("Invalid request: {}", e)
            } else {
                format!("Request failed: {}", e)
            };
            return HttpResult::Error(error_msg);
        }
    };

    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_string(),
                v.to_str().unwrap_or("<binary>").to_string(),
            )
        })
        .collect();

    let body = match response.text().await {
        Ok(text) => text,
        Err(e) => return HttpResult::Error(format!("Failed to read response body: {}", e)),
    };

    let size_bytes = body.len();

    HttpResult::Success(Response {
        status,
        status_text,
        headers,
        body,
        elapsed,
        size_bytes,
    })
}

fn build_url_with_params(base_url: &str, params: &[KeyValue]) -> String {
    let enabled_params: Vec<_> = params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .collect();

    if enabled_params.is_empty() {
        return base_url.to_string();
    }

    let query: String = enabled_params
        .iter()
        .map(|p| format!("{}={}", urlencoding::encode(&p.key), urlencoding::encode(&p.value)))
        .collect::<Vec<_>>()
        .join("&");

    if base_url.contains('?') {
        format!("{}&{}", base_url, query)
    } else {
        format!("{}?{}", base_url, query)
    }
}
