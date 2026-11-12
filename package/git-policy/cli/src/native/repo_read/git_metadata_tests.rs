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
use super::{
    MetadataOutput, path_from_git_bytes, read_stopped_text, run_metadata_git, run_metadata_git_within,
    strip_git_line,
};
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
///  Every read the wrapper parses runs in the C locale,
///  whatever locale the person's command carries.
/// Why:
///  The wrapper reads Git's own words to decide what a clean deletes,
///  and a translated word is not the word it matches
///  (`Supprimerait nested/` under `fr_FR`),
/// so a localized Git used to have every nested repository deleted unsaved.
///  The locale is a setting of the read,
/// never of the command Git runs for the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // probe: print "${LC_ALL-none}"; expect("C")
/// ```
#[test]
fn a_read_the_wrapper_parses_runs_in_the_c_locale() {
    let root: PathBuf = fixture("metadata-locale");
    let probe: PathBuf = root.join("locale-probe");
    let recorded: PathBuf = root.join("locale-recorded");
    executable(
        probe.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s' \"${{LC_ALL-none}}\" > '{}'\nprintf 'Would remove nested/\\n'\n",
            recorded.display()
        )
        .as_bytes(),
    );
    let output: MetadataOutput = run_metadata_git(
        probe.as_path(),
        &[OsString::from("clean"), OsString::from("-n")],
        &[],
    )
    .expect("the probe starts");
    assert!(output.success, "{}", String::from_utf8_lossy(output.stderr.as_slice()));
    assert_eq!(output.stdout, b"Would remove nested/\n".to_vec());
    assert_eq!(
        std::fs::read_to_string(recorded.as_path()).expect("the probe recorded its locale"),
        "C"
    );
    remove(root.as_path());
}

/// What:
///  A read that never returns is ended at the ceiling and said so,
///  while the same ceiling lets a
///       read that answers through.
/// Why:
///   A `core.fsmonitor` program,
///  a stalled mount or a Git waiting on an index can hold a Git read
///  for good;
///  the owner's "nothing should ever wait forever or indefinitely" bounds it,
///  and a read that
///       answers must not be touched.
///  The ceiling is the seam here,
///  so the control never waits for a real one to
///       pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // ids: readWithin(hang, ['ls-files'], 300); expect(result.code).toBe('ETIMEDOUT')
/// ```
#[test]
fn a_read_that_never_returns_is_ended_at_its_ceiling() {
    let root: PathBuf = fixture("metadata-hang");
    let quick: PathBuf = root.join("quick");
    executable(quick.as_path(), b"#!/bin/sh\nprintf 'answer\\n'\n");
    let hang: PathBuf = root.join("hang");
    executable(hang.as_path(), b"#!/bin/sh\nsleep 60\n");
    let ceiling: std::time::Duration = std::time::Duration::from_millis(300);
    // The positive control: the same ceiling leaves a read that answers alone.
    let answered: MetadataOutput = run_metadata_git_within(
        quick.as_path(),
        &[OsString::from("ls-files")],
        &[],
        ceiling,
    )
    .expect("a read that answers");
    assert_eq!(answered.stdout, b"answer\n".to_vec());
    assert!(answered.success);
    let started: std::time::Instant = std::time::Instant::now();
    let ended: std::io::Error = run_metadata_git_within(
        hang.as_path(),
        &[OsString::from("ls-files")],
        &[],
        ceiling,
    )
    .expect_err("a read that never returns");
    let took: std::time::Duration = started.elapsed();
    assert_eq!(ended.kind(), std::io::ErrorKind::TimedOut, "{ended}");
    let text: String = ended.to_string();
    assert!(text.contains("ls-files"), "{text}");
    assert!(text.contains("was ended"), "{text}");
    // The read is announced and reported under the Git program and its subcommand.
    assert!(text.contains("showed no sign of work"), "{text}");
    // The read is ended at the ceiling, not left until the script's own minute is over.
    assert!(took < std::time::Duration::from_secs(30), "{took:?}");
    // How the person is told about it: the sentence each caller reports.
    let sentence: String =
        read_stopped_text("the staging area", &ended).expect("a read the wrapper ended");
    assert!(
        sentence.starts_with(
            "monochromatic-git-policy-cli stopped waiting for the Git read of the staging area: "
        ),
        "{sentence}"
    );
    // The words of a watched step belong to a commit's steps, and a check's read says what stayed as it was instead.
    assert!(
        sentence.contains("The read changed nothing, so the index and the worktree are as they were; run the command again"),
        "{sentence}"
    );
    assert!(
        !sentence.contains("finishes or undoes that commit"),
        "{sentence}"
    );
    // Any other failure keeps the wording its caller already had.
    let unstartable: std::io::Error = run_metadata_git_within(
        root.join("missing").as_path(),
        &[],
        &[],
        ceiling,
    )
    .expect_err("a program that is not there");
    assert_eq!(read_stopped_text("the staging area", &unstartable), None);
    remove(root.as_path());
}

