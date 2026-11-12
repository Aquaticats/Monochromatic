//! Controls for the landing's follow-up with real Git:
//! the index entries of a commit Git built,
//! whatever its names and
//!  however many,
//! and the conclusion state removal.

use super::*;
use crate::recording_git::{recorded_starts, recording_context};
use crate::repo_read_hook::SWITCH_VARIABLE;
use crate::index_reset_file::support::git_with;
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};
use crate::transaction_git::GitOutput;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// A repository whose index is at the first commit while the branch is at a second commit that adds the given files.
pub(super) fn index_behind(name: &str, files: &[Vec<u8>]) -> (PathBuf, PathBuf, String, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    for file in files {
        let path: PathBuf = repo.join(std::ffi::OsStr::from_bytes(file.as_slice()));
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
        std::fs::write(path, b"x\n").expect("write");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let second: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["read-tree", first.as_str()]);
    return (root, repo, first, second);
}


/// What the index still differs from the commit by.
pub(super) fn differs(repo: &Path, commit: &str) -> bool {
    return !git_output(repo, &["diff", "--cached", "--quiet", commit]).status.success();
}

/// The paths that differ between the old branch and the commit are updated,
/// as bytes,
/// whatever their number.
#[test]
fn derived_paths_cover_names_that_are_not_text_and_lists_past_the_request_limit() {
    let mut files: Vec<Vec<u8>> = vec![b"not-utf8-\xff\xfe.txt".to_vec(), b"back\\slash \"quoted\".txt".to_vec(), b"line\nbreak.txt".to_vec()];
    files.extend((0..crate::service_protocol::MAX_CHANGES + 50).map(|number| { return format!("many/file-{number}.txt").into_bytes(); }));
    let (root, repo, first, second) = index_behind("service-follow-derived", files.as_slice());
    assert!(differs(repo.as_path(), second.as_str()), "the index starts behind the commit");
    update_index(&git_context(), repo.as_path(), repo.join(".git/index").as_path(), second.as_str(), IndexPaths::Derived { old: first.as_str() }).expect("updated");
    assert!(!differs(repo.as_path(), second.as_str()));
    remove(root.as_path());
}

/// A branch that did not exist has an all-zero old value,
/// and the commit's whole tree is what the index takes.
#[test]
fn a_root_commit_is_updated_from_the_all_zero_old_value() {
    let (root, repo, first, _second) = index_behind("service-follow-root", &[b"new.txt".to_vec()]);
    git(repo.as_path(), &["read-tree", "--empty"]);
    let zero: String = "0".repeat(first.len());
    update_index(&git_context(), repo.as_path(), repo.join(".git/index").as_path(), first.as_str(), IndexPaths::Derived { old: zero.as_str() }).expect("updated");
    assert!(!differs(repo.as_path(), first.as_str()));
    remove(root.as_path());
}

/// An empty list changes nothing,
/// because `reset` with no path would reset the whole index.
#[test]
fn an_empty_list_leaves_the_index_alone() {
    let (root, repo, first, second) = index_behind("service-follow-empty", &[b"new.txt".to_vec()]);
    update_index(&git_context(), repo.as_path(), repo.join(".git/index").as_path(), second.as_str(), IndexPaths::Listed(&[])).expect("nothing to do");
    assert!(differs(repo.as_path(), second.as_str()));
    update_index(&git_context(), repo.as_path(), repo.join(".git/index").as_path(), second.as_str(), IndexPaths::Derived { old: second.as_str() }).expect("no difference");
    assert!(differs(repo.as_path(), second.as_str()), "{first}");
    remove(root.as_path());
}

/// A holder proven alive that keeps the lock for longer than every try of the old five put together is waited for:
/// the index is
///  updated once it lets go,
/// and nothing failed.
#[test]
fn a_holder_proven_alive_is_waited_for_past_all_the_tries() {
    let (root, repo, _first, second) = index_behind("service-follow-proven-holder", &[b"new.txt".to_vec()]);
    let index: PathBuf = repo.join(".git/index");
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock, b"").expect("a lock");
    // The PID file names this process, which runs and started before the lock was made.
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {}\n", std::process::id())).expect("PID file");
    let holding: std::time::Duration = INDEX_PAUSE * u32::try_from(INDEX_ATTEMPTS * 3).expect("a small count");
    let holder = std::thread::spawn(move || {
        std::thread::sleep(holding);
        std::fs::remove_file(&lock).expect("the holder lets go");
    });
    let began: std::time::Instant = std::time::Instant::now();
    update_index(&git_context(), repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Named(&[b"new.txt".to_vec()])).expect("updated once the holder let go");
    holder.join().expect("holder");
    assert!(began.elapsed() >= holding, "{:?}", began.elapsed());
    assert!(!differs(repo.as_path(), second.as_str()));
    remove(root.as_path());
}

/// A lock nothing proves anyone holds is waited for like one proven alive:
/// the index is updated once the lock goes,
/// however much
///  longer than the old backoff that is.
#[test]
fn an_owner_nothing_proves_alive_is_waited_for_like_a_proven_one() {
    let (root, repo, _first, second) = index_behind("service-follow-unproven-unbounded", &[b"new.txt".to_vec()]);
    let index: PathBuf = repo.join(".git/index");
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock, b"").expect("a lock nobody holds");
    let holding: std::time::Duration = std::time::Duration::from_millis(2500);
    let holder = std::thread::spawn(move || {
        std::thread::sleep(holding);
        std::fs::remove_file(&lock).expect("the lock goes");
    });
    let began: std::time::Instant = std::time::Instant::now();
    update_index(&git_context(), repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Named(&[b"new.txt".to_vec()])).expect("updated once the lock went");
    holder.join().expect("holder");
    assert!(began.elapsed() >= holding, "{:?}", began.elapsed());
    assert!(!differs(repo.as_path(), second.as_str()));
    remove(root.as_path());
}

