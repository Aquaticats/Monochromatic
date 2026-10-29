//! Controls for `git cli-git doctor`: which entries count as traditional hooks, which directories are scanned,
//! and the exit status and output of the command.

use super::*;
use crate::test_support::{executable, fixture, git, git_context, remove, repository};

/// Names of the paths in a list, for compact comparison.
fn names(paths: &[PathBuf]) -> Vec<String> {
    return paths
        .iter()
        .map(|path| { return path.file_name().expect("name").to_string_lossy().into_owned(); })
        .collect();
}

/// Unpack the exit status, standard output and standard error of an exit action.
fn exit_parts(action: Action) -> (i32, String, String) {
    let Action::Exit { code, stdout, stderr } = action else {
        panic!("doctor must end in an exit action");
    };
    return (code, stdout, stderr);
}

/// Executable regular files and symbolic links are hooks; samples, plain files and directories are not.
#[cfg(unix)]
#[test]
fn only_executable_files_and_links_are_hooks() {
    let root: PathBuf = fixture("doctor-scan");
    let hooks: PathBuf = root.join("hooks");
    std::fs::create_dir(&hooks).expect("hooks");
    executable(hooks.join("pre-commit").as_path(), b"#!/bin/sh\n");
    executable(hooks.join("commit-msg.sample").as_path(), b"#!/bin/sh\n");
    std::fs::write(hooks.join("post-commit"), b"#!/bin/sh\n").expect("plain file");
    std::fs::create_dir(hooks.join("pre-push")).expect("directory");
    std::os::unix::fs::symlink(root.join("nowhere"), hooks.join("update")).expect("dangling link");
    std::os::unix::fs::symlink(hooks.join("pre-commit"), hooks.join("pre-rebase")).expect("working link");
    std::os::unix::fs::symlink(hooks.join("pre-commit"), hooks.join("applypatch-msg.sample")).expect("sample link");
    let found: Vec<PathBuf> = scan_hooks_directory(hooks.as_path()).expect("scan");
    assert_eq!(names(found.as_slice()), vec!["pre-commit", "pre-rebase", "update"]);
    assert!(found.iter().all(|path| { return path.starts_with(&hooks); }));
    remove(root.as_path());
}

/// A directory that does not exist, or is a file, holds no hooks; nothing is created.
#[test]
fn a_missing_or_non_directory_path_holds_no_hooks() {
    let root: PathBuf = fixture("doctor-missing");
    assert_eq!(scan_hooks_directory(root.join("absent").as_path()).expect("scan"), Vec::<PathBuf>::new());
    std::fs::write(root.join("file"), b"x").expect("file");
    assert_eq!(scan_hooks_directory(root.join("file").as_path()).expect("scan"), Vec::<PathBuf>::new());
    assert!(!root.join("absent").exists());
    remove(root.as_path());
}

/// A directory that cannot be read is an error, not silence.
#[cfg(unix)]
#[test]
fn an_unreadable_directory_is_an_error() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("doctor-unreadable");
    let hooks: PathBuf = root.join("hooks");
    std::fs::create_dir(&hooks).expect("hooks");
    std::fs::set_permissions(&hooks, std::fs::Permissions::from_mode(0o000)).expect("seal");
    let readable_anyway: bool = std::fs::read_dir(&hooks).is_ok();
    let outcome = scan_hooks_directory(hooks.as_path());
    std::fs::set_permissions(&hooks, std::fs::Permissions::from_mode(0o700)).expect("unseal");
    // A process that ignores permission bits (root) cannot be refused, so only a refusing process proves the error.
    if !readable_anyway {
        assert!(outcome.is_err());
    }
    remove(root.as_path());
}

/// The default directory comes first, the configured one follows, and one directory named two ways is scanned once.
#[test]
fn directories_are_the_default_then_the_configured_one_once() {
    let plain: HooksLocation = HooksLocation {
        common_directory: PathBuf::from("/repo/.git"),
        configured: None,
    };
    assert_eq!(hook_directories(&plain), vec![PathBuf::from("/repo/.git/hooks")]);
    let configured: HooksLocation = HooksLocation {
        common_directory: PathBuf::from("/repo/.git"),
        configured: Some(PathBuf::from("/repo/custom")),
    };
    assert_eq!(
        hook_directories(&configured),
        vec![PathBuf::from("/repo/.git/hooks"), PathBuf::from("/repo/custom")]
    );
    let same: HooksLocation = HooksLocation {
        common_directory: PathBuf::from("/repo/.git"),
        configured: Some(PathBuf::from("/repo/./sub/../.git/hooks/")),
    };
    assert_eq!(hook_directories(&same), vec![PathBuf::from("/repo/.git/hooks")]);
}

