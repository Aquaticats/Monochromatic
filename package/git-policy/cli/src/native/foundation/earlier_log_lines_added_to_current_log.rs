//! What:
//!  Add every byte of a log an earlier release wrote (`cli-git-commit-timing.jsonl`) to the log under the current name,
//! then
//!  remove the earlier log,
//! so that a repository whose history is split across both names has one log again.
//! Why:
//!  Development builds of the renamed program created the current names while the installed earlier release kept writing the
//!  earlier ones,
//! and the owner said "never discard user data".
//! Other programs of this release append to the current log
//!  with no lock at this moment,
//! so the current log is never rewritten or replaced:
//! the earlier log is moved out of its
//!  writers' way by one rename that never replaces,
//! its bytes are appended to the current log in one write,
//! and only then is
//!  it removed.
//! Lines are therefore added after the current log's lines,
//! in the earlier log's own order,
//! and no line changes
//!  a byte (the only bytes added are a newline that ends a last line that had none).
//!  A journal makes the fold restartable after a kill at any instruction.
//! It records where the current log ended before the
//!  append and which bytes of the moved log are being appended;
//! a restart looks for exactly those bytes at or after that
//!  place (and in the current log's numbered older copies,
//! because the next append may have rotated the log),
//! appends them
//!  only when they are not there,
//! and then removes the moved log and the journal.
//! The journal only grows:
//! each record is a new
//!  last line,
//! and the last whole line that is a record wins,
//! so no instant exists in which a truncated journal records
//!  nothing.
//! A journal left by an earlier fold is emptied before the move that starts a new one,
//! so a restart never resumes
//!  its last line against a new moved log.
//! The journal file is also the lock:
//! an advisory `flock` on it is held while one
//!  process folds,
//! and the kernel releases it when that process dies,
//! so no stale lock can outlive a kill.
//! The one lock
//!  covers one log and is held for one fold.
//!  A write that a kill cut short can leave the first lines of the added bytes in the current log;
//! the restart then adds the
//!  whole block again,
//! so the cut lines appear twice and none is lost.
//!  A program of an earlier release that opened the log before the move (a checkout whose lock file names an older release
//!  runs that release's wrapper) can still write a line to the moved log after
//!  any size check.
//! So the moved log is removed only after `file_open_for_write_by_any_process.rs` found no process holding
//!  it open for writing,
//! and the size is checked once more after that probe:
//! with no writer open and the name private,
//! no line
//!  can arrive,
//! so that check is final.
//! While a writer holds the moved log,
//! the fold appends a last journal line that says
//!  every byte before a place is already in the current log,
//! leaves the moved log and the journal,
//! and ends silently;
//! the
//!  next command resumes from that line,
//! adds only what grew,
//! and tries the probe again.
//! One command runs at most
//!  `ROUNDS_PER_COMMAND` rounds,
//! so a log that never stops growing cannot keep a command waiting.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await foldLog(gitDir, COMMIT_TIMING_LOG); // cli-git-commit-timing.jsonl is gone, its lines end the new log
//! ```

use crate::file_open_for_write_by_any_process::{WriterCheck, writer_check};
use crate::private_storage::sync_directory;
use crate::rename_that_never_replaces::rename_noreplace;
use crate::wrapper_names_old_and_new::RenamedName;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// The end of the name the earlier log is moved to while its bytes are added:
/// the name of the current log with this after it.
pub const WAITING_SUFFIX: &str = ".earlier-lines-to-add";

/// The end of the name of the journal and lock file of one fold:
/// the name of the current log with this after it.
pub const JOURNAL_SUFFIX: &str = ".earlier-lines-journal";

