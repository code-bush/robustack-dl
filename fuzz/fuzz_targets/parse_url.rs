//! Fuzz target for URL input parsing.
//!
//! Tests that arbitrary byte sequences passed as URL strings
//! do not cause panics or undefined behavior.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // TODO: Replace with the actual URL parsing function once implemented.
        // e.g., robustack_dl::client::parse_url(s);
        let _ = s.trim();
    }
});
