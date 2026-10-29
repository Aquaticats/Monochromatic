//! What:
//!  Exemptions and verdicts of the require-root decision,
//!  with every case of
//!       `require-root.unit.test.ts` restated as the facts its fixture would measure.
//! Why:
//!  The incumbent's fixtures build repository shapes on disk;
//!  the native core receives
//!      the measured root instead,
//!  so each fixture becomes the measurement it produces.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(resolveRequireRoot({ effectiveDirectory: '/r/sub', repositoryRoot: '/r' }).kind).toBe('not-at-root');
//! ```

/// The decision,
///  its result types and the argument and real-Git fixture helpers.
use super::RequireRootExemption::{ConfigScope, NoCommand, Subcommand};
use super::{
    NOT_AT_ROOT_CODE, RequireRootDecision, RequireRootFacts, RequireRootVerdict,
    RequireRootViolation, decide_require_root, resolve_require_root,
};
use crate::command_test_support::{fixture, git, os_arguments, remove, repository};
use std::path::PathBuf;
use std::process::Output;

/// The wrapper-only escape hatch of this policy,
///  as the caller passes it.
const HATCH: &[u8] = b"--no-enforce-require-root";

/// Decide a space-separated argument list with the policy's own wrapper flag.
fn decide(line: &str) -> RequireRootDecision {
    let values: Vec<&str> = line.split_whitespace().collect();
    return decide_require_root(os_arguments(values.as_slice()).as_slice(), &[HATCH]);
}

/// Measurements with a known repository root,
///  or with none.
fn facts(directory: &str, root: Option<&str>) -> RequireRootFacts {
    let mut repository_root: Option<PathBuf> = None;
    if let Some(found) = root {
        repository_root = Some(PathBuf::from(found));
    }
    return RequireRootFacts {
        effective_directory: PathBuf::from(directory),
        repository_root,
    };
}

/// The rejection text of the incumbent,
///  verbatim.
fn violation(root: &str, directory: &str) -> RequireRootVerdict {
    return RequireRootVerdict::NotAtRoot(RequireRootViolation {
        message: format!(
            "cli-git: not at the root of the git repository. Repo root is {root} but effective cwd is {directory}. This rule exists because agents and humans frequently forget which directory they are in. Tip: cd to {root} or pass -C {root} before the subcommand."
        ),
    });
}

/// Ported:
///  every incumbent case,
///  as the decision on its arguments plus its measurements.
#[test]
fn ports_the_incumbent_cases() {
    // The fixtures run `-C <directory> status`; a nonexempt command needs the measurement.
    assert_eq!(
        decide("-C /fixture/x status"),
        RequireRootDecision::NeedsRepositoryRoot
    );
    // Outside a repository; and below an empty `.git` or a malformed gitfile, which the
    // measurement does not accept as a repository.
    assert_eq!(
        resolve_require_root(&facts("/fixture/x", None)),
        RequireRootVerdict::Pass
    );
    // At the root, whether `.git` is a directory or a gitfile.
    assert_eq!(
        resolve_require_root(&facts("/r", Some("/r"))),
        RequireRootVerdict::Pass
    );
    // Below the root, whether `.git` is a directory or a gitfile.
    assert_eq!(
        resolve_require_root(&facts("/r/subdir", Some("/r"))),
        violation("/r", "/r/subdir")
    );
    assert_eq!(
        decide("-C /r/subdir clone https://example.invalid/repo.git"),
        RequireRootDecision::Exempt(Subcommand)
    );
    assert_eq!(
        decide("-C /r/subdir config --global user.name"),
        RequireRootDecision::Exempt(ConfigScope)
    );
    assert_eq!(NOT_AT_ROOT_CODE, "not-at-root");
}

/// The four exempt subcommands,
///  exactly as spelled,
///  after any global prefix.
#[test]
fn exempts_commands_that_need_no_repository() {
    for line in [
        "init",
        "clone x",
        "version",
        "help",
        "-C sub init",
        "-c a.b=c --no-pager help status",
    ] {
        assert_eq!(
            decide(line),
            RequireRootDecision::Exempt(Subcommand),
            "{line}"
        );
    }
    for line in [
        "status",
        "initialize",
        "Init",
        "-C sub status",
        "commit -m init",
        "x-help",
    ] {
        assert_eq!(
            decide(line),
            RequireRootDecision::NeedsRepositoryRoot,
            "{line}"
        );
    }
}

