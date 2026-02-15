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
//! - Global flags (auth, proxy, rate, verbose, date filters) live on `Cli`
//!   and propagate to all subcommands.
//! - The version banner matches the project spec exactly.

use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};

/// Multi-line version banner emitted by `robustack-dl -V`.
const VERSION_BANNER: &str = concat!(
    "RoBustack-DL v",
    env!("CARGO_PKG_VERSION"),
    "\n\"Own Your Reading. Byte by Byte.\"\n[GPLv3 + Commercial License]"
);

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Output format for downloaded content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Raw HTML as served by Substack.
    Html,
    /// Converted Markdown (CommonMark).
    Md,
    /// Plain text with headings preserved.
    Txt,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Html => write!(f, "html"),
            Self::Md => write!(f, "md"),
            Self::Txt => write!(f, "txt"),
        }
    }
}

/// Image quality tier for downloaded images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ImageQuality {
    /// Original resolution.
    High,
    /// CDN medium variant (typically 800px wide).
    Medium,
    /// CDN thumbnail variant (typically 400px wide).
    Low,
}

impl std::fmt::Display for ImageQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------

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

    // -------------------------------------------------------------------
    // Global flags — inherited by all subcommands
    // -------------------------------------------------------------------
    /// Cookie name for Substack authentication (substack.sid or connect.sid).
    #[arg(long, global = true, env = "ROBUSTACK_COOKIE_NAME")]
    pub cookie_name: Option<String>,

    /// Cookie value for Substack authentication.
    #[arg(long, global = true, env = "ROBUSTACK_COOKIE_VAL")]
    pub cookie_val: Option<secrecy::SecretString>,

    /// HTTP/SOCKS5 proxy URL (e.g. `socks5://127.0.0.1:1080`).
    #[arg(short = 'x', long, global = true, env = "ROBUSTACK_PROXY")]
    pub proxy: Option<String>,

    /// Maximum requests per second (default: 2).
    #[arg(short = 'r', long, global = true, default_value_t = 2)]
    pub rate: u32,

    /// Enable verbose output (sets RUST_LOG=debug).
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    /// Only process posts published on or after this date (YYYY-MM-DD).
    #[arg(long, global = true)]
    pub after: Option<String>,

    /// Only process posts published on or before this date (YYYY-MM-DD).
    #[arg(long, global = true)]
    pub before: Option<String>,
}

// ---------------------------------------------------------------------------
// Subcommands
// ---------------------------------------------------------------------------

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Download and archive Substack content.
    Download(DownloadArgs),
    /// List the posts of a Substack.
    List(ListArgs),
    /// Verify archive integrity against a manifest.
    Audit(AuditArgs),
    /// Generate shell completions to stdout.
    Completions(CompletionsArgs),
    /// Display the current version of the app.
    Version,
}

// ---------------------------------------------------------------------------
// Download arguments
// ---------------------------------------------------------------------------

/// Arguments for the `download` subcommand.
#[allow(clippy::struct_excessive_bools)]
#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Substack URL to download (e.g. `https://example.substack.com`).
    #[arg(short, long)]
    pub url: String,

    /// Output directory for downloaded content.
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// Output format: "html", "md", "txt".
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Html)]
    pub format: OutputFormat,

    /// Enable dry run — show what would be downloaded without writing files.
    #[arg(short = 'd', long)]
    pub dry_run: bool,

    /// Download images locally and update content references.
    #[arg(long)]
    pub download_images: bool,

    /// Directory name for downloaded images (relative to output).
    #[arg(long, default_value = "images")]
    pub images_dir: String,

    /// Image quality tier: "high", "medium", "low".
    #[arg(long, value_enum, default_value_t = ImageQuality::High)]
    pub image_quality: ImageQuality,

    /// Download file attachments locally and update content references.
    #[arg(long)]
    pub download_files: bool,

    /// Directory name for downloaded attachments (relative to output).
    #[arg(long, default_value = "files")]
    pub files_dir: String,

    /// Comma-separated list of file extensions to download (e.g. "pdf,docx").
    /// If empty, all attachment types are downloaded.
    #[arg(long, default_value = "")]
    pub file_extensions: String,

    /// Append the original post URL at the end of each downloaded file.
    #[arg(long)]
    pub add_source_url: bool,

    /// Create an archive index page linking all downloaded posts.
    #[arg(long)]
    pub create_archive: bool,
}

// ---------------------------------------------------------------------------
// Audit arguments
// ---------------------------------------------------------------------------

/// Arguments for the `audit` subcommand.
#[derive(Args, Debug)]
pub struct AuditArgs {
    /// Path to the manifest.json to verify against.
    #[arg(short, long, default_value = "manifest.json")]
    pub manifest: PathBuf,
}

