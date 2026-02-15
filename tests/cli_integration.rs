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
//! Functional (integration) tests for the robustack-dl CLI binary.
//!
//! These tests invoke the compiled binary as a subprocess to validate
//! end-to-end behavior. We use `wiremock` to simulate the Substack API
//! so that tests are deterministic and network-independent.

use std::process::Command;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to invoke the compiled binary.
fn cli_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_robustack-dl"))
}

/// Helper to collect stdout from a successful command.
fn stdout_of(args: &[&str]) -> String {
    let output = cli_cmd().args(args).output().expect("failed to run binary");
    assert!(output.status.success(), "expected exit 0 for {args:?}");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to collect stderr from a failed command.
fn stderr_of(args: &[&str]) -> String {
    let output = cli_cmd().args(args).output().expect("failed to run binary");
    assert!(
        !output.status.success(),
        "expected non-zero exit for {args:?}"
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Helper to start a mock server that accepts posts listing and generic requests.
async fn start_mock() -> MockServer {
    let mock_server = MockServer::start().await;

    // Default response for listing posts (empty list, success).
    Mock::given(method("GET"))
        .and(path("/api/v1/posts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "posts": [],
            "total": 0,
            "limit": 50,
            "offset": 0
        })))
        .mount(&mock_server)
        .await;

    // Fail-safe for other requests: return 404 to uncover issues, 
    // unless strictly needed.
    mock_server
}

// ===========================================================================
// Help output
// ===========================================================================

#[test]
fn help_flag_exits_zero() {
    let stdout = stdout_of(&["--help"]);
    assert!(stdout.contains("Usage"), "help output should contain Usage");
}

#[test]
fn help_lists_all_subcommands() {
    let stdout = stdout_of(&["--help"]);
    assert!(stdout.contains("download"), "help should list download");
    assert!(stdout.contains("list"), "help should list list");
    assert!(stdout.contains("audit"), "help should list audit");
    assert!(stdout.contains("version"), "help should list version");
}

#[test]
fn download_help_exits_zero() {
    let stdout = stdout_of(&["download", "--help"]);
    assert!(stdout.contains("--url"), "download help should list --url");
}

#[test]
fn download_help_lists_new_flags() {
    let stdout = stdout_of(&["download", "--help"]);
    for flag in [
        "--format",
        "--dry-run",
        "--download-images",
        "--images-dir",
        "--download-files",
        "--create-archive",
    ] {
        assert!(stdout.contains(flag), "download help should list {flag}");
    }
}

// ===========================================================================
// Version output
// ===========================================================================

#[test]
fn version_flag_shows_banner() {
    let stdout = stdout_of(&["--version"]);
    assert!(stdout.contains("RoBustack-DL"), "version project name");
    assert!(stdout.contains("GPLv3"), "version license");
}

// ===========================================================================
// Error exit codes
// ===========================================================================

#[test]
fn no_subcommand_exits_nonzero() {
    let output = cli_cmd().output().expect("failed to run binary");
    assert!(!output.status.success());
}

#[test]
fn download_without_url_exits_nonzero() {
    let stderr = stderr_of(&["download"]);
    assert!(stderr.contains("--url"));
}

// ===========================================================================
// Download subcommand smoke tests (Async with WireMock)
// ===========================================================================

#[tokio::test]
async fn download_with_valid_args_exits_zero() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args(["download", "--url", &mock_server.uri()])
        .output()
        .expect("failed");
    assert!(output.status.success(), "download valid args failed: {:?}", output);
}

#[tokio::test]
async fn download_dry_run_exits_zero() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args(["download", "--url", &mock_server.uri(), "--dry-run"])
        .output()
        .expect("failed");
    assert!(output.status.success());
}

#[tokio::test]
async fn download_dry_run_creates_no_files() {
    let mock_server = start_mock().await;
    let dir = std::env::temp_dir().join("robustack_test_dry_run_no_files");
    let _ = std::fs::remove_dir_all(&dir);

    let output = cli_cmd()
        .args([
            "download",
            "--url",
            &mock_server.uri(),
            "--output",
            dir.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .expect("failed");

    assert!(output.status.success());
    assert!(!dir.exists(), "dry run should not create the output directory");
}

#[tokio::test]
async fn download_with_format_md_exits_zero() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args(["download", "--url", &mock_server.uri(), "--format", "md"])
        .output()
        .expect("failed");
    assert!(output.status.success());
}

#[tokio::test]
async fn download_with_image_options_exits_zero() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args([
            "download",
            "--url",
            &mock_server.uri(),
            "--download-images",
            "--image-quality",
            "low",
        ])
        .output()
        .expect("failed");
    assert!(output.status.success());
}

#[tokio::test]
async fn download_with_archive_flag_exits_zero() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args([
            "download",
            "--url",
            &mock_server.uri(),
            "--create-archive",
        ])
        .output()
        .expect("failed");
    assert!(output.status.success());
}

// ===========================================================================
// List subcommand (Async with WireMock)
// ===========================================================================

#[test]
fn list_with_url_without_server_fails() {
    // Proves that without mock server, it fails (fail-fast check)
    // We use a reserved invalid IP to ensure connection failure
    let output = cli_cmd()
        .args(["list", "--url", "http://127.0.0.1:0"])
        .output()
        .expect("failed");
    assert!(!output.status.success());
}

#[tokio::test]
async fn list_with_url_exits_zero() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args(["list", "--url", &mock_server.uri()])
        .output()
        .expect("failed");
    assert!(output.status.success(), "list with valid url failed: {:?}", output);
}

// ===========================================================================
// Global flags (Async wrappers where needed for download)
// ===========================================================================

#[tokio::test]
async fn verbose_flag_accepted() {
    let mock_server = start_mock().await;
    // Verbose should effectively just run the command with more logging.
    let output = cli_cmd()
        .args([
            "--verbose",
            "download",
            "--url",
            &mock_server.uri(),
            "--dry-run",
        ])
        .output()
        .expect("failed");
    assert!(output.status.success());
}

#[tokio::test]
async fn rate_flag_accepted() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args([
            "--rate",
            "10",
            "download",
            "--url",
            &mock_server.uri(),
            "--dry-run",
        ])
        .output()
        .expect("failed");
    assert!(output.status.success());
}

#[tokio::test]
async fn date_filter_flags_accepted() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args([
            "--after",
            "2024-01-01",
            "--before",
            "2025-12-31",
            "download",
            "--url",
            &mock_server.uri(),
            "--dry-run",
        ])
        .output()
        .expect("failed");
    assert!(output.status.success());
}

#[tokio::test]
async fn cookie_auth_flags_accepted() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args([
            "--cookie-name",
            "substack.sid",
            "--cookie-val",
            "test_token",
            "download",
            "--url",
            &mock_server.uri(),
            "--dry-run",
        ])
        .output()
        .expect("failed");
    assert!(output.status.success());
}

#[tokio::test]
async fn short_flags_accepted() {
    let mock_server = start_mock().await;
    let output = cli_cmd()
        .args([
            "-v",
            "-r",
            "3",
            "download",
            "-u",
            &mock_server.uri(),
            "-o",
            ".",
            "-f",
            "txt",
            "-d",
        ])
        .output()
        .expect("failed");
    assert!(output.status.success());
}
