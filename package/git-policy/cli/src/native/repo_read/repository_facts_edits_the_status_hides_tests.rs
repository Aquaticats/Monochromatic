//! What:
//!  The bytes of a marked path against the object the index holds,
//!  the entries the index's own listing names,
//!  and the failures that say nothing about either.
//! Why:
//!  This reading decides whether an edit a mark hides is visible,
//!  and a comparison that trusts size, time and mode misses a same-length edit whose time was put
//!  back: the seventh review replaced `evil\n` for `base\n`, restored the nanosecond time, and a
//!  wrapped `git reset --hard` replaced the edit with no refusal
//!  (measured on this host with native Git 2.56.0, 2026-11-15).
//!  The bytes are the only proof, and one batch of `hash-object` compares them for the cost of a few
//!  Git processes.
//!  A failure is never "no edit", and a path with no worktree file is no edit of the person's,
//!  because Git writes that file back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // editsTheStatusHides(facts, repo) === ['tracked.txt']  when the marked copy differs in bytes
//! ```

use super::{MarkedEntry, edits_the_status_hides, entry_of, is_marked, parse_marked_entries, what_the_worktree_holds};
use crate::command_test_support::{git, repository_with_tracked_file};
use crate::repo_read::repository_facts::{GitFacts, git_facts};
use crate::test_support::{REAL_GIT, isolated_overlay, remove};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// A provider for real Git after `-C <repository>`,
///  which is the same Git the wrapper runs a command against.
fn facts_in(repository: &Path) -> GitFacts {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repository.as_os_str().to_os_string()];
    return git_facts(Path::new(REAL_GIT), prefix.as_slice(), isolated_overlay().as_slice());
}

/// One entry's own bytes, as `git ls-files -v --stage -z` writes them:
///  the tag, the mode, the object, the stage, a tab and the path.
fn answer_of(entries: &[(&str, &str, &[u8])]) -> Vec<u8> {
    let mut answer: Vec<u8> = Vec::<u8>::new();
    for (tag, object, path) in entries {
        answer.extend_from_slice(format!("{tag} 100644 {object} 0\t").as_bytes());
        answer.extend_from_slice(path);
        answer.push(0);
    }
    return answer;
}

/// One entry is read with the mode the index records, the object it holds and its own path.
#[test]
fn an_entry_is_read_with_its_mode_object_and_path() {
    let answer: Vec<u8> = answer_of(&[("h", "1111111111111111111111111111111111111111", b"a.txt")]);
    let entries: Vec<MarkedEntry> = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].object, "1111111111111111111111111111111111111111");
    assert_eq!(entries[0].path, b"a.txt".to_vec());
}

/// Two entries are both read, in the order Git printed them,
/// which is the order the batch of objects comes back in.
#[test]
fn two_entries_are_both_read() {
    let answer: Vec<u8> = answer_of(&[
        ("h", "1111111111111111111111111111111111111111", b"a.txt"),
        ("h", "2222222222222222222222222222222222222222", b"dir/b.txt"),
    ]);
    let entries: Vec<MarkedEntry> = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].path, b"dir/b.txt".to_vec());
    assert_eq!(entries[1].object, "2222222222222222222222222222222222222222");
}

/// A path that holds a newline is one entry, because Git ends an entry with a NUL and not a line end.
#[test]
fn a_path_that_holds_a_newline_is_read_whole() {
    let answer: Vec<u8> = answer_of(&[("h", "1111111111111111111111111111111111111111", b"a\nb.txt")]);
    let entries: Vec<MarkedEntry> = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, b"a\nb.txt".to_vec());
}

/// A chunk that names no path is no entry, and one that is not at stage zero is no entry either.
#[test]
fn a_chunk_that_is_no_entry_names_nothing() {
    assert!(entry_of(b"not an entry at all").is_none(), "a chunk with no path names nothing");
    assert!(
        entry_of(b"h 100644 1111111111111111111111111111111111111111 2\ta.txt").is_none(),
        "an unmerged entry is not the index's own record of the path"
    );
}

/// The lower-case tag `git ls-files -v` prints is a mark,
///  and `S` is the `skip-worktree` mark only when the caller asks for it.
#[test]
fn the_assume_unchanged_tag_is_a_mark_and_skip_worktree_only_when_asked() {
    assert!(is_marked(b"h", false), "a lower-case tag is a mark");
    assert!(is_marked(b"s", true), "`s` is the skip-worktree mark");
    assert!(!is_marked(b"H", false), "an upper-case tag is no mark");
    assert!(!is_marked(b"S", false), "the skip-worktree tag is not asked for here");
}

