//! What:
//! Where each long pass of `cargo-shadow lean run` stopped,
//! kept in this tool's own state directory,
//! so the next run continues after that point instead of starting at the front again.
//! Why:
//! One deadline covers the whole run,
//! and the idle-trees pass alone took longer than it on every run measured (2026-11-09,
//! 2026-11-10),
//! so a pass that always starts at the front reaches only the same first trees and the same first profile
//! directories and starves everything after them
//! (`doc/handover/cargo-shadow-lean-build-folders.md`,
//! section "Open work",
//! and issue 6282).
//! A record that is missing,
//! unreadable or of another shape means "start at the front":
//! a lost position costs one pass over the front of the list,
//! never a failed run.
//! The state directory is `$XDG_STATE_HOME/cargo-shadow` or `$HOME/.local/state/cargo-shadow`,
//! and the file is written by an atomic rename of a name the run made for itself,
//! so a run that is killed leaves the position it last recorded and a staging file this version never opens.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function readProgress(directory: string): Progress
//! export function resumeIndex(keys: string[], after?: string): number
//! ```

/// Files,
/// paths,
/// the descriptor the record files are opened on,
/// and the count that makes each staging name this run's own.
use std::{fs, io::{self, Write}, os::fd::OwnedFd, path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}};

/// The bounded read that waits on nothing and follows no link.
use crate::lean_text::read_unlinked_small_text;

/// Imports `rustix::fs::{Mode, OFlags, open}` for this file:
/// both record files are opened with the flags that make a link and a pipe answer at once.
use rustix::fs::{Mode, OFlags, open};

/// The directory under the state home that holds this tool's own records.
pub const STATE_DIRECTORY_NAME: &str = "cargo-shadow";

/// The file a run records where each pass stopped in.
pub const PROGRESS_FILE_NAME: &str = "lean-progress";

/// The staging name versions before this one wrote the record into before one rename replaced the record with it:
/// the record's name and `.new`.
/// A run of one of those that was killed leaves such a file behind,
/// and this version writes the record into a name of its own and neither opens nor removes this one.
pub const RETIRED_STAGING_FILE_NAME: &str = "lean-progress.new";

/// What a staging name a run creates for itself starts with:
/// the record's own name and a dot.
const STAGING_PREFIX: &str = "lean-progress.";

/// What a staging name a run creates for itself ends with.
const STAGING_SUFFIX: &str = ".new";

/// How many staging names a save tries before it reports that it could not write the record.
/// Why:
/// A name this run makes is taken only by a file a killed run of this same process left at it,
/// which needs that process id and that count together;
/// one such leftover fails one try,
/// a killed run leaves at most a handful,
/// and eight is more than that and still a bounded number of opens.
const STAGING_TRIES: u32 = 8;

/// The count that makes each staging name this process creates its own.
static STAGING_COUNT: AtomicU64 = AtomicU64::new(0);

/// The first line of the record,
/// so a file of another shape is not read as this one.
pub const HEADER: &str = "cargo-shadow lean progress 1";

/// The tag of the line that names the last tree the idle-trees pass judged.
const TREES_TAG: &str = "trees";

/// The tag of the line that names the last profile directory the units pass judged.
const UNITS_TAG: &str = "units";

/// What:
/// `pub struct Progress` is where the last run stopped in each long pass.
/// Why:
/// The two passes are the ones a deadline cuts,
/// and each needs its own position.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Progress {
    /// The last tree the idle-trees pass judged,
    /// when that pass did not finish its list.
    trees: Option<PathBuf>,
    /// The last profile directory the units pass judged,
    /// when that pass did not finish its list.
    units: Option<PathBuf>,
}

/// What:
/// `pub fn state_directory() -> Option<PathBuf>` is this tool's own state directory,
/// from `$XDG_STATE_HOME` or `$HOME/.local/state`,
/// or nothing when neither names one.
/// Why:
/// The position outlives a run and a reboot,
/// and the runtime directory a login owns does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stateDirectory = (): string | undefined => { const base = process.env.XDG_STATE_HOME || (process.env.HOME ? join(process.env.HOME, '.local/state') : undefined); return base ? join(base, 'cargo-shadow') : undefined; };
/// ```
pub fn state_directory() -> Option<PathBuf> {
    if let Some(chosen) = std::env::var_os("XDG_STATE_HOME").filter(is_named) {
        return Some(PathBuf::from(chosen).join(STATE_DIRECTORY_NAME));
    }
    let home: PathBuf = std::env::var_os("HOME").filter(is_named).map(PathBuf::from)?;
    return Some(home.join(".local").join("state").join(STATE_DIRECTORY_NAME));
}

