//! What:
//!  The record of one leaving a removal makes:
//!  the directory its subvolume is renamed aside to,
//!  written before the rename and settled by nothing,
//!  and the queueing of that delete which outlives the delete's own queue record.
//! Why:
//!  A removal hands a subvolume's delete to the commit service through a work item,
//!  and the first round of the service that looks at the item settles it.
//!  A round that looked in the moment between the record and the rename finds no directory there,
//!  settles the delete as done,
//!  and the person's own command destroys the subvolume it renamed
//!  (`worktree_remove_hand_over.rs`,
//! `a_record_is_queued`).
//!  That repair runs only while that command lives,
//!  and the destroy can fail for a reason that lasts:
//!  a subvolume that holds another one (the kernel refuses with `ENOTEMPTY`),
//!  a mount that allows no unprivileged destroy (`EPERM`),
//!  or a volume that went read-only.
//!  A command killed between the rename and the repair's own queueing leaves the same state with less:
//!  a queue record nothing reads again,
//!  or nothing at all.
//!  So the leaving is a record of its own.
//!  It is written before the rename,
//!  nothing consumes or settles it,
//!  and it names the path and the subvolume id a delete needs.
//!  A command that runs later finds it,
//!  proves what stands there is the subvolume it names,
//!  and queues the delete again,
//!  and once the attempts a record may cost have run out it names the directory and the two commands that end it
//!  (`worktree_removal_leavings_a_command_queues_again.rs`).
//!  Nothing here deletes anything:
//!  the destroy stays off the path a person's command waits on
//!  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const leaving = await recordALeaving(commonDir, item);  // before the rename
//! forgetTheLeaving(commonDir, id);                        // once nothing of ours stands there
//! await queueTheDeleteAgain(commonDir, leaving);           // a record no round owes any more
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::private_storage::{create_private_directory_all, replace_private_file_unsynced};
use crate::process_identity::current_birth_identity;
use crate::service_work::work_directory;
use crate::work_item::{DONE, FAILED, PENDING, WorkItem, requeue_after_giving_up, state_directory};
use std::fs::File;
use std::path::{Path, PathBuf};

/// The folder of the common Git directory that holds one record per subvolume a removal has renamed aside and not yet
/// proved gone.
pub const LEAVINGS_FOLDER: &str = "monochromatic-git-policy-cli/leavings";

/// What:
///  The folder of the common Git directory that holds one record per directory a removal of this program has renamed a
///  subvolume inside,
///  so a command still knows where to look once the registration of the worktree that stood there is gone.
/// Why:
///   The look for a leftover of a removal reads the places a removal renames into,
///  and it finds them in the registrations of the linked worktrees that stand now
/// (`worktree_removal_leavings_no_record_names.rs`).
///  A removal is exactly what takes a registration away:
///  where a directory held one worktree,
///  the command that renames that worktree's subvolume also removes the last registration,
///  and every later command would have no directory to read.
///  What the removal knows at that moment is the directory it renames in,
///  so it writes it down before the rename,
///  and a command that finds nothing of this program's left there drops the record.
pub(crate) const LEAVING_PLACES_FOLDER: &str = "monochromatic-git-policy-cli/leaving-places";

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeaving";

/// The prefix of the record's line that names the command that wrote it,
/// as its process and the identity that tells it from a later process of the same number.
const OWNER_PREFIX: &str = "owner=";

/// The prefix of the record's line that counted the times a command had queued this leaving's delete again.
/// What reads it:
/// a record an earlier build of this branch wrote,
/// which this release accepts and no longer writes,
/// because the attempts a leaving may cost are counted by its slot files
/// (`worktree_removal_leaving_attempts.rs`).
const TRIES_PREFIX: &str = "tries=";

/// The prefix of the record's line that says a command found this leaving by its name alone,
/// so no command of this program renamed it and none may still be between a record and a rename.
const ADOPTED_PREFIX: &str = "adopted=";

