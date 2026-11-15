//! What:
//!  The record of a branch's move to a published merge that no worktree had checked out,
//!  and the command that finishes the checkout a stop left behind.
//! Why:
//!  A branch can be moved without its checkout only through a compare-and-swap,
//! and a worktree may take the branch in the instant before that write.
//!  A process stopped after the write leaves the branch at the merge and that checkout holding the
//! pushed tip's index and files, while its `HEAD` resolves to the merge:
//! a commit made there records the older files, a reversal of the published merge.
//!  So the move writes this record **before** it writes the reference,
//! and any command in the worktree that holds the branch finishes the checkout the record names.
//!  The record lives in the repository's common Git directory,
//! because the worktree that takes the branch is not known when the record is written,
//! and every worktree's command reads that directory.
//!
//! The steps of an advance, what each proves, what record exists at each, and what a stop after each
//! leaves:
//!
//! 1. The first look at the worktree listing (`live_holder_awaiting`):
//!    proves who, if anyone, has the branch checked out.
//!    No record yet.
//!    A stop leaves nothing done;
//!    a failed listing ends the advance here, because "the question failed" is never read as
//!    "nobody holds it".
//! 2. The tip read (`rev-parse <branch>`): proves the branch still stands where the push sent it.
//!    No record.
//!    A stop leaves nothing done.
//! 3. This record is written and made durable in the common Git directory.
//!    Proves that a move from `oldOid` to `newOid` may be under way.
//!    A stop leaves the branch where it was and the record behind;
//!    the next command sees the branch is not at `newOid`, so it removes the record and changes nothing.
//! 4. The reference's compare-and-swap (`update-ref <branch> <new> <old>`): proves no other writer
//!    moved the branch since step 2.
//!    The record exists.
//!    A refused write removes the record.
//!    A stop leaves the branch where it was (Git's own atomicity) and the record behind;
//!    the next command removes it, because the branch is not at `newOid`.
//! 5. The look after the write: proves who holds the branch now.
//!    The record exists.
//!    A stop leaves the branch at the merge and a checkout that took it in the window holding the
//!    pushed tip's files; the next command in that worktree finishes it.
//!    A listing that fails here keeps the record and says so.
//! 6. The repair (`bring_the_checkout_to_its_branch_awaiting`): writes, over the paths this move
//!    wrote, only the content its own write can prove is the old tip's or the merge's.
//!    The record exists.
//!    A stop leaves some paths written and the record behind;
//!    the next command in that worktree finishes the rest.
//! 7. The record is removed once the repair has run.
//!    A stop before the removal leaves everything repaired and the record behind;
//!    the next command finds the checkout agrees with its branch, writes nothing, and removes it.
//!
//! What finishes each stop is the same command (`finish_branch_moves_awaiting` in
//! `auto_push_branch_move_repairs.rs`),
//! run by every wrapper command in a worktree before its own work,
//! and by the advance itself right after the write;
//! what it is allowed to write, who may run it and when a record may go are that file's own words.
//!
//! When a record is finished, and who may finish it:
//!
//!  A record is finished when every worktree that has its branch checked out shows the tip the branch was
//!  moved to, in its index and in its files, or holds changes that are a person's own, which are left to
//!  them and named.
//!  No single comparison answers that, and each one alone was a defect found in this work:
//!  the branch tip alone is not the answer, because the tip moves before any checkout does;
//!  the index alone is not, because a `git reset` moves the index without the files
//!  (a review of commit `cae78e1d7e`, its finding 3);
//!  and one path is not, because a checkout can hold the merge's content on one path and the old tip's
//!  on another.
//!  So the checkout is asked all three: the branch it shows and that branch's tip, the index against
//!  that tip, and every path of the move in the worktree.
//!  A path of the move that holds what the old tip held is this record's own leftover and is brought to
//!  the branch's entry;
//!  a path that holds neither the old tip's entry nor the merge's is a person's and is left as it is,
//!  and the refusal names it.
//!  The record carries both tips (`old_oid`, `new_oid`) for exactly this: `old_oid` is what tells a
//!  leftover of the move from a change of the person's on a path where the move wrote nothing.
//!
//!  Who may finish it: any later command in a worktree that has the branch checked out, once the process
//!  that wrote the record is gone or has said it left the record (`owner_left_it`).
//!  A live writer says only that it may be repairing this checkout right now,
//!  and that is a moment inside the advance, not the record's whole life:
//!  the resident service outlives every record it writes,
//!  so a writer that keeps its record and goes on to other work marks it,
//!  and the next command finishes it however long that writer lives
//!  (a review of commit `cae78e1d7e`, its finding 2).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = await writeBranchMove({ commonDir, owner, targetRef, oldOid, newOid });
//! // ... compare-and-swap the reference, repair the checkout ... // then remove(record)
//! ```

