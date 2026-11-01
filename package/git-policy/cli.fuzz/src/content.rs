//! What:
//!  Generators and invariants for the content-policy boundaries that read untrusted
//!       text:
//!  the index listing a `git add` prediction parses,
//!  the staged delta computed
//!       from two listings,
//!  the `rulesFile` option value,
//!  and the final-newline rule over
//!       file bytes.
//! Why:
//!  Listing pathnames,
//!  configuration values and file contents are chosen by whoever
//!      writes the repository.
//!  Each check states its property without calling the code
//!      under test a second time:
//!  a listing is rendered back to bytes,
//!  a delta is
//!      recomputed from sorted maps,
//!  a rules-file name is joined to a root and inspected
//!      component by component,
//!  and normalized bytes are compared with the trimmed input.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkStageRecords(data); checkGeneratedListings(data); checkRulesFile(data); checkFinalNewline(data);
//! ```

/// Import the layer's failure causes.
use git_policy_cli::candidate_error::CandidateFailure;
/// Import the mode type and the object-name parser.
use git_policy_cli::candidate_object::{CandidateMode, parse_object_id};
/// Import the listing parser,
///  the delta and their record types.
use git_policy_cli::candidate_stage::{
    ChangedPath, StageRecord, parse_stage_records, staged_delta,
};
/// Import the `rulesFile` check and its refusals.
use git_policy_cli::config_rules_file::{RulesFileRefusal, check_rules_file};
/// Import the final-newline rule.
use git_policy_cli::policy_final_newline::normalized_final_newline;
/// `BTreeMap<K, V>` is a map kept sorted by key.
use std::collections::BTreeMap;
/// `Component`,
///  `Path` and `PathBuf` read a filesystem path component by component.
use std::path::{Component, Path, PathBuf};

/// Git's mode text for a candidate mode,
///  restated from `git ls-files --stage` output.
fn mode_text(mode: CandidateMode) -> &'static str {
    match mode {
        CandidateMode::Regular => return "100644",
        CandidateMode::Executable => return "100755",
        CandidateMode::Symlink => return "120000",
        CandidateMode::Gitlink => return "160000",
    }
}

/// What:
///  Git's `ls-files --stage -z` bytes for a list of records.
/// Why:
///   Whatever the parser accepts must be exactly these bytes,
///  so acceptance is
///       checked by rendering the result and comparing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderStageRecords(records: StageRecord[]): Buffer;
/// ```
pub fn render_stage_records(records: &[StageRecord]) -> Vec<u8> {
    let mut rendered: Vec<u8> = Vec::new();
    for record in records {
        rendered.extend_from_slice(
            format!(
                "{} {} {}\t",
                mode_text(record.mode),
                record.object.as_str(),
                record.stage
            )
            .as_bytes(),
        );
        rendered.extend_from_slice(record.path.as_slice());
        rendered.push(0);
    }
    return rendered;
}

/// What:
///  Assert the invariants of parsing arbitrary bytes as an index listing.
/// Why:
///   An accepted listing is exactly Git's rendering of what was returned,
///  so no
///       byte was skipped or invented;
///  a refusal is a malformed listing or a mode the
///       candidate layer does not read,
///  never anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkStageRecords(data: Buffer): void;
/// ```
pub fn check_stage_records(data: &[u8]) {
    match parse_stage_records(data) {
        Ok(records) => {
            assert_eq!(
                render_stage_records(records.as_slice()),
                data,
                "an accepted listing is not Git's rendering of its records"
            );
            for record in &records {
                assert!(
                    !record.path.is_empty(),
                    "an accepted record has no pathname"
                );
                assert!(record.stage <= 3, "an accepted stage is above 3");
            }
        }
        Err(error) => {
            assert!(
                error.failure == CandidateFailure::ListingMalformed
                    || error.failure == CandidateFailure::UnsupportedMode,
                "a listing was refused for {:?}",
                error.failure
            );
            assert!(
                error
                    .message
                    .starts_with("cli-git could not read the index: entry record "),
                "{}",
                error.message
            );
        }
    }
}

/// A byte cursor over fuzz input that yields zero once exhausted.
struct Cursor<'a> {
    /// The remaining bytes.
    rest: &'a [u8],
}

