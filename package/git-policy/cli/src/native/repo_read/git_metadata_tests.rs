//! What:
//!  Byte-level controls for captured real-Git queries.
//! Why:
//!  Query output is path bytes;
//!  exactly one terminator is removed and nothing is decoded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(stripGitLine('/repo/.git\n')).toBe('/repo/.git');
//! ```
#![cfg(unix)]

/// Import the query helpers under test and shared fixtures.
use super::{MetadataOutput, path_from_git_bytes, run_metadata_git, run_metadata_git_within, strip_git_line};
use crate::test_support::{REAL_GIT, executable, fixture, remove};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

/// Exactly one final LF or CRLF is removed;
///  everything else is value bytes.
#[test]
fn strip_removes_exactly_one_terminator() {
    for (input, expected) in [
        (b"/repo/.git\n".as_slice(), b"/repo/.git".as_slice()),
        (b"/repo/.git\r\n", b"/repo/.git"),
        (b"/repo/.git", b"/repo/.git"),
        (b"a\n\n", b"a\n"),
        (b"a\r\n\r\n", b"a\r\n"),
        (b"a\r", b"a\r"),
        (b"a\n\r", b"a\n\r"),
        (b"\n", b""),
        (b"\r\n", b""),
        (b"", b""),
        (b"\xff\n", b"\xff"),
    ] {
        assert_eq!(strip_git_line(input), expected, "{input:?}");
    }
}

/// Path bytes round-trip unchanged;
///  an empty value is not a path.
#[test]
fn path_bytes_round_trip_without_decoding() {
    assert_eq!(path_from_git_bytes(b""), None);
    assert_eq!(path_from_git_bytes(b"/repo"), Some(PathBuf::from("/repo")));
    assert_eq!(
        path_from_git_bytes(b"/r\xff/\xfe "),
        Some(PathBuf::from(OsString::from_vec(b"/r\xff/\xfe ".to_vec())))
    );
}

/// A successful query returns exact stdout bytes;
///  a failed one is an answer,
///  not an error.
#[test]
fn queries_capture_output_and_exit_state() {
    let version: MetadataOutput =
        run_metadata_git(Path::new(REAL_GIT), &[OsString::from("--version")], &[])
            .expect("Git starts");
    assert_eq!(
        version,
        MetadataOutput {
            success: true,
            code: Some(0),
            stdout: b"git version 2.56.0\n".to_vec(),
            stderr: Vec::<u8>::new(),
        }
    );
    let root: PathBuf = fixture("metadata-outside");
    let outside: MetadataOutput = run_metadata_git(
        Path::new(REAL_GIT),
        &[
            OsString::from("-C"),
            root.clone().into_os_string(),
            OsString::from("rev-parse"),
            OsString::from("--git-dir"),
        ],
        &[],
    )
    .expect("Git starts");
    assert!(!outside.success);
    assert_eq!(outside.code, Some(128));
    assert_eq!(outside.stdout, Vec::<u8>::new());
    assert!(
        String::from_utf8_lossy(&outside.stderr).contains("not a git repository"),
        "{:?}",
        outside.stderr
    );
    remove(root.as_path());
}

/// The child receives unchanged arguments and the overlay,
///  reads no input,
///  and its streams are captured.
#[test]
fn queries_pass_arguments_and_overlay_and_close_stdin() {
    let root: PathBuf = fixture("metadata-probe");
    let probe: PathBuf = root.join("probe");
    executable(
        probe.as_path(),
        b"#!/bin/sh\nprintf '%s\\0' \"$@\"\nprintf '%s' \"$MARKER\" >&2\nif read line; then exit 9; fi\nexit 3\n",
    );
    let output: MetadataOutput = run_metadata_git(
        probe.as_path(),
        &[
            OsString::from("a b"),
            OsString::new(),
            OsString::from_vec(b"\xff-raw".to_vec()),
            OsString::from("--"),
        ],
        &[(OsString::from("MARKER"), OsString::from("overlay value"))],
    )
    .expect("probe starts");
    assert_eq!(
        output,
        MetadataOutput {
            success: false,
            code: Some(3),
            stdout: b"a b\0\0\xff-raw\0--\0".to_vec(),
            stderr: b"overlay value".to_vec(),
        }
    );
    assert_eq!(
        run_metadata_git(root.join("missing").as_path(), &[], &[])
            .expect_err("missing program")
            .kind(),
        std::io::ErrorKind::NotFound
    );
    remove(root.as_path());
}

/// What:
///  A query that does not answer is ended at its ceiling,
/// and says so.
/// Why:
///   A read-only query has no watch and no meter,
/// so a program that accepts its arguments and never answers held a command for
///  good
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 5).
/// The stand-in Git would outlive the control's own bound if nothing ended it,
/// so a query that outlasts the bound shows.
#[test]
fn a_query_that_does_not_answer_is_ended_and_says_so() {
    let root: PathBuf = fixture("git-metadata-ceiling");
    let answering: PathBuf = root.join("answering");
    let silent: PathBuf = root.join("silent");
    std::fs::write(&answering, "#!/bin/sh\nprintf '/repo/.git\\n'\n").expect("the answering stand-in");
    std::fs::write(&silent, "#!/bin/sh\nsleep 6\n").expect("the silent stand-in");
    for program in [&answering, &silent] {
        let mut mode = std::fs::metadata(program).expect("the stand-in's metadata").permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o755);
        std::fs::set_permissions(program, mode).expect("the stand-in runs");
    }
    let ceiling: std::time::Duration = std::time::Duration::from_millis(700);
    let arguments: [OsString; 1] = [OsString::from("rev-parse")];
    let answers_git: [(OsString, OsString); 1] = [(OsString::from(crate::repo_read_hook::SWITCH_VARIABLE), OsString::from("off"))];
    let answered: MetadataOutput = run_metadata_git_within(answering.as_path(), &arguments, &answers_git, ceiling).expect("the stand-in answers");
    let began: std::time::Instant = std::time::Instant::now();
    let held: std::io::Result<MetadataOutput> = run_metadata_git_within(silent.as_path(), &arguments, &answers_git, ceiling);
    let waited: std::time::Duration = began.elapsed();
    crate::test_support::remove(root.as_path());
    assert_eq!(strip_git_line(answered.stdout.as_slice()), b"/repo/.git", "the query that answers is not ended");
    assert!(waited < std::time::Duration::from_secs(3), "the query outlasted its end by {waited:?}");
    let failure: std::io::Error = held.expect_err("a query that does not answer is ended");
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    assert!(failure.to_string().contains("did not answer within"), "{failure}");
}