/// The service's context when the command that first started it came from a shell that turned pathspec magic off,
/// which every Git the service then starts inherits.
fn inherited_literal_pathspec_context() -> GitContext {
    let mut context = git_context();
    context.overlay.push((std::ffi::OsString::from("GIT_LITERAL_PATHSPECS"), std::ffi::OsString::from("1")));
    return context;
}

/// The follow-up's `reset` reads its own `:(top,literal)` pathspecs whatever pathspec mode the service inherited,
/// so a listed path's index entry is still installed when that environment turned pathspec magic off.
#[test]
fn the_index_update_of_listed_paths_survives_an_inherited_literal_pathspec_mode() {
    let (root, repo, _first, second) = index_behind("service-follow-listed-literal-mode", &[b"new.txt".to_vec()]);
    let index: PathBuf = repo.join(".git/index");
    update_index(&inherited_literal_pathspec_context(), repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Listed(&[String::from("new.txt")])).expect("the listed path is installed under the inherited mode");
    assert!(!differs(repo.as_path(), second.as_str()), "the commit's index entry must be installed");
    remove(root.as_path());
}

/// The same for a list of names handed over as bytes,
/// which `reset` reads from its standard input.
#[test]
fn the_index_update_of_named_paths_survives_an_inherited_literal_pathspec_mode() {
    let (root, repo, _first, second) = index_behind("service-follow-named-literal-mode", &[b"new.txt".to_vec()]);
    let index: PathBuf = repo.join(".git/index");
    update_index(&inherited_literal_pathspec_context(), repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Named(&[b"new.txt".to_vec()])).expect("the named path is installed under the inherited mode");
    assert!(!differs(repo.as_path(), second.as_str()), "the commit's index entry must be installed");
    remove(root.as_path());
}

/// The same for the paths derived from the branch and the commit,
/// which take the same standard-input route.
#[test]
fn the_index_update_of_derived_paths_survives_an_inherited_literal_pathspec_mode() {
    let (root, repo, first, second) = index_behind("service-follow-derived-literal-mode", &[b"new.txt".to_vec()]);
    let index: PathBuf = repo.join(".git/index");
    update_index(&inherited_literal_pathspec_context(), repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Derived { old: first.as_str() }).expect("the derived paths are installed under the inherited mode");
    assert!(!differs(repo.as_path(), second.as_str()), "the commit's index entries must be installed");
    remove(root.as_path());
}

/// A repository holding two names that differ only in case,
/// where the commit asked about changes only `a.txt`,
/// the index and the worktree stand at the commit before it,
/// and `A.txt` holds staged contents that exist in the index and the object store alone.
fn index_with_a_staged_case_twin(name: &str) -> (PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("A.txt"), b"upper\n").expect("write A.txt");
    std::fs::write(repo.join("a.txt"), b"lower\n").expect("write a.txt");
    git(repo.as_path(), &["add", "--", "A.txt", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base", "--", "A.txt", "a.txt"]);
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("a.txt"), b"lower changed\n").expect("change a.txt");
    git(repo.as_path(), &["commit", "--quiet", "--message=change", "--", "a.txt"]);
    let second: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    // `-u` has no long form: it is the only spelling that also updates the worktree.
    git(repo.as_path(), &["read-tree", "--reset", "-u", first.as_str()]);
    std::fs::write(repo.join("A.txt"), b"staged only\n").expect("stage other contents");
    git(repo.as_path(), &["add", "--", "A.txt"]);
    std::fs::write(repo.join("A.txt"), b"worktree again\n").expect("overwrite the worktree copy");
    return (root, repo, second);
}

/// The contents the index holds for one path.
fn staged_contents(repo: &Path, path: &str) -> String {
    return git_text(repo, &["show", format!(":{path}").as_str()]);
}

/// A pathspec mode that turns magic off and leaves matching case-insensitive,
/// as a shell holding both variables hands the service.
fn inherited_literal_and_case_insensitive_pathspec_context() -> GitContext {
    let mut context = git_context();
    context.overlay.push((std::ffi::OsString::from("GIT_LITERAL_PATHSPECS"), std::ffi::OsString::from("1")));
    context.overlay.push((std::ffi::OsString::from("GIT_ICASE_PATHSPECS"), std::ffi::OsString::from("1")));
    return context;
}

/// A case-insensitive reading of the module's own `:(top,literal)a.txt` must not reach `A.txt`:
/// the index entry the batch selected takes the commit's version,
/// and the staged entry it never selected stays as it was,
/// whether the paths come as text,
/// as bytes,
/// or from the branch the landing replaced.
#[test]
fn the_index_update_of_listed_paths_leaves_an_unselected_case_twin_alone() {
    let (root, repo, second) = index_with_a_staged_case_twin("service-follow-listed-icase-mode");
    let index: PathBuf = repo.join(".git/index");
    update_index(&inherited_literal_and_case_insensitive_pathspec_context(), repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Listed(&[String::from("a.txt")])).expect("the listed path is installed");
    assert_eq!(staged_contents(repo.as_path(), "a.txt"), "lower changed", "the selected path takes the commit's entry");
    assert_eq!(staged_contents(repo.as_path(), "A.txt"), "staged only", "the staged entry the batch never selected stays");
    remove(root.as_path());
}

