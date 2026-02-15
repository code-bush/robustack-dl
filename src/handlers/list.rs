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
//! List handler — lists posts from a Substack.
//!
//! # Responsibilities
//! 1. Fetch post listings from the Substack API.
//! 2. Display each post's title, date, and URL.
//!
//! # Design
//! Follows the same pattern as `download.rs`: receives `&AppConfig`
//! and `&dyn HttpClient`, never raw CLI types.

use tracing::info;

use crate::client::HttpClient;
use crate::config::AppConfig;

/// Execute the list pipeline.
///
/// # Arguments
/// - `url` — Substack base URL to list posts from.
/// - `config` — Typed application configuration.
/// - `client` — HTTP client (trait object for testability).
///
/// # Errors
/// Returns `anyhow::Error` on network failure or API errors.
#[allow(clippy::unused_async)]
pub async fn run(url: &str, config: &AppConfig, client: &dyn HttpClient) -> anyhow::Result<()> {
    info!(url = %url, "Listing posts");

    if let Some(ref after) = config.after {
        info!(after = %after, "Filtering posts after date");
    }

    if let Some(ref before) = config.before {
        info!(before = %before, "Filtering posts before date");
    }

    // Fetch all posts using shared logic (handles pagination & filtering)
    let posts = crate::handlers::substack::fetch_posts(url, config, client).await?;

    if posts.is_empty() {
        info!("No posts found matching criteria.");
    } else {
        info!(count = posts.len(), "Found posts");
        for post in &posts {
            info!(
                date = %post.post_date,
                title = %post.title,
                url = %post.canonical_url,
                "Post"
            );
        }
    }

    info!("List pipeline completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock HTTP client for handler testing — no real network calls.
    #[derive(Debug)]
    struct MockClient;

    #[async_trait::async_trait]
    impl HttpClient for MockClient {
        async fn get_bytes(&self, _url: &str) -> anyhow::Result<Vec<u8>> {
            Ok(b"<html>mock</html>".to_vec())
        }
        async fn get_text(&self, url: &str) -> anyhow::Result<String> {
            if url.contains("/api/v1/posts") {
                return Ok(r#"{ "posts": [], "total": 0, "limit": 50, "offset": 0 }"#.to_string());
            }
            Ok("stub".to_string())
        }
        fn rate_limit(&self) -> u32 {
            100
        }
    }

    fn test_config() -> AppConfig {
        use crate::cli::Cli;
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "list",
            "--url",
            "https://example.substack.com",
        ])
        .unwrap();
        AppConfig::from_cli(&cli, None)
    }

    #[tokio::test]
    async fn list_stub_succeeds() {
        let config = test_config();
        let client = MockClient;
        let result = run("https://example.substack.com", &config, &client).await;
        assert!(result.is_ok());
    }
}
