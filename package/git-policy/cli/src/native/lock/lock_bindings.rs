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
//!
//!  A record is **one file**:
//!  the pin itself,
//!  named `<path digest>.<device>.<inode>.<pid>.<birth identity>.hold`,
//!  and the path that digest stands for is written once per path in `<path digest>.lock-path`.
//!  Nothing else is written for a record,
//!  so publishing it is one `link` and nothing else,
//!  and a command killed at any instruction leaves either no record or the whole of one:
//!  no sweep, no forget and no write ever meets a half-published record to take away
//!  (a review of `2ac27d2a29`,
//! finding 1,
//! which a preload widening nothing but the window between a pin and a record reproduced).
//!  The name is unique to one take by one Git:
//!  a path's digest names the lock's path,
//!  and the process number and birth identity name the Git that took it,
//!  which no later run can be,
//!  so a removal acts on a name no other command writes.
//!  The path's text is what lets the sweep judge a record it finds:
//!  it is the one file of a path that is written before any of its records and never rewritten,
//!  so it cannot be half written while a record of that path exists.
//!
//!  What a process killed at each step leaves,
//!  and who removes it:
//!
//!  - after the path's text and before any pin:
//!    one file naming the path and no record;
//!    the next write for that path reuses it,
//!    and nothing needs removing,
//!    because a record is a pin and there is none;
//!  - after the pin, which is the whole record:
//!    a record naming the lock,
//!    kept by every command while its pin names the file at the path,
//!    taken away by the command that wrote it when its Git finished the lock
//!    (`lock_watch.rs`,
//! `forget_what_the_child_finished`)
//!    or by the next command's sweep once its lock is gone;
//!  - inside one removal:
//!    one `unlink`,
//!    which the kernel does whole,
//!    so no half-removed record exists;
//!    a pin a command wrote under a name its own run never reached again is litter,
//!    and the sweep removes it once its lock is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeBinding({ commonDir, lock, holder }); const bound = bindingOf(commonDir, lock); sweepBindings(commonDir);
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

/// What a record's own file name ends with:
/// the pin,
/// which is the whole record.
const HOLD_SUFFIX: &str = ".hold";

/// What the file naming one path's text ends with.
const LOCK_PATH_SUFFIX: &str = ".lock-path";

/// The longest text one read of a path's file takes:
/// a path and nothing else.
const LOCK_PATH_LIMIT: u64 = 64 * 1024;

/// The most bytes a record's own file name may take,
/// which keeps a long birth identity from failing the link with a name that is too long.
const MOST_NAME_BYTES: usize = 200;

/// What:
///  The instant the pin of a record has just been made reachable under its own name,
///  which is the whole of the record.
/// Why:
///   A control states the interleaving the publication must survive:
///  another command's sweep runs at this instant and may not take away the pin of a lock that stands
///  (`a_sweep_beside_a_write_leaves_the_record_of_a_standing_lock`).
///  Named here so both halves of that promise reach one point,
/// and inert unless the test variable or a control names it.
pub const A_PIN_IS_REACHABLE: &str = "a-pin-is-reachable";

/// What:
///  One lock a command's own Git took,
///  as the command wrote it down.
/// Why:
///   The pin is the lock's identity and is the whole record;
///  the holder is the Git that took the lock,
///  read from the record's own name.
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
    /// The record's own file:
    ///  the pin that names the lock's inode.
    pub pin: PathBuf,
    /// The Git that took the lock,
    ///  by the number its PID file carries.
    pub pid: i64,
    /// That Git's birth identity,
    ///  which the command read of its own child.
    pub identity: String,
}

/// What:
///  One record a command wrote:
///  the file that pins the lock's inode,
///  and the path it stands for.
/// Why:
///  This is the whole of what taking the record away needs,
///  and it is worked out before the lock can be renamed away or killed,
///  so a remover never has to ask the path what it held.
///  A command holds these for the locks its own Git took,
///  and only for those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Written = { pin: string; lock: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Written {
    /// The record's own file:
    ///  the hard link that pins the lock's inode,
    ///  and the whole record.
    pub pin: PathBuf,
    /// The path the record is about.
    pub lock: PathBuf,
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
///  The digest a lock's path is named by in the records folder.
/// Why:
///   A path can hold a newline and a space,
///  so the name is not built from the path itself,
///  and a digest compares whole texts by their bytes,
///  so two different paths never share a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// digestOfBytes(Buffer.from(lockPath))
/// ```
fn digest_of_path(lock: &Path) -> String {
    use std::os::unix::ffi::OsStrExt as _;
    return digest_of_bytes(lock.as_os_str().as_bytes());
}

