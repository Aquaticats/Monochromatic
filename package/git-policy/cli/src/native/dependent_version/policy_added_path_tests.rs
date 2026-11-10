//! What:
//! Controls for the added-path precondition of a direct fix and its message.
//! Why:
//! Admitting a file that differs from `HEAD` anywhere would discard someone's change;
//!      refusing a clean one would make a narrow fix fail for no reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => assertAddablePath({ ... })).toThrow('its worktree copy has unstaged changes');
//! ```
#![cfg(unix)]

/// Import the precondition,
/// its reasons,
/// its messages and the real-index listing.
use super::{
    ADDED_PATH_HEAD_REASON, ADDED_PATH_MISSING_REASON, ADDED_PATH_STAGED_REASON,
    ADDED_PATH_UNSTAGED_REASON, CommitAddedPath, Entry, added_path_message, commit_added_path_message,
    commit_refusal, head_mode, real_index_entries, worktree_refusal,
};
/// The modes and object names entries carry.
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};
/// Shared fixture directories and real Git.
use crate::test_support::{fixture, git, git_context, remove, repository};
/// Unix permission bits.
use std::os::unix::fs::PermissionsExt;
/// Owned filesystem paths.
use std::path::{Path, PathBuf};

/// The object named by forty copies of one hexadecimal digit.
fn object(digit: u8) -> ObjectId {
    return parse_object_id(&[digit; 40]).expect("forty hexadecimal digits");
}

/// An entry of `mode` naming the object of one digit.
fn entry(mode: CandidateMode, digit: u8) -> Entry {
    return (mode, object(digit));
}

/// The message is the installed wrapper's direct-fix wording,
/// with both remedies.
#[test]
fn the_message_names_the_file_the_reason_and_both_remedies() {
    assert_eq!(
        added_path_message("package/module/b/package.json", ADDED_PATH_UNSTAGED_REASON),
        "A policy fix needs to change package/module/b/package.json, which this fix did not select, \
         but its worktree copy has unstaged changes. Include package/module/b/package.json in the fix \
         pathspecs so the fix applies to its worktree copy, or restore it to match HEAD \
         (git restore --staged --worktree -- package/module/b/package.json), then run the fix again."
    );
}

/// `HEAD` must hold the computed-against object as an ordinary file;
/// the index must hold it
/// with `HEAD`'s mode.
#[test]
fn head_and_index_must_hold_the_same_ordinary_file() {
    let regular: Entry = entry(CandidateMode::Regular, b'a');
    let executable: Entry = entry(CandidateMode::Executable, b'a');
    assert_eq!(
        head_mode(Some(&regular), &regular),
        Ok(CandidateMode::Regular)
    );
    assert_eq!(
        head_mode(Some(&executable), &executable),
        Ok(CandidateMode::Executable)
    );
    assert_eq!(head_mode(None, &regular), Err(ADDED_PATH_HEAD_REASON));
    assert_eq!(
        head_mode(Some(&entry(CandidateMode::Regular, b'b')), &regular),
        Err(ADDED_PATH_HEAD_REASON)
    );
    for mode in [CandidateMode::Symlink, CandidateMode::Gitlink] {
        assert_eq!(
            head_mode(Some(&entry(mode, b'a')), &entry(mode, b'a')),
            Err(ADDED_PATH_HEAD_REASON),
            "{mode:?}"
        );
    }
    assert_eq!(
        head_mode(Some(&regular), &executable),
        Err(ADDED_PATH_STAGED_REASON)
    );
    assert_eq!(
        head_mode(Some(&executable), &regular),
        Err(ADDED_PATH_STAGED_REASON)
    );
}

