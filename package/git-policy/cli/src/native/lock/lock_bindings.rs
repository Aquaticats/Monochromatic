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
/// The name Git's PID file beside a lock has.
use crate::stale_ref_lock_walk::pid_file_beside;
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

/// What sits between one path's digest and the writer of its text,
/// in the temporary the text is published from.
const PATH_TEXT_TEMP_INFIX: &str = ".lock-path.";

/// The longest text one read of a path's file takes:
/// a path and nothing else.
const LOCK_PATH_LIMIT: u64 = 64 * 1024;

/// What every record's own file name carries after the path's digest:
/// the mark of the shape this release writes.
/// Why:
///   A record written by an earlier release named the file's device and inode and the holder,
///  and nothing of the folder the lock was taken in.
///  This release reads no such record:
///  a name without this mark is litter to it and is swept,
///  and a lock such a record described is kept,
///  because keeping a lock costs a command that waits and reporting it,
///  and clearing one on a record this release cannot check costs another writer's work
///  (a review of `6a506fc03f`,
/// finding 1).
const NAME_MARK: &str = "h2";

/// The most bytes a record's own file name may take,
/// which keeps a long birth identity from failing the link with a name that is too long.
/// The file system's own limit is 255 bytes for one name.
const MOST_NAME_BYTES: usize = 240;

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

/// The instant the path's text stands complete under its temporary name,
/// before it is renamed into place.
/// Why:
///   A control states the interleaving that publication must survive:
///  another command's sweep runs at this instant and may not take away a file a live writer is still
///  publishing
///  (`a_sweep_beside_a_path_text_write_leaves_the_record_of_a_standing_lock`).
///  Named here so both halves of that promise reach one point,
/// and inert unless the test variable or a control names it.
pub const A_PATH_TEXT_IS_BEING_WRITTEN: &str = "a-path-text-is-being-written";

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
    /// The folder the lock was taken in,
    ///  by the device and inode it had when the watch was opened there.
    pub folder: (u64, u64),
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
fn pin_name(digest: &str, folder: (u64, u64), file: (u64, u64), holder: &(i64, String)) -> Option<String> {
    let name: String = format!("{digest}.{NAME_MARK}.{}.{}.{}.{}.{}.{}{HOLD_SUFFIX}", folder.0, folder.1, file.0, file.1, holder.0, holder.1);
    if name.len() > MOST_NAME_BYTES || holder.1.contains('/') || holder.1.contains('\0') {
        return None;
    }
    return Some(name);
}