/// The report is one path per line and nothing else.
#[test]
fn the_report_is_one_path_per_line() {
    assert_eq!(render_hooks(&[]), "");
    assert_eq!(
        render_hooks(&[PathBuf::from("/a/hooks/pre-commit"), PathBuf::from("/b/update")]),
        "/a/hooks/pre-commit\n/b/update\n"
    );
}

/// The locator asks Git, so a relative `core.hooksPath` resolves against the worktree root, not the current directory.
#[test]
fn a_relative_hooks_path_resolves_against_the_worktree_root() {
    let root: PathBuf = fixture("doctor-locate");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir(repo.join("sub")).expect("sub");
    let context: GitContext = git_context();
    let plain: HooksLocation = locate_hooks(&context, repo.join("sub").as_path()).expect("locate");
    assert_eq!(plain.common_directory, repo.join(".git"));
    assert_eq!(plain.configured, None);
    git(repo.as_path(), &["config", "core.hooksPath", "custom-hooks"]);
    let relative: HooksLocation = locate_hooks(&context, repo.join("sub").as_path()).expect("locate");
    assert_eq!(relative.configured, Some(repo.join("custom-hooks")));
    git(repo.as_path(), &["config", "core.hooksPath", "/elsewhere/hooks"]);
    let absolute: HooksLocation = locate_hooks(&context, repo.as_path()).expect("locate");
    assert_eq!(absolute.configured, Some(PathBuf::from("/elsewhere/hooks")));
    remove(root.as_path());
}

/// A linked worktree shares the main repository's hooks directory.
#[test]
fn a_linked_worktree_reports_the_shared_common_directory() {
    let root: PathBuf = fixture("doctor-linked");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("UTF-8")]);
    let location: HooksLocation = locate_hooks(&git_context(), linked.as_path()).expect("locate");
    assert_eq!(location.common_directory, repo.join(".git"));
    remove(root.as_path());
}

/// A directory with no hooks ends with status 0 and prints nothing.
#[test]
fn a_repository_without_traditional_hooks_exits_zero_silently() {
    let root: PathBuf = fixture("doctor-clean");
    let repo: PathBuf = repository(root.as_path(), "repo");
    executable(repo.join(".git/hooks/pre-commit.sample").as_path(), b"#!/bin/sh\n");
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!((code, stdout.as_str(), stderr.as_str()), (0, "", ""));
    remove(root.as_path());
}

/// A hook in the default directory ends with status 1 and is listed by path.
#[cfg(unix)]
#[test]
fn a_default_directory_hook_is_listed_with_status_one() {
    let root: PathBuf = fixture("doctor-default");
    let repo: PathBuf = repository(root.as_path(), "repo");
    executable(repo.join(".git/hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1);
    assert_eq!(stdout, format!("{}\n", repo.join(".git/hooks/pre-commit").display()));
    assert!(stderr.contains("unsupported"), "{stderr}");
    remove(root.as_path());
}

/// Hooks in a configured directory and in the default one are all listed, the default directory first.
#[cfg(unix)]
#[test]
fn hooks_in_both_directories_are_listed_default_first() {
    let root: PathBuf = fixture("doctor-both");
    let repo: PathBuf = repository(root.as_path(), "repo");
    executable(repo.join(".git/hooks/commit-msg").as_path(), b"#!/bin/sh\n");
    std::fs::create_dir(repo.join("custom")).expect("custom");
    executable(repo.join("custom/pre-commit").as_path(), b"#!/bin/sh\n");
    git(repo.as_path(), &["config", "core.hooksPath", "custom"]);
    let (code, stdout, _stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1);
    assert_eq!(
        stdout,
        format!(
            "{}\n{}\n",
            repo.join(".git/hooks/commit-msg").display(),
            repo.join("custom/pre-commit").display()
        )
    );
    remove(root.as_path());
}

/// Outside a repository the command reports the failure with status 2 and lists nothing.
#[test]
fn outside_a_repository_the_command_fails_with_status_two() {
    let root: PathBuf = fixture("doctor-outside");
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), root.as_path()));
    assert_eq!((code, stdout.as_str()), (2, ""));
    assert!(stderr.starts_with("git cli-git doctor: "), "{stderr}");
    remove(root.as_path());
}
