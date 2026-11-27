//! Controls for forwarded-command resolution:
//!  alias value splitting,
//!  alias lookup through Git's
//! `GIT_CONFIG_*` overlay,
//!  expansion limits,
//!  loops,
//!  shell aliases and worktree creation.

use super::*;
use crate::test_support::{
    executable,
    fixture,
    git,
    git_output,
    isolated_overlay,
    remove,
};
use crate::transaction_git::GitContext;
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};
/// The finished child each real Git fixture run is.
use std::process::Output;

/// A context whose only aliases are the given name and value pairs,
///  read by real Git.
fn context_with_aliases(pairs: &[(&str, &str)]) -> GitContext {
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.push((
        OsString::from("GIT_CONFIG_COUNT"),
        OsString::from(pairs.len().to_string()),
    ));
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
///  The directory a control's alias reader starts Git in,
///  when every setting the control sets comes from the configuration overlay.
/// Why:
///   An overlay holds the whole configuration these controls read,
///  so any directory Git can start in answers the same;
///  an absolute one no control moves or deletes keeps each answer the test's own,
///  where a read of the process could answer with another test's fixture or a deleted one.
fn outside_every_repository() -> &'static Path {
    return Path::new("/");
}

/// Resolve one command as a caller started in `outside_every_repository` would.
fn resolved_from(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
) -> ResolvedCommand {
    return resolve_forwarded_command(
        context,
        environment,
        arguments,
        Some(outside_every_repository()),
    );
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

/// Whichever whitespace class Git's split reads separates words;
///  quotes group;
///  backslashes escape outside single quotes.
/// Git's `isspace` here is `GIT_SPACE`,
///  which is space,
///  TAB,
///  LF and CR only (`ctype.c`,
///  `sane_ctype`),
///  so a vertical tab or a form feed is an ordinary byte inside one word.
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
    // A vertical tab and a form feed are inside the word, not between words.
    assert_eq!(
        split_alias_command(b"rev-parse\x0b--git-dir"),
        words(&["rev-parse\x0b--git-dir"])
    );
    assert_eq!(
        split_alias_command(b"rev-parse\x0c--git-dir"),
        words(&["rev-parse\x0c--git-dir"])
    );
}

/// What:
///  A value that ends with whitespace splits into one more,
///  empty word,
///  exactly as Git's `split_cmdline` leaves it.
/// Why:
///  Git points its next word at the byte after the NUL it wrote over the whitespace
///  (`alias.c`,
///  `split_cmdline`),
///  so `'alias.x=status '` runs `git status ''` and fails with `empty string is not a valid pathspec`;
///  a split that dropped the empty word ran `git status` and lost no work but ran a command Git refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// splitAliasCommand('status ') === ['status', '']
/// ```
#[test]
fn a_value_that_ends_with_whitespace_keeps_gits_empty_last_word() {
    assert_eq!(split_alias_command(b"status "), words(&["status", ""]));
    assert_eq!(
        split_alias_command(b"rev-parse --git-dir\t"),
        words(&["rev-parse", "--git-dir", ""])
    );
    assert_eq!(
        split_alias_command(b"rev-parse --git-dir\n"),
        words(&["rev-parse", "--git-dir", ""])
    );
    assert_eq!(
        split_alias_command(b"rev-parse --git-dir\r"),
        words(&["rev-parse", "--git-dir", ""])
    );
    // Two spaces between words are still one separator, and whitespace at the end is one empty word, not two.
    assert_eq!(split_alias_command(b"a  b"), words(&["a", "b"]));
    assert_eq!(split_alias_command(b"a  "), words(&["a", ""]));
}

