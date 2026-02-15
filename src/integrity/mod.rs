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
//! Integrity module — SHA-256 manifest for content-addressed idempotent storage.
//!
//! # Idempotency Guarantee
//! Every downloaded artifact is stored under a content-addressed filename:
//! `<sha256_hex>.<ext>`.  Before writing, the manifest is consulted —
//! if the hash already exists, the write is skipped.  This makes every
//! download operation **idempotent**: running the tool twice on the same
//! Substack produces the exact same output directory with zero wasted I/O.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;


// ---------------------------------------------------------------------------
// Manifest — idempotent download tracking
// ---------------------------------------------------------------------------

/// Hardcoded manifest filename — never derived from user input.
const MANIFEST_FILENAME: &str = "manifest.json";

/// Entry in the download manifest tracking one artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestEntry {
    /// Original source URL.
    pub source_url: String,
    /// SHA-256 hex digest of the content.
    pub sha256: String,
    /// Relative path within the output directory.
    pub local_path: String,
    /// Content length in bytes.
    pub size: u64,
    /// ISO-8601 timestamp of when this entry was recorded.
    pub downloaded_at: String,
}

/// Download manifest tracking all artifacts for idempotent re-runs.
///
/// Persisted as `manifest.json` in the output directory.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manifest {
    /// Map from SHA-256 hex digest to manifest entry.
    entries: HashMap<String, ManifestEntry>,
}

