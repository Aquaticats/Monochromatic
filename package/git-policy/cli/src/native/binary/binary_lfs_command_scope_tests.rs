//! What:
//!  The configuration a person gives on the command line reaches the `git-lfs` upload that precedes a push,
//! so an object travels to the remote the ref is published to.
//! Why:
//!  A push publishes pointer files,
//! and a pointer whose object sits on another server is a published pointer no collaborator can fetch.
//! The fixture has two bare remotes with their own file-backed LFS stores:
//! `origin`, which the push fetches from,
//! and a second remote named only by a command-scoped push URL.
//! The wrapped push publishes the pointer commit to that second remote,
//! so its object must be stored there,
//! and a positive control first shows that the fixture's ordinary upload does reach a remote's store.
//! Hosts without `git-lfs` on PATH report that and check nothing;
//! the stand-in controls in
//!  `binary_lfs_pre_push_tests.rs` cover the same upload paths there.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, direct, fixture, git, observe, repository, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The variable the `--config-env` control names its push URL through.
const PUSH_URL_VARIABLE: &str = "FIXTURE_PUSH_URL";

/// What:
///  The real `git-lfs` this process's `PATH` finds,
/// linked into the fixture's `bin` directory,
/// or nothing with the line that says the check was skipped.
/// Why:
///   The fixture's `PATH` holds only the wrapper's own `bin` and the system directories,
/// so the upload finds a program there or reports that none is installed.
fn link_git_lfs(fixture: &Fixture) -> Option<PathBuf> {
    let Some(path) = std::env::var_os("PATH") else {
        eprintln!("skipped: PATH names no directory, so the real upload was not exercised");
        return None;
    };
    let program: Option<PathBuf> = std::env::split_paths(&path)
        .map(|directory| { return directory.join("git-lfs"); })
        .find(|candidate| { return candidate.is_file(); });
    let Some(installed) = program else {
        eprintln!("skipped: git-lfs is not on PATH, so the real upload was not exercised");
        return None;
    };
    std::os::unix::fs::symlink(installed, fixture.root.join("bin/git-lfs")).expect("git-lfs link");
    return Some(fixture.root.join("bin/git-lfs"));
}

/// What:
///  A repository whose `origin` fetches from one bare remote,
/// and a second bare remote that only a command-scoped push URL names,
/// with the repository configured to use Git LFS.
/// Why:
///   The two stores are separate on the filesystem,
/// so where an object landed is read from the files and not inferred.
fn two_remotes(fixture: &Fixture) -> Option<(PathBuf, PathBuf, PathBuf)> {
    link_git_lfs(fixture)?;
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let fetch: PathBuf = fixture.root.join("fetch.git");
    let push: PathBuf = fixture.root.join("push.git");
    for remote in [&fetch, &push] {
        let name: String = remote.file_name().expect("a name").to_string_lossy().into_owned();
        git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", name.as_str()]);
    }
    git(fixture, repo.as_path(), &["remote", "add", "origin", fetch.to_str().expect("UTF-8")]);
    // The wrapper uploads before a push only for a repository whose own configuration holds an LFS key.
    git(fixture, repo.as_path(), &["config", "lfs.repositoryformatversion", "0"]);
    return Some((repo, fetch, push));
}

/// What:
///  Commit a pointer file named `name`,
/// whose object the repository's own LFS store holds,
/// and answer the object's name.
/// Why:
///   The object is what the upload moves;
/// the pointer is what the push publishes,
/// and the fixture therefore needs no clean filter,
/// no `.gitattributes` and no LFS server.
fn commit_pointer(fixture: &Fixture, repo: &Path, name: &str) -> String {
    let content: Vec<u8> = format!("{name} LFS content\n").into_bytes();
    let digest: [u8; 32] = git_policy_cli::subvolume_sha::sha256(content.as_slice());
    let oid: String = git_policy_cli::hexadecimal_of_bytes::hexadecimal_of_bytes(digest.as_slice());
    let pointer: String = format!(
        "version https://git-lfs.github.com/spec/v1\noid sha256:{oid}\nsize {}\n",
        content.len()
    );
    std::fs::write(repo.join(name), pointer.as_bytes()).expect("pointer file");
    let storage: PathBuf = repo.join(".git/lfs/objects").join(&oid[..2]).join(&oid[2..4]);
    std::fs::create_dir_all(&storage).expect("LFS object directory");
    std::fs::write(storage.join(oid.as_str()), content.as_slice()).expect("LFS object");
    git(fixture, repo, &["add", "--", name]);
    git(fixture, repo, &["commit", "--quiet", &format!("--message={name}")]);
    return oid;
}

