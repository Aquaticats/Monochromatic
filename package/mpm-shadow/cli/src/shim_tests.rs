//! What:     Unit tests of the child launch and the exit-status mapping in `shim.rs`.
//! Why:      The shadow must report exactly what mpm reported: its exit code, or the shell's code for a signal death,
//!           and it must tell a failed start from a finished run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.unit.test.ts
//! describe('runProgram', () => { it('returns the exit code of the program', () => { ... }); });
//! ```
//! The `/bin/sh` programs below are the child stand-ins: a shell that exits with a chosen code, or kills itself.

/// Imports the function under test from this crate.
use crate::shim::run_program;

/// Imports the owned operating-system text type the arguments use.
use std::ffi::OsString;

/// Imports the borrowed path type.
use std::path::Path;

/// What:     `fn shell(script: &str) -> Vec<OsString>` declares a helper building the argument list `-c <script>`.
/// Why:      `/bin/sh -c <script>` is a program whose behavior the test fully controls.
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
    assert_eq!(run_program(Path::new("/bin/sh"), &shell("exit 7")).expect("the shell starts"), 7);
}

/// A child killed by SIGTERM (signal 15) is reported as 128 + 15, the way a shell reports it.
#[test]
fn reports_death_by_signal_as_128_plus_the_signal() {
    assert_eq!(run_program(Path::new("/bin/sh"), &shell("kill -TERM $$")).expect("the shell starts"), 143);
}

/// A program that cannot be started is an error, not an exit code, so the caller does not prune after it.
#[test]
fn reports_a_failed_start_as_an_error() {
    // `.is_err()` is true for the `Err` case of a `Result`.
    assert!(run_program(Path::new("/nonexistent/mpm"), &[]).is_err());
}
