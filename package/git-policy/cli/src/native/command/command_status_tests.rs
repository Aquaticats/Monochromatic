//! What:
//!  `git status` facts in every spelling Git 2.56.0 accepts,
//!  the `advice.statusHints`
//!       override reading,
//!  and a real-Git control of the table.
//! Why:
//!  A wrapper note appended to short or porcelain output would break its parsers,
//!  and
//!      an unnoticed override would discard the caller's explicit choice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseStatusPostRegion(['--porcelain=v2']).isMachineReadable).toBe(true);
//! ```

/// The functions under test,
///  the oracles and the real-Git fixture helpers.
use super::{STATUS_TABLE, StatusRegion, has_status_hints_override, parse_status_region};
use crate::command_options::OptionErrorKind;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, git, git_status, os_arguments, output_text, remove,
    repository_with_tracked_file,
};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

/// Whether a region Git accepts is machine readable.
fn machine_readable(values: &[&str]) -> bool {
    return parse_status_region(os_arguments(values).as_slice(), &[])
        .expect("valid region")
        .machine_readable;
}

/// Incumbent spellings (`--porcelain`,
///  `--porcelain=`,
///  `-z`,
///  `-s`,
///  `--short`) and the
/// clusters and abbreviations Git also accepts.
#[test]
fn detects_machine_readable_formats() {
    for values in [
        vec!["--porcelain"],
        vec!["--porcelain=v2"],
        vec!["--porcelain=v1", "path"],
        vec!["-z"],
        vec!["-s"],
        vec!["--short"],
        vec!["--null"],
        vec!["-sb"],
        vec!["-bs"],
        vec!["-bz"],
        vec!["--por"],
        vec!["--shor"],
        vec!["--long", "--short"],
        vec!["--no-short", "-s"],
    ] {
        assert!(machine_readable(values.as_slice()), "{values:?}");
    }
}

/// Human-readable forms,
///  including a later `--long` or negation (Git's last option wins).
#[test]
fn detects_human_readable_formats() {
    for values in [
        vec![],
        vec!["path"],
        vec!["--long"],
        vec!["-b", "-v", "--show-stash"],
        vec!["--short", "--long"],
        vec!["-s", "--no-short"],
        vec!["--porcelain", "--no-porcelain"],
        vec!["-z", "--no-null"],
        // In a path position the tokens are not options.
        vec!["--", "--porcelain", "-s"],
        // `-us`: `s` is the untracked-files mode, not `--short`.
        vec!["-us"],
    ] {
        assert!(!machine_readable(values.as_slice()), "{values:?}");
    }
}

/// Optional-value options never take the next token,
///  and Git's refusals are reported.
#[test]
fn reads_optional_values_and_refusals() {
    assert!(machine_readable(&["--porcelain", "v2"]));
    assert!(machine_readable(&["--untracked-files", "-s"]));
    assert!(machine_readable(&["--column", "--ignored", "-M", "-s"]));
    for (values, kind) in [
        (vec!["--unknown"], OptionErrorKind::UnknownOption),
        (vec!["--short=1"], OptionErrorKind::UnexpectedValue),
        (vec!["--no-find-renames"], OptionErrorKind::UnknownOption),
        (vec!["--ignore"], OptionErrorKind::AmbiguousOption),
        // `--sh` prefixes both `--short` and `--show-stash` here, unlike in `git commit`.
        (vec!["--sh"], OptionErrorKind::AmbiguousOption),
        (vec!["-h"], OptionErrorKind::HelpRequested),
    ] {
        assert_eq!(
            parse_status_region(os_arguments(values.as_slice()).as_slice(), &[])
                .expect_err("refused")
                .kind,
            kind,
            "{values:?}"
        );
    }
    let wrapped: StatusRegion = parse_status_region(
        os_arguments(&["--cli-git-keep-going", "-s"]).as_slice(),
        &[b"--cli-git-keep-going"],
    )
    .expect("valid region");
    assert_eq!(wrapped.wrapper.len(), 1);
    assert!(wrapped.machine_readable);
}

/// Whether a global prefix sets the advice key.
fn overridden(values: &[&str]) -> bool {
    return has_status_hints_override(os_arguments(values).as_slice());
}