/// What:
///  The file that names one path's text,
///  which is what the sweep reads to judge any record of that path.
/// Why:
///   A digest cannot be read back,
///  and a sweep that knew no path could not tell a record whose lock stands from litter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(folder, `${digest}.lock-path`)
/// ```
fn lock_path_file(folder: &Path, digest: &str) -> PathBuf {
    return folder.join(format!("{digest}{LOCK_PATH_SUFFIX}"));
}

/// What:
///  The record's own file name for one take by one Git:
///  the path's digest,
///  the device and inode of the file the take made,
///  the process number and the birth identity.
/// Why:
///   The name is unique to that take:
///  no later run of any command can carry that process number and that birth,
///  and no other take can hold that inode while this pin does,
///  so a removal acts on a name no other command's write produces.
///  A path alone and an inode alone were not enough:
///  one path holds one lock after another,
///  and an inode number comes back to a path once its file is gone
///  (a review of `f8f5cba090`, finding 1;
/// a review of `2ac27d2a29`, finding 1).
///  The file in the name also keeps one take of one Git from sharing a name with its next take:
///  the second take writes its own record beside the first,
///  and nothing has to be unlinked before a record exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${digest}.${dev}.${ino}.${pid}.${identity}.hold`
/// ```
fn pin_name(digest: &str, file: (u64, u64), holder: &(i64, String)) -> Option<String> {
    let name: String = format!("{digest}.{}.{}.{}.{}{HOLD_SUFFIX}", file.0, file.1, holder.0, holder.1);
    if name.len() > MOST_NAME_BYTES || holder.1.contains('/') || holder.1.contains('\0') {
        return None;
    }
    return Some(name);
}

/// What:
///  The process number and the birth identity a record's own name carries,
///  or nothing when the name is not one of this module's.
/// Why:
///   The name is the whole record,
///  so a name that is not this shape names nobody,
///  and the two numbers before the process are checked only for their shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parsePinName('3f9a…') // [pid, identity] | undefined
/// ```
fn holder_of_pin_name(digest: &str, name: &str) -> Option<(i64, String)> {
    let rest: &str = name.strip_prefix(format!("{digest}.").as_str())?.strip_suffix(HOLD_SUFFIX)?;
    let mut fields = rest.splitn(4, '.');
    let _device: u64 = fields.next()?.parse::<u64>().ok()?;
    let _inode: u64 = fields.next()?.parse::<u64>().ok()?;
    let pid: i64 = fields.next()?.parse::<i64>().ok()?;
    let identity: &str = fields.next()?;
    if identity.is_empty() {
        return None;
    }
    return Some((pid, identity.to_owned()));
}

/// What:
///  Keep the file at `lock` from having its inode given to another file,
///  by holding a second name for it in the records folder,
///  and name that second name for the Git that took it.
/// Why:
///   A record of an inode number alone is no proof:
///  a file system gives a freed inode to a new file,
///  and two files of two subvolumes of one mount share the number.
///  The link keeps the inode allocated for as long as the record stands,
///  so a lock at the path later is a different file and the record cannot match it.
///  A link cannot be made when the lock is gone or on another file system,
///  and then no record is written:
///  the lock keeps the bounded wait and report it has today.
///  The record is one file,
///  so it appears whole or not at all,
///  and the only thing written before it is the path's own text,
///  which is the same for every record of that path.
///  The link is looked at once more before this returns:
///  a lock renamed away and made again at the same path between the look above and the link leaves a pin
///  for another file,
///  and then the pin is taken away and nothing is recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePathText(digest, lock); linkSync(lock, pinName); if (statSync(pin).ino !== statSync(lock).ino) return undefined;
/// ```
pub fn write(common_dir: &Path, lock: &Path, holder: &(i64, String)) -> Option<Written> {
    let folder: PathBuf = folder_of(common_dir);
    std::fs::create_dir_all(folder.as_path()).ok()?;
    // The file this record is named for is the one standing at the path now; the link below is checked against it
    //  before this returns.
    let seen: LockMetadata = crate::index_lock_evidence::read_lock_metadata(lock).ok()??;
    let file: (u64, u64) = (seen.device, seen.inode);
    let digest: String = digest_of_path(lock);
    name_the_path(folder.as_path(), digest.as_str(), lock)?;
    let name: String = pin_name(digest.as_str(), file, holder)?;
    let pin: PathBuf = folder.join(name.as_str());
    // A file of a killed run of this very take, or a planted one, goes first: this name belongs to this one take.
    let _removed = std::fs::remove_file(pin.as_path());
    std::fs::hard_link(lock, pin.as_path()).ok()?;
    let linked: Option<LockMetadata> = crate::index_lock_evidence::read_lock_metadata(pin.as_path()).ok().flatten();
    // What: A control reaches here, with the pin made reachable under its own name and the record complete, to act as
    //       another command whose pass sweeps the folder at this instant.
    // Why:  The publication has to survive that sweep: the lock stands, so the record is the proof that clears it once the
    //       Git holding it is killed (a review of `2ac27d2a29`, finding 1).
    crate::service_fault::reach(A_PIN_IS_REACHABLE);
    if !matches!(linked, Some(again) if (again.device, again.inode) == file) {
        // The path holds another file than the one this record is named for, so this pin describes nothing.
        let _unlinked = std::fs::remove_file(pin.as_path());
        return None;
    }
    return Some(Written { pin, lock: lock.to_path_buf() });
}

