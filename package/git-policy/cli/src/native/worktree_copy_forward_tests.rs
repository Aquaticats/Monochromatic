//! Forwarding-run checks against real Git: which invocations the copy applies to, and how a settled run ends.

use super::*;
use crate::test_support::{REAL_GIT, fixture, git, isolated_overlay, remove, repository};
use std::path::Path;

/// A repository with an ignored rules file, returned with its canonical root and common directory.
fn source(root: &Path) -> (PathBuf, PathBuf) {
    let main: PathBuf = repository(root, "main");
    std::fs::write(main.join(".gitignore"), b".cache/\n").expect("gitignore");
    std::fs::create_dir_all(main.join(".cache")).expect("cache");
    std::fs::write(main.join(".cache/rules.txt"), b"rules").expect("rules");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    return (std::fs::canonicalize(main).expect("canonical main"), common);
}

/// The identity of the main worktree of a source.
fn main_identity(main: &Path, common: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: common.to_path_buf(),
        git_dir: common.to_path_buf(),
        worktree_root: main.to_path_buf(),
    };
}

/// Text arguments as owned operating-system arguments.
fn arguments(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// The copy applies to linked and bare repositories always, to the main checkout only for a creation with paths,
/// and never outside a repository.
#[test]
fn the_copy_applies_by_repository_shape() {
    let named: Vec<String> = vec![String::from(".cache/rules.txt")];
    let main: WorktreeIdentity = main_identity(Path::new("/r"), Path::new("/r/.git"));
    let linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: PathBuf::from("/r/.git"),
        git_dir: PathBuf::from("/r/.git/worktrees/w"),
        worktree_root: PathBuf::from("/w"),
    };
    let bare: WorktreeIdentity = WorktreeIdentity::BareRepository {
        common_dir: PathBuf::from("/b.git"),
        git_dir: PathBuf::from("/b.git"),
    };
    for creates in [true, false] {
        assert!(copy_applies(&linked, creates, &[]), "linked {creates}");
        assert!(copy_applies(&bare, creates, &[]), "bare {creates}");
        assert!(!copy_applies(&WorktreeIdentity::OutsideWorktree, creates, named.as_slice()));
    }
    assert!(copy_applies(&main, true, named.as_slice()));
    assert!(!copy_applies(&main, false, named.as_slice()), "a main checkout command that creates nothing");
    assert!(!copy_applies(&main, true, &[]), "a main checkout without a configured file");
}

/// Without a copy the run is plain Git: its exit status passes through, and an unstartable Git is an engine failure.
#[test]
fn a_plain_run_reports_git_status_or_the_start_failure() {
    let root: PathBuf = fixture("forward-plain");
    let (main, _common) = source(root.as_path());
    assert_eq!(
        run_forwarded_git(
            Path::new(REAL_GIT),
            arguments(&["-C", main.to_str().expect("utf8"), "rev-parse", "--git-dir"]).as_slice(),
            isolated_overlay().as_slice(),
            None,
        ),
        ForwardedRun::Ran(0)
    );
    assert_eq!(
        run_forwarded_git(
            Path::new(REAL_GIT),
            arguments(&["no-such-subcommand"]).as_slice(),
            isolated_overlay().as_slice(),
            None,
        ),
        ForwardedRun::Ran(1)
    );
    match run_forwarded_git(
        Path::new("/nonexistent/git"),
        arguments(&["status"]).as_slice(),
        &[],
        None,
    ) {
        ForwardedRun::Failed { stderr, code } => {
            assert_eq!(code, 2);
            assert!(stderr.starts_with("cli-git: cannot start /nonexistent/git: "), "{stderr}");
        }
        other => panic!("expected a start failure, got {other:?}"),
    }
    remove(root.as_path());
}

