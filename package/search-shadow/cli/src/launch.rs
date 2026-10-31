//! What:
//! The one place that replaces the running process with the real tool.
//! Why:
//! Replacing (instead of starting a child and waiting) keeps the real tool's exit code,
//! signals and terminal behavior exactly as they are
//! and leaves no extra process behind,
//! so the shadow adds almost no latency.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // launch.ts
//! export function replaceProcess(program: string, args: string[]): Error { ... }
//! // Node cannot replace its own process; the closest thing is spawnSync with stdio: 'inherit' and process.exit(code).
//! ```

/// What:
/// `use std::ffi::OsString;` imports the owned text type the operating system uses for arguments and file names.
/// The siblings are `String` (always valid UTF-8) and `Vec<u8>` (raw bytes).
/// Why:
/// Search patterns and file names may be invalid UTF-8.
/// `OsString` carries them byte for byte,
/// where `String` would reject or alter them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS strings are UTF-16; Node cannot carry arbitrary bytes in argv, so the problem never shows up.
/// ```
use std::ffi::OsString;

/// What:
/// `use std::io;` imports the standard input/output module for its error type `io::Error`.
/// Why:
/// A failed launch is reported as an `io::Error`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Errors are plain Error objects in TS.
/// ```
use std::io;

/// What:
/// `use std::os::unix::process::CommandExt;` imports a trait (a named bundle of methods that types can opt
/// into) that adds Unix-only methods (here `.exec()`) to `Command`.
/// Why:
/// Without this import `.exec()` does not exist on `Command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: Node has no exec().
/// ```
use std::os::unix::process::CommandExt;

/// Imports `Path` (a borrowed read-only file path).
use std::path::Path;

/// Imports `Command`,
/// the builder for starting another program.
use std::process::Command;

/// What:
/// `pub fn replace_process(program: &Path, arguments: &[OsString]) -> io::Error` declares a public function.
/// It returns an `io::Error` and never returns at all when everything works.
/// Why:
/// On success the operating system swaps this process's code for the real tool,
/// so there is no later line to run.
/// If we ever get a return value,
/// the swap failed and the error says why.
/// Gotcha:
/// A function whose success path never returns has no TypeScript analogue.
/// Read the return value as "this is only what went wrong".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceProcess(program: string, args: string[]): Error {
///   // Not possible in Node; the nearest idea is: spawn the program, then exit with its code.
/// }
/// ```
pub fn replace_process(program: &Path, arguments: &[OsString]) -> io::Error {
    // What:     `let mut command = Command::new(program);` creates a description of the program to run. `mut`
    // lets the next line change it.
    // Why:      We add the arguments before launching.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const command = { program, args: [] as string[] };
    // ```
    let mut command: Command = Command::new(program);

    // `command.args(arguments)` appends every argument; `arguments` is already a borrowed list.
    command.args(arguments);

    // What:     `command.exec()` asks the operating system to replace this process with the program. It returns an
    // `io::Error` only when that failed.
    // Why:      This is the launch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const error = replaceCurrentProcess(command);
    // ```
    let error: io::Error = command.exec();
    return error;
}