/// The most rounds one command adds before it leaves the moved log for a later command.
///  Derived from two measurements and the owner's rule "nothing should ever wait forever or indefinitely".
/// The fastest writing of
///  the timing logs read on 2026-11-07 (1,112 records of the current commit timing log,
/// 257 of the earlier one,
/// 114 and 19 of the
///  two worktree-add timing logs) was 4 records in one second,
/// so a program writes a record at most about every 250 ms. A whole
///  fold of one log (one round plus the move,
/// the directory syncs,
/// the probe and the removals,
/// so an upper bound on one round) added a median of 43 ms to `status --short` on this
///  host (47.8 ms against 4.7 ms with nothing to fold,
/// release build,
/// nine alternating trials in fresh repositories,
/// the
///  same-binary control spread 44.3 to 53.1 ms).
/// A round is therefore at least about six times shorter than the gap between records,
/// so one
///  extra round already catches every writer seen.
/// Eight rounds cost about 0.35 s at that median,
/// one fourteenth of the 5 s
///  by which the owner said a commit must not be slower than native Git,
/// and leave the log to the next command after that.
pub const ROUNDS_PER_COMMAND: usize = 8;

/// The steps of a fold at which a control can stop it,
/// as if the process had been killed there.
pub trait FoldSteps {
    /// Called as a fold reaches `step`;
    /// `false` ends the fold there,
    /// leaving the disk as a kill at that instant would.
    fn may_continue(&mut self, step: &'static str) -> bool;
}

/// The steps of a fold that is never stopped.
pub struct NeverStopped;

impl FoldSteps for NeverStopped {
    fn may_continue(&mut self, _step: &'static str) -> bool {
        return true;
    }
}

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

/// The place in the current log and the bytes of the moved log that one round of a fold adds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Round {
    /// The length of the current log before the round's append:
    /// the append starts at or after it.
    offset: u64,
    /// The first byte of the moved log that the round adds.
    from: u64,
    /// One past the last byte of the moved log that the round adds.
    to: u64,
}

