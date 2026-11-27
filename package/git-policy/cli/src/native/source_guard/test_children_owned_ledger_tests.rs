//! What:
//!  The ledger of the child starts that still name no owner,
//!  and the comparison that fails on a new site and on a line whose site is gone.
//! Why:
//!  The scan refuses every bare start,
//!  and this crate holds many of them today,
//!  so the control would fail on the tree it must pass.
//!  The debt is paid folder by folder,
//!  and the ledger is what says which sites are left:
//!  one line per site,
//!  its path and why the site is not moved yet,
//!  so moving one site is deleting one line.
//!  A site a file gains without a ledger line fails here,
//!  and a ledger line whose site the file no longer holds fails too,
//!  so the list can only shrink.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function entries(ledger: string): Entry[]
//! export function ledgerFindings(sites: Map<string, number[]>, sources: string[], entries: Entry[]): string[]
//! ```

use std::collections::BTreeMap;

/// The character that ends the path of a ledger line and begins its reason.
const SEPARATOR: char = '|';

/// What:
///  `struct Entry { ... }` is one line of the ledger:
///  the file that holds the site,
///  relative to `src/native`,
///  and why the site is not moved onto `spawn_owned` yet.
///  `String` is an owned text buffer (sibling:
/// `&str`,
/// a borrowed view).
/// Why:
///  `String` and not `&str` because the entries outlive the text they were read from,
///  and the reason is read back in a failure message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { path: string; reason: string };
/// ```
#[derive(Debug)]
pub(super) struct Entry {
    /// The file that holds the site,
    /// `/`-separated and relative to `src/native`.
    pub(super) path: String,
    /// Why the site is not moved yet.
    pub(super) reason: String,
}

/// What:
///  `pub(super) fn entries(ledger: &str) -> Result<Vec<Entry>, String>` reads the ledger text into its entries.
///  A line that is blank or begins with `#` is a note and is skipped.
///  `Result<Vec<Entry>, String>` is "the entries or one text saying what is wrong" (sibling:
/// `Option<Vec<Entry>>`,
///  which cannot say why).
/// Why:
///  The ledger is data beside the code,
///  so a person can add a line while moving a folder,
///  and a malformed line fails here rather than silently excusing nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entries(ledger: string): Entry[]  // throws Error
/// ```
pub(super) fn entries(ledger: &str) -> Result<Vec<Entry>, String> {
    let mut read: Vec<Entry> = Vec::new();
    for (index, line) in ledger.lines().enumerate() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // `split_once` cuts at the first separator only, so a reason may carry the character itself.
        let Some((path, reason)) = trimmed.split_once(SEPARATOR) else {
            return Err(format!(
                "ledger line {} holds no `{SEPARATOR}`: {trimmed}",
                index + 1
            ));
        };
        read.push(Entry {
            path: String::from(path.trim()),
            reason: String::from(reason.trim()),
        });
    }
    return Ok(read);
}

/// What:
///  `fn count_by_path(entries: &[Entry]) -> BTreeMap<String, usize>` is how many ledger lines name each file.
///  `BTreeMap<String, usize>` is a map kept in key order (TS:
/// `Map`,
/// but iterated sorted).
/// Why:
///  One line per site means the count is the number of sites the ledger still expects in that file,
///  and it is what a start added to a ledgered file changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function countByPath(entries: Entry[]): Map<string, number>
/// ```
fn count_by_path(entries: &[Entry]) -> BTreeMap<String, usize> {
    let mut counted: BTreeMap<String, usize> = BTreeMap::new();
    for entry in entries {
        // What: `entry` gives `Some(count)` when the path is already counted, and `unwrap_or(0)` reads 0 otherwise.
        // Why:  A file's first line starts its count at one.
        let seen: usize = counted.get(entry.path.as_str()).copied().unwrap_or(0);
        counted.insert(entry.path.clone(), seen + 1);
    }
    return counted;
}