/// What a subvolume this program is about to rename aside carries from then on:
/// an extended attribute naming that subvolume's own id,
/// which travels with it through the rename.
/// Why:
///  a name is no proof of who renamed a directory
/// (`worktree_removal_leavings_no_record_names.rs`),
///  and a record lives in the repository's Git directory,
///  which a person may delete or a disaster may take.
///  The attribute is carried by the directory itself,
///  so the look that finds one of this program's leftovers can tell it from a person's directory even when every record
///  is gone.
///  The bytes are the id in the same sixteen hexadecimal digits the leaving name ends in,
///  so a copy of the attribute onto another subvolume names an id that directory does not have and proves nothing.
const LEAVING_MARK: &str = "user.monochromatic-git-policy-cli-leaving";

/// The most bytes of a leaving mark this reads,
/// which is the sixteen digits and nothing else.
const MARK_LIMIT: usize = 32;

/// The most bytes of a leaving record this reads.
/// What bounds it:
/// four short lines and the JSON of one work item,
/// which names three paths,
/// and a path is at most the kernel's `PATH_MAX`,
/// 4096 bytes.
pub(crate) const RECORD_LIMIT: u64 = 16384;

/// What:
///  One leaving,
/// as its record holds it:
///  the command that renamed the subvolume,
///  whether a scan found it by its name alone,
///  and the work item whose path and object name the delete acts on.
/// Why:
///   The item's path is the place the rename made and its object name is the subvolume's own id,
///  which together are the proof a delete needs.
///   The owner says whether a command may still be between its record and its rename,
///  which is the one state a later command may not decide about.
///   No count of attempts lives here:
///  the attempts a leaving may cost are counted by the slot files of its own
///  (`worktree_removal_leaving_attempts.rs`),
///  and a second count in the record would be one more number a command reads and writes back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Leaving = { owner?: string; adopted: boolean; item: WorkItem };
/// ```
pub struct Leaving {
    /// The command that wrote the record,
    /// as `<pid>:<birth identity>`;
    /// nothing for a record written without one.
    pub owner: Option<String>,
    /// Whether a command found this leaving by its name alone,
    /// no command of this program having renamed anything into it,
    /// which is the one state every command may decide about at once.
    pub adopted: bool,
    /// The record of the delete:
    /// its `parent` is the directory the rename made,
    /// and its `oid` is the subvolume id it holds.
    pub item: WorkItem,
}

/// The file one leaving is recorded in,
/// named by the id of its item so that one leaving is one record.
fn record_path(common_dir: &Path, id: &str) -> PathBuf {
    return common_dir.join(LEAVINGS_FOLDER).join(id);
}

/// What:
///  The bytes of one leaving's record:
///  the command that wrote it,
///  whether a scan found it,
///  and the work item.
/// Why:
///   The short lines come first,
///  one per line,
///  so nothing a path or a repository name holds can be taken for them,
///  and everything after them is the item's own JSON.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // `adopted` is written only when it is true, so a record a command renamed into keeps the bytes an earlier reader wrote.
/// const bytes = (item, owner, adopted) => `${owner ? 'owner=' + owner + '\n' : ''}${adopted ? 'adopted=1\n' : ''}${item.encode()}`;
/// ```
fn bytes_of_the_leaving(item: &WorkItem, owner: Option<&str>, adopted: bool) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    if let Some(spelled) = owner {
        bytes.extend_from_slice(format!("{OWNER_PREFIX}{spelled}\n").as_bytes());
    }
    if adopted {
        bytes.extend_from_slice(format!("{ADOPTED_PREFIX}1\n").as_bytes());
    }
    bytes.extend_from_slice(item.encode().as_bytes());
    return bytes;
}

/// What:
///  Write the record of one leaving whole,
///  over whatever the name held,
///  and answer the reason it cannot be written.
/// Why:
///   The record is published by one rename,
///  so a command killed while it is written leaves the earlier record or the complete new one and never half of either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await replacePrivateFile(join(commonDir, LEAVINGS_FOLDER, item.id()), bytes);
/// ```
fn write_the_leaving(common_dir: &Path, item: &WorkItem, owner: Option<&str>, adopted: bool) -> Result<(), String> {
    let folder: PathBuf = common_dir.join(LEAVINGS_FOLDER);
    if let Err(error) = create_private_directory_all(folder.as_path()) {
        return Err(format!("the folder {} cannot be made ({error})", folder.display()));
    }
    let path: PathBuf = record_path(common_dir, item.id().as_str());
    return match replace_private_file_unsynced(path.as_path(), bytes_of_the_leaving(item, owner, adopted).as_slice()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} cannot be written ({error})", path.display())),
    };
}