/// What:
///  A marked edit is named,
///  an unmarked one is named by the status instead,
///  and a touched file whose bytes are unchanged is no edit.
/// Why:
///  The comparison is by bytes, so a `touch` of an unchanged file must not be read as an edit,
///  while any difference in the bytes must be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// editsTheStatusHides(facts, repo) === ['tracked.txt']; // after writing other bytes
/// editsTheStatusHides(facts, repo) === [];              // after putting the committed bytes back
/// ```
#[test]
fn a_marked_edit_is_named_and_an_unmarked_one_is_not() {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-edits-the-status-hides");
    std::fs::write(repository.join("tracked.txt"), b"the person's edit\n").expect("the edit");
    git(repository.as_path(), &["update-index", "--assume-unchanged", "tracked.txt"]);
    // The status the guard asks is empty, so the edit would be lost without this reading.
    let status = crate::command_test_support::git_status(repository.as_path(), &["status", "--porcelain=v1", "--untracked-files=no"]);
    assert!(status.stdout.is_empty(), "the mark must hide the edit from the status: {status:?}");
    let mut marked_facts: GitFacts = facts_in(repository.as_path());
    assert_eq!(
        edits_the_status_hides(&mut marked_facts, repository.as_path(), false),
        Ok(vec![String::from("tracked.txt")])
    );
    // A marked path that still holds the index's bytes is no edit, so nothing is named.
    std::fs::write(repository.join("tracked.txt"), b"one\n").expect("the committed bytes");
    let mut unchanged_facts: GitFacts = facts_in(repository.as_path());
    assert_eq!(
        edits_the_status_hides(&mut unchanged_facts, repository.as_path(), false),
        Ok(Vec::<String>::new())
    );
    // A touched path whose bytes are unchanged is no edit either: the bytes decide, not the clock.
    std::fs::write(repository.join("tracked.txt"), b"one\n").expect("the committed bytes again");
    let mut touched_facts: GitFacts = facts_in(repository.as_path());
    assert_eq!(
        edits_the_status_hides(&mut touched_facts, repository.as_path(), false),
        Ok(Vec::<String>::new())
    );
    remove(root.as_path());
}

/// What:
///  A same-length edit with its modification time put back is named,
///  because the bytes are compared and not the stat.
/// Why:
///  Equal size, nanosecond modification time and mode do not make equal content:
///  the seventh review replaced `base\n` with `evil\n`, restored the time, saw an empty status, and a
///  wrapped `git reset --hard` replaced the edit with no refusal
///  (measured on this host with native Git 2.56.0, 2026-11-15).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = statSync(file).mtimeNs; writeFileSync(file, 'tw0\n'); restoreMtime(file, before);
/// editsTheStatusHides(facts, repo) === ['tracked.txt'];
/// ```
#[test]
fn a_same_length_edit_with_its_time_put_back_is_named() {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-edits-equal-metadata");
    let file: PathBuf = repository.join("tracked.txt");
    let before = std::fs::metadata(file.as_path()).expect("the committed file");
    assert_eq!(before.len(), 4, "the fixture's file must be as long as the edit that follows");
    git(repository.as_path(), &["update-index", "--assume-unchanged", "tracked.txt"]);
    std::fs::write(file.as_path(), b"tw0\n").expect("the same-length edit");
    // Put the modification time back, nanoseconds included, so only the bytes differ.
    let kept: std::fs::File = std::fs::OpenOptions::new().write(true).open(file.as_path()).expect("the edited file");
    kept.set_times(std::fs::FileTimes::new().set_modified(before.modified().expect("the time Git kept")))
        .expect("the time is put back");
    let after = std::fs::metadata(file.as_path()).expect("the edited file");
    assert_eq!(after.len(), before.len(), "the edit is the same length");
    assert_eq!(
        after.modified().expect("the time"),
        before.modified().expect("the time"),
        "the edit keeps the committed file's time"
    );
    // The status is empty for this edit, which is what the mark is for.
    let status = crate::command_test_support::git_status(repository.as_path(), &["status", "--porcelain=v1", "--untracked-files=no"]);
    assert!(status.stdout.is_empty(), "the mark hides this edit from the status: {status:?}");
    let mut facts: GitFacts = facts_in(repository.as_path());
    assert_eq!(
        edits_the_status_hides(&mut facts, repository.as_path(), false),
        Ok(vec![String::from("tracked.txt")]),
        "the bytes differ, so the edit the mark hides is named"
    );
    remove(root.as_path());
}

/// What:
///  Two marked edits are both named, in one comparison,
///  which is what the batch hands back in the order it asked.
/// Why:
///  The objects of a batch come back one per name, in order;
///  an answer read against the wrong entry would name one edit and hide another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// editsTheStatusHides(facts, repo) === ['a.txt', 'b.txt'];
/// ```
#[test]
fn two_marked_edits_are_compared_in_one_batch() {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-edits-in-one-batch");
    std::fs::write(repository.join("a.txt"), b"a committed\n").expect("a");
    std::fs::write(repository.join("b.txt"), b"b committed\n").expect("b");
    git(repository.as_path(), &["add", "--", "a.txt", "b.txt"]);
    git(repository.as_path(), &["commit", "--quiet", "--message=two", "--", "a.txt", "b.txt"]);
    git(repository.as_path(), &["update-index", "--assume-unchanged", "a.txt", "b.txt"]);
    std::fs::write(repository.join("a.txt"), b"a edited\n").expect("a edit");
    std::fs::write(repository.join("b.txt"), b"b edited\n").expect("b edit");
    let mut facts: GitFacts = facts_in(repository.as_path());
    let named: Result<Vec<String>, String> = edits_the_status_hides(&mut facts, repository.as_path(), false);
    let mut sorted: Vec<String> = named.expect("the comparison answers");
    sorted.sort();
    assert_eq!(sorted, vec![String::from("a.txt"), String::from("b.txt")]);
    remove(root.as_path());
}

