//! Shared bounded HTTP transport and retry policy.
use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use reqwest::header::RETRY_AFTER;
use serde_json::Value;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
pub const QUERY_BUDGET: Duration = Duration::from_secs(30);

pub fn client() -> Result<Client> {
    Client::builder()
        .user_agent(concat!("check_vpn/", env!("CARGO_PKG_VERSION")))
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("failed to build HTTP client")
}

pub fn validate_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url).context("invalid provider URL")?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        bail!("provider URL must be an absolute HTTP(S) URL");
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        bail!("provider URL must not contain credentials");
    }
    Ok(())
}

pub fn cancelled(cancel: Option<&AtomicBool>) -> bool {
    cancel.is_some_and(|c| c.load(Ordering::Relaxed))
}

pub fn wait(duration: Duration, cancel: Option<&AtomicBool>) -> Result<()> {
    let start = Instant::now();
    loop {
        if cancelled(cancel) {
            bail!("provider query cancelled");
        }
        let remaining = duration.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            return Ok(());
        }
        std::thread::sleep(remaining.min(Duration::from_millis(50)));
    }
}

pub fn get_json(
    client: &Client,
    url: &str,
    attempts: usize,
    cancel: Option<&AtomicBool>,
    budget: Duration,
) -> Result<Value> {
    validate_url(url)?;
    let attempts = attempts.max(1);
    let max_bytes = std::env::var("CHECK_VPN_MAX_RESPONSE_BYTES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5 * 1024 * 1024);
    let start = Instant::now();
    let mut last = anyhow::anyhow!("query deadline exceeded");
    for attempt in 0..attempts {
        if cancelled(cancel) {
            bail!("provider query cancelled");
        }
        let remaining = budget.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            break;
        }
        let mut delay = Duration::from_millis(500 * (attempt as u64 + 1).min(120));
        let request = client.get(url);
        // Preserve a caller's shorter client timeout (important for embedding/tests).
        let request = if remaining < REQUEST_TIMEOUT {
            request.timeout(remaining)
        } else {
            request
        };
        match request.send() {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    if response
                        .content_length()
                        .is_some_and(|len| len > max_bytes as u64)
                    {
                        bail!("response too large");
                    }
                    let mut bytes = Vec::new();
                    response
                        .take((max_bytes as u64).saturating_add(1))
                        .read_to_end(&mut bytes)
                        .context("failed to read response body")?;
                    if bytes.len() > max_bytes {
                        bail!("response too large (>{max_bytes} bytes)");
                    }
                    if cancelled(cancel) {
                        bail!("provider query cancelled");
                    }
                    return serde_json::from_slice(&bytes).context("failed to parse json");
                }
                last = anyhow::anyhow!("non-success status: {status}");
                if status.as_u16() != 429 && !status.is_server_error() {
                    return Err(last);
                }
                if let Some(seconds) = response
                    .headers()
                    .get(RETRY_AFTER)
                    .and_then(|h| h.to_str().ok())
                    .and_then(|h| h.parse::<u64>().ok())
                {
                    delay = Duration::from_secs(seconds.min(60));
                }
            }
            Err(error) => last = error.into(),
        }
        if attempt + 1 < attempts {
            wait(delay.min(budget.saturating_sub(start.elapsed())), cancel)?;
        }
    }
    Err(last.context(format!(
        "failed to query ip api/provider after {attempts} attempt(s)"
    )))
}

pub fn identity(value: &Value, keys: &[&str]) -> Result<String> {
    keys.iter()
        .filter_map(|key| value.get(*key).and_then(Value::as_str))
        .find(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing nonempty identity field (tried {})",
                keys.join(", ")
            )
        })
}
