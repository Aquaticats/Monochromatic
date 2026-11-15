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
/// type BranchMove = { owner: TransactionOwner; targetRef: string; oldOid: string; newOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchMove {
    /// The process that wrote the record,
    /// whose death is what tells a later command the move is finished or stopped.
    pub owner: TransactionOwner,
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
        let entry = entry?;
        let name: std::ffi::OsString = entry.file_name();
        if name.to_string_lossy().ends_with(RECORD_SUFFIX) {
            records.push(entry.path());
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
        target_ref: target_ref.to_owned(),
        old_oid: old_oid.to_owned(),
        new_oid: new_oid.to_owned(),
    };
    let path: PathBuf = write_branch_move(common_dir, &movement).ok()?;
    return Some((path, movement));
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
) -> Result<(), String> {
    let records: Vec<PathBuf> = list_branch_moves(common_dir).map_err(|error| format!("the branch moves of {} could not be listed: {error}", common_dir.display()))?;
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
        // A move that still runs is not finished by another command.
        let alive: bool = match crate::transaction_owner::classify_transaction_owner_record(&movement.owner, record.as_path()) {
            Ok(crate::transaction_owner::OwnerLiveness::Alive) => true,
            Ok(crate::transaction_owner::OwnerLiveness::Dead) => false,
            // A probe that cannot be made reads as alive, as it does for the merge records.
            Err(_) => true,
        };
        if alive {
            continue;
        }
        let holder = match crate::auto_push_reconcile_branch_worktree::live_holder_awaiting(git, movement.target_ref.as_str()).await {
            Ok(found) => found,
            // The question failed, so this command does not write anything for this record and leaves it.
            Err(error) => {
                debug_line(out, &format!("the worktrees of {} could not be read, so the record {} is left for a later command ({error})", movement.target_ref, record.display()));
                continue;
            }
        };
        let Some(place) = holder else {
            // No worktree holds the branch, so no checkout is left behind: the record has nothing to finish.
            let _removed = remove_branch_move(record.as_path());
            continue;
        };
        let here = match crate::auto_push_reconcile_branch_worktree::is_this_worktree(git, place.as_path()).await {
            Ok(answer) => answer,
            Err(error) => {
                debug_line(out, &format!("this command could not tell whether it runs in {} ({error}), so the record {} is left", place.display(), record.display()));
                continue;
            }
        };
        if !here {
            // The worktree that holds the branch finishes its own checkout.
            continue;
        }
        let tip = run_query_awaiting(git, &argv(&["rev-parse", "--verify", format!("{}^{{commit}}", movement.target_ref).as_str()])).await;
        let tip: String = match tip {
            Ok(printed) if printed.code == Some(0) => String::from_utf8_lossy(&printed.stdout).trim().to_owned(),
            _ => {
                debug_line(out, &format!("the tip of {} could not be read, so the record {} is left", movement.target_ref, record.display()));
                continue;
            }
        };
        if tip != movement.new_oid {
            // The move never happened, or another push moved the branch since: nothing of this record is left to finish.
            let _removed = remove_branch_move(record.as_path());
            continue;
        }
        let there: PushGit = crate::auto_push_reconcile_branch_worktree::git_at(git, place.as_path());
        let brought = crate::auto_push_reconcile_branch_checkout::bring_the_checkout_to_its_branch_awaiting(
            &there,
            movement.target_ref.as_str(),
            movement.old_oid.as_str(),
            movement.new_oid.as_str(),
        )
        .await;
        match brought {
            Ok(crate::auto_push_reconcile_branch_checkout::Brought::Right) => {
                let _removed = remove_branch_move(record.as_path());
            }
            Ok(crate::auto_push_reconcile_branch_checkout::Brought::Left { paths, reason, write_refused }) => {
                if write_refused {
                    // A write Git refused is one a later command may finish, so the record stays for it.
                    debug_line(out, &format!("the checkout at {} was not written ({reason}), so the record {} is left", place.display(), record.display()));
                    continue;
                }
                // The repair has run and told what it left; a record kept would only tell it again at every command.
                let _removed = remove_branch_move(record.as_path());
                let _noted = writeln!(
                    out,
                    "monochromatic-git-policy-cli: the checkout at {} still differs from {} on {} and was left as it is ({reason}); \
                     a commit there would record the older files.",
                    place.display(),
                    movement.target_ref,
                    crate::auto_push_dirty_paths::listed(&paths)
                );
            }
            Err(error) => {
                debug_line(out, &format!("the checkout at {} was not brought to {} ({error}), so the record {} is left", place.display(), movement.target_ref, record.display()));
            }
        }
    }
    return Ok(());
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
pub fn finish_branch_moves(git: &PushGit, common_dir: &Path, out: &mut dyn Write) -> Result<(), String> {
    return match crate::lazy_runtime::run_on_runtime(finish_branch_moves_awaiting(git, common_dir, out)) {
        Ok(finished) => finished,
        Err(refusal) => Err(format!("{refusal}")),
    };
}