/// The same for a list of names handed over as bytes,
/// which `reset` reads from its standard input.
#[test]
fn the_index_update_of_named_paths_leaves_an_unselected_case_twin_alone() {
    let (root, repo, second) = index_with_a_staged_case_twin("service-follow-named-icase-mode");
    let index: PathBuf = repo.join(".git/index");
    update_index(&inherited_literal_and_case_insensitive_pathspec_context(), repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Named(&[b"a.txt".to_vec()])).expect("the named path is installed");
    assert_eq!(staged_contents(repo.as_path(), "a.txt"), "lower changed", "the selected path takes the commit's entry");
    assert_eq!(staged_contents(repo.as_path(), "A.txt"), "staged only", "the staged entry the batch never selected stays");
    remove(root.as_path());
}

/// The same for the paths derived from the branch and the commit,
/// which take the same standard-input route.
#[test]
fn the_index_update_of_derived_paths_leaves_an_unselected_case_twin_alone() {
    let (root, repo, second) = index_with_a_staged_case_twin("service-follow-derived-icase-mode");
    let index: PathBuf = repo.join(".git/index");
    let old: String = git_text(repo.as_path(), &["rev-parse", format!("{second}^").as_str()]);
    update_index(&inherited_literal_and_case_insensitive_pathspec_context(), repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Derived { old: old.as_str() }).expect("the derived paths are installed");
    assert_eq!(staged_contents(repo.as_path(), "a.txt"), "lower changed", "the selected path takes the commit's entry");
    assert_eq!(staged_contents(repo.as_path(), "A.txt"), "staged only", "the staged entry the batch never selected stays");
    remove(root.as_path());
}

/// A pathspec mode the caller wrote as a global option reaches no reset either:
/// `--icase-pathspecs` in the prefix makes the generated `:(top,literal)a.txt` match `A.txt`,
/// and the reset then replaces the staged entry of a path the batch never selected.
#[test]
fn the_index_update_of_listed_paths_leaves_an_unselected_case_twin_alone_under_the_callers_option() {
    let (root, repo, second) = index_with_a_staged_case_twin("service-follow-listed-icase-option");
    let index: PathBuf = repo.join(".git/index");
    let mut context: GitContext = git_context();
    context.global_prefix.push(std::ffi::OsString::from("--icase-pathspecs"));
    update_index(&context, repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Listed(&[String::from("a.txt")])).expect("the listed path is installed");
    assert_eq!(staged_contents(repo.as_path(), "a.txt"), "lower changed", "the selected path takes the commit's entry");
    assert_eq!(staged_contents(repo.as_path(), "A.txt"), "staged only", "the staged entry the batch never selected stays");
    remove(root.as_path());
}

/// The same for a list of names read from standard input.
#[test]
fn the_index_update_of_named_paths_leaves_an_unselected_case_twin_alone_under_the_callers_option() {
    let (root, repo, second) = index_with_a_staged_case_twin("service-follow-named-icase-option");
    let index: PathBuf = repo.join(".git/index");
    let mut context: GitContext = git_context();
    context.global_prefix.push(std::ffi::OsString::from("--icase-pathspecs"));
    update_index(&context, repo.as_path(), index.as_path(), second.as_str(), IndexPaths::Named(&[b"a.txt".to_vec()])).expect("the named path is installed");
    assert_eq!(staged_contents(repo.as_path(), "a.txt"), "lower changed", "the selected path takes the commit's entry");
    assert_eq!(staged_contents(repo.as_path(), "A.txt"), "staged only", "the staged entry the batch never selected stays");
    remove(root.as_path());
}

/// A removal request whose scratch directory is gone and whose copy was never kept compares nothing,
/// so it removes nothing and says so:
/// answering success left the worktree's `MERGE_HEAD` behind,
/// and the next commit concluded the merge again.
#[test]
fn a_cleanup_whose_comparison_is_gone_removes_nothing_and_answers_so() {
    let root: PathBuf = fixture("service-follow-gone");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let cleanup: Cleanup = Cleanup { git_dir: repo.join(".git"), scratch: repo.join(".git").join("gone"), reftable: false };
    std::fs::write(repo.join(".git").join("MERGE_HEAD"), "kept\n").expect("state");
    assert!(!remove_conclusion_state(&git_context(), repo.as_path(), &cleanup, None), "nothing can be compared, so nothing is done");
    assert_eq!(std::fs::read_to_string(repo.join(".git").join("MERGE_HEAD")).expect("read"), "kept\n", "the state is not touched");
    remove(root.as_path());
}

/// What:
///  A worktree with the state a concluded merge leaves,
/// the copies a preparation makes of it in the client's scratch directory,
/// and the copy the
///  landing keeps in its journal.
/// Why:
///   The finding is about the instant between the two:
/// the client is gone and its scratch directory with it,
/// and the
///  state it would have removed is still there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, cleanup, kept } = concludedMergeWithTheScratchGone('service-follow-kept');
/// ```
fn concluded_merge_with_the_scratch_gone(name: &str) -> (PathBuf, PathBuf, Cleanup, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    std::fs::write(git_dir.join("MERGE_HEAD"), "0123456789012345678901234567890123456789\n").expect("state");
    std::fs::write(git_dir.join("MERGE_MSG"), "Merge branch 'other'\n").expect("state");
    let scratch: PathBuf = root.join("scratch");
    let copies: PathBuf = scratch.join(crate::conclusion_cleanup::CONCLUSION_COPY_DIRECTORY);
    std::fs::create_dir_all(copies.as_path()).expect("copies");
    std::fs::create_dir_all(scratch.join("shadow").as_path()).expect("shadow");
    for state in ["MERGE_HEAD", "MERGE_MSG"] {
        std::fs::copy(git_dir.join(state), copies.join(state)).expect("copy");
    }
    // The record of the pseudorefs the store holds, which the files backend holds none of.
    std::fs::write(copies.join(crate::conclusion_cleanup::STORE_RECORD_FILENAME), "{}\n").expect("store record");
    // The shadow repository removed them as native Git's own commit does, which the copy keeps by holding none.
    let cleanup: Cleanup = Cleanup { git_dir, scratch: scratch.clone(), reftable: false };
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("journal");
    let kept: PathBuf = crate::conclusion_state_copy::keep(&cleanup, journal.as_path(), 7, repo.as_path()).expect("the landing keeps what the cleanup compares");
    std::fs::remove_dir_all(scratch.as_path()).expect("the service removes the client's scratch directory");
    return (root, repo, cleanup, kept);
}

