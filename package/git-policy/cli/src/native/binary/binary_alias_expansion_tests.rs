//! Binary-level controls for the command an alias really runs:
//!  the wrapper hands real Git the expansion it resolved,
//!  so that expansion must be the command Git itself would run.
//! Each control is differential:
//!  the same invocation goes to real Git started directly,
//!  which is what Git does,
//!  and to the wrapper,
//!  and the uncommitted work the alias would discard must survive both.
//!
//! The audit that found the divergences:
//!  `doc/planning/cli-git-invocation-and-argument-route-findings-on-main.md`.

use super::support::{Fixture, Observed, direct, executable, fixture, git, observe, remove, repository, run_wrapped, stderr_of, wrapped};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The tracked file content a person has not committed.
const USER_DATA: &[u8] = b"USER DATA\n";

/// What a repository looks like before the divergence decides anything:
///  one commit holding `tracked.txt`,
///  and the same file overwritten in the worktree without being staged.
fn repository_with_unstaged_work(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("tracked.txt"), b"BASE\n").expect("baseline file");
    git(fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=baseline"]);
    std::fs::write(repo.join("tracked.txt"), USER_DATA).expect("the person's work");
    return repo;
}

/// The tracked file of a repository, as it is on disk.
fn tracked(repo: &Path) -> Vec<u8> {
    return std::fs::read(repo.join("tracked.txt")).expect("tracked file");
}

/// What an alias that changes the directory must not do:
///  Git refuses the alias before it runs anything,
///  so the other repository keeps the work it holds.
#[test]
fn a_directory_changing_alias_is_refused_instead_of_resetting_another_repository() {
    let fixture: Fixture = fixture("alias-expansion-directory");
    let source: PathBuf = repository_with_unstaged_work(&fixture, "source");
    let victim: PathBuf = repository_with_unstaged_work(&fixture, "victim");
    let alias: String = format!("alias.audit=-C {} reset --hard", victim.display());
    let arguments: [&str; 3] = ["-c", alias.as_str(), "audit"];
    // Git itself refuses this alias, so the same invocation run directly leaves the work alone.
    let direct_run: Observed = observe(direct(&fixture).current_dir(source.as_path()).args(arguments), b"");
    assert_ne!(direct_run.code, Some(0), "direct Git accepted the alias: {}", stderr_of(&direct_run));
    assert_eq!(tracked(victim.as_path()), USER_DATA, "direct Git reset the other repository");
    let observed: Observed = run_wrapped(&fixture, source.as_path(), arguments.as_slice());
    assert_eq!(tracked(victim.as_path()), USER_DATA, "the wrapper reset the other repository: {}", stderr_of(&observed));
    assert_eq!(tracked(source.as_path()), USER_DATA, "the wrapper reset its own repository");
    assert_ne!(observed.code, Some(0), "the wrapper ran a command Git refuses");
    // The caller reads Git's own words, not a refusal the wrapper invented.
    assert!(stderr_of(&observed).contains("changes environment variables"), "{}", stderr_of(&observed));
    remove(&fixture);
}

/// What an alias that shadows an external command must not do:
///  Git runs the external command,
///  so the alias never runs and the work stays.
#[test]
fn an_existing_external_command_wins_over_an_alias_of_its_name() {
    let fixture: Fixture = fixture("alias-expansion-external");
    let repo: PathBuf = repository_with_unstaged_work(&fixture, "repo");
    let external_directory: PathBuf = fixture.root.join("exec");
    std::fs::create_dir(&external_directory).expect("external command directory");
    executable(
        external_directory.join("git-audit-external").as_path(),
        b"#!/bin/sh\necho 'EXTERNAL: no mutation'\n",
    );
    let exec_path: OsString = external_directory.into_os_string();
    let arguments: [&str; 3] = ["-c", "alias.audit-external=reset --hard", "audit-external"];
    let direct_run: Observed = observe(
        direct(&fixture)
            .current_dir(repo.as_path())
            .args(arguments)
            .env("GIT_EXEC_PATH", exec_path.as_os_str()),
        b"",
    );
    let direct_stdout: String = String::from_utf8_lossy(&direct_run.stdout).into_owned();
    assert!(direct_stdout.contains("EXTERNAL: no mutation"), "the positive control did not run: {direct_stdout}");
    let observed: Observed = observe(
        wrapped(&fixture)
            .current_dir(repo.as_path())
            .args(arguments)
            .env("GIT_EXEC_PATH", exec_path.as_os_str()),
        b"",
    );
    assert!(
        String::from_utf8_lossy(&observed.stdout).contains("EXTERNAL: no mutation"),
        "the wrapper ran the alias instead of the external command: {}",
        stderr_of(&observed)
    );
    assert_eq!(tracked(repo.as_path()), USER_DATA, "the wrapper ran the alias's reset");
    remove(&fixture);
}

/// What a malformed alias must not do:
///  Git keeps the empty first word its split produced and fails,
///  so nothing runs and the work stays.
#[test]
fn an_alias_value_that_starts_with_a_space_fails_instead_of_resetting() {
    let fixture: Fixture = fixture("alias-expansion-trim");
    let repo: PathBuf = repository_with_unstaged_work(&fixture, "repo");
    let arguments: [&str; 3] = ["-c", "alias.audit= reset --hard", "audit"];
    let direct_run: Observed = observe(direct(&fixture).current_dir(repo.as_path()).args(arguments), b"");
    assert_ne!(direct_run.code, Some(0), "direct Git accepted the alias: {}", stderr_of(&direct_run));
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
    assert_eq!(tracked(repo.as_path()), USER_DATA, "the wrapper trimmed the value into a reset");
    assert_ne!(observed.code, Some(0), "the wrapper ran a command Git refuses");
    assert!(stderr_of(&observed).contains("is not a git command"), "{}", stderr_of(&observed));
    remove(&fixture);
}