/// An empty value is one empty word,
///  as the incumbent splits it,
///  and an unfinished quote or a
/// trailing backslash yields no words at all.
#[test]
fn empty_and_unfinished_values_split_to_one_empty_word_or_fail() {
    assert_eq!(split_alias_command(b""), words(&[""]));
    // Git starts with one word before it reads any byte, and each whitespace run starts another, so three
    //  spaces are an empty first word and a second empty word.
    assert_eq!(split_alias_command(b"   "), words(&["", ""]));
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
        lookup_alias(
            &context,
            &[],
            &OsString::from("fixturespaces"),
            Some(outside_every_repository())
        ),
        Some(b"  commit \n".to_vec())
    );
}

/// A value that starts with a space keeps its empty first word,
///  which names no command Git can run,
///  so the invocation is left to Git instead of being trimmed into a command.
#[test]
fn a_value_with_a_leading_space_is_unresolved_and_not_trimmed_into_a_command() {
    let context: GitContext = context_with_aliases(&[("fixturetrim", " commit -v")]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["fixturetrim"]),
    );
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// What:
///  Both spellings of one alias are read,
///  and the last one in config order wins,
///  as Git's `config_alias_cb` overwrites on every match.
/// Why:
///  A lookup that read `alias.<name>` first and `alias.<name>.command` only when the first was unset
///  ran the command Git does not select (`forwarded_command.rs`,
///  `lookup_alias`),
///  and the caller saw a command Git itself would never run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // alias.x = 'status --short' then alias.x.command = 'diff --stat' => the alias runs diff
/// ```
#[test]
fn the_last_of_the_two_spellings_in_config_order_is_the_alias() {
    for (pairs, expected) in [
        (
            vec![
                ("fixturetwo", "status --short"),
                ("fixturetwo.command", "diff --stat"),
            ],
            "diff",
        ),
        (
            vec![
                ("fixturetwo.command", "diff --stat"),
                ("fixturetwo", "status --short"),
            ],
            "status",
        ),
        (
            vec![
                ("fixturetwo", "status --short"),
                ("fixturetwo", "diff --stat"),
            ],
            "diff",
        ),
    ] {
        let context: GitContext = context_with_aliases(pairs.as_slice());
        let resolved: ResolvedCommand = resolved_from(
            &context,
            no_environment().as_slice(),
            &args(&["fixturetwo"]),
        );
        assert_eq!(resolved.route, Route::Alias, "{pairs:?}");
        assert_eq!(
            String::from_utf8_lossy(resolved.word()),
            expected,
            "{pairs:?}"
        );
    }
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
        let resolved: ResolvedCommand = resolved_from(
            &context,
            no_environment().as_slice(),
            &args(&["fixtureenv"]),
        );
        assert_eq!(resolved.route, Route::Unresolved, "{value}");
        let spelling: String = format!("alias.fixtureenv={value}");
        let output: std::process::Output =
            git_output(root.as_path(), &["-c", spelling.as_str(), "fixtureenv"]);
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(
            !output.status.success(),
            "{value}: Git ran the alias: {stderr}"
        );
        assert!(
            stderr.contains("changes environment variables"),
            "{value}: {stderr}"
        );
    }
    for value in [
        "-c a.b=c status",
        "--config-env=a.b=HOME status",
        "--exec-path=/tmp status",
        "-p status",
        "-C \"\" status",
    ] {
        let context: GitContext = context_with_aliases(&[("fixtureenv", value)]);
        let resolved: ResolvedCommand = resolved_from(
            &context,
            no_environment().as_slice(),
            &args(&["fixtureenv"]),
        );
        assert_eq!(resolved.route, Route::Alias, "{value}");
        assert_eq!(resolved.word(), b"status", "{value}");
        let spelling: String = format!("alias.fixtureenv={value}");
        let output: std::process::Output =
            git_output(root.as_path(), &["-c", spelling.as_str(), "fixtureenv"]);
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(
            !stderr.contains("changes environment variables"),
            "{value}: {stderr}"
        );
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
    executable(
        directory.join("git-fixtureexternal").as_path(),
        b"#!/bin/sh\necho ran\n",
    );
    let path: String = format!("{}:/usr/bin:/bin", directory.display());
    let environment: Vec<(OsString, OsString)> =
        vec![(OsString::from("PATH"), OsString::from(path))];
    let context: GitContext = context_with_aliases(&[("fixtureexternal", "status --short")]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        environment.as_slice(),
        &args(&["fixtureexternal"]),
    );
    assert_eq!(resolved.route, Route::Unresolved);
    assert_eq!(resolved.word(), b"fixtureexternal");
    // Git runs the program, so the alias's own command is not what the caller gets.
    let mut command: std::process::Command =
        std::process::Command::new(crate::test_support::REAL_GIT);
    command
        .current_dir(root.as_path())
        .env_clear()
        .env("PATH", directory.display().to_string())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args([
            "-c",
            "alias.fixtureexternal=status --short",
            "fixtureexternal",
        ]);
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
    let followed: ResolvedCommand = resolved_from(
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
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["fixtureshell"]),
    );
    assert_eq!(resolved.route, Route::ShellAlias);
    assert_eq!(
        resolved.shell_command,
        Some(args(&["reset", "--hard"])),
        "the shell's own command is read"
    );
    assert_eq!(
        resolved.arguments,
        args(&["fixtureshell"]),
        "the caller's arguments stay untouched"
    );
    // A nested alias named after `git` is the command the shell runs, so the chain is followed.
    let chained: GitContext = context_with_aliases(&[
        ("fixtureshell", "!git fixturewipe"),
        ("fixturewipe", "reset --hard"),
    ]);
    let followed: ResolvedCommand = resolved_from(
        &chained,
        no_environment().as_slice(),
        &args(&["fixtureshell"]),
    );
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
fn real_git_stdout(
    directory: &Path,
    path: &str,
    arguments: &[&str],
) -> String {
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
    git(
        root.as_path(),
        &["init", "--quiet", "--initial-branch=main", "."],
    );
    let directory: PathBuf = root.join("sub/bin");
    std::fs::create_dir_all(directory.as_path()).expect("program directory");
    executable(
        directory.join("git-fixturewipe").as_path(),
        b"#!/bin/sh\necho external\n",
    );
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
        Some(root.as_path()),
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
    assert!(
        !stdout.contains("external"),
        "Git ran the program under sub/bin: {stdout}"
    );
    // The positive control: an absolute entry reaches the program, the reading sees it and Git runs it.
    let absolute: String = format!("{}:/usr/bin:/bin", directory.display());
    let reachable: Vec<(OsString, OsString)> =
        vec![(OsString::from("PATH"), OsString::from(absolute.as_str()))];
    let external: ResolvedCommand = resolve_forwarded_command(
        &context,
        reachable.as_slice(),
        &args(&["-C", "sub", "fixtureouter"]),
        Some(root.as_path()),
    );
    assert_eq!(external.route, Route::ShellAlias);
    assert_eq!(
        external.shell_command, None,
        "the program is reachable, so Git runs it"
    );
    let reachable_stdout: String =
        real_git_stdout(root.as_path(), absolute.as_str(), invocation.as_slice());
    assert!(
        reachable_stdout.contains("external"),
        "the control did not reach the program, so the relative case proves nothing: {reachable_stdout}"
    );
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
        let resolved: ResolvedCommand = resolved_from(
            &context,
            no_environment().as_slice(),
            &args(&["fixtureshell"]),
        );
        assert_eq!(resolved.route, Route::ShellAlias, "{value}");
        assert_eq!(resolved.shell_command, None, "{value} must stay unread");
        assert_eq!(resolved.arguments, args(&["fixtureshell"]), "{value}");
    }
}

