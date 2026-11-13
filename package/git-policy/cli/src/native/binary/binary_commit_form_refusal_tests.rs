//! What:
//!  Commits whose options are a form real Git refuses,
//!  through the built wrapper and the resident service:
//!  each is refused with the status and the words the real Git of this host
//!  prints,
//!  and leaves no commit,
//!  no reference and no record of the commit service.
//! Why:
//!  Native preparation removes the tokens Git validates,
//!  so before the change these commands reached the private `git commit` as
//!  ordinary ones and made a commit real Git would have refused
//!  (finding 8 of the audit of 0.6.1-alpha.14;
//!  `-i` with `-o` from the review of the commit-argument fixes).
//!  The owner's answer of 2026-11-12 is "Refuse as Git does",
//!  with Git's own message,
//!  so every control here compares the wrapper's answer with the answer real
//!  Git gives the same command line in a repository in the same state,
//!  never with a message this program keeps.
//!  The one recorded exception stays and is the last control in this file:
//!  a commit that names an untracked path commits it
//!  (`doc/decision/cli-git-policies-platform.md`,
//!  "Transactional commit gate").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const real = spawnSync('/usr/bin/git', ['commit', '-i', '-o', '-m', 'x', '--', 'a.txt'], { cwd: twin });
//! const wrapped = throughService(['commit', '-i', '-o', '-m', 'x', '--', 'a.txt']);
//! expect(wrapped).toEqual(real); expect(head).toBe(headBefore);
//! ```

use super::commit::{git_text, remove_settled};
use super::service_commit::{Running, start, through_service};
use super::support::{Fixture, direct, fixture, git, repository};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;

/// What a refusal must leave as it was:
///  the commit `HEAD` names,
///  how many commits there are,
///  every reference,
///  whether the commit's timing record was written,
///  and what the service's journal holds.
/// Why:
///   "No commit,
/// no reference and no record" is one statement about all of these,
///  and a refusal that staged a path or reached the service would move one of them.
#[derive(Debug, Eq, PartialEq)]
struct State {
    /// The commit `HEAD` points at.
    head: String,
    /// The number of commits `HEAD` reaches.
    commits: String,
    /// Every reference with the object it names.
    refs: String,
    /// Whether the commit's timing record exists.
    timing_record: bool,
    /// Every name under the service's journal directory.
    journal: Vec<String>,
}

/// The names under `directory`,
/// relative to it,
/// sorted.
fn names_under(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let name: String = path.strip_prefix(directory).unwrap_or(path.as_path()).display().to_string();
        found.push(name);
        if path.is_dir() {
            for inner in names_under(path.as_path()) {
                found.push(format!("{}/{inner}", entry.file_name().display()));
            }
        }
    }
    found.sort();
    return found;
}

/// The state of one repository and the fixture's journal.
fn state(fixture: &Fixture, repo: &Path) -> State {
    return State {
        head: git_text(fixture, repo, &["rev-parse", "HEAD"]).trim().to_owned(),
        commits: git_text(fixture, repo, &["rev-list", "--count", "HEAD"]).trim().to_owned(),
        refs: git_text(fixture, repo, &["for-each-ref", "--format=%(refname) %(objectname)"]),
        timing_record: repo.join(".git/monochromatic-git-policy-cli-commit-timing.jsonl").exists(),
        journal: names_under(fixture.root.join("journal").as_path()),
    };
}

/// A repository with one commit and one tracked file whose edit is staged,
///  so every row below has a path to name and a `HEAD` to amend or to fix up.
fn a_staged_change(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), "one\ntwo\n").expect("edit a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    return repo;
}

