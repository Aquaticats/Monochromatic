//! What:
//!  When a record of a branch's move is finished, who may finish it, and the one place a command takes
//!  to repair the checkout it names.
//! Why:
//!  A record is finished when every worktree that has its branch checked out shows the tip the branch was
//!  moved to, in its index and in its files, or holds changes that are a person's own, which are left to
//!  them and named.
//!  No single comparison answers that:
//!  the branch tip alone is not the answer, because the tip moves before any checkout does;
//!  the index alone is not, because a `git reset` moves the index without the files;
//!  and one path is not, because a checkout can hold the merge's content on one path and the old tip's on
//!  another.
//!  A record's writer is the move and not the process that hosts moves:
//!  every exit after the record is written hands it on, by a value whose end publishes it as left, so the
//!  sweep never asks whether the writing process lives.
//!  A record is deleted only on a proof that holds at the moment it is deleted:
//!  the tip it names is no longer the branch's tip, which makes its move void;
//!  or this command brought the checkout that shows that branch to it.
//!  A listing that names no holder is not a proof, because a checkout that read the old tip may finish
//!  publishing its `HEAD` after that listing.
//!  One command repairs one checkout at a time, by one record created exclusively in the worktree's own
//!  Git directory; a command that finds a live one waits for it bounded by `REPAIR_WAIT`, and one that
//!  finds a dead one takes the place.
//!  Kept apart from `auto_push_branch_move_record.rs`, which holds the record itself, because that file
//!  reached the repository's line budget, and because what may be written into a person's checkout is a
//!  different subject from the bytes of a record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settled = await finishBranchMoves({ git, commonDir }); // yes | old-tip-left | another-command-at-work | unknown
//! ```
//!
//! The questions below are asked of the checkout, in this order.

use crate::auto_push_branch_move_record::{
    BRANCH_MOVE_SCHEMA_VERSION, list_branch_moves, parse_branch_move,
};
use crate::auto_push_reconcile_basics_and_limits::argv;
use crate::json_record::{ObjectWriter, parse_object, safe_integer_field, string_field};
use crate::private_storage::write_exclusive_synced;
use crate::auto_push_single_flight::{PushGit, run_query_awaiting};
use crate::transaction_owner::TransactionOwner;
use std::io::Write;
use std::path::{Path, PathBuf};

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
    /// The transaction this command gave the place, which is what makes giving it back this holder's own act.
    mine: String,
    /// Whether it was given back already.
    done: bool,
}

impl ARepairInHand {
    /// The record's place, so a later command may judge the owner it names.
    pub fn record_path(&self) -> &Path {
        return self.path.as_path();
    }