/// What:
///  The bytes the shell reads as they are:
///  the letters,
///  the digits,
///  the ten bytes of ordinary arguments,
///  and the two blanks,
///  which separate two words rather than standing in one.
///  Written out here rather than read from the reader,
///  so a widening of that list is a row that fails.
/// Why:
///  The table below is the whole alphabet,
///  and a set taken from `is_plain_shell_byte` would agree with whatever that function says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const READ = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.:/=@+, \t';
/// ```
const BYTES_THE_SHELL_READS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.:/=@+, \t";

/// What:
///  One `!git` value that holds the byte inside its second word,
///  where the shell would keep the byte in that word if it passed the byte through.
/// Why:
///  A byte at the end of a value would leave it unread for the value's shape and not for the byte,
///  so the row would pass whatever the byte meant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const valueHolding = (byte) => new TextEncoder().encode(`!git reset --har${String.fromCharCode(byte)}d`);
/// ```
fn a_git_value_holding(byte: u8) -> Vec<u8> {
    let mut value: Vec<u8> = b"!git reset --har".to_vec();
    value.push(byte);
    value.extend_from_slice(b"d");
    return value;
}

/// What:
///  Every byte of the alphabet is read or left unread as the shell would read it:
///  the letters, digits and the ten bytes of ordinary arguments are read,
///  and every other byte leaves the value unread.
/// Why:
///  `is_plain_shell_byte` is the one list of bytes the shell-alias reading trusts,
///  so a widening by any byte decides about a command nobody runs:
///  a quote, a backslash, a brace, a parenthesis, `~`, `#`, a carriage return or a line feed
///  left the old controls green (the alpha.50 review's L3).
///  This table names every byte,
///  so a widening shows as the rows whose byte is now read and a narrowing as the row that is
///  no longer read:
///  with one byte widened into the list, that byte's row fails and no other row does.
///  A blank byte splits the value into two words rather than standing in one,
///  and the value is read either way,
///  which `a_shell_alias_is_read_in_every_form_the_shell_passes_through` holds for the two
///  blanks the shell separates on.
#[test]
fn every_byte_the_shell_does_not_pass_through_leaves_a_git_value_unread() {
    // Every offending byte is collected before the claim is made, so one widening reports each byte
    //  it read and the table it broke, rather than stopping at the first.
    let mut read_through: Vec<u8> = Vec::new();
    let mut left_unread: Vec<u8> = Vec::new();
    for byte in 0..=u8::MAX {
        let value: Vec<u8> = a_git_value_holding(byte);
        let read: bool =
            crate::alias_value::shell_git_words(value.as_slice()).is_some();
        if read && !BYTES_THE_SHELL_READS.contains(&byte) {
            read_through.push(byte);
        }
        if !read && BYTES_THE_SHELL_READS.contains(&byte) {
            left_unread.push(byte);
        }
    }
    assert!(
        read_through.is_empty(),
        "each of these bytes means something to the shell, so a value holding one stays unread: {read_through:?}"
    );
    assert!(
        left_unread.is_empty(),
        "each of these bytes is read as it is, so a value holding one is read: {left_unread:?}"
    );
}