use crate::auto_push_single_flight::{PushGit, SingleFlightError, run_query_awaiting};
use crate::auto_push_reconcile_basics_and_limits::argv;
use crate::json_record::{ObjectWriter, decode_fatal, parse_object, safe_integer_field, string_field};
use crate::private_storage::{create_private_directory, sync_directory, write_exclusive_synced};
use crate::transaction_owner::TransactionOwner;
use std::path::{Path, PathBuf};

/// Directory,
/// inside the repository's common Git directory,
/// that holds the records of moves that no worktree had checked out.
pub const BRANCH_MOVE_DIRECTORY: &str = "monochromatic-git-policy-cli-branch-moves";

/// The `state` a record of this kind names itself with.
const RECORD_STATE: &str = "branch-move";

/// Schema version of a record.
pub const BRANCH_MOVE_SCHEMA_VERSION: i64 = 1;

/// Suffix of a record's file name.
const RECORD_SUFFIX: &str = ".json";

/// What:
///  One move of a branch that no worktree had checked out:
/// who ran it,
/// which branch,
/// and the two commits it joins.
/// Why:
///   The command that finishes a stopped move needs exactly these:
/// the branch to find a holder of,
/// the tip that holder's checkout may still hold,
/// and the merge it must hold instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BranchMove = { owner: TransactionOwner; ownerLeftIt: boolean; targetRef: string; oldOid: string; newOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchMove {
    /// The process that wrote the record,
    /// whose death is what tells a later command the move is finished or stopped.
    pub owner: TransactionOwner,
    /// Whether that process is alive and has gone on to other work,
    /// so any later command may finish this record.
    /// Why:
    ///  A live writer may be repairing this checkout right now, which is why liveness alone stops a
    ///  later command;
    /// but a writer that kept the record and went on marks it here,
    /// and the resident service outlives every record it writes.
    pub owner_left_it: bool,
    /// The branch that was moved.
    pub target_ref: String,
    /// The tip the branch held before the move.
    pub old_oid: String,
    /// The merge the branch was moved to.
    pub new_oid: String,
}

/// The text of one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// encodeBranchMove(move) // one JSON object and a line feed
/// ```
pub fn encode_branch_move(movement: &BranchMove) -> String {
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", BRANCH_MOVE_SCHEMA_VERSION)
        .string("state", RECORD_STATE)
        .string("transactionId", movement.owner.transaction_id.as_str())
        .integer("ownerPid", movement.owner.owner_pid)
        .string("ownerIdentity", movement.owner.owner_identity.as_str())
        .string("createdAt", movement.owner.created_at.as_str());
    if let Some(namespace) = &movement.owner.owner_pid_namespace {
        writer.string("ownerPidNamespace", namespace.as_str());
    }
    if movement.owner_left_it {
        writer.boolean("ownerLeftIt", true);
    }
    writer
        .string("targetRef", movement.target_ref.as_str())
        .string("oldOid", movement.old_oid.as_str())
        .string("newOid", movement.new_oid.as_str());
    return format!("{}\n", writer.finish());
}

/// What:
///  Parse one record,
/// or nothing when the bytes are not one this release reads.
/// Why:
///   A record that cannot be read is never guessed at:
/// the command that meets one keeps it and names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseBranchMove(bytes) // BranchMove | undefined
/// ```
pub fn parse_branch_move(bytes: &[u8]) -> Option<BranchMove> {
    let map: serde_json::Map<String, serde_json::Value> = parse_object(decode_fatal(bytes)?)?;
    if safe_integer_field(&map, "schemaVersion")? != BRANCH_MOVE_SCHEMA_VERSION || string_field(&map, "state")? != RECORD_STATE {
        return None;
    }
    let owner_pid: i64 = safe_integer_field(&map, "ownerPid")?;
    let owner_identity: &str = string_field(&map, "ownerIdentity")?;
    let target_ref: &str = string_field(&map, "targetRef")?;
    let old_oid: &str = string_field(&map, "oldOid")?;
    let new_oid: &str = string_field(&map, "newOid")?;
    if owner_pid < 1 || owner_identity.is_empty() || target_ref.is_empty() || old_oid.is_empty() || new_oid.is_empty() {
        return None;
    }
    return Some(BranchMove {
        owner_left_it: matches!(map.get("ownerLeftIt"), Some(serde_json::Value::Bool(true))),
        owner: TransactionOwner {
            transaction_id: String::from(string_field(&map, "transactionId")?),
            owner_pid,
            owner_identity: String::from(owner_identity),
            created_at: String::from(string_field(&map, "createdAt")?),
            owner_pid_namespace: match string_field(&map, "ownerPidNamespace") {
                Some(namespace) if !namespace.is_empty() => Some(String::from(namespace)),
                _ => None,
            },
        },
        target_ref: String::from(target_ref),
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
    });
}