/// Divergence:
///  when Git runs no subcommand at all,
///  nothing needs a root.
#[test]
fn exempts_arguments_that_run_no_subcommand() {
    for line in [
        "",
        "-C sub",
        "--version status",
        "-v",
        "--help status",
        "--exec-path",
        "-C",
        "--bogus status",
        "--list-cmds=main",
    ] {
        assert_eq!(
            decide(line),
            RequireRootDecision::Exempt(NoCommand),
            "{line}"
        );
    }
}

/// `git config` is exempt for the per-user or system file and for listing,
///  in option
/// position only;
///  a region Git refuses is not exempt.
#[test]
fn exempts_config_by_its_file_scope() {
    for line in [
        "config --global user.name",
        "config --system user.name",
        "config --list",
        "config -l",
        "config -lz",
        "config --local --list",
        "config list",
        "config list --show-origin",
        "config get --global user.name",
        "config set --system a.b c",
        "config -- --global user.name",
        "config --no-enforce-require-root --global user.name",
        "config --no-enforce-require-root list",
        "-C sub config --glob user.name",
    ] {
        assert_eq!(
            decide(line),
            RequireRootDecision::Exempt(ConfigScope),
            "{line}"
        );
    }
    for line in [
        "config user.name",
        "config user.name --global",
        "config set user.name --system",
        "config --no-enforce-require-root set user.name --global",
        "config -f --global user.name",
        "config --global --no-global user.name",
        "config get user.name",
        "config --bogus --global",
        "config --global=1",
        "config edit",
    ] {
        assert_eq!(
            decide(line),
            RequireRootDecision::NeedsRepositoryRoot,
            "{line}"
        );
    }
    // Without the wrapper flag in the caller's list, Git would refuse the hatch spelling.
    let refused: RequireRootDecision = decide_require_root(
        os_arguments(&["config", "--no-enforce-require-root", "--global", "a"]).as_slice(),
        &[],
    );
    assert_eq!(refused, RequireRootDecision::NeedsRepositoryRoot);
}

/// Paths compare by components,
///  and non-UTF-8 directories still produce a diagnostic.
#[test]
fn compares_paths_by_component() {
    assert_eq!(
        resolve_require_root(&facts("/r/", Some("/r"))),
        RequireRootVerdict::Pass
    );
    assert_eq!(
        resolve_require_root(&facts("/r//", Some("/r"))),
        RequireRootVerdict::Pass
    );
    assert_eq!(
        resolve_require_root(&facts("/r/..", Some("/r"))),
        violation("/r", "/r/..")
    );
    assert_eq!(
        resolve_require_root(&facts("/", Some("/r"))),
        violation("/r", "/")
    );
}

/// A directory name that is not UTF-8 is shown with a replacement character.
#[cfg(unix)]
#[test]
fn reports_a_non_utf8_directory() {
    use crate::command_test_support::byte_argument;
    let directory: PathBuf = PathBuf::from(byte_argument(b"/r/sub\xff"));
    let verdict: RequireRootVerdict = resolve_require_root(&RequireRootFacts {
        effective_directory: directory,
        repository_root: Some(PathBuf::from("/r")),
    });
    assert_eq!(verdict, violation("/r", "/r/sub\u{fffd}"));
}

/// `git config list` and `git config --list` print the same thing,
///  so both are listing.
#[test]
fn config_list_spellings_match_git() {
    let directory: PathBuf = fixture("require-root-config-list");
    let root: PathBuf = repository(directory.as_path(), "repository");
    git(root.as_path(), &["config", "fixture.value", "one"]);
    let subcommand: Output = git(root.as_path(), &["config", "list"]);
    let option: Output = git(root.as_path(), &["config", "--list"]);
    let short: Output = git(root.as_path(), &["config", "-l"]);
    assert!(String::from_utf8_lossy(&subcommand.stdout).contains("fixture.value=one"));
    assert_eq!(subcommand.stdout, option.stdout);
    assert_eq!(subcommand.stdout, short.stdout);
    remove(directory.as_path());
}