/// What:
///  `impl Cursor<'_>` attaches the one read.
/// Why:
///   Generators draw small choices from the input in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Cursor { next(): number }
/// ```
impl Cursor<'_> {
    /// The next byte,
    ///  or zero after the input ends.
    fn next(&mut self) -> u8 {
        match self.rest.split_first() {
            Some((first, rest)) => {
                self.rest = rest;
                return *first;
            }
            None => return 0,
        }
    }
}

/// Pathnames a generated listing draws from:
///  few enough to collide,
///  odd enough to matter.
const PATHS: [&[u8]; 8] = [
    b"a.txt",
    b"b",
    b"dir/c.md",
    b"dir/sub/d",
    b"tab\tin name",
    b"space in name",
    b"\xff-not-utf8",
    b"z",
];

/// Object names a generated listing draws from,
///  in both hash formats.
const OBJECTS: [&str; 3] = [
    "0123456789abcdef0123456789abcdef01234567",
    "fedcba9876543210fedcba9876543210fedcba98",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
];

/// Modes a generated listing draws from.
const MODES: [CandidateMode; 4] = [
    CandidateMode::Regular,
    CandidateMode::Executable,
    CandidateMode::Symlink,
    CandidateMode::Gitlink,
];

/// What:
///  One generated index:
///  at most one record per path and stage,
///  in Git's order
///       (pathname bytes,
///  then stage).
/// Why:
///   `git ls-files --stage` lists a real index that way,
///  which the delta relies on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedIndex(cursor: Cursor): StageRecord[];
/// ```
fn generated_index(cursor: &mut Cursor<'_>) -> Vec<StageRecord> {
    let mut by_key: BTreeMap<(Vec<u8>, u8), StageRecord> = BTreeMap::new();
    let count: usize = usize::from(cursor.next() % 8);
    for _ in 0..count {
        let path: Vec<u8> = PATHS[usize::from(cursor.next()) % PATHS.len()].to_vec();
        let choice: u8 = cursor.next();
        // Most entries are merged (stage 0); some are conflict stages 1 to 3.
        let stage: u8 = if choice.is_multiple_of(4) {
            choice / 4 % 4
        } else {
            0
        };
        let Some(object) =
            parse_object_id(OBJECTS[usize::from(cursor.next()) % OBJECTS.len()].as_bytes())
        else {
            unreachable!("every listed object name is valid");
        };
        let mode: CandidateMode = MODES[usize::from(cursor.next()) % MODES.len()];
        by_key.insert(
            (path.clone(), stage),
            StageRecord {
                mode,
                object,
                stage,
                path,
            },
        );
    }
    return by_key.into_values().collect();
}

/// What:
///  Two generated indexes,
///  before and after an add,
///  drawn from the input.
/// Why:
///   The second is often the first with a few entries changed,
///  added or removed,
///       so equal,
///  changed,
///  removed and new paths all occur.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedListings(data: Buffer): [StageRecord[], StageRecord[]];
/// ```
pub fn generated_listings(data: &[u8]) -> (Vec<StageRecord>, Vec<StageRecord>) {
    let mut cursor: Cursor<'_> = Cursor { rest: data };
    let before: Vec<StageRecord> = generated_index(&mut cursor);
    if cursor.next().is_multiple_of(2) {
        return (before, generated_index(&mut cursor));
    }
    // Derive the second index from the first: keep, drop or change each record, then add some.
    let mut by_key: BTreeMap<(Vec<u8>, u8), StageRecord> = BTreeMap::new();
    for record in &before {
        let choice: u8 = cursor.next() % 4;
        if choice == 1 {
            continue;
        }
        let mut kept: StageRecord = record.clone();
        if choice == 2 {
            kept.mode = MODES[usize::from(cursor.next()) % MODES.len()];
        }
        by_key.insert((kept.path.clone(), kept.stage), kept);
    }
    for record in generated_index(&mut cursor) {
        by_key.insert((record.path.clone(), record.stage), record);
    }
    return (before, by_key.into_values().collect());
}

/// The pathnames of a listing in first-seen order,
///  and each one's records in listing order.
type PathGroups = (Vec<Vec<u8>>, BTreeMap<Vec<u8>, Vec<StageRecord>>);

