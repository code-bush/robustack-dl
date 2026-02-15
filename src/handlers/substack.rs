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

/// API response wrapper.
#[derive(Debug, Deserialize)]
struct PostResponse {
    posts: Vec<SubstackPost>,
    total: u64,
    #[allow(dead_code)]
    limit: u64,
    #[allow(dead_code)]
    offset: u64,
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
        let response: PostResponse = serde_json::from_str(&text)?;

        if response.posts.is_empty() {
            break;
        }

        for post in response.posts {
            // Date filtering via string comparison (works for ISO8601)
            if let Some(ref after) = config.after {
                // post_date "2024-01-01T..." >= after "2024-01-01"?
                // We want post_date >= after.
                // "2024-01-01T" > "2024-01-01". So if strict >, it works.
                // Determining exact "on or after" behavior with string compare of different lengths is tricky.
                // We assume strict string comparison: "2024-01-01T..." > "2024-01-01" is true.
                if post.post_date < *after {
                    continue;
                }
            }
            if let Some(ref before) = config.before {
                // We want post_date <= before.
                // "2024-01-01T..." > "2024-01-01".
                // So "2024-01-01T12:00" > "2024-01-01".
                // If user says before "2024-01-01", they usually mean end of that day?
                // Or start?
                // If we assume start, then "2024-01-01T12:00" is > "2024-01-01", so it is excluded.
                // This is safe/conservative.
                if post.post_date > *before {
                    continue;
                }
            }
            all_posts.push(post);
        }

        offset += limit;
        if offset >= response.total {
            break;
        }

        // Safety break for extremely large blogs to prevent infinite loops
        if offset > 10_000 {
            tracing::warn!("Hit 10k post limit safety break");
            break;
        }
    }

    Ok(all_posts)
}
