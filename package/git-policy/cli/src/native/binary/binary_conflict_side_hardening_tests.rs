//! What:
//!  Command lines that look like one conflicted file to the guard and are more to Git:
//! pathspec modes that
//!  widen a match,
//! a symbolic link or a file where a directory was,
//! and an option value that reads as an option,
//! each through the built executable against real Git.
//! Why:
//!  A cross-vendor review of the old carve-out named these.
//! Each control makes the situation with real Git,
//!  gives the file the person had,
//! runs the wrapper,
//! and compares what is left with what the person had:
//!  the refusal must never name a file the command would not replace,
//!  and it must never replace one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await makeConflict('merge'); edit('A.rs'); run(['--icase-pathspecs', 'checkout', '--theirs', '--', 'a.rs']);
//! expect(readFileSync('A.rs')).toEqual(whatThePersonHad);
//! ```

use super::super::support::{Fixture, Observed, direct, fixture, git, observe, remove, run_wrapped, stderr_of, wrapped};
use super::{HAND_EDITED, Stop, conflicted, nothing_was_saved, rejected_by_policy};
use std::path::PathBuf;

/// What a person had in a clean-looking file that only a case-insensitive match would reach.
const LOOK_ALIKE_EDIT: &[u8] = b"my unsaved edit of the look-alike\n";

/// A merge stopped with `a.rs` conflicted and a tracked,
/// staged `A.rs` that has an unsaved edit.
fn with_look_alike(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = conflicted(fixture, Stop::Merge);
    std::fs::write(repo.join("A.rs"), "staged\n").expect("look-alike");
    git(fixture, repo.as_path(), &["add", "--", "A.rs"]);
    std::fs::write(repo.join("A.rs"), LOOK_ALIKE_EDIT).expect("unsaved edit");
    return repo;
}

/// Nothing was replaced or saved:
/// the person's files are as they were,
/// and no copy exists.
fn assert_untouched(repo: &std::path::Path, said: &str) {
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "a.rs is as the person had it: {said}");
    assert_eq!(std::fs::read(repo.join("A.rs")).expect("A.rs"), LOOK_ALIKE_EDIT, "A.rs is as the person had it: {said}");
    nothing_was_saved(repo, said);
}

/// Real Git,
/// given the pathspec mode,
/// overwrites the look-alike too:
/// the situation exists,
/// and the control can show a difference.
#[test]
fn real_git_with_case_insensitive_pathspecs_overwrites_the_look_alike() {
    let fixture: Fixture = fixture("icase-oracle");
    let repo: PathBuf = with_look_alike(&fixture);
    let mut command = direct(&fixture);
    command.current_dir(&repo).env("GIT_ICASE_PATHSPECS", "1").args(["checkout", "--theirs", "--", "a.rs"]);
    let observed: Observed = observe(&mut command, b"");
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(std::fs::read(repo.join("A.rs")).expect("A.rs"), b"staged\n", "Git replaced the look-alike with the index copy");
    remove(&fixture);
}