/// What:
///  Each pathname's records,
///  in listing order,
///  and the pathnames in first-seen order.
/// Why:
///   The delta's expected value is computed from these maps,
///  not from the subject.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordsByPath(records: StageRecord[]): [Buffer[], Map<string, StageRecord[]>];
/// ```
fn records_by_path(records: &[StageRecord]) -> PathGroups {
    let mut order: Vec<Vec<u8>> = Vec::new();
    let mut groups: BTreeMap<Vec<u8>, Vec<StageRecord>> = BTreeMap::new();
    for record in records {
        if !groups.contains_key(&record.path) {
            order.push(record.path.clone());
        }
        groups
            .entry(record.path.clone())
            .or_default()
            .push(record.clone());
    }
    return (order, groups);
}

/// What:
///  Assert the staged delta of two listings:
///  every pathname whose records differ,
///       with its records afterwards,
///  paths of the first listing first,
///  then new ones.
/// Why:
///   This is what a `git add` stages,
///  so a missed path escapes every content policy
///       and an extra one is checked for nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkStagedDelta(before: StageRecord[], after: StageRecord[]): void;
/// ```
pub fn check_staged_delta(before: &[StageRecord], after: &[StageRecord]) {
    let (before_order, before_groups) = records_by_path(before);
    let (after_order, after_groups) = records_by_path(after);
    let mut expected: Vec<ChangedPath> = Vec::new();
    for path in &before_order {
        let later: Vec<StageRecord> = after_groups.get(path).cloned().unwrap_or_default();
        if Some(&later) != before_groups.get(path) {
            expected.push(ChangedPath {
                path: path.clone(),
                after: later,
            });
        }
    }
    for path in &after_order {
        if !before_groups.contains_key(path) {
            expected.push(ChangedPath {
                path: path.clone(),
                after: after_groups.get(path).cloned().unwrap_or_default(),
            });
        }
    }
    assert_eq!(
        staged_delta(before, after),
        expected,
        "{before:?} -> {after:?}"
    );
    // Rendering and parsing each listing returns it unchanged.
    for listing in [before, after] {
        match parse_stage_records(render_stage_records(listing).as_slice()) {
            Ok(parsed) => assert_eq!(parsed.as_slice(), listing),
            Err(error) => panic!("a rendered listing was refused: {}", error.message),
        }
    }
}

/// What:
///  Assert the generated-listing invariants for one input.
/// Why:
///   One call per fuzz input reaches the delta through well-formed listings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedListings(data: Buffer): void;
/// ```
pub fn check_generated_listings(data: &[u8]) {
    let (before, after): (Vec<StageRecord>, Vec<StageRecord>) = generated_listings(data);
    check_staged_delta(before.as_slice(), after.as_slice());
}

/// Whether one character is `/`.
fn is_slash(character: char) -> bool {
    return character == '/';
}

/// What:
///  The refusal a `rulesFile` value must get,
///  stated from its words alone,
///  or nothing.
/// Why:
///   An independent statement of the documented order:
///  empty,
///  absolute,
///  drive,
///       backslash,
///  NUL,
///  then the first component that is empty or a dot name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function expectedRefusal(value: string): RulesFileRefusal | undefined;
/// ```
fn expected_refusal(value: &str) -> Option<RulesFileRefusal> {
    let bytes: &[u8] = value.as_bytes();
    if bytes.is_empty() {
        return Some(RulesFileRefusal::Empty);
    }
    if bytes[0] == b'/' {
        return Some(RulesFileRefusal::Absolute);
    }
    if bytes.len() > 1 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Some(RulesFileRefusal::Drive);
    }
    if bytes.contains(&b'\\') {
        return Some(RulesFileRefusal::Backslash);
    }
    if bytes.contains(&0) {
        return Some(RulesFileRefusal::Nul);
    }
    for component in value.split(is_slash) {
        if component.is_empty() {
            return Some(RulesFileRefusal::EmptyComponent);
        }
        if component == "." || component == ".." {
            return Some(RulesFileRefusal::DotComponent);
        }
    }
    return None;
}

