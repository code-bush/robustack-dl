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
//! Audit handler — verifies archive integrity against the manifest.
//!
//! Reads `manifest.json` from the output directory, then verifies every
//! entry's SHA-256 hash against the file on disk.  Reports mismatches
//! and missing files.

use std::path::Path;

use tracing::{error, info, warn};

use crate::integrity::{self, Manifest};

/// Execute the audit pipeline.
///
/// # Arguments
/// - `manifest_path` — Path to the manifest file to verify.
///
/// # Errors
/// Returns `anyhow::Error` on I/O failure or if any integrity check fails.
pub fn run(manifest_path: &Path) -> anyhow::Result<()> {
    info!(manifest = %manifest_path.display(), "Starting integrity audit");

    let output_dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    // Canonicalize the output directory to resolve symlinks and traversal
    // sequences *before* constructing any file paths from manifest data.
    let canonical_dir = std::fs::canonicalize(output_dir).map_err(|e| {
        anyhow::anyhow!(
            "Cannot resolve output directory {}: {e}",
            output_dir.display()
        )
    })?;

    let manifest = Manifest::load_or_create(&canonical_dir)?;

    if manifest.is_empty() {
        warn!("Manifest is empty — nothing to verify");
        return Ok(());
    }

    let mut pass_count: u32 = 0;
    let mut fail_count: u32 = 0;
    let mut missing_count: u32 = 0;

    for entry in manifest.entries() {
        let safe_name = integrity::sanitize_filename(&entry.local_path);
        let file_path = canonical_dir.join(&safe_name);

        // Verify the resolved path stays inside the canonical directory.
        if let Ok(canonical_file) = std::fs::canonicalize(&file_path) {
            if !canonical_file.starts_with(&canonical_dir) {
                error!(
                    path = %entry.local_path,
                    "Path traversal blocked — file escapes output directory"
                );
                fail_count += 1;
                continue;
            }
        }

        if !file_path.exists() {
            error!(
                path = %entry.local_path,
                expected_hash = %entry.sha256,
                "File missing from archive"
            );
            missing_count += 1;
            continue;
        }

        match integrity::verify_file(&canonical_dir, &entry.local_path, &entry.sha256) {
            Ok(true) => {
                info!(path = %entry.local_path, "Integrity OK");
                pass_count += 1;
            }
            Ok(false) => {
                error!(
                    path = %entry.local_path,
                    expected = %entry.sha256,
                    "Hash mismatch — file may be corrupted"
                );
                fail_count += 1;
            }
            Err(e) => {
                error!(
                    path = %entry.local_path,
                    error = %e,
                    "Failed to verify file"
                );
                fail_count += 1;
            }
        }
    }

    info!(
        total = manifest.len(),
        passed = pass_count,
        failed = fail_count,
        missing = missing_count,
        "Audit complete"
    );

    if fail_count > 0 || missing_count > 0 {
        anyhow::bail!(
            "Audit failed: {fail_count} hash mismatch(es), {missing_count} missing file(s)"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::{ManifestEntry, sha256_hex};

    #[test]
    fn audit_empty_manifest_succeeds() {
        let dir = std::path::PathBuf::from("target/robustack_test_audit_empty");
        let _ = std::fs::create_dir_all(&dir);

        // Write an empty manifest.
        let m = Manifest::default();
        m.save(&dir).unwrap();

        let result = run(&dir.join("manifest.json"));
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn audit_valid_file_passes() {
        let dir = std::path::PathBuf::from("target/robustack_test_audit_valid");
        let _ = std::fs::create_dir_all(&dir);

        let content = b"test content for audit";
        let hash = sha256_hex(content);
        std::fs::write(dir.join("test.html"), content).unwrap();

        let mut m = Manifest::default();
        m.insert(ManifestEntry {
            source_url: "https://example.com".into(),
            sha256: hash,
            local_path: "test.html".into(),
            size: content.len() as u64,
            downloaded_at: "2026-02-15T00:00:00Z".into(),
        });
        m.save(&dir).unwrap();

        let result = run(&dir.join("manifest.json"));
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn audit_corrupted_file_fails() {
        let dir = std::path::PathBuf::from("target/robustack_test_audit_corrupt");
        let _ = std::fs::create_dir_all(&dir);

        std::fs::write(dir.join("test.html"), b"corrupted content").unwrap();

        let mut m = Manifest::default();
        m.insert(ManifestEntry {
            source_url: "https://example.com".into(),
            sha256: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            local_path: "test.html".into(),
            size: 100,
            downloaded_at: "2026-02-15T00:00:00Z".into(),
        });
        m.save(&dir).unwrap();

        let result = run(&dir.join("manifest.json"));
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn audit_missing_file_fails() {
        let dir = std::path::PathBuf::from("target/robustack_test_audit_missing");
        let _ = std::fs::create_dir_all(&dir);

        let mut m = Manifest::default();
        m.insert(ManifestEntry {
            source_url: "https://example.com".into(),
            sha256: "abc123".into(),
            local_path: "nonexistent.html".into(),
            size: 100,
            downloaded_at: "2026-02-15T00:00:00Z".into(),
        });
        m.save(&dir).unwrap();

        let result = run(&dir.join("manifest.json"));
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