/// What:
///  Every spelling made only of forms the shell passes through is read as Git runs
///  it,
///  and real Git is asked the same shape beside each reading.
/// Why:
///  The alpha.49 release pass found five such spellings unread:
///  a `!` alias whose value names another `!` alias,
///  a value written `! git`,
///  a TAB after `!git`,
///  a bare `!git` whose caller's own words begin with a global option,
///  and a chain of aliases longer than the reading followed.
///  Git answers each shape with its version inside the shell,
///  so a reading that answered no command would be wrong about the command Git
///  runs;
///  the chain case beside them shows the count the reading does keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const { value, caller, command, run } of readable) {
///   expect(realGitVerbose(value, caller)).toContain('git version');
///   expect(resolved(value, caller).shellCommand).toEqual(command);
/// }
/// ```
#[test]
fn a_shell_alias_is_read_in_every_form_the_shell_passes_through() {
    // What: One spelling: the value the reading reads, the caller's own words, the command the shell of that
    //  value runs, and the same shape written so Git prints its version instead of running a command.
    // Why:  One row answers both questions, so a reading that drifts from Git's own shell fails here.
    let readable: Vec<(&str, Vec<&str>, Vec<&str>, &str, Vec<&str>)> = vec![
        // A blank between `!` and `git` is one separator to the shell.
        (
            "! git reset --hard",
            vec![],
            vec!["reset", "--hard"],
            "! git --version",
            vec![],
        ),
        // A TAB separates the words of a shell command as a space does.
        (
            "!git\treset --hard",
            vec![],
            vec!["reset", "--hard"],
            "!git\t--version",
            vec![],
        ),
        // A run of blanks, spaces and TABs, is still one separator.
        (
            "!  git \t reset --hard",
            vec![],
            vec!["reset", "--hard"],
            "!  git \t --version",
            vec![],
        ),
        // A value of `!git` alone leaves the command to the caller's own words, which may begin with a global.
        (
            "!git",
            vec!["--no-pager", "reset", "--hard"],
            vec!["--no-pager", "reset", "--hard"],
            "!git",
            vec!["--no-pager", "--version"],
        ),
    ];
    for (value, caller, command, run, run_caller) in &readable {
        let context: GitContext = context_with_aliases(&[("fixtureshell", value)]);
        let mut words: Vec<&str> = vec!["fixtureshell"];
        words.extend(caller.iter().copied());
        let resolved: ResolvedCommand = resolved_from(
            &context,
            no_environment().as_slice(),
            &args(words.as_slice()),
        );
        assert_eq!(resolved.route, Route::ShellAlias, "{value}");
        assert_eq!(
            resolved.shell_command.as_deref(),
            Some(args(command).as_slice()),
            "{value} is read as the shell runs it"
        );
        // What: The same shape in real Git, asking the version so the probe writes nothing.
        // Why:  The answer comes from the `git` inside the shell, which is the command this reading follows.
        let asked: String = format!("alias.fixtureshell={run}");
        let mut invocation: Vec<&str> = vec!["-c", asked.as_str(), "fixtureshell"];
        invocation.extend(run_caller.iter().copied());
        let stdout: String = real_git_stdout(
            outside_every_repository(),
            "/usr/bin:/bin",
            invocation.as_slice(),
        );
        assert!(
            stdout.contains("git version"),
            "{value}: Git did not run the command the reading follows: {stdout}"
        );
    }
    // The shell alias that names another shell alias: the outer shell runs the inner alias, whose own
    //  shell runs the command Git finally runs.
    let doubled: GitContext = context_with_aliases(&[
        ("fixtureshell", "!git fixturesecond"),
        ("fixturesecond", "!git reset --hard"),
    ]);
    let followed: ResolvedCommand = resolved_from(
        &doubled,
        no_environment().as_slice(),
        &args(&["fixtureshell"]),
    );
    assert_eq!(followed.route, Route::ShellAlias);
    assert_eq!(followed.shell_command, Some(args(&["reset", "--hard"])));
    let stdout: String = real_git_stdout(
        outside_every_repository(),
        "/usr/bin:/bin",
        &[
            "-c",
            "alias.fixtureshell=!git fixturesecond",
            "-c",
            "alias.fixturesecond=!git --version",
            "fixtureshell",
        ],
    );
    assert!(
        stdout.contains("git version"),
        "Git did not run the second shell the reading follows: {stdout}"
    );
}