    /// What:
    ///  Give the place back, and only if the record there still names this command.
    /// Why:
    ///  A taker that lost its place to a successor must not remove the successor's record:
    /// the removal is the holder's own act, and the name it writes is what identifies the holder
    ///  (a review of commit `777c491325`, its finding 2).
    pub fn finish(&mut self) {
        let mine: &str = self.mine.as_str();
        if the_owner_named_by(self.path.as_path()).is_some_and(|named| return named.transaction_id == mine) {
            let _removed: std::io::Result<()> = std::fs::remove_file(self.path.as_path());
        }
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
        self.finish();
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
    return Some(ARepairInHand {
        path,
        mine: owner.transaction_id.clone(),
        done: false,
    });
}

/// What:
///  Take a repair place whose owner is gone,
/// by moving that very record aside and taking the place, in one compare-and-take.
/// Why:
///   A repair whose command was killed leaves its record; the next command must be able to take it
/// without a person's help,
///  and two commands that both read the dead record must not both take the place:
/// the rename of the record each of them read succeeds for exactly one of them,
///  so the one that renamed it away is the one that takes the place, and the other finds the record gone
/// and takes nothing
///  (a review of commit `777c491325`, its finding 2).
///  A taker that renamed the dead record away and then could not create its own puts the record back,
/// so the place is not left free with no owner named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aside = `${dead}.taken-${owner.transactionId}`; try { renameSync(dead, aside) } catch { return undefined } const taken = writeExclusiveSynced(path, bytes); if (!taken) renameSync(aside, dead); else rmSync(aside, { force: true }); return taken;
/// ```
pub fn the_repair_owner_after_its_owner_died(
    git_dir: &Path,
    owner: &TransactionOwner,
    dead: &Path,
    the_dead_owner: &TransactionOwner,
) -> Option<ARepairInHand> {
    let aside: PathBuf = dead.with_extension(format!("taken-{}", owner.transaction_id));
    // The rename is the compare: exactly one taker of this dead record can move it.
    std::fs::rename(dead, aside.as_path()).ok()?;
    // What: the record that was moved aside must be the very dead one this command judged.
    // Why: another command may have moved the dead record out and put its own live one in the place this
    // command read a moment ago; taking that one away would give two commands the same place
    //  (a review of commit `777c491325`, its finding 2).
    let moved: Option<TransactionOwner> = the_owner_named_by(aside.as_path());
    if !moved.is_some_and(|found| return found.transaction_id == the_dead_owner.transaction_id) {
        // A live owner's record was in that place when this command renamed it: it goes back and nothing is taken.
        let _restored: std::io::Result<()> = std::fs::rename(aside.as_path(), dead);
        return None;
    }
    let taken: Option<ARepairInHand> = the_repair_owner(git_dir, owner);
    if taken.is_none() {
        // The place was taken by another command in the instant since; the dead record goes back where it was.
        let _restored: std::io::Result<()> = std::fs::rename(aside.as_path(), dead);
        return None;
    }
    let _removed: std::io::Result<()> = std::fs::remove_file(aside.as_path());
    return taken;
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
        if tip == movement.old_oid {
            // What: the branch is still where it was, so the compare-and-swap never moved it.
            // Why: that is the proof of a void move, read at the moment the record goes, and it is the only
            //      tip comparison that may end a record: a branch that moved on leaves the checkout this record
            //      names still to be brought forward, to the tip the branch has now
            //      (a review of commit `777c491325`, its finding 1).
            crate::auto_push_branch_move_record::AMoveInHand::adopt(record, movement)
                .finish(crate::auto_push_branch_move_record::AFinishedRecord::NeverMoved);
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
        let brought: crate::auto_push_reconcile_branch_checkout::Brought =
            crate::auto_push_reconcile_branch_checkout::bring_the_checkout_to_its_branch_awaiting(
                git,
                movement.target_ref.as_str(),
                movement.old_oid.as_str(),
                tip.as_str(),
            )
            .await;
        match brought {
            crate::auto_push_reconcile_branch_checkout::Brought::NoOldTip { a_persons } => {
                // What: the repair answered that no path of the move holds what the branch moved from, and that
                // is the proof the record ends on.
                // Why: the person's own changes, if any, are theirs to commit, which is the rule the gate keeps too
                //      (a review of commit `777c491325`, its findings 3 to 5).
                repair.finish();
                crate::auto_push_branch_move_record::AMoveInHand::adopt(record, movement.clone())
                    .finish(crate::auto_push_branch_move_record::AFinishedRecord::TheCheckoutHoldsTheTip(
                        crate::auto_push_branch_move_record::TheCheckoutHoldsTheTip::from_the_checkout(
                            movement.target_ref.as_str(),
                            tip.as_str(),
                        ),
                    ));
                let _noted = writeln!(
                    out,
                    "monochromatic-git-policy-cli: the checkout at this worktree now holds {branch} at {merge}, which a push that stopped \
                     before it finished that checkout left behind; the edits you made elsewhere are kept.",
                    branch = movement.target_ref,
                    merge = tip
                );
                if !a_persons.is_empty() {
                    let _said = writeln!(
                        out,
                        "monochromatic-git-policy-cli: {} there hold a change of your own and were left as they are.",
                        crate::auto_push_dirty_paths::listed(&a_persons)
                    );
                }
            }
            crate::auto_push_reconcile_branch_checkout::Brought::OldTipLeft { paths, why } => {
                // What: a path of the move still holds what the branch moved from, so the record stays.
                // Why: a commit here would record that tree, and a later command may finish the write.
                let _kept = repair;
                debug_line(out, &format!("the checkout here was not written ({why}), so the record {} is left", record.display()));
                settled = Settled::OldTipLeft {
                    branch: movement.target_ref.clone(),
                    paths,
                };
            }
            crate::auto_push_reconcile_branch_checkout::Brought::CouldNotTell { why } => {
                // What: the checkout could not be read or written, so nothing follows from it and the record stays.
                // Why: a read that failed is not a proof that the checkout is right
                //      (a review of commit `777c491325`, its findings 4 and 5).
                let _kept = repair;
                debug_line(out, &format!("the checkout here could not be read or written ({why}), so the record {} is left", record.display()));
                settled = Settled::Unknown { why };
            }
        }
    }
    return Ok(settled);
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
        Ok(crate::transaction_owner::OwnerLiveness::Dead) => return the_repair_owner_after_its_owner_died(git_dir, &owner, place.as_path(), &theirs),
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