/// What:
///  Record one leaving,
///  naming this command,
///  before the rename that makes it.
/// Why:
///   A removal that cannot write this record must not rename:
///  the queue record of the delete is settled by the first round that looks at it,
///  and this record is the only thing left that names the directory once that has happened.
///  A command that cannot name itself is refused here for the same reason:
///  a record naming no owner is one every later command must leave alone,
///  because nothing proves the command that wrote it is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leaving = await recordALeavingWithOwner(commonDir, item, `${process.pid}:${birthIdentity()}`);
/// ```
///
/// # Errors
///
/// The words for a command that cannot name itself and for a record that cannot be written.
pub fn record_a_leaving(common_dir: &Path, item: &WorkItem) -> Result<Leaving, String> {
    let owner: String = match current_birth_identity() {
        Ok(identity) => format!("{}:{identity}", std::process::id()),
        Err(error) => {
            return Err(format!(
                "the name of this command cannot be read ({error}), so the leaving of {} cannot be recorded and the subvolume is destroyed where it stands",
                item.parent
            ));
        }
    };
    return record_a_leaving_with_owner(common_dir, item, Some(owner.as_str()));
}

/// What:
///  `record_a_leaving`,
///  with the command the record names given instead of this process,
///  so a control can write the record of a removal that has ended.
///  `None` writes no owner line.
/// Why:
///   A later command leaves a leaving alone while the command that renamed it may still be running,
///  and a control that simulates a killed removal has to write one whose command is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordALeavingWithOwner(commonDir, item, '1:ended');
/// ```
///
/// # Errors
///
/// The words for a record that cannot be written.
pub fn record_a_leaving_with_owner(common_dir: &Path, item: &WorkItem, owner: Option<&str>) -> Result<Leaving, String> {
    write_the_leaving(common_dir, item, owner, false)?;
    return Ok(Leaving { owner: owner.map(str::to_owned), adopted: false, item: item.clone() });
}

/// What:
///  Record a leaving a command found by its name alone,
///  which no command of this program renamed into that place.
/// Why:
///   A directory whose name is the subvolume id it holds is one a removal of this program left,
///  even when no record of it survived:
///  an earlier release wrote only the queue record,
///  and the first round that looked at it settled it.
///  A command that writes the leaving it found gives the directory the same bound every other leaving has:
///  the attempts a record may cost,
///  and the line that names the directory and the command that removes it once those are spent.
///  It names no owner,
///  because no command of this program is between a record and a rename here,
///  and it says so in its own line rather than in the absence of an owner,
///  which a record an earlier release wrote also has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordALeavingFoundByAScan(commonDir, item); // the item a name and the kernel's id agree on
/// ```
///
/// # Errors
///
/// The words for a record that cannot be written.
pub(crate) fn record_a_leaving_found_by_a_scan(common_dir: &Path, item: &WorkItem) -> Result<Leaving, String> {
    write_the_leaving(common_dir, item, None, true)?;
    return Ok(Leaving { owner: None, adopted: true, item: item.clone() });
}

/// What:
///  Mark an open subvolume as one this program is about to rename aside,
///  by writing its own id into an extended attribute on it.
/// Why:
///   The mark is written before the rename and travels with the subvolume,
///  so a command killed between the two leaves a marked subvolume under the worktree's own name,
///  which is nothing this program acts on,
///  and never a directory under the leaving name that carries no mark.
///  It is not a record:
///  it lives on the directory the removal moves,
///  so it outlives the Git directory a person may delete.
///  A mark that cannot be written costs that proof and nothing else:
///  the leaving and queue records still name the directory,
///  so the caller warns rather than refusing the hand-over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// markTheLeaving(held, id); // setxattr(held, 'user.monochromatic-git-policy-cli-leaving', hex(id))
/// ```
///
/// # Errors
///
/// The words for an attribute the filesystem or the permission refuses.
pub(crate) fn mark_the_leaving(held: &File, id: u64) -> Result<(), String> {
    use std::os::fd::AsRawFd as _;
    let name = match std::ffi::CString::new(LEAVING_MARK) {
        Ok(named) => named,
        Err(_nul) => return Err(String::from("the mark's own name holds a NUL byte")),
    };
    let value: String = format!("{id:016x}");
    // SAFETY: `name` and `value` outlive the call, and `held` is an open descriptor of the directory the mark goes on.
    let result: libc::c_int = unsafe { libc::fsetxattr(held.as_raw_fd(), name.as_ptr(), value.as_ptr().cast(), value.len(), 0) };
    if result != 0 {
        return Err(format!("the mark {} cannot be set ({})", LEAVING_MARK, std::io::Error::last_os_error()));
    }
    return Ok(());
}

