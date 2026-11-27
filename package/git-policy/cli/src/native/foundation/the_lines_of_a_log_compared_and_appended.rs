//! What:
//!  The bytes of a log,
//! as a fold of it uses them:
//!  whether the log or its moved copy exists,
//!  the bytes of one round and the numbered older copies they may already be in,
//!  the one write that adds a round to the current log,
//!  where the moved log stands,
//!  and the record that leaves a log that kept growing for a later command.
//! Why:
//!   A fold's order is `earlier_log_lines_added_to_current_log.rs`;
//!  what a single file holds and how bytes reach it is here,
//!  so the reader of either file can read it on its own.
//! A journal records a round's place and the answer that leaves a moved log for a later command,
//!  so the type that says how a fold ended lives here too,
//!  beside that answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LogFold = 'nothing' | 'folded' | 'leftForALaterCommand' | 'busy' | { stopped: string };
//! function present(path: string): boolean;
//! function readRange(path: string, from: number, to: number): Buffer;
//! function blockStartsALine(haystack: Buffer, from: number, block: Buffer): boolean;
//! function fileHoldsBlock(path: string, from: number, block: Buffer): boolean;
//! function numberedCopies(parent: string, current: string): string[];
//! function roundAlreadyAdded(parent: string, current: string, round: Round, bytes: Buffer): boolean;
//! function appendBlock(current: string, bytes: Buffer): void;
//! function lengthOf(path: string): number | undefined;
//! function grownPast(path: string, from: number): boolean;
//! function leaveForALaterCommand(held: HeldJournal, last: Round | undefined, from: number, current: string): LogFold;
//! ```

/// The journal a fold holds,
/// and the round it records.
use crate::earlier_log_lines_journal_held_and_recorded::{
    HeldJournal,
    Round,
};
/// Reading and appending the current log.
use std::fs::{
    File,
    OpenOptions,
};
/// Reading a range of a log,
/// seeking and writing.
use std::io::{
    Read,
    Seek,
    SeekFrom,
    Write,
};
/// Paths of the parent folder and the logs.
use std::path::{
    Path,
    PathBuf,
};

/// How a fold of one log ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogFold {
    /// No earlier log,
    /// moved log or journal existed.
    Nothing,
    /// Every byte of the earlier log is in the current log,
    /// and the earlier log and the journal are gone.
    Folded,
    /// A program still has the moved log open for writing,
    /// or the rounds of this command ran out while it kept growing:
    /// every byte
    ///  added so far is in the current log and the journal says where the next command continues,
    /// so no line is lost and none
    ///  is added twice.
    LeftForALaterCommand,
    /// Another process holds the fold of this log,
    /// or the earlier log vanished under another process's rename.
    Busy,
    /// A control stopped the fold at the named step.
    Stopped(&'static str),
}

/// Whether anything exists at a path,
/// a final symbolic link counting as present.
pub(crate) fn present(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// The bytes of a file from `from` to `to`.
pub(crate) fn read_range(
    path: &Path,
    from: u64,
    to: u64,
) -> std::io::Result<Vec<u8>> {
    let mut file: File = File::open(path)?;
    file.seek(SeekFrom::Start(from))?;
    let wanted: usize = usize::try_from(to - from).map_err(std::io::Error::other)?;
    let mut bytes: Vec<u8> = vec![0; wanted];
    file.read_exact(bytes.as_mut_slice())?;
    return Ok(bytes);
}

/// Whether a byte ends a line.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// The place of the first line end in `bytes`,
/// or nothing when there is none.
fn first_newline_in(bytes: &[u8]) -> Option<usize> {
    return bytes.iter().position(is_newline);
}

/// Whether `block` starts at the start of a line of `haystack`,
/// looking at the first start at or after `from`.
///  Only line starts are compared,
/// so a long block costs one comparison per line,
/// not one per byte.
fn block_starts_a_line(
    haystack: &[u8],
    from: usize,
    block: &[u8],
) -> bool {
    let mut start: usize = from.min(haystack.len());
    loop {
        if haystack.get(start..start + block.len()) == Some(block) {
            return true;
        }
        let Some(newline) = haystack.get(start..).and_then(first_newline_in) else {
            return false;
        };
        start += newline + 1;
    }
}

/// Whether the file at `path` holds `block` at a line start at or after `from`;
/// a missing file holds nothing.
fn file_holds_block(
    path: &Path,
    from: u64,
    block: &[u8],
) -> std::io::Result<bool> {
    let bytes: Vec<u8> = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let start: usize = usize::try_from(from).map_err(std::io::Error::other)?;
    return Ok(block_starts_a_line(bytes.as_slice(), start, block));
}

/// Whether a byte is an ASCII digit.
fn is_ascii_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// The numbered older copies of `current` that exist (`<current>.1`,
/// `<current>.2`,
/// ...).
pub fn numbered_copies(
    parent: &Path,
    current: &str,
) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(parent) else {
        return Vec::new();
    };
    let wanted_start: String = format!("{current}.");
    let mut copies: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let file_name: String = entry.file_name().to_string_lossy().into_owned();
        if let Some(number) = file_name.strip_prefix(wanted_start.as_str())
            && !number.is_empty()
            && number.bytes().all(is_ascii_digit_byte)
        {
            copies.push(entry.path());
        }
    }
    return copies;
}

