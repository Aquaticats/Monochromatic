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
//!  A record's two file names carry the very file they describe,
//!  the lock's path **and** its device and inode,
//!  and never the path alone
//!  (a review of `f8f5cba090`,
//! finding 1):
//!  two commands of one repository take the same lock path one after the other,
//!  so a name built from the path alone is shared by records of different locks,
//!  and a remover that decided on that name would take away a record a concurrent command had just written for
//!  its own lock.
//!  With the file in the name,
//!  a remover takes away exactly the records of the lock instance it is about,
//!  and no run of another command can be holding that instance:
//!  the pin keeps its inode out of every later file at that path,
//!  so a later lock there is a different file with a different name.
//!  Nothing here decides on a name two commands can share,
//!  so nothing here needs a check between a look and a removal.
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
///  the digest of the path's bytes and of the device and inode of the file standing at that path.
/// Why:
///   A path can hold a newline and a space,
///  so the name is not built from the path itself,
///  and a digest compares whole texts by their bytes,
///  so two different locks never share a name.
///  The file is in the name because the path alone is shared:
///  one command's Git renames its lock over its target and the next command's Git takes the same path
///  microseconds later,
///  so a name of the path alone would be one name for two locks,
///  and the command that finished its own would have no way to take away only its own record
///  (a review of `f8f5cba090`,
/// finding 1).
///  The name is computed from a file the caller has already looked at,
///  never from a fresh look at the path:
///  a fresh look can only say what stands there now,
///  which is the fact a remover must not act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// digestOfBytes(Buffer.concat([Buffer.from(lockPath), Buffer.from([0]), u64le(dev), u64le(ino)])) // '3f9a…'
/// ```
pub fn name_of(lock: &Path, file: (u64, u64)) -> String {
    use std::os::unix::ffi::OsStrExt as _;
    let mut bytes: Vec<u8> = lock.as_os_str().as_bytes().to_vec();
    // The separator and the fixed width of the two numbers keep two different locks off one name.
    bytes.push(0);
    bytes.extend_from_slice(&file.0.to_le_bytes());
    bytes.extend_from_slice(&file.1.to_le_bytes());
    return digest_of_bytes(bytes.as_slice());
}

/// What:
///  One record a command wrote:
///  the lock's path and the identity of the very file it pinned.
/// Why:
///   This is the whole of what taking the record away needs,
///  and it is worked out before the lock can be renamed away or killed,
///  so a remover never has to ask the path what it held.
///  A command holds these for the locks its own Git took,
///  and only for those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Written = { lock: string; file: [device, inode] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Written {
    /// The path the lock was taken at.
    pub lock: PathBuf,
    /// The device and inode of the file that path held when the record was written.
    pub file: (u64, u64),
}

