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

use super::support::{Fixture, Observed, bounded, direct, executable, fixture, git, observe, remove, repository, run_wrapped, stderr_of, wrapped};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// What:
///  A command for the wrapper started by its own path,
///  with real Git's directory first on `PATH`.
/// Why:
///  A plain alias is run as a child `git` that Git finds on `PATH` (`git.c:885`),
///  so with the wrapper's own link first there that child is the wrapper,
///  whose guard refuses the expansion before the reading under test is reached;
///  a control must start a `git` that is real Git to see what Git does with the alias.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // spawn(fixture.wrapper, args, { env: { PATH: '/usr/bin:/bin', ... } })
/// ```
fn wrapper_program(fixture: &Fixture) -> Command {
    return bounded(
        fixture,
        fixture.root.join("wrapper/monochromatic-git-policy-cli").as_path(),
        OsStr::new("/usr/bin:/bin"),
    );
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
///  The caller's `--exec-path` decides where Git looks for an installed command,
///  over a `GIT_EXEC_PATH` the environment also names.
/// Why:
///   `--exec-path=<dir>` sets the exec path
///  (`git.c:178` to `git.c:181`),
///  and `git_exec_path` answers with that setting before it reads the environment
///  (`exec-cmd.c:313` to `exec-cmd.c:323`);
///  a reading that read `GIT_EXEC_PATH` first finds a program Git never runs and leaves the alias
///       unguarded.
///  The control starts the wrapper by its own path so that the `git` Git would run for the alias is real Git:
///  the wrapper's link first on `PATH` makes that child the wrapper,
///  whose guard refuses the expansion either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // git-wipe only in old-exec, GIT_EXEC_PATH=old-exec, --exec-path=<empty new-exec>
/// // real Git: alias's reset runs; the wrapper: refused
/// ```
#[test]
fn the_exec_path_option_decides_the_search_over_the_environment() {
    let fixture: Fixture = fixture("alias-expansion-exec-path-option");
    let control: PathBuf = repository_with_unstaged_work(&fixture, "control");
    let refused: PathBuf = repository_with_unstaged_work(&fixture, "refused");
    let old_exec: PathBuf = fixture.root.join("old-exec");
    let new_exec: PathBuf = fixture.root.join("new-exec");
    std::fs::create_dir(old_exec.as_path()).expect("old exec directory");
    std::fs::create_dir(new_exec.as_path()).expect("new exec directory");
    executable(
        old_exec.join("git-wipe").as_path(),
        b"#!/bin/sh\necho 'EXTERNAL: no mutation'\n",
    );
    let option: OsString = OsString::from(format!("--exec-path={}", new_exec.display()));
    let arguments: Vec<OsString> = vec![
        option,
        OsString::from("-c"),
        OsString::from("alias.wipe=reset --hard"),
        OsString::from("wipe"),
    ];
    // The positive control: Git searches the empty directory the option names and the alias discards the work.
    let direct_run: Observed = observe(
        direct(&fixture)
            .current_dir(control.as_path())
            .args(arguments.as_slice())
            .env("GIT_EXEC_PATH", old_exec.as_os_str()),
        b"",
    );
    let direct_stdout: String = String::from_utf8_lossy(&direct_run.stdout).into_owned();
    assert!(
        !direct_stdout.contains("EXTERNAL"),
        "the positive control ran the program instead of the alias: {direct_stdout}"
    );
    assert_eq!(tracked(control.as_path()), b"BASE\n", "the positive control discarded nothing");
    let observed: Observed = observe(
        wrapper_program(&fixture)
            .current_dir(refused.as_path())
            .args(arguments.as_slice())
            .env("GIT_EXEC_PATH", old_exec.as_os_str()),
        b"",
    );
    let stdout: String = String::from_utf8_lossy(&observed.stdout).into_owned();
    assert!(!stdout.contains("EXTERNAL"), "the wrapper ran a program Git does not run: {stdout}");
    assert_ne!(observed.code, Some(0), "the wrapper ran the alias's reset: {}", stderr_of(&observed));
    assert_eq!(tracked(refused.as_path()), USER_DATA, "the alias discarded the work");
    remove(&fixture);
}

/// What:
///  A `GIT_EXEC_PATH` relative to the directory Git runs in is searched there,
///  so an ordinary alias of a word no program names reaches the guard.
/// Why:
///   Git makes a relative exec path absolute against its own directory before it adds it to
///       `PATH`
///  (`exec-cmd.c:325` to `exec-cmd.c:340`),
///  so the search has an answer;
///  a reading that calls the relative path uncertain leaves the alias to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cwd repo, mkdir exec, GIT_EXEC_PATH=exec, alias.wipe=reset --hard => real Git resets; the wrapper refuses
/// ```
#[test]
fn a_relative_exec_path_is_searched_where_git_searches_it() {
    let fixture: Fixture = fixture("alias-expansion-relative-exec-path");
    let control: PathBuf = repository_with_unstaged_work(&fixture, "control");
    let refused: PathBuf = repository_with_unstaged_work(&fixture, "refused");
    for repo in [control.as_path(), refused.as_path()] {
        std::fs::create_dir(repo.join("exec")).expect("exec path directory");
    }
    let arguments: [&str; 3] = ["-c", "alias.wipe=reset --hard", "wipe"];
    let exec_path: &OsStr = OsStr::new("exec");
    // The positive control: the directory holds no `git-wipe`, so Git expands the alias and discards the work.
    let direct_run: Observed = observe(
        direct(&fixture)
            .current_dir(control.as_path())
            .args(arguments)
            .env("GIT_EXEC_PATH", exec_path),
        b"",
    );
    assert_eq!(direct_run.code, Some(0), "direct Git failed: {}", stderr_of(&direct_run));
    assert_eq!(tracked(control.as_path()), b"BASE\n", "the positive control discarded nothing");
    let observed: Observed = observe(
        wrapper_program(&fixture)
            .current_dir(refused.as_path())
            .args(arguments)
            .env("GIT_EXEC_PATH", exec_path),
        b"",
    );
    assert_ne!(observed.code, Some(0), "the wrapper ran the alias's reset: {}", stderr_of(&observed));
    assert_eq!(tracked(refused.as_path()), USER_DATA, "the alias discarded the work");
    remove(&fixture);
}

/// What:
///  A relative executable-search entry is searched from the directory Git changed into,
///  not from the one the caller stood in before `-C`.
/// Why:
///   `-C` changes Git's directory before it looks a command up
///  (`git.c:316` to `git.c:328`,
///  then `git.c:988`),
///  and a `PATH` entry without
///       a leading separator is resolved against that directory;
///  a reading that resolves it against the caller's directory finds a program Git never runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // A/bin/git-wipe exists, B is a repository without a bin, PATH=bin:/usr/bin:/bin, git -C ../B ... wipe
/// // real Git: alias's reset runs; the wrapper: refused
/// ```
#[test]
fn a_relative_search_entry_is_read_from_the_directory_git_changed_into() {
    let fixture: Fixture = fixture("alias-expansion-relative-path-entry");
    let caller: PathBuf = fixture.root.join("A");
    std::fs::create_dir(caller.as_path()).expect("caller directory");
    let program_directory: PathBuf = caller.join("bin");
    std::fs::create_dir(program_directory.as_path()).expect("program directory");
    executable(
        program_directory.join("git-wipe").as_path(),
        b"#!/bin/sh\necho 'EXTERNAL: no mutation'\n",
    );
    let control: PathBuf = repository_with_unstaged_work(&fixture, "B");
    let refused: PathBuf = repository_with_unstaged_work(&fixture, "C");
    let path: &OsStr = OsStr::new("bin:/usr/bin:/bin");
    let control_arguments: [&str; 5] = ["-C", "../B", "-c", "alias.wipe=reset --hard", "wipe"];
    let refused_arguments: [&str; 5] = ["-C", "../C", "-c", "alias.wipe=reset --hard", "wipe"];
    // The positive control: real Git searches `B/bin`, finds no program, and the alias discards the work.
    let direct_run: Observed = observe(
        direct(&fixture)
            .current_dir(caller.as_path())
            .args(control_arguments)
            .env("PATH", path),
        b"",
    );
    let direct_stdout: String = String::from_utf8_lossy(&direct_run.stdout).into_owned();
    assert!(
        !direct_stdout.contains("EXTERNAL"),
        "the positive control ran the program instead of the alias: {direct_stdout}"
    );
    assert_eq!(tracked(control.as_path()), b"BASE\n", "the positive control discarded nothing");
    let observed: Observed = observe(
        wrapper_program(&fixture)
            .current_dir(caller.as_path())
            .args(refused_arguments)
            .env("PATH", path),
        b"",
    );
    let stdout: String = String::from_utf8_lossy(&observed.stdout).into_owned();
    assert!(!stdout.contains("EXTERNAL"), "the wrapper ran a program Git does not run: {stdout}");
    assert_ne!(observed.code, Some(0), "the wrapper ran the alias's reset: {}", stderr_of(&observed));
    assert_eq!(tracked(refused.as_path()), USER_DATA, "the alias discarded the work");
    remove(&fixture);
}

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

/// What:
///  A value is split into the words Git's `split_cmdline` produces,
///  so the same invocation through real Git and through the wrapper runs the same command.
/// Why:
///  Git's `isspace` in the splitter is `GIT_SPACE`
///  (space,
///  TAB,
///  LF and CR only,
///  `ctype.c`),
///  and the split leaves an empty last word when the value ends with whitespace,
///  so a wider whitespace class ran `rev-parse --git-dir` where Git ran one command word
///  and for a trailing space ran `status` where Git refused `status ''`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // for each value: direct.stdout === wrapped.stdout and direct.code === wrapped.code
/// ```
#[test]
fn an_alias_value_is_split_into_the_words_git_splits() {
    let fixture: Fixture = fixture("alias-expansion-split");
    let repo: PathBuf = repository_with_unstaged_work(&fixture, "repo");
    let trailing: [&str; 5] = [" ", "\t", "\n", "\r", "  "];
    let between: [&str; 4] = ["\u{0b}", "\u{0c}", "\t", "  "];
    let mut values: Vec<String> = Vec::new();
    for tail in trailing {
        values.push(format!("rev-parse --git-dir{tail}"));
        values.push(format!("status{tail}"));
    }
    for separator in between {
        values.push(format!("rev-parse{separator}--git-dir"));
    }
    for value in values {
        let option: String = format!("alias.audit={value}");
        let arguments: [&str; 3] = ["-c", option.as_str(), "audit"];
        let direct_run: Observed = observe(
            direct(&fixture).current_dir(repo.as_path()).args(arguments),
            b"",
        );
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(
            (observed.code, observed.stdout.clone()),
            (direct_run.code, direct_run.stdout.clone()),
            "{value:?}: the wrapper ran a different command; direct stderr: {}",
            stderr_of(&direct_run)
        );
        assert_eq!(tracked(repo.as_path()), USER_DATA, "{value:?} changed the worktree");
    }
    remove(&fixture);
}

/// What:
///  Two spellings of one alias select the value Git selects:
///  the last one in config order,
///  wherever it is defined.
/// Why:
///  Git's `config_alias_cb` replaces the value it kept on every matching variable
///  (`alias.c`),
///  so the caller reads the answer of the last definition,
///  and a lookup that preferred `alias.<name>` ran the command Git does not select.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // alias.audit = 'rev-parse --is-inside-work-tree', then [alias "audit"] command = 'rev-parse --git-dir'
/// // => both arms print .git
/// ```
#[test]
fn two_alias_spellings_run_the_one_git_selects() {
    let fixture: Fixture = fixture("alias-expansion-spellings");
    let plain_then_section: PathBuf = repository_with_unstaged_work(&fixture, "plain-then-section");
    git(
        &fixture,
        plain_then_section.as_path(),
        &["config", "--local", "alias.audit", "rev-parse --is-inside-work-tree"],
    );
    git(
        &fixture,
        plain_then_section.as_path(),
        &["config", "--local", "alias.audit.command", "rev-parse --git-dir"],
    );
    let section_then_plain: PathBuf = repository_with_unstaged_work(&fixture, "section-then-plain");
    git(
        &fixture,
        section_then_plain.as_path(),
        &["config", "--local", "alias.audit.command", "rev-parse --git-dir"],
    );
    git(
        &fixture,
        section_then_plain.as_path(),
        &["config", "--local", "alias.audit", "rev-parse --is-inside-work-tree"],
    );
    let command_line_last: PathBuf = repository_with_unstaged_work(&fixture, "command-line-last");
    git(
        &fixture,
        command_line_last.as_path(),
        &["config", "--local", "alias.audit", "rev-parse --is-inside-work-tree"],
    );
    let cases: Vec<(PathBuf, Vec<&str>)> = vec![
        (plain_then_section, vec!["audit"]),
        (section_then_plain, vec!["audit"]),
        (
            command_line_last,
            vec!["-c", "alias.audit.command=rev-parse --git-dir", "audit"],
        ),
    ];
    for (repo, arguments) in cases {
        let direct_run: Observed = observe(
            direct(&fixture).current_dir(repo.as_path()).args(arguments.as_slice()),
            b"",
        );
        assert_eq!(direct_run.code, Some(0), "the oracle did not answer: {}", stderr_of(&direct_run));
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(
            (observed.code, observed.stdout.clone()),
            (direct_run.code, direct_run.stdout.clone()),
            "{arguments:?}: the wrapper ran the other spelling; direct stderr: {}",
            stderr_of(&direct_run)
        );
    }
    remove(&fixture);
}

/// What:
///  The alias spellings Git matches by other rules keep their answers:
///  a plain key matches whatever its case,
///  a subsection is case-sensitive,
///  and a variable with no value is Git's own refusal.
/// Why:
///  The selection reads Git's variables itself,
///  so a rule it got wrong would run a different command or invent a refusal Git never prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // alias.AUDIT-CASE answers git audit-case; [alias "UPPER"] command does not answer git upper;
/// // a valueless alias.value-free fails with "missing value"
/// ```
#[test]
fn the_alias_matching_rules_git_applies_keep_their_answers() {
    let fixture: Fixture = fixture("alias-expansion-matching");
    let upper_plain: PathBuf = repository_with_unstaged_work(&fixture, "upper-plain");
    git(
        &fixture,
        upper_plain.as_path(),
        &["config", "--local", "alias.AUDIT-CASE", "rev-parse --git-dir"],
    );
    let upper_section: PathBuf = repository_with_unstaged_work(&fixture, "upper-section");
    git(
        &fixture,
        upper_section.as_path(),
        &["config", "--local", "alias.UPPER.command", "rev-parse --git-dir"],
    );
    let valueless: PathBuf = repository_with_unstaged_work(&fixture, "valueless");
    let config: PathBuf = valueless.join(".git/config");
    let mut text: Vec<u8> = std::fs::read(&config).expect("repository config");
    text.extend_from_slice(b"\n[alias]\n\tvalue-free\n");
    std::fs::write(&config, text).expect("config with a valueless alias");
    let cases: Vec<(PathBuf, Vec<&str>)> = vec![
        (upper_plain, vec!["audit-case"]),
        (upper_section, vec!["upper"]),
        (valueless, vec!["value-free"]),
    ];
    for (repo, arguments) in cases {
        let direct_run: Observed = observe(
            direct(&fixture).current_dir(repo.as_path()).args(arguments.as_slice()),
            b"",
        );
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(
            (observed.code, observed.stdout.clone()),
            (direct_run.code, direct_run.stdout.clone()),
            "{arguments:?}: direct stderr: {}",
            stderr_of(&direct_run)
        );
    }
    remove(&fixture);
}