/// The chain of 17 links the second pass over alpha.49 found unread,
///  one link more than the count was then,
///  so a control reads the depth that pass named as a gap.
const A_CHAIN_PAST_THE_COUNT_THE_PASS_FOUND: usize = 17;

/// The chain of 30 links real Git ran while that pass measured it,
///  which no count Git keeps would stop.
const A_CHAIN_REAL_GIT_RAN: usize = 30;

/// What:
///  One chain of `links` aliases,
///  each naming the next,
///  whose last runs `status --short`,
///  and the name of the first.
/// Why:
///  A chain is read one link at a time,
///  so a control needs a chain of an exact depth on both sides of the reading and
///  in real Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [pairs, first, words] = alias_chain(17);
/// ```
fn alias_chain(links: usize) -> (Vec<(String, String)>, OsString, Vec<OsString>) {
    let mut names: Vec<String> = Vec::new();
    for index in 0..links {
        names.push(format!("fixturealischain{index}"));
    }
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut words: Vec<OsString> = Vec::new();
    for index in 0..links {
        let target: String = if index + 1 < links {
            names[index + 1].clone()
        } else {
            String::from("status --short")
        };
        pairs.push((names[index].clone(), target));
        words.push(OsString::from("-c"));
        words.push(OsString::from(format!(
            "alias.{}={}",
            names[index], pairs[index].1
        )));
    }
    words.push(OsString::from(names[0].as_str()));
    return (pairs, OsString::from(names[0].as_str()), words);
}

