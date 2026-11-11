//! What:
//!  The Git directories whose after-commit work this service instance has been woken for,
//! kept on disk in the instance's own
//!  journal,
//! so that a service that starts again looks at them by itself.
//! Why:
//!  The service learns which repository to drain only from a client's wake,
//! and a client wakes it only after a commit.
//! A service
//!  that was killed while it held a claim,
//! or that ended with work still pending,
//! was started again by the next connection and
//!  then had nothing to drain until the next commit,
//! which might never come:
//! the backup push of the last commit of the day waited
//!  without a bound.
//! The journal survives a restart and has one writer,
//! so it can name the directories to look at when the service
//!  starts.
//! The name is written by the worker thread before it drains a directory for the first time in a process,
//! never by the
//!  thread that lands commits.
//!
//! Identity:
//! an instance is named by a hash of the canonical path of its worktree's Git directory (`service_instance::instance_of`),
//!  so the instance is the repository.
//! A file is kept only when the path it holds names this instance,
//! which a path that now
//!  resolves through a link elsewhere,
//! a copied journal or a damaged file does not;
//! the work directory it names must be a real
//!  directory,
//! not a link.
//! A repository that moved is another instance and is found by its own wake.
//! Errors:
//! only a file that is certainly of no use is removed (it holds no path,
//! names another instance,
//! or its work directory is
//!  absent or not a directory).
//! A read or write that fails for any other reason leaves the file where it is and is tried again
//!  after a growing pause (`needs_retry`,
//! `retry`),
//! so a refused read or a full disk delays the look and never ends it.
//! Durability:
//! a file is written whole under a staging name and published by one rename (`write_private_file_unsynced`),
//! so a kill
//!  leaves a complete file or none.
//! It is not synced:
//! it survives the end of the process,
//! like the work records it points at,
//!  and not a power cut.
//! Size:
//! a journal belongs to one instance,
//! so it holds one file per spelling of that instance's Git directory that clients have
//!  used,
//! and a file is removed at the start once its work directory is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = restored(journalDir, instance); watch.remember(gitDir); for (const dir of watch.directories()) await drain(dir);
//! ```

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::push_failure_notice::fnv_hex;
use crate::service_instance::instance_of;
use crate::work_item::WORK_DIRECTORY;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Directory,
/// under the service instance's journal,
/// that holds one small file per watched Git directory.
pub const WATCH_DIRECTORY: &str = "claim-watch";

/// The extension of a watch file.
const WATCH_EXTENSION: &str = "watch";

/// The first words of a watch file,
/// which also say its format.
const FORMAT_PREFIX: &str = "claim-watch-1 ";

/// How a read of a watch file is done;
/// a control replaces it to make a read fail.
pub type ReadFile<'a> = &'a dyn Fn(&Path) -> std::io::Result<Vec<u8>>;

/// What:
///  The Git directories to look at when the service starts,
/// the ones whose file could not be written yet,
/// and whether a read of
///  the store failed and must be made again.
/// Why:
///   A directory that was written down is not written again,
/// so the file system is touched once per directory per process;
/// one
///  that could not be written,
/// or a store that could not be read,
/// is tried again until it works.
/// The watch is cloned into a job of the blocking pool when the loop that owns it is a task,
/// because a job
///  may not borrow it;
/// the job hands the changed watch back and the clone is dropped,
/// so only one watch is ever the
///  owner's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ClaimWatch { store?: string; instance?: string; known: string[]; unwritten: string[]; unread: boolean; failures: number; }
/// ```
#[derive(Clone, Debug, Default)]
pub struct ClaimWatch {
    /// The directory the watch files are kept in;
    /// nothing for a watch kept in memory only.
    store: Option<PathBuf>,
    /// The instance this journal belongs to,
    /// or nothing for a run that serves any worktree.
    instance: Option<String>,
    /// The Git directories known to this process,
    /// from the files read or written.
    known: Vec<PathBuf>,
    /// The Git directories this process was woken for whose file could not be written.
    unwritten: Vec<PathBuf>,
    /// Whether the store or one of its files could not be read,
    /// so that a file may hold a directory this process does not know.
    unread: bool,
    /// How many times in a row the store has failed,
    /// for the growth of the pause.
    failures: u32,
}

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
///  Reads a whole file.
/// Why:
///  `restored` and `retry` hand it to the versions that take the reading as a parameter,
/// so a control can make a read fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readFile = (path: string) => fs.readFileSync(path);
/// ```
fn read_file(path: &Path) -> std::io::Result<Vec<u8>> {
    return std::fs::read(path);
}

