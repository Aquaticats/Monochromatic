//! What:
//!  The record a command writes about a lock its own Git took,
//!  and the read that lets a later command act on it.
//! Why:
//!  A lock's PID file names a process number,
//!  and the number proves nothing about *this* lock:
//!  Git writes the PID file after the lock within microseconds,
//!  a file system of coarse birth times gives the two files one stamp
//!  (`lock_birth_times.rs`),
//!  and a PID file an earlier holder left in front of a later lock names an earlier process.
//!  A command that waits for its own Git knows what no reader on disk can:
//!  it watched its Git create the lock and write the PID file beside it,
//!  in that order, and nothing removed the lock in between
//!  (`lock_watch.rs`).
//!  So it writes what it saw here,
//!  and the one fact that makes the record outlive the file it describes is the pin:
//!  a hard link to the very inode the lock had when its Git took it.
//!  While that link exists the inode cannot be given to another file,
//!  so a lock at the same path later is a different inode
//!  and the record cannot match it.
//!  The record names the Git that took the lock by the birth identity the command read of its
//!  own child,
//!  so a lock a Git this repository's commands never started is never cleared on it.
//!  The record is litter as soon as the lock it pinned is gone,
//!  and the sweep a command runs removes it then (`sweep`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeBinding({ commonDir, lock, holder, pidFile }); const bound = bindingOf(commonDir, lock); sweepBindings(commonDir);
//! ```

/// The folder the records of a repository's commands live in,
/// whose `pinned` folder holds these.
use crate::lock_holder_records::folder_of as records_folder_of;
/// The identity a file had when the lock's own Git took it.
use crate::index_lock_evidence::LockMetadata;
/// Reading a small file without following a link and without waiting on a pipe.
use crate::plain_file::read_plain_file;
/// The one digest the records and the service's instance names are built from.
use crate::service_instance::digest_of_bytes;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The folder below a repository's record folder that holds these records.
pub const PINNED_FOLDER: &str = "pinned";

/// The first line of every record.
pub const BINDING_HEADER: &str = "monochromatic-git-policy-cli lock-binding v1";

/// What a lock's path ends with,
/// which the record's own file is named for.
const PIN_SUFFIX: &str = ".pin";

/// What the record beside the pin is named for.
const HOLDER_SUFFIX: &str = ".holder";

/// The longest record one read takes:
/// a path written as its bytes in hexadecimal and three short lines.
const BINDING_LIMIT: u64 = 64 * 1024;

/// What:
///  One lock a command's own Git took,
///  as the command wrote it down.
/// Why:
///   The pin is the lock's identity and lives in the folder beside this value;
///  and the holder is the Git that took the lock.
///  The PID file is not part of it:
///  an inode number comes back to the same path as soon as its file is removed
///  (measured on this volume by the controls of this module),
///  so the pin of the lock is the only identity a record may rest on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Binding = { lock: string; pid: number; identity: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    /// The lock file this record was written for.
    pub lock: PathBuf,
    /// The Git that took the lock,
    ///  by the number its PID file carries.
    pub pid: i64,
    /// That Git's birth identity,
    ///  which the command read of its own child.
    pub identity: String,
}

/// What:
///  The folder one repository's records of taken locks live in.
/// Why:
///   A record belongs to the repository whose commands wrote it,
///  and every worktree of a repository shares one common Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'cli-git-lock-holders', 'pinned')
/// ```
pub fn folder_of(common_dir: &Path) -> PathBuf {
    return records_folder_of(common_dir).join(PINNED_FOLDER);
}

/// What:
///  The name one lock's record and its pin carry:
///  the digest of the path's own bytes.
/// Why:
///   A path can hold a newline and a space,
///  so the name is not built from the path itself;
///  and a digest compares whole texts by their bytes,
///  so two different paths never share a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// digestOfBytes(Buffer.from(lockPath)) // '3f9a…'
/// ```
pub(crate) fn name_of(lock: &Path) -> String {
    use std::os::unix::ffi::OsStrExt;
    return digest_of_bytes(lock.as_os_str().as_bytes());
}

/// What:
///  One byte of a path as two hexadecimal digits,
///  and back,
///  so a path with a newline or a space still fits one line of the record.
/// Why:
///   A record is read as lines,
///  and the pass may meet a path of any bytes a file system holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hexOf(Buffer.from(path)); bytesOfHex('2f686f6d65')
/// ```
fn hex_of(bytes: &[u8]) -> String {
    let mut text: String = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push_str(format!("{byte:02x}").as_str());
    }
    return text;
}