/// What:
///  A chain of ordinary aliases is read as deep as real Git runs it,
///  so the count the reader keeps bounds its own work and not Git.
/// Why:
///  The pass over alpha.49 found a chain of 17 links unread,
///  and Git has no count at all:
///  it ends a chain by reaching a name it has already expanded.
///  This control reads the depth that pass named,
///  the depth it measured real Git running,
///  and the count the reader keeps,
///  and has real Git run each of them beside the reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const links of [17, 30, MAX_ALIAS_EXPANSIONS]) {
///   expect(realGit(aliasChain(links))).toBe('');
///   expect(resolved(aliasChain(links)).word()).toBe('status');
/// }
/// ```
#[test]
fn an_alias_chain_is_read_as_deep_as_git_runs_it() {
    let root: PathBuf = fixture("alias-chain-depth");
    git(
        root.as_path(),
        &["init", "--quiet", "--initial-branch=main", "."],
    );
    for links in [
        A_CHAIN_PAST_THE_COUNT_THE_PASS_FOUND,
        A_CHAIN_REAL_GIT_RAN,
        MAX_ALIAS_EXPANSIONS,
    ] {
        let (pairs, first, words): (Vec<(String, String)>, OsString, Vec<OsString>) =
            alias_chain(links);
        let mut borrowed: Vec<(&str, &str)> = Vec::new();
        for (name, value) in &pairs {
            borrowed.push((name.as_str(), value.as_str()));
        }
        let context: GitContext = context_with_aliases(borrowed.as_slice());
        let resolved: ResolvedCommand =
            resolved_from(&context, no_environment().as_slice(), &[first]);
        assert_eq!(resolved.route, Route::Alias, "{links} links");
        assert_eq!(resolved.word(), b"status", "{links} links are read");
        // The same chain in real Git, in a repository it can answer `status` in: an empty output
        //  is a clean tree, and a failure is Git not reaching the command at all.
        let asked: Vec<&OsStr> = words.iter().map(OsStr::new).collect();
        let output: Output = git_output(root.as_path(), asked.as_slice());
        assert!(
            output.status.success(),
            "{links} links: Git did not run the alias chain: {}",
            String::from_utf8_lossy(output.stderr.as_slice())
        );
    }
    remove(root.as_path());
}

/// What:
///  A chain of `length` shell aliases,
///  each naming the next with `!git <name>`,
///  the last running `!git reset --hard`,
///  and the name of the first.
/// Why:
///  Every shell in the chain starts another `git`,
///  so the reading follows them one at a time and has to stop somewhere;
///  this fixture puts both sides of that depth in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [context, first] = shellChainContext(MAX_SHELL_ALIAS_DEPTH);
/// ```
fn shell_chain_context(length: usize) -> (GitContext, OsString) {
    let mut names: Vec<String> = Vec::new();
    for index in 0..length {
        names.push(format!("fixtureshellchain{index}"));
    }
    let mut pairs: Vec<(String, String)> = Vec::new();
    for index in 0..length {
        let target: String = if index + 1 < length {
            format!("!git {}", names[index + 1])
        } else {
            String::from("!git reset --hard")
        };
        pairs.push((names[index].clone(), target));
    }
    let mut pair_refs: Vec<(&str, &str)> = Vec::new();
    for (name, target) in &pairs {
        pair_refs.push((name.as_str(), target.as_str()));
    }
    let first: OsString = OsString::from(names[0].as_str());
    return (context_with_aliases(pair_refs.as_slice()), first);
}

