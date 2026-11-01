//! What:
//!  Prompt-only fence behavior and original-byte preservation controls.
//! Why:
//!  Source-normalized code values cannot be used as raw source lengths for CRLF or Unicode inputs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Parse real Markdown, apply the advertised grouped fixes, and compare every authored byte.
//! ```

/// Import actual rule,
///  parser,
///  diagnostic and edit application.
use super::commands_show_output;
use crate::diagnostic::{Diagnostic, Severity};
use crate::edits::{Fix, apply_fixes};
use crate::markdown_source::MarkdownSource;

/// Check a Markdown fixture through the native adapter.
fn check(source: &str) -> Vec<Diagnostic> {
    let context: MarkdownSource =
        MarkdownSource::new(String::from("commands.md"), String::from(source), false)
            .expect("fixture parses");
    return commands_show_output(&context, Severity::Error);
}

/// Prompts are localized edits,
///  preserving fence spelling,
///  blank lines,
///  Unicode and newline sequences.
#[test]
fn prompt_only_examples_preserve_authored_source() {
    for newline in ["\n", "\r\n", "\r"] {
        let source: String =
            format!("```sh{newline}$ echo 🚀{newline}{newline}$ pwd{newline}```{newline}");
        let findings: Vec<Diagnostic> = check(source.as_str());
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].message,
            "Shell prompts with no shown output; remove the `$ ` prompts."
        );
        let fix: Fix = findings[0].fix.clone().expect("mapped prefix fixes");
        assert_eq!(fix.edits.len(), 2);
        let output: String = apply_fixes(source.as_str(), &[fix])
            .expect("atomic fixes")
            .source;
        assert_eq!(
            output,
            format!("```sh{newline}echo 🚀{newline}{newline}pwd{newline}```{newline}")
        );
        assert!(check(output.as_str()).is_empty());
    }
}

/// Prompt-looking text inside a command is content,
///  not another physical line's prefix.
#[test]
fn embedded_prompt_text_is_never_removed() {
    for newline in ["\n", "\r\n", "\r"] {
        let source: String =
            format!("```sh{newline}$ echo '$ value' and $ other{newline}```{newline}");
        let findings: Vec<Diagnostic> = check(source.as_str());
        assert_eq!(findings.len(), 1);
        let fix: Fix = findings[0].fix.clone().expect("line prefix fix");
        assert_eq!(fix.edits.len(), 1);
        assert_eq!(
            apply_fixes(source.as_str(), &[fix]).expect("apply").source,
            format!("```sh{newline}echo '$ value' and $ other{newline}```{newline}")
        );
    }
}

/// Edits stay absolute source offsets when prose and a multi-byte info string precede the commands.
#[test]
fn fences_after_other_content_keep_absolute_prompt_offsets() {
    for newline in ["\n", "\r\n", "\r"] {
        let source: String = format!(
            "🚀 intro{newline}{newline}```шелл{newline}$ pwd{newline}$ ls{newline}```{newline}"
        );
        let findings: Vec<Diagnostic> = check(source.as_str());
        assert_eq!(findings.len(), 1, "{source:?}");
        let fix: Fix = findings[0].fix.clone().expect("mapped prefix fixes");
        // Independent positions come from searching the authored text, not from the rule's own arithmetic.
        let first: usize = source.find("$ pwd").expect("first prompt");
        let second: usize = source.find("$ ls").expect("second prompt");
        assert_eq!(fix.edits.len(), 2);
        assert_eq!((fix.edits[0].start, fix.edits[0].end), (first, first + 2));
        assert_eq!((fix.edits[1].start, fix.edits[1].end), (second, second + 2));
        assert_eq!(
            apply_fixes(source.as_str(), &[fix]).expect("apply").source,
            format!(
                "🚀 intro{newline}{newline}```шелл{newline}pwd{newline}ls{newline}```{newline}"
            )
        );
    }
}

/// Empty,
///  output-bearing,
///  indented and non-fenced examples are not prompt-only column-one fences.
#[test]
fn exceptions_do_not_rewrite_unrelated_examples() {
    for source in [
        "```sh\n\n```\n",
        "```sh\n$ echo ok\nok\n```\n",
        "  ```sh\n  $ pwd\n  ```\n",
        "    $ pwd\n",
        "```sh\n  $ pwd\n```\n",
        "# Text\n",
    ] {
        assert!(check(source).is_empty(), "{source}");
    }
}

/// Output lines are a negative control for every normalized newline spelling.
#[test]
fn output_and_container_boundaries_remain_exceptions() {
    for newline in ["\n", "\r\n", "\r"] {
        let source: String = format!("```sh{newline}$ echo ok{newline}ok{newline}```{newline}");
        assert!(check(source.as_str()).is_empty(), "{source:?}");
    }
    for source in ["> ```sh\n> $ pwd\n> ```\n", "- ```sh\n  $ pwd\n  ```\n"] {
        assert!(check(source).is_empty(), "{source}");
    }
}

/// An unterminated fence still maps its final authored prompt without touching the opener.
#[test]
fn unclosed_fence_preserves_its_final_line() {
    let source: &str = "```sh\n$ pwd";
    let findings: Vec<Diagnostic> = check(source);
    assert_eq!(findings.len(), 1);
    let fix: Fix = findings[0].fix.clone().expect("mapped final prompt");
    assert_eq!(
        apply_fixes(source, &[fix]).expect("apply").source,
        "```sh\npwd"
    );
}