/// What:
///  Run one commit command line as real Git and through the wrapper in two
///  repositories in the same state,
///  and require the same status and the same words,
///  with nothing changed in the wrapped one.
/// Why:
///   The comparison is what "Git's own message" means:
///  a release that rewords a message changes both sides together,
///  and a wrapper that answered anything of its own fails here.
fn refuses_as_git_does(fixture_name: &str, arguments: &[&str]) {
    let fixture: Fixture = fixture(fixture_name);
    let real_repo: PathBuf = a_staged_change(&fixture, "real");
    let wrapped_repo: PathBuf = a_staged_change(&fixture, "wrapped");
    std::fs::write(wrapped_repo.join("paths.txt"), "a.txt\n").expect("pathspecs");
    std::fs::write(real_repo.join("paths.txt"), "a.txt\n").expect("pathspecs");
    let name: String = format!("{fixture_name}-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let before: State = state(&fixture, wrapped_repo.as_path());
    let mut git_command: std::process::Command = direct(&fixture);
    git_command.current_dir(real_repo.as_path());
    let real: Output = git_command.args(arguments).output().expect("real Git runs");
    let wrapped: Output = through_service(&fixture, wrapped_repo.as_path(), name.as_str(), arguments);
    assert_eq!(
        wrapped.status.code(),
        real.status.code(),
        "the status real Git answered {arguments:?} with; Git said: {}; the wrapper said: {}",
        String::from_utf8_lossy(real.stderr.as_slice()),
        String::from_utf8_lossy(wrapped.stderr.as_slice())
    );
    assert_eq!(
        String::from_utf8_lossy(wrapped.stderr.as_slice()),
        String::from_utf8_lossy(real.stderr.as_slice()),
        "the words of {arguments:?}"
    );
    assert_ne!(real.status.code(), Some(0), "the row must be one real Git refuses: {arguments:?}");
    assert_eq!(state(&fixture, wrapped_repo.as_path()), before, "nothing changed: no commit, no reference, no record");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// `-i` with `-o`:
/// the row the review of the commit-argument fixes found.
#[test]
fn include_with_only_is_refused() {
    refuses_as_git_does("commit-form-include-only", &["commit", "-i", "-o", "-m", "x", "--", "a.txt"]);
}

/// The long spellings of the same row,
///  which a transform could miss where it read only the short ones.
#[test]
fn include_with_only_is_refused_in_long_spellings() {
    refuses_as_git_does(
        "commit-form-include-only-long",
        &["commit", "--include", "--only", "--message", "x", "--", "a.txt"],
    );
}

/// `-i` with `-a`:
/// one of the rows the audit found.
#[test]
fn include_with_all_is_refused() {
    refuses_as_git_does("commit-form-include-all", &["commit", "-i", "-a", "-m", "x", "--", "a.txt"]);
}

/// `-i`,
/// `-o` and `-a` together:
///  Git names all three,
///  so the wrapper must print that whole sentence.
#[test]
fn include_with_only_and_all_is_refused() {
    refuses_as_git_does("commit-form-include-only-all", &["commit", "-i", "-o", "-a", "-m", "x", "--", "a.txt"]);
}

/// `-a` with `--interactive`:
///  the selection count Git makes treats the two interactive spellings as one.
#[test]
fn all_with_interactive_is_refused() {
    refuses_as_git_does("commit-form-all-interactive", &["commit", "-a", "--interactive"]);
}

/// `--fixup=reword:` with a path.
#[test]
fn rewording_with_a_path_is_refused() {
    refuses_as_git_does("commit-form-reword-path", &["commit", "--fixup=reword:HEAD", "--", "a.txt"]);
}

/// `--fixup=reword:` with `-a`.
#[test]
fn rewording_with_all_is_refused() {
    refuses_as_git_does("commit-form-reword-all", &["commit", "--fixup=reword:HEAD", "-a"]);
}

/// `--fixup=reword:` with `-i`.
#[test]
fn rewording_with_include_is_refused() {
    refuses_as_git_does("commit-form-reword-include", &["commit", "--fixup=reword:HEAD", "-i"]);
}

/// `--fixup=reword:` with `--patch`.
#[test]
fn rewording_with_patch_is_refused() {
    refuses_as_git_does("commit-form-reword-patch", &["commit", "--fixup=reword:HEAD", "--patch"]);
}

/// `--fixup=reword:` with `--interactive`.
#[test]
fn rewording_with_interactive_is_refused() {
    refuses_as_git_does("commit-form-reword-interactive", &["commit", "--fixup=reword:HEAD", "--interactive"]);
}

/// `--pathspec-from-file` with pathspec arguments.
#[test]
fn a_pathspec_file_with_paths_is_refused() {
    refuses_as_git_does(
        "commit-form-pathspec-file-with-paths",
        &["commit", "--pathspec-from-file=paths.txt", "-m", "x", "--", "a.txt"],
    );
}

/// What:
///  The recorded exception still commits:
///  a commit that names an untracked path records it,
///  where real Git refuses the pathspec.
/// Why:
///  `doc/decision/cli-git-policies-platform.md`,
///  "Transactional commit gate",
/// records the selection of untracked paths as chosen,
///  and the owner's answer of 2026-11-12 keeps it.
///  A refusal here would take away what an explicit-path commit is for.
#[test]
fn a_commit_that_names_an_untracked_path_still_commits() {
    let fixture: Fixture = fixture("commit-form-untracked-path");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("u.txt"), "untracked\n").expect("u");
    let name: String = format!("commit-form-untracked-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let before: String = git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim().to_owned();
    let committed: Output = through_service(&fixture, repo.as_path(), name.as_str(), &["commit", "-m", "x", "--", "u.txt"]);
    assert_eq!(
        committed.status.code(),
        Some(0),
        "the recorded exception commits: {}",
        String::from_utf8_lossy(committed.stderr.as_slice())
    );
    let after: String = git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim().to_owned();
    assert_ne!(after, before, "the commit landed");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "--name-only", "--format=", "HEAD"]).trim(), "u.txt");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
