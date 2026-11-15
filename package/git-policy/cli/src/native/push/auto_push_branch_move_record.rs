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
//! What finishes each stop is the same command (`finish_branch_moves_awaiting`),
//! run by every wrapper command in a worktree before its own work,
//! and by the advance itself right after the write.
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
use std::io::Write;
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

/// How long a command waits for another command that is repairing this checkout before it goes on.
/// Why:
///   A repair of one checkout is a handful of Git starts, and the machine's own filesystem stalls for seconds
/// under another session's load:
/// the arm without a window measured in this run's report puts a repair at a few milliseconds, and this is a
/// thousandfold that, so a wait that runs out names a repair that is not merely slow.
pub const REPAIR_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

/// How long a waiting command pauses between two looks at the repair's owner record.
/// Why:
///   One look costs the sweep's own Git starts (a few milliseconds), so a pause this long fits twenty attempts
/// inside `REPAIR_WAIT` and each of them is a real question rather than a busy loop.
pub const REPAIR_PAUSE: std::time::Duration = std::time::Duration::from_millis(250);

/// Suffix of the name of the one record that says which command is repairing this worktree's checkout.
/// Why:
///   One checkout is repaired by one command at a time, and a command about to commit must not fail on
/// Git's index lock because the wrapper itself started a second write.
pub const REPAIR_OWNER_SUFFIX: &str = "-repair-owner.json";

/// The name of that record, inside the worktree's own Git directory.
const REPAIR_OWNER_FILE: &str = "monochromatic-git-policy-cli-repair-owner.json";

/// What:
///  The place of one repair of a checkout.
/// Why:
///   It is taken before the repair writes and given back when the repair ends or is given up on,
/// so a command that finds it held by a live command waits instead of writing beside it,
///  and a repair whose command was killed leaves a record whose owner is dead, which the next command
/// takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ARepairInHand { [Symbol.dispose]() { rmSync(this.path, { force: true }) } }
/// ```
pub struct ARepairInHand {
    /// The record's place.
    path: PathBuf,
    /// Whether it was given back already.
    done: bool,
}

impl ARepairInHand {
    /// The record's place, so a later command may judge the owner it names.
    pub fn record_path(&self) -> &Path {
        return self.path.as_path();
    }

    /// Give the place back: the repair this command ran is over.
    pub fn finish(&mut self) {
        let _removed: std::io::Result<()> = std::fs::remove_file(self.path.as_path());
        self.done = true;
    }

    /// Give the place back although the record stays for a later command: this command's repair is over too,
    /// and the record says the checkout it names is still to be finished.
    pub fn hand_on(&mut self) {
        self.finish();
    }
}

impl Drop for ARepairInHand {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        let _removed: std::io::Result<()> = std::fs::remove_file(self.path.as_path());
    }
}

/// What:
///  Take this worktree's repair place,
/// or nothing when a record is already there.
/// Why:
///   The place is one file created exclusively, which is the lock-free way to give one repair to one
/// command: a command that finds it waits or takes it when its owner is dead,
/// and no lock is taken on anything a person's own Git uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = writeExclusiveSynced(join(gitDir, REPAIR_OWNER_FILE), encode(owner)); // undefined when it exists
/// ```
pub fn the_repair_owner(git_dir: &Path, owner: &TransactionOwner) -> Option<ARepairInHand> {
    let path: PathBuf = git_dir.join(REPAIR_OWNER_FILE);
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", BRANCH_MOVE_SCHEMA_VERSION)
        .string("state", "repair-owner")
        .string("transactionId", owner.transaction_id.as_str())
        .integer("ownerPid", owner.owner_pid)
        .string("ownerIdentity", owner.owner_identity.as_str())
        .string("createdAt", owner.created_at.as_str());
    let text: String = format!("{}\n", writer.finish());
    write_exclusive_synced(path.as_path(), text.as_bytes()).ok()?;
    return Some(ARepairInHand { path, done: false });
}

