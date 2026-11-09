//! What:
//!  The end of a command that stops without running the caller's request:
//! its report on standard output and its text on standard error.
//! Why:
//!  Two shapes reach it.
//! A report of text is written as it stands;
//! a report of pathnames is written as the bytes it
//!  is made of,
//! because a pathname is a byte string and its text spelling with replacement characters names a
//!  different file (`health/doctor.rs`,
//! `render_hooks`).
//!  The exit status carries the result either way,
//! so a stream that refuses its bytes changes nothing but the stream.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const writeExit = (code, stdout: Buffer, stderr: Buffer) => { fs.writeSync(1, stdout); fs.writeSync(2, stderr); return code; };
//! ```

/// The trait that gives both streams their `write_all`.
use std::io::Write;

/// What:
///  Write the report and the explanation of an exit,
/// and give the exit status back.
///  `&[u8]` is each of them as bytes,
/// already line-terminated.
/// Why:
///   One place writes both shapes,
/// so a report of pathnames cannot be spelled as text by accident,
///  and the doctor's explanation of the Git LFS shims it found names pathnames on the second stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeExit(code, report, explanation);
/// ```
pub(crate) fn write_exit(code: i32, report: &[u8], text: &[u8]) -> i32 {
    // The exit status carries the result, so a refused write changes nothing here.
    let _written = std::io::stdout().write_all(report);
    let _said = std::io::stderr().write_all(text);
    return code;
}
