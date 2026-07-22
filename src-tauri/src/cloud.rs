use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;

const DEFAULT_ASSISTANT_ENDPOINT: &str = "https://winreclaim.vercel.app/api/assistant";

#[derive(Debug, Deserialize)]
struct CloudEnvelope<T> {
    model: String,
    result: T,
}

pub fn request<TRequest, TResponse>(task: &str, data: &TRequest) -> Result<(String, TResponse)>
where
    TRequest: Serialize,
    TResponse: DeserializeOwned,
{
    let endpoint = assistant_endpoint();
    let body = serde_json::json!({
        "task": task,
        "data": data,
    });

    let response = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?
        .post(&endpoint)
        .header("X-WinReclaim-Client", "desktop/1.2.1")
        .json(&body)
        .send()
        .with_context(|| format!("Unable to reach the WinReclaim cloud assistant at {endpoint}"))?;

    let status = response.status();
    let response_body = response
        .text()
        .context("Unable to read the WinReclaim cloud assistant response")?;

    if !status.is_success() {
        let detail = response_body
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(400)
            .collect::<String>();
        return Err(anyhow!(
            "WinReclaim cloud assistant returned {status}: {detail}"
        ));
    }

    let envelope: CloudEnvelope<TResponse> = serde_json::from_str(&response_body)
        .context("WinReclaim cloud assistant returned an invalid response")?;

    if envelope.model.trim().is_empty() {
        return Err(anyhow!(
            "WinReclaim cloud assistant did not report the routed model"
        ));
    }

    Ok((envelope.model, envelope.result))
}

pub fn assistant_endpoint() -> String {
    std::env::var("WINRECLAIM_ASSISTANT_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| value.starts_with("https://"))
        .unwrap_or_else(|| DEFAULT_ASSISTANT_ENDPOINT.to_string())
}
