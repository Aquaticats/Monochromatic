//! Controls for appending to the shared live log:
//! append only,
//! created when missing,
//! once per commit,
//! and whole under
//! concurrent appenders.

use super::*;
use crate::test_support::{fixture, remove};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Threads that deliver at the same time,
/// each through its own descriptor,
/// as separate processes would.
const APPENDERS: usize = 6;

/// Entries each thread delivers.
const ENTRIES_PER_APPENDER: usize = 5;

/// Lines in one concurrent entry;
/// long enough that a writer splitting an entry into several writes is overtaken.
const LINES_PER_ENTRY: usize = 200;

/// A commit name whose first characters are `tag`.
fn sha(tag: &str) -> String {
    return format!("{tag:0<40}");
}

/// An entry text that names the commit,
/// ends with its receipt line and the blank line that separates entries.
fn entry(tag: &str) -> String {
    return format!("- Date:\n  D\n  Subject:\n  {} `s`\n  reviewed: {}\n\n", token(sha(tag).as_str()), sha(tag));
}

/// A log path inside a fixture directory that does not exist yet,
/// as the live log is before the first review.
fn absent_log(root: &Path) -> PathBuf {
    return root.join("monochromatic-git-policy-cli").join("standards-review-shadow.md");
}

/// An entry is added after what the log holds,
/// whatever its ending,
/// and nothing already there changes.
#[test]
fn an_entry_goes_after_the_existing_content() {
    let root: PathBuf = fixture("append-after");
    let log: PathBuf = root.join("log.md");
    std::fs::write(log.as_path(), "# Log\nlast line without newline").expect("log");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::Appended));
    let text: String = std::fs::read_to_string(log.as_path()).expect("log");
    assert_eq!(text, format!("# Log\nlast line without newline{}", entry("aaa")));
    remove(root.as_path());
}

/// A missing log and its missing directory are created by the first delivery,
/// and the next one appends to the same file.
#[test]
fn a_missing_log_and_its_directory_are_created() {
    let root: PathBuf = fixture("append-created");
    let log: PathBuf = absent_log(root.as_path());
    assert!(!log.parent().expect("parent").exists());
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::Appended));
    assert_eq!(deliver(log.as_path(), sha("bbb").as_str(), entry("bbb").as_str()), Ok(Delivery::Appended));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("log"), format!("{}{}", entry("aaa"), entry("bbb")));
    remove(root.as_path());
}

/// A log the service creates is readable by its owner only,
/// because entries carry commit subjects.
#[cfg(unix)]
#[test]
fn a_created_log_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("append-private");
    let log: PathBuf = absent_log(root.as_path());
    deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()).expect("delivery");
    let mode: u32 = std::fs::metadata(log.as_path()).expect("log metadata").permissions().mode();
    let others: u32 = 0o077;
    assert_eq!(mode & others, 0, "{mode:o}");
    remove(root.as_path());
}

/// Delivering the same commit again adds nothing,
/// even when its entry was reflowed by another tool in between.
#[test]
fn a_commit_is_added_once_even_after_a_reflow() {
    let root: PathBuf = fixture("append-once");
    let log: PathBuf = absent_log(root.as_path());
    deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()).expect("first");
    let reflowed: String = std::fs::read_to_string(log.as_path()).expect("log").replace("  Subject:\n  ", "  Subject: ");
    std::fs::write(log.as_path(), reflowed.as_str()).expect("reflow");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::AlreadyThere));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("log"), reflowed);
    remove(root.as_path());
}

/// A log that exists but cannot be read is reported and appended to nowhere,
/// because appending blind could repeat an entry.
#[test]
fn an_unreadable_log_is_reported_and_left_alone() {
    let root: PathBuf = fixture("append-unreadable");
    // A directory where the log should be: reading it fails with something other than "not found".
    let log: PathBuf = root.join("log.md");
    std::fs::create_dir(log.as_path()).expect("directory in place of the log");
    let result: Result<Delivery, String> = deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str());
    assert!(matches!(&result, Err(message) if message.contains("reading")), "{result:?}");
    assert!(log.is_dir());
    remove(root.as_path());
}

