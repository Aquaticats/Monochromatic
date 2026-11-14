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
use std::path::{Path, PathBuf};

/// The folder of the common Git directory that holds one record per subvolume a removal has renamed aside and not yet
/// proved gone.
pub const LEAVINGS_FOLDER: &str = "monochromatic-git-policy-cli/leavings";

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeaving";

/// The prefix of the record's line that names the command that wrote it,
/// as its process and the identity that tells it from a later process of the same number.
const OWNER_PREFIX: &str = "owner=";

/// The prefix of the record's line that counts the times a command has queued this leaving's delete again.
const TRIES_PREFIX: &str = "tries=";

/// The most bytes of a leaving record this reads.
/// What bounds it:
/// three short lines and the JSON of one work item,
/// which names three paths,
/// and a path is at most the kernel's `PATH_MAX`,
/// 4096 bytes.
pub(crate) const RECORD_LIMIT: u64 = 16384;

/// What:
///  One leaving,
/// as its record holds it:
///  the command that renamed the subvolume,
///  how many times a command has queued its delete again,
///  and the work item whose path and object name the delete acts on.
/// Why:
///   The item's path is the place the rename made and its object name is the subvolume's own id,
///  which together are the proof a delete needs.
///   The owner says whether a command may still be between its record and its rename,
///  which is the one state a later command may not decide about.
///   The count is what keeps one stuck leaving from costing this machine's commands a delete attempt for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Leaving = { owner?: string; tries: number; item: WorkItem };
/// ```
pub struct Leaving {
    /// The command that wrote the record,
    /// as `<pid>:<birth identity>`;
    /// nothing for a record written without one.
    pub owner: Option<String>,
    /// How many times a command has queued this leaving's delete again.
    pub tries: u32,
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
///  the count of queueings,
///  and the work item.
/// Why:
///   The two short lines come first,
///  one per line,
///  so nothing a path or a repository name holds can be taken for them,
///  and everything after them is the item's own JSON.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = (item, owner, tries) => `${owner ? 'owner=' + owner + '\n' : ''}tries=${tries}\n${item.encode()}`;
/// ```
fn bytes_of_the_leaving(item: &WorkItem, owner: Option<&str>, tries: u32) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    if let Some(spelled) = owner {
        bytes.extend_from_slice(format!("{OWNER_PREFIX}{spelled}\n").as_bytes());
    }
    bytes.extend_from_slice(format!("{TRIES_PREFIX}{tries}\n").as_bytes());
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
fn write_the_leaving(common_dir: &Path, item: &WorkItem, owner: Option<&str>, tries: u32) -> Result<(), String> {
    let folder: PathBuf = common_dir.join(LEAVINGS_FOLDER);
    if let Err(error) = create_private_directory_all(folder.as_path()) {
        return Err(format!("the folder {} cannot be made ({error})", folder.display()));
    }
    let path: PathBuf = record_path(common_dir, item.id().as_str());
    return match replace_private_file_unsynced(path.as_path(), bytes_of_the_leaving(item, owner, tries).as_slice()) {
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
    write_the_leaving(common_dir, item, owner, 0)?;
    return Ok(Leaving { owner: owner.map(str::to_owned), tries: 0, item: item.clone() });
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
    // The folder goes with the last record, as the record of a removal's destroy does; one that still holds another record
    // stays, which its own failure says.
    let _kept = std::fs::remove_dir(common_dir.join(LEAVINGS_FOLDER));
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leaving = readTheLeaving(bytes); // throws on a line that is no line, a count that is no count, an item that is no item
/// ```
pub(crate) fn read_the_leaving(bytes: &[u8]) -> Result<Leaving, String> {
    let mut rest: &[u8] = bytes;
    let mut owner: Option<String> = None;
    let mut tries: u32 = 0;
    loop {
        let Some(split_at) = rest.iter().position(is_newline) else { break };
        let (line, after): (&[u8], &[u8]) = rest.split_at(split_at);
        let Ok(text) = std::str::from_utf8(line) else { break };
        if let Some(spelled) = text.strip_prefix(OWNER_PREFIX) {
            owner = Some(spelled.to_owned());
            rest = after.strip_prefix(b"\n").unwrap_or(after);
            continue;
        }
        match text.strip_prefix(TRIES_PREFIX) {
            Some(spelled) => {
                let Ok(count) = spelled.parse::<u32>() else {
                    return Err(format!("its count {spelled} is no count of tries"));
                };
                tries = count;
                rest = after.strip_prefix(b"\n").unwrap_or(after);
            }
            None => break,
        }
    }
    let item: &str = match std::str::from_utf8(rest) {
        Ok(text) => text,
        Err(_not_text) => return Err(String::from("the record of the delete it names is not text")),
    };
    return Ok(Leaving { owner, tries, item: WorkItem::decode(item)? });
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
    if requeue_after_giving_up(work.as_path(), id.as_str()) {
        note_the_attempt(common_dir, leaving);
        return Ok(());
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
    let _settled_done = std::fs::remove_file(state_directory(work.as_path(), DONE).join(id.as_str()));
    let _settled_failed = std::fs::remove_file(state_directory(work.as_path(), FAILED).join(id.as_str()));
    note_the_attempt(common_dir, leaving);
    return Ok(());
}

/// What:
///  Write one more queueing into the leaving's own record.
/// Why:
///   The count is what bounds a delete the kernel keeps refusing:
///  a command that finds the count spent says the directory and stops queueing it
///  (`worktree_removal_leavings_a_command_queues_again.rs`).
///   A count that cannot be written costs one more attempt and nothing else,
///  which is why the queueing stands whether this write does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rewriteLeaving({ ...leaving, tries: leaving.tries + 1 });
/// ```
fn note_the_attempt(common_dir: &Path, leaving: &Leaving) {
    let tries: u32 = leaving.tries.saturating_add(1);
    if let Err(reason) = write_the_leaving(common_dir, &leaving.item, leaving.owner.as_deref(), tries) {
        debug(TAG, format!("{reason}; the delete of {} is queued, and the count of its attempts is not", leaving.item.parent).as_str());
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_leaving_that_outlives_its_record_tests.rs"]
mod tests;
