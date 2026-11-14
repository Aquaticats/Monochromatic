//! What:
//!  The index's own record of a marked path read back from a `git ls-files -v --stage --debug -z`
//!  answer.
//! Why:
//!  This reading decides whether a marked edit is visible,
//!  and a header read from the wrong bytes compares the wrong file:
//!  the format puts the five stat lines after the entry's own NUL,
//!  so the next header follows them rather than the NUL.
//!  A chunk that is not a header must name nothing,
//!  because a path nobody checked must never be reported as unedited.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseMarkedEntries(Buffer.from('h 100644 <sha> 0\ta.txt\0  ctime: …\n  mtime: 1:2\n  …'))
//! //   === [{ mode: '100644', object: '<sha>', path: 'a.txt', size: 2, modified: [1, 2] }]
//! ```

use super::{MarkedEntry, after_stat_lines, differs_from_the_index, edits_the_status_hides, entry_of, is_marked, parse_marked_entries};
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

/// The five stat lines Git writes after one entry,
/// as this control spells them.
const STAT_BLOCK: &str = "  ctime: 1:2\n  mtime: 1700000000:123456789\n  dev: 1\tino: 2\n  uid: 3\tgid: 4\n  size: 2\tflags: 8000\n";

/// One entry's own bytes,
/// with the stat block that follows them.
fn answer_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut answer: Vec<u8> = Vec::<u8>::new();
    for (header, path) in entries {
        answer.extend_from_slice(header.as_bytes());
        answer.push(b'\t');
        answer.extend_from_slice(path);
        answer.push(0);
        answer.extend_from_slice(STAT_BLOCK.as_bytes());
    }
    return answer;
}

/// A marked entry is read with its mode,
/// its object,
/// its path,
/// the size the index recorded and the modification time it kept.
#[test]
fn a_marked_entry_is_read_with_its_own_stat() {
    let answer: Vec<u8> = answer_of(&[("h 100644 1111111111111111111111111111111111111111 0", b"a.txt")]);
    let entries = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].mode, "100644");
    assert_eq!(entries[0].object, "1111111111111111111111111111111111111111");
    assert_eq!(entries[0].path, b"a.txt".to_vec());
    assert_eq!(entries[0].size, 2);
    assert_eq!(entries[0].modified, (1700000000, 123456789));
}

/// Two entries are both read,
/// and the second one's header is found after the first one's stat lines.
#[test]
fn the_second_entry_is_read_after_the_first_stat_block() {
    let answer: Vec<u8> = answer_of(&[
        ("h 100644 1111111111111111111111111111111111111111 0", b"a.txt"),
        ("h 100755 2222222222222222222222222222222222222222 0", b"dir/b.txt"),
    ]);
    let entries = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].path, b"dir/b.txt".to_vec());
    assert_eq!(entries[1].mode, "100755");
}

/// A path that holds a newline is read whole:
/// the boundary between two entries is the fifth stat line, never the path's own bytes.
#[test]
fn a_path_that_holds_a_newline_is_read_whole() {
    let answer: Vec<u8> = answer_of(&[
        ("h 100644 1111111111111111111111111111111111111111 0", b"a\nb.txt"),
        ("h 100644 2222222222222222222222222222222222222222 0", b"c.txt"),
    ]);
    let entries = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path, b"a\nb.txt".to_vec());
    assert_eq!(entries[1].path, b"c.txt".to_vec());
}

/// A chunk that is not a header names nothing,
/// and an answer that holds fewer stat lines than Git writes stops the reading.
#[test]
fn a_chunk_that_is_no_header_names_nothing() {
    assert!(entry_of(b"not a header at all", STAT_BLOCK.as_bytes()).is_none());
    assert!(entry_of(b"h 100644 1111111111111111111111111111111111111111 1\ta.txt", STAT_BLOCK.as_bytes()).is_none());
    assert!(entry_of(b"h 100644 1111111111111111111111111111111111111111 0\ta.txt", b"no stat here\n").is_none());
    assert!(after_stat_lines(b"  ctime: 1:2\n").is_none());
}

