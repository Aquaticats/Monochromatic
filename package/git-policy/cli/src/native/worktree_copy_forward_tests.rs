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

/// A settled creation from the main checkout runs Git with the lease, copies the named file, and keeps Git's status.
#[cfg(unix)]
#[test]
fn a_settled_creation_copies_the_named_file() {
    let root: PathBuf = fixture("forward-settled");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = root.join("topic");
    let facts: CopyFacts = CopyFacts {
        identity: main_identity(main.as_path(), common.as_path()),
        creates_or_moves: true,
        main_checkout_paths: vec![String::from(".cache/rules.txt")],
        global_prefix: arguments(&["-C", main.to_str().expect("utf8")]),
        environment: Vec::new(),
    };
    let created: ForwardedRun = run_forwarded_git(
        Path::new(REAL_GIT),
        arguments(&[
            "-C",
            main.to_str().expect("utf8"),
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ])
        .as_slice(),
        isolated_overlay().as_slice(),
        Some(&facts),
    );
    assert_eq!(created, ForwardedRun::Ran(0));
    assert_eq!(std::fs::read(topic.join(".cache/rules.txt")).expect("copied"), b"rules");
    // Git's own failure keeps its status and copies nothing.
    let failed: ForwardedRun = run_forwarded_git(
        Path::new(REAL_GIT),
        arguments(&[
            "-C",
            main.to_str().expect("utf8"),
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            root.join("again").to_str().expect("utf8"),
        ])
        .as_slice(),
        isolated_overlay().as_slice(),
        Some(&facts),
    );
    assert_eq!(failed, ForwardedRun::Ran(255));
    assert!(!root.join("again").exists());
    remove(root.as_path());
}

/// A settled run whose repository cannot be asked about again ends as an engine failure with the reason.
#[cfg(unix)]
#[test]
fn a_settled_run_that_cannot_ask_git_fails_with_the_reason() {
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
