//! What:
//!  Disposable-repository controls for repository and worktree identity.
//! Why:
//!  Configuration location and worktree policies depend on what real Git 2.56.0
//!      reports for main,
//!  linked,
//!  bare,
//!  absent and option-selected repositories.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await resolveWorktreeIdentity(git, ['-C', linked], [])).toMatchObject({ kind: 'linked-worktree' });
//! ```
#![cfg(unix)]

/// Import the identity functions under test and shared fixtures.
use super::{WorktreeIdentity, parse_identity_output, resolve_worktree_identity, worktree_root};
use crate::test_support::{REAL_GIT, fixture, git, remove, repository};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

/// Build an owned argument list from path and text parts.
fn prefix(parts: &[&Path]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for part in parts {
        result.push(part.as_os_str().to_os_string());
    }
    return result;
}

/// Resolve identity through real Git with the given global prefix.
fn identity(parts: &[&Path]) -> WorktreeIdentity {
    return resolve_worktree_identity(Path::new(REAL_GIT), prefix(parts).as_slice(), &[])
        .expect("identity query");
}

/// Four lines on success are a worktree:
///  main when both directories are equal,
///  linked otherwise.
#[test]
fn successful_output_is_a_main_or_linked_worktree() {
    assert_eq!(
        parse_identity_output(true, b"false\n/r/.git\n/r/.git\n/r\n").expect("main"),
        WorktreeIdentity::MainWorktree {
            common_dir: PathBuf::from("/r/.git"),
            git_dir: PathBuf::from("/r/.git"),
            worktree_root: PathBuf::from("/r"),
        }
    );
    assert_eq!(
        parse_identity_output(true, b"false\n/r/.git/worktrees/w\n/r/.git\n/w\n").expect("linked"),
        WorktreeIdentity::LinkedWorktree {
            common_dir: PathBuf::from("/r/.git"),
            git_dir: PathBuf::from("/r/.git/worktrees/w"),
            worktree_root: PathBuf::from("/w"),
        }
    );
    // CRLF line endings and non-UTF-8 path bytes are carried exactly.
    assert_eq!(
        parse_identity_output(true, b"false\r\n/r\xff/.git\r\n/r\xff/.git\r\n/r\xff\r\n")
            .expect("CRLF"),
        WorktreeIdentity::MainWorktree {
            common_dir: PathBuf::from(OsString::from_vec(b"/r\xff/.git".to_vec())),
            git_dir: PathBuf::from(OsString::from_vec(b"/r\xff/.git".to_vec())),
            worktree_root: PathBuf::from(OsString::from_vec(b"/r\xff".to_vec())),
        }
    );
}

/// A failed query is "outside" with no output,
///  bare after three lines saying so,
///  and outside inside `.git`.
#[test]
fn failed_output_distinguishes_absent_bare_and_git_directory() {
    assert_eq!(
        parse_identity_output(false, b"").expect("outside"),
        WorktreeIdentity::OutsideWorktree
    );
    assert_eq!(
        parse_identity_output(false, b"true\n/b.git\n/b.git\n").expect("bare"),
        WorktreeIdentity::BareRepository {
            common_dir: PathBuf::from("/b.git"),
            git_dir: PathBuf::from("/b.git"),
        }
    );
    assert_eq!(
        parse_identity_output(false, b"false\n/r/.git\n/r/.git\n").expect("inside .git"),
        WorktreeIdentity::OutsideWorktree
    );
}

/// Output shapes Git never produces for this query are rejected instead of guessed.
#[test]
fn unexpected_output_shapes_are_rejected() {
    for (success, stdout) in [
        (true, b"".as_slice()),
        (true, b"false\n/r/.git\n/r/.git\n"),
        (true, b"false\n/r/.git\n/r/.git\n/r\nextra\n"),
        (true, b"false\n/r/.git\n/r/.git\n/a\nb\n"),
        (true, b"maybe\n/r/.git\n/r/.git\n/r\n"),
        (true, b"\n/r/.git\n/r/.git\n/r\n"),
        (true, b"false\n\n/r/.git\n/r\n"),
        (true, b"false\n/r/.git\n\n/r\n"),
        (true, b"false\n/r/.git\n/r/.git\n\n"),
        (false, b"true\n"),
        (false, b"true\n/b.git\n"),
        (false, b"true\n/b.git\n/b.git\n/extra\n"),
        (false, b"TRUE\n/b.git\n/b.git\n"),
        (false, b"true\n\n/b.git\n"),
    ] {
        let error = parse_identity_output(success, stdout).expect_err("malformed");
        assert!(
            error.to_string().starts_with(
                "cli-git could not classify the Git worktree: git rev-parse returned "
            ),
            "{error}"
        );
    }
}