/// A copy of the comparison that cannot be made publishes nothing:
/// the name the cleanup reads is never taken by a partial copy,
/// so a landing that could not keep everything is one that kept nothing.
#[test]
fn a_copy_of_the_conclusion_state_that_fails_half_way_publishes_nothing() {
    let root: PathBuf = fixture("service-follow-partial-copy");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let scratch: PathBuf = root.join("scratch");
    let copies: PathBuf = scratch.join(crate::conclusion_cleanup::CONCLUSION_COPY_DIRECTORY);
    std::fs::create_dir_all(copies.as_path()).expect("copies");
    std::fs::write(copies.join("MERGE_HEAD"), "0123456789012345678901234567890123456789\n").expect("copy");
    std::fs::write(copies.join(crate::conclusion_cleanup::STORE_RECORD_FILENAME), "{}\n").expect("store record");
    // The shadow's entry cannot be copied: a directory stands where its file belongs.
    std::fs::create_dir_all(scratch.join("shadow").join("MERGE_HEAD")).expect("a directory in the way");
    let cleanup: Cleanup = Cleanup { git_dir, scratch, reftable: false };
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("journal");
    assert!(crate::conclusion_state_copy::keep(&cleanup, journal.as_path(), 9, repo.as_path()).is_none(), "a copy that cannot be made keeps nothing");
    assert!(!crate::conclusion_state_copy::path_of(&journal, 9).exists(), "the name the cleanup reads is never taken by a partial copy");
    assert!(!journal.join("9.cleanup.tmp").exists(), "the staging copy goes with the failure");
    remove(root.as_path());
}

/// A cleanup that failed once is finished after the client's scratch directory is gone:
/// the copy the landing kept in its journal is what the comparison reads,
/// so the concluded merge's state goes and the next
///  commit does not conclude the merge again.
#[test]
fn a_conclusions_state_goes_when_the_clients_scratch_directory_is_gone_and_its_copy_was_kept() {
    let (root, repo, cleanup, kept) = concluded_merge_with_the_scratch_gone("service-follow-kept");
    assert!(remove_conclusion_state(&git_context(), repo.as_path(), &cleanup, Some(kept.as_path())), "the state the commit concluded is removed");
    assert!(!repo.join(".git").join("MERGE_HEAD").exists(), "the merge is not in progress any more");
    assert!(!repo.join(".git").join("MERGE_MSG").exists(), "the concluded merge's message goes with it");
    remove(root.as_path());
}

/// What:
///  Every start of the index update,
/// in each form it takes,
/// as the service makes it.
/// Why:
///   The owner's rule is that no Git start of the service gets `--quiet` or `-q`:
/// the three forms of the update (a listed
///  path,
/// a named path as bytes,
/// a path derived from the old branch) each start `reset`,
/// and each is looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const paths of [listed, named, derived]) await updateIndex(recordingGit, repo, second, paths); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_index_update_starts_reset_without_quiet_in_every_form() {
    let (root, repo, first, second) = index_behind("service-follow-no-quiet", &[b"new.txt".to_vec()]);
    let (mut context, log) = crate::recording_git::recording_context(root.as_path());
    // The named form hands its paths on standard input, which a request makes in the common directory.
    context.common_dir = Some(repo.join(".git"));
    let listed: Vec<String> = vec![String::from("new.txt")];
    let named: Vec<Vec<u8>> = vec![b"new.txt".to_vec()];
    for paths in [IndexPaths::Listed(listed.as_slice()), IndexPaths::Named(named.as_slice()), IndexPaths::Derived { old: first.as_str() }] {
        git(repo.as_path(), &["read-tree", first.as_str()]);
        update_index(&context, repo.as_path(), repo.join(".git/index").as_path(), second.as_str(), paths).expect("updated");
        assert!(!differs(repo.as_path(), second.as_str()), "{paths:?} brings the index to the commit");
    }
    let resets: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.starts_with("reset "); }).collect();
    assert_eq!(resets.len(), 3, "one guarded reset for each of the three forms: {resets:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(root.as_path());
}

/// What `reset` says while the index is updated (the list of files still changed in the worktree) is kept in the service's log.
#[test]
fn what_reset_says_is_kept_in_the_journal() {
    let (root, repo, _first, second) = index_behind("service-follow-words", &[b"new.txt".to_vec()]);
    let shim: PathBuf = root.join("saying-git");
    let words: &str = "Unstaged changes after reset:";
    let script: String = format!("#!/bin/sh\ncase \" $* \" in *\" reset \"*) printf '%s\\n' '{words}';; esac\nexec {} \"$@\"\n", crate::test_support::REAL_GIT);
    crate::test_support::executable(shim.as_path(), script.as_bytes());
    let mut context: GitContext = git_context();
    context.real_git = shim;
    let listed: Vec<String> = vec![String::from("new.txt")];
    let entries = crate::service_timing::capture::entries_of(|| {
        update_index(&context, repo.as_path(), repo.join(".git/index").as_path(), second.as_str(), IndexPaths::Listed(listed.as_slice())).expect("updated");
    });
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert!(entries[0].message.contains(words), "{:?}", entries[0]);
    remove(root.as_path());
}

