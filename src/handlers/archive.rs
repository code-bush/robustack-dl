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
//! Archive handler — generates an index of downloaded content.

use crate::config::{AppConfig, OutputFormat};
use crate::handlers::substack::SubstackPost;
use crate::integrity;
use anyhow::Context;
use std::io::Write;
use tracing::info;

/// Generate an index file (index.html) listing all available posts.
///
/// This function relies on the fact that filenames are deterministically
/// derived from post slugs: `{slug}.{ext}`.
pub fn generate_index(posts: &[SubstackPost], config: &AppConfig) -> anyhow::Result<()> {
    if posts.is_empty() {
        return Ok(());
    }

    info!("Generating archive index");

    let ext = match config.format {
        OutputFormat::Html => "html",
        OutputFormat::Md => "md",
        OutputFormat::Txt => "txt",
    };

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<title>Archive Index</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: system-ui, sans-serif; max-width: 800px; margin: 2rem auto; line-height: 1.5; }\n");
    html.push_str("h1 { border-bottom: 2px solid #eee; padding-bottom: 0.5rem; }\n");
    html.push_str("ul { list-style-type: none; padding: 0; }\n");
    html.push_str("li { padding: 0.5rem 0; border-bottom: 1px solid #f0f0f0; }\n");
    html.push_str("a { text-decoration: none; color: #0066cc; font-weight: 500; }\n");
    html.push_str("a:hover { text-decoration: underline; }\n");
    html.push_str(".date { color: #666; font-size: 0.9em; margin-right: 1rem; font-family: monospace; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("<h1>Archive Index</h1>\n");
    html.push_str("<ul>\n");

    for post in posts {
        let safe_slug = integrity::sanitize_filename(&post.slug);
        let filename = format!("{safe_slug}.{ext}");
        // Escape HTML in title (basic)
        let title = post.title.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        
        html.push_str("<li>\n");
        html.push_str(&format!("<span class=\"date\">{}</span>", post.post_date));
        html.push_str(&format!("<a href=\"{filename}\">{title}</a>"));
        html.push_str("</li>\n");
    }

    html.push_str("</ul>\n");
    html.push_str("</body>\n</html>");

    if !config.dry_run {
        let path = config.output_dir.join("index.html");
        let mut file = std::fs::File::create(&path).context("Failed to create index.html")?;
        file.write_all(html.as_bytes())?;
        info!(path = %path.display(), "Saved archive index");
    } else {
        info!("Dry run: would save index.html");
    }

    Ok(())
}
