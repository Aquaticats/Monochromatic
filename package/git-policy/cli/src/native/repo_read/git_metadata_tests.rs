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
use super::{MetadataOutput, path_from_git_bytes, run_metadata_git, strip_git_line};
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