/// What a file holds,
/// or why it holds nothing.
fn decode(bytes: &[u8]) -> Result<PathBuf, String> {
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
fn encode(git_dir: &Path) -> String {
    // The crate's one encoder spells the path's bytes (`hexadecimal_of_bytes.rs`).
    let hex: String = crate::hexadecimal_of_bytes::hexadecimal_of_bytes(git_dir.as_os_str().as_bytes());
    return format!("{FORMAT_PREFIX}{hex}\n");
}

/// The file name of the `attempt`th candidate for `git_dir`:
/// a hash of its bytes,
/// and the number of the candidate,
/// which only
/// differs from zero when another directory's file holds the first name.
fn watch_name(git_dir: &Path, attempt: u32) -> String {
    return format!("{}-{attempt}.{WATCH_EXTENSION}", fnv_hex(encode(git_dir).as_str()));
}

/// Methods over the watch.
impl ClaimWatch {
    /// A watch kept in memory only.
    pub fn in_memory() -> Self {
        return Self::default();
    }

    /// What:
    ///  A watch for a service that starts now without having read its notes:
    /// nothing is known about the repositories an earlier service was woken for,
    /// the watch knows where they are kept,
    ///  and the store is read again after a pause.
    /// Why:
    ///   The read of a journal that stopped answering is bounded,
    /// so a service starts without it rather than never starting;
    /// a watch kept in memory only would then write no note for the whole life of that service,
    ///  so a restart after it killed the service would not know where pending after-commit work waits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new ClaimWatch(dir, instance, { unread: true, failures: 1 })
    /// ```
    pub fn unread(directory: &Path, instance: Option<&str>) -> Self {
        return Self {
            store: Some(directory.to_path_buf()),
            instance: instance.map(str::to_string),
            known: Vec::new(),
            unwritten: Vec::new(),
            unread: true,
            failures: 1,
        };
    }

    /// What:
    ///  A watch kept in `directory`,
    /// holding the Git directories the service before this one wrote down for `instance`.
    /// Why:
    ///   See the module words:
    /// what is kept,
    /// what is removed and what is tried again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new ClaimWatch(dir, instance).scan()
    /// ```
    pub fn restored(directory: &Path, instance: Option<&str>) -> Self {
        return Self::restored_reading(directory, instance, &read_file);
    }

    /// `restored` with the reading of the files given,
    /// so that a control can make one fail.
    pub fn restored_reading(directory: &Path, instance: Option<&str>, read: ReadFile<'_>) -> Self {
        let mut watch: Self = Self { store: Some(directory.to_path_buf()), instance: instance.map(str::to_string), ..Self::default() };
        let _found: Vec<PathBuf> = watch.scan(read);
        return watch;
    }

    /// The Git directories to look at,
    /// each once.
    pub fn directories(&self) -> Vec<PathBuf> {
        return self.known.clone();
    }

    /// The directory the store lives in,
    /// which names the watch's own entry in the work loop's schedule.
    pub fn key(&self) -> Option<PathBuf> {
        return self.store.clone();
    }

    /// Whether something failed that a later try can mend:
    /// a file that could not be written,
    /// or a read that could not be made.
    pub fn needs_retry(&self) -> bool {
        return self.store.is_some() && (self.unread || !self.unwritten.is_empty());
    }

    /// How long to wait before the next try:
    /// `first`,
    /// the first pause of a push that is tried again,
    /// doubled for each failure in a row,
    /// never past those pushes' longest.
    pub fn pause(&self, first: Duration) -> Duration {
        return crate::push_retry::pause_before(self.failures.max(1), first);
    }

    /// Whether `git_dir` is a directory of another instance,
    /// which this journal does not hold.
    fn is_foreign(&self, git_dir: &Path) -> bool {
        return of_another_instance(self.instance.as_deref(), git_dir);
    }

    /// Writes `git_dir` down unless this process has done so.
    /// A write that fails is said and kept for `retry`;
    /// the caller goes on
    /// with the work either way,
    /// because a failed note must not stop a backup.
    pub fn remember(&mut self, git_dir: &Path) {
        for known in self.known.iter().chain(self.unwritten.iter()) {
            if known == git_dir {
                return;
            }
        }
        if self.is_foreign(git_dir) {
            crate::service_warn!("{} belongs to another instance and is not kept for a restart", git_dir.display());
            return;
        }
        let Some(directory) = self.store.clone() else {
            self.known.push(git_dir.to_path_buf());
            return;
        };
        match write_down(directory.as_path(), git_dir) {
            Ok(()) => self.known.push(git_dir.to_path_buf()),
            Err(reason) => {
                crate::service_warn!("{} could not be kept for a restart, so it is tried again after a pause: {reason}", git_dir.display());
                self.unwritten.push(git_dir.to_path_buf());
                self.failures = self.failures.saturating_add(1);
            }
        }
    }

    /// What:
    ///  Keep `git_dir` as a directory whose note may be missing,
    /// so that `retry` writes it again after the loop's pause.
    /// Why:
    ///   A write of the note that ran on another thread,
    /// or gave no answer within its ceiling,
    /// wrote the note or did not,
    /// and nothing here can read which:
    /// the directory is tried again either way,
    /// which writes the same note twice at worst.
    /// A watch kept in memory only records it at once,
    /// as `remember` does.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watch.aNoteThatMayBeMissing(gitDir)
    /// ```
    pub fn a_note_that_may_be_missing(&mut self, git_dir: &Path) {
        if self.store.is_none() || self.holds(git_dir) {
            return;
        }
        self.unwritten.push(git_dir.to_path_buf());
        self.failures = self.failures.saturating_add(1);
    }

    /// Whether `git_dir` is already one of the notes this watch holds or owes.
    fn holds(&self, git_dir: &Path) -> bool {
        for kept in self.known.iter().chain(self.unwritten.iter()) {
            if kept == git_dir {
                return true;
            }
        }
        return false;
    }

    /// Tries again what failed:
    /// the files that could not be written,
    /// and the reading of the store.
    /// Gives the directories this made
    /// known,
    /// which the caller looks at.
    pub fn retry(&mut self) -> Vec<PathBuf> {
        return self.retry_reading(&read_file);
    }

    /// `retry` with the reading of the files given.
    pub fn retry_reading(&mut self, read: ReadFile<'_>) -> Vec<PathBuf> {
        let Some(directory) = self.store.clone() else { return Vec::new() };
        for git_dir in std::mem::take(&mut self.unwritten) {
            match write_down(directory.as_path(), git_dir.as_path()) {
                Ok(()) => self.known.push(git_dir),
                Err(reason) => {
                    crate::service_warn!("{} still could not be kept for a restart: {reason}", git_dir.display());
                    self.unwritten.push(git_dir);
                }
            }
        }
        let found: Vec<PathBuf> = if self.unread { self.scan(read) } else { Vec::new() };
        if self.needs_retry() {
            self.failures = self.failures.saturating_add(1);
        } else {
            self.failures = 0;
        }
        return found;
    }

    /// Reads the store:
    /// keeps the directories whose file is whole,
    /// names this instance and has a work directory,
    /// removes the files
    /// that are certainly of no use,
    /// and notes a failure to read as `unread`.
    /// Gives the directories that were not known before.
    fn scan(&mut self, read: ReadFile<'_>) -> Vec<PathBuf> {
        let Some(directory) = self.store.clone() else { return Vec::new() };
        self.unread = false;
        let listing = match std::fs::read_dir(directory.as_path()) {
            Ok(listing) => listing,
            // A journal that never held a note.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
            Err(error) => {
                crate::service_warn!("the notes of the repositories to look at in {} could not be listed, so they are listed again after a pause: {error}", directory.display());
                self.unread = true;
                return Vec::new();
            }
        };
        let mut found: Vec<PathBuf> = Vec::new();
        for listed in listing {
            let path: PathBuf = match listed {
                Ok(entry) => entry.path(),
                Err(error) => {
                    crate::service_warn!("an entry of {} could not be listed, so the notes are listed again after a pause: {error}", directory.display());
                    self.unread = true;
                    continue;
                }
            };
            if path.extension().and_then(std::ffi::OsStr::to_str) != Some(WATCH_EXTENSION) {
                continue;
            }
            let git_dir: PathBuf = match read(path.as_path()) {
                Ok(bytes) => match decode(bytes.as_slice()) {
                    Ok(git_dir) => git_dir,
                    Err(reason) => {
                        crate::service_warn!("the note {} holds no repository and is removed: {reason}", path.display());
                        let _removed = std::fs::remove_file(path.as_path());
                        continue;
                    }
                },
                // Removed between the listing and the read.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    crate::service_warn!("the note {} could not be read, so it is read again after a pause: {error}", path.display());
                    self.unread = true;
                    continue;
                }
            };
            if self.is_foreign(git_dir.as_path()) {
                crate::service_warn!("the note {} names {}, which is not this instance's repository, and is removed", path.display(), git_dir.display());
                let _removed = std::fs::remove_file(path.as_path());
                continue;
            }
            match std::fs::symlink_metadata(path_for_reading(git_dir.as_path(), WORK_DIRECTORY)) {
                Ok(metadata) if metadata.is_dir() => {
                    if !self.known.contains(&git_dir) {
                        self.known.push(git_dir.clone());
                        found.push(git_dir);
                    }
                }
                Ok(_) => {
                    crate::service_warn!("the work directory of {} is not a directory, so its note {} is removed", git_dir.display(), path.display());
                    let _removed = std::fs::remove_file(path.as_path());
                }
                Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) => {
                    let _removed = std::fs::remove_file(path.as_path());
                }
                Err(error) => {
                    crate::service_warn!("the work directory of {} could not be looked at, so its note is kept and read again after a pause: {error}", git_dir.display());
                    self.unread = true;
                }
            }
        }
        return found;
    }
}