/// What:
///  The subvolume id the mark on an open directory names,
///  or nothing when it carries no mark this program wrote.
/// Why:
///   The mark is written before the rename and read through the descriptor the look already proved,
///  so a directory replaced at the path between the open and this read cannot lend its mark to another subvolume.
///  An attribute that is missing, longer than a mark, or not sixteen hexadecimal digits is no mark of this program's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marked = theMarkOf(held); // hex id, or undefined
/// ```
pub(crate) fn the_mark_of(held: &File) -> Option<u64> {
    use std::os::fd::AsRawFd as _;
    let name: std::ffi::CString = std::ffi::CString::new(LEAVING_MARK).ok()?;
    let mut buffer: [u8; MARK_LIMIT] = [0; MARK_LIMIT];
    // SAFETY: `name` outlives the call, `buffer` is a live writable region of the length passed, and `held` is open.
    let read: libc::ssize_t = unsafe { libc::fgetxattr(held.as_raw_fd(), name.as_ptr(), buffer.as_mut_ptr().cast(), buffer.len()) };
    if read <= 0 {
        return None;
    }
    let length: usize = usize::try_from(read).ok()?;
    let text: &str = std::str::from_utf8(buffer.get(..length)?).ok()?;
    if text.len() != 16 || !text.bytes().all(|byte| { return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte) }) {
        return None;
    }
    return u64::from_str_radix(text, 16).ok();
}

/// What:
///  Drop the record of one leaving,
///  once nothing of this program's stands where it named.
/// Why:
///   A record that outlived its directory would make the next command look at a path that has nothing to do with the
///  leaving,
/// and a record whose directory was already deleted costs one look on every command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm(join(commonDir, LEAVINGS_FOLDER, id));
/// ```
pub fn forget_the_leaving(common_dir: &Path, id: &str) {
    let path: PathBuf = record_path(common_dir, id);
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            warn(TAG, format!("{} cannot be removed ({error}); a later command looks at it again", path.display()).as_str());
            return;
        }
    }
    // An attempt belongs to the leaving it was taken for: one that outlived its record would spend the budget of the next
    // leaving of the same place.
    crate::worktree_removal_leaving_attempts::forget_the_attempts(common_dir, id);
    // The folder goes with the last record, as the record of a removal's destroy does; one that still holds another record
    // stays, which its own failure says.
    let _kept = std::fs::remove_dir(common_dir.join(LEAVINGS_FOLDER));
}

/// What:
///  The two names a queue record of this delete may stand under:
///  the one this release writes,
///  and the one a release before the branch field wrote,
///  which `WorkItem` still reads (`id_before_the_branch`).
/// Why:
///  A queue outlives the release that wrote to it,
///  and a leftover of an earlier release whose record carries the older name is still its leftover.
///  A look that asked one name alone would prove nothing and wake nobody
/// (`worktree_removal_leavings_no_record_names.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ids = the_ids_of(item); // [item.id(), item.id_before_the_branch()]
/// ```
pub(crate) fn the_ids_of(item: &WorkItem) -> [String; 2] {
    return [item.id(), item.id_before_the_branch()];
}

/// What:
///  Write down the directory a removal of this program is renaming a subvolume inside,
///  before the rename,
///  so a command still knows where to look once the registration of that worktree is gone.
/// Why:
///  The record is one small file per directory,
///  named by a digest of the directory's own bytes,
///  so a directory whose name is not text is recorded as itself
///  (a name spelled in hexadecimal twice its length would not fit a file name).
///  Writing it twice is one record,
///  and a failure costs the look on that directory and nothing else,
///  so the caller warns rather than refusing the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordThePlace(commonDir, place); // leaving-places/<sha1(place bytes)>
/// ```
pub(crate) fn record_the_place(common_dir: &Path, place: &Path) -> Result<(), String> {
    let folder: PathBuf = common_dir.join(LEAVING_PLACES_FOLDER);
    if let Err(error) = create_private_directory_all(folder.as_path()) {
        return Err(format!("the folder {} cannot be made ({error})", folder.display()));
    }
    let path: PathBuf = place_record_path(common_dir, place);
    let bytes: &[u8] = crate::path_bytes::path_bytes_of(place);
    return match replace_private_file_unsynced(path.as_path(), bytes) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} cannot be written ({error})", path.display())),
    };
}