/// What:
///  Take a repair place whose owner is gone,
/// by removing its record and taking the place again.
/// Why:
///   A repair whose command was killed leaves its record; the next command must be able to take it
/// without a person's help,
///  and two commands doing that at once both remove the same name harmlessly and one of them wins the
/// exclusive creation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rmSync(path, { force: true }); return writeExclusiveSynced(path, bytes);
/// ```
pub fn the_repair_owner_after_its_owner_died(git_dir: &Path, owner: &TransactionOwner, dead: &Path) -> Option<ARepairInHand> {
    let _removed: std::io::Result<()> = std::fs::remove_file(dead);
    return the_repair_owner(git_dir, owner);
}

/// What:
///  Give a repair place back without running the rest of the repair,
/// which a control and a command that gives the work up both need.
/// Why:
///   The record is a file and nothing else:
/// whoever holds it gives it back by removing it, and a name that is already gone is a place already free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rmSync(path, { force: true });
/// ```
pub fn leave_the_repair_to_anyone(path: &Path) -> std::io::Result<()> {
    return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    };
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

/// What:
///  Finish every record of this repository whose owner is gone,
/// for the worktree this command runs in,
/// and say what was left when a checkout could not be brought to its branch.
/// Why:
///   A stopped move names a branch and a merge;
/// the command that reads it finishes the checkout of the worktree that holds the branch,
///  and only that worktree:
/// another worktree's checkout is its own command's to write,
///  so a record whose branch another worktree holds is left for it.
/// A record whose branch is not at the merge is one whose move never happened,
/// and one no worktree holds has no checkout to finish:
/// both are removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const record of await listBranchMoves(commonDir)) { if (ownerAlive(record)) continue; const holder = await liveHolder(git, record.targetRef); ... }
/// ```
pub async fn finish_branch_moves_awaiting<W: Write + ?Sized>(
    git: &PushGit,
    common_dir: &Path,
    out: &mut W,
) -> Result<Settled, String> {
    let records: Vec<PathBuf> = list_branch_moves(common_dir).map_err(|error| format!("the branch moves of {} could not be listed: {error}", common_dir.display()))?;
    if records.is_empty() {
        return Ok(Settled::Yes);
    }
    // What: the branch this worktree shows, read from the file Git keeps for it.
    // Why: a record of another branch is another worktree's to finish, and reading one file is what that
    //      costs: no Git is started for the ordinary case of a repository with a record it does not own
    //      (a review of commit `4ca7aebcdb`, its finding 4).
    let Some(git_dir) = the_git_directory_of(git) else {
        return Ok(Settled::Yes);
    };
    let Some(shown) = the_branch_this_worktree_shows(git_dir.as_path()) else {
        return Ok(Settled::Yes);
    };
    let mut settled: Settled = Settled::Yes;
    for record in records {
        let bytes: Vec<u8> = match std::fs::read(record.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("the branch move record {} could not be read: {error}", record.display())),
        };
        let Some(movement) = parse_branch_move(bytes.as_slice()) else {
            return Err(format!(
                "The branch move record {} is malformed. It was left in place; if no push is running, remove it, and check the branch and `git status` before pushing again.",
                record.display()
            ));
        };
        if movement.target_ref != shown {
            // Another worktree shows this branch, and only that worktree's own commands write its checkout.
            continue;
        }
        let asked = run_query_awaiting(git, &argv(&["rev-parse", "--verify", format!("{}^{{commit}}", movement.target_ref).as_str()])).await;
        let tip: String = match asked {
            Ok(printed) if printed.code == Some(0) => String::from_utf8_lossy(&printed.stdout).trim().to_owned(),
            _ => {
                debug_line(out, &format!("the tip of {} could not be read, so the record {} is left", movement.target_ref, record.display()));
                settled = Settled::Unknown {
                    why: format!("the tip of {} could not be read", movement.target_ref),
                };
                continue;
            }
        };
        if tip != movement.new_oid {
            // What: the branch is not at the tip this record names, so its move is void.
            // Why: that is the proof a later command needs, read at the moment the record goes
            //      (a review of commit `4ca7aebcdb`, its finding 4).
            let _removed = remove_branch_move(record.as_path());
            continue;
        }
        // What: one repair of this checkout at a time, taken by one exclusively created record.
        // Why: two repairs at once put two writes on one index, and the second would fail on Git's own lock,
        //      which a command about to commit must not do
        //      (a review of commit `4ca7aebcdb`, its finding 2).
        let Some(mut repair) = this_commands_repair_of_the_checkout(git_dir.as_path(), out) else {
            settled = Settled::AnotherCommandAtWork;
            continue;
        };
        let brought = crate::auto_push_reconcile_branch_checkout::bring_the_checkout_to_its_branch_awaiting(
            git,
            movement.target_ref.as_str(),
            movement.old_oid.as_str(),
            movement.new_oid.as_str(),
        )
        .await;
        match brought {
            Ok(crate::auto_push_reconcile_branch_checkout::Brought::Right) => {
                repair.finish();
                let _removed = remove_branch_move(record.as_path());
                // What: a copy of the line the merge recovery prints, for a state change the person did not ask for.
                // Why: a push that stopped left this checkout holding the older files, and an unrelated command may have
                //      written the merge into it; a person who learns of it only by a changed file would be right to ask
                //      what wrote it.
                let _noted = writeln!(
                    out,
                    "monochromatic-git-policy-cli: the checkout at this worktree now holds {branch} at {merge}, which a push that stopped \
                     before it finished that checkout left behind; the edits you made elsewhere are kept.",
                    branch = movement.target_ref,
                    merge = movement.new_oid
                );
            }
            Ok(crate::auto_push_reconcile_branch_checkout::Brought::Left { paths, reason, write_refused }) => {
                let old_tip_left: bool = left_the_old_tip(git, movement.old_oid.as_str(), paths.as_slice()).await?;
                if write_refused || old_tip_left {
                    // What: a path the record's move wrote still holds what the branch moved from, so the record stays.
                    // Why: a commit here would record that tree over the published merge, and the next command may
                    //      finish the write
                    //      (a review of commit `4ca7aebcdb`, findings 2 and 4).
                    repair.hand_on();
                    debug_line(out, &format!("the checkout here was not written ({reason}), so the record {} is left", record.display()));
                    settled = Settled::OldTipLeft {
                        branch: movement.target_ref.clone(),
                        paths: paths.clone(),
                    };
                    continue;
                }
                // The repair has run and told what it left; a record kept would only tell it again at every command.
                repair.finish();
                let _removed = remove_branch_move(record.as_path());
                let _noted = writeln!(
                    out,
                    "monochromatic-git-policy-cli: the checkout at this worktree still differs from {} on {} and was left as it is ({reason}); \
                     a commit there would record those files.",
                    movement.target_ref,
                    crate::auto_push_dirty_paths::listed(&paths)
                );
            }
            Err(error) => {
                repair.hand_on();
                debug_line(out, &format!("the checkout here was not brought to {} ({error}), so the record {} is left", movement.target_ref, record.display()));
                settled = Settled::Unknown {
                    why: format!("the checkout could not be read: {error}"),
                };
            }
        }
    }
    return Ok(settled);
}

