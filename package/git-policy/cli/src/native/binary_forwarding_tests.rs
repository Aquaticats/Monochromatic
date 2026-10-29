//! What:
//!  Exact-forwarding controls through the built executable.
//! Why:
//!  For commands the wrapper adds nothing to,
//!  a caller must not be able to tell
//!      the wrapper from Git:
//!  same argument bytes,
//!  streams,
//!  exit status and signals.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, args)).toEqual(run('/usr/bin/git', args));
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{
    Fixture, Observed, REAL_GIT, bounded, direct, executable, fixture, git, observe, remove,
    repository, wrapped,
};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Run the same arguments through the wrapper and through real Git,
///  in one directory.
fn both(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[OsString],
    input: &[u8],
) -> (Observed, Observed) {
    let through_wrapper: Observed = observe(
        wrapped(fixture).current_dir(directory).args(arguments),
        input,
    );
    let through_git: Observed = observe(
        direct(fixture).current_dir(directory).args(arguments),
        input,
    );
    return (through_wrapper, through_git);
}

/// Build owned arguments from text.
fn text_arguments(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// `git --version` through a `git`-named link is byte-identical to real Git 2.56.0.
#[test]
fn version_is_identical_to_native_git() {
    let fixture: Fixture = fixture("version");
    let (through_wrapper, through_git) = both(
        &fixture,
        fixture.root.as_path(),
        text_arguments(&["--version"]).as_slice(),
        b"",
    );
    assert_eq!(through_git.stdout, b"git version 2.56.0\n");
    assert_eq!(through_git.code, Some(0));
    assert_eq!(through_wrapper, through_git);
    remove(&fixture);
}

/// Every argument reaches Git as its own unchanged element,
///  including non-UTF-8 and empty ones.
#[test]
fn argument_bytes_reach_git_unchanged() {
    let fixture: Fixture = fixture("argv");
    let adversarial: Vec<OsString> = vec![
        OsString::from_vec(b"non-utf8-\xff\xfe\x80".to_vec()),
        OsString::new(),
        OsString::from("two words"),
        OsString::from("--"),
        OsString::from("-C"),
        OsString::from("$(touch injected); `id` 'single' \"double\" \\ * ? ~ ! #"),
        OsString::from("line\nbreak\ttab"),
        OsString::from("--no-enforce-worktree"),
        OsString::from("../../etc/passwd"),
        OsString::from("é\u{1F600}"),
    ];
    // Real Git echoes its arguments shell-quoted; identical bytes mean identical argv.
    let mut quoted: Vec<OsString> = text_arguments(&["rev-parse", "--sq-quote"]);
    quoted.extend(adversarial.iter().cloned());
    let (through_wrapper, through_git) =
        both(&fixture, fixture.root.as_path(), quoted.as_slice(), b"");
    assert_eq!(through_git.code, Some(0));
    assert!(through_git.stdout.windows(3).any(is_raw_marker));
    assert_eq!(through_wrapper, through_git);
    // A probe standing in for real Git prints each argument NUL-terminated.
    std::fs::create_dir(fixture.root.join("probe")).expect("probe directory");
    executable(
        fixture.root.join("probe/git").as_path(),
        b"#!/bin/sh\nprintf '%s\\0' \"$#\" \"$@\"\n",
    );
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(fixture.root.join("probe"));
    // `version` needs no repository fact, so the probe is started once, with the caller's arguments.
    let mut probe_arguments: Vec<OsString> = text_arguments(&["version"]);
    probe_arguments.extend(adversarial.iter().cloned());
    let probed: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path.as_os_str(),
        )
        .args(probe_arguments.as_slice()),
        b"",
    );
    // The wrapper's own hooks-off option comes first; the caller's bytes follow it unchanged.
    let mut expected: Vec<u8> = b"13\0-c\0core.hooksPath=/dev/null/cli-git-no-hooks\0version\0".to_vec();
    for argument in &adversarial {
        expected.extend_from_slice(argument.as_bytes());
        expected.push(0);
    }
    assert_eq!(probed.stdout, expected);
    assert_eq!(probed.code, Some(0));
    assert!(!fixture.root.join("injected").exists());
    remove(&fixture);
}

