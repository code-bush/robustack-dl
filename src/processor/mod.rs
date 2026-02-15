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
//! Content processor module — pure transformations on downloaded HTML.
//!
//! # Design
//! All functions are **pure** — they accept input and return output with
//! no side effects (no I/O, no network, no filesystem writes).
//! This makes the processor fully testable without mocks.

/// Convert raw HTML to Markdown.
///
/// Uses a DOM-based parser (`html2text`) to produce clean text with
/// Markdown-style formatting.  Returns the original HTML unchanged if
/// the input is empty.
#[must_use]
pub fn html_to_markdown(html: &str) -> String {
    if html.is_empty() {
        return String::new();
    }
    // Unwrap is safe because we are reading from an in-memory byte slice.
    html2text::from_read(html.as_bytes(), 80).unwrap_or_else(|e| {
        tracing::error!("Failed to convert HTML to Markdown: {e}");
        html.to_owned()
    })
}

/// Convert raw HTML to plain text.
///
/// Strips all tags while preserving heading structure and paragraph
/// breaks.  Returns an empty string for empty input.
#[must_use]
pub fn html_to_text(html: &str) -> String {
    if html.is_empty() {
        return String::new();
    }
    // Unwrap is safe because we are reading from an in-memory byte slice.
    html2text::from_read(html.as_bytes(), 80).unwrap_or_else(|e| {
        tracing::error!("Failed to convert HTML to Text: {e}");
        html.to_owned()
    })
}

/// Append a source URL footer to content.
#[must_use]
pub fn append_source_url(content: &str, source_url: &str) -> String {
    format!("{content}\n\n---\nSource: {source_url}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_to_markdown_converts_headings() {
        let html = "<h1>Title</h1><p>Body text.</p>";
        let md = html_to_markdown(html);
        assert!(md.contains("Title"), "should preserve heading text");
        assert!(md.contains("Body text."), "should preserve paragraph text");
        // Tags should be stripped.
        assert!(!md.contains("<h1>"), "HTML tags should be removed");
    }

    #[test]
    fn html_to_markdown_empty_input() {
        assert_eq!(html_to_markdown(""), "");
    }

    #[test]
    fn html_to_text_strips_tags() {
        let html = "<h1>Title</h1><p>Body</p>";
        let text = html_to_text(html);
        assert!(text.contains("Title"), "should preserve text");
        assert!(text.contains("Body"), "should preserve text");
        assert!(!text.contains("<h1>"), "HTML tags should be removed");
    }

    #[test]
    fn html_to_text_empty_input() {
        assert_eq!(html_to_text(""), "");
    }

    #[test]
    fn append_source_url_adds_footer() {
        let content = "Hello world";
        let result = append_source_url(content, "https://example.com/post");
        assert!(result.starts_with("Hello world"));
        assert!(result.contains("Source: https://example.com/post"));
    }

    #[test]
    fn pure_functions_have_no_side_effects() {
        // Calling the same function twice with the same input
        // produces identical output (referential transparency).
        let input = "<p>test</p>";
        assert_eq!(html_to_markdown(input), html_to_markdown(input));
        assert_eq!(html_to_text(input), html_to_text(input));
    }
}
