//! What:
//!  Command lines that look like one conflicted file to the conflict-side guard and are more to Git:
//! pathspec modes that
//!  widen a match,
//! a symbolic link where a directory was,
//! an option value that reads as an option,
//! and a file that changes
//!  while it is copied,
//! each through the built executable against real Git.
//! Why:
//!  A cross-vendor review of the conflict-side carve-out named these four.
//! Each control makes the situation with real Git,
//!  gives the file the person had,
//! runs the wrapper,
//! and compares what is left with what the person had.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await makeConflict('merge'); edit('A.rs'); run(['--icase-pathspecs', 'checkout', '--theirs', '--', 'a.rs']);
//! expect(readFileSync('A.rs')).toEqual(whatThePersonHad);
//! ```

use super::super::support::{Fixture, Observed, direct, fixture, git, observe, remove, run_wrapped, stderr_of, wrapped};
use super::{HAND_EDITED, Stop, audit_lines, conflicted, rejected_by_policy, saved_root};
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

/// Nothing was saved or replaced:
/// the person's files are as they were,
/// and no copy or saved-conflict record exists.
fn assert_untouched(repo: &std::path::Path, said: &str) {
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "a.rs is as the person had it: {said}");
    assert_eq!(std::fs::read(repo.join("A.rs")).expect("A.rs"), LOOK_ALIKE_EDIT, "A.rs is as the person had it: {said}");
    assert!(!saved_root(repo).exists(), "no copy was made: {said}");
    assert!(audit_lines(repo).iter().all(|line| { return !line.contains("saved-conflict"); }), "no saved-conflict record: {said}");
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
/// a mode changes which files a
///  pathspec names,
/// and the guard knows only the literal reading.
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
        assert_untouched(repo.as_path(), said.as_str());
    }
    for option in ["--icase-pathspecs", "--glob-pathspecs", "--noglob-pathspecs", "--literal-pathspecs"] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &[option, "checkout", "--theirs", "--", "a.rs"]);
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{option}: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    for token in [":(icase)a.rs", ":(glob)a.rs", ":(exclude)A.rs", ":!A.rs", ":/a.rs", ":(literal)a.rs"] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", token]);
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{token}: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    // The same fixture is let through without a mode, so the rejections above are the modes' doing and not the fixture's.
    let plain: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    assert_eq!(plain.code, Some(0), "{}", stderr_of(&plain));
    assert_eq!(std::fs::read(repo.join("A.rs")).expect("A.rs"), LOOK_ALIKE_EDIT, "the look-alike is not named, so it stays");
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
/// the wrapper saves
///  nothing of it,
/// so it rejects the command,
/// names `dir`,
/// and leaves everything as the person had it.
#[test]
fn something_where_a_directory_was_is_never_replaced_unsaved() {
    for (in_the_way, kind) in [(InTheWay::LinkToDirectory, "symbolic link"), (InTheWay::DanglingLink, "symbolic link"), (InTheWay::RegularFile, "file")] {
        let fixture: Fixture = fixture(format!("side-ancestor-{in_the_way:?}").to_lowercase().as_str());
        let repo: PathBuf = conflicted_below_a_replaced_directory(&fixture, in_the_way);
        for arguments in [["checkout", "--theirs", "--", "dir/file"], ["restore", "--ours", "--", "dir/file"]] {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), &arguments);
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{in_the_way:?} {arguments:?}: {said}");
            assert!(said.contains("`dir`") && said.contains(kind), "{in_the_way:?} the refusal names the ancestor and what it is: {said}");
            assert!(said.contains("--no-enforce-worktree") && said.contains("Move"), "{in_the_way:?} the refusal says what to do: {said}");
            assert_in_the_way_kept(repo.as_path(), in_the_way, said.as_str());
        }
        assert!(audit_lines(repo.as_path()).iter().all(|line| { return !line.contains("saved-conflict"); }), "{in_the_way:?}");
        remove(&fixture);
    }
}