/// What:
///  Make sure the file that names the path's text is there and says `lock`,
///  writing it when it is not.
/// Why:
///   The sweep judges every record of a path by statting that path,
///  and the digest cannot be read back without this text.
///  It is written before any record of the path and never rewritten,
///  so a record of the path never exists beside a half-written one;
///  a text that differs from the lock's own path is a planted or damaged file and is replaced,
///  because this folder's name is that path's digest and no other path belongs in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (readFileSync(pathFile) !== lockPath) writeFileSync(pathFile, lockPath)
/// ```
fn name_the_path(folder: &Path, digest: &str, lock: &Path) -> Option<()> {
    use std::os::unix::ffi::OsStrExt as _;
    let text: PathBuf = lock_path_file(folder, digest);
    let wanted: &[u8] = lock.as_os_str().as_bytes();
    let present: Option<Vec<u8>> = read_plain_file(text.as_path(), LOCK_PATH_LIMIT).ok();
    if matches!(present.as_deref(), Some(bytes) if bytes == wanted) {
        return Some(());
    }
    let temporary: PathBuf = folder.join(format!("{digest}{LOCK_PATH_SUFFIX}.writing"));
    std::fs::write(temporary.as_path(), wanted).ok()?;
    return std::fs::rename(temporary.as_path(), text.as_path()).ok();
}

/// What:
///  The record written for the lock at `lock`,
///  when its pin still holds the very file that stands at that path now.
/// Why:
///   A record only ever describes one file,
///  so a pin that no longer names the file at the path describes a lock that is gone,
///  and the pass reads nothing from it.
///  A record is found by the inode its pin names,
///  and the path's text says which path that inode was taken at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binding = readBinding(commonDir, lock); // undefined unless a pin names the file at that path
/// ```
pub fn of(common_dir: &Path, lock: &Path) -> Option<Binding> {
    let folder: PathBuf = folder_of(common_dir);
    let standing: LockMetadata = crate::index_lock_evidence::read_lock_metadata(lock).ok()??;
    let digest: String = digest_of_path(lock);
    if !lock_path_says(folder.as_path(), digest.as_str(), lock) {
        return None;
    }
    let prefix: String = format!("{digest}.");
    let Ok(entries) = std::fs::read_dir(folder.as_path()) else {
        return None;
    };
    for entry in entries.flatten() {
        let name: std::borrow::Cow<'_, str> = entry.file_name().to_string_lossy().into_owned().into();
        if !name.starts_with(prefix.as_str()) || !name.ends_with(HOLD_SUFFIX) {
            continue;
        }
        let Ok(Some(pinned)) = crate::index_lock_evidence::read_lock_metadata(entry.path().as_path()) else {
            continue;
        };
        if pinned.device != standing.device || pinned.inode != standing.inode {
            continue;
        }
        let (pid, identity) = holder_of_pin_name(digest.as_str(), name.as_ref())?;
        return Some(Binding { lock: lock.to_path_buf(), pin: entry.path(), pid, identity });
    }
    return None;
}