/// Real Git reports main and linked worktrees with canonical paths,
///  from any directory inside them.
#[test]
fn native_git_identifies_main_and_linked_worktrees() {
    let root: PathBuf = fixture("identity-worktrees");
    let main: PathBuf = repository(root.as_path(), "main");
    let linked: PathBuf = root.join("linked");
    git(
        main.as_path(),
        &[
            OsString::from("worktree"),
            OsString::from("add"),
            OsString::from("--quiet"),
            OsString::from("-b"),
            OsString::from("topic"),
            linked.clone().into_os_string(),
        ],
    );
    std::fs::create_dir(main.join("nested")).expect("nested directory");
    let expected_main: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: main.join(".git"),
        git_dir: main.join(".git"),
        worktree_root: main.clone(),
    };
    assert_eq!(identity(&[Path::new("-C"), main.as_path()]), expected_main);
    assert_eq!(
        identity(&[Path::new("-C"), main.join("nested").as_path()]),
        expected_main
    );
    assert_eq!(worktree_root(&expected_main), Some(main.as_path()));
    let expected_linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: main.join(".git"),
        git_dir: main.join(".git/worktrees/linked"),
        worktree_root: linked.clone(),
    };
    assert_eq!(
        identity(&[Path::new("-C"), linked.as_path()]),
        expected_linked
    );
    assert_eq!(worktree_root(&expected_linked), Some(linked.as_path()));
    remove(root.as_path());
}

/// Global options select the repository exactly as Git applies them:
///  chained `-C`,
///  symbolic links,
///  `--git-dir`.
#[test]
fn native_git_applies_global_selection_options() {
    let root: PathBuf = fixture("identity-options");
    let main: PathBuf = repository(root.as_path(), "main");
    std::fs::create_dir(main.join("nested")).expect("nested directory");
    std::os::unix::fs::symlink(main.join("nested"), root.join("link")).expect("symlink");
    let expected: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: main.join(".git"),
        git_dir: main.join(".git"),
        worktree_root: main.clone(),
    };
    // `link/..` through a symbolic link is the linked directory's real parent, the repository.
    assert_eq!(
        identity(&[
            Path::new("-C"),
            root.as_path(),
            Path::new("-C"),
            Path::new("link/.."),
            Path::new("-C"),
            Path::new(""),
        ]),
        expected
    );
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(&elsewhere).expect("work tree directory");
    assert_eq!(
        identity(&[
            Path::new("--git-dir"),
            main.join(".git").as_path(),
            Path::new("--work-tree"),
            elsewhere.as_path(),
        ]),
        WorktreeIdentity::MainWorktree {
            common_dir: main.join(".git"),
            git_dir: main.join(".git"),
            worktree_root: elsewhere.clone(),
        }
    );
    remove(root.as_path());
}

/// Bare repositories,
///  the inside of `.git`,
///  non-repositories and missing directories have no worktree.
#[test]
fn native_git_reports_locations_without_a_worktree() {
    let root: PathBuf = fixture("identity-none");
    let main: PathBuf = repository(root.as_path(), "main");
    let bare: PathBuf = root.join("bare.git");
    git(
        root.as_path(),
        &[
            OsString::from("init"),
            OsString::from("--quiet"),
            OsString::from("--bare"),
            bare.clone().into_os_string(),
        ],
    );
    let bare_identity: WorktreeIdentity = identity(&[Path::new("-C"), bare.as_path()]);
    assert_eq!(
        bare_identity,
        WorktreeIdentity::BareRepository {
            common_dir: bare.clone(),
            git_dir: bare.clone(),
        }
    );
    assert_eq!(worktree_root(&bare_identity), None);
    assert_eq!(
        identity(&[Path::new("-C"), main.join(".git").as_path()]),
        WorktreeIdentity::OutsideWorktree
    );
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    assert_eq!(
        identity(&[Path::new("-C"), plain.as_path()]),
        WorktreeIdentity::OutsideWorktree
    );
    assert_eq!(
        identity(&[Path::new("-C"), root.join("missing").as_path()]),
        WorktreeIdentity::OutsideWorktree
    );
    assert_eq!(worktree_root(&WorktreeIdentity::OutsideWorktree), None);
    remove(root.as_path());
}

/// A repository whose path is not UTF-8 is identified by its exact bytes.
#[test]
fn native_git_identifies_non_utf8_repository_paths() {
    let root: PathBuf = fixture("identity-bytes");
    let name: OsString = OsString::from_vec(b"repo-\xff\xfe".to_vec());
    let created: PathBuf = repository(root.as_path(), "placeholder");
    let renamed: PathBuf = root.join(&name);
    std::fs::rename(&created, &renamed).expect("rename to non-UTF-8 name");
    assert_eq!(
        identity(&[Path::new("-C"), renamed.as_path()]),
        WorktreeIdentity::MainWorktree {
            common_dir: renamed.join(".git"),
            git_dir: renamed.join(".git"),
            worktree_root: renamed.clone(),
        }
    );
    remove(root.as_path());
}

/// A program that cannot be started is an error,
///  never "outside a worktree".
#[test]
fn unstartable_git_is_an_error() {
    assert_eq!(
        resolve_worktree_identity(Path::new("/nonexistent-directory/git"), &[], &[])
            .expect_err("missing program")
            .kind(),
        std::io::ErrorKind::NotFound
    );
}
