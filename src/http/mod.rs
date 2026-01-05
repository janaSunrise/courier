mod client;

use std::time::Duration;

pub use client::{send_request, HttpResult, RequestData};
pub use reqwest::Client;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

pub fn build_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .user_agent("Courier/0.1.0")
        .build()
}