/// What:
///  A marked edit above the caller is named from a subdirectory,
///  and the batch hashes the file the index names.
/// Why:
///  The listing asks from the top (`--full-name` with `:(top)`),
///  so its paths are root-relative,
///  and a plain pathspec or a plain file name is read from the directory Git runs in.
///  From `sub`, `ls-files -- tracked.txt` matched `sub/tracked.txt` or nothing,
///  so the root edit was not in the answer at all and a `reset --hard` run in `sub` reverted it
///  (measured on this host with native Git 2.56.0, 2026-11-14).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// editsTheStatusHides(factsAt('repo/sub'), 'repo') === ['tracked.txt']
/// ```
#[test]
fn a_marked_edit_above_the_caller_is_read_from_a_subdirectory() {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-edits-read-from-a-subdirectory");
    let below: PathBuf = repository.join("sub");
    std::fs::create_dir(below.as_path()).expect("the directory the command runs in");
    // A file of the same name below the caller holds the index's own bytes, so a hash of that file
    //  would report the root edit as unedited.
    std::fs::write(below.join("tracked.txt"), b"one\n").expect("the file below the caller");
    std::fs::write(repository.join("tracked.txt"), b"the person's edit\n").expect("the edit at the root");
    git(repository.as_path(), &["update-index", "--assume-unchanged", "tracked.txt"]);
    let mut facts: GitFacts = facts_in(below.as_path());
    assert_eq!(
        edits_the_status_hides(&mut facts, repository.as_path(), false),
        Ok(vec![String::from("tracked.txt")]),
        "the edit at the root was not read from the subdirectory"
    );
    // The entry the comparison reads is the one at the root.
    let entry: MarkedEntry = MarkedEntry {
        object: String::from("1111111111111111111111111111111111111111"),
        kind: super::IndexKind::File { executable: false },
        path: Vec::from(b"tracked.txt".as_slice()),
    };
    assert!(
        matches!(what_the_worktree_holds(repository.as_path(), &entry), Ok(super::Held::Regular { .. })),
        "the root file is the one the index names"
    );
    remove(root.as_path());
}

/// What:
///  A metadata lookup that fails for a reason other than "it is not there" is a failure of the
///  reading,
///  never "this marked path is unedited".
/// Why:
///  A marked path the worktree does not hold is no edit of the person's,
///  while a lookup the file system refused says nothing at all,
///  and a discarding form would overwrite an edit nobody saw.
///  Here the directory that holds the marked path cannot be searched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whatTheWorktreeHolds(facts, root, { path: 'locked/a.txt' }); // throws
/// ```
#[test]
fn a_lookup_that_fails_for_another_reason_is_a_failure_and_not_an_unchanged_path() {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-edits-lookup-refused");
    let locked: PathBuf = repository.join("locked");
    std::fs::create_dir(locked.as_path()).expect("the directory");
    std::fs::write(locked.join("a.txt"), b"the person's edit\n").expect("the edit");
    let entry: MarkedEntry = MarkedEntry {
        object: String::from("1111111111111111111111111111111111111111"),
        kind: super::IndexKind::File { executable: false },
        path: Vec::from(b"locked/a.txt".as_slice()),
    };
    // The same entry with the directory readable is read: the failure below is the refusal, not the shape.
    assert!(matches!(what_the_worktree_holds(repository.as_path(), &entry), Ok(super::Held::Regular { .. })));
    let mut refused_permissions = std::fs::Permissions::from_mode(0o000);
    std::fs::set_permissions(locked.as_path(), refused_permissions.clone()).expect("the directory is made unsearchable");
    let answer: Result<super::Held, String> = what_the_worktree_holds(repository.as_path(), &entry);
    refused_permissions = std::fs::Permissions::from_mode(0o700);
    std::fs::set_permissions(locked.as_path(), refused_permissions).expect("the directory is readable again");
    match answer {
        Err(cause) => assert!(cause.contains("cannot tell whether the edit"), "{cause}"),
        Ok(held) => panic!("a refused lookup answered {held:?}"),
    }
    remove(root.as_path());
}

/// A marked path the worktree does not hold is no edit of the person's: Git writes that file back.
#[test]
fn a_marked_path_the_worktree_does_not_hold_is_no_edit() {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-edits-file-gone");
    git(repository.as_path(), &["update-index", "--assume-unchanged", "tracked.txt"]);
    std::fs::remove_file(repository.join("tracked.txt")).expect("the file is gone");
    let mut facts: GitFacts = facts_in(repository.as_path());
    assert_eq!(
        edits_the_status_hides(&mut facts, repository.as_path(), false),
        Ok(Vec::<String>::new())
    );
    remove(root.as_path());
}
