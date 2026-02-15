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
//! Substack API client / shared types.
//!
//! Handles pagination, date filtering, and type definition for posts.

use serde::Deserialize;

use crate::client::HttpClient;
use crate::config::AppConfig;

/// Represents a single post from the Substack API.
#[derive(Debug, Deserialize, Clone)]
pub struct SubstackPost {
    pub id: u64,
    pub title: String,
    pub slug: String,
    pub post_date: String,
    pub canonical_url: String,
    pub description: String,
    pub body_html: Option<String>,
    pub cover_image: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawResponse {
    Map {
        posts: Vec<SubstackPost>,
        total: Option<u64>,
    },
    Array(Vec<SubstackPost>),
}

/// Fetch all posts matching configuration filters.
///
/// Handles pagination automatically (limit=50).
/// Resets `after`/`before` filters against `post_date` (ISO8601 string).
pub async fn fetch_posts(
    base_url: &str,
    config: &AppConfig,
    client: &dyn HttpClient,
) -> anyhow::Result<Vec<SubstackPost>> {
    let mut all_posts = Vec::new();
    let mut offset = 0;
    let limit = 50;

    // Use the base URL stripping any trailing slash, then append api/v1/posts
    let api_url = format!("{}/api/v1/posts", base_url.trim_end_matches('/'));

    loop {
        let url = format!("{api_url}?limit={limit}&offset={offset}");
        let text = client.get_text(&url).await?;

        if config.verbose {
            tracing::debug!(url = %url, "Fetched raw API response");
        }

        let raw: RawResponse = serde_json::from_str(&text).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse JSON from {}: {} - Raw body snippet: {:.200}",
                url,
                e,
                text
            )
        })?;

        let (posts, total) = match raw {
            RawResponse::Map { posts, total } => (posts, total),
            RawResponse::Array(posts) => (posts, None),
        };

        if posts.is_empty() {
            break;
        }

        let received_count = posts.len() as u64;
        for post in posts {
            // Date filtering via string comparison (works for ISO8601)
            if let Some(ref after) = config.after {
                if post.post_date < *after {
                    continue;
                }
            }
            if let Some(ref before) = config.before {
                if post.post_date > *before {
                    continue;
                }
            }
            if let Some(l) = config.limit {
                if all_posts.len() >= l as usize {
                    break;
                }
            }
            all_posts.push(post);
        }

        if let Some(l) = config.limit {
            if all_posts.len() >= l as usize {
                break;
            }
        }

        offset += limit;

        // If we have a total, use it to break.
        // If not (Array mode), we break if we received fewer than 'limit' posts,
        // which usually indicates the end of the collection.
        let should_break = if let Some(t) = total {
            offset >= t
        } else {
            received_count < limit
        };

        if should_break {
            break;
        }

        // Safety break for extremely large blogs to prevent infinite loops
        if offset > 20_000 {
            tracing::warn!("Hit 20k post limit safety break");
            break;
        }
    }

    Ok(all_posts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::HttpClient;
    use crate::config::AppConfig;

    #[derive(Debug)]
    struct MockClient {
        response: String,
    }

    #[async_trait::async_trait]
    impl HttpClient for MockClient {
        async fn get_bytes(&self, _url: &str) -> anyhow::Result<Vec<u8>> {
            Ok(vec![])
        }
        async fn get_text(&self, _url: &str) -> anyhow::Result<String> {
            Ok(self.response.clone())
        }
        fn rate_limit(&self) -> u32 {
            100
        }
    }

    fn test_config() -> AppConfig {
        use crate::cli::Cli;
        use clap::Parser;
        // Minimal valid config
        let cli =
            Cli::try_parse_from(["robustack-dl", "download", "--url", "https://x.com"]).unwrap();
        AppConfig::from_cli(&cli, None, None)
    }

    #[tokio::test]
    async fn fetch_posts_handles_map_response() {
        let response = r#"{
            "posts": [{"id": 1, "slug": "slug1", "title": "T1", "description": "D1", "body_html": null, "post_date": "2024-01-01", "canonical_url": "u1", "cover_image": null}],
            "total": 1, "limit": 50, "offset": 0
        }"#;
        let client = MockClient {
            response: response.to_string(),
        };
        let config = test_config();

        let posts = fetch_posts("https://base", &config, &client).await.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].slug, "slug1");
    }

    #[tokio::test]
    async fn fetch_posts_handles_array_response() {
        let response = r#"[
            {"id": 1, "slug": "slug1", "title": "T1", "description": "D1", "body_html": null, "post_date": "2024-01-01", "canonical_url": "u1", "cover_image": null},
            {"id": 2, "slug": "slug2", "title": "T2", "description": "D2", "body_html": null, "post_date": "2024-01-02", "canonical_url": "u2", "cover_image": null}
        ]"#;
        let client = MockClient {
            response: response.to_string(),
        };
        let config = test_config();

        let posts = fetch_posts("https://base", &config, &client).await.unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[1].slug, "slug2");
    }

    #[tokio::test]
    async fn fetch_posts_respects_limit() {
        let response = r#"[
            {"id": 1, "slug": "slug1", "title": "T1", "description": "D1", "body_html": null, "post_date": "2024-01-01", "canonical_url": "u1", "cover_image": null},
            {"id": 2, "slug": "slug2", "title": "T2", "description": "D2", "body_html": null, "post_date": "2024-01-02", "canonical_url": "u2", "cover_image": null}
        ]"#;
        let client = MockClient {
            response: response.to_string(),
        };
        let mut config = test_config();
        config.limit = Some(1);

        let posts = fetch_posts("https://base", &config, &client).await.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].slug, "slug1");
    }
}
