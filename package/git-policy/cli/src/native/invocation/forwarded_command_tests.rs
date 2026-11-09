//! Controls for forwarded-command resolution:
//!  alias value splitting,
//!  alias lookup through Git's
//! `GIT_CONFIG_*` overlay,
//!  expansion limits,
//!  loops,
//!  shell aliases and worktree creation.

use super::*;
use crate::test_support::{executable, fixture, git, git_output, isolated_overlay, remove};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// A context whose only aliases are the given name and value pairs,
///  read by real Git.
fn context_with_aliases(pairs: &[(&str, &str)]) -> GitContext {
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.push((OsString::from("GIT_CONFIG_COUNT"), OsString::from(pairs.len().to_string())));
    for (index, (name, value)) in pairs.iter().enumerate() {
        overlay.push((
            OsString::from(format!("GIT_CONFIG_KEY_{index}")),
            OsString::from(format!("alias.{name}")),
        ));
        overlay.push((
            OsString::from(format!("GIT_CONFIG_VALUE_{index}")),
            OsString::from(*value),
        ));
    }
    return GitContext {
        real_git: std::path::PathBuf::from(crate::test_support::REAL_GIT),
        overlay,
        global_prefix: Vec::new(),
        common_dir: None,
    };
}

/// Arguments as the wrapper receives them.
fn args(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// What:
///  An environment that names no `PATH` and no `GIT_EXEC_PATH`.
/// Why:
///   The resolver asks the exec path of Git itself when the environment names none,
///  and a control that
///       wants no executable of the test process to answer for one passes this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noEnvironment = [];
/// ```
fn no_environment() -> Vec<(OsString, OsString)> {
    return Vec::new();
}

/// One word as the bytes the splitter produces for it.
fn word_bytes(word: &&str) -> Vec<u8> {
    return word.as_bytes().to_vec();
}

/// Alias value words as bytes,
///  for comparing the splitter's output.
fn words(list: &[&str]) -> Option<Vec<Vec<u8>>> {
    return Some(list.iter().map(word_bytes).collect());
}

/// Whitespace separates words;
///  quotes group;
///  backslashes escape outside single quotes.
#[test]
fn alias_values_split_as_git_splits_them() {
    assert_eq!(split_alias_command(b"commit -a"), words(&["commit", "-a"]));
    assert_eq!(split_alias_command(b"a \t  b"), words(&["a", "b"]));
    // The incumbent trims before splitting; untrimmed leading whitespace yields an empty first word.
    assert_eq!(split_alias_command(b"  a"), words(&["", "a"]));
    assert_eq!(split_alias_command(b"'a b' c"), words(&["a b", "c"]));
    assert_eq!(split_alias_command(b"\"x y\""), words(&["x y"]));
    assert_eq!(split_alias_command(b"a\\ b"), words(&["a b"]));
    assert_eq!(split_alias_command(b"'a\\b'"), words(&["a\\b"]));
    assert_eq!(split_alias_command(b"\"a\\\"b\""), words(&["a\"b"]));
    assert_eq!(split_alias_command(b"''"), words(&[""]));
}

/// An empty value is one empty word,
///  as the incumbent splits it,
///  and an unfinished quote or a
/// trailing backslash yields no words at all.
#[test]
fn empty_and_unfinished_values_split_to_one_empty_word_or_fail() {
    assert_eq!(split_alias_command(b""), words(&[""]));
    assert_eq!(split_alias_command(b"   "), words(&[""]));
    assert_eq!(split_alias_command(b"'a"), None);
    assert_eq!(split_alias_command(b"\"a"), None);
    assert_eq!(split_alias_command(b"a\\"), None);
}

/// The value Git reports is used byte for byte:
///  whitespace at either end belongs to it,
///  because Git's own split keeps it.
#[test]
fn alias_value_bytes_are_kept_exactly() {
    let context: GitContext = context_with_aliases(&[("fixturespaces", "  commit \n")]);
    assert_eq!(
        lookup_alias(&context, &[], &OsString::from("fixturespaces")),
        Some(b"  commit \n".to_vec())
    );
}

/// A value that starts with a space keeps its empty first word,
///  which names no command Git can run,
///  so the invocation is left to Git instead of being trimmed into a command.
#[test]
fn a_value_with_a_leading_space_is_unresolved_and_not_trimmed_into_a_command() {
    let context: GitContext = context_with_aliases(&[("fixturetrim", " commit -v")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixturetrim"]));
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// Git refuses an alias whose expansion moves the directory or sets an environment variable,
///  and runs
///  nothing,
///  so the wrapper must leave that alias to Git.
///  `-c`,
///  `--config-env`,
///  `--exec-path=` and an empty `-C` are not refusals,
///  and Git runs those.
#[test]
fn an_alias_whose_expansion_changes_the_environment_is_left_to_git() {
    let root: PathBuf = fixture("alias-environment-refusal");
    for value in [
        "-C . status",
        "--git-dir=/tmp/git status",
        "--work-tree /tmp status",
        "--namespace n status",
        "--bare status",
        "--literal-pathspecs status",
        "--no-literal-pathspecs status",
        "--glob-pathspecs status",
        "--noglob-pathspecs status",
        "--icase-pathspecs status",
        "--no-optional-locks status",
        "--shallow-file /tmp/shallow status",
        "--attr-source /tmp/attrs status",
        "--no-advice status",
        "-P status",
        "--no-lazy-fetch status",
        "--no-replace-objects status",
    ] {
        let context: GitContext = context_with_aliases(&[("fixtureenv", value)]);
        let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixtureenv"]));
        assert_eq!(resolved.route, Route::Unresolved, "{value}");
        let spelling: String = format!("alias.fixtureenv={value}");
        let output: std::process::Output = git_output(root.as_path(), &["-c", spelling.as_str(), "fixtureenv"]);
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(!output.status.success(), "{value}: Git ran the alias: {stderr}");
        assert!(stderr.contains("changes environment variables"), "{value}: {stderr}");
    }
    for value in [
        "-c a.b=c status",
        "--config-env=a.b=HOME status",
        "--exec-path=/tmp status",
        "-p status",
        "-C \"\" status",
    ] {
        let context: GitContext = context_with_aliases(&[("fixtureenv", value)]);
        let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixtureenv"]));
        assert_eq!(resolved.route, Route::Alias, "{value}");
        assert_eq!(resolved.word(), b"status", "{value}");
        let spelling: String = format!("alias.fixtureenv={value}");
        let output: std::process::Output = git_output(root.as_path(), &["-c", spelling.as_str(), "fixtureenv"]);
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(!stderr.contains("changes environment variables"), "{value}: {stderr}");
    }
    remove(root.as_path());
}

/// What:
///  An installed `git-<name>` wins over an alias of that name,
///  so the alias of an installed command must not be
///       expanded,
///  and the control runs the same configuration through Git.
/// Why:
///   Git tries the dashed external before the alias (git.c:905 to git.c:912),
///  and the wrapper hands Git
///       the expansion it resolved,
///  so an expansion here would run a command Git never selects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolveForwardedCommand({ args: ['fixtureexternal'], environment }) // route 'unresolved'
/// ```
#[test]
fn an_alias_shadowed_by_an_external_command_is_left_to_git() {
    let root: PathBuf = fixture("alias-external-precedence");
    let directory: PathBuf = root.join("bin");
    std::fs::create_dir(&directory).expect("program directory");
    executable(directory.join("git-fixtureexternal").as_path(), b"#!/bin/sh\necho ran\n");
    let path: String = format!("{}:/usr/bin:/bin", directory.display());
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("PATH"), OsString::from(path))];
    let context: GitContext = context_with_aliases(&[("fixtureexternal", "status --short")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(
        &context,
        environment.as_slice(),
        &args(&["fixtureexternal"]),
    );
    assert_eq!(resolved.route, Route::Unresolved);
    assert_eq!(resolved.word(), b"fixtureexternal");
    // Git runs the program, so the alias's own command is not what the caller gets.
    let mut command: std::process::Command = std::process::Command::new(crate::test_support::REAL_GIT);
    command
        .current_dir(root.as_path())
        .env_clear()
        .env("PATH", directory.display().to_string())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["-c", "alias.fixtureexternal=status --short", "fixtureexternal"]);
    let output: std::process::Output = command.output().expect("native Git fixture command");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("ran"),
        "Git did not run the program: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    remove(root.as_path());
}

/// What:
///  The Git a shell alias starts reads the configuration of the `-c` options the caller gave,
///  because Git exports them to what it starts.
/// Why:
///  `git_config_push_parameter` puts each `-c` entry in the environment
///  (`config.c:469` to `config.c:504`),
///  so
///  `git -c 'alias.outer=!git inner' -c 'alias.inner=reset --hard' outer`
///  runs the inner reset;
///  a nested reading that forgot the caller's options finds no `inner`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolveForwardedCommand({ args: ['-c', 'alias.outer=!git inner', '-c', 'alias.inner=reset --hard', 'outer'] })
/// //   => { route: 'shell-alias', shellCommand: ['reset', '--hard'] }
/// ```
#[test]
fn a_shell_alias_reads_the_configuration_the_caller_passed() {
    let context: GitContext = context_with_aliases(&[("fixtureouter", "!git fixtureinner")]);
    let followed: ResolvedCommand = resolve_forwarded_command(
        &context,
        no_environment().as_slice(),
        &args(&["-c", "alias.fixtureinner=reset --hard", "fixtureouter"]),
    );
    assert_eq!(followed.route, Route::ShellAlias);
    assert_eq!(followed.shell_command, Some(args(&["reset", "--hard"])));
}

/// What:
///  The one plain `git` command a `!` alias runs is read,
///  and the caller's arguments are not replaced with it.
/// Why:
///   Git runs the shell with its own executable directory first on `PATH`,
///  so the `git` such an alias
///       names is the real Git;
///  the reading reaches the guard,
///  while forwarding the shell itself is what keeps the
///       environment Git gives it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolveForwardedCommand({ args: ['fixtureshell'] }).shellCommand // ['reset', '--hard']
/// ```
#[test]
fn a_shell_alias_that_names_one_git_command_is_read_for_the_guard() {
    let context: GitContext = context_with_aliases(&[("fixtureshell", "!git reset --hard")]);
    let resolved: ResolvedCommand =
        resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixtureshell"]));
    assert_eq!(resolved.route, Route::ShellAlias);
    assert_eq!(
        resolved.shell_command,
        Some(args(&["reset", "--hard"])),
        "the shell's own command is read"
    );
    assert_eq!(resolved.arguments, args(&["fixtureshell"]), "the caller's arguments stay untouched");
    // A nested alias named after `git` is the command the shell runs, so the chain is followed.
    let chained: GitContext = context_with_aliases(&[("fixtureshell", "!git fixturewipe"), ("fixturewipe", "reset --hard")]);
    let followed: ResolvedCommand =
        resolve_forwarded_command(&chained, no_environment().as_slice(), &args(&["fixtureshell"]));
    assert_eq!(followed.shell_command, Some(args(&["reset", "--hard"])));
}

/// What:
///  What real Git prints for one invocation of a `!` alias in a fixture,
///  with `path` as the only `PATH` entry list it searches.
/// Why:
///  The control answers the same question the reading answers,
///  so a wrong reading fails against Git and not
///  against another reading of the same rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFile(REAL_GIT, args, { cwd, env: { PATH: path } }).stdout
/// ```
fn real_git_stdout(directory: &Path, path: &str, arguments: &[&str]) -> String {
    let output: std::process::Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(directory)
        .env_clear()
        .env("PATH", path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(arguments)
        .output()
        .expect("native Git fixture command");
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// What:
///  A shell alias searches for an installed `git-<word>` from the top of the working tree,
///  which is the directory Git runs the shell in,
///  so the caller's `-C` moves neither search.
/// Why:
///  Git changes to the top of the working tree before it starts the shell of a `!` alias
///  (`git.c:388` to `git.c:407`),
///  so `git -C sub fixtureouter` with `fixtureouter = !git fixturewipe` runs `fixturewipe` from
///  the repository root;
///  a reading that applied the caller's `-C` again searched `sub/bin`,
///  found a program the shell never reaches,
///  and reported no shell command at all,
///  which left the alias to Git and skipped the guard.
///  The absolute entry is the positive control:
///  with it the program is reachable and both the reading and Git find it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // sub/bin/git-fixturewipe exists, args ['-C', 'sub', 'fixtureouter'], PATH 'bin:/usr/bin:/bin'
/// resolveForwardedCommand(...).shellCommand // ['version'], and Git runs the same alias
/// ```
#[test]
fn a_shell_alias_searches_for_an_executable_where_git_runs_it() {
    let root: PathBuf = fixture("alias-shell-run-directory");
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main", "."]);
    let directory: PathBuf = root.join("sub/bin");
    std::fs::create_dir_all(directory.as_path()).expect("program directory");
    executable(directory.join("git-fixturewipe").as_path(), b"#!/bin/sh\necho external\n");
    let started_in: PathBuf = std::env::current_dir().expect("the directory this test started in");
    std::env::set_current_dir(root.as_path()).expect("the fixture is where the wrapper starts");
    let context: GitContext = context_with_aliases(&[
        ("fixtureouter", "!git fixturewipe"),
        ("fixturewipe", "version"),
    ]);
    let invocation: Vec<&str> = vec![
        "-c",
        "alias.fixtureouter=!git fixturewipe",
        "-c",
        "alias.fixturewipe=version",
        "-C",
        "sub",
        "fixtureouter",
    ];
    // A relative entry is the repository root's `bin`, which holds no program: Git expands the alias there.
    let relative: String = String::from("bin:/usr/bin:/bin");
    let environment: Vec<(OsString, OsString)> =
        vec![(OsString::from("PATH"), OsString::from(relative.as_str()))];
    let resolved: ResolvedCommand = resolve_forwarded_command(
        &context,
        environment.as_slice(),
        &args(&["-C", "sub", "fixtureouter"]),
    );
    assert_eq!(resolved.route, Route::ShellAlias);
    assert_eq!(
        resolved.shell_command,
        Some(args(&["version"])),
        "the shell's own command is read from the root"
    );
    let stdout: String = real_git_stdout(root.as_path(), relative.as_str(), invocation.as_slice());
    assert!(
        stdout.contains("git version"),
        "Git did not run the alias the reading followed: {stdout}"
    );
    assert!(!stdout.contains("external"), "Git ran the program under sub/bin: {stdout}");
    // The positive control: an absolute entry reaches the program, the reading sees it and Git runs it.
    let absolute: String = format!("{}:/usr/bin:/bin", directory.display());
    let reachable: Vec<(OsString, OsString)> =
        vec![(OsString::from("PATH"), OsString::from(absolute.as_str()))];
    let external: ResolvedCommand = resolve_forwarded_command(
        &context,
        reachable.as_slice(),
        &args(&["-C", "sub", "fixtureouter"]),
    );
    assert_eq!(external.route, Route::ShellAlias);
    assert_eq!(external.shell_command, None, "the program is reachable, so Git runs it");
    let stdout: String = real_git_stdout(root.as_path(), absolute.as_str(), invocation.as_slice());
    assert!(
        stdout.contains("external"),
        "the control did not reach the program, so the relative case proves nothing: {stdout}"
    );
    std::env::set_current_dir(started_in.as_path()).expect("leave the fixture before removing it");
    remove(root.as_path());
}

/// What:
///  Anything the shell would expand,
///  quote,
///  separate or run a second time is left unread.
/// Why:
///   Reading it would be a shell parser,
///  and a wrong reading would decide about a command nobody runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const value of ["!sh -c 'git reset --hard'", "!git -C /elsewhere reset --hard", "!git status; rm -rf x"]) assert.equal(read(value), undefined);
/// ```
#[test]
fn a_shell_alias_the_reading_cannot_follow_is_left_unread() {
    for value in [
        "!sh -c 'git reset --hard'",
        "!git -C /elsewhere reset --hard",
        "!git status; rm -rf x",
        "!git reset --hard && echo done",
        "!git reset --hard $BRANCH",
        "!git reset --hard `cat file`",
        "!git reset --hard *.txt",
        "!git",
        "!gitk reset",
        "!git reset --hard | tee log",
        "!git reset --hard > log",
    ] {
        let context: GitContext = context_with_aliases(&[("fixtureshell", value)]);
        let resolved: ResolvedCommand =
            resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixtureshell"]));
        assert_eq!(resolved.route, Route::ShellAlias, "{value}");
        assert_eq!(resolved.shell_command, None, "{value} must stay unread");
        assert_eq!(resolved.arguments, args(&["fixtureshell"]), "{value}");
    }
    // One shell is read, so a shell alias that names another shell alias is left unread rather than followed twice.
    let doubled: GitContext = context_with_aliases(&[("fixtureshell", "!git fixturesecond"), ("fixturesecond", "!git reset --hard")]);
    let resolved: ResolvedCommand =
        resolve_forwarded_command(&doubled, no_environment().as_slice(), &args(&["fixtureshell"]));
    assert_eq!(resolved.shell_command, None, "a second shell is not read");
}

/// A built-in named directly is resolved without an alias lookup and keeps its global options.
#[test]
fn a_direct_builtin_resolves_past_its_global_options() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["-C", "/x", "status"]));
    assert_eq!(resolved.route, Route::Direct);
    assert_eq!(resolved.word(), b"status");
    assert_eq!(resolved.subcommand_index, 2);
    assert!(resolved.is_builtin());
}

/// A bare `git --version` has no subcommand and is not a built-in command.
#[test]
fn a_query_has_no_subcommand() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["--version"]));
    assert_eq!(resolved.subcommand, None);
    assert_eq!(resolved.word(), b"");
    assert!(!resolved.is_builtin());
}