impl Written {
    /// The name the two files of this record carry.
    pub fn name(&self) -> String {
        return name_of(self.lock.as_path(), self.file);
    }
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
    let every_byte_is_a_digit: bool = text.bytes().all(|byte: u8| { return byte.is_ascii_hexdigit(); });
    if text.len() % 2 != 0 || !every_byte_is_a_digit {
        return None;
    }
    let digits: &[u8] = text.as_bytes();
    let mut bytes: Vec<u8> = Vec::with_capacity(digits.len() / 2);
    for at in (0..digits.len()).step_by(2) {
        let pair: &str = std::str::from_utf8(&digits[at..at + 2]).ok()?;
        bytes.push(u8::from_str_radix(pair, 16).ok()?);
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
///  Every file this writes is named for the lock's path **and** the identity of the file the caller found
///  there,
///  so the two files of one removal are never the two of another command's lock.
///  The link is made first and the record published after it,
///  so a reader meets either no record, or a record with its pin.
///  The pin is looked at once more before the record is published:
///  a lock renamed away and made again at the same path between the caller's look and the link leaves a pin
///  for another file,
///  and then nothing is written at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// linkSync(lock, pin); if (statSync(pin).ino !== file.ino) return undefined; renameSync(holderTemp, holder);
/// ```
pub fn write(common_dir: &Path, lock: &Path, file: (u64, u64), holder: &(i64, String)) -> Option<Written> {
    let folder: PathBuf = folder_of(common_dir);
    std::fs::create_dir_all(folder.as_path()).ok()?;
    let name: String = name_of(lock, file);
    let pin: PathBuf = folder.join(format!("{name}{PIN_SUFFIX}"));
    let record: PathBuf = folder.join(format!("{name}{HOLDER_SUFFIX}"));
    // These two names belong to the very file the caller looked at, and the pin keeps that inode out of every later
    //  file at this path, so no other command writes them. A file of a killed run of this very instance, or a planted
    //  one, goes first.
    let _removed_record = std::fs::remove_file(record.as_path());
    let _removed_pin = std::fs::remove_file(pin.as_path());
    std::fs::hard_link(lock, pin.as_path()).ok()?;
    let linked: LockMetadata = crate::index_lock_evidence::read_lock_metadata(pin.as_path()).ok()??;
    if (linked.device, linked.inode) != file {
        let _removed_record = std::fs::remove_file(record.as_path());
        let _removed_pin = std::fs::remove_file(pin.as_path());
        return None;
    }
    let temporary: PathBuf = folder.join(format!("{name}{HOLDER_SUFFIX}.writing"));
    std::fs::write(temporary.as_path(), text_of(lock, holder).as_bytes()).ok()?;
    std::fs::rename(temporary.as_path(), record.as_path()).ok()?;
    return Some(Written { lock: lock.to_path_buf(), file });
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
    let Ok(Some(standing)) = crate::index_lock_evidence::read_lock_metadata(lock) else {
        return None;
    };
    let name: String = name_of(lock, (standing.device, standing.inode));
    let record: PathBuf = folder.join(format!("{name}{HOLDER_SUFFIX}"));
    let bytes: Vec<u8> = read_plain_file(record.as_path(), BINDING_LIMIT).ok()?;
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    let bound: Binding = parse(text.as_ref())?;
    if bound.lock != lock {
        return None;
    }
    let pin: PathBuf = folder.join(format!("{name}{PIN_SUFFIX}"));
    let pinned: LockMetadata = crate::index_lock_evidence::read_lock_metadata(pin.as_path()).ok()??;
    if pinned.device != standing.device || pinned.inode != standing.inode {
        return None;
    }
    return Some(bound);
}

/// What:
///  Remove the record `written` describes and its pin.
/// Why:
///  A lock that was cleared needs no record,
///  and the pin it leaves holds the inode of a file nothing names.
///  The name is the one the writer worked out from the file it pinned,
///  so this takes away that record and no other:
///  a command finishing its own lock never removes a record another command wrote,
///  whatever the path holds by then
///  (a review of `f8f5cba090`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unlink(recordName(written)); unlink(pinName(written))
/// ```
pub fn remove(common_dir: &Path, written: &Written) {
    remove_named(folder_of(common_dir).as_path(), written.name().as_str());
}

/// What:
///  Remove one record's two files by the name they carry.
/// Why:
///  The sweep decides on a record it has read and names it by its own file name,
///  and the removal of a record is the same two unlinks either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unlink(join(folder, `${name}.holder`)); unlink(join(folder, `${name}.pin`))
/// ```
fn remove_named(folder: &Path, name: &str) {
    let _removed_record = std::fs::remove_file(folder.join(format!("{name}{HOLDER_SUFFIX}")).as_path());
    let _removed_pin = std::fs::remove_file(folder.join(format!("{name}{PIN_SUFFIX}")).as_path());
}

/// What:
///  Whether the pin of `written` still holds the file standing at the path `written` names.
/// Why:
///   A record is kept exactly as long as its own lock stands:
///  a pin that names the file at the path describes a lock that is still there,
///  and one that does not describes a lock its Git finished, rolled back or lost.
///  The question is about this record's name only,
///  so a concurrent command's lock at the same path cannot change the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statSync(pinOf(written)).ino === statSync(written.lock).ino
/// ```
pub fn pin_holds_the_lock(common_dir: &Path, written: &Written) -> bool {
    let folder: PathBuf = folder_of(common_dir);
    let name: String = written.name();
    let pin: PathBuf = folder.join(format!("{name}{PIN_SUFFIX}"));
    let Ok(Some(pinned)) = crate::index_lock_evidence::read_lock_metadata(pin.as_path()) else {
        return false;
    };
    let Ok(Some(standing)) = crate::index_lock_evidence::read_lock_metadata(written.lock.as_path()) else {
        return false;
    };
    return pinned.device == standing.device && pinned.inode == standing.inode;
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
        let name: String = record.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let stem: &str = name.strip_suffix(HOLDER_SUFFIX).unwrap_or(name.as_str());
        let explained: bool = matches!(read_plain_file(record.as_path(), BINDING_LIMIT), Ok(bytes) if {
            let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
            // The removal acts on the very name this record carries, so a record another command wrote after this look
            //  has a name of its own and is never it.
            matches!(parse(text.as_ref()), Some(bound) if pin_still_names(folder.as_path(), stem, &bound))
        });
        if explained {
            kept.push(name);
            continue;
        }
        remove_named(folder.as_path(), stem);
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
///  Whether the pin carrying `name` still holds the file at the path the record `bound` names.
/// Why:
///   The sweep asks it of a record it has not judged,
///  and a lock that was renamed over its target,
///  cleared or replaced leaves a pin that names no file at that path.
///  The name is the record's own,
///  read from the file the sweep opened and never worked out again from the path:
///  a look at the path again would answer about whatever stands there now,
///  which is the fact that makes a remover act on a record it does not own
///  (a review of `f8f5cba090`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statSync(pinOf(name)).ino === statSync(record.lock).ino
/// ```
fn pin_still_names(folder: &Path, name: &str, bound: &Binding) -> bool {
    let pin: PathBuf = folder.join(format!("{name}{PIN_SUFFIX}"));
    let Ok(Some(pinned)) = crate::index_lock_evidence::read_lock_metadata(pin.as_path()) else {
        return false;
    };
    let Ok(Some(standing)) = crate::index_lock_evidence::read_lock_metadata(bound.lock.as_path()) else {
        return false;
    };
    return pinned.device == standing.device && pinned.inode == standing.inode;
}
