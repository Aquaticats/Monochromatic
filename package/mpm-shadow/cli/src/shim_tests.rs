//! What:
//! Unit tests of the child launch and the exit-status mapping in `shim.rs`.
//! Why:
//! The shadow must report exactly what mpm reported:
//! its exit code,
//! or the shell's code for a signal death,
//!           and it must tell a failed start from a finished run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.unit.test.ts
//! describe('runProgram', () => { it('returns the exit code of the program', () => { ... }); });
//! ```
//! The `/bin/sh` programs below are the child stand-ins:
//! a shell that exits with a chosen code,
//! or kills itself.

/// Imports the exit-code offset and the functions under test from this crate.
use crate::constants::SIGNAL_EXIT_CODE_OFFSET;
use crate::shim::{
    ProgramRun,
    invoked_as_name,
    run_program,
    start_and_wait,
};
/// Imports the message for a start that did not happen,
/// the two reasons one does not happen,
/// and the record of the retries.
use crate::start_failure::{
    StartFailure,
    StartRefusal,
    describe_start_failure,
};
use crate::start_retry::RetryReport;
/// Imports the disposable directory the vanished-program case names a path in.
use crate::test_scratch::scratch;

/// Imports the owned operating-system text type the arguments use.
use std::ffi::OsString;

/// Imports the kinds a failed start is told apart by.
use std::io::ErrorKind;
/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// What:
/// `fn shell(script: &str) -> Vec<OsString>` declares a helper building the argument list `-c <script>`.
/// Why:
/// `/bin/sh -c <script>` is a program whose behavior the test fully controls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shell = (script: string) => ['-c', script];
/// ```
fn shell(script: &str) -> Vec<OsString> {
    // `vec![...]` builds a growable list; `OsString::from(text)` converts a text into the operating-system text type.
    return vec![OsString::from("-c"), OsString::from(script)];
}

/// The child's own exit code comes back unchanged.
#[test]
fn returns_the_exit_code_of_the_program() {
    // `.expect("...")` unwraps the `Ok` value or fails the test with that message.
    assert_eq!(
        run_program(Path::new("/bin/sh"), &shell("exit 7")).expect("the shell starts"),
        7
    );
}

/// A child killed by SIGTERM (signal 15) is reported as 128 + 15,
/// the way a shell reports it.
#[test]
fn reports_death_by_signal_as_128_plus_the_signal() {
    assert_eq!(
        run_program(Path::new("/bin/sh"), &shell("kill -TERM $$")).expect("the shell starts"),
        SIGNAL_EXIT_CODE_OFFSET + 15
    );
}

/// A program that cannot be started is an error,
/// not an exit code,
/// so the caller does not prune after it.
#[test]
fn reports_a_failed_start_as_an_error() {
    // `.is_err()` is true for the `Err` case of a `Result`.
    assert!(run_program(Path::new("/nonexistent/mpm"), &[]).is_err());
}

/// What:
/// A program that is there when the lookup finds it and gone before the start opens it keeps the system's
/// own kind through the vetting,
/// so the message is chosen by what happened;
/// a program that vanished before its start also gets the missing-program words,
/// never the advice to run it by hand.
/// Why:
/// The start's error used to be wrapped in a text with no kind,
/// so a missing program reached the catch-all arm and the message told the person to run the missing
/// program by hand to see the same error.
#[test]
fn a_program_that_is_gone_before_its_start_keeps_the_systems_kind_and_gets_the_missing_program_words()
 {
    let folder: tempfile::TempDir = scratch();
    let missing: PathBuf = folder.path().join("mpm");
    let arguments: Vec<OsString> = vec![OsString::from("installed")];

    let (error, retry): (std::io::Error, RetryReport) =
        match start_and_wait(&missing, &arguments, 1) {
            ProgramRun::NotStarted {
                refusal: StartRefusal::System {
                    error,
                },
                retry,
            } => (error, retry),
            other => panic!("a program that is not there starts nothing: {other:?}"),
        };

    assert_eq!(error.kind(), ErrorKind::NotFound, "{error}");
    assert_eq!(
        retry.refusals, 0,
        "a missing program is not a refusal to retry"
    );
    let text: String = describe_start_failure(&StartFailure {
        outer: &arguments,
        program: &missing,
        refusal: &StartRefusal::System {
            error,
        },
        retry,
        depth: 0,
    });
    assert!(text.contains("is not there"), "{text}");
    assert!(text.contains("install it again"), "{text}");
    assert!(
        !text.contains("by hand"),
        "a file that is not there shows nothing when it is run by hand: {text}"
    );
}

/// A process started with an empty argument list has no argv[0];
/// the name is then empty,
/// the way the absent-name case already reads it,
/// instead of an index panic at startup.
#[test]
fn an_empty_argument_list_yields_an_empty_invoked_name_instead_of_panicking() {
    assert_eq!(invoked_as_name(&[]), String::new());
}

/// The invoked name is argv[0]'s last path segment.
#[test]
fn the_invoked_name_is_the_last_segment_of_argument_zero() {
    assert_eq!(
        invoked_as_name(&[OsString::from("/x/y/mpm")]),
        "mpm".to_string()
    );
    assert_eq!(
        invoked_as_name(&[OsString::from("mpm-shadow"), OsString::from("prune")]),
        "mpm-shadow".to_string()
    );
}