// ---------------------------------------------------------------------------
// List arguments
// ---------------------------------------------------------------------------

/// Arguments for the `list` subcommand.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Substack URL to list posts from (e.g. `https://example.substack.com`).
    #[arg(short, long)]
    pub url: String,
}

// ---------------------------------------------------------------------------
// Completions arguments
// ---------------------------------------------------------------------------

/// Arguments for the `completions` subcommand.
#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for (bash, zsh, fish, powershell, elvish).
    #[arg(short, long)]
    pub shell: clap_complete::Shell,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // -----------------------------------------------------------------------
    // Unit: Parser validation — download
    // -----------------------------------------------------------------------

    #[test]
    fn parse_download_with_all_args() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "download",
            "--url",
            "https://example.substack.com",
            "--output",
            "/tmp/out",
            "--format",
            "md",
            "--download-images",
            "--image-quality",
            "medium",
            "--download-files",
            "--dry-run",
            "--add-source-url",
            "--create-archive",
        ])
        .expect("valid args should parse");

        match cli.command {
            Commands::Download(args) => {
                assert_eq!(args.url, "https://example.substack.com");
                assert_eq!(args.output, PathBuf::from("/tmp/out"));
                assert_eq!(args.format, OutputFormat::Md);
                assert!(args.download_images);
                assert_eq!(args.image_quality, ImageQuality::Medium);
                assert!(args.download_files);
                assert!(args.dry_run);
                assert!(args.add_source_url);
                assert!(args.create_archive);
            }
            _ => panic!("expected Download command"),
        }
    }

    #[test]
    fn parse_download_defaults() {
        let cli = Cli::try_parse_from(["robustack-dl", "download", "--url", "https://example.com"])
            .expect("valid args should parse");

        match cli.command {
            Commands::Download(args) => {
                assert_eq!(args.output, PathBuf::from("."));
                assert_eq!(args.format, OutputFormat::Html);
                assert_eq!(args.image_quality, ImageQuality::High);
                assert_eq!(args.images_dir, "images");
                assert_eq!(args.files_dir, "files");
                assert!(!args.download_images);
                assert!(!args.download_files);
                assert!(!args.dry_run);
                assert!(!args.add_source_url);
                assert!(!args.create_archive);
            }
            _ => panic!("expected Download command"),
        }
    }

    #[test]
    fn parse_download_format_txt() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "download",
            "--url",
            "https://x.com",
            "--format",
            "txt",
        ])
        .expect("valid args should parse");

        match cli.command {
            Commands::Download(args) => assert_eq!(args.format, OutputFormat::Txt),
            _ => panic!("expected Download command"),
        }
    }

    #[test]
    fn parse_download_file_extensions() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "download",
            "--url",
            "https://x.com",
            "--file-extensions",
            "pdf,docx",
        ])
        .expect("valid args should parse");

        match cli.command {
            Commands::Download(args) => assert_eq!(args.file_extensions, "pdf,docx"),
            _ => panic!("expected Download command"),
        }
    }

    // -----------------------------------------------------------------------
    // Unit: Global flags
    // -----------------------------------------------------------------------

    #[test]
    fn parse_global_proxy_flag() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "--proxy",
            "socks5://127.0.0.1:1080",
            "download",
            "--url",
            "https://x.com",
        ])
        .expect("valid args should parse");

        assert_eq!(cli.proxy, Some("socks5://127.0.0.1:1080".to_string()));
    }

    #[test]
    fn parse_global_rate_flag() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "--rate",
            "5",
            "download",
            "--url",
            "https://x.com",
        ])
        .expect("valid args should parse");

        assert_eq!(cli.rate, 5);
    }

    #[test]
    fn parse_rate_defaults_to_two() {
        let cli = Cli::try_parse_from(["robustack-dl", "download", "--url", "https://x.com"])
            .expect("valid args should parse");

        assert_eq!(cli.rate, 2);
    }

    #[test]
    fn parse_global_verbose_flag() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "--verbose",
            "download",
            "--url",
            "https://x.com",
        ])
        .expect("valid args should parse");

        assert!(cli.verbose);
    }

    #[test]
    fn parse_global_date_filters() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "--after",
            "2024-01-01",
            "--before",
            "2024-12-31",
            "download",
            "--url",
            "https://x.com",
        ])
        .expect("valid args should parse");

        assert_eq!(cli.after, Some("2024-01-01".to_string()));
        assert_eq!(cli.before, Some("2024-12-31".to_string()));
    }

    #[test]
    fn parse_global_cookie_auth() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "--cookie-name",
            "substack.sid",
            "--cookie-val",
            "abc123secret",
            "download",
            "--url",
            "https://x.com",
        ])
        .expect("valid args should parse");

        assert_eq!(cli.cookie_name, Some("substack.sid".to_string()));
        assert!(cli.cookie_val.is_some());
    }

    #[test]
    fn parse_global_config_flag() {
        let cli = Cli::try_parse_from(["robustack-dl", "--config", "/etc/robustack.toml", "audit"])
            .expect("valid args should parse");

        assert_eq!(cli.config, Some(PathBuf::from("/etc/robustack.toml")));
    }

    #[test]
    fn config_defaults_to_none() {
        let cli = Cli::try_parse_from(["robustack-dl", "audit"]).expect("valid args should parse");

        assert!(cli.config.is_none());
    }

    // -----------------------------------------------------------------------
    // Unit: Audit
    // -----------------------------------------------------------------------

    #[test]
    fn parse_audit_default_manifest() {
        let cli = Cli::try_parse_from(["robustack-dl", "audit"]).expect("valid args should parse");

        match cli.command {
            Commands::Audit(args) => {
                assert_eq!(args.manifest, PathBuf::from("manifest.json"));
            }
            _ => panic!("expected Audit command"),
        }
    }

    #[test]
    fn parse_audit_custom_manifest() {
        let cli = Cli::try_parse_from(["robustack-dl", "audit", "--manifest", "/data/custom.json"])
            .expect("valid args should parse");

        match cli.command {
            Commands::Audit(args) => {
                assert_eq!(args.manifest, PathBuf::from("/data/custom.json"));
            }
            _ => panic!("expected Audit command"),
        }
    }

    // -----------------------------------------------------------------------
    // Unit: List
    // -----------------------------------------------------------------------

    #[test]
    fn parse_list_with_url() {
        let cli = Cli::try_parse_from([
            "robustack-dl",
            "list",
            "--url",
            "https://example.substack.com",
        ])
        .expect("valid args should parse");

        match cli.command {
            Commands::List(args) => {
                assert_eq!(args.url, "https://example.substack.com");
            }
            _ => panic!("expected List command"),
        }
    }

    #[test]
    fn list_missing_url_is_error() {
        let result = Cli::try_parse_from(["robustack-dl", "list"]);
        assert!(result.is_err(), "list without --url should fail");
    }

    // -----------------------------------------------------------------------
    // Unit: Version
    // -----------------------------------------------------------------------

    #[test]
    fn parse_version_subcommand() {
        let cli = Cli::try_parse_from(["robustack-dl", "version"])
            .expect("valid args should parse");

        assert!(matches!(cli.command, Commands::Version));
    }

    // -----------------------------------------------------------------------
    // Unit: Completions
    // -----------------------------------------------------------------------

    #[test]
    fn parse_completions_bash() {
        let cli = Cli::try_parse_from(["robustack-dl", "completions", "--shell", "bash"])
            .expect("valid args should parse");

        match cli.command {
            Commands::Completions(args) => {
                assert_eq!(args.shell, clap_complete::Shell::Bash);
            }
            _ => panic!("expected Completions command"),
        }
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
        let result =
            Cli::try_parse_from(["robustack-dl", "completions", "--shell", "invalid_shell"]);
        assert!(result.is_err(), "invalid shell name should fail");
    }

    #[test]
    fn unknown_subcommand_is_error() {
        let result = Cli::try_parse_from(["robustack-dl", "explode"]);
        assert!(result.is_err(), "unknown subcommand should fail");
    }

    #[test]
    fn invalid_format_is_error() {
        let result = Cli::try_parse_from([
            "robustack-dl",
            "download",
            "--url",
            "https://x.com",
            "--format",
            "pdf",
        ]);
        assert!(result.is_err(), "invalid format should fail");
    }

    #[test]
    fn invalid_image_quality_is_error() {
        let result = Cli::try_parse_from([
            "robustack-dl",
            "download",
            "--url",
            "https://x.com",
            "--image-quality",
            "ultra",
        ]);
        assert!(result.is_err(), "invalid image quality should fail");
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
    // Unit: Enum Display
    // -----------------------------------------------------------------------

    #[test]
    fn output_format_display() {
        assert_eq!(OutputFormat::Html.to_string(), "html");
        assert_eq!(OutputFormat::Md.to_string(), "md");
        assert_eq!(OutputFormat::Txt.to_string(), "txt");
    }

    #[test]
    fn image_quality_display() {
        assert_eq!(ImageQuality::High.to_string(), "high");
        assert_eq!(ImageQuality::Medium.to_string(), "medium");
        assert_eq!(ImageQuality::Low.to_string(), "low");
    }

    // -----------------------------------------------------------------------
    // Unit: Clap internal consistency
    // -----------------------------------------------------------------------

    #[test]
    fn clap_debug_assert() {
        Cli::command().debug_assert();
    }
}
