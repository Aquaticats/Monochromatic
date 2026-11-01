//! Pin the plain matcher's deliberate differences from captured Chrome find-in-page results.

/// The production matching function,
///  not a parallel reimplementation of its pattern construction.
use ide_app::find::{MAX_FIND_MATCHES, find_matches};
/// Typed fixture decoding keeps source strings and expected offsets separate from executable patterns.
use serde::Deserialize;

/// What:
///  Serde derives a JSON decoder for this owned record;
///  String owns text rather than borrowing &str.
/// Why:
///  The parsed corpus outlives the temporary parser call and records the exact tested browser build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Corpus = { browserVersion: string; cases: Case[] };
/// ```
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Corpus {
    /// Full build read from the disposable browser profile,
    ///  not the reduced user agent.
    browser_version: String,
    /// Measured cases are finite,
    ///  synthetic,
    ///  and independent of project files.
    cases: Vec<Case>,
}

/// Browser offsets count UTF-16 units while the matcher returns source character positions.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    /// Stable case name identifies a semantic difference without relying on display ordering.
    name: String,
    /// Synthetic source remains unchanged by the matcher.
    text: String,
    /// Literal query,
    ///  passed to the matcher exactly as the find input would hold it.
    query: String,
    /// False denotes no match,
    ///  not an empty selected range.
    found: bool,
    /// Original source substring selected by the browser.
    selected: String,
    /// Inclusive browser boundary in UTF-16 units.
    start_utf16: usize,
    /// Exclusive browser boundary in UTF-16 units.
    end_utf16: usize,
}

/// What:
///  A fixed-size list of borrowed names;
///  `&str` borrows text baked into the test binary.
/// Why:
///  The user decided on 2026-10-05 that Chrome's collation folding is not wanted,
///  so exactly these
/// captured cases differ;
///  any other set means matching behavior changed by accident.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DELIBERATE_DIFFERENCES = ['canonical-accent', /* ... */] as const;
/// ```
const DELIBERATE_DIFFERENCES: [&str; 13] = [
    "canonical-accent",
    "plain-accent",
    "case-expansion",
    "compatibility-ligature",
    "dotted-i",
    "nbsp-as-space",
    "kana-script",
    "kana-width",
    "kana-composed",
    "single-quote",
    "double-quote",
    "soft-hyphen",
    "combining-mark-only",
];

/// Convert a source character position to the UTF-16 unit offset the browser reported.
fn utf16_offset(text: &str, position: usize) -> usize {
    let mut units = 0;
    // What: `chars()` walks Unicode scalar values and `take` stops after `position` of them.
    // Why: Astral characters occupy two UTF-16 units but one source character position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const character of [...text].slice(0, position)) units += character.length;
    // ```
    for character in text.chars().take(position) {
        units += character.len_utf16();
    }
    return units;
}

/// The first production match must agree with the browser except for the pinned deliberate differences.
#[test]
fn plain_matcher_differs_from_browser_reference_only_in_pinned_cases() {
    // What: `include_str!` embeds the fixture at compile time; `expect` fails the test on invalid JSON.
    // Why: The comparison never reads project files at run time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const corpus: Corpus = JSON.parse(fixtureText);
    // ```
    let corpus: Corpus = serde_json::from_str(include_str!("fixture/browser-find.json"))
        .expect("captured browser corpus");
    // What: `Vec::new()` creates an empty growable list of owned names.
    // Why: Differences are collected in corpus order and compared as one exact set.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const differences: string[] = [];
    // ```
    let mut differences = Vec::new();
    let mut seen_positive = false;
    let mut seen_negative = false;
    let total = corpus.cases.len();
    for case in corpus.cases {
        let matches =
            find_matches(&case.text, &case.query, MAX_FIND_MATCHES).expect("corpus query is valid");
        // What: `first()` returns `Some(&range)` for a non-empty list or `None`.
        // Why: The browser fixture records only the first selected match.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const first = matches.ranges[0];
        // ```
        let first = matches.ranges.first();
        let mut selected = String::new();
        let mut start = 0;
        let mut end = 0;
        if let Some(range) = first {
            selected = case
                .text
                .chars()
                .skip(range.start)
                .take(range.end - range.start)
                .collect();
            start = utf16_offset(&case.text, range.start);
            end = utf16_offset(&case.text, range.end);
        }
        let agrees = first.is_some() == case.found
            && selected == case.selected
            && start == case.start_utf16
            && end == case.end_utf16;
        if !agrees {
            differences.push(case.name.clone());
        }
        if case.name == "positive-literal" {
            assert!(first.is_some() && agrees);
            seen_positive = true;
        }
        if case.name == "negative-literal" {
            assert!(first.is_none() && agrees);
            seen_negative = true;
        }
        // Exact machine-readable terminal output is an inspection artifact, not application logging.
        println!(
            "{}",
            ide_app::json_value!({ "case": case.name, "agrees": agrees, "browserFound": case.found, "matcherFound": first.is_some(), "matcherSelected": selected, "matcherStartUtf16": start, "matcherEndUtf16": end })
        );
    }
    assert!(
        seen_positive && seen_negative,
        "the corpus must exercise both match outcomes"
    );
    println!(
        "{}",
        ide_app::json_value!({ "browserVersion": corpus.browser_version, "differentCases": differences })
    );
    assert_eq!(total, 29, "the captured corpus changed size");
    assert_eq!(
        differences, DELIBERATE_DIFFERENCES,
        "the plain matcher's differences from the browser reference changed"
    );
}
