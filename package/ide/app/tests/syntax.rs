//! Consumer syntax tests use the pinned runtime prepared by the package runtime task.

/// Canonical text and the application-owned Helix adapter.
use helix_core::Rope;
/// Parser roles must address source characters rather than UTF-8 byte offsets.
use ide_app::{source_style::StyleSpan, syntax::SyntaxEngine};
/// Paths drive Helix's built-in filename and shebang recognition.
use std::path::Path;

/// The source text of every span painted with one style,
///  in source order.
fn styled(spans: &[StyleSpan], text: &Rope, style: usize) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for span in spans.iter() {
        if span.style == style {
            texts.push(text.slice(span.start..span.end).to_string());
        }
    }
    return texts;
}

/// Whether some text contains the fragment.
fn some_contains(texts: &[String], fragment: &str) -> bool {
    for text in texts {
        if text.contains(fragment) {
            return true;
        }
    }
    return false;
}

/// Real Rust grammar/query assets identify keywords,
///  Unicode strings,
///  and comments.
#[test]
fn rust_highlights_use_unicode_source_positions() {
    let engine = SyntaxEngine::new().expect("language configuration");
    let text = Rope::from_str("fn main() { let 猫 = \"🐈\"; } // 猫\n");
    let spans = engine
        .highlight(Path::new("source.rs"), &text)
        .expect("pinned Rust grammar and queries")
        .expect("recognized Rust");
    assert!(styled(&spans, &text, 1).contains(&"fn".to_string()));
    assert!(some_contains(&styled(&spans, &text, 2), "🐈"));
    let mut classified: Vec<(usize, String)> = Vec::new();
    for span in spans.iter() {
        classified.push((span.style, text.slice(span.start..span.end).to_string()));
    }
    assert!(
        some_contains(&styled(&spans, &text, 3), "// 猫"),
        "classified source: {classified:?}"
    );
    let mut previous_end = 0;
    for span in spans.iter() {
        assert!(span.start >= previous_end);
        assert!(span.start < span.end && span.end <= text.len_chars());
        previous_end = span.end;
    }
}

/// TypeScript and its inherited ECMAScript query rules are loaded from the same runtime.
#[test]
fn typescript_highlights_inherited_queries() {
    let engine = SyntaxEngine::new().expect("language configuration");
    let text = Rope::from_str("const 猫: string = 'cat'; // note\n");
    let spans = engine
        .highlight(Path::new("source.ts"), &text)
        .expect("pinned TypeScript grammar and queries")
        .expect("recognized TypeScript");
    assert!(styled(&spans, &text, 1).contains(&"const".to_string()));
    assert!(some_contains(&styled(&spans, &text, 2), "cat"));
    assert!(styled(&spans, &text, 5).contains(&"string".to_string()));
}

/// Shebang recognition works without inventing an extension or loading project configuration.
#[test]
fn javascript_shebang_is_recognized() {
    let engine = SyntaxEngine::new().expect("language configuration");
    let text = Rope::from_str("#!/usr/bin/env node\nconst value = 42;\n");
    let spans = engine
        .highlight(Path::new("script"), &text)
        .expect("pinned JavaScript grammar and queries")
        .expect("recognized shebang");
    assert!(styled(&spans, &text, 4).contains(&"42".to_string()));
}

/// Unknown plain text and empty recognized source are successful distinct results.
#[test]
fn unknown_and_empty_source_are_not_parser_failures() {
    let engine = SyntaxEngine::new().expect("language configuration");
    assert!(
        engine
            .highlight(
                Path::new("notes.unrecognized-source-kind"),
                &Rope::from_str("plain text")
            )
            .expect("unknown language is plain text")
            .is_none()
    );
    assert!(
        engine
            .highlight(Path::new("empty.rs"), &Rope::new())
            .expect("empty Rust parses")
            .expect("recognized Rust")
            .is_empty()
    );
}
