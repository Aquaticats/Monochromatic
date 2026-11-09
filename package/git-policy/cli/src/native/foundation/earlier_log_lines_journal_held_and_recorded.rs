//! What:
//!  The journal of a log fold:
//! the round it records (a place in the current log and a range of the moved log),
//! the text of a record,
//! and the journal file held with an exclusive advisory lock.
//! Why:
//!  `earlier_log_lines_added_to_current_log.rs` holds the fold itself and the rules that make it restartable;
//! the journal's own parts use nothing of the fold,
//! so they sit in a file of their own and the fold uses them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Round = { offset: number; from: number; to: number };
//! const held = await tryLock(journalPath); // undefined when another process holds it
//! held.record({ offset, from, to }); // a new last line, durable before the append starts
//! ```

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// The place in the current log and the bytes of the moved log that one round of a fold adds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Round {
    /// The length of the current log before the round's append:
    /// the append starts at or after it.
    pub(crate) offset: u64,
    /// The first byte of the moved log that the round adds.
    pub(crate) from: u64,
    /// One past the last byte of the moved log that the round adds.
    pub(crate) to: u64,
}

/// The journal's text for a round:
/// three decimal numbers and a newline.
pub(crate) fn journal_text(round: Round) -> String {
    return format!("{} {} {}\n", round.offset, round.from, round.to);
}

/// The round one complete journal line records,
/// or nothing when the line is not a record.
fn parse_journal_line(whole_line: &str) -> Option<Round> {
    let line: &str = whole_line.strip_suffix('\n')?;
    let mut numbers = line.split(' ').map(str::parse::<u64>);
    let offset: u64 = numbers.next()?.ok()?;
    let from: u64 = numbers.next()?.ok()?;
    let to: u64 = numbers.next()?.ok()?;
    if numbers.next().is_some() || from > to {
        return None;
    }
    return Some(Round { offset, from, to });
}

/// The round a journal's text records:
/// its last complete line that is a record,
/// or nothing when there is none.
/// A line cut short
///  records nothing (nothing was appended yet then,
/// because the append starts after the journal is synced),
/// and the journal only
///  grows,
/// so a round is never recorded by overwriting the one before it.
pub(crate) fn parse_journal(text: &str) -> Option<Round> {
    return text.split_inclusive('\n').rev().find_map(parse_journal_line);
}

/// The journal file,
/// held with an exclusive advisory lock until it is dropped.
pub(crate) struct HeldJournal {
    /// The open file;
    /// closing it releases the lock.
    file: File,
}

/// What:
///  Open the journal (creating it),
/// take its exclusive lock without waiting,
/// and check that the name still refers to the
///  file that was locked.
/// Why:
///   A process that locks a file another process has just removed holds a lock nobody else sees;
/// the check of the name after
///  the lock makes that case a retry.
/// A lock held by another process means that process is folding this log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = await tryLock(journalPath); // undefined when another process holds it
/// ```
#[cfg(unix)]
pub(crate) fn hold_journal(path: &Path) -> std::io::Result<Option<HeldJournal>> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    // The loop ends: each pass either returns or follows the removal of the name by a finished fold, which happens once per fold.
    loop {
        let file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(crate::private_storage::PRIVATE_FILE_MODE)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)?;
        // SAFETY: the descriptor is open for the call.
        let locked: i32 = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if locked != 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EWOULDBLOCK) {
                return Ok(None);
            }
            return Err(error);
        }
        let held = file.metadata()?;
        match std::fs::symlink_metadata(path) {
            Ok(named) if (named.dev(), named.ino()) == (held.dev(), held.ino()) => return Ok(Some(HeldJournal { file })),
            Ok(_) | Err(_) => {}
        }
    }
}

/// A platform without `flock` cannot hold the fold exclusive,
/// so it does not fold.
#[cfg(not(unix))]
pub(crate) fn hold_journal(_path: &Path) -> std::io::Result<Option<HeldJournal>> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

impl HeldJournal {
    /// The round the journal records,
    /// if it records one.
    pub(crate) fn recorded(&mut self) -> std::io::Result<Option<Round>> {
        let mut text: String = String::new();
        self.file.seek(SeekFrom::Start(0))?;
        // A journal that is not text was not written by a complete record, so it records nothing.
        if self.file.read_to_string(&mut text).is_err() {
            return Ok(None);
        }
        return Ok(parse_journal(text.as_str()));
    }

    /// Record a round as a new last line and make the record durable before the round's append starts.
    /// A last line that a kill cut
    ///  short is ended first,
    /// so the new line is a line of its own and the cut one stays unreadable.
    pub(crate) fn record(&mut self, round: Round) -> std::io::Result<()> {
        let length: u64 = self.file.seek(SeekFrom::End(0))?;
        let mut line: Vec<u8> = Vec::new();
        if length > 0 {
            let mut last: [u8; 1] = [0];
            self.file.seek(SeekFrom::End(-1))?;
            self.file.read_exact(&mut last)?;
            if last[0] != b'\n' {
                line.push(b'\n');
            }
        }
        line.extend_from_slice(journal_text(round).as_bytes());
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(line.as_slice())?;
        // A control whose subject is not what a crash leaves skips the flush, which on a bad disk costs a second and more.
        if crate::private_storage::syncing_is_faithful() {
            self.file.sync_all()?;
        }
        return Ok(());
    }

    /// Whether the journal holds no byte.
    pub(crate) fn is_empty(&self) -> std::io::Result<bool> {
        return Ok(self.file.metadata()?.len() == 0);
    }

    /// Empty the journal and make that durable,
    /// for a journal that recorded an earlier fold's rounds.
    pub(crate) fn clear(&mut self) -> std::io::Result<()> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        if crate::private_storage::syncing_is_faithful() {
            self.file.sync_all()?;
        }
        return Ok(());
    }
}