/// An alias to a built-in expands to its words,
///  with the rest of the original arguments after them.
#[test]
fn an_alias_expands_to_its_words_then_the_rest() {
    let context: GitContext = context_with_aliases(&[("fixturecommit", "commit -v")]);
    let resolved: ResolvedCommand =
        resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixturecommit", "-m", "x"]));
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.word(), b"commit");
    assert_eq!(resolved.arguments, args(&["commit", "-v", "-m", "x"]));
    assert!(resolved.is_builtin());
}

/// Chained aliases expand one after another until a built-in is reached.
#[test]
fn chained_aliases_resolve_to_the_builtin_at_the_end() {
    let context: GitContext = context_with_aliases(&[("fixturex", "fixturey -a"), ("fixturey", "commit")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixturex"]));
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.arguments, args(&["commit", "-a"]));
}

/// A built-in name is never replaced by an alias of the same name,
///  as Git ignores such aliases.
#[test]
fn a_builtin_name_is_not_shadowed_by_an_alias() {
    let context: GitContext = context_with_aliases(&[("commit", "status")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["commit"]));
    assert_eq!(resolved.route, Route::Direct);
    assert_eq!(resolved.word(), b"commit");
}

/// A `!` alias runs a shell command,
///  so it is never a built-in whatever its text says.
#[test]
fn a_shell_alias_is_never_a_builtin() {
    let context: GitContext = context_with_aliases(&[("fixtureshell", "!git commit -a")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixtureshell"]));
    assert_eq!(resolved.route, Route::ShellAlias);
    assert!(!resolved.is_builtin());
    assert!(!creates_worktrees(&resolved));
}

/// An alias that names itself,
///  directly or through another,
///  is an unresolved loop.
#[test]
fn an_alias_loop_is_unresolved() {
    let context: GitContext = context_with_aliases(&[("fixtureloopa", "fixtureloopb"), ("fixtureloopb", "fixtureloopa")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixtureloopa"]));
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// An unknown name with no alias is unresolved and never a built-in.
#[test]
fn an_unknown_name_is_unresolved() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixture-no-such-command"]));
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// A value whose quoting cannot be split is unresolved,
///  not guessed.
#[test]
fn a_malformed_alias_value_is_unresolved() {
    let context: GitContext = context_with_aliases(&[("fixturebad", "'commit")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixturebad"]));
    assert_eq!(resolved.route, Route::Unresolved);
}

/// An empty alias value has no command word and is unresolved.
#[test]
fn an_empty_alias_value_is_unresolved() {
    let context: GitContext = context_with_aliases(&[("fixtureempty", "   ")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["fixtureempty"]));
    assert_eq!(resolved.route, Route::Unresolved);
}

/// A chain of `length` aliases,
///  each naming the next,
///  the last naming `commit`.
fn chain_context(length: usize) -> (GitContext, OsString) {
    let mut names: Vec<String> = Vec::new();
    for index in 0..length {
        names.push(format!("fixturechain{index}"));
    }
    let mut pairs: Vec<(String, String)> = Vec::new();
    for index in 0..length {
        let target: String = if index + 1 < length {
            names[index + 1].clone()
        } else {
            String::from("commit")
        };
        pairs.push((names[index].clone(), target));
    }
    let mut pair_refs: Vec<(&str, &str)> = Vec::new();
    for (name, target) in &pairs {
        pair_refs.push((name.as_str(), target.as_str()));
    }
    return (context_with_aliases(pair_refs.as_slice()), OsString::from(names[0].as_str()));
}

/// A chain as long as the limit,
///  each alias one expansion,
///  reaches the built-in.
#[test]
fn the_expansion_limit_reaches_the_builtin() {
    let (context, first): (GitContext, OsString) = chain_context(MAX_ALIAS_EXPANSIONS);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &[first]);
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.word(), b"commit");
}

/// One chained alias more than the limit allows and the command stays unresolved.
#[test]
fn one_expansion_past_the_limit_is_unresolved() {
    let (context, first): (GitContext, OsString) = chain_context(MAX_ALIAS_EXPANSIONS + 1);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &[first]);
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// Only `worktree add` creates a linked worktree;
/// a move registers nothing new,
/// and other forms do not create one.
#[test]
fn only_worktree_add_creates_worktrees() {
    let context: GitContext = context_with_aliases(&[]);
    let add: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["worktree", "add", "x"]));
    let moved: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["worktree", "move", "a", "b"]));
    let verbose_add: ResolvedCommand =
        resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["worktree", "--verbose", "add", "x"]));
    let list: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["worktree", "list"]));
    let bare: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["worktree"]));
    let other: ResolvedCommand = resolve_forwarded_command(&context, no_environment().as_slice(), &args(&["status", "add"]));
    assert!(creates_worktrees(&add));
    assert!(!creates_worktrees(&moved));
    assert!(creates_worktrees(&verbose_add));
    assert!(!creates_worktrees(&list));
    assert!(!creates_worktrees(&bare));
    assert!(!creates_worktrees(&other));
}
