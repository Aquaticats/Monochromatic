//! Controls for the registration lookup of a worktree removal:
//! Git's match by last components,
//! the scope of the
//!  registration to the invoking repository,
//! and the pointers that must name each other.

use super::*;
use crate::test_support::{fixture, git, remove, repository};

/// The match that picks the worktree to remove starts at a directory boundary,
/// compares the bytes,
/// and needs a non-empty name.
/// Why:
///  Git's own comparison (`fspathcmp` in `dir.c`) folds case only when `core.ignorecase` is set,
/// so a byte comparison is the widest match that cannot pick a worktree Git would refuse.
#[test]
fn the_last_components_match_at_a_directory_boundary_byte_for_byte() {
    for (candidate, operand, expected) in [
        ("/a/b/topic", "topic", true),
        ("/a/b/topic", "b/topic", true),
        ("/a/b/topic", "/a/b/topic", true),
        ("/a/b/mytopic", "topic", false),
        ("/a/b/topic", "Topic", false),
        ("/a/b/TOPIC", "topic", false),
        ("/a/b/topic", "", false),
        ("/b/topic", "/a/b/topic", false),
        ("/a/b/topic", "topic/", false),
    ] {
        assert_eq!(ends_with_name(candidate.as_bytes(), operand.as_bytes()), expected, "{candidate} against {operand}");
    }
}

/// The guard that declines a removal matches ignoring ASCII case,
/// which is at least every name Git's own comparison can match under any `core.ignorecase` setting;
/// a match here only sends the removal to Git.
#[test]
fn the_guard_match_ignores_ascii_case() {
    for (candidate, operand, expected) in [
        ("/a/b/topic", "topic", true),
        ("/a/b/topic", "Topic", true),
        ("/a/b/TOPIC", "topic", true),
        ("/a/b/mytopic", "Topic", false),
        ("/a/b/topic", "", false),
        ("/a/b/topic", "topic/", false),
    ] {
        assert_eq!(ends_with_name_ignoring_case(candidate.as_bytes(), operand.as_bytes()), expected, "{candidate} against {operand}");
    }
}

/// The worktree an operand picks is the one Git's byte comparison picks:
/// a name that differs from the registered path only by case picks none,
/// because Git folds the case only when its own settings say so.
#[test]
fn a_name_that_differs_only_by_case_picks_no_worktree() {
    let root: PathBuf = fixture("registry-suffix-case");
    let common: PathBuf = root.join("repo/.git");
    let topic: PathBuf = root.join("topic");
    std::fs::create_dir_all(common.join("worktrees/topic")).expect("registration");
    std::fs::create_dir_all(topic.as_path()).expect("worktree");
    std::fs::write(common.join("worktrees/topic/gitdir"), format!("{}/.git\n", topic.display())).expect("gitdir");
    assert_eq!(path_git_picks_for_operand(common.as_path(), OsStr::new("TOPIC")), None, "a name that differs only by case");
    let canonical: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical");
    assert_eq!(path_git_picks_for_operand(common.as_path(), OsStr::new("topic")), Some(canonical), "the registered bytes");
    remove(root.as_path());
}

