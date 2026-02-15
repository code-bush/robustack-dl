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
//! CLI definitions for RoBustack-DL using clap v4 (derive).
//!
//! # Design Decisions
//! - `Commands` is **not** `Option` — a subcommand is always required.
//! - Filesystem paths use `PathBuf`, not `String` (avoids primitive obsession).
//! - The version banner matches the project spec exactly.

use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand};

/// Multi-line version banner emitted by `robustack-dl -V`.
const VERSION_BANNER: &str = concat!(
    "RoBustack-DL v",
    env!("CARGO_PKG_VERSION"),
    "\n\"Own Your Reading. Byte by Byte.\"\n[GPLv3 + Commercial License]"
);

/// Top-level CLI parser.
#[derive(Parser, Debug)]
#[command(author, version = VERSION_BANNER, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Subcommand to execute (required).
    #[command(subcommand)]
    pub command: Commands,

    /// Path to an optional config.toml override.
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Download and archive web content.
    Download(DownloadArgs),
    /// Verify archive integrity against a manifest.
    Audit(AuditArgs),
    /// Generate shell completions to stdout.
    Completions(CompletionsArgs),
}

/// Arguments for the `download` subcommand.
#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Target URL to download.
    #[arg(short, long)]
    pub url: String, // TODO: Replace with url::Url newtype after adding `url` crate

    /// Output directory for downloaded content.
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,
}

/// Arguments for the `audit` subcommand.
#[derive(Args, Debug)]
pub struct AuditArgs {
    /// Path to the manifest.json to verify against.
    #[arg(short, long, default_value = "manifest.json")]
    pub manifest: PathBuf,
}

/// Arguments for the `completions` subcommand.
#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for (bash, zsh, fish, powershell, elvish).
    #[arg(short, long)]
    pub shell: clap_complete::Shell,
}

/// Generate and write shell completions to stdout.
///
/// # Panics
/// None — `clap_complete::generate` writes to the provided `Write` impl.
pub fn print_completions(shell: clap_complete::Shell) {
    clap_complete::generate(
        shell,
        &mut Cli::command(),
        env!("CARGO_PKG_NAME"),
        &mut std::io::stdout(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // -----------------------------------------------------------------------
    // Unit: Parser validation
    // -----------------------------------------------------------------------

    #[test]
    fn parse_download_with_all_args() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "download",
            "--url",
            "https://example.com",
            "--output",
            "/tmp/out",
        ])
        .expect("valid args should parse");

        match cli.command {
            Commands::Download(args) => {
                assert_eq!(args.url, "https://example.com");
                assert_eq!(args.output, PathBuf::from("/tmp/out"));
            }
            _ => panic!("expected Download command"),
        }
    }

    #[test]
    fn parse_download_default_output() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "download",
            "--url",
            "https://example.com",
        ])
        .expect("valid args should parse");

        match cli.command {
            Commands::Download(args) => {
                assert_eq!(args.output, PathBuf::from("."));
            }
            _ => panic!("expected Download command"),
        }
    }

    #[test]
    fn parse_audit_default_manifest() {
        let cli =
            Cli::try_parse_from(["robustack-dl", "audit"]).expect("valid args should parse");

        match cli.command {
            Commands::Audit(args) => {
                assert_eq!(args.manifest, PathBuf::from("manifest.json"));
            }
            _ => panic!("expected Audit command"),
        }
    }

    #[test]
    fn parse_audit_custom_manifest() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "audit",
            "--manifest",
            "/data/custom.json",
        ])
        .expect("valid args should parse");

        match cli.command {
            Commands::Audit(args) => {
                assert_eq!(args.manifest, PathBuf::from("/data/custom.json"));
            }
            _ => panic!("expected Audit command"),
        }
    }

    #[test]
    fn parse_completions_bash() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "completions",
            "--shell",
            "bash",
        ])
        .expect("valid args should parse");

        match cli.command {
            Commands::Completions(args) => {
                assert_eq!(args.shell, clap_complete::Shell::Bash);
            }
            _ => panic!("expected Completions command"),
        }
    }

    #[test]
    fn parse_global_config_flag() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "--config",
            "/etc/robustack.toml",
            "audit",
        ])
        .expect("valid args should parse");

        assert_eq!(cli.config, Some(PathBuf::from("/etc/robustack.toml")));
    }

    #[test]
    fn config_defaults_to_none() {
        let cli =
            Cli::try_parse_from(["robustack-dl", "audit"]).expect("valid args should parse");

        assert!(cli.config.is_none());
    }

    // -----------------------------------------------------------------------
    // Unit: Error cases
    // -----------------------------------------------------------------------

    #[test]
    fn missing_subcommand_is_error() {
        let result = Cli::try_parse_from(["robustack-dl"]);
        assert!(result.is_err(), "no subcommand should be an error");
    }

    #[test]
    fn download_missing_url_is_error() {
        let result = Cli::try_parse_from(["robustack-dl", "download"]);
        assert!(result.is_err(), "download without --url should fail");
    }

    #[test]
    fn completions_missing_shell_is_error() {
        let result = Cli::try_parse_from(["robustack-dl", "completions"]);
        assert!(result.is_err(), "completions without --shell should fail");
    }

    #[test]
    fn invalid_shell_is_error() {
        let result = Cli::try_parse_from([
            "robustack-dl",
            "completions",
            "--shell",
            "invalid_shell",
        ]);
        assert!(result.is_err(), "invalid shell name should fail");
    }

    #[test]
    fn unknown_subcommand_is_error() {
        let result = Cli::try_parse_from(["robustack-dl", "explode"]);
        assert!(result.is_err(), "unknown subcommand should fail");
    }

    // -----------------------------------------------------------------------
    // Unit: Version banner
    // -----------------------------------------------------------------------

    #[test]
    fn version_banner_contains_project_name() {
        assert!(
            VERSION_BANNER.contains("RoBustack-DL"),
            "banner should contain project name"
        );
    }

    #[test]
    fn version_banner_contains_tagline() {
        assert!(
            VERSION_BANNER.contains("Own Your Reading. Byte by Byte."),
            "banner should contain tagline"
        );
    }

    #[test]
    fn version_banner_contains_license() {
        assert!(
            VERSION_BANNER.contains("GPLv3 + Commercial License"),
            "banner should contain license"
        );
    }

    #[test]
    fn version_banner_contains_version() {
        assert!(
            VERSION_BANNER.contains(env!("CARGO_PKG_VERSION")),
            "banner should contain crate version"
        );
    }

    // -----------------------------------------------------------------------
    // Unit: clap internal consistency
    // -----------------------------------------------------------------------

    #[test]
    fn clap_debug_assert() {
        // clap's own internal assertion that the command definition is valid.
        Cli::command().debug_assert();
    }
}
