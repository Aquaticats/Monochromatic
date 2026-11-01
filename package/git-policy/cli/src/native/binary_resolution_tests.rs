//! What:
//!  Self-exclusion and recursion-prevention controls through the built executable.
//! Why:
//!  The wrapper is installed as `git` ahead of real Git.
//!  It must never forward to
//!      itself,
//!  to a link to itself,
//!  to a copy of itself earlier or later on PATH,
//!  or
//!      to the TypeScript wrapper's launcher,
//!  and it must stop instead of looping when
//!      a different wrapper build is all that PATH offers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(join(fixture, 'bin/git'), ['--version'], { PATH: wrappersThenGit }).stdout).toBe('git version 2.56.0\n');
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{
    Fixture, Observed, TIMED_OUT_SIGNAL, bounded, bounded_for, copy_executable, direct, executable,
    fixture, link_real_git, observe, remove,
};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::Command;

/// Join fixture-relative directories (and absolute ones) into one PATH value.
fn path_of(fixture: &Fixture, entries: &[&str]) -> OsString {
    let mut path: OsString = OsString::new();
    for entry in entries {
        if !path.is_empty() {
            path.push(":");
        }
        if entry.starts_with('/') {
            path.push(entry);
        } else {
            path.push(fixture.root.join(entry));
        }
    }
    return path;
}

/// Assert that standard output is Git 2.56.0's version line followed by exactly one banner line.
fn assert_banner_follows_git_line(stdout: &[u8]) {
    let text: String = String::from_utf8_lossy(stdout).into_owned();
    let banner: &str = text
        .strip_prefix("git version 2.56.0\n")
        .expect("Git's version line comes first");
    assert!(
        banner.starts_with(concat!("monochromatic-git-policy-cli ", env!("CARGO_PKG_VERSION"), ", shadowing git at /")),
        "{text}"
    );
    assert_eq!(banner.matches('\n').count(), 1, "{text}");
    assert!(banner.ends_with('\n'), "{text}");
}

/// Place every form of "this wrapper" in the fixture:
///  hard link,
///  byte copies,
///  link chains.
fn plant_wrapper_forms(fixture: &Fixture) {
    for directory in ["hard", "copy-early", "copy-late", "chain", "nested"] {
        std::fs::create_dir(fixture.root.join(directory)).expect("directory");
    }
    std::fs::hard_link(&fixture.wrapper, fixture.root.join("hard/git")).expect("hard link");
    // Copies are made by a child process so no control can inherit them open for writing.
    copy_executable(
        fixture.wrapper.as_path(),
        fixture.root.join("copy-early/git").as_path(),
    );
    copy_executable(
        fixture.wrapper.as_path(),
        fixture.root.join("copy-late/git").as_path(),
    );
    // A link to the link: two hops to the same file.
    std::os::unix::fs::symlink(fixture.root.join("bin/git"), fixture.root.join("chain/git"))
        .expect("chained link");
    // A directory link: the same `bin` directory under another name.
    std::os::unix::fs::symlink(
        fixture.root.join("bin"),
        fixture.root.join("nested/bin-link"),
    )
    .expect("directory link");
}

/// The bound itself detects a forwarding loop:
///  a self-executing `git` is killed and reported.
#[test]
fn time_bound_detects_a_forwarding_loop() {
    let fixture: Fixture = fixture("loop-control");
    std::fs::create_dir(fixture.root.join("loop")).expect("directory");
    let looping: PathBuf = fixture.root.join("loop/git");
    executable(looping.as_path(), b"#!/bin/sh\nexec \"$0\" \"$@\"\n");
    // One second is ample to prove the bound fires; real commands finish in milliseconds.
    let status = bounded_for(
        &fixture,
        looping.as_path(),
        path_of(&fixture, &["loop"]).as_os_str(),
        "1",
    )
    .arg("--version")
    .status()
    .expect("bounded loop");
    assert_eq!(
        status.signal(),
        Some(TIMED_OUT_SIGNAL),
        "a forwarding loop must be killed at the bound"
    );
    remove(&fixture);
}

/// Whichever wrapper form is started,
///  with wrapper forms before and after real Git on
/// PATH (repeated,
///  relative,
///  linked and copied),
///  exactly real Git answers.
#[test]
fn wrapper_never_selects_itself_or_a_copy_of_itself() {
    let fixture: Fixture = fixture("self-exclusion");
    plant_wrapper_forms(&fixture);
    link_real_git(&fixture);
    let expected: Observed = observe(
        direct(&fixture).args(["rev-parse", "--sq-quote", "reached real git"]),
        b"",
    );
    assert_eq!(expected.code, Some(0));
    assert!(expected.stdout.ends_with(b"reached real git'\n"));
    let path: OsString = path_of(
        &fixture,
        &[
            "bin",
            "bin",
            "chain",
            "hard",
            "copy-early",
            "nested/bin-link",
            "missing",
            "real",
            "copy-late",
            "bin",
        ],
    );
    for started in [
        "bin/git",
        "chain/git",
        "hard/git",
        "copy-early/git",
        "copy-late/git",
        "nested/bin-link/git",
        "wrapper/cli-git-native",
    ] {
        let observed: Observed = observe(
            bounded(
                &fixture,
                fixture.root.join(started).as_path(),
                path.as_os_str(),
            )
            .args(["rev-parse", "--sq-quote", "reached real git"]),
            b"",
        );
        assert_eq!(observed, expected, "{started}");
    }
    // A relative PATH entry naming the wrapper's own directory is excluded the same way.
    let relative: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            OsString::from("bin:copy-early::real").as_os_str(),
        )
        .arg("--version"),
        b"",
    );
    assert_banner_follows_git_line(&relative.stdout);
    assert_eq!(relative.code, Some(0));
    remove(&fixture);
}

