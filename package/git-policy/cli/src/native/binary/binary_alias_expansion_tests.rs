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

/// The tracked file of a repository,
/// as it is on disk.
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

/// What:
///  A shell alias that names one plain `git` command is read by the guard,
///  so the discard guard refuses it and the
///       work stays.
/// Why:
///  Git runs the shell with its own executable directory first on `PATH`,
///  so the `git` inside such an
///       alias is real Git and the wrapper never sees that command start;
///  without the reading the prefix
///       `reset --hard` discards the person work unguarded,
///  and `git by the word Git will run` would have a
///       hole a one line alias opens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // real Git: tracked.txt becomes BASE; the wrapper: refused, tracked.txt stays USER DATA
/// ```
#[test]
fn a_shell_alias_that_names_one_git_command_is_refused_like_the_command() {
    let fixture: Fixture = fixture("alias-expansion-shell");
    let refused: PathBuf = repository_with_unstaged_work(&fixture, "refused");
    let control: PathBuf = repository_with_unstaged_work(&fixture, "control");
    let arguments: [&str; 3] = ["-c", "alias.wipe=!git reset --hard", "wipe"];
    // The positive control proves the alias discards the work when nothing reads it.
    let direct_run: Observed = observe(direct(&fixture).current_dir(control.as_path()).args(arguments), b"");
    assert_eq!(direct_run.code, Some(0), "direct Git failed: {}", stderr_of(&direct_run));
    assert_eq!(tracked(control.as_path()), b"BASE\n", "the positive control discarded nothing");
    let observed: Observed = run_wrapped(&fixture, refused.as_path(), arguments.as_slice());
    assert_ne!(observed.code, Some(0), "the wrapper ran the shell alias's reset: {}", stderr_of(&observed));
    assert_eq!(tracked(refused.as_path()), USER_DATA, "the shell alias discarded the work");
    remove(&fixture);
}

/// What:
///  The caller's own arguments are what the shell of a `!git` alias runs,
///  so an alias that names only `git` still reaches the discard guard.
/// Why:
///  Git appends the caller's words to the alias value before it starts the shell
///  (`git.c:401` to `git.c:402`),
///  so `git -c 'alias.wipe=!git' wipe reset --hard` runs a hard reset;
///  a reading that stops at the alias value sees a shell command with no words and
///  cannot guard the command the caller asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // real Git: tracked.txt becomes BASE; the wrapper: refused, tracked.txt stays USER DATA
/// ```
#[test]
fn a_shell_alias_naming_only_git_runs_the_callers_arguments_under_the_guard() {
    let fixture: Fixture = fixture("alias-expansion-shell-arguments");
    let control: PathBuf = repository_with_unstaged_work(&fixture, "control");
    let refused: PathBuf = repository_with_unstaged_work(&fixture, "refused");
    let arguments: [&str; 5] = ["-c", "alias.wipe=!git", "wipe", "reset", "--hard"];
    // The positive control proves the invocation discards the work when the alias value alone is not read.
    let direct_run: Observed = observe(direct(&fixture).current_dir(control.as_path()).args(arguments), b"");
    assert_eq!(direct_run.code, Some(0), "direct Git failed: {}", stderr_of(&direct_run));
    assert_eq!(tracked(control.as_path()), b"BASE\n", "the positive control discarded nothing");
    let observed: Observed = run_wrapped(&fixture, refused.as_path(), arguments.as_slice());
    assert_ne!(observed.code, Some(0), "the wrapper ran the shell alias's reset: {}", stderr_of(&observed));
    assert_eq!(tracked(refused.as_path()), USER_DATA, "the shell alias discarded the work");
    remove(&fixture);
}

/// What:
///  A shell alias resolves the aliases the caller configured,
///  because Git hands the Git inside the shell the configuration of every `-c` option it read.
/// Why:
///   `git_config_push_parameter` puts each `-c` entry in the environment Git starts
///  (`config.c:469` to `config.c:504`),
///  so `git -c 'alias.outer=!git inner' -c 'alias.inner=reset --hard' outer`
///  runs the inner reset;
///  a nested reading without those options finds no `inner` and leaves the reset unguarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // real Git: tracked.txt becomes BASE; the wrapper: refused, tracked.txt stays USER DATA
/// ```
#[test]
fn a_shell_alias_reads_the_aliases_the_caller_configured() {
    let fixture: Fixture = fixture("alias-expansion-shell-configuration");
    let control: PathBuf = repository_with_unstaged_work(&fixture, "control");
    let refused: PathBuf = repository_with_unstaged_work(&fixture, "refused");
    let arguments: [&str; 5] = [
        "-c",
        "alias.outer=!git inner",
        "-c",
        "alias.inner=reset --hard",
        "outer",
    ];
    // The positive control proves the invocation discards the work when the nested alias is not read.
    let direct_run: Observed = observe(direct(&fixture).current_dir(control.as_path()).args(arguments), b"");
    assert_eq!(direct_run.code, Some(0), "direct Git failed: {}", stderr_of(&direct_run));
    assert_eq!(tracked(control.as_path()), b"BASE\n", "the positive control discarded nothing");
    let observed: Observed = run_wrapped(&fixture, refused.as_path(), arguments.as_slice());
    assert_ne!(observed.code, Some(0), "the wrapper ran the inner alias's reset: {}", stderr_of(&observed));
    assert_eq!(tracked(refused.as_path()), USER_DATA, "the inner alias discarded the work");
    remove(&fixture);
}

/// What:
/// What:
///  The same refusal from a clean linked worktree,
///  where the guard of uncommitted work has nothing to reject and the copy of a
///       main checkout is not involved.
/// Why:
///   Git refuses such an alias before any command runs,
///  so no fact of the linked worktree describes the repository the
///       alias names,
///  and neither repository loses its work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // git -c 'alias.wipe=-C <other> reset --hard' wipe in a clean linked worktree => status 128
/// ```
#[test]
fn a_directory_changing_alias_from_a_linked_worktree_resets_nothing() {
    let fixture: Fixture = fixture("alias-expansion-linked-worktree");
    let main: PathBuf = repository_with_unstaged_work(&fixture, "main");
    let other: PathBuf = repository_with_unstaged_work(&fixture, "other");
    let linked: PathBuf = fixture.root.join("linked");
    let linked_argument: String = linked.display().to_string();
    git(
        &fixture,
        main.as_path(),
        &["worktree", "add", "--quiet", "-b", "side", linked_argument.as_str()],
    );
    // The linked worktree is clean, so no dirty check is what keeps the other repository.
    let cleaned: Observed = observe(direct(&fixture).current_dir(linked.as_path()).args(["reset", "--quiet", "--hard"]), b"");
    assert_eq!(cleaned.code, Some(0), "{}", stderr_of(&cleaned));
    let status: Vec<u8> = super::support::porcelain(&fixture, linked.as_path()).into_bytes();
    assert!(status.is_empty(), "the linked worktree is clean: {}", String::from_utf8_lossy(&status));
    let alias: String = format!("alias.wipe=-C {} reset --hard", other.display());
    let arguments: [&str; 3] = ["-c", alias.as_str(), "wipe"];
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
    assert_ne!(observed.code, Some(0), "the wrapper ran the alias: {}", stderr_of(&observed));
    assert!(stderr_of(&observed).contains("changes environment variables"), "{}", stderr_of(&observed));
    assert_eq!(tracked(other.as_path()), USER_DATA, "the other repository lost its work");
    assert_eq!(tracked(linked.as_path()), b"BASE\n", "the linked worktree changed");
    remove(&fixture);
}
