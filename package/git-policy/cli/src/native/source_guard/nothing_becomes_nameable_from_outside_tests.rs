//! What:
//!  The ban on an item becoming nameable from outside the crate without a line in the list beside this file:
//!  every name the library target publishes is written once in
//!  `items_nameable_from_outside_release.txt` with the reason it is there,
//!  and a name that is published without its line fails this check.
//! Why:
//!  The library target is a published surface,
//!  and the release `0.6.1-alpha.48` widened it without anyone choosing to:
//!  a split moved a private helper into a file of its own and wrote bare `pub` so a sibling could call it,
//!  which the `pub mod` chain then published to every consumer
//!  (the review's M1 counted 31 items newly nameable,
//!  7 of them private before the split).
//!  A `pub(crate)` or `pub(super)` or `pub(in path)` says the same thing to the caller
//!  and publishes nothing,
//!  so the list is where publishing becomes a decision a reader sees.
//!  The reading is wider than the one the first version of this check made:
//!  a function whose kind is qualified (`pub async fn`),
//!  a field and a method of a published type,
//!  an item of a kind and name another module already publishes (keyed by its module path),
//!  and a `pub use` written over several lines are all read now,
//!  because a split writes all of them and the first version saw none
//!  (review M1: 154 published `async` names, 1327 methods, 3915 fields and eight carried facades had no line).
//! Gotcha:
//!  The reader's own doc, `nameable_items_of_a_source_tree_tests.rs`, states what this reading cannot see,
//!  and every limit there is a shape no line covers yet.
//!  The list is compared in both directions:
//!  a published name that is missing fails,
//!  and so does a line whose name is no longer published,
//!  because a list that keeps stale lines stops saying what the surface is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const item of surface.items) if (!listed.has(item.key)) fail(item);
//! for (const line of listed) if (!published.has(line)) fail(line);
//! ```

use super::nameable_items_of_a_source_tree_tests::surface_of;
use crate::source_scan_tests::source_root;
use std::collections::BTreeSet;

/// The list beside this file,
/// which is also the ledger of every publishing decision.
const THE_LIST: &str = include_str!("items_nameable_from_outside_release.txt");

/// What:
///  `fn listed_lines() -> BTreeSet<String>` reads the list's entries,
///  each written `<module path> <kind> <name>` or `<module path> pub use <target>::<name>`,
///  and ignores its comments and blank lines.
///  `BTreeSet` is a sorted set (as TS's `Set` iterated in sorted order).
/// Why:
///  A reason is written above the entries it covers,
///  so only the entry lines are compared with the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listedLines = (): Set<string> => new Set(THE_LIST.split('\n').filter((l) => l && !l.startsWith('#')));
/// ```
fn listed_lines() -> BTreeSet<String> {
    let mut listed: BTreeSet<String> = BTreeSet::new();
    for line in THE_LIST.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        listed.insert(trimmed.to_owned());
    }
    return listed;
}

/// What:
///  `fn is_carried(entry: &str) -> bool` answers whether a list line is a `pub use` entry rather than an item.
/// Why:
///  The two halves of the surface are compared against their own lines,
///  and a `pub use` line names a module and a name while an item line names a kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isCarried = (entry: string): boolean => entry.includes(' pub use ');
/// ```
fn is_carried(entry: &str) -> bool {
    return entry.contains(" pub use ");
}

/// What:
///  `fn the_findings(listed: &BTreeSet<String>) -> Vec<String>` compares the published surface with the list in
///  both directions and answers one line per difference.
/// Why:
///  One answer names every item a reader has to decide about,
///  rather than stopping at the first,
///  so a round that narrows seven helpers sees all seven.
///  Each line starts with the word the list would take,
///  so an entry can be copied from a run's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFindings(listed: Set<string>): string[]
/// ```
fn the_findings(listed: &BTreeSet<String>) -> Vec<String> {
    let surface = surface_of(&source_root()).expect("this crate's own source tree");
    let mut published: BTreeSet<String> = BTreeSet::new();
    for item in &surface.items {
        published.insert(item.key.clone());
    }
    let mut findings: Vec<String> = Vec::new();
    for item in &surface.items {
        if listed.contains(item.key.as_str()) {
            continue;
        }
        findings.push(format!(
            "NAMEABLE BUT NOT LISTED: {} in {} takes `pub` where `pub(crate)` or `pub(super)` would do, or it belongs in the list with a reason",
            item.key, item.file
        ));
    }
    for entry in listed.iter().filter(|entry| return !is_carried(entry)) {
        if !published.contains(entry) {
            findings.push(format!(
                "LISTED BUT NOT NAMEABLE: {entry} is no longer published from outside the crate: delete its line from the list"
            ));
        }
    }
    let mut carried: BTreeSet<String> = BTreeSet::new();
    for site in &surface.carried {
        carried.insert(site.key.clone());
        if !listed.contains(site.key.as_str()) {
            findings.push(format!(
                "CARRIED BUT NOT LISTED: `{}` in {} publishes a name out of a module that `pub mod` does not reach",
                site.key, site.file
            ));
        }
    }
    for entry in listed.iter().filter(|entry| return is_carried(entry)) {
        if !carried.contains(entry) {
            findings.push(format!(
                "LISTED BUT NOT CARRIED: {entry} no longer re-exports a name out of a private module: delete its line from the list"
            ));
        }
    }
    return findings;
}

/// What:
///  `the list of names the library target publishes holds every published name and no other` is the check:
///  it fails with one line per difference between the tree's published surface and the list beside it.
/// Why:
///  Without it a split can widen the published API in a commit that says it moves code,
///  which is what happened in `0.6.1-alpha.48` (review M1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('matches the list', () => { expect(theFindings(listedLines())).toEqual([]); });
/// ```
#[test]
fn the_list_of_published_names_holds_every_published_name_and_no_other() {
    let listed: BTreeSet<String> = listed_lines();
    let findings: Vec<String> = the_findings(&listed);
    assert!(
        findings.is_empty(),
        "the library target's published surface and the list beside this check disagree:\n{}",
        findings.join("\n")
    );
}