/// What:
///  A chain of shell aliases as deep as the reading follows is read to the command
///  Git runs,
///  and one shell deeper is read as no command at all.
/// Why:
///  The depth is what stops a value that names itself from starting shells without
///  end,
///  so both sides of it are pinned:
///  the deepest chain read still answers the command,
///  and a chain past it answers no command rather than a wrong one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolved(shellChain(MAX_SHELL_ALIAS_DEPTH)).shellCommand // ['reset', '--hard']
/// resolved(shellChain(MAX_SHELL_ALIAS_DEPTH + 1)).shellCommand // undefined
/// ```
#[test]
fn a_shell_alias_chain_deeper_than_the_reading_follows_is_unread() {
    let (within, first): (GitContext, OsString) = shell_chain_context(MAX_SHELL_ALIAS_DEPTH);
    let followed: ResolvedCommand = resolved_from(&within, no_environment().as_slice(), &[first]);
    assert_eq!(followed.route, Route::ShellAlias);
    assert_eq!(
        followed.shell_command,
        Some(args(&["reset", "--hard"])),
        "the deepest chain the reading follows reaches the command"
    );
    let (past, first_of_past): (GitContext, OsString) =
        shell_chain_context(MAX_SHELL_ALIAS_DEPTH + 1);
    let unread: ResolvedCommand =
        resolved_from(&past, no_environment().as_slice(), &[first_of_past]);
    assert_eq!(unread.route, Route::ShellAlias);
    assert_eq!(unread.shell_command, None, "a shell deeper is not read");
    assert_eq!(the_command_git_runs(&unread), None);
}

/// A built-in named directly is resolved without an alias lookup and keeps its global options.
#[test]
fn a_direct_builtin_resolves_past_its_global_options() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["-C", "/x", "status"]),
    );
    assert_eq!(resolved.route, Route::Direct);
    assert_eq!(resolved.word(), b"status");
    assert_eq!(resolved.subcommand_index, 2);
    assert!(resolved.is_builtin());
}

/// A bare `git --version` has no subcommand and is not a built-in command.
#[test]
fn a_query_has_no_subcommand() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand =
        resolved_from(&context, no_environment().as_slice(), &args(&["--version"]));
    assert_eq!(resolved.subcommand, None);
    assert_eq!(resolved.word(), b"");
    assert!(!resolved.is_builtin());
}

/// An alias to a built-in expands to its words,
///  with the rest of the original arguments after them.
#[test]
fn an_alias_expands_to_its_words_then_the_rest() {
    let context: GitContext = context_with_aliases(&[("fixturecommit", "commit -v")]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["fixturecommit", "-m", "x"]),
    );
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.word(), b"commit");
    assert_eq!(resolved.arguments, args(&["commit", "-v", "-m", "x"]));
    assert!(resolved.is_builtin());
}

/// Chained aliases expand one after another until a built-in is reached.
#[test]
fn chained_aliases_resolve_to_the_builtin_at_the_end() {
    let context: GitContext =
        context_with_aliases(&[("fixturex", "fixturey -a"), ("fixturey", "commit")]);
    let resolved: ResolvedCommand =
        resolved_from(&context, no_environment().as_slice(), &args(&["fixturex"]));
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.arguments, args(&["commit", "-a"]));
}

/// A built-in name is never replaced by an alias of the same name,
///  as Git ignores such aliases.
#[test]
fn a_builtin_name_is_not_shadowed_by_an_alias() {
    let context: GitContext = context_with_aliases(&[("commit", "status")]);
    let resolved: ResolvedCommand =
        resolved_from(&context, no_environment().as_slice(), &args(&["commit"]));
    assert_eq!(resolved.route, Route::Direct);
    assert_eq!(resolved.word(), b"commit");
}

