//! What:
//!  Controls for the reason a relink's last try failed,
//!  kept for the notice a given-up record leaves.
//! Why:
//!  The words a person reads about a worktree whose links still name another checkout must say what failed,
//!  and the answer the tool gave is the only thing that does;
//!  a reason that is lost, or read twice, or left behind after the record settled,
//!  is a line the person cannot act on.

use super::*;
use crate::test_support::{fixture, remove};

/// What:
///  A kept reason is read back exactly as it was kept,
///  and keeping another one replaces it.
/// Why:
///   One record has one last try,
///   so what the person reads is that try's answer and never an earlier one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember(gitDir, 'relink-1', 'first'); remember(gitDir, 'relink-1', 'second'); assert(recall(gitDir, 'relink-1') === 'second');
/// ```
#[test]
fn a_kept_reason_is_read_back_and_replaced_by_the_next_try() {
    let root: PathBuf = fixture("relink-failure-kept");
    remember(root.as_path(), "relink-1", "pnpm: exit status 1").expect("kept");
    assert_eq!(recall(root.as_path(), "relink-1").as_deref(), Some("pnpm: exit status 1"));
    remember(root.as_path(), "relink-1", "pnpm: exit status 1: the second answer").expect("kept");
    assert_eq!(recall(root.as_path(), "relink-1").as_deref(), Some("pnpm: exit status 1: the second answer"));
    remove(root.as_path());
}

/// What:
///  A record no try has failed on names no reason.
/// Why:
///   The notice words a record whose tries ran out with no answer as one no tool ever answered,
///   so a missing reason must read as nothing rather than as an empty answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(recall(gitDir, 'relink-never-tried') === undefined);
/// ```
#[test]
fn a_record_with_no_failure_names_no_reason() {
    let root: PathBuf = fixture("relink-failure-none");
    assert_eq!(recall(root.as_path(), "relink-never-tried"), None);
    remove(root.as_path());
}

/// What:
///  A record that settled leaves no reason behind,
///  and forgetting one that is not there is nothing to report.
/// Why:
///   Nothing reads a settled record's reason again,
///  so it would be a file the person never sees and no step removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember(...); forget(gitDir, id); assert(recall(gitDir, id) === undefined); forget(gitDir, id);
/// ```
#[test]
fn a_settled_record_takes_its_reason_away() {
    let root: PathBuf = fixture("relink-failure-settled");
    remember(root.as_path(), "relink-1", "toolchain: exit status 1").expect("kept");
    forget(root.as_path(), "relink-1");
    assert_eq!(recall(root.as_path(), "relink-1"), None);
    forget(root.as_path(), "relink-1");
    remove(root.as_path());
}