/// What:
/// `fn is_named(value: &std::ffi::OsString) -> bool` says whether an environment value names something.
/// Why:
/// An empty `XDG_STATE_HOME` means the variable is unset rather than that the state belongs at the root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNamed = (value: string): boolean => value !== '';
/// ```
fn is_named(value: &std::ffi::OsString) -> bool {
    return !value.is_empty();
}

/// What:
/// `fn escape(text: &str) -> String` is one path as one line,
/// and `fn unescape(text: &str) -> String` is that line back.
/// Why:
/// A path may hold a newline,
/// and the record is one line per position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const escape = (text: string): string => text.replaceAll('\\', '\\\\').replaceAll('\n', '\\n');
/// ```
fn escape(text: &str) -> String {
    return text.replace('\\', "\\\\").replace('\n', "\\n");
}

/// What:
/// `fn unescape(text: &str) -> String` reverses `escape`.
/// Why:
/// Only the two sequences `escape` writes are undone;
/// anything else stays as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unescape = (text: string): string => text.replace(/\\(\\|n)/g, (_all, one: string) => (one === 'n' ? '\n' : '\\'));
/// ```
fn unescape(text: &str) -> String {
    let mut out: String = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match characters.next() {
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    return out;
}

/// What:
/// `pub fn resume_index(keys: &[PathBuf], after: Option<&Path>) -> usize` is the first index of a sorted list whose key is
/// after the recorded one,
/// and zero when there is no record.
/// Why:
/// The list changes between runs,
/// so the position is a key and not a count:
/// a key that is gone still orders the list,
/// and a key past the end means the next run starts the pass again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const resumeIndex = (keys: string[], after?: string): number => after === undefined ? 0 : keys.findIndex((key) => key > after) === -1 ? keys.length : keys.findIndex((key) => key > after);
/// ```
pub fn resume_index(keys: &[PathBuf], after: Option<&Path>) -> usize {
    let Some(recorded) = after else { return 0 };
    let mut start: usize = 0;
    while start < keys.len() && keys[start].as_path() <= recorded {
        start += 1;
    }
    return start;
}

/// What a read of the record found.
struct ReadRecord {
    /// What the record says,
    /// when it could be read.
    progress: Progress,
    /// Why it could not,
    /// when it could not.
    failure: Option<String>,
}

/// The mode of the record files:
/// this user alone reads and writes them,
/// because they name this user's own directories.
const RECORD_MODE: u32 = 0o600;

/// The most bytes of the record that are read.
/// Why:
/// A record is a header and two escaped paths,
/// a path on Linux is at most 4096 bytes (`PATH_MAX`),
/// and escaping one at most doubles it,
/// so a record is under 16 KiB and this limit is four times the largest one this version writes.
/// A file longer than it is not a record,
/// and a device at the record's name never reaches its end,
/// so reading stops at the limit rather than at an end that may never come.
const RECORD_READ_LIMIT: usize = 64 * 1024;

/// What:
/// `fn read_record(path: &Path) -> ReadRecord` reads the positions,
/// or says why it could not.
/// Why:
/// A missing file is the ordinary first run and stays silent;
/// a file of another shape or one that cannot be read is said once,
/// and the run starts at the front.
/// The record is read as a regular file and never through a link:
/// its name is fixed inside this tool's own state directory,
/// so a link at it names a file this run never wrote,
/// and a pipe or a device at it is not a record.
/// The read waits on nothing and stops at the length no record exceeds,
/// so a record left behind cannot hold the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readRecord(path: string): { progress: Progress; failure?: string }
/// ```
fn read_record(path: &Path) -> ReadRecord {
    let text: String = match read_unlinked_small_text(path, RECORD_READ_LIMIT) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return ReadRecord { progress: Progress::default(), failure: None },
        Err(error) => return ReadRecord { progress: Progress::default(), failure: Some(format!("{}: {error}", path.display())) },
    };
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return ReadRecord { progress: Progress::default(), failure: Some(format!("{}: not a record this version reads", path.display())) };
    }
    let mut progress: Progress = Progress::default();
    for line in lines {
        let Some((tag, value)) = line.split_once(' ') else { continue };
        let found: PathBuf = PathBuf::from(unescape(value));
        if tag == TREES_TAG {
            progress.trees = Some(found);
        } else if tag == UNITS_TAG {
            progress.units = Some(found);
        }
    }
    return ReadRecord { progress, failure: None };
}

/// What:
/// `pub fn read(directory: &Path) -> (Progress, Vec<String>)` is the recorded position,
/// with one line for each thing about it that could not be used.
/// Why:
/// The caller says those lines and carries on:
/// a lost record means the pass starts at the front,
/// never that the run fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function read(directory: string): [Progress, string[]]
/// ```
pub fn read(directory: &Path) -> (Progress, Vec<String>) {
    let record = read_record(&directory.join(PROGRESS_FILE_NAME));
    let mut notes: Vec<String> = Vec::new();
    if let Some(failure) = record.failure {
        notes.push(format!("lean progress: {failure}, so every pass starts at the front"));
    }
    return (record.progress, notes);
}