/// The destination of a plain `git worktree add` is read the way Git's option parser reads the command line: after the
/// global options, with clustered short options, abbreviated long options and attached values.
#[test]
fn the_added_path_is_read_from_the_command_line() {
    let cwd: &Path = Path::new("/work/repo");
    let read = |values: &[&str]| -> AddedPath { return added_worktree(arguments(values).as_slice(), cwd); };
    let named = |path: &str| -> AddedPath { return AddedPath::Named(PathBuf::from(path)); };
    assert_eq!(read(&["worktree", "add", "../topic"]), named("/work/repo/../topic"));
    assert_eq!(read(&["worktree", "add", "/abs/topic", "main"]), named("/abs/topic"));
    assert_eq!(
        read(&["worktree", "add", "--quiet", "--lock", "--reason", "initializing", "-b", "new", "../topic"]),
        named("/work/repo/../topic"),
        "the values of --reason and -b are not the path"
    );
    assert_eq!(read(&["worktree", "add", "-B", "reset", "--detach", "-f", "../topic", "HEAD"]), named("/work/repo/../topic"));
    assert_eq!(read(&["worktree", "add", "--", "-odd"]), named("/work/repo/-odd"), "after -- everything is a path");
    assert_eq!(read(&["-C", "sub", "worktree", "add", "x"]), named("/work/repo/sub/x"));
    assert_eq!(read(&["-C", "/elsewhere", "-C", "inner", "worktree", "add", "x"]), named("/elsewhere/inner/x"));
    assert_eq!(read(&["-c", "core.quotepath=false", "--no-pager", "worktree", "add", "x"]), named("/work/repo/x"));
    assert_eq!(read(&["worktree", "move", "a", "b"]), AddedPath::NotAdd);
}

/// Git's own forms that the first reading missed: a cluster of short options ending in one that takes a value, an
/// attached value, an abbreviated long option with its value in the next word, and `--reason=` with its value attached.
#[test]
fn the_added_path_survives_clustered_and_abbreviated_options() {
    let cwd: &Path = Path::new("/work/repo");
    let read = |values: &[&str]| -> AddedPath { return added_worktree(arguments(values).as_slice(), cwd); };
    let named = |path: &str| -> AddedPath { return AddedPath::Named(PathBuf::from(path)); };
    assert_eq!(read(&["worktree", "add", "--lock", "-fb", "new-branch", "../dest"]), named("/work/repo/../dest"));
    assert_eq!(read(&["worktree", "add", "-bnew-branch", "../dest"]), named("/work/repo/../dest"));
    assert_eq!(read(&["worktree", "add", "-fBnew", "../dest"]), named("/work/repo/../dest"));
    assert_eq!(read(&["worktree", "add", "--lock", "--reas", "why", "../dest"]), named("/work/repo/../dest"));
    assert_eq!(read(&["worktree", "add", "--lock", "--reason=why", "../dest"]), named("/work/repo/../dest"));
    assert_eq!(read(&["worktree", "add", "--no-checkout", "--det", "../dest", "HEAD"]), named("/work/repo/../dest"));
    assert_eq!(read(&["worktree", "add", "--no-lock", "../dest"]), named("/work/repo/../dest"));
}

/// A form this reading does not know is unreadable, which the copy reports; it is never read as some other word.
#[test]
fn an_unknown_command_line_is_unreadable_and_never_guessed() {
    let cwd: &Path = Path::new("/work/repo");
    let read = |values: &[&str]| -> AddedPath { return added_worktree(arguments(values).as_slice(), cwd); };
    assert_eq!(read(&["worktree", "add", "--frobnicate", "../dest"]), AddedPath::Unreadable);
    assert_eq!(read(&["worktree", "add", "-x", "../dest"]), AddedPath::Unreadable);
    assert_eq!(read(&["worktree", "add", "--r", "why", "../dest"]), AddedPath::Unreadable, "--r is ambiguous");
    assert_eq!(read(&["worktree", "add"]), AddedPath::Unreadable);
    assert_eq!(read(&["wta", "../dest"]), AddedPath::Unreadable, "an alias hides the command line");
    assert_eq!(read(&["status"]), AddedPath::Unreadable);
    assert_eq!(read(&[]), AddedPath::Unreadable);
}

/// A settled run whose Git cannot be started ends as an engine failure with the reason.
#[cfg(unix)]
#[test]
fn a_settled_run_that_cannot_start_git_fails_with_the_reason() {
    let root: PathBuf = fixture("forward-unstartable");
    let (main, common) = source(root.as_path());
    git(main.as_path(), &["config", "user.name", "t"]);
    let facts: CopyFacts = CopyFacts {
        identity: main_identity(main.as_path(), common.as_path()),
        creates_or_moves: true,
        main_checkout_paths: vec![String::from(".cache/rules.txt")],
        global_prefix: Vec::new(),
        environment: Vec::new(),
    };
    match run_forwarded_git(
        Path::new("/nonexistent/git"),
        arguments(&["worktree", "list"]).as_slice(),
        &[],
        Some(&facts),
    ) {
        ForwardedRun::Failed { stderr, code } => {
            assert_eq!(code, 2);
            assert!(stderr.ends_with('\n'), "{stderr}");
            assert!(stderr.contains("/nonexistent/git"), "{stderr}");
        }
        other => panic!("expected a failure, got {other:?}"),
    }
    remove(root.as_path());
}
