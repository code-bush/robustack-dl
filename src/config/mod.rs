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
//! Configuration module — typed `AppConfig` merging CLI > Env > config.toml.
//!
//! # Design
//! `AppConfig` is the single source of truth for runtime configuration.
//! It is constructed once from CLI args and passed by reference into handlers.
//! No handler ever reads `Cli` directly — this decouples business logic
//! from the presentation layer (clean architecture boundary).

use std::path::PathBuf;

pub use crate::cli::{ImageQuality, OutputFormat};

// ---------------------------------------------------------------------------
// AppConfig — single source of truth for all runtime settings
// ---------------------------------------------------------------------------

/// Typed, immutable application configuration.
///
/// Constructed once from CLI arguments and passed by shared reference
/// into all handlers.  Handlers never access `Cli` directly.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct AppConfig {
    // -- Authentication --
    /// Cookie name for Substack session (e.g. `substack.sid`).
    pub cookie_name: Option<String>,
    /// Cookie value (opaque; never logged in plaintext).
    pub cookie_value: Option<secrecy::SecretString>,

    // -- Network --
    /// HTTP/SOCKS5 proxy URL.
    pub proxy: Option<String>,
    /// Maximum requests per second.
    pub rate_limit: u32,

    // -- Filtering --
    /// Only process posts published on or after this date.
    pub after: Option<String>,
    /// Only process posts published on or before this date.
    pub before: Option<String>,

    // -- Output --
    /// Output format (html, md, txt).
    pub format: OutputFormat,
    /// Top-level output directory.
    pub output_dir: PathBuf,

    // -- Download behaviour --
    /// If `true`, no files are written to disk.
    pub dry_run: bool,
    /// Download images locally and rewrite references.
    pub download_images: bool,
    /// Subdirectory name for images.
    pub images_dir: String,
    /// Image quality tier.
    pub image_quality: ImageQuality,
    /// Download file attachments locally and rewrite references.
    pub download_files: bool,
    /// Subdirectory name for file attachments.
    pub files_dir: String,
    /// Comma-separated extension allowlist (empty = all).
    pub file_extensions: String,
    /// Append source URL to each downloaded file.
    pub add_source_url: bool,
    /// Generate an archive index page.
    pub create_archive: bool,

    // -- Diagnostics --
    /// Verbose / debug logging enabled.
    pub verbose: bool,
}

impl AppConfig {
    /// Build an `AppConfig` from parsed CLI arguments.
    ///
    /// This is the **only** place where `Cli` types cross into the domain
    /// layer.  After this point every consumer works with `AppConfig`.
    #[must_use]
    pub fn from_cli(cli: &crate::cli::Cli, download: Option<&crate::cli::DownloadArgs>) -> Self {
        let (
            format,
            output_dir,
            dry_run,
            download_images,
            images_dir,
            image_quality,
            download_files,
            files_dir,
            file_extensions,
            add_source_url,
            create_archive,
        ) = if let Some(dl) = download {
            (
                dl.format,
                dl.output.clone(),
                dl.dry_run,
                dl.download_images,
                dl.images_dir.clone(),
                dl.image_quality,
                dl.download_files,
                dl.files_dir.clone(),
                dl.file_extensions.clone(),
                dl.add_source_url,
                dl.create_archive,
            )
        } else {
            (
                OutputFormat::Html,
                PathBuf::from("."),
                false,
                false,
                "images".to_owned(),
                ImageQuality::High,
                false,
                "files".to_owned(),
                String::new(),
                false,
                false,
            )
        };

        Self {
            cookie_name: cli.cookie_name.clone(),
            cookie_value: cli.cookie_val.clone(),
            proxy: cli.proxy.clone(),
            rate_limit: cli.rate,
            after: cli.after.clone(),
            before: cli.before.clone(),
            verbose: cli.verbose,
            format,
            output_dir,
            dry_run,
            download_images,
            images_dir,
            image_quality,
            download_files,
            files_dir,
            file_extensions,
            add_source_url,
            create_archive,
        }
    }

    /// Returns parsed file extension allowlist (empty vec = accept all).
    #[must_use]
    pub fn allowed_extensions(&self) -> Vec<&str> {
        if self.file_extensions.is_empty() {
            return Vec::new();
        }
        self.file_extensions.split(',').map(str::trim).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a minimal `Cli` for testing.
    fn test_cli() -> crate::cli::Cli {
        use clap::Parser;
        crate::cli::Cli::try_parse_from([
            "robustack-dl",
            "--rate",
            "5",
            "download",
            "--url",
            "https://example.substack.com",
            "--format",
            "md",
        ])
        .expect("valid test args")
    }

    #[test]
    fn from_cli_captures_global_flags() {
        let cli = test_cli();
        let config = AppConfig::from_cli(&cli, None);
        assert_eq!(config.rate_limit, 5);
        assert!(!config.verbose);
    }

    #[test]
    fn from_cli_captures_download_args() {
        let cli = test_cli();
        if let crate::cli::Commands::Download(ref dl) = cli.command {
            let config = AppConfig::from_cli(&cli, Some(dl));
            assert_eq!(config.format, OutputFormat::Md);
            assert_eq!(config.output_dir, PathBuf::from("."));
            assert!(!config.dry_run);
        } else {
            panic!("expected Download command");
        }
    }

    #[test]
    fn from_cli_defaults_when_no_download() {
        let cli = test_cli();
        let config = AppConfig::from_cli(&cli, None);
        assert_eq!(config.format, OutputFormat::Html);
        assert!(!config.download_images);
        assert!(!config.download_files);
    }

    #[test]
    fn allowed_extensions_empty_string() {
        let cli = test_cli();
        let config = AppConfig::from_cli(&cli, None);
        assert!(config.allowed_extensions().is_empty());
    }

    #[test]
    fn allowed_extensions_parses_csv() {
        let cli = test_cli();
        let mut config = AppConfig::from_cli(&cli, None);
        config.file_extensions = "pdf, docx, epub".to_owned();
        let exts = config.allowed_extensions();
        assert_eq!(exts, vec!["pdf", "docx", "epub"]);
    }

    #[test]
    fn config_implements_debug() {
        let cli = test_cli();
        let config = AppConfig::from_cli(&cli, None);
        let debug = format!("{config:?}");
        assert!(debug.contains("AppConfig"));
    }

    #[test]
    fn config_implements_clone() {
        let cli = test_cli();
        let config = AppConfig::from_cli(&cli, None);
        let cloned = config.clone();
        assert_eq!(cloned.rate_limit, config.rate_limit);
    }
}