/// Whether `git_dir` belongs to an instance other than `instance`.
/// Why:
///   A journal belongs to one instance,
/// and a note that names another repository is removed by `scan` and refused by `remember`;
/// the landing thread's `note_for_the_next_service` asks the same question,
/// so a note it writes cannot be one the next service throws away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ofAnotherInstance = (instance, gitDir) => instance !== undefined && instanceOf(gitDir) !== instance;
/// ```
fn of_another_instance(instance: Option<&str>, git_dir: &Path) -> bool {
    return match instance {
        Some(mine) => instance_of(git_dir) != mine,
        None => false,
    };
}

/// What:
///  Writes `git_dir` down for the next service of `instance`,
/// from a thread that does not own the watch.
/// Why:
///   The landing thread knows the repository before any client wakes anything,
/// and a wrapper killed between promoting a landed commit's push and waking the service leaves a `pending` record that
///  nothing would drain.
/// The promise that a kill is followed by the work being run with no commit needed rests on the journal naming the
///  repository,
/// so the note is written where the landing already records what it owes,
/// before the client is answered.
///   It is the same exclusive publish the work loop's `remember` uses,
/// so the two writers cannot disagree:
/// a note that is already there and holds this directory is left as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noteForTheNextService = (store, instance, gitDir) => { if (!ofAnotherInstance(instance, gitDir)) writeDown(store, gitDir); };
/// ```
pub fn note_for_the_next_service(store: &Path, instance: Option<&str>, git_dir: &Path) {
    if of_another_instance(instance, git_dir) {
        return;
    }
    if let Err(reason) = write_down(store, git_dir) {
        crate::service_warn!("{} could not be kept for a restart, so a service that starts may not look at it: {reason}", git_dir.display());
    }
}

