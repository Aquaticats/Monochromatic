//! What:
//!  The selection command lines landed through the service,
//! compared field by field with native Git:
//! `-i` and `--include`,
//!  `-p` and `--patch`,
//! `--interactive`,
//! and `--pathspec-from-file`,
//! and the dry run and the status formats,
//! which need
//!  neither the transaction nor the service.
//! Why:
//!  Each of these lines used to be declined to the transaction.
//! Real Git does the staging in the private index,
//! the service
//!  lands the result,
//! and the person must find the commit,
//! the index and the worktree exactly as native Git leaves them.

use super::commit::{remove_settled, wrapped_commit};
use super::service_commit::{Running, start};
use super::service_landing_support::{Case, case, compare, copy_beside, finished, text_of};
use super::support::{Fixture, direct, fixture, git, observe, repository};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn write(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("write");
}

/// Thirty numbered lines,
/// so an edit at each end is two hunks.
fn numbered() -> String {
    return (1..=30).map(|number| { return format!("line {number}\n"); }).collect();
}

/// A repository with three tracked files,
/// the first long enough to hold two hunks.
pub(super) fn base_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    write(&repo, "a.txt", numbered().as_str());
    write(&repo, "b.txt", "other\n");
    write(&repo, "c.txt", "third\n");
    git(fixture, &repo, &["add", "a.txt", "b.txt", "c.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    return repo;
}

/// Edits at both ends of the first file,
/// and a staged edit of the second.
fn two_hunks_and_a_staged_file(fixture: &Fixture, repo: &Path) {
    write(repo, "a.txt", numbered().replace("line 1\n", "line 1 changed\n").replace("line 30\n", "line 30 changed\n").as_str());
    write(repo, "b.txt", "other\nstaged\n");
    git(fixture, repo, &["add", "b.txt"]);
}

/// Each selection line commits what native Git commits and leaves what native Git leaves.
#[test]
fn include_patch_and_interactive_selections_equal_the_native_commit() {
    let fixture: Fixture = fixture("landing-selection");
    let base: PathBuf = base_repository(&fixture);
    two_hunks_and_a_staged_file(&fixture, &base);
    let name: String = format!("landing-selection-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let cases: Vec<Case> = vec![
        case("short include", &["commit", "-i", "-m", "included", "--", "a.txt"]),
        case("long include", &["commit", "--include", "-m", "included", "--", "a.txt"]),
        Case { index_commit: true, input: b"y\nn\n", ..case("short patch, first hunk only", &["commit", "-p", "-m", "patched"]) },
        Case { index_commit: true, input: b"n\ny\n", ..case("long patch, second hunk only", &["commit", "--patch", "-m", "patched"]) },
        Case { index_commit: true, input: b"y\ny\n", ..case("patch of every hunk", &["commit", "-p", "-m", "patched"]) },
        Case { input: b"y\nn\n", ..case("patch of a named path", &["commit", "-p", "-m", "patched", "--", "a.txt"]) },
        Case { index_commit: true, input: b"u\n1\n\nq\n", ..case("interactive update of one file", &["commit", "--interactive", "-m", "chosen"]) },
        case("include with a message from a file", &["commit", "-i", "-F", "../message.txt", "--", "a.txt"]),
    ];
    std::fs::write(fixture.root.join("message.txt"), "from a file\n\nbody\n").expect("message file");
    for entry in &cases {
        compare(&fixture, &base, name.as_str(), entry);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A file named by `--pathspec-from-file` is committed like the paths it lists,
/// with NUL or newline separators and from
///  standard input.
#[test]
fn a_pathspec_file_selects_the_paths_it_names() {
    let fixture: Fixture = fixture("landing-pathspec-file");
    let base: PathBuf = base_repository(&fixture);
    write(&base, "a.txt", "changed a\n");
    write(&base, "b.txt", "changed b\n");
    write(&base, "c.txt", "changed c\n");
    write(&base, "list.txt", "a.txt\nc.txt\n");
    std::fs::write(base.join("list-nul.txt"), b"b.txt\0c.txt\0").expect("nul list");
    git(&fixture, &base, &["add", "list.txt", "list-nul.txt"]);
    git(&fixture, &base, &["commit", "--quiet", "--message=lists"]);
    write(&base, "a.txt", "changed a\nagain\n");
    write(&base, "b.txt", "changed b\nagain\n");
    write(&base, "c.txt", "changed c\nagain\n");
    let name: String = format!("landing-pathspec-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let cases: Vec<Case> = vec![
        case("a list file", &["commit", "-m", "listed", "--pathspec-from-file=list.txt"]),
        case("a list file given as a separate word", &["commit", "-m", "listed", "--pathspec-from-file", "list.txt"]),
        case("a NUL-separated list file", &["commit", "-m", "listed", "--pathspec-from-file=list-nul.txt", "--pathspec-file-nul"]),
        Case { input: b"a.txt\nb.txt\n", ..case("a list from standard input", &["commit", "-m", "listed", "--pathspec-from-file=-"]) },
        Case { input: b"b.txt\0", ..case("a NUL list from standard input", &["commit", "-m", "listed", "--pathspec-from-file=-", "--pathspec-file-nul"]) },
        case("a list file with an editor-free amend", &["commit", "--amend", "--no-edit", "--pathspec-from-file=list.txt"]),
    ];
    for entry in &cases {
        compare(&fixture, &base, name.as_str(), entry);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The dry run and every status format commit nothing,
/// answer as native Git answers,
/// and need neither the transaction nor
///  the service:
/// no service is running,
/// no runtime directory is given,
/// and the repository gains no area of ours.
#[test]
fn a_dry_run_and_the_status_formats_need_neither_the_transaction_nor_the_service() {
    let fixture: Fixture = fixture("landing-dry-run");
    let base: PathBuf = base_repository(&fixture);
    write(&base, "a.txt", "changed a\n");
    write(&base, "d.txt", "untracked\n");
    git(&fixture, &base, &["add", "a.txt"]);
    let head: String = text_of(&fixture, &base, &["rev-parse", "HEAD"]);
    let lines: Vec<&[&str]> = vec![
        &["commit", "--dry-run", "-m", "x", "--", "a.txt"],
        &["commit", "--dry-run", "--no-enforce-only"],
        &["commit", "--short", "--no-enforce-only"],
        &["commit", "--short", "--branch", "--no-enforce-only"],
        &["commit", "--porcelain", "--no-enforce-only"],
        &["commit", "--long", "--no-enforce-only"],
        &["commit", "-z", "--no-enforce-only"],
        &["commit", "--porcelain", "-z", "--ahead-behind", "--no-enforce-only"],
        &["commit", "--dry-run", "--untracked-files=no", "--no-enforce-only"],
    ];
    for line in lines {
        let served_repo: PathBuf = copy_beside(&fixture, &base, "dry");
        let native_line: Vec<&str> = line.iter().copied().filter(|word| { return *word != "--no-enforce-only"; }).collect();
        let mut native = direct(&fixture);
        native.current_dir(&base).args(native_line.as_slice());
        let expected = observe(&mut native, b"");
        let mut wrapped = wrapped_commit(&fixture, &served_repo, line);
        // No runtime directory and no service: a command that reached the service would stop with a plain failure.
        wrapped.env_remove("XDG_RUNTIME_DIR");
        let found = observe(&mut wrapped, b"");
        assert_eq!(found.code, expected.code, "{line:?}: {}", String::from_utf8_lossy(&found.stderr));
        assert_eq!(found.stdout, expected.stdout, "{line:?}");
        assert_eq!(text_of(&fixture, &served_repo, &["rev-parse", "HEAD"]), head, "{line:?}: nothing was committed");
        assert!(!served_repo.join(".git/monochromatic-git-policy-cli").exists(), "{line:?}: the transaction's area was not made");
        let left: Vec<String> = super::support::left_behind(served_repo.as_path());
        assert!(left.is_empty(), "{line:?}: no scratch directory of the service was made, and these were: {left:?}");
        std::fs::remove_dir_all(&served_repo).expect("remove the copy");
    }
    assert_eq!(finished(&fixture), 0, "no landing happened");
    remove_settled(&fixture);
}