/// The landing's index update starts no Git `reset` and no Git `diff-tree`,
/// whichever way the paths are given,
/// and the index it
/// builds equals the commit for those paths.
#[test]
fn the_follow_up_starts_no_git_reset_and_no_git_diff_tree() {
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec(), b"dir/inner.txt".to_vec()];
    let listed: Vec<String> = vec![String::from("top.txt"), String::from("dir/inner.txt")];
    for form in ["listed", "named", "derived"] {
        let (root, repo, first, second) = index_behind(format!("service-follow-starts-{form}").as_str(), files.as_slice());
        let (mut context, log) = recording_context(root.as_path());
        // The recorder switches the in-process readers off; the update under test must run with them on.
        context.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
        let paths: IndexPaths<'_> = match form {
            "listed" => IndexPaths::Listed(listed.as_slice()),
            "named" => IndexPaths::Named(files.as_slice()),
            _ => IndexPaths::Derived { old: first.as_str() },
        };
        let index: PathBuf = repo.join(".git").join("index");
        // The service builds the index while it holds the live index lock.
        let held = crate::service_index_lock::try_acquire(index.as_path(), 1).expect("the lock is free");
        let built = build_index(&context, repo.as_path(), second.as_str(), paths, &held, 1).expect("built");
        let words: Vec<String> = recorded_starts(log.as_path()).iter().map(|start| { return start.split(' ').next().unwrap_or_default().to_string(); }).collect();
        assert!(!words.iter().any(|word| { return word == "reset" || word == "diff-tree"; }), "{form} started {words:?}");
        held.release_with(built.path()).expect("installed");
        assert!(!differs(repo.as_path(), second.as_str()), "{form} left the index behind the commit");
        remove(root.as_path());
    }
}


/// The names Git lists as staged against `commit`.
fn staged_against(repo: &Path, commit: &str) -> Vec<String> {
    return git_text(repo, &["diff", "--cached", "--name-only", commit]).lines().map(str::to_string).collect();
}

/// A change another session staged before the lock was taken,
/// to a path the commit does not name,
/// survives the in-process
/// build and its installation;
/// a stage attempted after the lock was taken is refused until the installation;
/// and no Git starts.
#[test]
fn what_was_staged_before_the_lock_survives_and_a_stage_after_it_is_refused() {
    use crate::service_index_lock::{lock_path_of, try_acquire};
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec(), b"dir/inner.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-staged", files.as_slice());
    std::fs::write(repo.join("unselected.txt"), b"staged by another session\n").expect("write");
    // An entry recorded in the second of the lock's copy is racily clean and sends the update to Git; this one is older.
    crate::test_support::age(repo.join("unselected.txt").as_path());
    git(repo.as_path(), &["add", "unselected.txt"]);
    let (mut context, log) = recording_context(root.as_path());
    context.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
    context.common_dir = Some(repo.join(".git"));
    let index: PathBuf = repo.join(".git").join("index");
    let held = try_acquire(index.as_path(), 3).expect("the lock is free");
    std::fs::write(repo.join("late.txt"), b"staged too late\n").expect("write");
    assert!(!git_output(repo.as_path(), &["add", "late.txt"]).status.success(), "a stage after the lock is taken is refused");
    let built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 3).expect("built");
    assert!(lock_path_of(index.as_path()).exists(), "the lock is held until the installation");
    held.release_with(built.path()).expect("installed");
    let words: Vec<String> = recorded_starts(log.as_path()).iter().map(|start| { return start.split(' ').next().unwrap_or_default().to_string(); }).collect();
    assert!(!words.iter().any(|word| { return word == "reset"; }), "no Git reset started: {words:?}");
    assert_eq!(staged_against(repo.as_path(), second.as_str()), vec![String::from("unselected.txt")], "only the earlier stage is left, and it survived");
    assert!(git_output(repo.as_path(), &["add", "late.txt"]).status.success(), "the lock is gone, so the stage works now");
    remove(root.as_path());
}

/// An installation that cannot be made leaves the old index and removes the lock;
/// a lock another program replaced keeps the
/// index and stays with that program.
#[test]
fn a_failed_installation_keeps_the_index_and_a_replaced_lock_is_left_alone() {
    use crate::service_index_lock::{lock_path_of, try_acquire};
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-install", files.as_slice());
    let mut context = git_context();
    context.common_dir = Some(repo.join(".git"));
    let index: PathBuf = repo.join(".git").join("index");
    let before: Vec<u8> = std::fs::read(&index).expect("index");
    // A lock replaced by another program's: the check sees it, the index is not replaced, the other program's lock stays.
    let held = try_acquire(index.as_path(), 4).expect("the lock is free");
    let built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 4).expect("built");
    std::fs::remove_file(lock_path_of(index.as_path())).expect("removed");
    std::fs::write(lock_path_of(index.as_path()), b"another program's lock").expect("foreign lock");
    assert!(held.release_with(built.path()).is_err());
    assert_eq!(std::fs::read(&index).expect("index"), before, "the index was not replaced");
    assert_eq!(std::fs::read(lock_path_of(index.as_path())).expect("lock"), b"another program's lock".to_vec());
    std::fs::remove_file(lock_path_of(index.as_path())).expect("removed");
    std::fs::remove_file(&built).expect("built removed");
    // A rename that fails: the live index path holds a directory, so the copy cannot take its place; the lock goes with the error.
    let second_hold = try_acquire(index.as_path(), 5).expect("the lock is free");
    let second_built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &second_hold, 5).expect("built");
    std::fs::remove_file(&index).expect("index removed");
    std::fs::create_dir_all(index.join("occupied")).expect("a directory where the index goes");
    assert!(second_hold.release_with(second_built.path()).is_err());
    assert!(!lock_path_of(index.as_path()).exists(), "the lock is removed with the failure");
    assert!(index.join("occupied").is_dir());
    remove(root.as_path());
}