/// The directory that holds a repository's records of these moves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = join(commonDir, BRANCH_MOVE_DIRECTORY);
/// ```
fn move_directory(common_dir: &Path) -> PathBuf {
    return common_dir.join(BRANCH_MOVE_DIRECTORY);
}

/// What:
///  Write a record of a move durably:
/// the private directory,
/// the exclusive synced file,
/// the synced directory.
/// Why:
///   The reference may be moved only after the record would survive a crash,
/// so a stop at any later instruction leaves something a later command reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory); await writeExclusiveSynced(path, text); await syncDirectory(directory);
/// ```
pub fn write_branch_move(common_dir: &Path, movement: &BranchMove) -> std::io::Result<PathBuf> {
    let directory: PathBuf = move_directory(common_dir);
    match create_private_directory(directory.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let path: PathBuf = directory.join(format!("{}{RECORD_SUFFIX}", movement.owner.transaction_id));
    write_exclusive_synced(path.as_path(), encode_branch_move(movement).as_bytes())?;
    sync_directory(directory.as_path())?;
    return Ok(path);
}

/// What:
///  Say that the process that wrote a record is alive and has gone on to other work,
/// by publishing the record again with `ownerLeftIt` set.
/// Why:
///   Liveness alone stops a later command, because a live writer may be repairing this checkout
///   right now;
/// that is a moment inside the advance, and the resident service outlives every record it writes,
///  so a writer that keeps its record and goes on says so, and anyone may then finish it
///  (a review of commit `cae78e1d7e`, its finding 2).
///  The record is published over its name in one rename, so a kill leaves either the record that was
///  there or the complete new one, and either is a record a later command reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await replacePrivateFile(record, encode({ ...movement, ownerLeftIt: true }));
/// ```
pub fn leave_the_branch_move_to_anyone(record: &Path, movement: &BranchMove) -> std::io::Result<()> {
    let left: BranchMove = BranchMove {
        owner_left_it: true,
        ..movement.clone()
    };
    return crate::private_storage::replace_private_file(record, encode_branch_move(&left).as_bytes());
}

/// What:
///  Remove a record once what it names is finished,
/// without flushing the directory.
/// Why:
///   A record that outlives its move is read by every later command;
/// one whose branch is not at the merge is removed on the next command instead,
/// so a removal that a stop or a failed write lost costs one read and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { force: true });
/// ```
pub fn remove_branch_move(path: &Path) -> std::io::Result<()> {
    return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    };
}

/// What:
///  The records of this repository's moves,
/// oldest name first,
/// or none when the directory does not exist.
/// Why:
///   The common case,
/// every command of a repository that moves no branch this way,
/// costs one failed directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(directory).catch(() => [])).filter((name) => name.endsWith('.json')).sort().map(join)
/// ```
pub fn list_branch_moves(common_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let directory: PathBuf = move_directory(common_dir);
    let listing = match std::fs::read_dir(directory.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut records: Vec<PathBuf> = Vec::new();
    for entry in listing {
        let read = entry?;
        let name: std::ffi::OsString = read.file_name();
        if name.to_string_lossy().ends_with(RECORD_SUFFIX) {
            records.push(read.path());
        }
    }
    records.sort();
    return Ok(records);
}

/// What:
///  Whether the repository holds a record of a move a command may have to finish.
/// Why:
///   The gate reads this first,
/// so a repository that never moves a branch this way pays one failed directory read per command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync(directory)
/// ```
pub fn branch_moves_exist(common_dir: &Path) -> bool {
    return move_directory(common_dir).is_dir();
}

/// What:
///  The repository's common Git directory,
/// which is where these records live.
/// Why:
///   The worktree that takes the branch is not known when the record is written,
/// and every worktree's command reads this one directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['rev-parse', '--path-format=absolute', '--git-common-dir'])
/// ```
pub async fn common_git_dir_awaiting(git: &PushGit) -> Result<PathBuf, SingleFlightError> {
    let printed = run_query_awaiting(git, &argv(&["rev-parse", "--path-format=absolute", "--git-common-dir"])).await?;
    if printed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("rev-parse --path-format=absolute --git-common-dir"),
            code: printed.code,
            stderr: String::from_utf8_lossy(&printed.stderr).into_owned(),
        });
    }
    let bytes: &[u8] = printed.stdout.strip_suffix(b"\n".as_slice()).unwrap_or(printed.stdout.as_slice());
    return Ok(PathBuf::from(crate::path_bytes::os_from_path_bytes(bytes)));
}

