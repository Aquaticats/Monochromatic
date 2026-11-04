//! What:
//!  The panic hook of the executable:
//!  one line naming where an internal error
//!       happened,
//!  never its message.
//! Why:
//!  The linked scanner catches a matcher panic and reports it as a failed scan,
//!  but
//!      Rust runs the process's panic hook before the panic can be caught,
//!  and the default
//!      hook prints the panic's message,
//!  which can hold bytes of the file being scanned.
//!      The scanner's own executable installs a hook that prints nothing;
//!  this one keeps
//!      the source location,
//!  which holds no repository content,
//!  so a wrapper defect is
//!      still visible.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // process.on('uncaughtException', () => process.stderr.write('cli-git: internal error ...\n'));
//! ```

/// What:
///  `Write` adds `write_all` to the standard error stream.
/// Why:
///   The notice is written whole,
///  as bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.stderr.write(text)
/// ```
use std::io::Write;
/// What:
///  `Location` is a source file,
///  line and column;
///  `PanicHookInfo` is what a panic
///       hook is given,
///  its message and its location.
/// Why:
///   Only the location is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Location = { file: string; line: number; column: number };
/// ```
use std::panic::{Location, PanicHookInfo};

/// What:
///  The line the hook writes,
///  with the location when the panic has one.
///       `Option<&Location<'_>>` is "a borrowed location or nothing";
///  `'_` lets the
///       compiler choose how long the borrow lasts.
/// Why:
///   A pure function of the location can be checked exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function panicNotice(location?: Location): string;
/// ```
pub fn panic_notice(location: Option<&Location<'_>>) -> String {
    // `match` unpacks "a location or nothing".
    match location {
        Some(found) => {
            return format!(
                "cli-git: internal error at {}:{}:{}; its message is not shown because it may \
                 contain repository content. Hint: if you were running a Git command, run it again; your files are \
                 not changed by this notice, so `git status` shows where things stand. If it repeats, report this \
                 location.\n",
                found.file(),
                found.line(),
                found.column()
            );
        }
        None => {
            return String::from(
                "cli-git: internal error; its message is not shown because it may contain \
                 repository content. Hint: if you were running a Git command, run it again; `git status` shows where \
                 things stand. If it repeats, report the command you ran.\n",
            );
        }
    }
}

/// What:
///  The hook itself:
///  write the notice to standard error and nothing else.
///       `&PanicHookInfo<'_>` borrows what the panic carries;
///  its message is never read.
/// Why:
///   A hook cannot report a failure to write,
///  so a closed stream is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function payloadFreePanicHook(info: PanicInfo): void;
/// ```
pub fn payload_free_panic_hook(information: &PanicHookInfo<'_>) {
    // What: `let _ = ...` deliberately discards the write's `Result`.
    // Why:  There is nowhere left to report a failure to write a diagnostic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { process.stderr.write(notice); } catch { /* stream closed */ }
    // ```
    let _ = std::io::stderr().write_all(panic_notice(information.location()).as_bytes());
}

/// What:
///  Make `payload_free_panic_hook` the process's panic hook.
///       `Box::new(..)` puts the named function where the standard library keeps its hook.
/// Why:
///   The executable calls this before anything can scan,
///  as the scanner's embedding
///       contract requires.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installPayloadFreePanicHook(): void;
/// ```
pub fn install_payload_free_panic_hook() {
    std::panic::set_hook(Box::new(payload_free_panic_hook));
}

/// Hook controls,
///  each in its own process,
///  stay out of the release executable.
#[cfg(test)]
#[path = "panic_notice_tests.rs"]
mod tests;