/// Only a single lower-case tag is the assume-unchanged mark,
/// an upper-case tag is an entry Git looks at,
/// and `skip-worktree` (`S`) is a mark only where the caller asks for it,
/// because Git keeps that file for a whole-tree write and writes over it for a path form.
#[test]
fn the_assume_unchanged_tag_is_a_mark_and_skip_worktree_only_when_asked() {
    assert!(is_marked(b"h", false));
    assert!(is_marked(b"h", true));
    assert!(!is_marked(b"H", true));
    assert!(!is_marked(b"S", false));
    assert!(is_marked(b"S", true));
    assert!(!is_marked(b"hh", true));
    assert!(!is_marked(b"", true));
}

/// A tracked edit whose path the index marks assume-unchanged is named,
/// and the same edit read without the mark is named by the status instead.
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
///  A metadata lookup that fails for a reason other than "it is not there" is a failure of the
///  reading,
///  never "this marked path is unedited".
/// Why:
///  The two answers mean opposite things for the guard:
///  a marked path the worktree does not hold is no edit of the person's,
///  because Git writes that file back,
///  while a lookup the file system refused says nothing at all,
///  and a discarding form would overwrite an edit nobody saw.
///  Here the directory that holds the marked path cannot be searched,
///  which is a refusal a stalled mount and a broken permission both look like.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const differs = differsFromTheIndex(facts, root, { path: 'locked/a.txt', ... }); // throws
/// ```
#[test]
fn a_lookup_that_fails_for_another_reason_is_a_failure_and_not_an_unchanged_path() {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-edits-lookup-refused");
    let locked: PathBuf = repository.join("locked");
    std::fs::create_dir(locked.as_path()).expect("the directory");
    std::fs::write(locked.join("a.txt"), b"the person's edit\n").expect("the edit");
    let entry: MarkedEntry = MarkedEntry {
        mode: String::from("100644"),
        object: String::from("1111111111111111111111111111111111111111"),
        path: Vec::from(b"locked/a.txt".as_slice()),
        size: 17,
        modified: (0, 0),
    };
    let mut facts: GitFacts = facts_in(repository.as_path());
    // The same entry with the directory readable is read: the failure below is the refusal, not the shape.
    assert!(matches!(differs_from_the_index(&mut facts, repository.as_path(), &entry), Ok(true)));
    let mut refused_permissions = std::fs::Permissions::from_mode(0o000);
    std::fs::set_permissions(locked.as_path(), refused_permissions.clone()).expect("the directory is made unsearchable");
    let mut refused_facts: GitFacts = facts_in(repository.as_path());
    let answer: Result<bool, String> = differs_from_the_index(&mut refused_facts, repository.as_path(), &entry);
    refused_permissions = std::fs::Permissions::from_mode(0o700);
    std::fs::set_permissions(locked.as_path(), refused_permissions).expect("the directory is readable again");
    match answer {
        Err(cause) => assert!(cause.contains("cannot tell whether the edit"), "{cause}"),
        Ok(unchanged) => panic!("a refused lookup was read as an unchanged path: {unchanged}"),
    }
    remove(root.as_path());
}

//region The fourth review: a path read from the wrong folder (2026-11-14)

/// What:
///  A marked edit at the worktree root is read when the command runs in a subdirectory,
///  and its bytes are hashed from the file the index names.
/// Why:
///  The listing that names the marked paths asks from the top (`--full-name` with `:(top)`),
///  so those names are root-relative,
///  while a plain pathspec is matched from the directory Git runs in and a plain file name is
///  opened from there.
///  From `sub`, `ls-files -- tracked.txt` matched `sub/tracked.txt` or nothing,
///  and `hash-object -- tracked.txt` hashed that other file,
///  so the root edit was not in the answer at all and a `reset --hard` run in `sub` reverted it
///  (measured on this host with native Git 2.56.0,
///  2026-11-14).
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
    // The bytes half on its own: the entry is the index's, and the file it names is the root's.
    let entry: MarkedEntry = MarkedEntry {
        mode: String::from("100644"),
        object: String::from("1111111111111111111111111111111111111111"),
        path: Vec::from(b"tracked.txt".as_slice()),
        size: 1,
        modified: (0, 0),
    };
    let mut bytes_facts: GitFacts = facts_in(below.as_path());
    assert!(
        matches!(differs_from_the_index(&mut bytes_facts, repository.as_path(), &entry), Ok(true)),
        "the object hashed was the file below the caller and not the one the index names"
    );
    remove(root.as_path());
}

//endregion