/// Whether the bare remote's file-backed LFS store holds the object named `oid`.
fn stored(remote: &Path, oid: &str) -> bool {
    return remote.join("lfs/objects").join(&oid[..2]).join(&oid[2..4]).join(oid).is_file();
}

/// The remote's `refs/heads/main`,
/// or `absent`.
fn remote_main(fixture: &Fixture, remote: &Path) -> String {
    let output = direct(fixture)
        .arg(format!("--git-dir={}", remote.display()))
        .args(["rev-parse", "--verify", "--quiet", "refs/heads/main"])
        .output()
        .expect("rev-parse starts");
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return if text.is_empty() { String::from("absent") } else { text };
}

/// What:
///  A push whose URL is selected on the command line publishes the pointer to that remote
/// with its object already there.
/// Why:
///   The invocation's overlay adds a numbered configuration pair of its own,
/// and a caller's `-c` pair that took the same slot never reached
///  `git-lfs`:
/// the push moved the second remote's `main` while the object stayed in the fetch remote's store.
#[test]
fn a_command_scoped_push_url_carries_the_upload_to_the_push_remote() {
    let fixture: Fixture = fixture("lfs-command-scope-pushurl");
    let Some((repo, fetch, push)) = two_remotes(&fixture) else {
        return;
    };
    let first: String = commit_pointer(&fixture, repo.as_path(), "first.bin");
    let control: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["push", "--quiet", "origin", "main"]), b"");
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    assert!(stored(fetch.as_path(), first.as_str()), "the positive control stored the first object in the fetch remote: {}", stderr_of(&control));
    let second: String = commit_pointer(&fixture, repo.as_path(), "second.bin");
    let selection: String = format!("remote.origin.pushurl={}", push.display());
    let pushed: Observed = observe(
        &mut wrapped_commit(&fixture, repo.as_path(), &["-c", selection.as_str(), "push", "--quiet", "origin", "main"]),
        b"",
    );
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, Some(0), "{stderr}");
    assert_eq!(remote_main(&fixture, push.as_path()), git_text(&fixture, repo.as_path(), &["rev-parse", "main"]).trim(), "the ref moved to the push remote: {stderr}");
    assert!(stored(push.as_path(), second.as_str()), "the object of the pointer is in the push remote's store: {stderr}");
    assert!(!stored(fetch.as_path(), second.as_str()), "no object of the selected push reached the fetch remote: {stderr}");
    remove_settled(&fixture);
}

/// What:
///  The same push, with the second remote named by `--config-env` instead of `-c`.
/// Why:
///   An environment-named configuration is the second way to select a push URL,
/// and a translation that covered only `-c` would
///  leave this one uploading to the fetch remote.
#[test]
fn a_push_url_named_by_the_environment_carries_the_upload_to_the_push_remote() {
    let fixture: Fixture = fixture("lfs-command-scope-config-env");
    let Some((repo, fetch, push)) = two_remotes(&fixture) else {
        return;
    };
    let first: String = commit_pointer(&fixture, repo.as_path(), "first.bin");
    let control: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["push", "--quiet", "origin", "main"]), b"");
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    assert!(stored(fetch.as_path(), first.as_str()), "the positive control stored the first object in the fetch remote: {}", stderr_of(&control));
    let second: String = commit_pointer(&fixture, repo.as_path(), "second.bin");
    let selection: String = format!("--config-env=remote.origin.pushurl={PUSH_URL_VARIABLE}");
    let mut command = wrapped_commit(&fixture, repo.as_path(), &[selection.as_str(), "push", "--quiet", "origin", "main"]);
    command.env(PUSH_URL_VARIABLE, push.as_os_str());
    let pushed: Observed = observe(&mut command, b"");
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, Some(0), "{stderr}");
    assert_eq!(remote_main(&fixture, push.as_path()), git_text(&fixture, repo.as_path(), &["rev-parse", "main"]).trim(), "the ref moved to the push remote: {stderr}");
    assert!(stored(push.as_path(), second.as_str()), "the object of the pointer is in the push remote's store: {stderr}");
    assert!(!stored(fetch.as_path(), second.as_str()), "no object of the selected push reached the fetch remote: {stderr}");
    remove_settled(&fixture);
}