/// What:
///  What one record's own file name says:
///  the folder the lock was taken in,
///  the file the take made,
///  and the Git that took it.
/// Why:
///  The name is the whole record,
///  and every part of it is checked against the things themselves:
///  the pin's own metadata against the file part,
///  the folder standing at the lock's path against the folder part,
///  and the PID file's number against the holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PinFields = { folder: [number, number]; file: [number, number]; pid: number; identity: string };
/// ```
struct PinFields {
    /// The device and inode of the folder the lock was taken in.
    folder: (u64, u64),
    /// The device and inode of the file the take made.
    file: (u64, u64),
    /// The Git that took the lock, by the number its PID file carries.
    pid: i64,
    /// That Git's birth identity.
    identity: String,
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
fn pin_fields(digest: &str, name: &str) -> Option<PinFields> {
    let rest: &str = name.strip_prefix(format!("{digest}.").as_str())?.strip_suffix(HOLD_SUFFIX)?;
    let mut fields = rest.splitn(7, '.');
    if fields.next()? != NAME_MARK {
        return None;
    }
    let folder: (u64, u64) = (fields.next()?.parse::<u64>().ok()?, fields.next()?.parse::<u64>().ok()?);
    let file: (u64, u64) = (fields.next()?.parse::<u64>().ok()?, fields.next()?.parse::<u64>().ok()?);
    let pid: i64 = fields.next()?.parse::<i64>().ok()?;
    let identity: &str = fields.next()?;
    if identity.is_empty() {
        return None;
    }
    return Some(PinFields { folder, file, pid, identity: identity.to_owned() });
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
///  Two things are looked at once more after the link, and the pin is taken away if either says this
///  record is not the take it is named for:
///
///  - the pin still names the file standing at the path,
///    so a lock renamed away and made again between the look above and the link records nothing;
///  - the PID file beside that file still names the Git this record is about,
///    so a take that finished in the gap between the look that reported it and the link records
///    nothing.
///    Without this look the record would pin the new file and name the Git that no longer holds it,
///    and a reader that stopped at the first record pinning the file would be stopped by that one
///    and never judge the right record, so a dead Git's lock would stand and the next commit fail
///    (a review of `5327cbbfa4`,
///    finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// linkSync(lock, pinName); if (statSync(pin).ino !== statSync(lock).ino || pidOf(pidFile) !== holder.pid) return undefined;
/// ```
pub fn write(common_dir: &Path, lock: &Path, holder: &(i64, String), folder: (u64, u64)) -> Option<Written> {
    // What: The folder the lock's path names must still be the folder the watch opened there.
    // Why:  The name of this record carries that folder, and a record whose folder is gone describes a path this run
    //       never saw the lock made at (a review of `6a506fc03f`, finding 1).
    if folder_of_the_lock(lock) != Some(folder) {
        return None;
    }
    let records: PathBuf = folder_of(common_dir);
    std::fs::create_dir_all(records.as_path()).ok()?;
    // The file this record is named for is the one standing at the path now; the link below is checked against it
    //  before this returns.
    let seen: LockMetadata = crate::index_lock_evidence::read_lock_metadata(lock).ok()??;
    let file: (u64, u64) = (seen.device, seen.inode);
    let digest: String = digest_of_path(lock);
    name_the_path(records.as_path(), digest.as_str(), lock, holder)?;
    let name: String = pin_name(digest.as_str(), folder, file, holder)?;
    let pin: PathBuf = records.join(name.as_str());
    // What: The pin is published by one link, and a writer that finds the name taken keeps the file that is there.
    // Why:  Two writers can publish one take: a watcher that is late and the caller that waited for it.
    //       A removal first would take away the other writer's pin for the instant between the removal and the link,
    //       and a kill there leaves the lock with no record at all (a review of `bbda166ce2`, finding 6).
    //       The name is the take's own: the path's digest with the file's device and inode and the Git's number and
    //       identity, so a file already there is this same take's record, or the litter of a killed run of it.
    //       A file there that is not the file this take made is litter of another file's name and is removed once.
    if std::fs::hard_link(lock, pin.as_path()).is_err() {
        let existing: Option<LockMetadata> = crate::index_lock_evidence::read_lock_metadata(pin.as_path()).ok().flatten();
        if !matches!(existing, Some(there) if (there.device, there.inode) == file) {
            let _litter = std::fs::remove_file(pin.as_path());
            std::fs::hard_link(lock, pin.as_path()).ok()?;
        }
    }
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
    // The second look: the PID file beside the lock must name the Git this record names. A take that finished here
    //  took its PID file with it, and another Git's PID file names another process, so either way this record would name
    //  a Git that does not hold the file it pins.
    //  A lock with no PID file is unproven and no record is written for it: the sight of its creation does not say who
    //  made it (`SPEC.md`, "The rule: a lock is removed only on a proof about its holder";
    //  a review of `412fb03192`, finding 1).
    if crate::stopped_git_locks::pid_named_by(pid_file_beside(lock).as_path()) != Some(i32::try_from(holder.0).ok()?) {
        let _unlinked = std::fs::remove_file(pin.as_path());
        return None;
    }
    // The third look: the folder standing at the path must still be the one this record names. A folder renamed away
    //  between the first look and the link would leave a record of a path that is another folder's now.
    if folder_of_the_lock(lock) != Some(folder) {
        let _unlinked = std::fs::remove_file(pin.as_path());
        return None;
    }
    return Some(Written { pin, lock: lock.to_path_buf() });
}

/// What:
///  The device and inode of the folder the lock at `lock` stands in,
///  or nothing when the path has no folder this process can read.
/// Why:
///  A record names the folder a take was watched in,
///  and both the writer and the reader compare it with the folder standing at the path now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// folderOfTheLock(dir/index.lock) // [dev, ino] of dir
/// ```
fn folder_of_the_lock(lock: &Path) -> Option<(u64, u64)> {
    let folder: &Path = lock.parent()?;
    let seen: LockMetadata = crate::index_lock_evidence::read_lock_metadata(folder).ok()??;
    return Some((seen.device, seen.inode));
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
///  The text is made under a temporary name that carries **the process writing it** - this command, not
///  the Git the record is about -
///  and the rename into place is tried twice.
///  A sweep keeps a temporary whose writer runs and removes one whose writer is gone
///  (`sweep`),
///  and the Git a record is about is often already dead when its command writes that record,
///  which is the whole case the record exists for:
///  a temporary named for that Git was therefore read as a dead run's litter and removed,
///  and two sweeps in the two attempts left the lock with no record at all
///  (the second finding of a review of `5327cbbfa4`).
///  A temporary named for the command itself is kept by every sweep for as long as the write can
///  happen,
///  and the second attempt is what covers a temporary another actor took away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (readFileSync(pathFile) !== lockPath) for (let attempt = 0; attempt < 2; attempt++) writeTempThenRename()
/// ```
fn name_the_path(folder: &Path, digest: &str, lock: &Path, holder: &(i64, String)) -> Option<()> {
    use std::os::unix::ffi::OsStrExt as _;
    let text: PathBuf = lock_path_file(folder, digest);
    let wanted: &[u8] = lock.as_os_str().as_bytes();
    let present: Option<Vec<u8>> = read_plain_file(text.as_path(), LOCK_PATH_LIMIT).ok();
    if matches!(present.as_deref(), Some(bytes) if bytes == wanted) {
        return Some(());
    }
    // The process writing this text is this one; the Git the record is about may already be gone, and a sweep that read
    //  the temporary as that Git's would remove it.
    let writer: (i64, String) = crate::lock_holder_records::identity_of_process(i64::from(std::process::id()))
        .map(|identity| { return (i64::from(std::process::id()), identity); })
        .unwrap_or_else(|| { return holder.clone(); });
    let temporary: PathBuf = folder.join(format!("{digest}{PATH_TEXT_TEMP_INFIX}{}.{}.writing", writer.0, writer.1));
    for _attempt in 0..2 {
        std::fs::write(temporary.as_path(), wanted).ok()?;
        // A control reaches here to act as another command whose pass sweeps the folder at this instant.
        crate::service_fault::reach(A_PATH_TEXT_IS_BEING_WRITTEN);
        if std::fs::rename(temporary.as_path(), text.as_path()).is_ok() {
            return Some(());
        }
    }
    return None;
}

/// What:
///  The process number and birth identity of the Git that is writing one path's text,
///  from the name of its temporary file,
///  or nothing when the name is not one of those.
/// Why:
///   A sweep that removes a temporary a live writer is still publishing takes that writer's record
///  away,
///  so the sweep has to know whose temporary it is before it decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writerOfPathTemp('lock-path.4242.linux:1#…​.writing') // [4242, identity]
/// ```
fn writer_of_path_temp(rest: &str) -> Option<(i64, String)> {
    // `rest` is the name after the digest and its dot, so the infix is matched without its own leading dot.
    let fields: &str = rest.strip_prefix(PATH_TEXT_TEMP_INFIX.trim_start_matches('.'))?.strip_suffix(".writing")?;
    let (pid, identity) = fields.split_once('.')?;
    if identity.is_empty() {
        return None;
    }
    return Some((pid.parse::<i64>().ok()?, identity.to_owned()));
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
///  Several records can pin the one file,
///  because a record written for the wrong Git pins a file it does not describe
///  (a review of `5327cbbfa4`, finding 1),
///  and this answers every one of them:
///  a caller that decides about the lock judges each,
///  so a record that cannot be the holder never stops it from reaching the one that can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bindings = readBindings(commonDir, lock); // [] unless a pin names the file at that path
/// ```
pub fn bindings_of(common_dir: &Path, lock: &Path) -> Vec<Binding> {
    let folder: PathBuf = folder_of(common_dir);
    let mut found: Vec<Binding> = Vec::new();
    let Ok(Some(standing)) = crate::index_lock_evidence::read_lock_metadata(lock) else {
        return found;
    };
    // What: The folder standing at the lock's path now, which every record of it must have been written about.
    // Why:  A record is a licence to delete a file, and a record written in a folder that has since been renamed
    //       away describes a path that is another folder's now (a review of `6a506fc03f`, finding 1).
    let Some(here): Option<(u64, u64)> = folder_of_the_lock(lock) else {
        return found;
    };
    let digest: String = digest_of_path(lock);
    if !lock_path_says(folder.as_path(), digest.as_str(), lock) {
        return found;
    }
    let prefix: String = format!("{digest}.");
    let Ok(entries) = std::fs::read_dir(folder.as_path()) else {
        return found;
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
        let Some(named) = pin_fields(digest.as_str(), name.as_ref()) else {
            continue;
        };
        if named.folder != here || named.file != (pinned.device, pinned.inode) {
            continue;
        }
        found.push(Binding { lock: lock.to_path_buf(), pin: entry.path(), folder: named.folder, pid: named.pid, identity: named.identity });
    }
    // The order a directory lists its names in belongs to the file system; the records are judged in the same order
    //  everywhere, so a repository judged on one machine is judged the same way on another.
    found.sort_by(|one, other| { return one.pin.cmp(&other.pin); });
    return found;
}

/// What:
///  The first record that pins the file standing at `lock`,
///  for the callers that ask about a lock's existence and not about a decision.
/// Why:
///  A decision must judge every record (`bindings_of`), and this is that list's first for the places
///  that only ask whether any record describes the lock at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = readBindings(commonDir, lock)[0]
/// ```
pub fn of(common_dir: &Path, lock: &Path) -> Option<Binding> {
    return bindings_of(common_dir, lock).into_iter().next();
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
///  The process numbers that the pins standing in this repository's records name.
/// Why:
///  A pin is the proof about one lock,
///  and a process record is the other half of that proof:
///  the number and the birth identity the record names are what says the Git that took the lock is
///  gone.
///  So a process record may go only when no pin that stands names a number in it,
///  and that is asked when the record is removed,
///  not when the lock directories were walked
///  (a review of `6c2272132b`,
/// finding 1).
///  A pin whose lock no longer stands is litter and names nothing:
///  this is the list after `sweep` removed those,
///  or the list of the pins that still hold a file standing at the path their text names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holderNumbersThatStand(commonDir) // [pid, …] of the pins whose lock still stands
/// ```
pub fn holder_numbers_that_stand(common_dir: &Path) -> Vec<i64> {
    let folder: PathBuf = folder_of(common_dir);
    let mut numbers: Vec<i64> = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder.as_path()) else {
        return numbers;
    };
    for entry in entries.flatten() {
        let name: std::borrow::Cow<'_, str> = entry.file_name().to_string_lossy().into_owned().into();
        let Some((digest, rest)) = name.split_once('.') else {
            continue;
        };
        if !rest.ends_with(HOLD_SUFFIX) {
            continue;
        }
        let Some(named) = pin_fields(digest, name.as_ref()) else {
            continue;
        };
        let Ok(bytes) = read_plain_file(lock_path_file(folder.as_path(), digest).as_path(), LOCK_PATH_LIMIT) else {
            continue;
        };
        use std::os::unix::ffi::OsStringExt as _;
        let lock: PathBuf = PathBuf::from(std::ffi::OsString::from_vec(bytes));
        let (Ok(Some(pinned)), Ok(Some(standing))) = (
            crate::index_lock_evidence::read_lock_metadata(entry.path().as_path()),
            crate::index_lock_evidence::read_lock_metadata(lock.as_path()),
        ) else {
            continue;
        };
        if pinned.device != standing.device || pinned.inode != standing.inode {
            continue;
        }
        numbers.push(named.pid);
    }
    numbers.sort_unstable();
    numbers.dedup();
    return numbers;
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
///  A path's text being published is kept while the Git writing it runs and removed once that Git is
///  gone,
///  so a sweep never takes away the temporary of a live writer and a killed writer's temporary does
///  not stay.
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
        if let Some(writer) = writer_of_path_temp(rest) {
            // A text being published: kept while the Git writing it runs, and taken away once that Git is gone.
            if crate::service_index_identity::holder_state(Some(&writer)) == crate::service_index_identity::HolderState::Gone {
                let _removed = std::fs::remove_file(entry.path().as_path());
            }
            continue;
        }
        if !rest.ends_with(HOLD_SUFFIX) || pin_fields(digest, name.as_ref()).is_none() {
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