/// Named predicate:
///  the three raw non-UTF-8 bytes of the first adversarial argument.
fn is_raw_marker(window: &[u8]) -> bool {
    return window == b"\xff\xfe\x80";
}

/// A non-UTF-8 directory in the global prefix and a non-UTF-8 path in output are preserved.
#[test]
fn non_utf8_paths_select_and_report_exactly() {
    let fixture: Fixture = fixture("bytes");
    let name: OsString = OsString::from_vec(b"repo-\xff\xfe".to_vec());
    let repo: PathBuf = repository(&fixture, name.as_os_str());
    std::fs::write(repo.join(OsStr::from_bytes(b"file-\xfd.txt")), b"content\n")
        .expect("non-UTF-8 file name");
    for rest in [
        vec!["rev-parse", "--show-toplevel"],
        vec!["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        vec!["ls-files", "--others", "-z"],
    ] {
        let mut arguments: Vec<OsString> =
            vec![OsString::from("-C"), repo.clone().into_os_string()];
        arguments.extend(text_arguments(rest.as_slice()));
        let (through_wrapper, through_git) =
            both(&fixture, fixture.root.as_path(), arguments.as_slice(), b"");
        assert_eq!(through_git.code, Some(0), "{rest:?}");
        assert!(
            through_git.stdout.windows(1).any(is_high_byte),
            "{rest:?} must print raw path bytes"
        );
        assert_eq!(through_wrapper, through_git, "{rest:?}");
    }
    remove(&fixture);
}

/// Named predicate:
///  a byte that cannot appear in ASCII output.
fn is_high_byte(window: &[u8]) -> bool {
    return window[0] >= 0xfd;
}

/// Standard input reaches Git and binary standard output returns unchanged.
#[test]
fn stdin_and_binary_stdout_pass_through() {
    let fixture: Fixture = fixture("streams");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("blob.bin"), b"\x00\xff\xfe binary \r\n\x80").expect("blob");
    git(&fixture, repo.as_path(), &["add", "blob.bin"]);
    let listed = git(&fixture, repo.as_path(), &["ls-files", "--stage"]);
    let object_id: Vec<u8> = listed.stdout[7..47].to_vec();
    let mut input: Vec<u8> = object_id.clone();
    input.extend_from_slice(b"\nHEAD\nmissing-object\n");
    for rest in [
        vec!["cat-file", "--batch"],
        vec!["cat-file", "--batch-check"],
    ] {
        let (through_wrapper, through_git) = both(
            &fixture,
            repo.as_path(),
            text_arguments(rest.as_slice()).as_slice(),
            input.as_slice(),
        );
        assert_eq!(through_git.code, Some(0), "{rest:?}");
        assert!(
            through_git.stdout.starts_with(object_id.as_slice()),
            "{rest:?}"
        );
        assert_eq!(through_wrapper, through_git, "{rest:?}");
    }
    remove(&fixture);
}