/// What:
///  Whether any of `paths` still holds what the branch moved from,
/// asked again after the repair ran.
/// Why:
///   The answer the startup gate needs is whether a commit here could record the tree the record exists to
/// prevent,
///  and that is exactly this question, asked at the moment it is answered
///  (a review of commit `4ca7aebcdb`, its finding 2).
async fn left_the_old_tip(git: &PushGit, old_oid: &str, paths: &[Vec<u8>]) -> Result<bool, String> {
    if paths.is_empty() {
        return Ok(true);
    }
    let differing = crate::auto_push_reconcile_branch_checkout_files::the_paths_the_checkout_disagrees_with_its_branch_awaiting(git, paths)
        .await
        .map_err(|error| format!("the files of the checkout could not be read: {error}"))?;
    if differing.is_empty() {
        return Ok(false);
    }
    let holding = crate::auto_push_reconcile_branch_checkout_files::the_paths_still_holding_the_old_tip_awaiting(git, old_oid, differing.as_slice())
        .await
        .map_err(|error| format!("the files of the checkout could not be read: {error}"))?;
    return Ok(!holding.is_empty());
}

/// What:
///  This worktree's own Git directory,
/// read from the `.git` entry at its root.
/// Why:
///   The command runs at the worktree's root (the wrapper refuses any other directory),
/// and `.git` there is the directory itself, or a file naming one for a linked worktree:
///  reading it costs one file and no Git start, which is what a record of another branch must cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entry = join(cwd, '.git'); const meta = statSync(entry); if (meta.isDirectory()) return entry; const named = readFileSync(entry,'utf8').trim().slice('gitdir: '.length); return resolve(cwd, named);
/// ```
fn the_git_directory_of(git: &PushGit) -> Option<PathBuf> {
    let entry: PathBuf = git.cwd.join(".git");
    match std::fs::metadata(entry.as_path()) {
        Ok(meta) if meta.is_dir() => return Some(entry),
        Ok(_) => {}
        Err(_) => return None,
    }
    let text: String = std::fs::read_to_string(entry.as_path()).ok()?;
    let named: &str = text.trim().strip_prefix("gitdir: ")?;
    let directory: PathBuf = PathBuf::from(named);
    return if directory.is_absolute() { Some(directory) } else { Some(git.cwd.join(directory)) };
}