/// An entry that is racily clean for the lock's copy (its modification time is not before the copy's) sends the follow-up to
/// Git's own `reset --no-refresh` on the private copy,
/// which gives the same result:
/// the committed paths at the commit and the
/// racily clean path still staged.
#[test]
fn a_racily_clean_entry_sends_the_follow_up_to_git_with_the_same_result() {
    use crate::service_index_lock::try_acquire;
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-racy", files.as_slice());
    let racy: PathBuf = repo.join("racy.txt");
    std::fs::write(&racy, b"staged an instant ago\n").expect("write");
    // A time an hour ahead is never before the copy's, however slow the run is.
    let ahead: std::time::SystemTime = std::time::SystemTime::now() + std::time::Duration::from_secs(3600);
    std::fs::OpenOptions::new().write(true).open(&racy).expect("open").set_modified(ahead).expect("set the time");
    git(repo.as_path(), &["add", "racy.txt"]);
    let (mut context, log) = recording_context(root.as_path());
    context.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
    context.common_dir = Some(repo.join(".git"));
    let index: PathBuf = repo.join(".git").join("index");
    let held = try_acquire(index.as_path(), 6).expect("the lock is free");
    let built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 6).expect("built");
    let starts: Vec<String> = recorded_starts(log.as_path());
    assert!(starts.iter().any(|start| { return start.starts_with("reset --no-refresh "); }), "Git was not asked: {starts:?}");
    held.release_with(built.path()).expect("installed");
    assert_eq!(staged_against(repo.as_path(), second.as_str()), vec![String::from("racy.txt")]);
    remove(root.as_path());
}

/// The record of the installed index is made after the index is installed and while the lock is still held;
/// a record that cannot
/// be made keeps the lock in place (the restart reclaims it) and the installed index,
/// and the error is returned.
#[test]
fn the_record_is_made_between_the_installation_and_the_unlock_and_a_failed_record_keeps_the_lock() {
    use crate::service_index_lock::{lock_path_of, try_acquire};
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-record", files.as_slice());
    let mut context = git_context();
    context.common_dir = Some(repo.join(".git"));
    let index: PathBuf = repo.join(".git").join("index");
    let lock: PathBuf = lock_path_of(index.as_path());
    let held = try_acquire(index.as_path(), 8).expect("the lock is free");
    let built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 8).expect("built");
    let built_bytes: Vec<u8> = std::fs::read(&built).expect("built bytes");
    let seen: std::cell::RefCell<Vec<(bool, bool)>> = std::cell::RefCell::new(Vec::new());
    let record = || -> std::io::Result<()> {
        seen.borrow_mut().push((lock.exists(), std::fs::read(&index)? == built_bytes));
        return Ok(());
    };
    held.release_with_record(built.path(), &record).expect("installed and recorded");
    assert_eq!(seen.into_inner(), vec![(true, true)], "the lock was held and the index installed when the record was made");
    assert!(!lock.exists());
    // A record that cannot be made.
    let second_hold = try_acquire(index.as_path(), 9).expect("the lock is free");
    let second_built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &second_hold, 9).expect("built");
    let broken = || -> std::io::Result<()> { return Err(std::io::Error::other("no space left")); };
    assert!(second_hold.release_with_record(second_built.path(), &broken).is_err());
    assert!(lock.exists(), "the lock stays for the restart to reclaim");
    assert!(!differs(repo.as_path(), second.as_str()), "the installed index stays");
    std::fs::remove_file(&lock).expect("the restart reclaims the lock");
    remove(root.as_path());
}

/// The landing's private copy is a file of its own:
/// when the live index is itself named like a private copy (a client's
/// `GIT_INDEX_FILE`),
/// the live index is neither removed nor taken for the copy,
/// and nothing staged in it is lost.
#[test]
fn a_live_index_named_like_a_private_copy_is_not_taken_for_one() {
    use crate::service_index_lock::try_acquire;
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-alias-live", files.as_slice());
    std::fs::write(repo.join("unselected.txt"), b"staged\n").expect("write");
    crate::test_support::age(repo.join("unselected.txt").as_path());
    git(repo.as_path(), &["add", "unselected.txt"]);
    let live: PathBuf = repo.join(".git").join("index.cli-git-7");
    std::fs::copy(repo.join(".git").join("index"), &live).expect("a client's index named like a private copy");
    let before: Vec<u8> = std::fs::read(&live).expect("live");
    let mut context = git_context();
    context.common_dir = Some(repo.join(".git"));
    let held = try_acquire(live.as_path(), 7).expect("the lock is free");
    let built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 7).expect("built");
    assert_ne!(built.path(), live, "the private copy is not the live index");
    assert_eq!(std::fs::read(&live).expect("the live index stays until the installation"), before);
    held.release_with(built.path()).expect("installed");
    let output = git_with(repo.as_path(), &[("GIT_INDEX_FILE", live.as_os_str().to_os_string())], &["diff", "--cached", "--name-only", second.as_str()], None);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "unselected.txt", "the unselected stage is still in the client's index");
    remove(root.as_path());
}

/// Another client's index that happens to have the name of the private copy is not overwritten or removed.
#[test]
fn another_clients_file_with_the_private_name_is_left_alone() {
    use crate::service_index_lock::try_acquire;
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-alias-other", files.as_slice());
    let index: PathBuf = repo.join(".git").join("index");
    let other: PathBuf = repo.join(".git").join("index.cli-git-7");
    std::fs::write(&other, b"another client's index").expect("other");
    let mut context = git_context();
    context.common_dir = Some(repo.join(".git"));
    let held = try_acquire(index.as_path(), 7).expect("the lock is free");
    let built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 7).expect("built");
    assert_ne!(built.path(), other);
    held.release_with(built.path()).expect("installed");
    assert_eq!(std::fs::read(&other).expect("other"), b"another client's index".to_vec());
    assert!(!differs(repo.as_path(), second.as_str()));
    remove(root.as_path());
}

