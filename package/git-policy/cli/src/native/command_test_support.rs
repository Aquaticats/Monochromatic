//! What:
//!  Argument builders,
//!  disposable real-Git fixtures and table oracles for unit tests.
//! Why:
//!  Every command module is checked against the Git 2.56.0 binary in the verification
//!      image,
//!  inside a fresh directory,
//!  never against the real home or global configuration.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const root = await mkdtemp(join(tmpdir(), 'native-')); await git(root, ['init']);
//! ```

/// Tokenizer types used by the shared synthetic table and the table oracles.
use super::command_options::{
    Arity, DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, parse_options, row,
};
/// The real Git path and fixture removal are shared with the wrapper's unit tests unchanged.
/// The fixture constructor and Git runners stay separate on purpose:
///  their directory prefix
/// keeps command fixtures apart from wrapper fixtures in one test process,
///  and the runners
/// also fix `GIT_EDITOR` and close standard input.
pub(crate) use super::test_support::{REAL_GIT, remove};
/// Native string,
///  path,
///  stream and process types used by the fixtures.
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Identifiers of the synthetic rows shared by the tokenizer test files.
pub(crate) const ALL: u16 = 1;
pub(crate) const MESSAGE: u16 = 2;
pub(crate) const UNTRACKED: u16 = 3;
pub(crate) const CONTAINS: u16 = 4;
pub(crate) const HARD: u16 = 5;
pub(crate) const NO_VERIFY: u16 = 6;
pub(crate) const AMEND: u16 = 7;
pub(crate) const ALLOW_EMPTY: u16 = 8;
pub(crate) const ALLOW_EMPTY_MESSAGE: u16 = 9;

/// One row per arity and negation class Git has.
pub(crate) const SYNTHETIC_TABLE: &[OptionSpec] = &[
    row(ALL, Some(b'a'), Some("all"), Arity::None, true),
    row(MESSAGE, Some(b'm'), Some("message"), Arity::Required, true),
    row(
        UNTRACKED,
        Some(b'u'),
        Some("untracked-files"),
        Arity::Optional,
        true,
    ),
    row(
        CONTAINS,
        None,
        Some("contains"),
        Arity::LastArgDefault,
        false,
    ),
    row(HARD, None, Some("hard"), Arity::None, false),
    row(NO_VERIFY, Some(b'n'), Some("no-verify"), Arity::None, true),
    row(AMEND, None, Some("amend"), Arity::None, true),
    row(ALLOW_EMPTY, None, Some("allow-empty"), Arity::None, true),
    row(
        ALLOW_EMPTY_MESSAGE,
        None,
        Some("allow-empty-message"),
        Arity::None,
        true,
    ),
];

/// Own a fixture argument vector without changing any spelling.
pub(crate) fn os_arguments(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// Tokenize text arguments against the synthetic table with Git's default flags.
pub(crate) fn parse_synthetic(values: &[&str]) -> Result<ParsedOptions, OptionError> {
    return parse_options(
        os_arguments(values).as_slice(),
        SYNTHETIC_TABLE,
        DEFAULT_MODE,
        &[],
    );
}

/// Build one argument from raw bytes that need not be UTF-8.
#[cfg(unix)]
pub(crate) fn byte_argument(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    return OsString::from_vec(bytes.to_vec());
}

/// Create one empty fixture directory owned by this test process and test name.
pub(crate) fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-command", name);
}

/// Start real Git in a fixture with no system or global configuration and a fixed identity.
fn git_command(directory: &Path) -> Command {
    let mut command: Command = Command::new(REAL_GIT);
    command
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_EDITOR", "true");
    return command;
}

/// Run real Git with native arguments and report its status and streams without judging them.
pub(crate) fn git_output(directory: &Path, arguments: &[OsString]) -> Output {
    return git_command(directory)
        .args(arguments)
        .stdin(Stdio::null())
        .output()
        .expect("native Git fixture command");
}

/// Run real Git with text arguments and require success.
pub(crate) fn git(directory: &Path, arguments: &[&str]) -> Output {
    let output: Output = git_output(directory, os_arguments(arguments).as_slice());
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    return output;
}

/// Run real Git with text arguments where failure is the observation.
pub(crate) fn git_status(directory: &Path, arguments: &[&str]) -> Output {
    return git_output(directory, os_arguments(arguments).as_slice());
}

/// Run real Git with bytes on standard input,
///  as `git rev-parse --parseopt` needs.
pub(crate) fn git_with_input(directory: &Path, arguments: &[OsString], input: &[u8]) -> Output {
    let mut child: std::process::Child = git_command(directory)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("native Git fixture command");
    child
        .stdin
        .take()
        .expect("piped standard input")
        .write_all(input)
        .expect("write option specification");
    return child.wait_with_output().expect("native Git fixture result");
}

/// The repository helper of the unit tests, with one empty commit on `main`.
pub(crate) use crate::test_support::repository;

/// Create a fixture whose repository has `tracked.txt` committed;
///  returns the fixture
/// directory to remove and the repository root.
pub(crate) fn repository_with_tracked_file(name: &str) -> (PathBuf, PathBuf) {
    let directory: PathBuf = fixture(name);
    let root: PathBuf = repository(directory.as_path(), "repository");
    std::fs::write(root.join("tracked.txt"), b"one\n").expect("write tracked file");
    git(root.as_path(), &["add", "--", "tracked.txt"]);
    git(
        root.as_path(),
        &["commit", "--quiet", "--message=base", "--", "tracked.txt"],
    );
    return (directory, root);
}

/// Leave the repository in a conflicted merge of `tracked.txt`,
///  with `MERGE_HEAD` present.
pub(crate) fn start_conflicted_merge(root: &Path) {
    git(root, &["switch", "--quiet", "--create", "other"]);
    std::fs::write(root.join("tracked.txt"), b"other\n").expect("write other side");
    git(
        root,
        &["commit", "--quiet", "--message=other", "--", "tracked.txt"],
    );
    git(root, &["switch", "--quiet", "main"]);
    std::fs::write(root.join("tracked.txt"), b"main\n").expect("write main side");
    git(
        root,
        &["commit", "--quiet", "--message=main", "--", "tracked.txt"],
    );
    let merge: Output = git_status(root, &["merge", "--quiet", "other"]);
    assert!(!merge.status.success(), "the merge must conflict");
}

/// Standard output of a command as trimmed text.
pub(crate) fn output_text(output: &Output) -> String {
    return String::from_utf8_lossy(&output.stdout).trim().to_owned();
}

/// Require what Git's `parse_options_check` requires of a table (parse-options.c:642-736):
/// unique short letters,
///  unique long names,
///  and spellings that can introduce an option.
pub(crate) fn assert_table_invariants(table: &[OptionSpec]) {
    let mut shorts: Vec<u8> = Vec::<u8>::new();
    let mut longs: Vec<&str> = Vec::<&str>::new();
    for spec in table {
        assert!(spec.short.is_some() || spec.long.is_some(), "{spec:?}");
        if let Some(short) = spec.short {
            assert!(short.is_ascii_graphic() && short != b'-', "{spec:?}");
            assert!(!shorts.contains(&short), "duplicate short {spec:?}");
            shorts.push(short);
        }
        if let Some(long) = spec.long {
            assert!(!long.is_empty() && !long.contains('='), "{spec:?}");
            assert!(!long.starts_with('-'), "{spec:?}");
            assert!(!longs.contains(&long), "duplicate long {spec:?}");
            longs.push(long);
        }
    }
}
