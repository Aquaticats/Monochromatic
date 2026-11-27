//! Unit tests of the message for a real `mpm` that could not be started:
//! one test per cause,
//! plus the parts every message has.

use std::ffi::OsString;
use std::io::{
    Error,
    ErrorKind,
};
use std::path::Path;
use std::time::Duration;

use crate::constants::EXEC_FAILED_EXIT_CODE;
use crate::start_failure::{
    StartFailure,
    StartRefusal,
    describe_start_failure,
};
use crate::start_retry::RetryReport;

const PROGRAM: &str = "/repo/.cache/mpm-shadow-bin/mpm";

fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// What:
/// The message for a start the system answered with `error`.
/// Why:
/// Most tests here have one system answer and nothing else to set,
/// so the refusal is built from it in one place.
fn message(
    outer: &[&str],
    error: Error,
    retry: RetryReport,
    depth: u32,
) -> String {
    let arguments: Vec<OsString> = words(outer);
    return describe_start_failure(&StartFailure {
        outer: &arguments,
        program: Path::new(PROGRAM),
        refusal: &StartRefusal::System {
            error,
        },
        retry,
        depth,
    });
}

/// What:
/// The message for a start refused because the program is a shadow of this tool.
/// Why:
/// That refusal carries a sentence rather than an error,
/// and its words are the whole of what a person has to go on.
fn shadow_message(sentence: &str) -> String {
    let arguments: Vec<OsString> = words(&["i", "-g"]);
    return describe_start_failure(&StartFailure {
        outer: &arguments,
        program: Path::new(PROGRAM),
        refusal: &StartRefusal::Shadow {
            sentence: sentence.to_string(),
        },
        retry: RetryReport::default(),
        depth: 0,
    });
}

fn no_retry() -> RetryReport {
    return RetryReport::default();
}

/// `ETXTBSY` on Linux is errno 26 (named here,
/// not in the assertions);
/// the system's own text for it is what the message must carry.
#[cfg(target_os = "linux")]
const LINUX_ETXTBSY: i32 = 26;

/// `ENOEXEC` on Linux is errno 8 (`asm-generic/errno-base.h`,
/// named here and not in the assertions);
/// `std::io::ErrorKind` has no variant for it,
/// and the message module keeps its own copy of the number private.
#[cfg(target_os = "linux")]
const LINUX_ENOEXEC: i32 = 8;

/// The parts every message has,
/// whatever the cause.
fn assert_has_the_common_parts(text: &str) {
    assert!(text.starts_with("mpm-shadow: "), "{text}");
    assert!(
        text.contains("mpm i -g"),
        "the command the person ran: {text}"
    );
    assert!(text.contains(PROGRAM), "the program it tried: {text}");
    assert!(
        text.contains("starting the real `mpm`"),
        "the stage: {text}"
    );
    assert!(
        text.contains(&EXEC_FAILED_EXIT_CODE.to_string()),
        "the exit status: {text}"
    );
    assert!(
        text.contains("could not be started"),
        "why that status: {text}"
    );
    assert!(
        text.contains("nothing") && text.contains("post-run cleanup did not run"),
        "what completed: {text}"
    );
}

#[test]
fn a_refusal_for_lack_of_processes_says_so_in_plain_words_with_the_retries_and_both_kinds_of_remedy()
 {
    let retry = RetryReport {
        refusals: 7,
        paused: Duration::from_millis(4980),
    };
    let error = Error::from(ErrorKind::WouldBlock);
    let text = message(&["i", "-g"], error, retry, 0);
    assert_has_the_common_parts(&text);
    assert!(text.contains("refused to create another process"), "{text}");
    assert!(
        text.contains("7 times") && text.contains("4980 ms"),
        "{text}"
    );
    for remedy in ["ulimit -u", "TasksMax", "wait", "run the command again"] {
        assert!(text.contains(remedy), "{remedy}: {text}");
    }
}

#[test]
fn a_program_without_permission_names_the_execute_bit_and_the_directory_search_bit() {
    let text = message(
        &["i", "-g"],
        Error::from(ErrorKind::PermissionDenied),
        no_retry(),
        0,
    );
    assert_has_the_common_parts(&text);
    assert!(text.contains("permission"), "{text}");
    assert!(text.contains("chmod"), "{text}");
}

#[cfg(target_os = "linux")]
#[test]
fn a_refusal_because_the_program_is_being_written_says_what_the_refusal_proves_and_what_to_do() {
    let error = Error::from_raw_os_error(LINUX_ETXTBSY);
    assert_eq!(error.kind(), ErrorKind::ExecutableFileBusy, "{error}");
    let text = message(&["i", "-g"], error, no_retry(), 0);
    assert_has_the_common_parts(&text);
    assert!(text.contains("Text file busy"), "{text}");
    for expected in [
        "is open for writing by some other process",
        "the interpreter named on the program's first line",
        "cannot tell which process holds that file",
        "run the same command again",
        "the start succeeds as soon as the file is free and whole",
        "writes a whole new copy into place",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("usual cause"),
        "no measurement shows a usual cause, so the message names none: {text}"
    );
    assert!(
        !text.contains("by hand"),
        "running the program by hand shows nothing once the write is over: {text}"
    );
}

#[test]
fn a_program_that_vanished_or_has_a_missing_interpreter_gets_one_cause_and_one_remedy() {
    let text = message(
        &["i", "-g"],
        Error::from(ErrorKind::NotFound),
        no_retry(),
        0,
    );
    assert_has_the_common_parts(&text);
    for expected in [
        "the interpreter named on its first line",
        "is not there",
        "moved it aside",
        "run the same command again",
        "install it again",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("by hand"),
        "a file that is not there shows nothing when it is run by hand: {text}"
    );
}