/// `git restore --source -W --staged -- a.rs` takes `-W` as the name of the source,
/// which a tag can be called,
/// and so restores
///  the index only;
/// the guard must not read that `-W` as the worktree option,
/// save a copy,
/// and let the line through as a
///  worktree restore.
#[test]
fn an_option_value_named_like_the_worktree_option_is_not_that_option() {
    let fixture: Fixture = fixture("side-option-value");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    git(&fixture, repo.as_path(), &["update-ref", "refs/tags/-W", "topic"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["restore", "--source", "-W", "--staged", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert!(!said.contains("saved your copy"), "no worktree file is replaced, so none is saved: {said}");
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "{said}");
    assert!(!saved_root(repo.as_path()).exists(), "no copy was made: {said}");
    assert!(audit_lines(repo.as_path()).iter().all(|line| { return !line.contains("saved-conflict"); }), "{said}");
    remove(&fixture);
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
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "{said}");
    // The worktree restore of the same file with a real `-W` is still read as a restore of file content.
    let worktree: Observed = run_wrapped(&fixture, repo.as_path(), &["restore", "--staged", "-W", "--theirs", "--source=topic", "--", "c.rs"]);
    assert!(stderr_of(&worktree).contains("overwrites files"), "{}", stderr_of(&worktree));
    remove(&fixture);
}

/// What every note about a pathspec mode or pathspec magic says once the line would be allowed without it.
const ALLOWED_WITHOUT: &str = "saved first and then allowed";

/// The global pathspec modes,
/// as the spelling the note must name and how to put each in effect.
fn modes() -> Vec<(&'static str, Option<&'static str>, Option<&'static str>)> {
    // (spelling named in the note, variable set to 1, global option placed before the subcommand)
    return vec![
        ("GIT_ICASE_PATHSPECS", Some("GIT_ICASE_PATHSPECS"), None),
        ("GIT_GLOB_PATHSPECS", Some("GIT_GLOB_PATHSPECS"), None),
        ("GIT_NOGLOB_PATHSPECS", Some("GIT_NOGLOB_PATHSPECS"), None),
        ("GIT_LITERAL_PATHSPECS", Some("GIT_LITERAL_PATHSPECS"), None),
        ("--icase-pathspecs", None, Some("--icase-pathspecs")),
        ("--glob-pathspecs", None, Some("--glob-pathspecs")),
        ("--noglob-pathspecs", None, Some("--noglob-pathspecs")),
        ("--literal-pathspecs", None, Some("--literal-pathspecs")),
        ("--no-literal-pathspecs", None, Some("--no-literal-pathspecs")),
    ];
}

/// Run `arguments` in `repo` under a mode:
/// the variable set,
/// or the option placed before the subcommand.
fn run_under_mode(fixture: &Fixture, repo: &std::path::Path, mode: (&str, Option<&str>, Option<&str>), arguments: &[&str]) -> Observed {
    let mut command = wrapped(fixture);
    command.current_dir(repo);
    if let Some(variable) = mode.1 {
        command.env(variable, "1");
    }
    if let Some(option) = mode.2 {
        command.arg(option);
    }
    command.args(arguments);
    return observe(&mut command, b"");
}

/// A line refused only because a mode is in effect is still refused,
/// and the refusal names the mode and says that the same
///  command without it would be saved first and then allowed.
#[test]
fn a_refusal_for_a_pathspec_mode_names_the_mode_and_the_way_out() {
    let fixture: Fixture = fixture("mode-note");
    let repo: PathBuf = with_look_alike(&fixture);
    for mode in modes() {
        for arguments in [["checkout", "--theirs", "--", "a.rs"], ["restore", "--ours", "--", "a.rs"]] {
            let observed: Observed = run_under_mode(&fixture, repo.as_path(), mode, &arguments);
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{mode:?} {arguments:?}: {said}");
            assert!(said.contains(format!("the same command without `{}`", mode.0).as_str()), "{mode:?} {arguments:?} names the mode: {said}");
            assert!(said.contains(ALLOWED_WITHOUT), "{mode:?} {arguments:?} says the way out: {said}");
            assert_untouched(repo.as_path(), said.as_str());
        }
    }
    remove(&fixture);
}

/// The note never appears for a line the carve-out would refuse anyway:
/// a path that is not conflicted,
/// a mix,
/// a form the
///  carve-out does not cover,
/// no operation in progress.
#[test]
fn no_pathspec_mode_note_appears_when_the_carve_out_would_refuse_anyway() {
    let fixture: Fixture = fixture("mode-note-absent");
    let repo: PathBuf = with_look_alike(&fixture);
    for mode in modes() {
        for arguments in [
            vec!["checkout", "--theirs", "--", "c.rs"],
            vec!["checkout", "--theirs", "--", "a.rs", "c.rs"],
            vec!["checkout", "--theirs", "--", "A.rs"],
            vec!["checkout", "-f", "--", "a.rs"],
            vec!["checkout", "--theirs", "a.rs"],
            vec!["restore", "--staged", "--theirs", "--", "a.rs"],
            vec!["checkout", "--theirs", "--", "*.rs"],
        ] {
            let observed: Observed = run_under_mode(&fixture, repo.as_path(), mode, arguments.as_slice());
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{mode:?} {arguments:?}: {said}");
            assert!(!said.contains(ALLOWED_WITHOUT) && !said.contains(mode.0), "{mode:?} {arguments:?} has no note: {said}");
        }
    }
    // The line is allowed without a mode, so the note is the only difference between the runs above and a run without one.
    let ended: PathBuf = repo.join(".git/MERGE_HEAD");
    let saved: Vec<u8> = std::fs::read(&ended).expect("MERGE_HEAD");
    std::fs::remove_file(&ended).expect("end the operation's state");
    for mode in modes() {
        let observed: Observed = run_under_mode(&fixture, repo.as_path(), mode, &["checkout", "--theirs", "--", "a.rs"]);
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{mode:?}: {said}");
        assert!(!said.contains(ALLOWED_WITHOUT), "{mode:?} no operation, no note: {said}");
    }
    std::fs::write(&ended, saved).expect("restore MERGE_HEAD");
    assert_untouched(repo.as_path(), "");
    remove(&fixture);
}

/// Pathspec magic on a conflicted file's name is refused,
/// and the refusal names the magic and the file and says that naming
///  the file itself is saved first and then allowed;
/// magic on a clean path,
/// or beside one,
/// gets no such sentence.
#[test]
fn a_refusal_for_pathspec_magic_names_the_magic_and_the_file() {
    let fixture: Fixture = fixture("magic-note");
    let repo: PathBuf = with_look_alike(&fixture);
    for (token, magic) in [(":(icase)a.rs", ":(icase)"), (":(glob)a.rs", ":(glob)"), (":(literal)a.rs", ":(literal)"), (":!a.rs", ":!"), (":/a.rs", ":/"), (":(top)a.rs", ":(top)")] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", token]);
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{token}: {said}");
        assert!(said.contains(format!("`{magic}`").as_str()) && said.contains("`a.rs`") && said.contains(ALLOWED_WITHOUT), "{token}: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    for arguments in [
        vec!["checkout", "--theirs", "--", ":(icase)c.rs"],
        vec!["checkout", "--theirs", "--", ":(icase)a.rs", "c.rs"],
        vec!["checkout", "--theirs", "--", ":(icase)*.rs"],
        vec!["checkout", "--theirs", "--", ":(attr:text)a.rs"],
        vec!["checkout", "-f", "--", ":(icase)a.rs"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{arguments:?}: {said}");
        assert!(!said.contains(ALLOWED_WITHOUT), "{arguments:?} has no note: {said}");
    }
    remove(&fixture);
}
