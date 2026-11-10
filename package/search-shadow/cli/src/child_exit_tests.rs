//! What:
//! Unit tests of the shell exit code in `child_exit.rs`.
//! Why:
//! Both child-running features report through it,
//! so a change here changes what every shadowed tool's `$?` means.
//! `ExitStatus::from_raw` builds the two shapes a real child produces without starting one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // child-exit.unit.test.ts
//! describe('shellExitCode', () => { ... });
//! ```

/// What:
/// Imports the function under test.
/// Why:
/// The tests call it directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { shellExitCode } from './child_exit.ts';
/// ```
use crate::child_exit::shell_exit_code;

/// Imports `ExitStatus`,
/// the value a finished child produces.
use std::process::ExitStatus;

/// What:
/// Imports the Unix-only trait that builds an exit status from the raw word a wait returns.
/// Why:
/// A test must not start a process to have a status to convert.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: Node has no exit-status constructor.
/// ```
use std::os::unix::process::ExitStatusExt;

/// What:
/// `const EXITED_WITH_CODE_3: i32 = 3 << 8;` is the raw wait word of a child that exited with code 3.
/// Why:
/// The kernel packs the code into the high byte,
/// which is what `from_raw` takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EXITED_WITH_CODE_3 = 3 << 8;
/// ```
const EXITED_WITH_CODE_3: i32 = 3 << 8;

/// A child that exited itself keeps its own code,
/// including a failure code a script checks for.
#[test]
fn an_exit_code_is_reported_as_it_is() {
    assert_eq!(shell_exit_code(ExitStatus::from_raw(EXITED_WITH_CODE_3)), 3);
    assert_eq!(shell_exit_code(ExitStatus::from_raw(0)), 0);
    assert_eq!(shell_exit_code(ExitStatus::from_raw(2 << 8)), 2);
}

/// A child a signal ended gets the shell's `128 + signal`,
/// which is what a shell prints for the same process.
#[test]
fn a_signal_is_reported_as_128_plus_the_signal() {
    // A raw word under 0x80 is a signal, not a code (wait(2)).
    assert_eq!(shell_exit_code(ExitStatus::from_raw(15)), 143);
    assert_eq!(shell_exit_code(ExitStatus::from_raw(9)), 137);
    assert_eq!(shell_exit_code(ExitStatus::from_raw(13)), 141);
}
