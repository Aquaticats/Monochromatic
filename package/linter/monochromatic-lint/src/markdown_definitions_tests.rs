//! What:
//!  Native reference-definition classification and deletion controls.
//! Why:
//!  Parser-normalized identities and source-line boundaries jointly determine a safe fix.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Run the real rule and grouped editor against resolved links, images, duplicates and container lines.
//! ```

/// Import production checking and exact source/fix types.
use super::reference_definitions;
use crate::diagnostic::{Diagnostic, Severity};
use crate::edits::{Fix, apply_fixes};
use crate::markdown_source::MarkdownSource;

/// Parse an actual Markdown fixture before checking definitions.
fn check(source: &str) -> Vec<Diagnostic> {
    // Own filename/source strings so the parsed context retains them for the rule call.
    let context: MarkdownSource =
        MarkdownSource::new(String::from("references.md"), String::from(source), false)
            .expect("fixture");
    // Borrow the parsed context while collecting owned findings.
    return reference_definitions(&context, Severity::Warn);
}

/// Apply exactly the advertised fix groups through the production conflict/safety boundary.
fn fixed(source: &str) -> String {
    // Vec owns a variable-length list; each Fix is cloned from its diagnostic's ownership.
    let mut fixes: Vec<Fix> = Vec::<Fix>::new();
    for diagnostic in check(source) {
        // Present edits are moved from owned findings into this list without a second source rewrite.
        if let Some(fix) = diagnostic.fix {
            fixes.push(fix);
        }
    }
    // The fixture must satisfy production edit validation; failed validation fails the test.
    return apply_fixes(source, fixes.as_slice())
        .expect("validated edits")
        .source;
}

/// Used definitions retain their first target;
///  unused repeats remain unused rather than duplicate.
#[test]
fn reference_classification_preserves_messages_and_targets() {
    let source: &str =
        "See [docs][ref].\n\n[ref]: /first\n[unused]: /other\n[ref]: /second\n[unused]: /last\n";
    let findings: Vec<Diagnostic> = check(source);
    assert_eq!(findings.len(), 3);
    assert_eq!(
        findings[0].message,
        "Unused reference definition \"unused\"."
    );
    assert_eq!(
        findings[1].message,
        "Duplicate reference definition \"ref\"."
    );
    assert_eq!(
        findings[2].message,
        "Unused reference definition \"unused\"."
    );
    assert_eq!(findings[0].severity, Severity::Warn);
    assert_eq!(findings[0].labels[0].span.line, 4);
    let output: String = fixed(source);
    assert_eq!(output, "See [docs][ref].\n\n[ref]: /first\n");
    assert!(check(output.as_str()).is_empty());
}

/// Link/image kinds,
///  collapsed whitespace and Unicode folding use the parser's normalized identity.
#[test]
fn parser_identity_and_comment_exemption_are_preserved() {
    for source in [
        "[ref]\n\n[ref]: /target\n",
        "![alt][ref]\n\n[ref]: /target\n",
        "[a b][]\n\n[A\t B]: /target\n",
        "[ẞ][]\n\n[SS]: /target\n",
        "[//]: # (comment)\n\n[//]: # (second comment)\n",
        "[unresolved]\n",
        "plain\n",
    ] {
        assert!(check(source).is_empty(), "{source}");
    }
}

/// Standalone removals consume complete line endings,
///  including indentation and final EOF.
#[test]
fn standalone_definitions_do_not_leave_line_fragments() {
    for newline in ["\n", "\r\n", "\r"] {
        let source: String =
            format!("Title{newline}{newline}  [unused]: /target{newline}# Next{newline}");
        assert_eq!(
            fixed(source.as_str()),
            format!("Title{newline}{newline}# Next{newline}")
        );
    }
    assert_eq!(fixed("Title\n\n[unused]: /target"), "Title\n\n");
}

/// Container prefixes keep their newline so adjacent quoted prose is not nested or joined.
#[test]
fn container_definitions_keep_neighboring_lines_separate() {
    let source: &str = "> [unused]: /target\n> Text\n";
    assert_eq!(fixed(source), "> \n> Text\n");
}