/// What:
///  Every directory a removal of this program has recorded as one it renames inside,
///  with that directory's bytes as it wrote them.
/// Why:
///  These are the places the look reads beside the registrations that stand now,
///  so a leftover of a directory whose last registration went is still looked at.
///  A record that cannot be read names no directory and is left for the next command,
///  and one that is not absolute or not a directory any more is dropped by the caller's look at it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = the_recorded_places(commonDir); // [Path]
/// ```
pub(crate) fn the_recorded_places(common_dir: &Path) -> Vec<PathBuf> {
    use std::os::unix::ffi::OsStrExt as _;
    let folder: PathBuf = common_dir.join(LEAVING_PLACES_FOLDER);
    let Ok(listing) = std::fs::read_dir(folder.as_path()) else { return Vec::new() };
    let mut places: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let record: PathBuf = entry.path();
        let Ok(bytes) = crate::plain_file::read_plain_file(record.as_path(), RECORD_LIMIT) else {
            debug(TAG, format!("{} is not a plain record of a place and is left alone", record.display()).as_str());
            continue;
        };
        if bytes.is_empty() {
            continue;
        }
        let place: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(bytes.as_slice()));
        if !place.is_absolute() {
            debug(TAG, format!("{} names no absolute directory, so it is not a place a removal renames into", record.display()).as_str());
            continue;
        }
        places.push(place);
    }
    return places;
}

/// What:
///  The file one directory's place is recorded in,
///  named by a digest of the directory's bytes.
/// Why:
///  A path spelled in hexadecimal is twice its length,
///  and a path of the length the kernel allows is already more than a file name holds,
///  so the name is a digest and the bytes are the record's content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = join(commonDir, LEAVING_PLACES_FOLDER, sha1(pathBytes(place)));
/// ```
fn place_record_path(common_dir: &Path, place: &Path) -> PathBuf {
    let digest: String = crate::hexadecimal_of_bytes::hexadecimal_of_bytes(crate::subvolume_sha::sha1(crate::path_bytes::path_bytes_of(place)).as_slice());
    return common_dir.join(LEAVING_PLACES_FOLDER).join(digest);
}

/// What:
///  Drop the record of one directory a removal renames in,
///  once nothing of this program's is left there to look for.
/// Why:
///  A record that outlived its need would be read by every command for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetAPlace(commonDir, place);
/// ```
pub(crate) fn forget_a_place(common_dir: &Path, place: &Path) {
    let _removed = std::fs::remove_file(place_record_path(common_dir, place).as_path());
    let _kept = std::fs::remove_dir(common_dir.join(LEAVING_PLACES_FOLDER));
}

/// What:
///  The records this repository holds,
///  each with the id its file name carries.
/// Why:
///   One reading of the folder is the whole of what a command costs when no removal has left anything,
///  which is every command of an ordinary repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const records = readdirSync(join(commonDir, LEAVINGS_FOLDER)).map((name) => [name, join(folder, name)]);
/// ```
pub(crate) fn leaving_records(common_dir: &Path) -> Vec<(String, PathBuf)> {
    let folder: PathBuf = common_dir.join(LEAVINGS_FOLDER);
    let Ok(listing) = std::fs::read_dir(folder.as_path()) else { return Vec::new() };
    let mut found: Vec<(String, PathBuf)> = Vec::new();
    for entry in listing.flatten() {
        found.push((entry.file_name().to_string_lossy().into_owned(), entry.path()));
    }
    return found;
}