/// What:
///  `pub(super) fn ledger_findings(sites, test_sources, entries) -> Vec<String>` lists every way the ledger and the
///  tree disagree.
///  `sites` maps a file to the line numbers of its bare starts,
///  `test_sources` holds the crate's test-only files,
///  and `entries` is the ledger.
/// Why:
///  A finding here is either a new site the ledger does not name,
///  a line whose site is gone,
///  a line whose reason is empty,
///  or a line naming a file that is no test source,
///  and each sentence says which of those it is so the way out is the repair written beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ledgerFindings(sites: Map<string, number[]>, testSources: string[], entries: Entry[]): string[]
/// ```
pub(super) fn ledger_findings(
    sites: &BTreeMap<String, Vec<usize>>,
    test_sources: &[String],
    entries: &[Entry],
) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let expected: BTreeMap<String, usize> = count_by_path(entries);
    // A path that is no test source is reported once, however many lines name it, and its count is not judged here.
    let mut reported_absent: Vec<String> = Vec::new();
    for entry in entries {
        if entry.reason.is_empty() {
            found.push(format!("{}: the ledger line gives no reason, and a line without one says nothing to the round that takes the site", entry.path));
        }
        let is_a_source: bool = test_sources.iter().any(|path| return *path == entry.path);
        if !is_a_source && !reported_absent.contains(&entry.path) {
            reported_absent.push(entry.path.clone());
            found.push(format!("{}: the ledger names it as a test source, and no `mod` declaration makes it one, so its lines outlived the file", entry.path));
        }
    }
    for (path, lines) in sites {
        let named: usize = expected.get(path).copied().unwrap_or(0);
        if lines.len() > named {
            found.push(format!(
                "{path}:{}: a start that goes through neither `spawn_owned` nor `own_group`, and the ledger names {named} of {} sites here",
                lines[named.min(lines.len() - 1)],
                lines.len()
            ));
        }
    }
    for (path, named) in &expected {
        if reported_absent.contains(path) {
            continue;
        }
        let held: usize = sites.get(path).map_or(0, |lines| return lines.len());
        if held < *named {
            found.push(format!("{path}: the ledger still names {named} site(s) and the file holds {held}, so remove {} ledger line(s)", named - held));
        }
    }
    return found;
}

/// The reader takes one line per site,
/// skips a note and a blank line,
/// and keeps a reason that carries the separator itself.
#[test]
fn one_line_is_one_site_and_a_note_is_no_site() {
    let read: Vec<Entry> = entries(
        "# a note\n\nbinary/a_tests.rs | not moved yet\nbinary/a_tests.rs | not moved | yet\n",
    )
    .expect("a readable ledger");
    assert_eq!(
        read.len(),
        2,
        "two lines are two sites, and the note and the blank line are none"
    );
    assert_eq!(read[0].path, "binary/a_tests.rs");
    assert_eq!(
        read[1].reason, "not moved | yet",
        "only the first separator cuts the line"
    );
}

/// A line with no separator fails with its number,
/// so a hand-edited ledger says where it went wrong instead of excusing nothing.
#[test]
fn a_line_without_a_separator_fails_with_its_number() {
    let error: String = entries("# a note\nbinary/a_tests.rs not moved\n")
        .expect_err("the second line has no separator");
    assert_eq!(
        error,
        "ledger line 2 holds no `|`: binary/a_tests.rs not moved"
    );
}

/// A line whose reason is empty says nothing to the round that takes the site,
/// and one that names no source of this crate has outlived its file.
#[test]
fn a_line_without_a_reason_and_one_of_an_absent_file_are_both_reported() {
    let read: Vec<Entry> =
        entries("binary/a_tests.rs |\ngone_tests.rs | not moved yet\n").expect("a readable ledger");
    let found: Vec<String> = ledger_findings(
        &BTreeMap::new(),
        &[String::from("binary/a_tests.rs")],
        &read,
    );
    assert!(
        found
            .iter()
            .any(|each| return each.contains("gives no reason")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|each| return each.contains("gone_tests.rs")
                && each.contains("no `mod` declaration")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|each| return each.contains("still names 1 site(s) and the file holds 0")),
        "{found:?}"
    );
    assert_eq!(
        found.len(),
        3,
        "an absent file is named once, and the count of the one that is a source is still judged: {found:?}"
    );
}
