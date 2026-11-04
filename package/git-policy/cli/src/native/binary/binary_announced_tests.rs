//! What:
//!  The text the built executable adds after real Git's answer to `--version`, `-v`, `--help` and `-h`.
//! Why:
//!  A caller must see Git's own output byte for byte, then the wrapper's lines, and must see nothing added
//!  when Git itself failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['--version']).stdout).toBe(run('/usr/bin/git', ['--version']).stdout + banner);
//! ```
#![cfg(unix)]

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, REAL_GIT, bounded, direct, executable, fixture, observe, remove, wrapped};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The crate version, read where the test is compiled.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The banner of `--version` and `-v` for this Git path.
fn banner(real_git: &Path) -> String {
    return format!("monochromatic-git-policy-cli {VERSION}, shadowing git at {}\n", real_git.display());
}

/// The section of `--help` and `-h` for this Git path.
fn help_section(real_git: &Path) -> String {
    return format!(
        "\nmonochromatic-git-policy-cli {VERSION} sits in front of this git\n\
         It adds repository policies on commit, push and worktree commands,\n\
         the `git cli-git` management command (`git cli-git --help`),\n\
         and `git cli-git doctor`, which lists the traditional hooks a repository holds.\n\
         real git: {}\n",
        real_git.display()
    );
}

/// Real Git's answer to these arguments and the wrapper's answer, run in the fixture root.
fn answers(fixture: &Fixture, arguments: &[&str]) -> (Observed, Observed) {
    let through_wrapper: Observed = observe(wrapped(fixture).current_dir(&fixture.root).args(arguments), b"");
    let through_git: Observed = observe(direct(fixture).current_dir(&fixture.root).args(arguments), b"");
    return (through_wrapper, through_git);
}

/// Both version spellings, with and without a global prefix, add one banner line after Git's line.
#[test]
fn version_spellings_add_one_banner_line() {
    let fixture: Fixture = fixture("announce-version");
    let banner_line: String = banner(Path::new(REAL_GIT));
    for arguments in [
        vec!["--version"],
        vec!["-v"],
        vec!["-C", "/", "--version"],
        vec!["--no-pager", "-c", "user.name=x", "-v"],
    ] {
        let (through_wrapper, through_git) = answers(&fixture, arguments.as_slice());
        assert_eq!(through_git.code, Some(0), "{arguments:?}");
        let mut expected: Vec<u8> = through_git.stdout.clone();
        expected.extend_from_slice(banner_line.as_bytes());
        assert_eq!(through_wrapper.stdout, expected, "{arguments:?}");
        assert_eq!(through_wrapper.stderr, through_git.stderr, "{arguments:?}");
        assert_eq!(through_wrapper.code, Some(0), "{arguments:?}");
    }
    remove(&fixture);
}

/// Both help spellings add the section after Git's usage text.
#[test]
fn help_spellings_add_the_section() {
    let fixture: Fixture = fixture("announce-help");
    let section: String = help_section(Path::new(REAL_GIT));
    for arguments in [vec!["--help"], vec!["-h"], vec!["-C", "/", "-h"]] {
        let (through_wrapper, through_git) = answers(&fixture, arguments.as_slice());
        assert_eq!(through_git.code, Some(0), "{arguments:?}");
        assert!(!through_git.stdout.is_empty(), "{arguments:?}");
        let mut expected: Vec<u8> = through_git.stdout.clone();
        expected.extend_from_slice(section.as_bytes());
        assert_eq!(through_wrapper.stdout, expected, "{arguments:?}");
        assert_eq!(through_wrapper.stderr, through_git.stderr, "{arguments:?}");
        assert_eq!(through_wrapper.code, Some(0), "{arguments:?}");
    }
    remove(&fixture);
}

/// Run the wrapper with a stand-in for real Git that prints one line on each stream and exits with `status`.
fn through_stand_in(fixture: &Fixture, arguments: &[&str], status: u8) -> (Observed, PathBuf) {
    std::fs::create_dir(fixture.root.join("probe")).expect("probe directory");
    let probe: PathBuf = fixture.root.join("probe/git");
    executable(
        probe.as_path(),
        format!("#!/bin/sh\necho probe-out\necho probe-err >&2\nexit {status}\n").as_bytes(),
    );
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(fixture.root.join("probe"));
    let observed: Observed =
        observe(bounded(fixture, fixture.root.join("bin/git").as_path(), path.as_os_str()).args(arguments), b"");
    return (observed, probe);
}

/// Git that exits with a nonzero status leaves the caller with Git's streams and status alone.
#[test]
fn a_failing_git_prints_nothing_extra() {
    for (name, arguments) in [
        ("announce-failing-version", vec!["--version"]),
        ("announce-failing-short", vec!["-v"]),
        ("announce-failing-help", vec!["--help"]),
        ("announce-failing-short-help", vec!["-h"]),
    ] {
        let fixture: Fixture = fixture(name);
        let (observed, _probe): (Observed, PathBuf) = through_stand_in(&fixture, arguments.as_slice(), 3);
        assert_eq!(observed.stdout, b"probe-out\n", "{arguments:?}");
        assert_eq!(observed.stderr, b"probe-err\n", "{arguments:?}");
        assert_eq!(observed.code, Some(3), "{arguments:?}");
        remove(&fixture);
    }
}

/// Positive control for the control above:
///  the same stand-in exiting 0 gets the text, naming the stand-in as the Git it shadows.
#[test]
fn a_succeeding_stand_in_gets_the_text_naming_its_path() {
    for (name, arguments, version) in [
        ("announce-stand-in-version", vec!["--version"], true),
        ("announce-stand-in-help", vec!["-h"], false),
    ] {
        let fixture: Fixture = fixture(name);
        let (observed, probe): (Observed, PathBuf) = through_stand_in(&fixture, arguments.as_slice(), 0);
        let mut expected: String = String::from("probe-out\n");
        expected.push_str(
            if version { banner(probe.as_path()) } else { help_section(probe.as_path()) }.as_str(),
        );
        assert_eq!(String::from_utf8_lossy(&observed.stdout), expected, "{arguments:?}");
        assert_eq!(observed.stderr, b"probe-err\n", "{arguments:?}");
        assert_eq!(observed.code, Some(0), "{arguments:?}");
        remove(&fixture);
    }
}

/// A subcommand's own help is untouched:
///  it is not a top-level query, so no section follows it.
#[test]
fn subcommand_help_stays_exact() {
    let fixture: Fixture = fixture("announce-subcommand");
    for arguments in [vec!["help", "--no-such-option"], vec!["version"]] {
        let (through_wrapper, through_git) = answers(&fixture, arguments.as_slice());
        assert_eq!(through_wrapper, through_git, "{arguments:?}");
    }
    remove(&fixture);
}
