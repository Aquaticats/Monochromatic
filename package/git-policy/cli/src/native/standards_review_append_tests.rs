//! Controls for appending to a shared log: append only, never create, once per commit, and a bounded repair.

use super::*;
use crate::test_support::{fixture, remove};
use std::time::Duration;

/// A commit name whose first characters are `tag`.
fn sha(tag: &str) -> String {
    return format!("{tag:0<40}");
}

/// An entry text that names the commit and ends with the blank line that separates entries.
fn entry(tag: &str) -> String {
    return format!("- Date:\n  D\n  Subject:\n  {} `s`\n\n", token(sha(tag).as_str()));
}

/// A directory with a log holding `content`, and a spool beside it.
fn setup(name: &str, content: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let log: PathBuf = root.join("log.md");
    std::fs::write(log.as_path(), content).expect("log");
    let spool: PathBuf = spool_directory(root.as_path(), "/worktree");
    return (root, log, spool);
}

/// An entry is added after what the log holds, whatever its ending, and nothing already there changes.
#[test]
fn an_entry_goes_after_the_existing_content() {
    let (root, log, spool) = setup("append-after", "# Log\nlast line without newline");
    assert_eq!(deliver(log.as_path(), Some(spool.as_path()), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::Appended));
    let text: String = std::fs::read_to_string(log.as_path()).expect("log");
    assert_eq!(text, format!("# Log\nlast line without newline{}", entry("aaa")));
    remove(root.as_path());
}

/// A missing log is never created, which would race the session that is about to write it.
#[test]
fn a_missing_log_is_not_created() {
    let root: PathBuf = fixture("append-missing");
    let log: PathBuf = root.join("absent.md");
    let result = deliver(log.as_path(), None, sha("aaa").as_str(), entry("aaa").as_str());
    assert!(result.is_err());
    assert!(!log.exists());
    remove(root.as_path());
}

/// Delivering the same commit again adds nothing, even when its entry was reflowed by another tool in between.
#[test]
fn a_commit_is_added_once_even_after_a_reflow() {
    let (root, log, spool) = setup("append-once", "");
    deliver(log.as_path(), Some(spool.as_path()), sha("aaa").as_str(), entry("aaa").as_str()).expect("first");
    let reflowed: String = std::fs::read_to_string(log.as_path()).expect("log").replace("  Subject:\n  ", "  Subject: ");
    std::fs::write(log.as_path(), reflowed.as_str()).expect("reflow");
    assert_eq!(deliver(log.as_path(), Some(spool.as_path()), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::AlreadyThere));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("log"), reflowed);
    remove(root.as_path());
}

/// An entry that another writer overwrote is put back by the next delivery, and the other writer's content stays.
#[test]
fn an_overwritten_entry_is_put_back() {
    let (root, log, spool) = setup("append-repair", "# Log\n");
    deliver(log.as_path(), Some(spool.as_path()), sha("aaa").as_str(), entry("aaa").as_str()).expect("first");
    // Another session read the log before the append and writes it back whole, with its own addition.
    std::fs::write(log.as_path(), "# Log\nanother session's note\n").expect("overwrite");
    deliver(log.as_path(), Some(spool.as_path()), sha("bbb").as_str(), entry("bbb").as_str()).expect("second");
    let text: String = std::fs::read_to_string(log.as_path()).expect("log");
    assert!(text.starts_with("# Log\nanother session's note\n"));
    assert!(text.contains(token(sha("aaa").as_str()).as_str()), "the lost entry came back");
    assert!(text.contains(token(sha("bbb").as_str()).as_str()));
    assert_eq!(text.matches(token(sha("aaa").as_str()).as_str()).count(), 1);
    remove(root.as_path());
}

/// The repair looks at the newest entries only, and the spool is trimmed to its limit.
#[test]
fn the_repair_and_the_spool_are_bounded() {
    let (root, log, spool) = setup("append-bounded", "");
    let total: usize = SPOOL_KEPT + 3;
    std::fs::create_dir_all(spool.as_path()).expect("spool");
    let base: SystemTime = SystemTime::now() - Duration::from_secs(10_000);
    for index in 0..total {
        let tag: String = format!("{index:03}");
        let path: PathBuf = spool.join(token(sha(tag.as_str()).as_str()));
        std::fs::write(path.as_path(), entry(tag.as_str())).expect("spool entry");
        std::fs::File::options().append(true).open(path.as_path()).expect("open").set_modified(base + Duration::from_secs(index as u64)).expect("age");
    }
    let repaired: usize = repair_recent(log.as_path(), spool.as_path());
    assert_eq!(repaired, REPAIR_WINDOW, "only the newest entries are checked");
    let newest: String = format!("{:03}", total - 1);
    assert!(std::fs::read_to_string(log.as_path()).expect("log").contains(token(sha(newest.as_str()).as_str()).as_str()));
    assert_eq!(std::fs::read_dir(spool.as_path()).expect("spool").count(), SPOOL_KEPT);
    assert_eq!(repair_recent(log.as_path(), spool.as_path()), 0, "a second repair finds nothing missing");
    remove(root.as_path());
}

/// Two worktrees of one repository never share a spool.
#[test]
fn worktrees_have_separate_spools() {
    let common: &Path = Path::new("/common");
    assert_ne!(spool_directory(common, "/one"), spool_directory(common, "/two"));
    assert_eq!(spool_directory(common, "/one"), spool_directory(common, "/one"));
}