/// `-c` in valued,
///  bare and mixed-case forms;
///  `--config-env` in both forms.
#[test]
fn detects_the_advice_key_in_global_options() {
    for values in [
        vec!["-c", "advice.statusHints=true"],
        vec!["-c", "advice.statusHints"],
        vec!["-c", "Advice.StatusHints=true"],
        vec!["-c", "ADVICE.STATUSHINTS=false"],
        vec!["-c", "advice.statusHints="],
        // `-c` splits at the first `=`, so the value may contain more.
        vec!["-c", "advice.statusHints=a=b"],
        vec![
            "-C",
            "/repo",
            "-c",
            "a.b=c",
            "-c",
            "advice.statusHints=true",
            "--no-pager",
        ],
        vec!["--config-env=advice.statusHints=HINTS"],
        vec!["--config-env", "advice.statusHints=HINTS"],
    ] {
        assert!(overridden(values.as_slice()), "{values:?}");
    }
}

/// Other keys,
///  and the key's spelling in a position that is not a configuration parameter.
#[test]
fn ignores_everything_that_does_not_set_the_advice_key() {
    for values in [
        vec![],
        vec!["-c", "advice.statusHintsX=true"],
        vec!["-c", "advice.status=true"],
        vec!["-c", "a.b=advice.statusHints=true"],
        vec!["-C", "advice.statusHints=true"],
        vec!["--git-dir", "advice.statusHints"],
        vec!["--namespace", "-c", "--no-pager"],
        vec!["--config-env=advice.statusHints"],
        vec!["--config-env", "advice.statusHints"],
        vec!["--config-env=a.b=advice.statusHints"],
        // `--config-env` splits at the last `=`: the key is `advice.statusHints=x`.
        vec!["--config-env=advice.statusHints=x=HINTS"],
        vec!["-c"],
        vec!["--no-pager", "--bare"],
    ] {
        assert!(!overridden(values.as_slice()), "{values:?}");
    }
    // `-C -c advice...`: `-c` is the directory, so the next token is not its value.
    assert!(!overridden(&["-C", "-c", "advice.statusHints=true"]));
}

/// The key comparison never decodes bytes that are not UTF-8.
#[cfg(unix)]
#[test]
fn compares_keys_as_bytes() {
    use crate::command_test_support::byte_argument;
    let prefix: Vec<OsString> = vec![
        OsString::from("-c"),
        byte_argument(b"advice.statusHints=\xff"),
    ];
    assert!(has_status_hints_override(prefix.as_slice()));
    let other: Vec<OsString> = vec![
        OsString::from("-c"),
        byte_argument(b"advice.status\xffints=true"),
    ];
    assert!(!has_status_hints_override(other.as_slice()));
}

/// The copied table matches the binary,
///  and Git reads `--config-env` as setting the key.
#[test]
fn table_and_config_env_match_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("status-table");
    assert_table_invariants(STATUS_TABLE);
    assert_eq!(
        render_completion(STATUS_TABLE),
        git_completion(root.as_path(), &["status"])
    );
    // `PATH` is the one variable the fixture environment always defines.
    let from_environment: Output = git(
        root.as_path(),
        &[
            "--config-env=advice.statusHints=PATH",
            "config",
            "get",
            "advice.statusHints",
        ],
    );
    assert_eq!(output_text(&from_environment), "/usr/bin:/bin");
    // `-c` keeps everything after the first `=` as the value.
    let first: Output = git(
        root.as_path(),
        &[
            "-c",
            "advice.statusHints=a=b",
            "config",
            "get",
            "advice.statusHints",
        ],
    );
    assert_eq!(output_text(&first), "a=b");
    // `--config-env` takes the variable name after the last `=`, leaving an invalid key.
    let last: Output = git_status(
        root.as_path(),
        &[
            "--config-env=advice.statusHints=x=PATH",
            "config",
            "get",
            "advice.statusHints",
        ],
    );
    assert!(!last.status.success());
    assert!(String::from_utf8_lossy(&last.stderr).contains("invalid key: advice.statusHints=x"));
    remove(directory.as_path());
}