/// The expected entry of appender `thread`'s `index`th delivery:
/// a header naming it and many lines that each name it again,
/// so an entry that was split across writes shows another entry's lines inside it.
fn concurrent_entry(thread: usize, index: usize) -> (String, String) {
    let tag: String = format!("{thread:06}{index:06}");
    let lines: String = (0..LINES_PER_ENTRY).map(|line| return format!("  {tag} line {line}\n")).collect::<String>();
    return (tag.clone(), format!("- Entry {tag}:\n{lines}\n"));
}

/// Several threads deliver at once to one log that does not exist yet,
/// each through its own open file,
/// and every entry
/// lands whole,
/// once,
/// with no lock:
/// no entry has another entry's lines inside it,
/// and none is missing or repeated.
#[test]
fn concurrent_appends_land_whole() {
    let root: PathBuf = fixture("append-concurrent");
    let log: PathBuf = absent_log(root.as_path());
    let start: std::sync::Barrier = std::sync::Barrier::new(APPENDERS);
    std::thread::scope(|scope| {
        for thread in 0..APPENDERS {
            let (shared, barrier) = (log.as_path(), &start);
            scope.spawn(move || {
                barrier.wait();
                for index in 0..ENTRIES_PER_APPENDER {
                    let (tag, text) = concurrent_entry(thread, index);
                    let delivered: Result<Delivery, String> = deliver(shared, tag.as_str(), text.as_str());
                    assert_eq!(delivered, Ok(Delivery::Appended));
                }
            });
        }
    });
    let written: String = std::fs::read_to_string(log.as_path()).expect("log");
    // Each expected entry, by tag.
    let mut expected: BTreeMap<String, String> = BTreeMap::new();
    for thread in 0..APPENDERS {
        for index in 0..ENTRIES_PER_APPENDER {
            let (tag, text) = concurrent_entry(thread, index);
            expected.insert(tag, text);
        }
    }
    // The log is the entries back to back; cutting it at each entry's header must give each entry exactly.
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    for chunk in written.split_inclusive("\n\n") {
        let tag: String = chunk.lines().next().unwrap_or_default().trim_start_matches("- Entry ").trim_end_matches(':').to_string();
        assert!(found.insert(tag.clone(), chunk.to_string()).is_none(), "entry {tag} appears twice");
    }
    assert_eq!(found.len(), expected.len());
    assert_eq!(found, expected);
    assert_eq!(written.len(), expected.values().map(String::len).sum::<usize>(), "no byte outside an expected entry");
    remove(root.as_path());
}

/// The name a remote answer can spell:
/// a commit's own name inside another entry's verdict text.
#[test]
fn a_commit_name_inside_another_entry_is_not_a_receipt() {
    let root: PathBuf = fixture("append-inside-text");
    let log: PathBuf = root.join("log.md");
    let named: String = sha("bbb");
    std::fs::write(log.as_path(), format!("- Date:\n  D\n  Subject:\n  {} `s`\n  cost_verdict: no ({named} 0.9)\n\n", token(sha("aaa").as_str()))).expect("log");
    assert!(!names_commit(log.as_path(), named.as_str()), "text inside another entry is not this commit's receipt");
    assert_eq!(deliver(log.as_path(), named.as_str(), entry("bbb").as_str()), Ok(Delivery::Appended), "so the review of the named commit still runs");
    remove(root.as_path());
}

/// An entry the operating system cut short before its receipt is not a finished review:
/// the retry appends the whole entry after the fragment and keeps the fragment.
#[test]
fn a_fragment_before_the_receipt_is_not_a_finished_review() {
    let root: PathBuf = fixture("append-fragment");
    let log: PathBuf = root.join("log.md");
    let whole: String = entry("aaa");
    let token_text: String = token(sha("aaa").as_str());
    let cut: usize = whole.find(token_text.as_str()).expect("the entry names the commit") + token_text.len();
    let fragment: &str = &whole[..cut];
    std::fs::write(log.as_path(), fragment).expect("seed the fragment");
    assert!(!names_commit(log.as_path(), sha("aaa").as_str()), "a fragment is not a receipt");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), whole.as_str()), Ok(Delivery::Appended));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("log"), format!("{fragment}{whole}"), "the fragment is kept and the whole entry follows it");
    remove(root.as_path());
}
