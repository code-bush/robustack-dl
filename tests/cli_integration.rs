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
//! end-to-end behavior: exit codes, stdout/stderr output, and flag handling.

use std::process::Command;

/// Helper to invoke the compiled binary.
fn cli_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_robustack-dl"))
}

// ---------------------------------------------------------------------------
// Functional: Help output
// ---------------------------------------------------------------------------

#[test]
fn help_flag_exits_zero() {
    let output = cli_cmd()
        .arg("--help")
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "--help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"), "help output should contain Usage");
}

#[test]
fn help_lists_all_subcommands() {
    let output = cli_cmd()
        .arg("--help")
        .output()
        .expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("download"), "help should list download");
    assert!(stdout.contains("audit"), "help should list audit");
    assert!(
        stdout.contains("completions"),
        "help should list completions"
    );
}

#[test]
fn download_help_exits_zero() {
    let output = cli_cmd()
        .args(["download", "--help"])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "download --help should exit 0");
}

#[test]
fn audit_help_exits_zero() {
    let output = cli_cmd()
        .args(["audit", "--help"])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "audit --help should exit 0");
}

// ---------------------------------------------------------------------------
// Functional: Version output
// ---------------------------------------------------------------------------

#[test]
fn version_flag_shows_banner() {
    let output = cli_cmd()
        .arg("--version")
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "--version should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("RoBustack-DL"),
        "version should contain project name"
    );
    assert!(
        stdout.contains("Own Your Reading. Byte by Byte."),
        "version should contain tagline"
    );
    assert!(
        stdout.contains("GPLv3 + Commercial License"),
        "version should contain license"
    );
}

// ---------------------------------------------------------------------------
// Functional: Error exit codes
// ---------------------------------------------------------------------------

#[test]
fn no_subcommand_exits_nonzero() {
    let output = cli_cmd().output().expect("failed to run binary");
    assert!(
        !output.status.success(),
        "no subcommand should exit non-zero"
    );
}

#[test]
fn download_without_url_exits_nonzero() {
    let output = cli_cmd()
        .arg("download")
        .output()
        .expect("failed to run binary");
    assert!(
        !output.status.success(),
        "download without --url should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--url"),
        "error message should mention --url"
    );
}

#[test]
fn unknown_subcommand_exits_nonzero() {
    let output = cli_cmd()
        .arg("nonexistent")
        .output()
        .expect("failed to run binary");
    assert!(
        !output.status.success(),
        "unknown subcommand should exit non-zero"
    );
}

// ---------------------------------------------------------------------------
// Functional: Subcommand execution (smoke tests)
// ---------------------------------------------------------------------------

#[test]
fn download_with_valid_args_exits_zero() {
    let output = cli_cmd()
        .args(["download", "--url", "https://example.com"])
        .output()
        .expect("failed to run binary");
    assert!(
        output.status.success(),
        "download with valid args should exit 0"
    );
}

#[test]
fn audit_with_defaults_exits_zero() {
    let output = cli_cmd()
        .arg("audit")
        .output()
        .expect("failed to run binary");
    // Audit currently succeeds even without a manifest file (stub)
    assert!(output.status.success(), "audit with defaults should exit 0");
}

// ---------------------------------------------------------------------------
// Functional: Global config flag
// ---------------------------------------------------------------------------

#[test]
fn config_flag_accepted_before_subcommand() {
    let output = cli_cmd()
        .args(["--config", "/nonexistent/path.toml", "audit"])
        .output()
        .expect("failed to run binary");
    // Config loading is a stub, so this should still succeed
    assert!(
        output.status.success(),
        "--config before subcommand should be accepted"
    );
}

#[test]
fn config_flag_accepted_after_subcommand() {
    // clap with global = true should accept --config after the subcommand too
    let output = cli_cmd()
        .args(["audit", "--config", "/nonexistent/path.toml"])
        .output()
        .expect("failed to run binary");
    assert!(
        output.status.success(),
        "--config after subcommand should be accepted"
    );
}
