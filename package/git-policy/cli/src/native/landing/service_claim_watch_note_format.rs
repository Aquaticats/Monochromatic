//! What:
//!  One note of the claim watch written and read back:
//!  the words that say its format,
//!  the bound on how long it may be,
//!  the spelling of a Git directory's path in hexadecimal and back,
//! and the name a note is written under.
//! Why:
//!  A note is the one durable word naming the repository whose after-commit work still waits,
//! so its format,
//! its length bound,
//! its spelling and its name answer together
//!  every question about a note that will not read.
//! The store that keeps notes is `service_claim_watch.rs`,
//! which re-exports what this file holds,
//!  so a caller names that file alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const encode = (gitDir) => `claim-watch-1 ${hexadecimalOfBytes(gitDir)}\n`
//! const decode = (bytes) => notePathOrRefusal(bytes) // the absolute path a note names, or why it is no note
//! const watchName = (gitDir, attempt) => `${fnvHex(encode(gitDir))}-${attempt}.watch`
//! ```

use crate::private_storage::{ReadRefusal, read_regular_file};
use crate::push_failure_notice::fnv_hex;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// The extension of a watch file.
pub(crate) const WATCH_EXTENSION: &str = "watch";

/// The first words of a watch file,
/// which also say its format.
const FORMAT_PREFIX: &str = "claim-watch-1 ";

/// The most bytes a path of this system may have,
/// which bounds every note this store writes.
const LONGEST_PATH_BYTES: u64 = 4096;

/// The most bytes one note may hold.
/// Why:
///   A note is the format words,
/// the bytes of one Git directory's path in hexadecimal,
/// and a last newline;
/// a path of this system's longest is `LONGEST_PATH_BYTES` bytes and its hex is twice that,
/// so a longer file is not one this store wrote,
/// and reading it whole would cost the service memory for nothing.
pub(crate) const NOTE_FILE_LIMIT: u64 = FORMAT_PREFIX.len() as u64 + 2 * LONGEST_PATH_BYTES + 1;

/// How a read of a watch file is done;
/// a control replaces it to make a read fail.
pub type ReadFile<'a> = &'a dyn Fn(&Path) -> Result<Vec<u8>, ReadRefusal>;

/// What:
///  The value of one hexadecimal digit,
/// nothing for a character that is not one.
/// Why:
///  `decode` turns the digits of a note into numbers before it pairs them into bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hexDigitValue = (digit: string) => /[0-9a-f]/i.test(digit) ? parseInt(digit, 16) : undefined;
/// ```
fn hex_digit_value(digit: char) -> Option<u32> {
    return digit.to_digit(16);
}

/// What:
///  The byte two hexadecimal digit values make,
/// nothing when they do not fit one byte.
/// Why:
///  `decode` pairs the digits of a note,
/// high digit first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byteOfDigitPair = ([high, low]: number[]) => high * 16 + low;
/// ```
fn byte_of_digit_pair(pair: &[u32]) -> Option<u8> {
    return u8::try_from(pair[0] * 16 + pair[1]).ok();
}

/// What:
///  Reads one note the way every record of this wrapper is read.
/// Why:
///  `restored` and `retry` hand it to the versions that take the reading as a parameter,
/// so a control can make a read fail,
/// and `private_storage::read_regular_file` answers a name that holds no regular file at once,
/// so a pipe at a note's name is never waited on,
/// a link at it is never read through,
/// and a file longer than a note can be is never read into memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readFile = (path: string) => readRegularFile(path, NOTE_FILE_LIMIT);
/// ```
pub(crate) fn read_file(path: &Path) -> Result<Vec<u8>, ReadRefusal> {
    return read_regular_file(path, NOTE_FILE_LIMIT);
}

/// What a file holds,
/// or why it holds nothing.
pub(crate) fn decode(bytes: &[u8]) -> Result<PathBuf, String> {
    let text: &str = match std::str::from_utf8(bytes) {
        Ok(found) => found,
        Err(_not_text) => return Err(String::from("the file is not text")),
    };
    let Some(after_prefix) = text.strip_prefix(FORMAT_PREFIX) else {
        return Err(String::from("the file does not start with the format words or lacks its last newline"));
    };
    let Some(hex) = after_prefix.strip_suffix('\n') else {
        return Err(String::from("the file does not start with the format words or lacks its last newline"));
    };
    if hex.len() % 2 != 0 || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err(String::from("the path is not written in hexadecimal"));
    }
    let digits: Vec<u32> = hex.chars().filter_map(hex_digit_value).collect();
    let raw: Vec<u8> = digits.chunks(2).filter_map(byte_of_digit_pair).collect();
    let path: PathBuf = PathBuf::from(std::ffi::OsString::from_vec(raw));
    if !path.is_absolute() || path.as_os_str().as_bytes().contains(&0) {
        return Err(String::from("the path is not absolute"));
    }
    return Ok(path);
}

/// The text of the file of `git_dir`:
/// the format words and the bytes of the path in hexadecimal,
/// so that any path is kept as it is.
pub(crate) fn encode(git_dir: &Path) -> String {
    // The crate's one encoder spells the path's bytes (`hexadecimal_of_bytes.rs`).
    let hex: String = crate::hexadecimal_of_bytes::hexadecimal_of_bytes(git_dir.as_os_str().as_bytes());
    return format!("{FORMAT_PREFIX}{hex}\n");
}

/// The file name of the `attempt`th candidate for `git_dir`:
/// a hash of its bytes,
/// and the number of the candidate,
/// which only
/// differs from zero when another directory's file holds the first name.
pub(crate) fn watch_name(git_dir: &Path, attempt: u32) -> String {
    return format!("{}-{attempt}.{WATCH_EXTENSION}", fnv_hex(encode(git_dir).as_str()));
}
