use std::time::{Duration, Instant};

use reqwest::Client;
use tokio::sync::mpsc;

use crate::models::{HttpMethod, Response};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug)]
pub enum HttpResult {
    Success(Response),
    Error(String),
}

fn build_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .user_agent("Courier/1.0.0")
        .build()
}

pub async fn send_request(
    method: HttpMethod,
    url: String,
    tx: mpsc::UnboundedSender<HttpResult>,
) {
    
    let result = execute_request(method, url).await;
    let _ = tx.send(result);
}

async fn execute_request(method: HttpMethod, url: String) -> HttpResult {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return HttpResult::Error(format!("Failed to create client: {}", e)),
    };

    let start = Instant::now();

    let request = match method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
        HttpMethod::Put => client.put(&url),
        HttpMethod::Patch => client.patch(&url),
        HttpMethod::Delete => client.delete(&url),
        HttpMethod::Head => client.head(&url),
        HttpMethod::Options => client.request(reqwest::Method::OPTIONS, &url),
    };

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