/// The live index of a repository whose index is behind,
/// and a lock on it.
fn locked_index(name: &str) -> (PathBuf, PathBuf, PathBuf, crate::service_index_lock::IndexLock) {
    let (root, repo, _first, _second) = index_behind(name, &[b"top.txt".to_vec()]);
    let index: PathBuf = repo.join(".git").join("index");
    let held = crate::service_index_lock::try_acquire(index.as_path(), 1).expect("the lock is free");
    return (root, repo, index, held);
}

/// The private copy's name does not grow with the live index's:
/// the longest index name the lock accepts (the lock's own staging
/// name overflows a file name first,
/// at `service_index_staging.rs`,
/// long before the 246 bytes the review named) still gets a copy.
#[test]
fn a_long_index_name_still_gets_a_private_copy() {
    let (root, repo, _first, _second) = index_behind("service-follow-long-name", &[b"top.txt".to_vec()]);
    let dot_git: PathBuf = repo.join(".git");
    let (_long, held) = (1..=246_usize)
        .rev()
        .find_map(|length| {
            let long: PathBuf = dot_git.join("i".repeat(length));
            std::fs::copy(dot_git.join("index"), &long).ok()?;
            let held = crate::service_index_lock::try_acquire(long.as_path(), 1)?;
            return Some((long, held));
        })
        .expect("some index name takes the lock");
    let copy = PrivateCopy::make(&held, 1).expect("a private copy of the longest index name that takes the lock");
    assert!(copy.path().exists());
    drop(copy);
    drop(held);
    remove(root.as_path());
}

/// Copies other programs left under any name the landing once used never use up its names,
/// and none of them is touched.
#[test]
fn leftovers_under_the_old_names_do_not_exhaust_the_private_copies() {
    let (root, repo, _index, held) = locked_index("service-follow-exhausted");
    let dot_git: PathBuf = repo.join(".git");
    let mut foreign: Vec<PathBuf> = vec![dot_git.join("index.cli-git-1")];
    foreign.extend((1..32).map(|attempt| { return dot_git.join(format!("index.cli-git-1-{attempt}")); }));
    for path in &foreign {
        std::fs::write(path, b"another program's file").expect("foreign file");
    }
    let copy = PrivateCopy::make(&held, 1).expect("a private copy although every old name is taken");
    assert!(copy.path().exists());
    for path in &foreign {
        assert_eq!(std::fs::read(path).expect("the foreign file stays"), b"another program's file".to_vec());
    }
    drop(copy);
    drop(held);
    remove(root.as_path());
}

/// A private copy left by a service that died is removed by the next landing,
/// identified by the directory name,
/// its owner and
/// the death of whoever held it:
/// a directory with the name pattern that is not ours,
/// one a live owner holds,
/// and a plain file with
/// the name are not touched.
/// A marker names a process number,
/// which a living process may have taken over since,
/// and a service
/// that died before it wrote a marker left none;
/// neither decides anything.
#[test]
fn only_a_dead_services_leftover_is_removed() {
    use std::os::unix::fs::PermissionsExt as _;
    let (root, repo, _index, held) = locked_index("service-follow-sweep");
    let dot_git: PathBuf = repo.join(".git");
    let named = |tail: &str| -> PathBuf { return dot_git.join(format!("cli-git-private-{tail}")); };
    let dead: PathBuf = named("00000000-0000-4000-8000-000000000001");
    let unmarked: PathBuf = named("00000000-0000-4000-8000-000000000002");
    let reused: PathBuf = named("00000000-0000-4000-8000-000000000003");
    let plain: PathBuf = named("00000000-0000-4000-8000-000000000004");
    let not_ours: PathBuf = named("00000000-0000-4000-8000-000000000005");
    for directory in [&dead, &unmarked, &reused, &not_ours] {
        std::fs::create_dir(directory).expect("leftover");
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700)).expect("owner only");
        std::fs::write(directory.join("index"), b"copy").expect("copy");
    }
    // Opened to others: not a directory this service makes.
    std::fs::set_permissions(&not_ours, std::fs::Permissions::from_mode(0o755)).expect("open to others");
    // A marker of a process that is gone, and one that names a process that lives now: the number was taken over.
    std::fs::write(dead.join("owner"), format!("cli-git-private-copy v1\npid={}\n", i32::MAX)).expect("marker");
    std::fs::write(reused.join("owner"), format!("cli-git-private-copy v1\npid={}\n", std::process::id())).expect("marker");
    std::fs::write(&plain, b"not a directory").expect("plain");
    // A copy a live owner holds, whatever its marker says.
    std::fs::copy(dot_git.join("index"), dot_git.join("other-index")).expect("another index");
    let owner = crate::service_index_lock::try_acquire(dot_git.join("other-index").as_path(), 2).expect("a lock on another index in the same directory");
    let alive = PrivateCopy::make(&owner, 2).expect("a live owner's copy");
    std::fs::write(alive.directory().join("owner"), "cli-git-private-copy v1\npid=1\n").ok();
    let copy = PrivateCopy::make(&held, 1).expect("a private copy");
    assert!(!dead.exists(), "the dead service's copy is removed");
    assert!(!unmarked.exists(), "the copy of a service that died before it wrote a marker is removed");
    assert!(!reused.exists(), "a marker whose process number a living process took over decides nothing");
    assert!(not_ours.join("index").exists() && plain.exists(), "nothing else is touched");
    assert!(alive.path().exists(), "a copy a live owner holds stays, which only the owner's hold can tell");
    drop(copy);
    drop(alive);
    drop(owner);
    drop(held);
    remove(root.as_path());
}

