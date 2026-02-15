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
//! Non-functional tests: startup performance, binary size, and
//! dependency hygiene checks.

use std::process::Command;
use std::time::Instant;

/// Helper to invoke the compiled binary.
fn cli_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_robustack-dl"))
}

// ---------------------------------------------------------------------------
// Non-Functional: Startup performance
// ---------------------------------------------------------------------------

#[test]
fn startup_completes_under_one_second() {
    let start = Instant::now();
    let output = cli_cmd()
        .arg("--help")
        .output()
        .expect("failed to run binary");
    let elapsed = start.elapsed();

    assert!(output.status.success());
    assert!(
        elapsed.as_millis() < 1000,
        "CLI startup should complete in under 1 second, took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn version_flag_completes_under_500ms() {
    let start = Instant::now();
    let output = cli_cmd()
        .arg("--version")
        .output()
        .expect("failed to run binary");
    let elapsed = start.elapsed();

    assert!(output.status.success());
    assert!(
        elapsed.as_millis() < 500,
        "-V should complete in under 500ms, took {}ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Non-Functional: Binary size
// ---------------------------------------------------------------------------

#[test]
fn binary_exists_and_is_reasonable_size() {
    let binary_path = env!("CARGO_BIN_EXE_robustack-dl");
    let metadata = std::fs::metadata(binary_path)
        .unwrap_or_else(|_| panic!("binary not found at {binary_path}"));

    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

    // A Rust CLI binary with reqwest (cookies, socks, TLS) + tokio is under
    // 100MB in debug mode. Release builds are significantly smaller (~10MB).
    assert!(
        size_mb < 200.0,
        "binary should be under 200MB, was {size_mb:.1}MB"
    );

    // And should exist (not be empty).
    assert!(metadata.len() > 0, "binary should not be empty");
}

// ---------------------------------------------------------------------------
// Non-Functional: Error output quality
// ---------------------------------------------------------------------------

#[test]
fn error_messages_go_to_stderr_not_stdout() {
    let output = cli_cmd()
        .arg("download") // missing required --url
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Error messages should be on stderr, not stdout.
    assert!(!stderr.is_empty(), "error output should appear on stderr");
    assert!(
        stdout.is_empty(),
        "stdout should be empty on error, got: {stdout}"
    );
}

#[test]
fn error_messages_are_human_readable() {
    let output = cli_cmd()
        .arg("download") // missing required --url
        .output()
        .expect("failed to run binary");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain actionable guidance, not just a stack trace.
    assert!(
        stderr.contains("--url") || stderr.contains("Usage"),
        "error should mention the missing argument or usage"
    );
}

// ---------------------------------------------------------------------------
// Non-Functional: Provenance metadata
// ---------------------------------------------------------------------------

#[test]
fn all_source_files_have_provenance_header() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut missing = Vec::new();

    fn check_dir(dir: &std::path::Path, missing: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("failed to read src directory") {
            let entry = entry.expect("failed to read dir entry");
            let path = entry.path();
            if path.is_dir() {
                check_dir(&path, missing);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("failed to read {}", path.display()));
                if !content.contains("AI PROVENANCE") {
                    missing.push(path.display().to_string());
                }
            }
        }
    }

    check_dir(&src_dir, &mut missing);
    assert!(
        missing.is_empty(),
        "The following files are missing the AI Provenance header:\n{}",
        missing.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Non-Functional: No unsafe code
// ---------------------------------------------------------------------------

#[test]
fn no_unsafe_in_source_files() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    fn check_dir(dir: &std::path::Path, violations: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("failed to read directory") {
            let entry = entry.expect("failed to read dir entry");
            let path = entry.path();
            if path.is_dir() {
                check_dir(&path, violations);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("failed to read {}", path.display()));
                for (i, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    // Skip comments and strings.
                    if trimmed.starts_with("//") || trimmed.starts_with("///") {
                        continue;
                    }
                    if trimmed.contains("unsafe ") {
                        violations.push(format!("{}:{}: {}", path.display(), i + 1, trimmed));
                    }
                }
            }
        }
    }

    check_dir(&src_dir, &mut violations);
    assert!(
        violations.is_empty(),
        "Found `unsafe` keyword in source files:\n{}",
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Non-Functional: No println! in source files
// ---------------------------------------------------------------------------

#[test]
fn no_println_in_source_files() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    fn check_dir(dir: &std::path::Path, violations: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("failed to read directory") {
            let entry = entry.expect("failed to read dir entry");
            let path = entry.path();
            if path.is_dir() {
                check_dir(&path, violations);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("failed to read {}", path.display()));
                for (i, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") || trimmed.starts_with("///") {
                        continue;
                    }
                    if trimmed.contains("println!") {
                        violations.push(format!("{}:{}: {}", path.display(), i + 1, trimmed));
                    }
                }
            }
        }
    }

    check_dir(&src_dir, &mut violations);
    assert!(
        violations.is_empty(),
        "Found `println!` in source files (use `tracing` instead):\n{}",
        violations.join("\n")
    );
}