/// A linked worktree is found in the repository that registered it and in no other;
/// a pointer pair that disagrees is not
///  a registration.
#[test]
fn a_registration_belongs_to_the_invoking_repository_and_the_pointers_agree() {
    let root: PathBuf = fixture("registry-scope");
    let first: PathBuf = repository(root.as_path(), "first");
    let second: PathBuf = repository(root.as_path(), "second");
    git(first.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    let first_common: PathBuf = std::fs::canonicalize(first.join(".git")).expect("common");
    let second_common: PathBuf = std::fs::canonicalize(second.join(".git")).expect("common");
    let found: Registration = registration_of(first_common.as_path(), worktree.as_path(), worktree.as_os_str()).expect("registered");
    assert_eq!(found.id, OsString::from("wt"));
    assert_eq!(found.admin, first_common.join("worktrees/wt"));
    let none: String = registration_of(second_common.as_path(), worktree.as_path(), worktree.as_os_str()).expect_err("a repository with no linked worktree");
    assert!(none.contains("no linked worktree"), "{none}");
    git(second.as_path(), &["worktree", "add", "--quiet", "-b", "other", "../other"]);
    let refused: String = registration_of(second_common.as_path(), worktree.as_path(), worktree.as_os_str()).expect_err("another repository's worktree");
    assert!(refused.contains("not one of the registrations"), "{refused}");
    // The main checkout has no registration in its own repository.
    let main: String = registration_of(first_common.as_path(), first.as_path(), first.as_os_str()).expect_err("the main checkout");
    assert!(main.contains(".git"), "{main}");
    // A `gitdir` file that records another place disagrees with the worktree's `.git` file.
    std::fs::write(first_common.join("worktrees/wt/gitdir"), b"/elsewhere/.git\n").expect("overwrite");
    let disagreeing: String = registration_of(first_common.as_path(), worktree.as_path(), worktree.as_os_str()).expect_err("disagreeing pointers");
    assert!(disagreeing.contains("records /elsewhere/.git"), "{disagreeing}");
    remove(root.as_path());
}

/// A registration that spells the worktree through a symbolic link belongs to the same worktree,
/// so a removal named either way is served;
/// a registration that records another place is still refused.
#[test]
fn a_registration_spelled_through_a_link_belongs_to_the_same_worktree() {
    let root: PathBuf = fixture("registry-link");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common");
    // The worktree moved behind a link: a link is left at the old name,
    // so the registration still spells the old name while naming the same directory.
    let moved: PathBuf = root.join("moved");
    std::fs::rename(root.join("wt"), &moved).expect("rename the worktree");
    std::os::unix::fs::symlink(&moved, root.join("wt")).expect("leave a link at the old name");
    let typed: PathBuf = root.join("wt");
    let found: Registration = registration_of(common.as_path(), moved.as_path(), typed.as_os_str()).expect("the two spellings are one worktree");
    assert_eq!(found.id, OsString::from("wt"));
    assert_eq!(found.admin, common.join("worktrees/wt"));
    registration_of(common.as_path(), moved.as_path(), OsStr::new("moved")).expect("the name matches only the target");
    // A record that names another place is still refused.
    std::fs::write(common.join("worktrees/wt/gitdir"), b"/elsewhere/.git\n").expect("overwrite");
    let elsewhere: String = registration_of(common.as_path(), moved.as_path(), typed.as_os_str()).expect_err("another place");
    assert!(elsewhere.contains("records /elsewhere/.git"), "{elsewhere}");
    remove(root.as_path());
}

/// A second registration that ends in the operand,
/// ignoring ASCII case,
/// is another worktree to Git
/// even when it names the same directory through a symbolic link:
/// Git counts registrations,
/// not directories,
/// so a name it finds ambiguous is left to Git and never removes the directory.
#[test]
fn a_second_registration_naming_the_same_directory_is_refused() {
    let root: PathBuf = fixture("registry-alias-case");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "TOPIC", "../TOPIC"]);
    // The second worktree moves and a link is left at its old name, so both registrations name the topic directory.
    std::fs::rename(root.join("TOPIC"), root.join("upper-kept")).expect("move the second worktree");
    std::os::unix::fs::symlink(root.join("topic"), root.join("TOPIC")).expect("leave a link at the old name");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common");
    let topic: PathBuf = std::fs::canonicalize(root.join("topic")).expect("canonical");
    let refused: String = registration_of(common.as_path(), topic.as_path(), OsStr::new("topic")).expect_err("Git's suffix match finds two registrations for the name");
    assert!(refused.contains("TOPIC") && refused.contains(topic.to_str().expect("UTF-8")), "{refused}");
    remove(root.as_path());
}

/// A name that Git would match to a different registered worktree is refused with both paths named;
/// a name that matches
///  only the target is accepted.
#[test]
fn a_name_that_matches_another_registered_worktree_is_refused() {
    let root: PathBuf = fixture("registry-suffix");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "one", "../x/topic"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "two", "../y/other"]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common");
    let other: PathBuf = std::fs::canonicalize(root.join("y/other")).expect("canonical");
    let topic: PathBuf = std::fs::canonicalize(root.join("x/topic")).expect("canonical");
    let refused: String = registration_of(common.as_path(), other.as_path(), OsStr::new("topic")).expect_err("Git picks x/topic for this name");
    assert!(refused.contains(topic.to_str().expect("UTF-8")) && refused.contains(other.to_str().expect("UTF-8")), "{refused}");
    registration_of(common.as_path(), topic.as_path(), OsStr::new("topic")).expect("the name picks the target itself");
    registration_of(common.as_path(), other.as_path(), other.as_os_str()).expect("a full path matches only itself");
    remove(root.as_path());
}

/// Make a named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_a_named_pipe(path: &std::path::Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "mkfifo {}", path.display());
}