/// Whether the bytes of a round are already in the current log at or after the recorded place,
/// or in an older copy that the
///  log's rotation made of it since.
pub(crate) fn round_already_added(
    parent: &Path,
    current: &str,
    round: Round,
    bytes: &[u8],
) -> std::io::Result<bool> {
    if file_holds_block(parent.join(current).as_path(), round.offset, bytes)? {
        return Ok(true);
    }
    for copy in numbered_copies(parent, current) {
        if file_holds_block(copy.as_path(), 0, bytes)? {
            return Ok(true);
        }
    }
    return Ok(false);
}

/// What:
///  Append `bytes` to the current log in one write,
/// ending the log's last line first when it has no newline and ending the
///  added bytes with a newline when they have none.
/// Why:
///   The log is a file of lines,
/// and the next writer starts its line where the file ends.
/// The append mode makes the write land at
///  the end whatever other programs appended meanwhile,
/// and the sync makes the bytes durable before the moved log is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await appendFile(current, lead + bytes + trail); await sync(current);
/// ```
pub(crate) fn append_block(
    current: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    let mut file: File = OpenOptions::new().append(true).create(true).open(current)?;
    let length: u64 = file.metadata()?.len();
    let mut block: Vec<u8> = Vec::with_capacity(bytes.len() + 2);
    if length > 0 {
        let mut last: [u8; 1] = [0];
        let mut reader: File = File::open(current)?;
        reader.seek(SeekFrom::Start(length - 1))?;
        reader.read_exact(&mut last)?;
        if last[0] != b'\n' {
            block.push(b'\n');
        }
    }
    block.extend_from_slice(bytes);
    if bytes.last() != Some(&b'\n') {
        block.push(b'\n');
    }
    file.write_all(block.as_slice())?;
    // A control whose subject is not what a crash leaves skips the flush, which on a bad disk costs a second and more.
    if crate::private_storage::syncing_is_faithful() {
        file.sync_all()?;
    }
    return Ok(());
}

/// The length of a file,
/// or nothing when it does not exist.
pub(crate) fn length_of(path: &Path) -> std::io::Result<Option<u64>> {
    return match std::fs::metadata(path) {
        Ok(found) => Ok(Some(found.len())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    };
}

/// Whether the moved log is longer than the bytes already added;
/// a log that is shorter counts as not grown.
pub(crate) fn grown_past(
    path: &Path,
    from: u64,
) -> std::io::Result<bool> {
    return match length_of(path)? {
        Some(length) => Ok(length > from),
        None => Ok(false),
    };
}

/// What:
///  Leave the moved log and the journal for a later command,
/// after recording that every byte before `from` is already in the
///  current log (a round whose first and last byte are both `from`),
/// unless the journal's last line already says so.
/// Why:
///   A program that opened the log before the move may write more lines to it,
/// so the log cannot be removed now.
/// The next
///  command resumes from this record and adds only what grew since,
/// which a record of the last real round would not say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leaveForALaterCommand(held: HeldJournal, last: Round | undefined, from: number): LogFold;
/// ```
pub(crate) fn leave_for_a_later_command(
    held: &mut HeldJournal,
    last: Option<Round>,
    from: u64,
    current: &Path,
) -> std::io::Result<LogFold> {
    let said: bool = match last {
        Some(round) => round.from == from && round.to == from,
        None => false,
    };
    if !said {
        let offset: u64 = length_of(current)?.unwrap_or(0);
        held.record(Round {
            offset,
            from,
            to: from,
        })?;
    }
    return Ok(LogFold::LeftForALaterCommand);
}