/// Every global pathspec mode,
/// as a variable and as a global option,
/// keeps the rejection:
/// a mode changes which files a pathspec names,
/// so no file is named and nothing is touched.
#[test]
fn a_global_pathspec_mode_keeps_the_rejection() {
    let fixture: Fixture = fixture("icase-modes");
    let repo: PathBuf = with_look_alike(&fixture);
    for variable in ["GIT_ICASE_PATHSPECS", "GIT_GLOB_PATHSPECS", "GIT_NOGLOB_PATHSPECS", "GIT_LITERAL_PATHSPECS"] {
        let mut command = wrapped(&fixture);
        command.current_dir(&repo).env(variable, "1").args(["checkout", "--theirs", "--", "a.rs"]);
        let observed: Observed = observe(&mut command, b"");
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{variable}: {said}");
        assert!(!said.contains("keeps no copy"), "{variable}: a mode names no file, so none is claimed: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    for option in ["--icase-pathspecs", "--glob-pathspecs", "--noglob-pathspecs", "--literal-pathspecs"] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &[option, "checkout", "--theirs", "--", "a.rs"]);
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{option}: {said}");
        assert!(!said.contains("keeps no copy"), "{option}: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    for token in [":(icase)a.rs", ":(glob)a.rs", ":(exclude)A.rs", ":!A.rs", ":/a.rs", ":(literal)a.rs"] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", token]);
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{token}: {said}");
        assert!(!said.contains("keeps no copy"), "{token}: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    remove(&fixture);
}

/// What stands where the directory `dir` was when the command runs.
#[derive(Clone, Copy, Debug)]
enum InTheWay {
    /// A symbolic link to a directory that holds a `file` of its own.
    LinkToDirectory,
    /// A symbolic link to nothing.
    DanglingLink,
    /// A regular file.
    RegularFile,
}

/// A merge stopped with `dir/file` conflicted,
/// then `dir` replaced by `in_the_way`;
/// the thing now at `dir` is returned with
///  its description,
/// which `assert_in_the_way_kept` compares.
fn conflicted_below_a_replaced_directory(fixture: &Fixture, in_the_way: InTheWay) -> PathBuf {
    let repo: PathBuf = super::super::support::repository(fixture, std::ffi::OsStr::new("repo"));
    std::fs::create_dir(repo.join("dir")).expect("dir");
    std::fs::write(repo.join("dir/file"), "base\n").expect("base");
    git(fixture, repo.as_path(), &["add", "--", "dir/file"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(repo.join("dir/file"), "topic\n").expect("topic");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=topic side"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("dir/file"), "main\n").expect("main");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main side"]);
    let started: Observed = super::super::support::run_direct(fixture, repo.as_path(), &["merge", "--no-edit", "topic"]);
    assert_eq!(started.code, Some(1), "the merge stops with a conflict: {}", stderr_of(&started));
    std::fs::remove_dir_all(repo.join("dir")).expect("remove the directory");
    match in_the_way {
        InTheWay::LinkToDirectory => {
            std::fs::create_dir(repo.join("elsewhere")).expect("elsewhere");
            std::fs::write(repo.join("elsewhere/file"), "the person's file reached through the link\n").expect("file");
            std::os::unix::fs::symlink("elsewhere", repo.join("dir")).expect("link");
        }
        InTheWay::DanglingLink => std::os::unix::fs::symlink("nowhere", repo.join("dir")).expect("link"),
        InTheWay::RegularFile => std::fs::write(repo.join("dir"), "the person's file named dir\n").expect("file"),
    }
    return repo;
}

/// The thing the person put at `dir` is still there,
/// as it was.
fn assert_in_the_way_kept(repo: &std::path::Path, in_the_way: InTheWay, said: &str) {
    match in_the_way {
        InTheWay::LinkToDirectory => {
            assert_eq!(std::fs::read_link(repo.join("dir")).expect("dir is still a link"), PathBuf::from("elsewhere"), "{said}");
            assert_eq!(std::fs::read(repo.join("elsewhere/file")).expect("file"), b"the person's file reached through the link\n", "{said}");
        }
        InTheWay::DanglingLink => assert_eq!(std::fs::read_link(repo.join("dir")).expect("dir is still a link"), PathBuf::from("nowhere"), "{said}"),
        InTheWay::RegularFile => assert_eq!(std::fs::read(repo.join("dir")).expect("dir is still a file"), b"the person's file named dir\n", "{said}"),
    }
}

/// A link or a file where a directory was is replaced by a real directory when Git writes `dir/file`;
/// so the command is
///  rejected naming the unmerged file,
///  and everything stays as the person had it.
#[test]
fn something_where_a_directory_was_is_never_replaced_unsaved() {
    for in_the_way in [InTheWay::LinkToDirectory, InTheWay::DanglingLink, InTheWay::RegularFile] {
        let fixture: Fixture = fixture(format!("side-ancestor-{in_the_way:?}").to_lowercase().as_str());
        let repo: PathBuf = conflicted_below_a_replaced_directory(&fixture, in_the_way);
        for arguments in [["checkout", "--theirs", "--", "dir/file"], ["restore", "--ours", "--", "dir/file"]] {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), &arguments);
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{in_the_way:?} {arguments:?}: {said}");
            assert!(said.contains("`dir/file`") && said.contains("unmerged"), "{in_the_way:?} the refusal names the unmerged file: {said}");
            assert!(said.contains("--no-enforce-worktree"), "{in_the_way:?} the refusal names the opt-out: {said}");
            assert_in_the_way_kept(repo.as_path(), in_the_way, said.as_str());
            nothing_was_saved(repo.as_path(), said.as_str());
        }
        remove(&fixture);
    }
}

/// The same line is rejected as the index-only restore it is,
/// not as a restore of file content:
/// the policy's own reading of
///  `--source -W --staged` takes the `-W` for the worktree option otherwise,
/// and the rejection names the wrong form.
#[test]
fn an_option_value_named_like_the_worktree_option_is_rejected_as_an_index_restore() {
    let fixture: Fixture = fixture("side-option-value-form");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    git(&fixture, repo.as_path(), &["update-ref", "refs/tags/-W", "topic"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["restore", "--source", "-W", "--staged", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert!(rejected_by_policy(&observed), "{said}");
    assert!(said.contains("git restore --staged") && said.contains("removes staged entries"), "the index-only form is named: {said}");
    assert!(!said.contains("keeps no copy"), "an index-only restore replaces no worktree file: {said}");
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "{said}");
    nothing_was_saved(repo.as_path(), said.as_str());
    // A worktree restore of the same file under a real `-W` is read as a restore of file content, and refused.
    let worktree: Observed = run_wrapped(&fixture, repo.as_path(), &["restore", "--staged", "-W", "--theirs", "--source=topic", "--", "c.rs"]);
    assert!(stderr_of(&worktree).contains("overwrites files"), "{}", stderr_of(&worktree));
    remove(&fixture);
}
