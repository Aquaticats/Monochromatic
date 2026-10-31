//! What:
//!  Pulls a few named files out of an npm tarball held in memory.
//! Why:
//!  The data files are the only part of caniuse-lite and node-releases this crate reads;
//!  nothing from the archive is ever written to disk by path,
//! so entry names cannot escape anywhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function extractFiles(tgz: Uint8Array, wanted: string[]): Uint8Array[]; // gunzip, walk tar entries, match by exact path
//! ```

/// What:
///  Import `BrowserslistError, Outcome` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome } from './error';
/// ```
use crate::error::{BrowserslistError, Outcome};
/// What:
///  Import `GzDecoder` from `flate2::read::GzDecoder`.
/// Why:
///  npm tarballs are gzip-compressed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { GzDecoder } from 'flate2/read';
/// ```
use flate2::read::GzDecoder;
/// What:
///  Import `Read` from `std::io::Read`.
/// Why:
///  Byte streams are read and written through these traits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Read } from 'std/io';
/// ```
use std::io::Read;
/// What:
///  Import `Archive` from `tar::Archive`.
/// Why:
///  npm tarballs are tar archives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Archive } from 'tar';
/// ```
use tar::Archive;

/// The most decompressed tar bytes the reader consumes (the real caniuse-lite tarball is under 25 MiB unpacked).
const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;

/// The largest single wanted file accepted.
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// Build an error naming the tarball.
fn failure(subject: &str, message: &str) -> BrowserslistError {
    return BrowserslistError::Data { file: format!("{subject} tarball"), offset: None, message: message.to_string() };
}

/// What:
///  Return the contents of each wanted path,
/// in the order requested.
///  Paths are matched exactly against tar entry paths such as `package/data/agents.js`.
/// Why:
///  A missing,
/// duplicated,
/// oversized or non-regular wanted entry is an error,
/// never a silent gap.
/// Gotcha:
///  Reading stops after `MAX_ARCHIVE_BYTES` of decompressed data,
/// so a gzip bomb cannot exhaust memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function extractFiles(subject: string, tgz: Uint8Array, wanted: string[]): Uint8Array[];
/// ```
pub fn extract_files(subject: &str, tgz: &[u8], wanted: &[&str]) -> Outcome<Vec<Vec<u8>>> {
    let decoder = GzDecoder::new(tgz).take(MAX_ARCHIVE_BYTES);
    let mut archive = Archive::new(decoder);
    let mut found: Vec<Option<Vec<u8>>> = vec![None; wanted.len()];
    let entries = archive.entries().map_err(|error| return failure(subject, &format!("unreadable archive: {error}")))?;
    for entry_result in entries {
        let mut entry = entry_result.map_err(|error| return failure(subject, &format!("unreadable archive entry: {error}")))?;
        let path = entry.path().map_err(|error| return failure(subject, &format!("unreadable entry path: {error}")))?.to_string_lossy().into_owned();
        let Some(position) = wanted.iter().position(|candidate| return *candidate == path) else {
            continue;
        };
        if !entry.header().entry_type().is_file() {
            return Err(failure(subject, &format!("{path} is not a regular file")));
        }
        if found[position].is_some() {
            return Err(failure(subject, &format!("{path} appears more than once")));
        }
        if entry.size() > MAX_FILE_BYTES {
            return Err(failure(subject, &format!("{path} is larger than {MAX_FILE_BYTES} bytes")));
        }
        let mut contents: Vec<u8> = Vec::new();
        entry.read_to_end(&mut contents).map_err(|error| return failure(subject, &format!("{path}: {error}")))?;
        found[position] = Some(contents);
    }
    let mut files: Vec<Vec<u8>> = Vec::with_capacity(wanted.len());
    for (path, contents) in wanted.iter().zip(found) {
        let Some(bytes) = contents else {
            return Err(failure(subject, &format!("{path} is missing")));
        };
        files.push(bytes);
    }
    return Ok(files);
}

/// What:
///  The unit tests of `tarball.rs`,
/// kept in the sidecar file `tarball_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './tarball.test';
/// ```
#[cfg(test)]
#[path = "tarball_tests.rs"]
mod tests;