/// Writes the file of `git_dir` under the first name that is free or already holds this directory.
/// A file that holds no whole path is
/// replaced;
/// one that holds another directory's path is left,
/// and the next name is tried.
fn write_down(directory: &Path, git_dir: &Path) -> Result<(), String> {
    if let Err(error) = create_private_directory_all(directory) {
        return Err(format!("creating the note directory failed: {error}"));
    }
    let text: String = encode(git_dir);
    let mut attempt: u32 = 0;
    loop {
        let path: PathBuf = directory.join(watch_name(git_dir, attempt));
        match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => match std::fs::read(path.as_path()) {
                Ok(held) if held == text.as_bytes() => return Ok(()),
                Ok(held) if decode(held.as_slice()).is_ok() => attempt = attempt.saturating_add(1),
                Ok(_) => {
                    if let Err(removal) = std::fs::remove_file(path.as_path()) {
                        return Err(format!("removing the damaged note {} failed: {removal}", path.display()));
                    }
                }
                Err(read_error) => return Err(format!("reading the note {} failed: {read_error}", path.display())),
            },
            Err(error) => return Err(format!("writing the note failed: {error}")),
        }
    }
}

/// Watch controls stay out of the release executable.
#[cfg(test)]
#[path = "service_claim_watch_tests.rs"]
mod tests;
