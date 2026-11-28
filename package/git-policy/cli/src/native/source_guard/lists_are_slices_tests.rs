//! What:
//!  The ban on a list of text written as an array with a hand-counted length:
//!  no source of this crate declares a constant of a text type with a fixed
//!  array length and a list beside it.
//! Why:
//!  Every entry added or removed edits the count beside it,
//!  and the count says nothing a reader needs
//!  (the owner,
//!  2026-11-25:
//!  "Isn't 95 a magic number that can be avoided?";
//!  debt ledger item 19,
//!  and the rule `doc/how-to-write-this-packages-code-and-tests.md` states).
//!  An array keeps a fixed length only where the length is the meaning,
//!  such as a header of a fixed byte width or a record's field count,
//!  and those arrays hold no text,
//!  so the ban names text rather than picking over every array.
//! Gotcha:
//!  The scan reads code only,
//!  so the refused shape may be named in a comment.
//!  It reads one declaration at a time,
//!  and a declaration written across lines is
//!  still one piece:
//!  the mask keeps every byte in its place,
//!  so the scan steps over whitespace rather than expecting one space.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function handCountedTextLists(text: string): number[] // one line per finding
//! ```

use super::source_code_places_tests::is_code;
use crate::source_scan_tests::{
    CrateRoot,
    classified_sources,
    real_roots,
    release_root,
    source_root,
    write_tree,
};

/// The word a constant's declaration begins with,
/// which is where every scan below starts looking.
const A_CONSTANT: &str = "const ";

/// The text of a reference to a text,
/// as an element type names it.
const A_TEXT_REFERENCE: &str = "&str";

/// The text of a reference to a text with `'static` written out,
/// which `A_TEXT_REFERENCE` is not a prefix of.
const A_STATIC_TEXT_REFERENCE: &str = "&'staticstr";

/// What:
///  `fn is_name_character(character: char) -> bool` answers whether `character`
///  may stand in a constant's name.
///  `char` is one Unicode scalar value (siblings:
///  `u8`,
///  one byte,
///  which cannot hold a character of several bytes).
/// Why:
///  The scan reads a whole name and then expects the colon after it,
///  so `const X: [T; N]` is read and `constant_thing` is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNameCharacter = (c: string): boolean => /[A-Z0-9_]/.test(c)
/// ```
fn is_name_character(character: char) -> bool {
    return character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_';
}

/// What:
///  `fn code_only(text: &str) -> String` is the file's text with every byte that
///  sits in a comment or a text literal replaced by a space.
///  `String` is a growable,
///  owned text (siblings:
///  `&str`,
///  a borrowed view;
///  `Vec<u8>`,
///  the same bytes without the promise that they are UTF-8).
/// Why:
///  A file that names the refused shape in a comment or in a planted text is not
///  declaring one,
///  and this crate's guard and its guide both name it in prose.
///  `String` and not `&str` because the mask is a buffer this reader makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function codeOnly(text: string): string {
///   const places = isCode(text);
///   return [...text].map((c, i) => (places[i] ? c : ' ')).join('');
/// }
/// ```
fn code_only(text: &str) -> String {
    let places: Vec<bool> = is_code(text);
    let mut masked: String = String::with_capacity(text.len());
    for (index, character) in text.char_indices() {
        // A character counts as code only when every byte of it is code, so a
        // character of several bytes inside a comment leaves spaces of its width.
        let width: usize = character.len_utf8();
        let code: bool = places[index..index + width]
            .iter()
            .all(|place| return *place);
        if code {
            masked.push(character);
        } else if character == '\n' {
            // A newline keeps its own place, so the number of lines before a
            // declaration is the same in the mask as in the file.
            masked.push('\n');
        } else {
            for _ in 0..width {
                masked.push(' ');
            }
        }
    }
    return masked;
}

