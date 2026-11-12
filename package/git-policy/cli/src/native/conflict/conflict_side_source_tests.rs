//! What:
//!  Finding and copying the worktree file of a conflicted path.
//! Why:
//!  A copy that is stale,
//! torn or taken from the wrong object would let the command that follows destroy the only
//!  current version of a half-resolved file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(locate(root, 'a.rs')).toBeDefined(); expect(locate(root, 'gone.rs')).toBeUndefined();
//! ```

use super::{FailureKind, SaveFailure, WhenTheSourceChanges, copy_located, locate};
use crate::test_support::{fixture, remove};

/// A file that is there is found and copied whole;
/// a path with nothing there is not found.
#[test]
fn a_present_file_is_located_and_copied() {
    let base = fixture("source-present");
    std::fs::write(base.join("a.rs"), b"half \xff resolved\n").expect("a");
    let located = locate(base.as_path(), b"a.rs").expect("lookup").expect("present");
    copy_located(&located, base.join("copy").as_path(), &mut || {}, WhenTheSourceChanges::Remove).expect("copy");
    assert_eq!(std::fs::read(base.join("copy")).expect("copy"), b"half \xff resolved\n");
    assert!(locate(base.as_path(), b"gone.rs").expect("lookup").is_none());
    remove(base.as_path());
}

/// A copy that meets a writer is kept when the caller asks for the bytes the source held,
///  and the same copy is removed when the caller would rather copy again.
#[test]
fn a_changed_source_leaves_its_copy_behind_only_when_the_caller_keeps_it() {
    let base = fixture("source-kept");
    let content: Vec<u8> = large_source(base.as_path());
    let located = locate(base.as_path(), b"a.rs").expect("lookup").expect("present");
    let mut appended: bool = false;
    let failure: SaveFailure = copy_located(
        &located,
        base.join("kept").as_path(),
        &mut || {
            if appended {
                return;
            }
            appended = true;
            let mut file = std::fs::OpenOptions::new().append(true).open(base.join("a.rs")).expect("open");
            std::io::Write::write_all(&mut file, b"one more line\n").expect("append");
        },
        WhenTheSourceChanges::Keep,
    )
    .expect_err("a source that grew while it was copied");
    assert_eq!(failure.kind, FailureKind::Changed, "{failure:?}");
    let kept: Vec<u8> = std::fs::read(base.join("kept")).expect("the copy the caller kept");
    assert_eq!(kept.len(), content.len(), "the copy holds what the file held when it was opened");
    // The same source with the other answer leaves nothing behind.
    let mut appended_again: bool = false;
    let removed: SaveFailure = copy_located(
        &located,
        base.join("removed").as_path(),
        &mut || {
            if appended_again {
                return;
            }
            appended_again = true;
            let mut file = std::fs::OpenOptions::new().append(true).open(base.join("a.rs")).expect("open");
            std::io::Write::write_all(&mut file, b"another line\n").expect("append");
        },
        WhenTheSourceChanges::Remove,
    )
    .expect_err("a source that grew while it was copied again");
    assert_eq!(removed.kind, FailureKind::Changed, "{removed:?}");
    assert!(!base.join("removed").exists(), "the copy is removed when the caller would copy again");
    remove(base.as_path());
}

/// A link or a file in front of the last name is a failure that names that ancestor;
/// a missing ancestor is nothing to save.
#[test]
fn an_ancestor_that_is_a_link_or_a_file_is_a_failure_naming_it() {
    let base = fixture("source-ancestors");
    std::fs::create_dir_all(base.join("elsewhere")).expect("elsewhere");
    std::fs::write(base.join("elsewhere/file"), b"reached through the link\n").expect("file");
    std::os::unix::fs::symlink("elsewhere", base.join("linked")).expect("link");
    std::os::unix::fs::symlink("nowhere", base.join("dangling")).expect("dangling link");
    std::fs::write(base.join("plain"), b"a file\n").expect("plain file");
    std::fs::create_dir_all(base.join("real/inner")).expect("real directories");
    std::os::unix::fs::symlink("../elsewhere", base.join("real/inner/deep")).expect("deep link");
    for (relative, subject) in [
        ("linked/file", "linked"),
        ("dangling/file", "dangling"),
        ("plain/file", "plain"),
        ("real/inner/deep/file", "real/inner/deep"),
    ] {
        let failure: SaveFailure = locate(base.as_path(), relative.as_bytes()).expect_err(relative);
        assert_eq!(failure.subject, subject, "{relative}: {failure:?}");
        assert_eq!(failure.kind, FailureKind::Ancestor, "{relative}: {failure:?}");
        assert!(!failure.reason.is_empty(), "{relative}: {failure:?}");
    }
    assert!(locate(base.as_path(), b"absent/file").expect("a missing ancestor").is_none());
    assert!(locate(base.as_path(), b"real/absent").expect("a missing name").is_none());
    remove(base.as_path());
}

