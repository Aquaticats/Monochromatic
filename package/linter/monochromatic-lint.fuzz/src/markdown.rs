//! What:
//!  Structured and raw-source checks for the implemented Markdown/MDX rules.
//! Why:
//!  Always-valid cases prove rule reach;
//!  raw bytes exercise syntax recovery and fix boundary safety.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Check an independently counted fixture on every draw, then lint arbitrary UTF-8 in both modes.
//! ```

use monochromatic_lint::diagnostic::{Diagnostic, Severity, Span};
use monochromatic_lint::edits::{Fix, apply_fixes};
/// Import production rule entry points rather than duplicate their recognition logic.
use monochromatic_lint::markdown_code::fenced_code_language;
use monochromatic_lint::markdown_commands::commands_show_output;
use monochromatic_lint::markdown_definitions::reference_definitions;
use monochromatic_lint::markdown_duplicate_headings::no_duplicate_heading;
use monochromatic_lint::markdown_headings::{heading_increment, no_emphasis_as_heading, single_h1};
use monochromatic_lint::markdown_links::{link_image_style, no_bare_urls};
use monochromatic_lint::markdown_punctuation::no_trailing_punctuation;
use monochromatic_lint::markdown_semantic_breaks::semantic_line_breaks;
/// Import the real parser,
///  source spans and atomic fix applier.
use monochromatic_lint::markdown_source::MarkdownSource;
/// Import the newly integrated table and prose checks through their actual implementation boundary.
use monochromatic_lint::markdown_tables::no_pipe_tables;

/// What:
///  A plain named function pointer;
///  no captured closure or per-input executable configuration.
/// Why:
///  The pure rules share one parse while the fence rule receives its explicit rustdoc flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Checker = (context: MarkdownSource, severity: Severity) => Diagnostic[];
/// ```
type Checker = fn(&MarkdownSource, Severity) -> Vec<Diagnostic>;

/// What:
///  Fixed grammar cases with independently known combined finding counts.
/// Why:
///  A raw-only generator could miss every successfully parsed violation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cases: readonly [string, number][] = [...];
/// ```
const CASES: &[(&str, usize)] = &[
    ("# Title\n", 0),
    ("# Title\n\n### Skip\n", 1),
    ("# Title\n\n# Other\n", 1),
    ("# Title\n\n# Title\n", 2),
    ("# Done:\n", 1),
    ("**Heading**\n", 1),
    ("https://example.com/a\n", 1),
    ("```sh\n$ pwd\n```\n", 1),
    ("```\nbody\n```\n", 1),
    ("[ref]\n\n[ref]: /ok\n", 1),
    ("text\n\n[unused]: /x\n", 1),
    ("`code` and plain\n", 0),
    ("left, right.\n", 1),
    ("| A | B |\n| --- | --- |\n| 1 | 2 |\n", 1),
];

/// Run the implemented rule catalog over one shared parse.
fn findings(context: &MarkdownSource) -> Vec<Diagnostic> {
    // The fixed array borrows no captured state; Vec collects the unknown number of actual findings.
    let checkers: [Checker; 11] = [
        heading_increment,
        single_h1,
        no_emphasis_as_heading,
        no_bare_urls,
        link_image_style,
        commands_show_output,
        no_duplicate_heading,
        no_trailing_punctuation,
        reference_definitions,
        no_pipe_tables,
        semantic_line_breaks,
    ];
    let mut result: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for checker in checkers {
        result.extend(checker(context, Severity::Error));
    }
    // Ordinary Markdown fixtures are not virtual rustdoc inputs.
    result.extend(fenced_code_language(context, Severity::Error, false));
    return result;
}

/// Verify diagnostic ranges and every advertised edit,
///  allowing only the deliberate empty-file refusal.
fn check_source(source: &str, mdx: bool) -> Option<usize> {
    // Own each snapshot; the parser cannot retain references into the fuzzer's changing input buffer.
    let parsed = MarkdownSource::new(String::from("fuzz.md"), String::from(source), mdx);
    let context: MarkdownSource = match parsed {
        Ok(value) => value,
        Err(error) => {
            // Explicit processing rejection is allowed, but its source location must remain bounded.
            assert!(error.offset <= source.len(), "{error}");
            return None;
        }
    };
    // Borrow the context only during rule execution; findings own their payloads.
    let diagnostics: Vec<Diagnostic> = findings(&context);
    let mut fixes: Vec<Fix> = Vec::<Fix>::new();
    for diagnostic in &diagnostics {
        assert_eq!(diagnostic.filename, "fuzz.md");
        assert_eq!(diagnostic.severity, Severity::Error);
        for label in &diagnostic.labels {
            // Span addresses use UTF-8 bytes, unlike the user-visible UTF-16 column.
            let span: &Span = &label.span;
            assert!(span.offset <= source.len());
            assert!(span.length <= source.len() - span.offset);
            assert!(source.is_char_boundary(span.offset));
            assert!(source.is_char_boundary(span.offset + span.length));
            assert!(span.line >= 1 && span.column >= 1);
        }
        // Clone only the present fix so grouped edits can be checked together without consuming diagnostics.
        if let Some(fix) = &diagnostic.fix {
            fixes.push(fix.clone());
        }
    }
    // Parse accepted fixed output again; an advertised edit must not create a parser processing failure.
    match apply_fixes(source, fixes.as_slice()) {
        Ok(applied) => {
            assert!(MarkdownSource::new(String::from("fixed.md"), applied.source, mdx).is_ok());
        }
        Err(error) => {
            assert_eq!(
                error.message,
                "Autofix would replace non-empty file with empty output; leaving file unchanged."
            );
        }
    }
    // Some distinguishes a successfully checked clean input from a typed parser rejection.
    return Some(diagnostics.len());
}

/// Exercise counted source with all newline spellings before arbitrary source in each parser mode.
pub fn check_markdown(data: &[u8]) {
    // Defaults ensure even empty input reaches a known parsed case.
    let selector: u8 = data.first().copied().unwrap_or(0);
    let newline_selector: u8 = data.get(1).copied().unwrap_or(0);
    let (fragment, expected): (&str, usize) = CASES[usize::from(selector) % CASES.len()];
    let newline: &str = ["\n", "\r\n", "\r"][usize::from(newline_selector) % 3];
    // Prefix Unicode shifts byte offsets without adding a rule violation.
    let source: String = format!("🚀\n\n{fragment}").replace('\n', newline);
    for mdx in [false, true] {
        assert_eq!(check_source(source.as_str(), mdx), Some(expected));
        // Invalid UTF-8 cannot be a source String, but its bytes already exercised a structured case.
        if let Ok(raw) = std::str::from_utf8(data) {
            check_source(raw, mdx);
        }
    }
}

/// Independently count the catalog and visit every newline/mode branch before libFuzzer starts.
#[test]
fn generated_markdown_cases_reach_each_rule_and_newline_mode() {
    let expected: [usize; 14] = [0, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1];
    assert_eq!(CASES.len(), expected.len());
    for (index, count) in expected.iter().enumerate() {
        assert_eq!(CASES[index].1, *count);
        for newline in 0..3_u8 {
            check_markdown(&[u8::try_from(index).expect("bounded case index"), newline]);
        }
    }
    check_markdown(&[]);
    check_markdown(&[255, 254, 253]);
}
