//! Unit tests of the message for a real `mpm` that could not be started:
//! one test per cause,
//! the parts every message has,
//! and the message for a PATH that holds no real `mpm` at all.

use std::ffi::OsString;
use std::fs;
use std::io::{
    Error,
    ErrorKind,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

use crate::constants::EXEC_FAILED_EXIT_CODE;
use crate::program_bytes::{
    WIDE,
    elf,
};
use crate::real_binary::{
    Candidate,
    Lookup,
    SkipReason,
};
use crate::start_failure::{
    StartFailure,
    StartRefusal,
    describe_missing_real_mpm,
    describe_start_failure,
};
use crate::start_retry::RetryReport;
use crate::test_scratch::scratch;

/// The program most tests name.
const PROGRAM: &str = "/repo/.cache/mpm-shadow-bin/mpm";

/// What:
/// `fn words(items: &[&str]) -> Vec<OsString>` turns test arguments into the owned arguments a start carries.
/// Why:
/// `StartFailure` holds the arguments as the shadow received them,
/// which are `OsString` rather than `&str`.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// What:
/// A lookup that chose one program and saw nothing else,
/// as the shadow's own lookup answers when the first `mpm` on PATH is the one to start.
/// Why:
/// The message carries the lookup,
/// and most tests here are about a cause rather than about the PATH.
fn lookup_that_chose(program: &str) -> Lookup {
    let path: PathBuf = PathBuf::from(program);
    return Lookup {
        candidates: vec![Candidate {
            path: path.clone(),
            skipped_because: None,
        }],
        chosen: Some(path),
    };
}

/// What:
/// The message for a start the system answered with `error`,
/// for the program at `program`.
/// Why:
/// Most tests here have one system answer and one program to set,
/// so the refusal is built from them in one place.
fn message_for(
    program: &Path,
    error: Error,
    retry: RetryReport,
    depth: u32,
) -> String {
    let arguments: Vec<OsString> = words(&["i", "-g"]);
    let lookup: Lookup = lookup_that_chose(&program.display().to_string());
    return describe_start_failure(&StartFailure {
        outer: &arguments,
        program,
        refusal: &StartRefusal::System {
            error,
        },
        retry,
        depth,
        lookup: &lookup,
    });
}

/// The message for a start of `PROGRAM` the system answered with `error`.
fn message(
    outer: &[&str],
    error: Error,
    retry: RetryReport,
    depth: u32,
) -> String {
    let arguments: Vec<OsString> = words(outer);
    let lookup: Lookup = lookup_that_chose(PROGRAM);
    return describe_start_failure(&StartFailure {
        outer: &arguments,
        program: Path::new(PROGRAM),
        refusal: &StartRefusal::System {
            error,
        },
        retry,
        depth,
        lookup: &lookup,
    });
}

/// What:
/// The message for a start refused because the program is a shadow of this tool.
/// Why:
/// That refusal carries a sentence rather than an error,
/// and its words are the whole of what a person has to go on.
fn shadow_message(
    sentence: &str,
    lookup: &Lookup,
) -> String {
    let arguments: Vec<OsString> = words(&["i", "-g"]);
    return describe_start_failure(&StartFailure {
        outer: &arguments,
        program: Path::new(PROGRAM),
        refusal: &StartRefusal::Shadow {
            sentence: sentence.to_string(),
        },
        retry: RetryReport::default(),
        depth: 0,
        lookup,
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

/// `E2BIG` on Linux is errno 7 (`asm-generic/errno-base.h`):
/// an argument list longer than the limit in force.
#[cfg(target_os = "linux")]
const LINUX_E2BIG: i32 = 7;

/// `ENAMETOOLONG` on Linux is errno 36 (`asm-generic/errno.h`).
#[cfg(target_os = "linux")]
const LINUX_ENAMETOOLONG: i32 = 36;

/// `ELOOP` on Linux is errno 40 (`asm-generic/errno.h`).
#[cfg(target_os = "linux")]
const LINUX_ELOOP: i32 = 40;

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

/// What:
/// `fn script_whose_interpreter_names(interpreter: &str) -> (tempfile::TempDir, PathBuf)` writes a script whose
/// first line names `interpreter`.
/// Why:
/// The `#!` case is what a launcher and a published console script both are,
/// and the message must name the program that line names.
fn script_whose_interpreter_names(interpreter: &str) -> (tempfile::TempDir, PathBuf) {
    let directory: tempfile::TempDir = scratch();
    let program: PathBuf = directory.path().join("mpm");
    fs::write(&program, format!("#!{interpreter}\nexit 0\n")).expect("write the fixture script");
    return (directory, program);
}

/// What:
/// `fn compiled_program_naming(linker: &str) -> (tempfile::TempDir, PathBuf)` writes a compiled program whose
/// own file names `linker` as its interpreter.
/// Why:
/// A compiled program has no first line,
/// and this is the fixture that shows the message knows which file such a program needs.
fn compiled_program_naming(linker: &str) -> (tempfile::TempDir, PathBuf) {
    let directory: tempfile::TempDir = scratch();
    let program: PathBuf = directory.path().join("mpm");
    fs::write(&program, elf(&WIDE, false, Some(&format!("{linker}\0"))))
        .expect("write the fixture program");
    return (directory, program);
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

/// What:
/// A script another process holds open for writing is refused with the interpreter its first line names,
/// and the message says which file that is.
/// Why:
/// A script has a first line,
/// and its interpreter is the other file the start needs.
#[cfg(target_os = "linux")]
#[test]
fn a_refusal_because_a_script_is_being_written_names_the_interpreter_its_first_line_names() {
    let (_directory, program) = script_whose_interpreter_names("/bin/sh");
    let error = Error::from_raw_os_error(LINUX_ETXTBSY);
    assert_eq!(error.kind(), ErrorKind::ExecutableFileBusy, "{error}");
    let text = message_for(&program, error, no_retry(), 0);
    for expected in [
        "Text file busy",
        "is open for writing by some other process",
        "the interpreter its first line names (`/bin/sh`)",
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

/// What:
/// A compiled program another process holds open for writing is refused with the dynamic linker its own file
/// names,
/// and the message never claims that program has a first line.
/// Why:
/// The realistic case is a system's linker being replaced during an upgrade,
/// and a reader sent to the program's first line is sent to a line it does not have.
#[cfg(target_os = "linux")]
#[test]
fn a_refusal_because_a_compiled_program_is_being_written_names_its_dynamic_linker() {
    let (_directory, program) = compiled_program_naming("/lib64/ld-fixture.so");
    let error = Error::from_raw_os_error(LINUX_ETXTBSY);
    let text = message_for(&program, error, no_retry(), 0);
    for expected in [
        "Text file busy",
        "is open for writing by some other process",
        "its dynamic linker (`/lib64/ld-fixture.so`)",
        "cannot tell which process holds that file",
        "run the same command again",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("named on the program's first line"),
        "a compiled program has no first line for the interpreter to be named on: {text}"
    );
}

/// A file the code cannot read as a program is described by both kinds of interpreter,
/// not by one of them.
#[cfg(target_os = "linux")]
#[test]
fn a_refusal_because_an_unreadable_file_is_being_written_names_both_kinds_of_interpreter() {
    let directory: tempfile::TempDir = scratch();
    let program: PathBuf = directory.path().join("gone");
    let error = Error::from_raw_os_error(LINUX_ETXTBSY);
    let text = message_for(&program, error, no_retry(), 0);
    assert!(
        text.contains("a script's first line or a compiled program's dynamic linker"),
        "{text}"
    );
}

/// What:
/// A script whose interpreter is missing is told so by name,
/// with the interpreter as the thing to install.
/// Why:
/// `ENOENT` answers for both files,
/// and the script's own first line says which one the start looked for.
#[test]
fn a_script_whose_interpreter_is_missing_names_that_interpreter_and_the_remedy_installs_it() {
    let (_directory, program) = script_whose_interpreter_names("/nonexistent/interpreter");
    let text = message_for(&program, Error::from(ErrorKind::NotFound), no_retry(), 0);
    for expected in [
        "is there, but the interpreter its first line names (`/nonexistent/interpreter`) is not",
        "a script is started by that program",
        "install the interpreter its first line names (`/nonexistent/interpreter`)",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("by hand"),
        "a file that is not there shows nothing when it is run by hand: {text}"
    );
}

/// What:
/// A compiled program whose dynamic linker is missing is told so by name,
/// and the remedy installs the package that provides it.
/// Why:
/// This is the case a person meets when a system's linker is removed or a tool's own loader is deleted,
/// and the message used to name a first line the program does not have.
#[test]
fn a_compiled_program_whose_dynamic_linker_is_missing_names_that_linker() {
    let (_directory, program) = compiled_program_naming("/nowhere/ld-fixture.so");
    let text = message_for(&program, Error::from(ErrorKind::NotFound), no_retry(), 0);
    for expected in [
        "is there, but the dynamic linker it names (`/nowhere/ld-fixture.so`) is not",
        "a compiled program cannot start without it",
        "install or upgrade the package that provides `/nowhere/ld-fixture.so`",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("named on its first line"),
        "a compiled program names no interpreter on a first line: {text}"
    );
}

/// What:
/// A program that is itself gone keeps both possibilities,
/// named as a script's first line or a compiled program's dynamic linker.
/// Why:
/// Nothing of a missing file can say which interpreter it needed,
/// so the message must not claim to know.
#[test]
fn a_program_that_is_gone_keeps_both_kinds_of_interpreter_and_the_two_step_remedy() {
    let text = message(
        &["i", "-g"],
        Error::from(ErrorKind::NotFound),
        no_retry(),
        0,
    );
    assert_has_the_common_parts(&text);
    for expected in [
        "the program, or the interpreter it names, is not there",
        "a script's first line, or a compiled program's dynamic linker",
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
/// An error no arm knows keeps the system's words and names no step that cannot show the error:
/// no hand run of the program,
/// and running the same command again only where repeating it can work.
/// Why:
/// The arm used to send every reader to run the program by hand,
/// which cannot reproduce a refusal of the shadow's own start.
#[test]
fn any_other_cause_keeps_the_systems_words_and_names_no_step_that_cannot_show_the_error() {
    let error = Error::new(ErrorKind::InvalidInput, "invalid argument");
    let text = message(&["i", "-g"], error, no_retry(), 0);
    assert_has_the_common_parts(&text);
    for expected in [
        "invalid argument",
        "no arm of this message knows that refusal",
        "read those words",
        "run the same command again",
        "a shortage (of memory, or of open files) is over by then",
        "installing or upgrading that program again replaces",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("by hand"),
        "a hand run of the program cannot show a refusal of this start: {text}"
    );
}

/// What:
/// An error that running the same command again cannot cure says so,
/// and names what to change instead.
/// Why:
/// A shell repeats `E2BIG` for the same argument list,
/// so advice to run the command again is advice that cannot work.
#[cfg(target_os = "linux")]
#[test]
fn a_refusal_that_repeating_cannot_cure_says_so_and_names_what_to_change() {
    let cases: [(i32, &str, &str); 3] = [
        (
            LINUX_E2BIG,
            "Argument list too long",
            "Shorten the command line",
        ),
        (
            LINUX_ENAMETOOLONG,
            "File name too long",
            "Move the program to a shorter path",
        ),
        (
            LINUX_ELOOP,
            "Too many levels of symbolic links",
            "repair the link that loops",
        ),
    ];
    for (number, words, change) in cases {
        let text = message(
            &["i", "-g"],
            Error::from_raw_os_error(number),
            no_retry(),
            0,
        );
        assert!(text.contains(words), "{words}: {text}");
        assert!(
            text.contains("Running the same command again cannot succeed"),
            "the arm must say repeating cannot cure it: {text}"
        );
        assert!(text.contains(change), "{change}: {text}");
        assert!(
            !text.contains("run the same command again"),
            "the remedy for this refusal is not the same command again: {text}"
        );
        assert!(
            !text.contains("by hand"),
            "a hand run cannot show this refusal: {text}"
        );
    }
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

/// What:
/// A refused shadow says what was found,
/// why it would loop,
/// what repairs it,
/// and prints the `mpm` commands the lookup read rather than naming a command a shell may reject.
/// Why:
/// `command -v -a mpm` fails in bash (`command: -a: invalid option`) and in zsh
/// (`command not found: -v`),
/// measured in this round,
/// so the list has to come from the shadow itself.
#[test]
fn a_program_that_is_or_leads_to_a_shadow_prints_the_commands_the_lookup_read() {
    // Built the way `vet` builds it (`shadow_identity/vetted_program.rs`): the path, the identity text it read, and the refusal.
    let sentence: String = format!(
        "{PROGRAM} is or leads to {} {}, a shadow, so it was not started",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    let lookup: Lookup = Lookup {
        candidates: vec![
            Candidate {
                path: PathBuf::from("/shadow/bin/mpm"),
                skipped_because: Some(SkipReason::ThisShadow),
            },
            Candidate {
                path: PathBuf::from("/other/bin/mpm"),
                skipped_because: Some(SkipReason::Shadow {
                    identity: "monochromatic-mpm-shadow 0.2.7-alpha.13".to_string(),
                }),
            },
            Candidate {
                path: PathBuf::from(PROGRAM),
                skipped_because: None,
            },
        ],
        chosen: Some(PathBuf::from(PROGRAM)),
    };
    let text: String = shadow_message(&sentence, &lookup);
    assert_has_the_common_parts(&text);
    for expected in [
        "is or leads to",
        "a shadow, so it was not started",
        "the PATH lookup a moment earlier took that path for the real `mpm`",
        "was different when the lookup read it",
        "mpm-shadow cannot tell which of them changed, or which process changed it",
        "a shadow looks for the real `mpm` again",
        "105 starts a second",
        "the `mpm` commands this run found on PATH, in the order the lookup read them:",
        "/shadow/bin/mpm: skipped: it is this shadow",
        "/other/bin/mpm: skipped: a shadow (monochromatic-mpm-shadow 0.2.7-alpha.13)",
        &format!("{PROGRAM}: chosen by the lookup"),
        "mpm install",
        "Nothing of the command ran",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("command -v -a mpm"),
        "bash and zsh reject `command -v -a mpm`, so the shadow prints the list itself: {text}"
    );
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
/// The message for a PATH whose only `mpm` is a file with no execute bit names that file,
/// its mode,
/// and the `chmod` that repairs it.
/// Why:
/// The message used to say `mpm` is not installed or that every `mpm` is a shadow,
/// and a file that lost its execute bit is neither:
/// the person was told to install what they already had.
#[test]
fn a_file_with_no_execute_bit_is_named_with_its_mode_and_the_chmod_that_repairs_it() {
    let lookup: Lookup = Lookup {
        candidates: vec![Candidate {
            path: PathBuf::from("/home/someone/.local/bin/mpm"),
            skipped_because: Some(SkipReason::NoExecuteBit {
                mode: 0o644,
            }),
        }],
        chosen: None,
    };
    let text: String = describe_missing_real_mpm(&lookup);
    for expected in [
        "no real mpm found on PATH",
        "the `mpm` commands this run found on PATH, in the order the lookup read them:",
        "/home/someone/.local/bin/mpm: skipped: a file with no execute bit (mode 0644)",
        "make `/home/someone/.local/bin/mpm` runnable (`chmod +x /home/someone/.local/bin/mpm`)",
        "then run the command again",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("is not installed"),
        "the file is there, so nothing was installed wrongly: {text}"
    );
}

/// What:
/// A directory of the name and a link that leads nowhere are named with the repair each one needs.
/// Why:
/// Three different faults that the old message folded into one sentence need three different steps.
#[test]
fn a_directory_and_a_link_that_leads_nowhere_are_named_with_their_own_repairs() {
    let lookup: Lookup = Lookup {
        candidates: vec![
            Candidate {
                path: PathBuf::from("/first/mpm"),
                skipped_because: Some(SkipReason::Directory),
            },
            Candidate {
                path: PathBuf::from("/second/mpm"),
                skipped_because: Some(SkipReason::BrokenLink {
                    target: PathBuf::from("/second/nowhere"),
                }),
            },
        ],
        chosen: None,
    };
    let text: String = describe_missing_real_mpm(&lookup);
    for expected in [
        "/first/mpm: skipped: a directory, not a program",
        "replace the directory `/first/mpm` with the real `mpm`",
        "/second/mpm: skipped: a symbolic link that leads to no file (it points at /second/nowhere)",
        "point the link `/second/mpm` at the real `mpm`",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

/// What:
/// A PATH that holds no `mpm` at all says so and names installing the real one.
/// Why:
/// Nothing was skipped,
/// so a list of skipped files would be a list of nothing and a `chmod` would have no path.
#[test]
fn a_path_that_holds_no_mpm_says_so_and_names_the_install() {
    let lookup: Lookup = Lookup {
        candidates: Vec::new(),
        chosen: None,
    };
    let text: String = describe_missing_real_mpm(&lookup);
    for expected in [
        "no real mpm found on PATH",
        "nothing named `mpm` is in any directory on PATH",
        "install the real `mpm`",
        "put its directory on PATH after this shadow's",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("chmod"),
        "no file was seen, so no file can be made runnable: {text}"
    );
}

/// What:
/// A shadow at the nesting bound prints the commands the lookup read,
/// and its remedy names no command a shell may reject.
/// Why:
/// The nesting message carried the same `command -v -a mpm`.
#[test]
fn the_nesting_refusal_prints_the_commands_the_lookup_read_and_the_status() {
    use crate::nesting::MAX_NESTING;
    use crate::start_failure::describe_nesting_refusal;
    let arguments: Vec<OsString> = words(&["i", "-g"]);
    let lookup: Lookup = Lookup {
        candidates: vec![
            Candidate {
                path: PathBuf::from("/shadow/bin/mpm"),
                skipped_because: Some(SkipReason::ThisShadow),
            },
            Candidate {
                path: PathBuf::from(PROGRAM),
                skipped_because: None,
            },
        ],
        chosen: Some(PathBuf::from(PROGRAM)),
    };
    let text: String =
        describe_nesting_refusal(&arguments, Path::new(PROGRAM), MAX_NESTING, &lookup);
    for expected in [
        "mpm i -g",
        PROGRAM,
        &format!("nesting level {MAX_NESTING}"),
        &format!("limit {MAX_NESTING}"),
        "started by another mpm-shadow",
        "/shadow/bin/mpm: skipped: it is this shadow",
        "the `mpm` commands this run found on PATH",
        &EXEC_FAILED_EXIT_CODE.to_string(),
        "post-run cleanup did not run",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        !text.contains("command -v -a mpm"),
        "bash and zsh reject `command -v -a mpm`, so the shadow prints the list itself: {text}"
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