/// What:
///  `fn hand_counted_text_lists(text: &str) -> Vec<usize>` lists the one-based line
///  numbers of the declarations in `text` that write a text list as an array with
///  a number after its type.
///  `Vec<usize>` is a growable list of indexes (siblings:
///  `&[usize]`,
///  a borrowed view;
///  `[usize; N]`,
///  a fixed array).
/// Why:
///  A finding names a line a person opens,
///  and one control reports every finding of a tree at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handCountedTextLists(text: string): number[] {
///   // for each `const NAME: [T; N] = [` whose T holds a text reference, push the line
/// }
/// ```
fn hand_counted_text_lists(text: &str) -> Vec<usize> {
    let code: String = code_only(text);
    let mut found: Vec<usize> = Vec::new();
    let mut from: usize = 0;
    // `find` gives `Some(offset)` of the next word and `None` when the file holds
    // no more, which ends the walk.
    while let Some(at) = code[from..].find(A_CONSTANT) {
        let begins: usize = from + at + A_CONSTANT.len();
        from = begins;
        let rest: &str = &code[begins..];
        let name_length: usize = rest
            .char_indices()
            .take_while(|(_, character)| return is_name_character(*character))
            .map(|(offset, character)| return offset + character.len_utf8())
            .last()
            .unwrap_or(0);
        if name_length == 0 {
            continue;
        }
        let after_name: &str = &rest[name_length..];
        let Some(after_colon) = after_name.strip_prefix(": [") else {
            continue;
        };
        let Some(inner_length) = brackets_end(after_colon) else {
            continue;
        };
        let inner: &str = &after_colon[..inner_length];
        if !begins_a_list_initializer(&after_colon[inner_length + 1..]) {
            continue;
        }
        if names_a_text_type(inner) {
            found.push(code[..begins].matches('\n').count() + 1);
        }
    }
    return found;
}

/// What:
///  `fn begins_a_list_initializer(after_the_type: &str) -> bool` answers whether the text
///  that follows an array's closing bracket writes `= [` next,
///  with any amount of whitespace on either side of the `=`,
///  a line break included.
/// Why:
///  A declaration written across lines,
///  as `const NAMES: [&str; 2] =` and the list on the
///  line below,
///  is still one declaration whose length is a hand count,
///  and the count of the one written on a single line is no different.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const beginsAListInitializer = (afterTheType: string): boolean =>
///   /^\s*=\s*\[/.test(afterTheType)
/// ```
fn begins_a_list_initializer(after_the_type: &str) -> bool {
    let without_leading_space: &str = after_the_type.trim_start();
    let Some(after_the_equals) = without_leading_space.strip_prefix('=') else {
        return false;
    };
    return after_the_equals.trim_start().starts_with('[');
}

/// What:
///  `fn brackets_end(text: &str) -> Option<usize>` is the offset of the `]` that
///  closes the bracket the text opens inside,
///  or `None` when no bracket closes.
///  `Option<usize>` is "an offset or nothing" (sibling:
///  `usize`,
///  which would need
///  a second value to say "none").
/// Why:
///  An element type may hold brackets of its own,
///  as `[(u8, i64); 3]` does,
///  so the closing one is the bracket at the opening one's depth and never the
///  first `]`.
fn brackets_end(text: &str) -> Option<usize> {
    let mut depth: usize = 1;
    for (offset, character) in text.char_indices() {
        if character == '[' {
            depth += 1;
        }
        if character == ']' {
            depth -= 1;
            if depth == 0 {
                return Some(offset);
            }
        }
    }
    return None;
}

/// What:
///  `fn names_a_text_type(type_inside_brackets: &str) -> bool` answers whether an
///  element type written inside a type's brackets names a reference to a text,
///  as `&str`,
///  `&'static str` and `Option<&str>` all do.
/// Why:
///  A text list is the shape whose length is a hand count of content;
///  a number,
///  a mode or a signal is often a table whose length is its meaning,
///  and no table can be mistaken for text here.
fn names_a_text_type(type_inside_brackets: &str) -> bool {
    let squeezed: String = type_inside_brackets
        .chars()
        .filter(|character| return !character.is_whitespace())
        .collect();
    let mut from: usize = 0;
    while let Some(at) = squeezed[from..].find('&') {
        let begins: usize = from + at;
        let rest: &str = &squeezed[begins..];
        let reference: &str = if rest.starts_with(A_STATIC_TEXT_REFERENCE) {
            A_STATIC_TEXT_REFERENCE
        } else if rest.starts_with(A_TEXT_REFERENCE) {
            A_TEXT_REFERENCE
        } else {
            from = begins + 1;
            continue;
        };
        // The reference ends its word when no name character follows it, so
        // `&str` is text and `&strategy` is a reference to something else.
        let next: Option<char> = rest[reference.len()..].chars().next();
        let ends_the_word: bool = match next {
            None => true,
            Some(character) => !is_name_character(character) && !character.is_ascii_lowercase(),
        };
        if ends_the_word {
            return true;
        }
        from = begins + 1;
    }
    return false;
}