/// What:
///  The bytes a hexadecimal text stands for,
///  or nothing when it is not one.
/// Why:
///   A record a person edited decides nothing:
///  a text of the wrong length or with a foreign digit names no path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from('2f686f6d65', 'hex')
/// ```
fn bytes_of_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || !text.bytes().all(|byte: u8| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() / 2);
    for pair in text.as_bytes().chunks_exact(2) {
        let digits: &str = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(digits, 16).ok()?);
    }
    return Some(bytes);
}

/// What:
///  The text of one record:
///  header,
///  the lock's path,
///  and the Git that took it.
/// Why:
///   One spelling with the writer and the controls,
///  so a control plants a record through the same function that writes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bindingText({ lock, holder: [42, identity] })
/// ```
pub fn text_of(lock: &Path, holder: &(i64, String)) -> String {
    use std::os::unix::ffi::OsStrExt;
    return format!("{BINDING_HEADER}\nlock {}\nholder {} {}\n", hex_of(lock.as_os_str().as_bytes()), holder.0, holder.1);
}

/// What:
///  The record a text holds,
///  or nothing when it is not one.
/// Why:
///   Every part is required and nothing is guessed:
///  a record missing its holder,
///  naming no path,
///  or cut short decides nothing and the lock stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseBinding(text) // Binding | undefined
/// ```
pub fn parse(text: &str) -> Option<Binding> {
    let mut lines = text.lines();
    if lines.next()? != BINDING_HEADER {
        return None;
    }
    let path_line: &str = lines.next()?.strip_prefix("lock ")?;
    let holder_line: &str = lines.next()?.strip_prefix("holder ")?;
    if lines.next().is_some() {
        return None;
    }
    let (pid, identity) = holder_line.split_once(' ')?;
    if identity.is_empty() || identity.contains(' ') {
        return None;
    }
    use std::os::unix::ffi::OsStringExt as _;
    let lock: PathBuf = PathBuf::from(std::ffi::OsString::from_vec(bytes_of_hex(path_line)?));
    return Some(Binding { lock, pid: pid.parse::<i64>().ok()?, identity: identity.to_owned() });
}

/// What:
///  Keep the file at `lock` from having its inode given to another file,
///  by holding a second name for it in the records folder,
///  and write the record that says whose it was.
/// Why:
///   A record of an inode number alone is no proof:
///  a file system gives a freed inode to a new file,
///  and two files of two subvolumes of one mount share the number.
///  The link keeps the inode allocated for as long as the record stands,
///  so a lock at the path later is a different inode.
///  A link cannot be made when the lock is gone or on another file system,
///  and then no record is written:
///  the lock keeps the bounded wait and report it has today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// linkSync(lock, pin); writeFileSync(holderTemp, text); renameSync(holderTemp, holder);
/// ```
pub fn write(common_dir: &Path, lock: &Path, holder: &(i64, String)) -> Option<()> {
    let folder: PathBuf = folder_of(common_dir);
    std::fs::create_dir_all(folder.as_path()).ok()?;
    let name: String = name_of(lock);
    let pin: PathBuf = folder.join(format!("{name}{PIN_SUFFIX}"));
    let record: PathBuf = folder.join(format!("{name}{HOLDER_SUFFIX}"));
    // The record of an earlier lock at this path goes first: a reader between these steps finds no record and keeps the lock.
    let _removed_record = std::fs::remove_file(record.as_path());
    let _removed_pin = std::fs::remove_file(pin.as_path());
    std::fs::hard_link(lock, pin.as_path()).ok()?;
    let temporary: PathBuf = folder.join(format!("{name}{HOLDER_SUFFIX}.writing"));
    std::fs::write(temporary.as_path(), text_of(lock, holder).as_bytes()).ok()?;
    return std::fs::rename(temporary.as_path(), record.as_path()).ok();
}