/// What:
///  Assert the `rulesFile` check for one value:
///  its answer is the stated one,
///  and an
///       accepted value joined to a root names a path below that root.
/// Why:
///   The option exists so configuration can name a file in the repository;
///  a value
///       that left it would let configuration pick any file on the machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRulesFileValue(value: string): void;
/// ```
pub fn check_rules_file_value(value: &str) {
    let answer: Result<(), RulesFileRefusal> = check_rules_file(value);
    match expected_refusal(value) {
        Some(refusal) => assert_eq!(answer, Err(refusal), "{value:?}"),
        None => assert_eq!(answer, Ok(()), "{value:?}"),
    }
    if answer.is_err() {
        return;
    }
    let root: &Path = Path::new("/repository/root");
    let joined: PathBuf = root.join(value);
    assert!(
        joined.starts_with(root),
        "{value:?} left the root: {joined:?}"
    );
    let mut depth: usize = 0;
    for component in Path::new(value).components() {
        match component {
            Component::Normal(_) => depth += 1,
            other => panic!("{value:?} has the component {other:?}"),
        }
    }
    assert_eq!(depth, value.split(is_slash).count(), "{value:?}");
}

/// Words a generated `rulesFile` value is made of.
const RULES_WORDS: [&str; 14] = [
    "rules.txt",
    ".cache",
    ".",
    "..",
    "",
    "/",
    "\\",
    "C:",
    "z:",
    "\0",
    "...",
    ".hidden",
    "\u{e9}",
    ":",
];

/// What:
///  A `rulesFile` value built from the input:
///  words joined by `/` or nothing.
/// Why:
///   Raw bytes rarely spell `..` between slashes;
///  built values reach every refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedRulesFile(data: Buffer): string;
/// ```
pub fn generated_rules_file(data: &[u8]) -> String {
    let mut value: String = String::new();
    for byte in data.iter().take(12) {
        if byte & 0x80 == 0x80 {
            value.push('/');
        }
        value.push_str(RULES_WORDS[usize::from(byte & 0x7f) % RULES_WORDS.len()]);
    }
    return value;
}

/// What:
///  Assert the final-newline rule over one byte string.
/// Why:
///   Left alone means empty,
///  holding NUL,
///  not UTF-8,
///  or already ending with exactly
///       one LF;
///  a replacement keeps every byte before the trailing LF run,
///  ends with
///       exactly one LF,
///  and is itself left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkFinalNewline(bytes: Buffer): void;
/// ```
pub fn check_final_newline(bytes: &[u8]) {
    let trimmed_length: usize = bytes.len() - bytes.iter().rev().take_while(is_line_feed).count();
    let trailing: usize = bytes.len() - trimmed_length;
    let text: bool = !bytes.is_empty() && !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok();
    match normalized_final_newline(bytes) {
        None => assert!(!text || trailing == 1, "{bytes:?} needed a correction"),
        Some(replacement) => {
            assert!(text && trailing != 1, "{bytes:?} was corrected needlessly");
            assert_eq!(
                &replacement[..replacement.len() - 1],
                &bytes[..trimmed_length]
            );
            assert_eq!(replacement.last(), Some(&b'\n'));
            assert_eq!(normalized_final_newline(replacement.as_slice()), None);
        }
    }
}

/// Named predicate for a line feed byte.
fn is_line_feed(byte: &&u8) -> bool {
    return **byte == b'\n';
}

/// What:
///  Text built from the input with a chosen number of trailing line feeds.
/// Why:
///   Raw bytes are mostly not UTF-8;
///  built text reaches every correction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedText(data: Buffer): Buffer;
/// ```
pub fn generated_text(data: &[u8]) -> Vec<u8> {
    let Some((first, rest)) = data.split_first() else {
        return Vec::new();
    };
    let mut text: Vec<u8> = String::from_utf8_lossy(rest).into_owned().into_bytes();
    // `repeat_n` yields the line feed `first % 4` times.
    text.extend(std::iter::repeat_n(b'\n', usize::from(first % 4)));
    return text;
}

/// Generator reach and fixed hard cases stay out of the fuzz targets.
#[cfg(test)]
#[path = "content_tests.rs"]
mod tests;
