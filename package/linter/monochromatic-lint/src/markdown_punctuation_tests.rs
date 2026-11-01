//! What:
//!  Heading punctuation fixes over decoded text and original syntax boundaries.
//! Why:
//!  Escapes,
//!  entities,
//!  emphasis delimiters and Unicode must survive localized byte editing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Apply the real fixes to adversarial source spellings, then reparse the result.
//! ```

/// Import the production checker and private suffix-boundary controls.
use super::{no_trailing_punctuation, punctuation_edit, punctuation_entity, suffix_start};
use crate::diagnostic::{Diagnostic, Severity};
use crate::edits::{Fix, apply_fixes};
use crate::markdown_source::MarkdownSource;

/// Parse one native Markdown fixture and run only this rule.
fn check(source: &str) -> Vec<Diagnostic> {
    let context: MarkdownSource =
        MarkdownSource::new(String::from("heading.md"), String::from(source), false)
            .expect("fixture");
    return no_trailing_punctuation(&context, Severity::Error);
}

/// All equivalent punctuation spellings remove complete authored units.
#[test]
fn localized_fixes_preserve_delimiters_and_unicode() {
    for (source, expected) in [
        ("# Done.\n", "# Done\n"),
        ("# .\n", "# \n"),
        ("# .:\n", "# \n"),
        ("# **Done:**\n", "# **Done**\n"),
        ("# 🚀 Done.:\n", "# 🚀 Done\n"),
        ("# Done\\.\n", "# Done\n"),
        ("# Done\\:\n", "# Done\n"),
        ("# Done&period;\n", "# Done\n"),
        ("# Done&#58;\n", "# Done\n"),
        ("# Done&#46;&colon;\n", "# Done\n"),
        ("# Done&#x2e;&#X3A;\n", "# Done\n"),
        ("# Done\\\\.\n", "# Done\\\\\n"),
    ] {
        let findings: Vec<Diagnostic> = check(source);
        assert_eq!(findings.len(), 1, "{source}");
        assert_eq!(
            findings[0].message,
            "Heading ends with punctuation; remove the trailing punctuation."
        );
        let fix: Fix = findings[0].fix.clone().expect("mapped suffix");
        let output: String = apply_fixes(source, &[fix])
            .expect("apply localized fix")
            .source;
        assert_eq!(output, expected);
        assert!(check(output.as_str()).is_empty());
    }
}

/// Non-heading content,
///  other punctuation and headings with no text node stay unchanged.
#[test]
fn unrelated_text_is_not_rewritten() {
    for source in [
        "Done.\n",
        "# Done!\n",
        "# `code.`\n",
        "# ![alt.](image.png)\n",
    ] {
        assert!(check(source).is_empty(), "{source}");
    }
}

/// Unrecognized or incomplete encodings never yield a guessed suffix range.
#[test]
fn suffix_mapping_rejects_incomplete_or_unrelated_entities() {
    assert_eq!(punctuation_edit("plain", 1, 0, 5), None);
    assert_eq!(suffix_start("plain", 1), None);
    assert_eq!(suffix_start("&amp;", 1), None);
    assert_eq!(suffix_start(";", 1), None);
    assert_eq!(suffix_start("", 1), None);
    assert_eq!(suffix_start("unchanged", 0), Some(9));
    assert!(!punctuation_entity("#invalid"));
    assert!(!punctuation_entity("#xnothex"));
    assert!(!punctuation_entity("#Xnothex"));
    assert!(!punctuation_entity("unknown"));
}

/// Backslash runs that reach the text's first byte still decide whether the punctuation is escaped.
/// An odd run escapes it,
///  so its last backslash leaves with it;
///  an even run is literal backslashes that stay.
#[test]
fn escape_runs_reaching_the_text_start_decide_the_suffix() {
    // Each tuple is (authored text, punctuation count, expected suffix start); usize offsets are byte positions.
    for (written, count, expected) in [
        (".", 1, 0),
        ("\\.", 1, 0),
        ("\\\\.", 1, 2),
        ("\\\\\\:", 1, 2),
        ("\\.\\:", 2, 0),
        ("a\\\\\\.", 1, 3),
    ] {
        // Some(expected) is the mapped suffix start; None would mean no safe edit exists.
        assert_eq!(suffix_start(written, count), Some(expected), "{written}");
    }
}