/// Copy `path` below `base` to `base/copy`,
/// running `change` once,
/// after the first chunk was written.
fn copy_while_changing(base: &std::path::Path, path: &str, change: &dyn Fn()) -> Result<(), SaveFailure> {
    let located = locate(base, path.as_bytes()).expect("lookup").expect("present");
    let mut changed: bool = false;
    return copy_located(
        &located,
        base.join("copy").as_path(),
        &mut || {
            if !changed {
                changed = true;
                change();
            }
        },
        WhenTheSourceChanges::Remove,
    )
    .map(|_stamp| ());
}

/// A source larger than one chunk of the copy loop,
/// so the copy is still running when the change happens.
fn large_source(base: &std::path::Path) -> Vec<u8> {
    let content: Vec<u8> = (0..300_000_u32).map(|each| { return u8::try_from(each % 251).expect("fits"); }).collect();
    std::fs::write(base.join("a.rs"), content.as_slice()).expect("a");
    // An old modification time, so a rewrite within the same clock tick still shows.
    let hour_ago: std::time::SystemTime = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    std::fs::File::options().write(true).open(base.join("a.rs")).expect("open").set_modified(hour_ago).expect("old time");
    return content;
}

/// A file that does not change during the copy is copied whole and kept.
#[test]
fn a_file_that_holds_still_is_copied_whole() {
    let base = fixture("source-still");
    let content: Vec<u8> = large_source(base.as_path());
    copy_while_changing(base.as_path(), "a.rs", &|| {}).expect("copy");
    assert_eq!(std::fs::read(base.join("copy")).expect("copy"), content);
    remove(base.as_path());
}

/// A source that changes while it is copied is never reported as saved,
/// whichever way it changes,
/// and no copy stays.
#[test]
fn a_source_that_changes_during_the_copy_is_a_failure_and_leaves_no_copy() {
    type Change = Box<dyn Fn(&std::path::Path)>;
    let changes: Vec<(&str, Change)> = vec![
        (
            "an atomic save over the path",
            Box::new(|base: &std::path::Path| {
                std::fs::write(base.join("a.rs.new"), b"the newer version\n").expect("new version");
                std::fs::rename(base.join("a.rs.new"), base.join("a.rs")).expect("rename over");
            }),
        ),
        (
            "an append in place",
            Box::new(|base: &std::path::Path| {
                let mut file = std::fs::OpenOptions::new().append(true).open(base.join("a.rs")).expect("open");
                std::io::Write::write_all(&mut file, b"more\n").expect("append");
            }),
        ),
        (
            "a truncation in place",
            Box::new(|base: &std::path::Path| {
                std::fs::File::options().write(true).open(base.join("a.rs")).expect("open").set_len(10).expect("truncate");
            }),
        ),
        (
            "a rewrite in place that keeps the size",
            Box::new(|base: &std::path::Path| {
                let mut file = std::fs::OpenOptions::new().write(true).open(base.join("a.rs")).expect("open");
                std::io::Write::write_all(&mut file, b"same size, other bytes").expect("rewrite");
            }),
        ),
        (
            "the path now names a link",
            Box::new(|base: &std::path::Path| {
                std::fs::remove_file(base.join("a.rs")).expect("remove");
                std::os::unix::fs::symlink("elsewhere", base.join("a.rs")).expect("link");
            }),
        ),
    ];
    for (round, (label, change)) in changes.into_iter().enumerate() {
        let base = fixture(format!("source-changes-{round}").as_str());
        large_source(base.as_path());
        let error = copy_while_changing(base.as_path(), "a.rs", &|| { change(base.as_path()); }).expect_err(label);
        assert!(!error.reason.is_empty() && error.kind == FailureKind::Changed, "{label}: {error:?}");
        assert!(!base.join("copy").exists(), "{label}: no cut-short or stale copy stays");
        remove(base.as_path());
    }
}

/// A file that keeps growing is read only as far as the size it had when it was opened,
///  so a writer that never stops cannot hold the copy open for good:
///  the copy stops at that size and the changed source is rejected.
#[test]
fn a_source_that_keeps_growing_is_read_only_to_the_size_it_was_opened_at() {
    let base = fixture("source-growing");
    let content: Vec<u8> = large_source(base.as_path());
    let located = locate(base.as_path(), b"a.rs").expect("lookup").expect("present");
    let mut appended: usize = 0;
    // The writer this control supplies stops at its own bound,
    //  so a copy that read to the end of a growing file would reach that bound
    //  instead of the size the file had when it was opened.
    let writer_bound: usize = 10_000;
    let error: SaveFailure = copy_located(
        &located,
        base.join("copy").as_path(),
        &mut || {
            if appended < writer_bound {
                let mut file = std::fs::OpenOptions::new().append(true).open(base.join("a.rs")).expect("open");
                std::io::Write::write_all(&mut file, b"x").expect("append");
                appended += 1;
            }
        },
        WhenTheSourceChanges::Remove,
    )
    .expect_err("a source that grew while it was copied");
    assert_eq!(error.kind, FailureKind::Changed, "{error:?}");
    assert!(
        appended < writer_bound,
        "the copy reads the {} bytes it opened, not the bytes a writer keeps adding: {appended} appends",
        content.len()
    );
    assert!(!base.join("copy").exists(), "no cut-short copy stays");
    remove(base.as_path());
}