/// What:
///  The record written for the lock at `lock`,
///  when its pin still holds the very file that stands at that path now.
/// Why:
///   A record only ever describes one file,
///  so a pin that no longer names the file at the path describes a lock that is gone,
///  and the pass reads nothing from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binding = readBinding(commonDir, lock); // undefined unless the pin is the lock
/// ```
pub fn of(common_dir: &Path, lock: &Path) -> Option<Binding> {
    let folder: PathBuf = folder_of(common_dir);
    let name: String = name_of(lock);
    let record: PathBuf = folder.join(format!("{name}{HOLDER_SUFFIX}"));
    let bytes: Vec<u8> = read_plain_file(record.as_path(), BINDING_LIMIT).ok()?;
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    let bound: Binding = parse(text.as_ref())?;
    if bound.lock != lock {
        return None;
    }
    let pin: PathBuf = folder.join(format!("{name}{PIN_SUFFIX}"));
    let pinned: LockMetadata = crate::index_lock_evidence::read_lock_metadata(pin.as_path()).ok()??;
    let standing: LockMetadata = crate::index_lock_evidence::read_lock_metadata(lock).ok()??;
    if pinned.device != standing.device || pinned.inode != standing.inode {
        return None;
    }
    return Some(bound);
}

/// What:
///  Remove the record of the lock at `lock` and its pin.
/// Why:
///  A lock that was cleared needs no record,
/// and the pin it leaves holds the inode of a file nothing names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unlink(record); unlink(pin)
/// ```
pub fn remove(common_dir: &Path, lock: &Path) {
    let folder: PathBuf = folder_of(common_dir);
    let name: String = name_of(lock);
    let _removed_record = std::fs::remove_file(folder.join(format!("{name}{HOLDER_SUFFIX}")).as_path());
    let _removed_pin = std::fs::remove_file(folder.join(format!("{name}{PIN_SUFFIX}")).as_path());
}

/// What:
///  Remove every record whose lock no longer stands,
///  and every pin no record explains.
/// Why:
///   A record is kept exactly as long as the lock it pinned stands,
///  because the pin keeps that inode from being given to another file.
///  The verdict is about the path and the file,
///  so a record of a lock that was cleared, renamed over its target, or removed by a person goes,
///  and a pin whose record a half-written write left behind goes too.
///  Two commands sweeping at once remove the same litter harmlessly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const record of recordsOf(commonDir)) if (!pinnedFileStands(record)) unlink(record, pin)
/// ```
pub fn sweep(common_dir: &Path) {
    let folder: PathBuf = folder_of(common_dir);
    let Ok(entries) = std::fs::read_dir(folder.as_path()) else {
        return;
    };
    let mut records: Vec<PathBuf> = Vec::new();
    let mut pins: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let name: std::ffi::OsString = entry.file_name();
        let text: std::borrow::Cow<'_, str> = name.to_string_lossy();
        if text.ends_with(HOLDER_SUFFIX) {
            records.push(entry.path());
        } else if text.ends_with(PIN_SUFFIX) {
            pins.push(entry.path());
        }
    }
    let mut kept: Vec<String> = Vec::new();
    for record in records {
        let explained: bool = matches!(read_plain_file(record.as_path(), BINDING_LIMIT), Ok(bytes) if {
            let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
            matches!(parse(text.as_ref()), Some(bound) if pin_still_names(&folder, &bound))
        });
        if explained {
            kept.push(record.file_name().unwrap_or_default().to_string_lossy().into_owned());
            continue;
        }
        let _removed = std::fs::remove_file(record.as_path());
    }
    for pin in pins {
        let name: std::borrow::Cow<'_, str> = pin.file_name().unwrap_or_default().to_string_lossy();
        let record_name: String = format!("{}{HOLDER_SUFFIX}", name.trim_end_matches(PIN_SUFFIX));
        if !kept.contains(&record_name) {
            let _removed = std::fs::remove_file(pin.as_path());
        }
    }
}

/// What:
///  Whether the pin of `bound` still holds the file at the path the record names.
/// Why:
///   The sweep asks it of a record it has not judged,
///  and a lock that was renamed over its target,
///  cleared or replaced leaves a pin that names no file at that path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statSync(pin).ino === statSync(record.lock).ino
/// ```
fn pin_still_names(folder: &Path, bound: &Binding) -> bool {
    let name: String = name_of(bound.lock.as_path());
    let pin: PathBuf = folder.join(format!("{name}{PIN_SUFFIX}"));
    let Ok(Some(pinned)) = crate::index_lock_evidence::read_lock_metadata(pin.as_path()) else {
        return false;
    };
    let Ok(Some(standing)) = crate::index_lock_evidence::read_lock_metadata(bound.lock.as_path()) else {
        return false;
    };
    return pinned.device == standing.device && pinned.inode == standing.inode;
}