/// What:
///  Leave the record of a move,
/// or nothing when it could not be written.
/// Why:
///   The reference is moved only after its record is durable,
/// so a stop after the write leaves something a later command reads;
/// the caller refuses the move when this answers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = writeBranchMove(commonDir, { owner, targetRef, oldOid, newOid }); // undefined when it could not be written
/// ```
pub fn record_branch_move(common_dir: &Path, target_ref: &str, old_oid: &str, new_oid: &str) -> Option<(PathBuf, BranchMove)> {
    let owner: TransactionOwner =
        crate::transaction_owner::current_transaction_owner(crate::random_id::random_uuid().ok()?.as_str(), crate::iso_time::now_iso().as_str())
            .ok()?;
    let movement: BranchMove = BranchMove {
        owner,
        // The writer has not left it: it is about to move the branch and repair the checkout.
        owner_left_it: false,
        target_ref: target_ref.to_owned(),
        old_oid: old_oid.to_owned(),
        new_oid: new_oid.to_owned(),
    };
    let path: PathBuf = write_branch_move(common_dir, &movement).ok()?;
    return Some((path, movement));
}

/// What:
///  The record of a move that this command has written and not yet finished.
/// Why:
///  A record left by a command that stopped is what a later command repairs from,
/// so every way out of the move must leave it readable:
///  the exit that finishes the repair removes it,
/// the exit that leaves the checkout to a later command hands it on,
/// and every other exit, an error return as much as a killed instruction, hands it on from `Drop` by
/// the value itself.
///  A guard is what makes the forgotten exit impossible to write: the record is a value the function
/// owns, and no return path can drop it without the `Drop` writing `ownerLeftIt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AMoveInHand { [Symbol.dispose]() { if (!this.done) leaveTheBranchMoveToAnyone(this.record) } }
/// ```
pub struct AMoveInHand {
    /// Where the record is.
    record: PathBuf,
    /// What it says.
    movement: BranchMove,
    /// Whether this command has finished with it.
    done: bool,
}

impl AMoveInHand {
    /// What:
    ///  Write the record of a move from `old_oid` to `new_oid`, durably, before the reference moves.
    /// Why:
    ///   Without a record nothing may move: a stop after the reference's write would leave a checkout
    /// nothing finishes
    ///  (a review of commit `36cae084ae`, its finding 3).
    pub fn take(common_dir: &Path, target_ref: &str, old_oid: &str, new_oid: &str) -> Option<AMoveInHand> {
        let (record, movement) = record_branch_move(common_dir, target_ref, old_oid, new_oid)?;
        return Some(AMoveInHand {
            record,
            movement,
            done: false,
        });
    }

    /// Where this move's record is.
    pub fn record_path(&self) -> &Path {
        return self.record.as_path();
    }

    /// What the record says.
    pub fn movement(&self) -> &BranchMove {
        return &self.movement;
    }

    /// What:
    ///  This command finished what the record named: the checkout it names was repaired, or the move
    ///  never happened, so the record goes.
    /// Why:
    ///   A record that outlives its move tells a later command to write a checkout that is already right.
    pub fn finish(&mut self) {
        let _removed = remove_branch_move(self.record.as_path());
        self.done = true;
    }

    /// What:
    ///  This command is going on and leaves the checkout to a later command, so the record says so.
    /// Why:
    ///   A later command must be able to finish it whatever this process does next,
    /// and this process outlives the move.
    pub fn hand_on(&mut self) {
        self.hand_on_quietly();
        self.done = true;
    }

    /// `hand_on` without the state, which `Drop` also uses.
    fn hand_on_quietly(&self) {
        let _renamed = leave_the_branch_move_to_anyone(self.record.as_path(), &self.movement);
    }
}

impl Drop for AMoveInHand {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        // Every exit that is not `finish` or `hand_on` leaves the record for a later command, by the value itself.
        self.hand_on_quietly();
    }
}