/// What:
///  Whether the path's text beside this digest still says `lock`.
/// Why:
///   A record is about the path its folder was named for,
///  and a text that names another path means this record is not that path's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(`${digest}.lock-path`) === lockPath
/// ```
fn lock_path_says(folder: &Path, digest: &str, lock: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt as _;
    let Ok(bytes) = read_plain_file(lock_path_file(folder, digest).as_path(), LOCK_PATH_LIMIT) else {
        return false;
    };
    return bytes.as_slice() == lock.as_os_str().as_bytes();
}

/// What:
///  Remove the record `written` describes.
/// Why:
///  A lock that was cleared needs no record,
///  and one its child finished explains nothing.
///  The record is one file,
///  named for the take it belongs to,
///  so this takes away that record and no other:
///  a command finishing its own lock never removes a record another command wrote,
///  whatever the path holds by then
///  (a review of `f8f5cba090`, finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unlinkSync(written.pin)
/// ```
pub fn remove(written: &Written) {
    let _removed = std::fs::remove_file(written.pin.as_path());
}

/// What:
///  Whether the pin of `written` still holds the file standing at the path `written` names.
/// Why:
///   A record is kept exactly as long as its own lock stands:
///  a pin that names the file at the path describes a lock that is still there,
///  and one that does not describes a lock its Git finished, rolled back or lost.
///  The question is about this record's own file only,
///  so a concurrent command's lock at the same path cannot change the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statSync(written.pin).ino === statSync(written.lock).ino
/// ```
pub fn pin_holds_the_lock(written: &Written) -> bool {
    let Ok(Some(pinned)) = crate::index_lock_evidence::read_lock_metadata(written.pin.as_path()) else {
        return false;
    };
    let Ok(Some(standing)) = crate::index_lock_evidence::read_lock_metadata(written.lock.as_path()) else {
        return false;
    };
    return pinned.device == standing.device && pinned.inode == standing.inode;
}

/// What:
///  Remove every record whose lock no longer stands,
///  and every file in the folder that is not a record of a path.
/// Why:
///   A record is kept exactly as long as the lock it pinned stands,
///  because the pin keeps that inode from being given to another file.
///  Each record carries the name of the take it belongs to,
///  and the path it is about is read from the text its digest names,
///  so the judgement is about one record and one file and never about what a path holds for another
///  command's lock.
///  A record that was written, and whose command was killed before it could take it away, goes here once
///  its lock is gone.
///  A pin whose path's text cannot be read is kept:
///  nothing shows which file it stands for, and a record kept is a clearance that may still be made.
///  The two files of the earlier format of this folder
///  (`<digest>.pin` and `<digest>.holder`, of the commits before this one,
/// never released)
///  are litter no reader of this release reads, and go.
///  Two commands sweeping at once remove the same litter harmlessly:
///  each removal is one `unlink` of a file whose name only its own take uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of readdirSync(pinned)) if (!pinnedFileStands(entry)) unlink(entry)
/// ```
pub fn sweep(common_dir: &Path) {
    let folder: PathBuf = folder_of(common_dir);
    let Ok(entries) = std::fs::read_dir(folder.as_path()) else {
        return;
    };
    for entry in entries.flatten() {
        let name: std::borrow::Cow<'_, str> = entry.file_name().to_string_lossy().into_owned().into();
        let Some((digest, rest)) = name.split_once('.') else {
            let _removed = std::fs::remove_file(entry.path().as_path());
            continue;
        };
        if rest == LOCK_PATH_SUFFIX.trim_start_matches('.') {
            // The path's own text: the one file of a path that is written before any of its records.
            continue;
        }
        if !rest.ends_with(HOLD_SUFFIX) || holder_of_pin_name(digest, name.as_ref()).is_none() {
            let _removed = std::fs::remove_file(entry.path().as_path());
            continue;
        }
        let Ok(bytes) = read_plain_file(lock_path_file(folder.as_path(), digest).as_path(), LOCK_PATH_LIMIT) else {
            continue;
        };
        use std::os::unix::ffi::OsStringExt as _;
        let lock: PathBuf = PathBuf::from(std::ffi::OsString::from_vec(bytes));
        let Ok(Some(pinned)) = crate::index_lock_evidence::read_lock_metadata(entry.path().as_path()) else {
            continue;
        };
        let standing: Option<LockMetadata> = crate::index_lock_evidence::read_lock_metadata(lock.as_path()).ok().flatten();
        if matches!(standing, Some(seen) if seen.device == pinned.device && seen.inode == pinned.inode) {
            continue;
        }
        let _removed = std::fs::remove_file(entry.path().as_path());
    }
}
