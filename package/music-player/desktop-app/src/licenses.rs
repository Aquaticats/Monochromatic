//! The license notices the binary carries,
//!  printed by `music-player --licenses`.
//!
//! `build.rs` copies the notice file named by `MUSIC_PLAYER_THIRD_PARTY_LICENSES`
//! (written by the `notices` mise task,
//!  `bin/third-party-notices.mjs`) into the build output,
//!  and this module embeds it.
//! Without that variable a build embeds a short text that says how to get the notices;
//! the release workflow sets `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES=true`,
//!  which turns a missing file into a build failure.
//! Decision:
//! `doc/decision/music-player-third-party-licenses.md`.

/// What:
///      `use std::io::{ErrorKind, Write};`.
///  `Write` is the byte-sink trait (`write_all`,
///  `flush`);
///  `ErrorKind` names the class of an I/O error.
/// Why:
///       The printer writes to any sink so tests can pass a buffer or a closed pipe,
///  and it tells a closed pipe apart from a real failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { Writable } from "node:stream";
/// ```
use std::io::{ErrorKind, Write};

/// What:
///      `pub static NOTICES: &str = include_str!(...)`.
///  The whole notice text,
///  compiled into the binary from the file `build.rs` wrote into `OUT_DIR`.
/// Why:
///       The Windows release is a single executable with nothing beside it,
///  so the notices must travel inside the program.
///  It is a `static`, not a `const`: a `const` is copied into every crate that uses it,
///  and the Licenses page of the window (the binary crate) and `--licenses` (this crate) would each carry the 2 MB text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const NOTICES: string = embedFile(`${OUT_DIR}/third_party_licenses.txt`);
/// ```
pub static NOTICES: &str = include_str!(concat!(env!("OUT_DIR"), "/third_party_licenses.txt"));

/// What:
///      `pub fn write_notices(sink: &mut impl Write) -> std::io::Result<()>`.
///  Writes [`NOTICES`] to `sink` and flushes it.
/// Why:
///       `--licenses` output is long and often piped into a pager or `head`;
///  a reader that closes the pipe early is a normal end,
///  so a broken pipe returns `Ok` instead of an error.
///
/// # Errors
///
/// Returns the sink's error for every failure other than a closed pipe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeNotices(sink: Writable): void {
///   try { sink.write(NOTICES); } catch (error) { if (error.code !== "EPIPE") throw error; }
/// }
/// ```
pub fn write_notices(sink: &mut impl Write) -> std::io::Result<()> {
    let written = match sink.write_all(NOTICES.as_bytes()) {
        Ok(()) => sink.flush(),
        Err(error) => Err(error),
    };
    return match written {
        Err(error) if error.kind() == ErrorKind::BrokenPipe => Ok(()),
        other => other,
    };
}

/// What:
///      `#[cfg(test)] #[path = "licenses_tests.rs"] mod tests;`.
///  The unit tests,
///  in the sibling file.
/// Why:
///       Keep tests beside the code without growing this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import "./licenses_tests";
/// ```
#[cfg(test)]
#[path = "licenses_tests.rs"]
mod tests;
