//! Binary-level controls for the Git version cache:
//!  a wrapped commit records the version text of the Git it found in the user's cache directory,
//!  a later commit answers from that record without trusting more than its exact identity,
//!  and removing the record makes the next commit record it again.

use super::commit::run_commit;
use super::support::{Fixture, Observed, fixture, git, remove, repository, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A repository with one tracked file and an uncommitted edit of it.
fn edited_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    return repo;
}

/// Edit the file and commit it through the wrapper.
fn commit_edit(fixture: &Fixture, repo: &Path, content: &str) -> Observed {
    std::fs::write(repo.join("a.txt"), content.as_bytes()).expect("edit a");
    return run_commit(fixture, repo, &["commit", "--quiet", "--message=edit", "--", "a.txt"]);
}

/// The cache files under the fixture's home.
fn cache_files(fixture: &Fixture) -> Vec<PathBuf> {
    let directory: PathBuf = fixture.root.join("home/.cache/cli-git");
    let mut files: Vec<PathBuf> = std::fs::read_dir(directory)
        .map(|entries| { return entries.flatten().map(|entry| { return entry.path(); }).collect(); })
        .unwrap_or_default();
    files.sort();
    return files;
}

/// A first commit records the version text privately;
///  a record that names an unsupported release stops the next commit;
///  removing the record lets the commit run Git again and record it afresh.
#[test]
fn a_commit_records_and_then_trusts_the_version_text() {
    use std::os::unix::fs::PermissionsExt;
    let fixture: Fixture = fixture("version-cache");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    assert_eq!(cache_files(&fixture), Vec::<PathBuf>::new());

    let first: Observed = commit_edit(&fixture, repo.as_path(), "one\ntwo\n");
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let files: Vec<PathBuf> = cache_files(&fixture);
    assert_eq!(files.len(), 1, "{files:?}");
    let mode: u32 = std::fs::metadata(&files[0]).expect("recorded").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let recorded: String = String::from_utf8(std::fs::read(&files[0]).expect("recorded")).expect("UTF-8");
    assert!(recorded.starts_with("cli-git git-version cache 1\n"), "{recorded}");
    assert!(recorded.contains("\ngit version 2."), "{recorded}");

    // The record is the only source of the answer now: an old release in it stops the commit before anything runs.
    let (identity_part, _text): (&str, &str) = recorded.rsplit_once("\ngit version ").expect("text line");
    std::fs::write(&files[0], format!("{identity_part}\ngit version 2.54.0\n")).expect("tamper");
    let head_before: String = String::from_utf8_lossy(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout.as_slice()).into_owned();
    let refused: Observed = commit_edit(&fixture, repo.as_path(), "one\ntwo\nthree\n");
    assert_ne!(refused.code, Some(0));
    assert!(stderr_of(&refused).contains("2.55.0 or newer"), "{}", stderr_of(&refused));
    let head_after: String = String::from_utf8_lossy(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout.as_slice()).into_owned();
    assert_eq!(head_before, head_after);

    // Without the record Git is asked again, the commit lands and the record comes back.
    std::fs::remove_dir_all(fixture.root.join("home/.cache")).expect("remove cache");
    let again: Observed = commit_edit(&fixture, repo.as_path(), "one\ntwo\nthree\n");
    assert_eq!(again.code, Some(0), "{}", stderr_of(&again));
    assert_eq!(cache_files(&fixture).len(), 1);
    remove(&fixture);
}