/// What:
///  The branch this worktree shows,
/// read from the `HEAD` file of its own Git directory.
/// Why:
///   A record of another branch is another worktree's, and a command in a repository with a record must not
/// start a Git just to learn that
///  (a review of commit `4ca7aebcdb`, its finding 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = readFileSync(join(gitDir, 'HEAD'), 'utf8').trim(); return head.startsWith('ref: ') ? head.slice(5) : undefined;
/// ```
fn the_branch_this_worktree_shows(git_dir: &Path) -> Option<String> {
    let head: String = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let named: &str = head.trim().strip_prefix("ref: ")?;
    return if named.starts_with("refs/heads/") { Some(String::from(named)) } else { None };
}

/// What:
///  This command's place at repairing this worktree's checkout,
/// or nothing when another live command holds it.
/// Why:
///   The place is one file created exclusively, and its owner is judged with the same liveness the
/// records use:
///  a place whose owner is dead is taken by the next command, and one whose owner lives is waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = takeRepairOwner(gitDir); if (taken) return taken; const owner = ownerNamedBy(gitDir); return owner && isDead(owner) ? takeAfterDeath(gitDir, owner.path) : undefined;
/// ```
fn this_commands_repair_of_the_checkout<W: Write + ?Sized>(git_dir: &Path, out: &mut W) -> Option<ARepairInHand> {
    let owner: TransactionOwner =
        match crate::transaction_owner::current_transaction_owner(crate::random_id::random_uuid().ok()?.as_str(), crate::iso_time::now_iso().as_str()) {
            Ok(owner) => owner,
            Err(error) => {
                debug_line(out, &format!("this command cannot name itself as the repair's owner: {error}"));
                return None;
            }
        };
    let place: PathBuf = git_dir.join(REPAIR_OWNER_FILE);
    if let Some(taken) = the_repair_owner(git_dir, &owner) {
        return Some(taken);
    }
    let Some(theirs) = the_owner_named_by(place.as_path()) else {
        // A record that cannot be read is not taken over: it may name a live command.
        debug_line(out, &format!("the repair owner's record {} could not be read, so this command does not repair the checkout", place.display()));
        return None;
    };
    match crate::transaction_owner::classify_transaction_owner_record(&theirs, place.as_path()) {
        Ok(crate::transaction_owner::OwnerLiveness::Dead) => return the_repair_owner_after_its_owner_died(git_dir, &owner, place.as_path()),
        // A probe that cannot be made reads as alive, as it does for the merge records.
        Ok(crate::transaction_owner::OwnerLiveness::Alive) | Err(_) => return None,
    }
}