/// What:
///  A `.git` that is a named pipe is refused without waiting for a writer.
/// Why:
///   The removal reads the worktree's pointer before it proves the registration,
///  and `std::fs::read` waits inside `open(2)` for a writer of a pipe,
///  so one malformed pointer would hold a removal for ever,
///  with nothing said and no bound
///  (`SPEC.md`,
///  "Every wait has a bound").
#[test]
fn a_pointer_that_is_a_named_pipe_is_refused_without_waiting() {
    use std::io::Write as _;
    let root: PathBuf = fixture("registry-pointer-pipe");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common");
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    let pointer: PathBuf = worktree.join(".git");
    let old_pointer: Vec<u8> = std::fs::read(pointer.as_path()).expect("the old pointer");
    std::fs::remove_file(pointer.as_path()).expect("the regular pointer is removed");
    make_a_named_pipe(pointer.as_path());
    let (sender, receiver) = std::sync::mpsc::channel::<Result<Registration, String>>();
    let asked_common: PathBuf = common.clone();
    let asked_worktree: PathBuf = worktree.clone();
    let worker: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let _ = sender.send(registered_worktree(asked_common.as_path(), asked_worktree.as_path()));
    });
    let bound: std::time::Duration = std::time::Duration::from_secs(5);
    let answered: Result<Result<Registration, String>, std::sync::mpsc::RecvTimeoutError> = receiver.recv_timeout(bound);
    if answered.is_err() {
        // Release the reader so this control reports what it found instead of leaving a thread waiting for ever.
        let mut writer: std::fs::File = std::fs::OpenOptions::new().write(true).open(pointer.as_path()).expect("release the reader");
        writer.write_all(old_pointer.as_slice()).expect("write the pointer");
        drop(writer);
    }
    let found: Result<Registration, String> = match answered {
        Ok(answer) => answer,
        Err(_) => receiver.recv_timeout(bound).expect("the released reader lets the read finish"),
    };
    worker.join().expect("join the read");
    assert!(found.is_err(), "a pointer that is no regular file answered as a registration: {found:?}");
}

/// What:
///  A named pipe in a registration this removal does not delete is not waited for either.
/// Why:
///   The removal proves the target's registration,
///  and then reads every registration of the repository to ask whether Git could pick another one for the typed name.
///  A `gitdir` of one of those that is no regular file would hold that read inside `open(2)`
///  for a writer that may never come,
///  and no bound covers it:
///  the removal never reaches a verdict and never falls back to Git
///  (`SPEC.md`,
///  "Every wait has a bound").
///  A record that cannot be read is skipped, as one Git would skip,
///  so the target's registration is still the answer.
#[test]
fn another_registrations_named_pipe_does_not_block_the_removal() {
    use std::io::Write as _;
    let root: PathBuf = fixture("registry-other-pipe");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "other", "../other"]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common");
    let topic: PathBuf = std::fs::canonicalize(root.join("topic")).expect("canonical");
    let other_back_link: PathBuf = common.join("worktrees/other/gitdir");
    let old_back_link: Vec<u8> = std::fs::read(other_back_link.as_path()).expect("the old back-link");
    std::fs::remove_file(other_back_link.as_path()).expect("the regular back-link is removed");
    make_a_named_pipe(other_back_link.as_path());
    let (sender, receiver) = std::sync::mpsc::channel::<Result<Registration, String>>();
    let asked_common: PathBuf = common.clone();
    let asked_topic: PathBuf = topic.clone();
    let worker: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let _ = sender.send(registration_of(asked_common.as_path(), asked_topic.as_path(), OsStr::new("topic")));
    });
    let bound: std::time::Duration = std::time::Duration::from_secs(5);
    let answered: Result<Result<Registration, String>, std::sync::mpsc::RecvTimeoutError> = receiver.recv_timeout(bound);
    if answered.is_err() {
        // Release the reader so this control reports what it found instead of leaving a thread waiting for ever.
        let mut writer: std::fs::File = std::fs::OpenOptions::new().write(true).open(other_back_link.as_path()).expect("release the reader");
        writer.write_all(old_back_link.as_slice()).expect("write the back-link");
        drop(writer);
    }
    let answered_before_release: bool = answered.is_ok();
    let found: Result<Registration, String> = match answered {
        Ok(answer) => answer,
        Err(_) => receiver.recv_timeout(bound).expect("the released reader lets the read finish"),
    };
    worker.join().expect("join the read");
    assert!(answered_before_release, "the removal waited for another registration's named pipe");
    assert_eq!(found.expect("the target's own registration is still the answer").id, OsString::from("topic"));
    remove(root.as_path());
}
