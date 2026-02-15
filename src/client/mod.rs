//! @project       RoBustack-DL
//! @organization  CodeBush Collective
//! @license       GPL-3.0-only
//! ---------------------------------------------------------------------------
//! AI PROVENANCE & HUMAN-IN-THE-LOOP (HITL) METADATA:
//! - Prompt Engineering: Gemini 3 Flash (Strategy, Scoping & Context Tuning)
//! - Code Generation:   Gemini 3 Pro (Core Systems Engineering & Async Logic)
//! - Technical Review:  Claude 4.6 Opus (Security Audit & Idiomatic Refinement)
//! - HITL Verification: Collisio-Adolebitque - AA0614550BDC21F1 (Manual Audit & Final Validation)
//! ---------------------------------------------------------------------------
//! Verified Date: 2026-02-15
//! Integrity: GPG-Signed | HITL-Certified
//!
//! HTTP client module — trait-based abstraction over `reqwest`.
//!
//! # Design
//! `HttpClient` is a trait that defines the contract for making HTTP requests.
//! `ReqwestClient` is the production implementation wrapping `reqwest::Client`.
//! Handlers depend on `&dyn HttpClient`, enabling:
//! - Unit testing with mock clients (no network I/O).
//! - Future swap to alternative HTTP backends without changing handlers.

use std::time::Duration;

use async_trait::async_trait;

// ---------------------------------------------------------------------------
// HttpClient trait — dependency inversion boundary
// ---------------------------------------------------------------------------

/// Contract for making HTTP GET requests.
///
/// All handlers depend on `&dyn HttpClient` instead of a concrete type.
/// This inverts the dependency: business logic defines the interface,
/// and infrastructure (reqwest) satisfies it.
#[async_trait]
pub trait HttpClient: Send + Sync + std::fmt::Debug {
    /// Perform an HTTP GET and return the response body as bytes.
    ///
    /// # Errors
    /// Returns `anyhow::Error` on network failure, timeout, or non-2xx status.
    async fn get_bytes(&self, url: &str) -> anyhow::Result<Vec<u8>>;

    /// Perform an HTTP GET and return the response body as a UTF-8 string.
    ///
    /// # Errors
    /// Returns `anyhow::Error` on network failure, timeout, non-2xx status,
    /// or invalid UTF-8 in the response body.
    async fn get_text(&self, url: &str) -> anyhow::Result<String>;

    /// Returns the configured rate limit (requests per second).
    fn rate_limit(&self) -> u32;
}

// ---------------------------------------------------------------------------
// ReqwestClient — production implementation
// ---------------------------------------------------------------------------

/// Production HTTP client wrapping `reqwest::Client`.
#[derive(Debug)]
pub struct ReqwestClient {
    inner: reqwest::Client,
    rate_limit: u32,
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new(None, None, 2)
    }
}

impl ReqwestClient {
    /// Create a new client with optional proxy and cookie authentication.
    ///
    /// # Arguments
    /// - `proxy` — Optional HTTP/SOCKS5 proxy URL.
    /// - `cookie` — Optional `(name, value)` pair for Substack session auth.
    /// - `rate_limit` — Maximum requests per second.
    #[must_use]
    pub fn new(proxy: Option<&str>, cookie: Option<(&str, &str)>, rate_limit: u32) -> Self {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!(
                "RoBustack-DL/",
                env!("CARGO_PKG_VERSION"),
                " (Digital Repair Tool)"
            ));

        if let Some(p) = proxy.and_then(|url| reqwest::Proxy::all(url).ok()) {
            builder = builder.proxy(p);
        }

        if let Some((name, value)) = cookie {
            let jar = reqwest::cookie::Jar::default();
            let cookie_str = format!("{name}={value}");
            jar.add_cookie_str(&cookie_str, &"https://substack.com".parse().unwrap());
            builder = builder.cookie_provider(std::sync::Arc::new(jar));
        }

        Self {
            inner: builder.build().unwrap_or_default(),
            rate_limit,
        }
    }

    /// Build a `ReqwestClient` from an `AppConfig`.
    #[must_use]
    pub fn from_config(config: &crate::config::AppConfig) -> Self {
        let cookie = match (&config.cookie_name, &config.cookie_value) {
            (Some(name), Some(value)) => {
                use secrecy::ExposeSecret;
                Some((name.as_str(), value.expose_secret()))
            }
            _ => None,
        };
        Self::new(config.proxy.as_deref(), cookie, config.rate_limit)
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn get_bytes(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let resp = self.inner.get(url).send().await?.error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    }

    async fn get_text(&self, url: &str) -> anyhow::Result<String> {
        let resp = self.inner.get(url).send().await?.error_for_status()?;
        Ok(resp.text().await?)
    }

    fn rate_limit(&self) -> u32 {
        self.rate_limit
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reqwest_client_default_creates_instance() {
        let client = ReqwestClient::default();
        assert_eq!(client.rate_limit(), 2);
    }

    #[test]
    fn reqwest_client_custom_rate() {
        let client = ReqwestClient::new(None, None, 10);
        assert_eq!(client.rate_limit(), 10);
    }

    #[test]
    fn reqwest_client_with_proxy() {
        let client = ReqwestClient::new(Some("http://127.0.0.1:8080"), None, 2);
        assert_eq!(client.rate_limit(), 2);
    }

    #[test]
    fn reqwest_client_with_cookie() {
        let client = ReqwestClient::new(None, Some(("substack.sid", "abc123")), 2);
        assert_eq!(client.rate_limit(), 2);
    }

    #[test]
    fn reqwest_client_implements_debug() {
        let client = ReqwestClient::default();
        let debug = format!("{client:?}");
        assert!(debug.contains("ReqwestClient"));
    }

    #[test]
    fn trait_object_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ReqwestClient>();
    }
}
