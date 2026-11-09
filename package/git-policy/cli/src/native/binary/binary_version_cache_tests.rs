//! Binary-level controls for the Git version cache:
//!  a wrapped commit records the version text of the Git it found in the user's cache directory,
//!  a later commit answers from that record without trusting more than its exact identity (an old release in it only
//!  draws the warning),
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

/// The version records under the fixture's home.
///  The cache directory holds one file per kind the wrapper learned for that Git,
/// and a control about the version
///  record names its kind rather than counting every other kind beside it:
/// a host whose Git reports a system attributes file
///  makes the wrapper record that too.
fn version_records(fixture: &Fixture) -> Vec<PathBuf> {
    return cache_files(fixture)
        .into_iter()
        .filter(|path: &PathBuf| { return path.file_name().is_some_and(|name| { return name.to_string_lossy().starts_with("git-version-"); }); })
        .collect();
}

/// The cache files under the fixture's home.
fn cache_files(fixture: &Fixture) -> Vec<PathBuf> {
    let directory: PathBuf = fixture.root.join("home/.cache/monochromatic-git-policy-cli");
    let mut files: Vec<PathBuf> = std::fs::read_dir(directory)
        .map(|entries| { return entries.flatten().map(|entry| { return entry.path(); }).collect(); })
        .unwrap_or_default();
    files.sort();
    return files;
}

/// A first commit records the version text privately;
///  a record that names a release before the minimum warns once and the next commit still lands;
///  removing the record lets the commit run Git again and record it afresh.
#[test]
fn a_commit_records_and_then_trusts_the_version_text() {
    use std::os::unix::fs::PermissionsExt;
    let fixture: Fixture = fixture("version-cache");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    assert_eq!(cache_files(&fixture), Vec::<PathBuf>::new());

    let first: Observed = commit_edit(&fixture, repo.as_path(), "one\ntwo\n");
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let files: Vec<PathBuf> = version_records(&fixture);
    assert_eq!(files.len(), 1, "{files:?}");
    let mode: u32 = std::fs::metadata(&files[0]).expect("recorded").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let recorded: String = String::from_utf8(std::fs::read(&files[0]).expect("recorded")).expect("UTF-8");
    assert!(recorded.starts_with("monochromatic-git-policy-cli git-version cache 1\n"), "{recorded}");
    assert!(recorded.contains("\ngit version 2."), "{recorded}");

    // The record is the only source of the answer now: an old release in it draws the warning and the commit goes on.
    let (identity_part, _text): (&str, &str) = recorded.rsplit_once("\ngit version ").expect("text line");
    std::fs::write(&files[0], format!("{identity_part}\ngit version 2.54.0\n")).expect("tamper");
    let warned: Observed = commit_edit(&fixture, repo.as_path(), "one\ntwo\nthree\n");
    assert_eq!(warned.code, Some(0), "{}", stderr_of(&warned));
    assert_eq!(stderr_of(&warned).matches("2.54.0").count(), 1, "{}", stderr_of(&warned));
    assert!(stderr_of(&warned).contains("continues"), "{}", stderr_of(&warned));
    assert!(!stderr_of(&first).contains("older than"), "{}", stderr_of(&first));

    // Without the record Git is asked again, the commit lands and the record comes back.
    std::fs::remove_dir_all(fixture.root.join("home/.cache")).expect("remove cache");
    let again: Observed = commit_edit(&fixture, repo.as_path(), "one\ntwo\nthree\nfour\n");
    assert_eq!(again.code, Some(0), "{}", stderr_of(&again));
    assert_eq!(version_records(&fixture).len(), 1, "the record comes back once, whatever else the cache learned");
    remove(&fixture);
}