/// Another client's lock at the name a private copy used to have does not stop a landing whose live index is absent:
/// the copy
/// is in a directory of its own,
/// so no name beside the index is shared.
#[test]
fn a_foreign_lock_at_the_old_copy_name_does_not_stop_a_landing_without_an_index() {
    use crate::service_index_lock::try_acquire;
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-foreign-lock", files.as_slice());
    let index: PathBuf = repo.join(".git").join("index");
    std::fs::remove_file(&index).expect("no live index");
    let foreign: PathBuf = repo.join(".git").join("index.cli-git-7.lock");
    std::fs::write(&foreign, b"another client holds this").expect("foreign lock");
    let mut context = git_context();
    context.common_dir = Some(repo.join(".git"));
    let held = try_acquire(index.as_path(), 7).expect("the lock is free");
    let built = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 7);
    assert!(built.is_ok(), "the landing builds its index: {:?}", built.as_ref().err());
    assert_eq!(std::fs::read(&foreign).expect("the foreign lock stays"), b"another client holds this".to_vec());
    drop(built);
    drop(held);
    remove(root.as_path());
}

/// The private copy is made where only its owner can open it,
/// and the index it installs when nothing changed has the mode the
/// live index has,
/// whatever the process mask is.
#[test]
fn the_private_copy_is_owner_only_and_a_no_change_keeps_the_live_mode() {
    if crate::umask_child_test_support::ran_in_child(module_path!(), "the_private_copy_is_owner_only_and_a_no_change_keeps_the_live_mode", 0o000) {
        return;
    }
    use std::os::unix::fs::PermissionsExt as _;
    let (root, repo, index, held) = locked_index("service-follow-private-mode");
    std::fs::set_permissions(&index, std::fs::Permissions::from_mode(0o640)).expect("a live mode");
    let mut context = git_context();
    context.common_dir = Some(repo.join(".git"));
    let none: Vec<Vec<u8>> = Vec::new();
    let built = build_index(&context, repo.as_path(), "0".repeat(40).as_str(), IndexPaths::Named(&none), &held, 1).expect("built");
    let path: PathBuf = AsRef::<Path>::as_ref(&built).to_path_buf();
    let directory: &Path = path.parent().expect("a directory");
    assert_ne!(directory, repo.join(".git").as_path(), "the copy is not loose in the live index's directory");
    assert_eq!(std::fs::metadata(directory).expect("directory").permissions().mode() & 0o777, 0o700, "only the owner opens the directory");
    assert_eq!(std::fs::metadata(&path).expect("copy").permissions().mode() & 0o777, 0o640, "the live index's mode");
    drop(built);
    drop(held);
    remove(root.as_path());
}

/// A read-only filesystem and a permission error are different failures,
/// by the error number and by Git's words,
/// so the person is told which one it is.
#[test]
fn a_read_only_filesystem_is_kept_apart_from_a_permission_error() {
    assert_eq!(storage_failure(&std::io::Error::from_raw_os_error(libc::EROFS)), FailureCode::RepositoryReadOnly);
    assert_eq!(storage_failure(&std::io::Error::from_raw_os_error(libc::EACCES)), FailureCode::PermissionDenied);
    assert_eq!(storage_failure(&std::io::Error::from_raw_os_error(libc::ENOSPC)), FailureCode::RepositoryFull);
    let said = |words: &str| {
        return classify(&GitOutput { code: Some(128), stdout: Vec::new(), stderr: words.as_bytes().to_vec() });
    };
    assert_eq!(said("error: could not write config file: Read-only file system"), FailureCode::RepositoryReadOnly);
    assert_eq!(said("fatal: Unable to create '/r/.git/index.lock': Permission denied"), FailureCode::PermissionDenied);
    assert_eq!(said("fatal: something else"), FailureCode::Unavailable);
}

/// A schedule of handoffs does not outlast the ceiling:
/// a holder that lets go and a new one that takes the lock again and again is waited for once,
/// under one deadline.
#[test]
fn a_repeated_handoff_ends_at_the_ceiling() {
    let (root, repo, _first, second) = index_behind("service-follow-handoff", &[b"new.txt".to_vec()]);
    let lock: PathBuf = repo.join(".git/index.lock");
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let holder_stop = std::sync::Arc::clone(&stop);
    let held: PathBuf = lock.clone();
    let holder = std::thread::spawn(move || {
        // Take the lock, let it go, and take it again: every wait the update makes is short, and the run of them is not.
        while !holder_stop.load(std::sync::atomic::Ordering::SeqCst) {
            if std::fs::write(&held, b"").is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(10));
                let _gone = std::fs::remove_file(&held);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        }
    });
    let (sender, receiver) = std::sync::mpsc::channel();
    let index: PathBuf = repo.join(".git/index");
    let second_oid: String = second.clone();
    let waiting = std::thread::spawn(move || {
        let answered = update_index_within(&git_context(), repo.as_path(), index.as_path(), std::time::Duration::from_millis(300), second_oid.as_str(), IndexPaths::Named(&[b"new.txt".to_vec()]));
        let _sent = sender.send(answered);
    });
    let ended: Result<Result<(), crate::service_protocol::FailureCode>, _> = receiver.recv_timeout(std::time::Duration::from_secs(20));
    stop.store(true, std::sync::atomic::Ordering::SeqCst);
    holder.join().expect("the holder ends");
    let _ = waiting.join();
    // Whether it took the index between two holds or ended at the ceiling, it ended: one deadline covers the whole run of handoffs.
    let answered = ended.expect("the update ended rather than waiting through every handoff");
    assert!(answered.is_ok() || answered == Err(crate::service_protocol::FailureCode::IndexInUse), "{answered:?}");
    remove(root.as_path());
}