/// Git's own exit codes and diagnostics are reported unchanged for failures and usage errors.
#[test]
fn exit_codes_and_stderr_match_native_git() {
    let fixture: Fixture = fixture("exit");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "tracked.txt"]);
    git(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=tracked"],
    );
    std::fs::write(repo.join("tracked.txt"), b"two\n").expect("modify");
    for (directory, arguments, code) in [
        (
            repo.as_path(),
            vec!["rev-parse", "--verify", "--quiet", "refs/heads/absent"],
            1,
        ),
        (
            repo.as_path(),
            vec!["rev-parse", "--verify", "refs/heads/absent"],
            128,
        ),
        (repo.as_path(), vec!["diff", "--exit-code", "--quiet"], 1),
        (repo.as_path(), vec!["diff", "--exit-code"], 1),
        (repo.as_path(), vec!["show", "--no-such-option"], 128),
        (fixture.root.as_path(), vec!["log"], 128),
        (
            fixture.root.as_path(),
            vec!["--no-such-global-option", "status"],
            129,
        ),
        (fixture.root.as_path(), vec!["-C"], 129),
        (fixture.root.as_path(), vec![], 1),
        (fixture.root.as_path(), vec!["--no-pager"], 1),
        (repo.as_path(), vec!["status", "--porcelain"], 0),
        (repo.as_path(), vec!["branch", "--list"], 0),
    ] {
        let (through_wrapper, through_git) = both(
            &fixture,
            directory,
            text_arguments(arguments.as_slice()).as_slice(),
            b"",
        );
        assert_eq!(through_git.code, Some(code), "{arguments:?}");
        assert_eq!(through_wrapper, through_git, "{arguments:?}");
    }
    remove(&fixture);
}

/// Wait until the process with this ID is real Git,
///  proving the wrapper replaced itself.
fn wait_until_git(child: &Child) {
    let link: PathBuf = PathBuf::from(format!("/proc/{}/exe", child.id()));
    for _attempt in 0..2_000 {
        if let Ok(target) = std::fs::read_link(&link)
            && target == Path::new(REAL_GIT)
        {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("process {} never became {REAL_GIT}", child.id());
}

/// A signal sent to the wrapper's process ID ends Git itself,
///  and the caller sees that signal.
#[test]
fn signals_reach_git_in_the_same_process() {
    let fixture: Fixture = fixture("signal");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    for (name, number) in [("TERM", 15), ("INT", 2), ("HUP", 1)] {
        // No time-bound helper here: the signalled process ID must be the wrapper's own.
        let mut child: Child = Command::new(fixture.root.join("bin/git"))
            .env_clear()
            .env("PATH", super::support::wrapper_first_path(&fixture))
            .env("HOME", fixture.root.join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .current_dir(&repo)
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("wrapper starts");
        // Git blocks reading the open pipe; the same process ID now runs Git.
        wait_until_git(&child);
        let sent = Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("kill -{name} {}", child.id()))
            .status()
            .expect("signal sender");
        assert!(sent.success());
        let status = child.wait().expect("wait for the signalled process");
        assert_eq!(status.signal(), Some(number), "SIG{name}");
        assert_eq!(status.code(), None, "SIG{name}");
    }
    remove(&fixture);
}

/// Git receives the lock PID injection after the caller's own numbered configuration,
/// the forward-target marker,
///  and every other variable unchanged.
#[test]
fn environment_overlay_reaches_git() {
    let fixture: Fixture = fixture("environment");
    std::fs::create_dir(fixture.root.join("probe")).expect("probe directory");
    let probe: PathBuf = fixture.root.join("probe/git");
    executable(
        probe.as_path(),
        b"#!/bin/sh\nprintf '%s\\0' \"$GIT_CONFIG_COUNT\" \"$GIT_CONFIG_KEY_0\" \"$GIT_CONFIG_VALUE_0\" \"$GIT_CONFIG_KEY_1\" \"$GIT_CONFIG_VALUE_1\" \"$CLI_GIT_NATIVE_FORWARD_TARGET\" \"$RAW_VALUE\"\n",
    );
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(fixture.root.join("probe"));
    let observed: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path.as_os_str(),
        )
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "user.name")
        .env("GIT_CONFIG_VALUE_0", "Caller Value")
        .env("RAW_VALUE", OsStr::from_bytes(b"raw-\xff"))
        .arg("version"),
        b"",
    );
    let mut expected: Vec<u8> = b"2\0user.name\0Caller Value\0core.lockfilePid\0true\0".to_vec();
    expected.extend_from_slice(probe.as_os_str().as_bytes());
    expected.extend_from_slice(b"\0raw-\xff\0");
    assert_eq!(observed.stdout, expected);
    assert_eq!(observed.code, Some(0));
    remove(&fixture);
}