/// The journal's text for a round:
/// three decimal numbers and a newline.
fn journal_text(round: Round) -> String {
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
fn parse_journal(text: &str) -> Option<Round> {
    return text.split_inclusive('\n').rev().find_map(parse_journal_line);
}

/// Whether anything exists at a path,
/// a final symbolic link counting as present.
fn present(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// The journal file,
/// held with an exclusive advisory lock until it is dropped.
struct HeldJournal {
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
fn hold_journal(path: &Path) -> std::io::Result<Option<HeldJournal>> {
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
fn hold_journal(_path: &Path) -> std::io::Result<Option<HeldJournal>> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

impl HeldJournal {
    /// The round the journal records,
    /// if it records one.
    fn recorded(&mut self) -> std::io::Result<Option<Round>> {
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
    fn record(&mut self, round: Round) -> std::io::Result<()> {
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
        return self.file.sync_all();
    }

    /// Whether the journal holds no byte.
    fn is_empty(&self) -> std::io::Result<bool> {
        return Ok(self.file.metadata()?.len() == 0);
    }

    /// Empty the journal and make that durable,
    /// for a journal that recorded an earlier fold's rounds.
    fn clear(&mut self) -> std::io::Result<()> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        return self.file.sync_all();
    }
}

/// The bytes of a file from `from` to `to`.
fn read_range(path: &Path, from: u64, to: u64) -> std::io::Result<Vec<u8>> {
    let mut file: File = File::open(path)?;
    file.seek(SeekFrom::Start(from))?;
    let wanted: usize = usize::try_from(to - from).map_err(std::io::Error::other)?;
    let mut bytes: Vec<u8> = vec![0; wanted];
    file.read_exact(bytes.as_mut_slice())?;
    return Ok(bytes);
}

/// Whether `block` starts at the start of a line of `haystack`,
/// looking at the first start at or after `from`.
///  Only line starts are compared,
/// so a long block costs one comparison per line,
/// not one per byte.
fn block_starts_a_line(haystack: &[u8], from: usize, block: &[u8]) -> bool {
    let mut start: usize = from.min(haystack.len());
    loop {
        if haystack.get(start..start + block.len()) == Some(block) {
            return true;
        }
        let Some(newline) = haystack.get(start..).and_then(|rest| return rest.iter().position(|byte| return *byte == b'\n')) else {
            return false;
        };
        start += newline + 1;
    }
}

/// Whether the file at `path` holds `block` at a line start at or after `from`;
/// a missing file holds nothing.
fn file_holds_block(path: &Path, from: u64, block: &[u8]) -> std::io::Result<bool> {
    let bytes: Vec<u8> = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let start: usize = usize::try_from(from).map_err(std::io::Error::other)?;
    return Ok(block_starts_a_line(bytes.as_slice(), start, block));
}

/// The numbered older copies of `current` that exist (`<current>.1`,
/// `<current>.2`,
/// ...).
pub fn numbered_copies(parent: &Path, current: &str) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(parent) else {
        return Vec::new();
    };
    let wanted_start: String = format!("{current}.");
    let mut copies: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let file_name: String = entry.file_name().to_string_lossy().into_owned();
        if let Some(number) = file_name.strip_prefix(wanted_start.as_str())
            && !number.is_empty()
            && number.bytes().all(|byte| return byte.is_ascii_digit())
        {
            copies.push(entry.path());
        }
    }
    return copies;
}

/// Whether the bytes of a round are already in the current log at or after the recorded place,
/// or in an older copy that the
///  log's rotation made of it since.
fn round_already_added(parent: &Path, current: &str, round: Round, bytes: &[u8]) -> std::io::Result<bool> {
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
fn append_block(current: &Path, bytes: &[u8]) -> std::io::Result<()> {
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
    return file.sync_all();
}

/// The length of a file,
/// or nothing when it does not exist.
fn length_of(path: &Path) -> std::io::Result<Option<u64>> {
    return match std::fs::metadata(path) {
        Ok(found) => Ok(Some(found.len())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    };
}

/// Whether the moved log is longer than the bytes already added;
/// a log that is shorter counts as not grown.
fn grown_past(path: &Path, from: u64) -> std::io::Result<bool> {
    return Ok(length_of(path)?.is_some_and(|length| return length > from));
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
fn leave_for_a_later_command(held: &mut HeldJournal, last: Option<Round>, from: u64, current: &Path) -> std::io::Result<LogFold> {
    let said: bool = last.is_some_and(|round| return round.from == from && round.to == from);
    if !said {
        let offset: u64 = length_of(current)?.unwrap_or(0);
        held.record(Round { offset, from, to: from })?;
    }
    return Ok(LogFold::LeftForALaterCommand);
}

/// What:
///  Fold the earlier log of `name` in `parent` into the current log,
/// or finish a fold that a kill interrupted.
/// Why:
///   See the module's account.
/// Every step is a rename that never replaces,
/// an append,
/// a sync or a removal,
/// and `steps` is told
///  before each so a control can stop the fold at that instant.
/// A caller that must not fold while an earlier program may
///  still write the earlier log (`earlier_names_folded_into_current.rs`) checks that before calling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function foldLog(parent: string, name: RenamedName, steps: FoldSteps): LogFold;
/// ```
///
/// # Errors
/// Any failure of the file system (a read-only Git directory,
/// a full disk) is returned with every byte still in place:
/// the
/// earlier log or the moved log,
/// and the journal,
/// are left for the next attempt.
pub fn fold_log(parent: &Path, name: RenamedName, steps: &mut dyn FoldSteps) -> std::io::Result<LogFold> {
    let earlier: PathBuf = parent.join(name.earlier);
    let current: PathBuf = parent.join(name.current);
    let waiting: PathBuf = parent.join(format!("{}{WAITING_SUFFIX}", name.current));
    let journal: PathBuf = parent.join(format!("{}{JOURNAL_SUFFIX}", name.current));
    if !present(earlier.as_path()) && !present(waiting.as_path()) && !present(journal.as_path()) {
        return Ok(LogFold::Nothing);
    }
    if !steps.may_continue("before-journal-lock") {
        return Ok(LogFold::Stopped("before-journal-lock"));
    }
    let Some(mut held) = hold_journal(journal.as_path())? else {
        return Ok(LogFold::Busy);
    };
    let mut recorded: Option<Round> = held.recorded()?;
    if !present(waiting.as_path()) {
        if !present(earlier.as_path()) {
            // A journal without a moved log is what a fold leaves between the removal of the log and of the journal.
            drop(held);
            let _gone = std::fs::remove_file(journal.as_path());
            return Ok(LogFold::Nothing);
        }
        // A journal that survived without its moved log recorded a finished fold's rounds, and a later command must never resume
        //  them against the moved log this one makes, so it is emptied before the move.
        if !held.is_empty()? {
            if !steps.may_continue("before-journal-clear") {
                return Ok(LogFold::Stopped("before-journal-clear"));
            }
            held.clear()?;
        }
        if !steps.may_continue("before-move-aside") {
            return Ok(LogFold::Stopped("before-move-aside"));
        }
        match rename_noreplace(earlier.as_path(), waiting.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(LogFold::Busy),
            Err(error) => return Err(error),
        }
        recorded = None;
        sync_directory(parent)?;
    }
    let mut last: Option<Round> = recorded;
    let mut from: u64 = 0;
    let mut rounds: usize = 0;
    loop {
        let round: Round = match recorded.take() {
            Some(resumed) => resumed,
            None => {
                let to: u64 = length_of(waiting.as_path())?.unwrap_or(from);
                let offset: u64 = length_of(current.as_path())?.unwrap_or(0);
                let fresh: Round = Round { offset, from, to };
                if !steps.may_continue("before-journal-record") {
                    return Ok(LogFold::Stopped("before-journal-record"));
                }
                held.record(fresh)?;
                last = Some(fresh);
                fresh
            }
        };
        let bytes: Vec<u8> = read_range(waiting.as_path(), round.from, round.to)?;
        if !bytes.is_empty() && !round_already_added(parent, name.current, round, bytes.as_slice())? {
            if !steps.may_continue("before-append") {
                return Ok(LogFold::Stopped("before-append"));
            }
            append_block(current.as_path(), bytes.as_slice())?;
        }
        if !steps.may_continue("after-append") {
            return Ok(LogFold::Stopped("after-append"));
        }
        from = round.to;
        rounds += 1;
        // A program that had the moved log open when it was moved may have added a line since; those bytes are the next round.
        if grown_past(waiting.as_path(), from)? {
            if rounds >= ROUNDS_PER_COMMAND {
                return leave_for_a_later_command(&mut held, last, from, current.as_path());
            }
            continue;
        }
        if !steps.may_continue("before-remove-moved-log") {
            return Ok(LogFold::Stopped("before-remove-moved-log"));
        }
        // The probe comes before the last size check: while no program has the moved log open for writing, none can write to it,
        //  so a length that is still `from` after a probe that found no writer is final. A length checked before the probe would
        //  leave room for a line written between the two.
        if writer_check(waiting.as_path())? == WriterCheck::WriterHolds {
            return leave_for_a_later_command(&mut held, last, from, current.as_path());
        }
        if grown_past(waiting.as_path(), from)? {
            if rounds >= ROUNDS_PER_COMMAND {
                return leave_for_a_later_command(&mut held, last, from, current.as_path());
            }
            continue;
        }
        break;
    }
    std::fs::remove_file(waiting.as_path())?;
    sync_directory(parent)?;
    if !steps.may_continue("before-remove-journal") {
        return Ok(LogFold::Stopped("before-remove-journal"));
    }
    std::fs::remove_file(journal.as_path())?;
    drop(held);
    sync_directory(parent)?;
    return Ok(LogFold::Folded);
}

#[cfg(test)]
#[path = "earlier_log_lines_added_to_current_log_tests.rs"]
mod tests;
