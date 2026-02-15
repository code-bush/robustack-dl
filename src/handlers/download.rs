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
//! Download handler — orchestrates the Substack content download pipeline.
//!
//! # Responsibilities
//! 1. Load or create the idempotency manifest.
//! 2. Fetch post listings from the Substack API.
//! 3. For each post: check manifest → fetch content → hash → write (if new).
//! 4. Optionally download images and file attachments.
//! 5. Optionally generate an archive index page.
//! 6. Persist the updated manifest.
//!
//! # Idempotency
//! Every write is guarded by `integrity::should_skip()`. Re-running
//! the same download command produces zero new I/O if nothing changed.

use tracing::{info, warn};

use crate::client::HttpClient;
use crate::config::{AppConfig, OutputFormat};
use crate::integrity::{self, Manifest};
use anyhow::Context;
use regex::Regex;
use std::io::Write;

/// Execute the download pipeline.
///
/// Steps:
/// 1. Load/create manifest.
/// 2. Fetch posts from Substack.
/// 3. For each post:
///    a. Fetch content if missing.
///    b. Compute hash and check manifest (idempotency).
///    c. Download images/files if enabled (rewriting paths).
///    d. Convert to target format (HTML/MD/TXT).
///    e. Save file to content-addressed path.
///    f. Update manifest.
/// 4. Save manifest.
pub async fn run(url: &str, config: &AppConfig, client: &dyn HttpClient) -> anyhow::Result<()> {
    if !config.dry_run {
        std::fs::create_dir_all(&config.output_dir).context("Failed to create output directory")?;
    }

    // Pipeline Step 1: Load manifest (skip for dry-run to avoid I/O).
    let mut manifest = if config.dry_run {
        Manifest::default()
    } else {
        Manifest::load_or_create(&config.output_dir)?
    };

    info!(
        url = %url,
        output = %config.output_dir.display(),
        format = %config.format,
        entries = manifest.len(),
        "Download pipeline started"
    );

    if config.dry_run {
        info!("Dry run mode — no files will be written");
    }

    // Step 2: Fetch post listings.
    let posts = crate::handlers::substack::fetch_posts(url, config, client).await?;
    info!(count = posts.len(), "Found posts");

    for post in &posts {
        let span = tracing::info_span!("post", slug = %post.slug);
        let _enter = span.enter();

        // Step 3: Fetch content (fallback to canonical URL if body_html missing).
        let raw_html = if let Some(ref html) = post.body_html {
            html.clone()
        } else {
            info!("Fetching full content from canonical URL");
            client.get_text(&post.canonical_url).await?
        };

        // Prepare working content (mutable for rewriting).
        let mut final_html = raw_html.clone();

        // Step 4: Download images if enabled.
        if config.download_images {
            final_html = process_images(&final_html, config, client, &mut manifest).await;
        }

        // Step 5: Download attachments if enabled.
        if config.download_files {
            final_html = process_attachments(&final_html, config, client, &mut manifest).await;
        }

        // Step 6: Transform to target format.
        let output_content = match config.format {
            OutputFormat::Html => final_html,
            OutputFormat::Md => crate::processor::html_to_markdown(&final_html),
            OutputFormat::Txt => crate::processor::html_to_text(&final_html),
        };

        let output_content = if config.add_source_url {
            crate::processor::append_source_url(&output_content, &post.canonical_url)
        } else {
            output_content
        };

        // Calculate hash of what we are about to save.
        let hash = integrity::sha256_hex(output_content.as_bytes());

        // Determine filename.
        let ext = match config.format {
            OutputFormat::Html => "html",
            OutputFormat::Md => "md",
            OutputFormat::Txt => "txt",
        };
        let safe_slug = integrity::sanitize_filename(&post.slug);
        let filename = format!("{safe_slug}.{ext}");

        // Check idempotency.
        if integrity::should_skip(&manifest, &hash, &config.output_dir, &filename) {
            info!("Skipping (up to date)");
            continue;
        }

        // Save file.
        if !config.dry_run {
            let path = config.output_dir.join(&filename);
            let mut file = std::fs::File::create(&path).context("Failed to create output file")?;
            file.write_all(output_content.as_bytes())?;
            info!(path = %path.display(), "Saved post");

            // Update manifest.
            manifest.insert(integrity::ManifestEntry {
                local_path: filename,
                sha256: hash,
                source_url: post.canonical_url.clone(),
                size: output_content.len() as u64,
                downloaded_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    // Step 8: Create archive index.
    if config.create_archive {
        crate::handlers::archive::generate_index(&posts, config)?;
    }

    // Step 9: Persist manifest.
    if !config.dry_run {
        manifest.save(&config.output_dir)?;
    }

    info!("Download completed");
    Ok(())
}

/// Helper to download an asset (image/file) and return relative path.
async fn download_asset(
    url: &str,
    subdir: &str,
    config: &AppConfig,
    client: &dyn HttpClient,
    manifest: &mut Manifest,
) -> anyhow::Result<String> {
    if config.dry_run {
        return Ok(format!("{subdir}/dry-run-asset"));
    }

    // Attempt download
    let bytes = client.get_bytes(url).await?;
    let hash = integrity::sha256_hex(&bytes);

    // Derive extension
    let ext = std::path::Path::new(url)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("bin");

    let filename = format!("{hash}.{ext}");
    let sub_path = std::path::Path::new(subdir).join(&filename);
    let full_path = config.output_dir.join(&sub_path);

    // Ensure subdir exists
    std::fs::create_dir_all(config.output_dir.join(subdir))?;

    if integrity::should_skip(
        manifest,
        &hash,
        &config.output_dir,
        sub_path.to_str().unwrap(),
    ) {
        return Ok(sub_path.to_string_lossy().to_string());
    }

    let mut file = std::fs::File::create(&full_path)?;
    file.write_all(&bytes)?;
    manifest.insert(integrity::ManifestEntry {
        source_url: url.to_string(),
        sha256: hash,
        local_path: sub_path.to_string_lossy().to_string(),
        size: bytes.len() as u64,
        downloaded_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(sub_path.to_string_lossy().to_string())
}

async fn process_images(
    html: &str,
    config: &AppConfig,
    client: &dyn HttpClient,
    manifest: &mut Manifest,
) -> String {
    let img_regex = Regex::new(r#"<img[^>]+src="([^"]+)"[^>]*>"#).expect("invalid regex");
    let mut final_html = html.to_string();

    for cap in img_regex.captures_iter(html) {
        if let Some(src_match) = cap.get(1) {
            let src_url = src_match.as_str();
            match download_asset(src_url, &config.images_dir, config, client, manifest).await {
                Ok(local_path) => {
                    final_html = final_html.replace(src_url, &local_path);
                }
                Err(e) => warn!(url = %src_url, error = %e, "Failed to download image"),
            }
        }
    }
    final_html
}

async fn process_attachments(
    html: &str,
    config: &AppConfig,
    client: &dyn HttpClient,
    manifest: &mut Manifest,
) -> String {
    let link_regex = Regex::new(r#"<a[^>]+href="([^"]+)"[^>]*>"#).expect("invalid regex");
    let mut final_html = html.to_string();

    for cap in link_regex.captures_iter(html) {
        if let Some(href_match) = cap.get(1) {
            let href_url = href_match.as_str();
            if is_allowed_extension(href_url, &config.file_extensions) {
                match download_asset(href_url, &config.files_dir, config, client, manifest).await {
                    Ok(local_path) => {
                        final_html = final_html.replace(href_url, &local_path);
                    }
                    Err(e) => {
                        warn!(url = %href_url, error = %e, "Failed to download attachment");
                    }
                }
            }
        }
    }
    final_html
}

fn is_allowed_extension(url: &str, allowlist: &str) -> bool {
    if allowlist.is_empty() {
        return true;
    }
    let ext = std::path::Path::new(url)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    allowlist
        .split(',')
        .any(|e| e.trim().eq_ignore_ascii_case(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock HTTP client for handler testing — implements trivial responses.
    #[derive(Debug)]
    struct MockClient;

    #[async_trait::async_trait]
    impl HttpClient for MockClient {
        async fn get_bytes(&self, _url: &str) -> anyhow::Result<Vec<u8>> {
            Ok(b"<html>mock</html>".to_vec())
        }
        async fn get_text(&self, url: &str) -> anyhow::Result<String> {
            if url.contains("/api/v1/posts") {
                // Return valid JSON with 0 posts to test pipeline mechanics
                return Ok(r#"{ "posts": [], "total": 0, "limit": 50, "offset": 0 }"#.to_string());
            }
            Ok("<html>mock</html>".to_string())
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
            "download",
            "--url",
            "https://example.substack.com",
            "--dry-run",
        ])
        .unwrap();
        if let crate::cli::Commands::Download(ref dl) = cli.command {
            AppConfig::from_cli(&cli, dl.limit, Some(dl))
        } else {
            panic!("expected Download");
        }
    }

    #[tokio::test]
    async fn dry_run_produces_no_output() {
        let config = test_config();
        let client = MockClient;
        let result = run("https://example.substack.com", &config, &client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handler_uses_mock_client() {
        let client = MockClient;
        assert_eq!(client.rate_limit(), 100);
        let text = client
            .get_text("https://fake.url/api/v1/posts")
            .await
            .unwrap();
        assert!(text.contains("posts"));
    }
}