/// With only wrapper forms on PATH nothing is forwarded:
///  a diagnostic and exit status 2.
#[test]
fn only_wrappers_on_path_is_a_reported_failure() {
    let fixture: Fixture = fixture("no-real-git");
    plant_wrapper_forms(&fixture);
    let observed: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path_of(&fixture, &["bin", "hard", "copy-early", "chain", "missing"]).as_os_str(),
        )
        .arg("--version"),
        b"",
    );
    assert_eq!(
        observed,
        Observed {
            code: Some(2),
            stdout: Vec::<u8>::new(),
            stderr: b"cli-git: Could not find a real Git executable after examining 5 PATH \
                      candidates and skipping 4 cli-git wrappers. Ensure Git is installed and \
                      PATH/PATHEXT expose its executable.\n"
                .to_vec(),
        }
    );
    remove(&fixture);
}

/// A launcher script of the TypeScript wrapper ahead of real Git is skipped,
///  never executed.
#[test]
fn typescript_wrapper_launcher_is_skipped() {
    let fixture: Fixture = fixture("typescript-shim");
    link_real_git(&fixture);
    std::fs::create_dir(fixture.root.join("shim")).expect("directory");
    executable(
        fixture.root.join("shim/git").as_path(),
        b"#!/bin/sh\n# node_modules/@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs\ntouch \"$(dirname \"$0\")/executed\"\nexit 99\n",
    );
    let observed: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path_of(&fixture, &["bin", "shim", "real"]).as_os_str(),
        )
        .arg("--version"),
        b"",
    );
    assert_banner_follows_git_line(&observed.stdout);
    assert_eq!(observed.code, Some(0));
    assert!(!fixture.root.join("shim/executed").exists());
    remove(&fixture);
}

/// A different wrapper build cannot be recognised by file identity or content;
///  the
/// forward-target marker makes it stop on arrival instead of forwarding back.
#[test]
fn different_wrapper_build_stops_instead_of_looping() {
    let fixture: Fixture = fixture("other-build");
    std::fs::create_dir(fixture.root.join("other")).expect("directory");
    let other: PathBuf = fixture.root.join("other/git");
    // One trailing byte changes length and content while the program still runs.
    let mut bytes: Vec<u8> = std::fs::read(&fixture.wrapper).expect("wrapper bytes");
    bytes.push(0);
    executable(other.as_path(), bytes.as_slice());
    let ran = Command::new(&other)
        .arg("--version")
        .env_clear()
        .env("PATH", "/usr/bin")
        .output();
    assert_banner_follows_git_line(&ran.expect("modified build still runs").stdout);
    let observed: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path_of(&fixture, &["bin", "other"]).as_os_str(),
        )
        .arg("--version"),
        b"",
    );
    let mut expected: Vec<u8> =
        b"cli-git: another cli-git wrapper selected this cli-git executable (".to_vec();
    expected.extend_from_slice(other.as_os_str().as_bytes());
    expected.extend_from_slice(
        b") as real Git. Forwarding again would never reach Git. Remove the extra cli-git from \
          PATH or place the real Git executable on PATH.\n",
    );
    assert_eq!(
        observed,
        Observed {
            code: Some(2),
            stdout: Vec::<u8>::new(),
            stderr: expected,
        }
    );
    // Known limitation: real Git later on PATH does not rescue the command, because the
    // other build is the first candidate that is not provably this wrapper.
    link_real_git(&fixture);
    let with_git: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path_of(&fixture, &["bin", "other", "real"]).as_os_str(),
        )
        .arg("--version"),
        b"",
    );
    assert_eq!(with_git.code, Some(2));
    assert_eq!(with_git.stdout, Vec::<u8>::new());
    // Real Git at a conventional location is promoted ahead of every other entry and is reached.
    let conventional: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path_of(&fixture, &["bin", "other", "/usr/bin"]).as_os_str(),
        )
        .arg("--version"),
        b"",
    );
    assert_banner_follows_git_line(&conventional.stdout);
    assert_eq!(conventional.code, Some(0));
    remove(&fixture);
}