/// A `!` alias runs a shell command,
///  so it is never a built-in whatever its text says.
#[test]
fn a_shell_alias_is_never_a_builtin() {
    let context: GitContext = context_with_aliases(&[("fixtureshell", "!git commit -a")]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["fixtureshell"]),
    );
    assert_eq!(resolved.route, Route::ShellAlias);
    assert!(!resolved.is_builtin());
    assert!(!creates_worktrees(&resolved));
}

/// An alias that names itself,
///  directly or through another,
///  is an unresolved loop.
#[test]
fn an_alias_loop_is_unresolved() {
    let context: GitContext = context_with_aliases(&[
        ("fixtureloopa", "fixtureloopb"),
        ("fixtureloopb", "fixtureloopa"),
    ]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["fixtureloopa"]),
    );
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// An unknown name with no alias is unresolved and never a built-in.
#[test]
fn an_unknown_name_is_unresolved() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["fixture-no-such-command"]),
    );
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// A value whose quoting cannot be split is unresolved,
///  not guessed.
#[test]
fn a_malformed_alias_value_is_unresolved() {
    let context: GitContext = context_with_aliases(&[("fixturebad", "'commit")]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["fixturebad"]),
    );
    assert_eq!(resolved.route, Route::Unresolved);
}

/// An empty alias value has no command word and is unresolved.
#[test]
fn an_empty_alias_value_is_unresolved() {
    let context: GitContext = context_with_aliases(&[("fixtureempty", "   ")]);
    let resolved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["fixtureempty"]),
    );
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
    return (
        context_with_aliases(pair_refs.as_slice()),
        OsString::from(names[0].as_str()),
    );
}

/// A chain as long as the limit,
///  each alias one expansion,
///  reaches the built-in.
#[test]
fn the_expansion_limit_reaches_the_builtin() {
    let (context, first): (GitContext, OsString) = chain_context(MAX_ALIAS_EXPANSIONS);
    let resolved: ResolvedCommand = resolved_from(&context, no_environment().as_slice(), &[first]);
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.word(), b"commit");
}

/// What:
///  One chained alias more than the reading follows,
///  and the answer is that these words are no command at all:
///  no subcommand,
///  nothing the guard or the slot refusals can read.
/// Why:
///  The chain stopped one link before its end,
///  so a word of the half-expanded vector is an alias's name and not the command
///  Git runs;
///  a vector whose word happened to be `worktree` would otherwise let a planner
///  serve a removal the wrapper never read.
#[test]
fn one_expansion_past_the_limit_is_unread() {
    let (context, first): (GitContext, OsString) = chain_context(MAX_ALIAS_EXPANSIONS + 1);
    let resolved: ResolvedCommand = resolved_from(&context, no_environment().as_slice(), &[first]);
    assert_eq!(resolved.route, Route::Unread);
    assert_eq!(resolved.word(), b"");
    assert_eq!(resolved.subcommand, None);
    assert!(!resolved.is_builtin());
    assert_eq!(the_command_git_runs(&resolved), None);
}

/// Only `worktree add` creates a linked worktree;
/// a move registers nothing new,
/// and other forms do not create one.
#[test]
fn only_worktree_add_creates_worktrees() {
    let context: GitContext = context_with_aliases(&[]);
    let add: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["worktree", "add", "x"]),
    );
    let moved: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["worktree", "move", "a", "b"]),
    );
    let verbose_add: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["worktree", "--verbose", "add", "x"]),
    );
    let list: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["worktree", "list"]),
    );
    let bare: ResolvedCommand =
        resolved_from(&context, no_environment().as_slice(), &args(&["worktree"]));
    let other: ResolvedCommand = resolved_from(
        &context,
        no_environment().as_slice(),
        &args(&["status", "add"]),
    );
    assert!(creates_worktrees(&add));
    assert!(!creates_worktrees(&moved));
    assert!(creates_worktrees(&verbose_add));
    assert!(!creates_worktrees(&list));
    assert!(!creates_worktrees(&bare));
    assert!(!creates_worktrees(&other));
}