/// The worktree copy must be a plain file with `HEAD`'s executable bit and bytes.
#[test]
fn the_worktree_copy_must_match_head() {
    let root: PathBuf = fixture("added-path-worktree");
    let file: PathBuf = root.join("file.json");
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Regular, b"x\n").expect("missing"),
        Some(ADDED_PATH_MISSING_REASON)
    );
    std::fs::write(&file, b"x\n").expect("file");
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Regular, b"x\n").expect("clean"),
        None
    );
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Regular, b"y\n").expect("changed"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Executable, b"x\n").expect("bit"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).expect("executable");
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Executable, b"x\n").expect("clean"),
        None
    );
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Regular, b"x\n").expect("bit"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    let link: PathBuf = root.join("link.json");
    std::os::unix::fs::symlink("file.json", &link).expect("link");
    assert_eq!(
        worktree_refusal(link.as_path(), CandidateMode::Executable, b"x\n").expect("link"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    let directory: PathBuf = root.join("directory.json");
    std::fs::create_dir(&directory).expect("directory");
    assert_eq!(
        worktree_refusal(directory.as_path(), CandidateMode::Regular, b"x\n").expect("dir"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    // A file the system refuses to read is an error, not a verdict.
    let unreadable: PathBuf = root.join("unreadable.json");
    std::fs::write(&unreadable, b"x\n").expect("unreadable file");
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o200))
        .expect("write-only");
    if std::fs::read(&unreadable).is_err() {
        assert!(worktree_refusal(unreadable.as_path(), CandidateMode::Regular, b"x\n").is_err());
    }
    // A path below a file cannot be inspected at all.
    let beneath: PathBuf = file.join("child");
    assert!(worktree_refusal(beneath.as_path(), CandidateMode::Regular, b"x\n").is_err());
    remove(root.as_path());
}

/// The commit-lifecycle message names the dependent, both versions, the reason and both
/// remedies,
/// so the person knows which dependent stayed behind and what to do about it.
#[test]
fn the_commit_message_names_the_dependent_and_both_remedies() {
    assert_eq!(
        commit_added_path_message(
            "package/module/b/package.json",
            "@s/b",
            "2.0.0",
            "2.0.1",
            ADDED_PATH_UNSTAGED_REASON
        ),
        "A policy fix needs to add package/module/b/package.json to this commit to bump @s/b from \
         2.0.0 to 2.0.1, but its worktree copy has unstaged changes. @s/b keeps 2.0.0 in this \
         commit, and its copy was left exactly as it is. Commit the file too, with its version set \
         to 2.0.1, or restore it to match HEAD (git restore --staged --worktree -- \
         package/module/b/package.json), then commit again."
    );
}

/// A commit adds a path only while `HEAD`, its own index, the real index and the worktree copy
/// all hold the file the bump was computed against,
/// and each copy that differs is named by its own
///  reason.
#[test]
fn a_commit_adds_a_path_only_while_every_copy_agrees() {
    let root: PathBuf = fixture("added-path-commit");
    std::fs::create_dir(root.join("package")).expect("package");
    let file: PathBuf = root.join("package/package.json");
    std::fs::write(&file, b"bumped\n").expect("file");
    let head: Entry = entry(CandidateMode::Regular, b'a');
    let other: Entry = entry(CandidateMode::Regular, b'b');
    let added = |head_entry: Option<Entry>, commit: Entry, real: Option<Entry>| {
        return CommitAddedPath {
            path: b"package/package.json",
            head: head_entry,
            commit,
            real,
            original: b"bumped\n",
            root: root.as_path(),
        };
    };
    assert_eq!(
        commit_refusal(&added(Some(head.clone()), head.clone(), Some(head.clone()))).expect("the copies are read"),
        None,
        "every copy holds the file the bump was computed against"
    );
    assert_eq!(
        commit_refusal(&added(None, head.clone(), Some(head.clone()))).expect("the copies are read"),
        Some(ADDED_PATH_HEAD_REASON)
    );
    assert_eq!(
        commit_refusal(&added(Some(head.clone()), head.clone(), None)).expect("the copies are read"),
        Some(ADDED_PATH_STAGED_REASON),
        "the real index holds no entry for it"
    );
    assert_eq!(
        commit_refusal(&added(Some(head.clone()), head.clone(), Some(other.clone()))).expect("the copies are read"),
        Some(ADDED_PATH_STAGED_REASON),
        "the real index holds another entry for it"
    );
    assert_eq!(
        commit_refusal(&added(Some(other.clone()), head.clone(), Some(head.clone()))).expect("the copies are read"),
        Some(ADDED_PATH_HEAD_REASON),
        "the commit's own entry is not HEAD's"
    );
    std::fs::write(&file, b"mine\n").expect("edit");
    assert_eq!(
        commit_refusal(&added(Some(head.clone()), head.clone(), Some(head.clone()))).expect("the copies are read"),
        Some(ADDED_PATH_UNSTAGED_REASON),
        "the worktree copy holds an edit"
    );
    std::fs::remove_file(&file).expect("remove");
    assert_eq!(
        commit_refusal(&added(Some(head.clone()), head.clone(), Some(head.clone()))).expect("the copies are read"),
        Some(ADDED_PATH_MISSING_REASON),
        "the worktree has no copy"
    );
    remove(root.as_path());
}

/// The real index's stage-0 entries are read for the paths asked about in one listing,
/// a path it does not
///  hold is absent from the answer,
/// and an unmerged entry is absent too,
/// because it has no
///  single content.
#[test]
fn the_real_index_entries_are_listed_for_the_paths_asked_about() {
    let root: PathBuf = fixture("added-path-real-index");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.json"), b"{\"version\":\"1.0.0\"}\n").expect("a");
    std::fs::write(repo.join("b.json"), b"{\"version\":\"2.0.0\"}\n").expect("b");
    git(repo.as_path(), &["add", "--", "a.json", "b.json"]);
    let index: PathBuf = repo.join(".git/index");
    let asked: Vec<Vec<u8>> = [b"a.json".as_slice(), b"absent.json".as_slice(), b"b.json".as_slice()]
        .iter()
        .map(|path| return path.to_vec())
        .collect();
    let listed = real_index_entries(
        &git_context(),
        repo.as_path(),
        index.as_path(),
        asked.as_slice(),
    )
    .expect("the real index is listed");
    assert_eq!(listed.len(), 2, "{listed:?}");
    for name in ["a.json", "b.json"] {
        let entry: &Entry = listed.get(name.as_bytes()).expect("the asked path is listed");
        assert_eq!(entry.0, CandidateMode::Regular);
        assert_eq!(
            git_context_index_object(repo.as_path(), name),
            entry.1.as_str(),
            "{name} names the blob the real index holds"
        );
    }
    remove(root.as_path());
}

/// The object the real index holds for `name`,
/// asked of real Git directly,
/// as the oracle of the listing.
fn git_context_index_object(repo: &Path, name: &str) -> String {
    let output = git(repo, &["ls-files", "--stage", "--", name]);
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let mut fields = text.split(' ');
    let _mode: Option<&str> = fields.next();
    return String::from(fields.next().expect("an object name"));
}