/// What:
/// An error no arm knows keeps the system's words,
/// and its advice promises only what a hand run can show:
/// the same words when the program's own file is at fault,
/// and not when a limit of the machine or of the shadow's own process is.
/// Why:
/// The arm used to send every reader to run the program by hand,
/// which shows nothing when the cause is a limit that has passed by then.
#[test]
fn any_other_cause_keeps_the_systems_words_and_says_what_a_hand_run_can_and_cannot_show() {
    let error = Error::new(ErrorKind::InvalidInput, "argument list too long");
    let text = message(&["i", "-g"], error, no_retry(), 0);
    assert_has_the_common_parts(&text);
    for expected in [
        "argument list too long",
        "no arm of this message knows that refusal",
        "read those words",
        "run the same command again",
        "a refusal caused by a shortage (of memory, or of open files) is over by then",
        "installing or upgrading that program again replaces",
        "only in the first case",
        "second reading, never the proof",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("by hand to see the same error"),
        "a hand run shows nothing when a limit is the cause: {text}"
    );
}

#[test]
fn a_program_that_is_or_leads_to_a_shadow_says_what_was_found_why_it_would_loop_and_what_repairs_it()
 {
    // Built the way `vet` builds it (`shadow_identity/vetted_program.rs`): the path, the identity text it read, and the refusal.
    let sentence: String = format!(
        "{PROGRAM} is or leads to {} {}, a shadow, so it was not started",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    let text: String = shadow_message(&sentence);
    assert_has_the_common_parts(&text);
    for expected in [
        "is or leads to",
        "a shadow, so it was not started",
        "the PATH lookup a moment earlier took that path for the real `mpm`",
        "was different when the lookup read it",
        "mpm-shadow cannot tell which of them changed, or which process changed it",
        "a shadow looks for the real `mpm` again",
        "105 starts a second",
        "command -v -a mpm",
        "the shadow's own directory belongs first",
        "mpm install",
        "Nothing of the command ran",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        text.contains("at the start was or led to a shadow"),
        "the program is named as what it turned out to be: {text}"
    );
    assert!(
        !text.contains("the first `mpm` on PATH that is not this shadow"),
        "the refused program is exactly not that: {text}"
    );
    assert!(
        !text.contains("by hand"),
        "a shadow run by hand answers as a shadow and never shows this refusal: {text}"
    );
}

/// What:
/// A file the kernel will not start as a program gets its own cause and remedy.
/// Why:
/// `std::io::ErrorKind` has no variant for `ENOEXEC`,
/// so the file used to reach the catch-all arm
/// and its advice to run the program by hand;
/// measured 2026-11-27 in a throwaway folder on this machine:
/// `bash -c './empty-file --version'` exits 0 and prints nothing,
/// `bash -c './text-file --version'` runs the text file as a script (exit 3),
/// and only a start without a shell reports `Exec format error` (Python's `subprocess`:
/// `OSError 8 Exec format error`).
#[cfg(target_os = "linux")]
#[test]
fn a_file_the_kernel_will_not_start_as_a_program_says_a_shell_does_not_show_that_refusal() {
    let error = Error::from_raw_os_error(LINUX_ENOEXEC);
    // What:     The number is the only handle on this refusal here: the kind it maps to is unstable to name.
    // Why:      The arm is chosen by the number, so the test pins that the number survived.
    assert_eq!(error.raw_os_error(), Some(LINUX_ENOEXEC), "{error}");
    let text = message(&["i", "-g"], error, no_retry(), 0);
    assert_has_the_common_parts(&text);
    for expected in [
        "Exec format error",
        "not a program the kernel can start",
        "runs it as a script or does nothing instead of reporting the refusal",
        "running it by hand shows something else",
        "install or upgrade the program again",
        "Nothing of this command ran",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("by hand to see the same error"),
        "a shell never shows this refusal: {text}"
    );
}

#[test]
fn a_shadow_started_by_another_shadow_says_how_deep_it_is_and_what_that_means() {
    let text = message(
        &["i", "-g"],
        Error::from(ErrorKind::WouldBlock),
        no_retry(),
        3,
    );
    assert!(text.contains("started by another mpm-shadow"), "{text}");
    assert!(text.contains("level 3"), "{text}");
    let top = message(
        &["i", "-g"],
        Error::from(ErrorKind::WouldBlock),
        no_retry(),
        0,
    );
    assert!(!top.contains("started by another mpm-shadow"), "{top}");
}

#[test]
fn the_command_line_is_quoted_so_an_argument_with_a_space_stays_one_argument_and_no_arguments_is_just_mpm()
 {
    let text = message(
        &["install", "a b"],
        Error::from(ErrorKind::NotFound),
        no_retry(),
        0,
    );
    assert!(text.contains("mpm install 'a b'"), "{text}");
    let bare = message(&[], Error::from(ErrorKind::NotFound), no_retry(), 0);
    assert!(bare.contains("`mpm`"), "{bare}");
}

#[test]
fn the_nesting_refusal_names_the_command_the_program_the_level_the_limit_the_cause_the_remedy_and_the_status()
 {
    use crate::nesting::MAX_NESTING;
    use crate::start_failure::describe_nesting_refusal;
    let arguments: Vec<OsString> = words(&["i", "-g"]);
    let text = describe_nesting_refusal(&arguments, Path::new(PROGRAM), MAX_NESTING);
    for expected in [
        "mpm i -g",
        PROGRAM,
        &format!("nesting level {MAX_NESTING}"),
        &format!("limit {MAX_NESTING}"),
        "started by another mpm-shadow",
        "command -v -a mpm",
        &EXEC_FAILED_EXIT_CODE.to_string(),
        "post-run cleanup did not run",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}