/// What:
///  The owner a repair's record names,
/// or nothing when the record is not one this release reads.
/// Why:
///   A place whose owner is dead may be taken, and that question is answered by the record's own fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const map = parseObject(readFileSync(path, 'utf8')); return map ? { transactionId: stringField(map,'transactionId'), ownerPid: integerField(map,'ownerPid'), ... } : undefined;
/// ```
fn the_owner_named_by(place: &Path) -> Option<TransactionOwner> {
    let text: String = std::fs::read_to_string(place).ok()?;
    let map: serde_json::Map<String, serde_json::Value> = parse_object(text.as_str())?;
    if safe_integer_field(&map, "schemaVersion")? != BRANCH_MOVE_SCHEMA_VERSION || string_field(&map, "state")? != "repair-owner" {
        return None;
    }
    return Some(TransactionOwner {
        transaction_id: String::from(string_field(&map, "transactionId")?),
        owner_pid: safe_integer_field(&map, "ownerPid")?,
        owner_identity: String::from(string_field(&map, "ownerIdentity")?),
        created_at: String::from(string_field(&map, "createdAt")?),
        owner_pid_namespace: match string_field(&map, "ownerPidNamespace") {
            Some(namespace) if !namespace.is_empty() => Some(String::from(namespace)),
            _ => None,
        },
    });
}

/// What:
///  What the sweep came to for the checkout this command is about to run in.
/// Why:
///  The startup gate clears a command on this answer,
/// and a clear that leaves a path of the move holding what the branch moved from admits a commit that
/// records it:
///  so the answer is about that question and not about whether a repair ran
///  (a review of commit `4ca7aebcdb`, its finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Settled = { kind: 'yes' } | { kind: 'old-tip-left'; paths: Uint8Array[] } | { kind: 'another-command-at-work' } | { kind: 'unknown'; why: string };
/// ```
#[derive(Debug)]
pub enum Settled {
    /// Nothing this worktree may commit over is left by a move.
    Yes,
    /// A path the record's move wrote still holds what the branch moved from, and the write of it was refused.
    OldTipLeft {
        /// The branch the record moved.
        branch: String,
        /// The paths that still hold the old tip's entries.
        paths: Vec<Vec<u8>>,
    },
    /// Another command holds this checkout's repair right now.
    AnotherCommandAtWork,
    /// The question could not be answered.
    Unknown {
        /// What could not be read.
        why: String,
    },
}

/// One diagnostic line about a record that was left,
/// when diagnostics are on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (verboseEnabled()) stderr.write(`... ${message}\n`);
/// ```
fn debug_line<W: Write + ?Sized>(out: &mut W, message: &str) {
    if !crate::diagnostic_log::verbose_enabled() {
        return;
    }
    let _written = writeln!(out, "monochromatic-git-policy-cli: {message}");
}

/// What:
///  The same sweep,
/// for a caller that is not a task of the runtime.
/// Why:
///   Every command runs it before its own work,
/// and that entry is synchronous.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(finishBranchMoves({ git, commonDir, stderr }))
/// ```
pub fn finish_branch_moves(git: &PushGit, common_dir: &Path, out: &mut dyn Write) -> Result<Settled, String> {
    return match crate::lazy_runtime::run_on_runtime(finish_branch_moves_awaiting(git, common_dir, out)) {
        Ok(finished) => finished,
        Err(refusal) => Err(format!("{refusal}")),
    };
}
