//! Fuzz target for manifest.json parsing.
//!
//! Tests that arbitrary byte sequences passed as manifest JSON
//! do not cause panics or undefined behavior.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // TODO: Replace with the actual manifest parsing function once implemented.
        // e.g., robustack_dl::integrity::parse_manifest(s);
        let _ = serde_json::from_str::<serde_json::Value>(s);
    }
});