/// What the two long passes are,
/// for the record of where each stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LongPass {
    /// The idle-trees pass.
    Trees,
    /// The units pass.
    Units,
}

/// What:
/// `fn staging_name(directory: &Path) -> PathBuf` is a name under the state directory that this run has not used before.
/// Why:
/// A name this run made up cannot be an entry that was already there,
/// so the open that creates it never writes into a file that existed before the run,
/// which one check followed by a separate open cannot promise:
/// between the two,
/// another program can put a second name for one of the user's files at the name that was checked,
/// and the open would then truncate the file through that other name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stagingName = (directory: string): string => join(directory, `lean-progress.${process.pid}-${count++}.new`);
/// ```
fn staging_name(directory: &Path) -> PathBuf {
    let count: u64 = STAGING_COUNT.fetch_add(1, Ordering::Relaxed);
    return directory.join(format!("{STAGING_PREFIX}{}-{count}{STAGING_SUFFIX}", std::process::id()));
}

/// What:
/// `fn write_staging(staging: &Path, text: &str) -> std::io::Result<()>` creates the staging file with the record and leaves it
/// there for the rename.
/// Why:
/// The open creates the file and cannot open one that is already there,
/// so nothing this run writes can reach the bytes of a file another name holds:
/// the name is new,
/// and a failure to create it is reported rather than written through.
/// A name that cannot be created is reported and the run carries on without a position:
/// a lost position costs one pass from the front,
/// never the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeStaging = (staging: string, text: string): void => writeFileSync(staging, text, { flag: 'wx', mode: 0o600 });
/// ```
fn write_staging(staging: &Path, text: &str) -> io::Result<()> {
    let flags: OFlags = OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC;
    let opened: OwnedFd = open(staging, flags, Mode::from_raw_mode(RECORD_MODE)).map_err(io::Error::from)?;
    let mut file: fs::File = fs::File::from(opened);
    file.write_all(text.as_bytes())?;
    return Ok(());
}

/// What:
/// `pub struct Progress`'s methods.
impl Progress {
    /// Where the given pass continues after,
    /// when its last run did not finish.
    pub fn after(&self, pass: LongPass) -> Option<&Path> {
        return match pass {
            LongPass::Trees => self.trees.as_deref(),
            LongPass::Units => self.units.as_deref(),
        };
    }

    /// Records where the given pass stopped,
    /// or that it finished its list and the next run starts at the front.
    pub fn set(&mut self, pass: LongPass, after: Option<PathBuf>) {
        match pass {
            LongPass::Trees => self.trees = after,
            LongPass::Units => self.units = after,
        }
    }

    /// What:
    /// `pub fn save(&self, directory: &Path) -> Result<(), String>` writes the record,
    /// replacing what was there in one rename.
    /// Why:
    /// A run killed while writing leaves either the old record or the new one,
    /// never half of either,
    /// and a key that cannot be written exactly is left out rather than written wrong.
    /// The record is written into a name this run made for itself and creates,
    /// so no link and no other name of a file is written through,
    /// and no open waits on a pipe:
    /// a save cannot put this text into a file of the user's or hold the run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const save = (directory: string, progress: Progress): void  // throws
    /// ```
    pub fn save(&self, directory: &Path) -> Result<(), String> {
        if let Err(error) = fs::create_dir_all(directory) {
            return Err(format!("{}: {error}", directory.display()));
        }
        let mut text: String = format!("{HEADER}\n");
        // A path that is not valid UTF-8 cannot be written exactly, and a lossy copy would name a different place,
        // so that key is left out and the next run starts that pass at the front.
        if let Some(trees) = self.trees.as_deref().and_then(Path::to_str) {
            text.push_str(&format!("{TREES_TAG} {}\n", escape(trees)));
        }
        if let Some(units) = self.units.as_deref().and_then(Path::to_str) {
            text.push_str(&format!("{UNITS_TAG} {}\n", escape(units)));
        }
        let path: PathBuf = directory.join(PROGRESS_FILE_NAME);
        for _try in 0..STAGING_TRIES {
            let staging: PathBuf = staging_name(directory);
            match write_staging(&staging, &text) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("{}: {error}", staging.display())),
            }
            return match fs::rename(&staging, &path) {
                Ok(()) => Ok(()),
                Err(error) => Err(format!("{}: {error}", path.display())),
            };
        }
        return Err(format!("{}: no name this run made was free to stage the record under", directory.display()));
    }
}