/// A read the in-process readers decline runs Git,
/// and the step ledger names that child with how long it ran,
/// so the preparation's record holds the child a slow commit waited on instead of the phase alone.
#[test]
fn a_read_the_in_process_readers_decline_is_noted_as_a_step() {
    let root: PathBuf = fixture("metadata-step");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a file");
    crate::test_support::git(repo.as_path(), &["add", "a.txt"]);
    let began: std::time::Instant = std::time::Instant::now();
    let arguments: Vec<OsString> = vec![
        OsString::from(format!("--git-dir={}", repo.join(".git").display())),
        OsString::from("ls-files"),
        OsString::from("--stage"),
        OsString::from("-z"),
    ];
    let output: MetadataOutput = run_metadata_git_within(
        Path::new(REAL_GIT),
        arguments.as_slice(),
        crate::repo_read_hook::switch_off().as_slice(),
        std::time::Duration::from_secs(30),
    )
    .expect("the read runs");
    assert!(output.success, "{}", String::from_utf8_lossy(output.stderr.as_slice()));
    let (steps, dropped) = crate::transaction_step_durations::drain_steps(began);
    assert_eq!(dropped, 0);
    assert!(
        steps.iter().any(|step| { return step.name == "git ls-files"; }),
        "the child the readers declined is a step of the record: {steps:?}"
    );
    remove(root.as_path());
}

/// What:
///  An answer larger than the capture's `STEP_OUTPUT_LIMIT` comes back whole from a read its ceiling
/// bounds,
/// byte for byte.
/// Why:
///   A read bounded at its ceiling is drained through pipes,
/// every byte of it
/// (`src/native/process/step_output_read_whole_through_pipes.rs`),
///  and not from the anonymous files the captured
///  run keeps the last `STEP_OUTPUT_LIMIT` of.
/// So a large answer cannot be cut here,
/// and no refusal of a cut one is needed;
///  this control states that,
/// and fails the day a bound read is routed through the capture.
#[test]
fn an_answer_larger_than_the_capture_keeps_comes_back_whole() {
    let root: PathBuf = fixture("git-metadata-whole-answer");
    let big: PathBuf = root.join("big");
    let too_many: usize = crate::output_capture::STEP_OUTPUT_LIMIT + 1;
    executable(big.as_path(), format!("#!/bin/sh\nhead -c {too_many} /dev/zero\n").as_bytes());
    let arguments: [OsString; 1] = [OsString::from("ls-files")];
    let ceiling: std::time::Duration = std::time::Duration::from_secs(60);
    let answer: MetadataOutput = run_metadata_git_within(big.as_path(), &arguments, &[], ceiling).expect("a large answer");
    remove(root.as_path());
    assert!(answer.success, "the read answered");
    assert_eq!(answer.stdout.len(), too_many, "the answer comes back whole, byte for byte");
}

/// What:
///  A read the wrapper starts as `git -C <directory> status` is announced by its subcommand,
///  and never by the directory it was pointed at,
///  while a read that names no subcommand still says it is a read-only query.
/// Why:
///   A removal of a linked worktree reads the target's tracked changes with exactly that command line,
///  and the notice a person saw during it said "still waiting for /usr/bin/git (a read-only Git query)",
///  which names no query at all;
///  the value of a global option can be a pathname,
///  so it is stepped over and never printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // subject(['-C', dir, 'status', '--porcelain=v1']) === `${git} status`
/// ```
#[test]
fn a_read_named_after_a_global_option_is_announced_by_its_subcommand() {
    let root: PathBuf = fixture("metadata-subject");
    let hang: PathBuf = root.join("hang");
    executable(hang.as_path(), b"#!/bin/sh\nsleep 60\n");
    let ceiling: std::time::Duration = std::time::Duration::from_millis(300);
    let ended: std::io::Error = run_metadata_git_within(
        hang.as_path(),
        &[
            OsString::from("-C"),
            OsString::from("/some/where"),
            OsString::from("status"),
            OsString::from("--porcelain=v1"),
            OsString::from("--untracked-files=no"),
        ],
        crate::repo_read_hook::switch_off().as_slice(),
        ceiling,
    )
    .expect_err("a read that never returns");
    let text: String = ended.to_string();
    assert!(
        text.contains(" status showed no sign of work"),
        "the subcommand after a global option's value names the read: {text}"
    );
    assert!(!text.contains("a read-only Git query"), "{text}");
    // The value of the global option is a pathname, and no pathname reaches the sentence.
    assert!(!text.contains("/some/where"), "{text}");
    // A read with no subcommand after its global option still says what it is.
    let unnamed: std::io::Error = run_metadata_git_within(
        hang.as_path(),
        &[OsString::from("-C"), OsString::from("/some/where")],
        crate::repo_read_hook::switch_off().as_slice(),
        ceiling,
    )
    .expect_err("a read that never returns");
    let unnamed_text: String = unnamed.to_string();
    assert!(unnamed_text.contains("(a read-only Git query)"), "{unnamed_text}");
    assert!(!unnamed_text.contains("/some/where"), "{unnamed_text}");
    remove(root.as_path());
}