/// Whether one byte of a record is the newline that ends one of its short lines.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  The leaving a record holds,
///  or the reason it cannot be used.
/// Why:
///   A record a person or another program wrote into this folder must be told apart from this program's own,
///  and a record whose count or item cannot be read names nothing a command may act on.
///   A `tries=` line is a count an earlier build of this branch wrote,
///  which this release reads and drops:
///  the attempts a leaving may cost are its slot files' and never a number in its record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leaving = readTheLeaving(bytes); // throws on a line that is no line, a count that is no count, an item that is no item
/// ```
pub(crate) fn read_the_leaving(bytes: &[u8]) -> Result<Leaving, String> {
    let mut rest: &[u8] = bytes;
    let mut owner: Option<String> = None;
    let mut adopted: bool = false;
    loop {
        let Some(split_at) = rest.iter().position(is_newline) else { break };
        let (line, after): (&[u8], &[u8]) = rest.split_at(split_at);
        let Ok(text) = std::str::from_utf8(line) else { break };
        if let Some(spelled) = text.strip_prefix(OWNER_PREFIX) {
            owner = Some(spelled.to_owned());
            rest = after.strip_prefix(b"\n").unwrap_or(after);
            continue;
        }
        if let Some(spelled) = text.strip_prefix(ADOPTED_PREFIX) {
            adopted = spelled == "1";
            rest = after.strip_prefix(b"\n").unwrap_or(after);
            continue;
        }
        match text.strip_prefix(TRIES_PREFIX) {
            Some(spelled) => {
                if spelled.parse::<u32>().is_err() {
                    return Err(format!("its count {spelled} is no count of tries"));
                }
                rest = after.strip_prefix(b"\n").unwrap_or(after);
            }
            None => break,
        }
    }
    let item: &str = match std::str::from_utf8(rest) {
        Ok(text) => text,
        Err(_not_text) => return Err(String::from("the record of the delete it names is not text")),
    };
    return Ok(Leaving { owner, adopted, item: WorkItem::decode(item)? });
}

/// What:
///  Queue the delete of one leaving again and count the queueing,
///  answering the reason a round will not look at it.
/// Why:
///  A record settled in `done` is a delete no round looked at,
///  and one settled in `failed` is a delete the kernel refused until the attempts a record may cost ran out.
///  Nothing reads either of them again (`queued_ids` omits both),
/// so without this the directory stands with nothing naming
///  its delete.
///  A record in `failed` goes back where a round finds it through the queue's own `requeue_after_giving_up`,
/// which is one
///  rename of the record an earlier attempt left.
///  A record in `done` is written whole as a pending record before the settled one goes,
///  because a `done` record's own bytes are a delete that did not happen,
///  and a leaving whose record never reached the queue is written the same way.
///  Nothing here waits:
///  one write and up to two unlinks,
///  and a round the queue's wake has yet to start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queuedAgain = (commonDir, leaving) => requeueAfterGivingUp(work(commonDir), leaving.item.id()) || writePending(commonDir, leaving.item);
/// ```
///
/// # Errors
///
/// The words for a folder or a record that cannot be written,
/// which are the reason no round will look at the leaving.
pub fn queue_the_delete_again(common_dir: &Path, leaving: &Leaving) -> Result<(), String> {
    let work: PathBuf = work_directory(common_dir);
    let id: String = leaving.item.id();
    let ids: [String; 2] = the_ids_of(&leaving.item);
    // A record a release before the branch field wrote stands under the older name, and a delete it gave up on is still
    // the work this one owes.
    for name in &ids {
        if requeue_after_giving_up(work.as_path(), name.as_str()) {
            return Ok(());
        }
    }
    let pending: PathBuf = state_directory(work.as_path(), PENDING);
    if let Err(error) = create_private_directory_all(pending.as_path()) {
        return Err(format!("the folder {} cannot be made ({error})", pending.display()));
    }
    let again: WorkItem = WorkItem { tries: 0, ..leaving.item.clone() };
    match crate::private_storage::write_private_file_unsynced(pending.join(id.as_str()).as_path(), again.encode().as_bytes()) {
        Ok(()) => {}
        // Another command queued the same delete between this one's look and its write, which is the work this one owes done
        // by that command.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(format!("{} cannot be written ({error})", pending.join(id.as_str()).display())),
    }
    for name in &ids {
        let _settled_done = std::fs::remove_file(state_directory(work.as_path(), DONE).join(name.as_str()));
        let _settled_failed = std::fs::remove_file(state_directory(work.as_path(), FAILED).join(name.as_str()));
    }
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_leaving_that_outlives_its_record_tests.rs"]
mod tests;
