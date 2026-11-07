//! What:
//! The write of one containment record to the group log,
//! forced to disk before the program it describes can start.
//! Why:
//! Split out of `bounded_run.rs`,
//! which runs a child process within a time bound,
//! to keep that file under the code-line limit.
//! The items are re-exported from `bounded_run`,
//! so their paths there are unchanged.

/// The trace of a computed text exists in a test build only.
#[cfg(test)]
use crate::bounded_run::trace_text;
/// The trace of the record write.
use crate::bounded_run::trace;

/// Imports `std::io` for this file.
use std::io;


/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;


/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path};








/// Imports `rustix::fs::{FileType, Mode, OFlags, fstat, fsync, open}` for this file.
use rustix::fs::{FileType, Mode, OFlags, fstat, fsync, open};


/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;



/// Imports the control group pieces for this file.
use crate::control_group::{Record};

/// What:
/// `fn append_record(log: &Path, record: &Record) -> io::Result<()>` appends one line to the journal's `groups` file and forces it to
/// disk.
/// Why:
/// The line must be durable before the child it describes starts.
/// The file is opened without waiting and without following a link,
/// and must be a regular file,
/// so a record path replaced by a pipe fails at once instead of blocking the run (6a R4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function appendRecord(log: string, record: Record): void
/// ```
pub(crate) fn append_record(log: &Path, record: &Record) -> io::Result<()> {
    return append_record_with(log, record, true);
}

/// What:
/// `fn append_record_with(log: &Path, record: &Record, sync: bool) -> io::Result<()>` appends one line to the journal's `groups` file
/// and forces it to disk when `sync` is set.
/// Why:
/// The launcher record is written without a sync (round 11):
/// it is read only by a recovery after the wrapper was killed (the page
/// cache keeps it),
/// or after a power loss,
/// where a lost or torn line is read as no proof and the boot rule decides.
/// The line is one `write`,
/// so a process kill leaves it whole or absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function appendRecordWith(log: string, record: Record, sync: boolean): void
/// ```
pub(crate) fn append_record_with(log: &Path, record: &Record, sync: bool) -> io::Result<()> {
    let file: OwnedFd = open(log, OFlags::WRONLY | OFlags::APPEND | OFlags::CLOEXEC | OFlags::NONBLOCK | OFlags::NOFOLLOW, Mode::empty()).map_err(io::Error::from)?;
    let kind: FileType = FileType::from_raw_mode(fstat(&file).map_err(io::Error::from)?.st_mode);
    if kind != FileType::RegularFile {
        return Err(io::Error::from(Errno::INVAL));
    }
    let line: String = format!("{record}\n");
    let kind_name: &str = match record {
        Record::Scope { .. } => "scope",
        Record::Settled { .. } => "settled",
        Record::Launcher { .. } => "launcher",
        Record::ProcessGroup(_) => "pgroup",
        Record::ProcessGroupStarting => "pgroup-starting",
    };
    trace(format_args!("attempt {kind_name}"));
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::RecordWrite) {
        return Err(io::Error::other("injected failure to write the containment record"));
    }
    // A test cuts the write short (only the first half of the line reaches the file), as a full disk would partway through the line.
    #[cfg(test)]
    let to_write: &[u8] = if crate::failpoints::fires(crate::failpoints::Point::RecordShortWrite) { &line.as_bytes()[..line.len() / 2] } else { line.as_bytes() };
    #[cfg(not(test))]
    let to_write: &[u8] = line.as_bytes();
    let written: usize = rustix::io::write(&file, to_write).map_err(io::Error::from)?;
    if written != line.len() {
        return Err(io::Error::new(io::ErrorKind::WriteZero, "the containment record was cut short"));
    }
    trace(format_args!("written {kind_name}"));
    if !sync {
        return Ok(());
    }
    // The record is durable before the program it describes can start: a sync that fails stops the start.
    // The text reads the log file, so this statement exists in a test build only.
    #[cfg(test)]
    trace_text(format!("sync {kind_name} whole={}", std::fs::read_to_string(log).is_ok_and(|text| { return text.ends_with(&line); })));
    // A test replaces the result of the sync, so the failure takes the path of a failure of the kernel (round 21, sixth pass, V3).
    #[cfg(test)]
    let synced: io::Result<()> = if crate::failpoints::fires(crate::failpoints::Point::RecordSync) { Err(io::Error::other("injected failure to make the containment record durable")) } else { fsync(&file).map_err(io::Error::from) };
    #[cfg(not(test))]
    let synced: io::Result<()> = fsync(&file).map_err(io::Error::from);
    synced?;
    trace(format_args!("synced {kind_name}"));
    return Ok(());
}