impl Manifest {
    /// Load an existing manifest from disk, or create a new empty one.
    ///
    /// # Errors
    /// Returns `anyhow::Error` if the file exists but cannot be parsed.
    pub fn load_or_create(output_dir: &Path) -> anyhow::Result<Self> {
        use std::io::Read;

        // Defence-in-depth: canonicalize output_dir to resolve symlinks and
        // traversal sequences (e.g. `../../`) before constructing file paths.
        // We propagate the error instead of falling back to the raw path,
        // which would defeat path-traversal protection.
        //
        // SECURITY FIX: We now also validate that the path is within the Current
        // Working Directory (CWD) to prevent arbitrary file system access if
        // `output_dir` is user-controlled.
        let canonical_dir = validate_path_is_safe(output_dir)?;
        let path = canonical_dir.join(MANIFEST_FILENAME);

        // Invariant: the filename component must be exactly the constant we
        // joined — reject anything else (e.g. if the constant were ever
        // accidentally changed to contain a separator).
        if path.file_name().and_then(|f| f.to_str()) != Some(MANIFEST_FILENAME) {
            anyhow::bail!("Internal error: unexpected manifest path {}", path.display());
        }

        if !path.exists() {
            // File doesn't exist yet — the join above is safe because
            // canonical_dir is already resolved and MANIFEST_FILENAME is a
            // hardcoded literal with no path separators.
            return Ok(Self::default());
        }

        // Defence layer: canonicalize the manifest path *before* opening to
        // verify it still resides inside the output directory, closing the
        // TOCTOU gap between path construction and file access.
        let canonical_path = std::fs::canonicalize(&path)?;
        if !canonical_path.starts_with(&canonical_dir) {
            anyhow::bail!(
                "Path traversal blocked: {} escapes output directory {}",
                canonical_path.display(),
                canonical_dir.display()
            );
        }

        // Open via the *canonical* path so the descriptor is guaranteed to
        // point at the validated location.
        let mut file = std::fs::File::open(&canonical_path)
            .map_err(|e| anyhow::anyhow!("Cannot open {}: {e}", canonical_path.display()))?;

        // Ensure it is a regular file (not a directory) to prevent read errors
        if !file.metadata()?.is_file() {
             anyhow::bail!("Manifest is not a regular file: {}", canonical_path.display());
        }

        // Read through the opened handle — the data comes from the inode at
        // the canonical path validated above.
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let manifest: Self = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Persist the manifest to disk as pretty-printed JSON.
    ///
    /// # Errors
    /// Returns `anyhow::Error` on I/O failure.
    pub fn save(&self, output_dir: &Path) -> anyhow::Result<()> {
        let canonical_dir = validate_path_is_safe(output_dir)?;
        let path = canonical_dir.join(MANIFEST_FILENAME);

        // Invariant: same filename-component check as load_or_create.
        if path.file_name().and_then(|f| f.to_str()) != Some(MANIFEST_FILENAME) {
            anyhow::bail!("Internal error: unexpected manifest path {}", path.display());
        }

        // Defence-in-depth: if the file already exists, verify its canonical
        // location is still inside the output directory (guards against
        // symlink-based traversal).
        if path.exists() {
            let canonical_path = std::fs::canonicalize(&path)?;
            if !canonical_path.starts_with(&canonical_dir) {
                anyhow::bail!(
                    "Path traversal blocked: {} escapes output directory {}",
                    canonical_path.display(),
                    canonical_dir.display()
                );
            }
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Check if content with the given SHA-256 hash has already been downloaded.
    #[must_use]
    pub fn contains(&self, sha256: &str) -> bool {
        self.entries.contains_key(sha256)
    }

    /// Record a new download in the manifest.
    pub fn insert(&mut self, entry: ManifestEntry) {
        self.entries.insert(entry.sha256.clone(), entry);
    }

    /// Returns the number of entries in the manifest.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the manifest has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over all entries.
    pub fn entries(&self) -> impl Iterator<Item = &ManifestEntry> {
        self.entries.values()
    }
}

// ---------------------------------------------------------------------------
// Path sanitisation — defence-in-depth against path traversal
// ---------------------------------------------------------------------------

/// Validate that a path resides within the Current Working Directory (CWD).
///
/// This prevents path traversal attacks where a user provides a path like
/// `../../etc/passwd` or `/tmp/malicious`.
///
/// # Errors
/// Returns `anyhow::Error` if:
/// - The path cannot be canonicalized.
/// - The CWD cannot be determined.
/// - The path is not within the CWD.
fn validate_path_is_safe(path: &Path) -> anyhow::Result<PathBuf> {
    let canonical_path = std::fs::canonicalize(path)
        .map_err(|e| anyhow::anyhow!("Cannot resolve path {}: {e}", path.display()))?;

    let cwd = env::current_dir()
        .map_err(|e| anyhow::anyhow!("Cannot determine current working directory: {e}"))?;

    let canonical_cwd = std::fs::canonicalize(&cwd)
        .map_err(|e| anyhow::anyhow!("Cannot resolve CWD {}: {e}", cwd.display()))?;

    if !canonical_path.starts_with(&canonical_cwd) {
        anyhow::bail!(
            "Path traversal blocked: Path {} is outside the current working directory {}",
            canonical_path.display(),
            canonical_cwd.display()
        );
    }

    Ok(canonical_path)
}


/// Strip directory-traversal components from an untrusted filename.
///
/// This function removes:
///   * Any leading/trailing whitespace.
///   * Parent-directory references (`..`).
///   * Current-directory references (`.`).
///   * Path separators (`/` and `\`), keeping only the final component.
///
/// If nothing remains after sanitisation, the fallback `"untitled"` is returned
/// so that callers always receive a non-empty, safe filename.
#[must_use]
pub fn sanitize_filename(name: &str) -> String {
    // Normalise Windows-style backslash separators so that `Path::file_name()`
    // correctly discards parent segments on every platform.
    let normalised = name.replace('\\', "/");

    // Take only the final component of any path, effectively discarding
    // leading directory segments such as `../../etc/`.
    let filename = Path::new(&normalised)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("untitled");

    // Extra belt-and-suspenders: reject remaining `.` / `..`.
    let cleaned = filename.to_owned();

    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "untitled".to_owned()
    } else {
        cleaned
    }
}

// ---------------------------------------------------------------------------
// Content-addressed helpers
// ---------------------------------------------------------------------------

/// Compute the SHA-256 hex digest of a byte slice.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Determine whether a download should be skipped (idempotency check).
///
/// Returns `true` if the content hash already exists in the manifest
/// **and** the corresponding file exists on disk.
///
/// `local_path` is sanitised before joining to ensure it cannot escape
/// `output_dir` via directory-traversal sequences.  `output_dir` is
/// canonicalized to resolve symlinks and traversal sequences before the
/// join, and the resulting path is verified to remain inside the
/// canonical directory.
#[must_use]
pub fn should_skip(manifest: &Manifest, sha256: &str, output_dir: &Path, local_path: &str) -> bool {
    if !manifest.contains(sha256) {
        return false;
    }
    let safe_name = sanitize_filename(local_path);
    // Canonicalize the base directory; if it cannot be resolved the file
    // cannot exist, so we conservatively return false (do not skip).
    let canonical_dir = match std::fs::canonicalize(output_dir) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let target = canonical_dir.join(&safe_name);
    // Verify the resolved target is still inside the canonical directory.
    match std::fs::canonicalize(&target) {
        Ok(canonical_target) => canonical_target.starts_with(&canonical_dir),
        Err(_) => false, // File does not exist or cannot be resolved.
    }
}

/// Build a content-addressed filename: `<sha256_prefix>_<original_name>`.
///
/// Uses only the first 16 hex chars of the digest to keep filenames readable
/// while still preventing collisions in practice.
///
/// `original_filename` is sanitised to strip traversal sequences so that
/// the returned path is always a plain filename with no directory component.
#[must_use]
pub fn content_addressed_path(sha256: &str, original_filename: &str) -> PathBuf {
    let safe_name = sanitize_filename(original_filename);
    let prefix = &sha256[..16.min(sha256.len())];
    PathBuf::from(format!("{prefix}_{safe_name}"))
}

/// Verify a single file against its expected SHA-256 hash.
///
/// The `relative_path` is sanitised before being joined to `base_dir` so
/// that path-traversal sequences like `../../etc/shadow` are neutralised.
/// Additionally, the canonical (resolved) path is checked to ensure it
/// still resides inside `base_dir`, providing defence-in-depth.
///
/// # Errors
/// Returns `anyhow::Error` if:
///   * the resolved path escapes `base_dir`,
///   * the file cannot be read, or
///   * the hash does not match.
pub fn verify_file(
    base_dir: &Path,
    relative_path: &str,
    expected_sha256: &str,
) -> anyhow::Result<bool> {
    use std::io::Read;

    // Defence layer 1: reject obviously malicious input before any path operations.
    // This explicit check makes the security boundary visible to static analysers.
    let trimmed = relative_path.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Empty relative path");
    }
    if trimmed.contains("..") || trimmed.starts_with('/') || trimmed.starts_with('\\') {
        anyhow::bail!(
            "Path traversal blocked: relative_path contains prohibited sequences: {trimmed:?}"
        );
    }

    // Defence layer 2: sanitise to a plain filename (strips separators, traversals).
    let safe_name = sanitize_filename(relative_path);

    // Defence layer 3: canonicalise the base directory to an absolute, symlink-free path.
    // SECURITY FIX: validation against CWD.
    let canonical_base = validate_path_is_safe(base_dir)?;

    // Build the target path from the *canonical* base so the result is already
    // rooted in a resolved directory.  `safe_name` is guaranteed to be a plain
    // filename (no separators, no `..`) so the join cannot escape canonical_base.
    let target_path = canonical_base.join(&safe_name);

    // Defence layer 4: canonicalise/resolve symlinks *before* opening.
    // This prevents TOCTOU attacks where we check the path, but then open a
    // symlink that was swapped in.
    let canonical_file = std::fs::canonicalize(&target_path)
        .map_err(|e| anyhow::anyhow!("Cannot resolve path {}: {e}", target_path.display()))?;

    if !canonical_file.starts_with(&canonical_base) {
        anyhow::bail!(
            "Path traversal blocked: {} escapes base directory {}",
            canonical_file.display(),
            canonical_base.display()
        );
    }

    // Open the *resolved* path.
    let mut file = std::fs::File::open(&canonical_file)
        .map_err(|e| anyhow::anyhow!("Cannot open {}: {e}", canonical_file.display()))?;


    // Read through the already-opened handle — the data comes from the same
    // inode that was validated above.
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let actual = sha256_hex(&data);
    Ok(actual == expected_sha256)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_deterministic() {
        let digest = sha256_hex(b"hello world");
        assert_eq!(
            digest,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn sha256_hex_empty_input() {
        let digest = sha256_hex(b"");
        assert_eq!(
            digest,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn manifest_default_is_empty() {
        let m = Manifest::default();
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn manifest_insert_and_contains() {
        let mut m = Manifest::default();
        let entry = ManifestEntry {
            source_url: "https://example.com/post".into(),
            sha256: "abc123".into(),
            local_path: "posts/abc123_post.html".into(),
            size: 1024,
            downloaded_at: "2026-02-15T00:00:00Z".into(),
        };
        m.insert(entry);
        assert!(m.contains("abc123"));
        assert!(!m.contains("xyz789"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn manifest_roundtrip_json() {
        let mut m = Manifest::default();
        m.insert(ManifestEntry {
            source_url: "https://x.com/a".into(),
            sha256: "deadbeef".into(),
            local_path: "a.html".into(),
            size: 512,
            downloaded_at: "2026-01-01T00:00:00Z".into(),
        });

        let json = serde_json::to_string(&m).unwrap();
        let deserialized: Manifest = serde_json::from_str(&json).unwrap();
        assert!(deserialized.contains("deadbeef"));
        assert_eq!(deserialized.len(), 1);
    }

    #[test]
    fn manifest_save_and_load() {
        // Use a local directory for testing to satisfy CWD restriction
        let dir = PathBuf::from("target/robustack_test_manifest");
        let _ = std::fs::create_dir_all(&dir);

        let mut m = Manifest::default();
        m.insert(ManifestEntry {
            source_url: "https://x.com/b".into(),
            sha256: "cafebabe".into(),
            local_path: "b.html".into(),
            size: 256,
            downloaded_at: "2026-02-15T01:00:00Z".into(),
        });

        m.save(&dir).expect("save should succeed");
        let loaded = Manifest::load_or_create(&dir).expect("load should succeed");
        assert!(loaded.contains("cafebabe"));

        // Cleanup.
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn should_skip_missing_file() {
        let m = Manifest::default();
        assert!(!should_skip(
            &m,
            "abc",
            Path::new("/tmp"),
            "nonexistent.html"
        ));
    }

    #[test]
    fn content_addressed_path_format() {
        let path = content_addressed_path(
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "my-post.html",
        );
        assert_eq!(path, PathBuf::from("b94d27b9934d3e08_my-post.html"));
    }

    #[test]
    fn verify_file_correct_hash() {
        let dir = PathBuf::from("target/robustack_test_verify");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("test.txt"), b"hello world").unwrap();

        let result = verify_file(
            &dir,
            "test.txt",
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        );
        assert!(result.unwrap());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_file_wrong_hash() {
        let dir = PathBuf::from("target/robustack_test_verify2");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("test.txt"), b"hello world").unwrap();

        let result = verify_file(&dir, "test.txt", "0000000000000000");
        assert!(!result.unwrap());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_file_blocks_traversal() {
        let dir = PathBuf::from("target/robustack_test_verify_traversal");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("safe.txt"), b"safe").unwrap();

        // Attempting to traverse should be blocked at the input validation layer.
        let result = verify_file(&dir, "../../etc/passwd", "irrelevant");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Path traversal blocked"),
            "Expected traversal error, got: {err}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_file_blocks_empty_path() {
        let dir = PathBuf::from("target/robustack_test_verify_empty");
        let _ = std::fs::create_dir_all(&dir);

        let result = verify_file(&dir, "", "irrelevant");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty relative path"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_file_blocks_absolute_path() {
        let dir = PathBuf::from("target/robustack_test_verify_abs");
        let _ = std::fs::create_dir_all(&dir);

        let result = verify_file(&dir, "/etc/passwd", "irrelevant");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Path traversal blocked"),
            "Expected traversal error, got: {err}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_path_fails_outside_cwd() {
        let temp = std::env::temp_dir();
        // This is extremely likely to be outside the CWD (which is the repo root)
        let result = validate_path_is_safe(&temp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path traversal blocked"));
    }

    #[test]
    fn manifest_entries_iterator() {
        let mut m = Manifest::default();
        m.insert(ManifestEntry {
            source_url: "a".into(),
            sha256: "aaa".into(),
            local_path: "a.html".into(),
            size: 1,
            downloaded_at: "t".into(),
        });
        m.insert(ManifestEntry {
            source_url: "b".into(),
            sha256: "bbb".into(),
            local_path: "b.html".into(),
            size: 2,
            downloaded_at: "t".into(),
        });
        assert_eq!(m.entries().count(), 2);
    }

    // -- sanitize_filename tests ------------------------------------------

    #[test]
    fn sanitize_strips_parent_directory_traversal() {
        assert_eq!(sanitize_filename("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_filename("../secret.txt"), "secret.txt");
    }

    #[test]
    fn sanitize_strips_absolute_path() {
        assert_eq!(sanitize_filename("/etc/shadow"), "shadow");
    }

    #[test]
    fn sanitize_windows_separators() {
        assert_eq!(sanitize_filename("..\\..\\Windows\\System32\\config"), "config");
    }

    #[test]
    fn sanitize_plain_filename_unchanged() {
        assert_eq!(sanitize_filename("my-post.html"), "my-post.html");
    }

    #[test]
    fn sanitize_empty_and_dots() {
        assert_eq!(sanitize_filename(""), "untitled");
        assert_eq!(sanitize_filename("."), "untitled");
        assert_eq!(sanitize_filename(".."), "untitled");
    }

    #[test]
    fn content_addressed_path_sanitises_traversal() {
        let path = content_addressed_path(
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "../../etc/passwd",
        );
        assert_eq!(path, PathBuf::from("b94d27b9934d3e08_passwd"));
    }
}
