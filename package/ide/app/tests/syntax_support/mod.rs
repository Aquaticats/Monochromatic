//! Shared assertion for syntax tests:
//!  one sample is recognized and painted by the application engine.

/// What:
///  `use helix_core::Rope;` brings the `Rope` type into this file under its short name.
///       A rope is the text container the application keeps source in.
/// Why:
///   The engine classifies exactly this type,
///  so tests must build the same input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// The application-owned adapter under test;
///  no Helix loader is constructed here.
use ide_app::syntax::SyntaxEngine;
/// Paths drive filename recognition exactly as a project file would.
use std::path::Path;

/// What:
///  `const ROLES: &[&str]` is a fixed,
///  read-only list of borrowed text values.
///       Siblings:
///  `Vec<String>` (growable,
///  owned) and `[&str; 22]` (length in the type).
/// Why:
///   The engine numbers paint roles as one plus the position in this same ordered list,
///       so tests can name a role (`"keyword"`) instead of repeating its number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROLES = ['keyword', 'string', /* ... */ 'special'] as const;
/// ```
const ROLES: &[&str] = &[
    "keyword",
    "string",
    "comment",
    "constant",
    "type",
    "function",
    "variable",
    "operator",
    "punctuation",
    "tag",
    "attribute",
    "namespace",
    "special",
    "markup.heading",
    "markup.bold",
    "markup.italic",
    "markup.strikethrough",
    "markup.raw.inline",
    "markup.link",
    "markup.link.url",
    "markup.list",
    "markup.quote",
];

/// What:
///  `pub fn assert_reads_as(path: &str, ...)` is a public function taking five borrowed
///       text values (`&str`:
///  the caller keeps ownership;
///  sibling `String` would take it).
///       It returns nothing and stops the test by panicking when an expectation fails.
/// Why:
///   Every bundled language needs the same two facts checked through the application path:
///       the file is recognized as `language`,
///  and `fragment` is painted with `role`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertReadsAs(
///   path: string, source: string, language: string, role: string, fragment: string,
/// ): void { /* throws when an expectation fails */ }
/// ```
pub fn assert_reads_as(path: &str, source: &str, language: &str, role: &str, fragment: &str) {
    // What: `ROLES.iter().position(|name| ...)` scans the list with a closure (`|name| ...` is
    //       an arrow function) and returns `Option<usize>`: `Some(index)` or `None`.
    //       `*name` reads through the reference the iterator lends. `.expect("...")` extracts
    //       the index or panics with that message. `usize` is the platform's index integer;
    //       siblings `u32`/`u64`/`i32` would need a cast wherever a position is used.
    // Why:  A misspelled role name must fail loudly instead of matching nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const index = ROLES.indexOf(role);
    // if (index < 0) throw new Error('role is one of the engine palette roles');
    // ```
    let mut found_index = None;
    for (position, name) in ROLES.iter().enumerate() {
        if *name == role {
            found_index = Some(position);
            break;
        }
    }
    let index: usize = found_index.expect("role is one of the engine palette roles");
    // What: `SyntaxEngine::new()` returns `Result<SyntaxEngine>`: success or an error value.
    // Why:  A missing bundled manifest must stop the test with its own diagnostic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engine = new SyntaxEngine(); // throws when the manifest is missing
    // ```
    let engine = SyntaxEngine::new().expect("bundled language manifest");
    let text = Rope::from_str(source);
    let file = Path::new(path);
    // What: `&text` lends the rope without giving it away. `language_id` returns
    //       `Option<String>`; `.as_deref()` views it as `Option<&str>` so it can be compared
    //       with `Some(language)`, the present variant wrapping the expected name.
    //       `assert_eq!` is a macro (the `!`) that panics when both sides differ.
    // Why:  Recognition is asserted separately from painting, so a wrong language cannot hide
    //       behind a fragment that two grammars happen to paint alike.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert.equal(engine.languageId(file, text), language);
    // ```
    assert_eq!(
        engine.language_id(file, &text).as_deref(),
        Some(language),
        "language recognized for {path}"
    );
    // What: `painted` runs the engine again and lists every interval as its role name and visible text.
    // Why:  The failure message below must show what the grammar produced.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const classified = painted(path, source);
    // ```
    let classified = painted(path, source);
    let mut found = false;
    for (name, shown) in classified.iter() {
        if ROLES[index] == *name && shown == fragment {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "expected {role} on {fragment:?} in {path}; painted fragments: {classified:?}"
    );
}

/// What:
///  `pub fn painted(path: &str, source: &str) -> Vec<(&'static str, String)>` returns every painted interval of
///       one sample as its role name and its text with trailing line ends removed.
///  `&'static str` is a borrowed name that lives for the whole program,
/// here an entry of `ROLES`.
/// Why:
///   Markdown paints whole nodes,
/// and a node can end with its line end,
///  so tests compare the visible text of an interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function painted(path: string, source: string): Array<[string, string]>
/// ```
pub fn painted(path: &str, source: &str) -> Vec<(&'static str, String)> {
    let engine = SyntaxEngine::new().expect("bundled language manifest");
    let text = Rope::from_str(source);
    let spans = engine
        .highlight(Path::new(path), &text)
        .expect("bundled grammar and highlighting rules load")
        .expect("bundled language is not plain text");
    let mut fragments: Vec<(&'static str, String)> = Vec::new();
    for span in spans.iter() {
        fragments.push((
            ROLES[span.style - 1],
            text.slice(span.start..span.end)
                .to_string()
                .trim_end_matches(['\n', '\r'])
                .to_string(),
        ));
    }
    return fragments;
}