/// What:
///  `fn findings_in(root, roots) -> Result<Vec<String>, String>` is every
///  `path:line` of the sources under `root` that writes a text list with a
///  hand-counted length.
///  `Result` is "an answer or the reason there is none" (sibling:
///  a panic,
///  which would not say which file could not be read).
/// Why:
///  A file the scan cannot read stops the check rather than passing as a file
///  with no findings,
///  so a walk that read nothing shows itself.
fn findings_in(
    root: &std::path::Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in classified_sources(root, roots)? {
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for number in hand_counted_text_lists(text.as_str()) {
            found.push(format!("{}:{number}", file.path));
        }
    }
    return Ok(found);
}

/// The findings in this crate's own sources.
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's sources");
}

/// No source of this crate writes a list of text as an array with a
/// hand-counted length.
/// The shape to write instead is `const NAMES: &[&str] = &[...]`.
#[test]
fn no_constant_list_of_text_carries_a_hand_counted_length() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "a list of text is a slice, so no entry added or removed edits a count beside it (`const NAMES: &[&str] = \
         &[...]`); a length that is the meaning belongs to an array that holds no text:\n{}",
        found.join("\n")
    );
}

/// The scan reads both kinds of source this crate has,
/// so a check over a tree it could not classify fails rather than passing every
/// file it never opened.
#[test]
fn the_scan_reads_release_and_test_only_sources() {
    let sources = classified_sources(&source_root(), &real_roots()).expect("the crate's sources");
    let release: usize = sources.iter().filter(|file| return !file.test_only).count();
    let test_only: usize = sources.iter().filter(|file| return file.test_only).count();
    assert!(
        release > 0 && test_only > 0,
        "the scan must reach release code and test sidecars alike, and it read {release} release and {test_only} \
         test-only files"
    );
}

/// The scan fires on a planted text list in release code and in a test sidecar,
/// on the one whose list starts on the line below as much as on the one written in a
/// single line,
/// and never on a comment that names the shape or on a byte array whose length
/// is the meaning.
#[test]
fn the_scan_finds_a_planted_text_list() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-hand-counted-list");
    let release_text: &str = "// const NAMES: [&str; 2] = [\"a\", \"b\"]; names the refused shape.\n\
                              const NAMES: [&str; 2] = [\"a\", \"b\"];\n\
                              const WORDS: [&str; 2] =\n\
                                  [\"a\", \"b\"];\n\
                              const MAGIC: [u8; 4] = *b\"DIRC\";\n";
    let sidecar_text: &str = "const REPEATED: [&'static str; 1] = [\"a\"];\n\
                              const STAGES: [(u8, i32); 2] = [(1, 2), (3, 4)];\n";
    write_tree(
        root.as_path(),
        &[
            ("lib.rs", "#[cfg(test)]\nmod planted_tests;\nmod release;\n"),
            ("planted_tests.rs", sidecar_text),
            ("release.rs", release_text),
        ],
    );
    let found: Vec<String> =
        findings_in(root.as_path(), &[release_root("lib.rs")]).expect("the fixture sources");
    // Each planted declaration's line, read from the fixture's own text, so the
    // fixture may grow without editing an expected number here.
    let release_line: usize = release_text
        .lines()
        .position(|line| return line.starts_with("const NAMES: ["))
        .expect("the planted release list")
        + 1;
    let words_line: usize = release_text
        .lines()
        .position(|line| return line.starts_with("const WORDS: ["))
        .expect("the planted release list written across lines")
        + 1;
    let sidecar_line: usize = sidecar_text
        .lines()
        .position(|line| return line.starts_with("const REPEATED: ["))
        .expect("the planted sidecar list")
        + 1;
    assert_eq!(
        found,
        vec![
            format!("planted_tests.rs:{sidecar_line}"),
            format!("release.rs:{release_line}"),
            format!("release.rs:{words_line}"),
        ],
        "the planted text lists were not found, in the order the walk reads them"
    );
    crate::test_support::remove(root.as_path());
}

/// The same lists written as slices are no finding,
/// so the scan answers the shape it is about rather than any fixed length.
#[test]
fn the_scan_passes_the_same_lists_written_as_slices() {
    let root: std::path::PathBuf =
        crate::test_support::fixture("source-guard-hand-counted-list-slices");
    write_tree(
        root.as_path(),
        &[(
            "lib.rs",
            "const NAMES: &[&str] = &[\"a\", \"b\"];\n\
                 const STAGES: &[(u8, i32)] = &[(1, 2), (3, 4)];\n",
        )],
    );
    let found: Vec<String> =
        findings_in(root.as_path(), &[release_root("lib.rs")]).expect("the fixture sources");
    assert!(
        found.is_empty(),
        "slices carry no hand-counted length, so nothing here is a finding: {found:?}"
    );
    crate::test_support::remove(root.as_path());
}
