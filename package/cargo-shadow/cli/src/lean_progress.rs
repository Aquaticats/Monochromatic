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
//! and the file is written by an atomic rename,
//! so a run that is killed leaves the position it last recorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function readProgress(directory: string): Progress
//! export function resumeIndex(keys: string[], after?: string): number
//! ```

/// Files,
/// paths,
/// the descriptor the two record files are opened on,
/// and the rename that replaces the record.
use std::{fs, io::{self, Read, Write}, os::fd::OwnedFd, path::{Path, PathBuf}};

/// Imports `rustix::fs::{Mode, OFlags, open}` for this file:
/// both record files are opened with the flags that make a link and a pipe answer at once.
use rustix::fs::{Mode, OFlags, open};

/// The directory under the state home that holds this tool's own records.
pub const STATE_DIRECTORY_NAME: &str = "cargo-shadow";

/// The file a run records where each pass stopped in.
pub const PROGRESS_FILE_NAME: &str = "lean-progress";

/// The file a run writes the record into before one rename replaces the record with it:
/// the record's name and `.new`.
pub const STAGING_FILE_NAME: &str = "lean-progress.new";

/// The first line of the record,
/// so a file of another shape is not read as this one.
const HEADER: &str = "cargo-shadow lean progress 1";

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

/// What:
/// `fn read_text(path: &Path) -> std::io::Result<String>` is the whole file,
/// opened so that a pipe at the path answers at once instead of waiting for a writer.
/// Why:
/// The record's name is fixed and lives in this tool's own state directory,
/// and a pipe left at it would hold the run until someone wrote to it,
/// which nothing here promises to do:
/// a run waits for nothing indefinitely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readText = (path: string): string => readFileSync(path, { encoding: 'utf8' });
/// ```
fn read_text(path: &Path) -> io::Result<String> {
    let opened: OwnedFd = open(path, OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NONBLOCK, Mode::empty()).map_err(io::Error::from)?;
    let mut file: fs::File = fs::File::from(opened);
    let mut text: String = String::new();
    file.read_to_string(&mut text)?;
    return Ok(text);
}

/// What:
/// `fn read_record(path: &Path) -> ReadRecord` reads the positions,
/// or says why it could not.
/// Why:
/// A missing file is the ordinary first run and stays silent;
/// a file of another shape or one that cannot be read is said once,
/// and the run starts at the front.
/// A pipe is one of those shapes:
/// it is read for what it already holds and never waited on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readRecord(path: string): { progress: Progress; failure?: string }
/// ```
fn read_record(path: &Path) -> ReadRecord {
    let text: String = match read_text(path) {
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
/// `fn write_staging(staging: &Path, text: &str) -> std::io::Result<()>` writes the record into the staging file and leaves it
/// there for the rename.
/// Why:
/// The staging name is fixed and lives in this tool's own state directory,
/// so whatever else is at that name is not a file to write through:
/// following a link would put this text into the file of the user's that it names,
/// and opening a pipe would hold the run until a reader arrived.
/// The open therefore never follows a link and never waits on a pipe,
/// a link or a pipe already at the name is removed first so this run can create its own file,
/// and the staging file a killed run left behind is a regular file and is simply replaced.
/// A name that is still not writable after that is reported and the run carries on without a position:
/// a lost position costs one pass from the front, never the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeStaging = (staging: string, text: string): void => { rmSync(staging, { force: true }); writeFileSync(staging, text, { flag: 'wx', mode: 0o600 }); };
/// ```
fn write_staging(staging: &Path, text: &str) -> io::Result<()> {
    match fs::symlink_metadata(staging) {
        Ok(metadata) if metadata.is_file() => {}
        // A directory at the name is not this run's to remove, so the open below refuses it and says why.
        Ok(_) => fs::remove_file(staging)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let flags: OFlags = OFlags::WRONLY | OFlags::CREATE | OFlags::TRUNC | OFlags::CLOEXEC | OFlags::NONBLOCK | OFlags::NOFOLLOW;
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
    /// Neither file is written through a link or waited on as a pipe,
    /// so a save cannot put this text into a file of the user's or hold the run.
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
        let staging: PathBuf = directory.join(STAGING_FILE_NAME);
        if let Err(error) = write_staging(&staging, &text) {
            return Err(format!("{}: {error}", staging.display()));
        }
        return match fs::rename(&staging, &path) {
            Ok(()) => Ok(()),
            Err(error) => Err(format!("{}: {error}", path.display())),
        };
    }
}
