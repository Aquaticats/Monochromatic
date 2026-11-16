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
    BRANCH_MOVE_SCHEMA_VERSION, BranchMove, list_branch_moves, parse_branch_move,
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

/// The states a repair place's records are written with: the place's owner, and the claim on a dead owner's place.
/// Why:
///   One reader answers for both, so the list is one value the writer and the reader share and cannot drift:
///  a claim written here and read by a reader that accepts only the owner's state was never cleared, and every
///  later command waited and stopped for good
///  (a review of commit `986a82bbab`, its finding 3).
pub const REPAIR_PLACE_STATES: [&str; 2] = ["repair-owner", "repair-claim"];

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
    let text: String = the_places_record(REPAIR_PLACE_STATES[0], owner);
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
    dead_path: &Path,
    the_dead_owner: &TransactionOwner,
) -> Option<ARepairInHand> {
    // What: a claim made only once, under a name made from the dead owner's own identity.
    // Why:  the claim is the whole of the mutual exclusion for taking this dead record: only the command that
    // created it may remove the dead record, and no live record can appear at that path while the dead one
    // stands there, because creating a record there is exclusive. So the removal below never takes a live
    // record, and nothing is moved aside and put back
    //  (a review of commit `7aef6368e7`, its finding 4).
    let claim: PathBuf = dead_path.with_extension(format!("claim-{}", the_dead_owner.transaction_id));
    match claim_the_take(claim.as_path(), owner) {
        Ok(()) => {}
        Err(ClaimRefused::TakenByAnother) => {
            // What: a claim whose claimant is gone, or whose bytes cannot be read, is set aside by renaming it, and
            // exactly one command wins that rename.
            // Why:  removing it and claiming again let two commands that both read the same dead claim each remove the
            // other's live claim and each take the dead place, so two repairs wrote one index
            //  (a review of commit `4a2425a378`, its finding 5); a rename is the step only one of them can win, and a
            // claim that names nobody is no live claim at all  (its finding 4).
            let claimant: Option<TransactionOwner> = the_owner_named_by(claim.as_path());
            let gone: bool = match claimant.as_ref().map(|named| return crate::transaction_owner::classify_transaction_owner_record(named, claim.as_path())) {
                Some(Ok(crate::transaction_owner::OwnerLiveness::Dead)) => true,
                // A claim whose bytes cannot be read names no live command either.
                None => true,
                // A live claimant, or one that cannot be judged, keeps its claim.
                _ => false,
            };
            if !gone {
                return None;
            }
            let aside: PathBuf = claim.with_extension(format!("cleared-{}", owner.transaction_id));
            std::fs::rename(claim.as_path(), aside.as_path()).ok()?;
            claim_the_take(claim.as_path(), owner).ok()?;
        }
        Err(ClaimRefused::CannotWrite(error)) => {
            let _why = error;
            return None;
        }
    }
    // What: the dead record goes only while this command holds the claim, and only if it is still the dead one.
    // Why:  no live record can stand here while the dead one does, so the content check is the second witness and
    // not the only one.
    let still_the_dead_one: bool = the_owner_named_by(dead_path)
        .is_some_and(|found| return found.transaction_id == the_dead_owner.transaction_id);
    let taken: Option<ARepairInHand> = if still_the_dead_one {
        let _removed: std::io::Result<()> = std::fs::remove_file(dead_path);
        the_repair_owner(git_dir, owner)
    } else {
        None
    };
    let _released: std::io::Result<()> = std::fs::remove_file(claim.as_path());
    let _aside_removed: std::io::Result<()> = std::fs::remove_file(claim.with_extension(format!("cleared-{}", owner.transaction_id)));
    return taken;
}

/// Why a claim on one dead repair record was not made.
enum ClaimRefused {
    /// Another command holds the claim.
    TakenByAnother,
    /// The claim could not be written at all.
    CannotWrite(std::io::Error),
}

/// What:
///  Claim the right to take one dead repair owner's record,
/// in a step exactly one command can win.
/// Why:
///   The claim is one file created exclusively, whose name is the dead owner's own transaction id, so the
/// command that creates it is the only one that may remove that dead record.
///  It never touches the repair place itself, so a live record there is never moved or replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeExclusiveSynced(claim, encode(owner)) // exactly one of two commands succeeds
/// ```
fn claim_the_take(claim: &Path, owner: &TransactionOwner) -> Result<(), ClaimRefused> {
    let text: String = the_places_record(REPAIR_PLACE_STATES[1], owner);
    return match write_exclusive_synced(claim, text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(ClaimRefused::TakenByAnother),
        Err(error) => Err(ClaimRefused::CannotWrite(error)),
    };
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
    let git_dir: PathBuf = match crate::auto_push_reconcile_branch_checkout_files::the_git_directory_of(git) {
        Ok(Some(directory)) => directory,
        Ok(None) => return Ok(Settled::Yes),
        Err(why) => return Ok(Settled::Unknown { why }),
    };
    let shown: Option<String> = match crate::auto_push_reconcile_branch_checkout_files::the_branch_this_worktree_shows(git_dir.as_path()) {
        Ok(branch) => branch,
        Err(why) => return Ok(Settled::Unknown { why }),
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
        let asked = run_query_awaiting(git, &argv(&["rev-parse", "--verify", format!("{}^{{commit}}", movement.target_ref).as_str()])).await;
        let tip: String = match asked {
            Ok(printed) if printed.code == Some(0) => String::from_utf8_lossy(&printed.stdout).trim().to_owned(),
            // What: `rev-parse --verify` answers 1 for a ref that does not exist, which is the answer "the branch is
            // gone", and every other status is a read that failed.
            // Why:  a record whose branch was deleted or renamed names nothing that can be finished, so it goes and the
            // command runs; taking the answer for a failed read stopped every command in every worktree of the
            // repository, `git commit` and `git branch` among them, and the message named no file
            //  (a review of commit `4a2425a378`, its finding 1).
            Ok(printed) if printed.code == Some(1) => {
                let _noted = writeln!(
                    out,
                    "monochromatic-git-policy-cli: the record {} names {}, which this repository no longer has, so the record is removed",
                    record.display(),
                    movement.target_ref
                );
                crate::auto_push_branch_move_record::AMoveInHand::adopt(record, movement)
                    .finish(crate::auto_push_branch_move_record::AFinishedRecord::NeverMoved);
                continue;
            }
            _ => {
                debug_line(out, &format!("the tip of {} could not be read, so the record {} is left", movement.target_ref, record.display()));
                settled = Settled::Unknown {
                    why: format!("the tip of {} could not be read, so the record {} is left", movement.target_ref, record.display()),
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
        // What: whether this worktree is one the record is about, asked now that the move is known to have
        // happened.
        // Why:  the record is a record of the index and files of whichever worktree took the moved branch, and a
        // worktree's HEAD can change between any two of this command's questions: keying the question to the
        // branch HEAD names left a switch away from the branch with a reversal nothing repaired
        //  (a review of commit `7aef6368e7`, its findings 1 and 3).
        // What: whether this worktree is the checkout the move is for: it shows the record's branch.
        // Why:  an absent index entry is the move's own step only in that checkout, and in a worktree that shows
        // another branch it is what a person's `git rm` leaves
        //  (a review of commit `4a2425a378`, its finding 2, its `git rm` case).
        let shows_the_record: bool = shown.as_deref() == Some(movement.target_ref.as_str());
        let ours: bool = match this_worktree_holds_what_the_move_wrote(git, &movement, shows_the_record).await {
            Ok(holds) => holds,
            Err(why) => {
                debug_line(out, &format!("this worktree's files could not be read ({why}), so the record {} is left", record.display()));
                settled = Settled::Unknown { why };
                continue;
            }
        };
        if !ours {
            // What: a worktree that shows the record's branch and stands at the tip that branch names has nothing of
            // the move left to write, so the record goes; the paths where the checkout still differs from that tip
            // are the person's and are named.
            // Why:  the record exists to finish a checkout an interrupted move left behind, and this one is not it;
            // nothing else removed a record for a branch no worktree took, so every command paid its Git starts for
            // ever  (a review of commit `4a2425a378`, its finding 1; `README.md` and `SPEC.md` say the record is removed
            // when a command in the worktree that shows the branch agrees with it, and that changes which are the
            // person's own are left to them and named).
            // Taking their absence for the move's leftover would write over a deletion they made, which is what a
            // command must never do  (the same review, its finding 2).
            let agrees: bool = shows_the_record
                && crate::auto_push_reconcile_branch_checkout_files::the_head_commit_awaiting(git).await.is_ok_and(|head| return head == tip);
            if agrees {
                let theirs: Vec<Vec<u8>> = match crate::auto_push_reconcile_branch_checkout::the_moves_paths_awaiting(git, movement.old_oid.as_str(), movement.new_oid.as_str()).await {
                    Ok(paths) => match crate::auto_push_reconcile_branch_checkout_files::the_paths_the_checkout_disagrees_with_its_branch_awaiting(git, paths.as_slice()).await {
                        Ok(differing) => differing,
                        // A read that failed is not the answer "nothing differs": the record stays, as for every
                        // other failed read here.
                        Err(why) => {
                            debug_line(out, &format!("this worktree's checkout could not be read ({why}), so the record {} is left", record.display()));
                            settled = Settled::Unknown { why: format!("{why}") };
                            continue;
                        }
                    },
                    Err(why) => {
                        debug_line(out, &format!("the paths of the move could not be read ({why}), so the record {} is left", record.display()));
                        settled = Settled::Unknown { why: format!("{why}") };
                        continue;
                    }
                };
                crate::auto_push_branch_move_record::AMoveInHand::adopt(record, movement.clone())
                    .finish(crate::auto_push_branch_move_record::AFinishedRecord::TheCheckoutHoldsTheTip(
                        crate::auto_push_branch_move_record::TheCheckoutHoldsTheTip::from_the_checkout(
                            movement.target_ref.as_str(),
                            tip.as_str(),
                        ),
                    ));
                if !theirs.is_empty() {
                    let _said = writeln!(
                        out,
                        "monochromatic-git-policy-cli: {} there hold a change of your own and were left as they are.",
                        crate::auto_push_dirty_paths::listed(&theirs)
                    );
                }
            }
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
                movement.old_oid.as_str(),
                movement.new_oid.as_str(),
                shows_the_record,
            )
            .await;
        match brought {
            crate::auto_push_reconcile_branch_checkout::Brought::NoOldTip { a_persons } => {
                // What: the repair answered that no path of the move holds what the branch moved from; the record
                // ends on that proof only in the worktree that shows the moved branch, which is the one that took it.
                // Why:  a worktree that shows another branch was repaired for that branch's sake, and its answer
                // says nothing about the worktree the move is for
                //  (a review of commit `777c491325`, its findings 3 to 5, and commit `7aef6368e7`, its finding 3).
                repair.finish();
                if !shows_the_record {
                    continue;
                }
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
///  Whether this worktree is one a record is about:
/// does its index or its files hold an entry the move wrote there, on a path where that entry differs from the
/// one this worktree's HEAD names now.
/// Why:
///   A record is a record of the index and files of whichever worktree took the moved branch, and not of a
/// branch name in a worktree.
///  Asking it of HEAD's tree is what keeps a worktree that legitimately stands at the moved-from tip (its own
/// branch is there) out of it: nothing of the move is left on a path its HEAD does not already hold.
///  A read that failed is an error and never "this worktree is not ours", because the answer lets a command go
/// on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const end of [old, new]) for (const path of diff(end, HEAD)) if (indexHolds(end, path) || fileHolds(end, path)) return true
/// ```
async fn this_worktree_holds_what_the_move_wrote(git: &PushGit, movement: &BranchMove, the_moves_own_checkout: bool) -> Result<bool, String> {
    return holds_what_a_move_wrote(git, movement.old_oid.as_str(), movement.new_oid.as_str(), the_moves_own_checkout).await;
}

/// What:
///  Whether this worktree holds an entry a move wrote, on a path where the tree its HEAD names does not.
/// Why:
///   The question is the same for a record's move and for the move a push just made in a worktree that could not
/// follow it, so it is asked once.
///  `the_moves_own_checkout` says whether this worktree shows the branch the move is about: only there is an
/// absent index entry the move's own step, while in a worktree that shows another branch it is what a person's
/// `git rm` leaves, which nothing may write back
///  (a review of commit `4a2425a378`, its finding 2, its `git rm` case).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdsWhatAMoveWrote({ git, old, merge, theMovesOwnCheckout }) // HEAD contains merge && (index or file holds old or merge on a differing path)
/// ```
pub(crate) async fn holds_what_a_move_wrote(git: &PushGit, old_oid: &str, new_oid: &str, the_moves_own_checkout: bool) -> Result<bool, String> {
    let target: String = crate::auto_push_reconcile_branch_checkout_files::the_head_commit_awaiting(git)
        .await
        .map_err(|error| format!("this worktree's HEAD could not be read: {error}"))?;
    // What: the commit this worktree's HEAD names must hold the move's own new tip in its history.
    // Why:  bytes equal to a tip are not proof that the move wrote this checkout: a person's staged edit in a
    // worktree that never showed the pushed branch can hold the merge's own bytes, and the repair read that as
    // the move's debris and wrote the old bytes back over it
    //  (a review of commit `986a82bbab`, its finding 1).
    // A checkout the move wrote shows the moved branch at the merge, or a branch made from it after the move,
    // so the merge is in its history; a worktree that never showed the branch does not have it at all.
    let carries_the_move: bool = match crate::auto_push_reconcile_basics_and_limits::is_ancestor_awaiting(git, new_oid, target.as_str()).await {
        Ok(Some(false)) => {
            return Ok(false);
        }
        Ok(Some(true)) => true,
        _ => return Err(String::from("the tip this worktree shows could not be related to the move")),
    };
    if !carries_the_move {
        return Ok(false);
    }
    for end in [old_oid, new_oid] {
        let paths: Vec<Vec<u8>> = crate::auto_push_reconcile_branch_checkout::the_moves_paths_awaiting(git, end, target.as_str())
            .await
            .map_err(|error| format!("the paths of the move could not be read: {error}"))?;
        if paths.is_empty() {
            continue;
        }
        // What: an entry this end really has, present in this worktree's index or in its file.
        // Why:  an absent path is not the move's leftover: a file the merge added and a person deleted, or one they
        // untracked with `git rm --cached`, holds nothing an end wrote  (a review of commit `4a2425a378`, its
        // findings 2 and 3).
        let wrote: Vec<Vec<u8>> =
            crate::auto_push_reconcile_branch_checkout_files::the_paths_the_move_wrote_awaiting(git, end, paths.as_slice(), the_moves_own_checkout)
                .await
                .map_err(|error| format!("this worktree's index and files could not be read: {error}"))?;
        if !wrote.is_empty() {
            return Ok(true);
        }
    }
    return Ok(false);
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
        // What: a place whose bytes cannot be read is not a live claim: it is set aside under another name and the
        // place is taken.
        // Why:  a live claim names its holder, and bytes that name nobody proved nothing when they were written, so
        // waiting the whole bound for them and then stopping the command for ever is an answer the person can do
        // nothing about  (a review of commit `4a2425a378`, its finding 4).
        debug_line(out, &format!("the repair owner's record {} could not be read, so it is set aside and the place is taken", place.display()));
        let aside: PathBuf = place.with_extension(format!("unreadable-{}", owner.transaction_id));
        let _moved: std::io::Result<()> = std::fs::rename(place.as_path(), aside.as_path());
        return the_repair_owner(git_dir, &owner);
    };
    match crate::transaction_owner::classify_transaction_owner_record(&theirs, place.as_path()) {
        Ok(crate::transaction_owner::OwnerLiveness::Dead) => return the_repair_owner_after_its_owner_died(git_dir, &owner, place.as_path(), &theirs),
        // A probe that cannot be made reads as alive, as it does for the merge records.
        Ok(crate::transaction_owner::OwnerLiveness::Alive) | Err(_) => return None,
    }
}

/// What:
///  The bytes of one repair place record, in the state its writer chose.
/// Why:
///   One writer's shape is what one reader reads, so both are made here and nowhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = (state, owner) => JSON.stringify({ schemaVersion, state, transactionId, ownerPid, ownerIdentity, createdAt }) + '\n'
/// ```
/// What:
///  The bytes of one repair place record, in the state its writer chose, for the control that plants one.
/// Why:
///   A control that plants a claim has to write the shape this file's own writer writes.
pub fn the_places_record_for_a_control(state: &str, owner: &TransactionOwner) -> String {
    return the_places_record(state, owner);
}

fn the_places_record(state: &str, owner: &TransactionOwner) -> String {
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", BRANCH_MOVE_SCHEMA_VERSION)
        .string("state", state)
        .string("transactionId", owner.transaction_id.as_str())
        .integer("ownerPid", owner.owner_pid)
        .string("ownerIdentity", owner.owner_identity.as_str())
        .string("createdAt", owner.created_at.as_str());
    // What: the pid namespace the place names, when the owner has one, as the branch-move record writes it.
    // Why:  a holder in another namespace judged by a bare pid in this one may be taken over while it is alive
    //  (a review of commit `4a2425a378`, its finding 8).
    if let Some(namespace) = &owner.owner_pid_namespace {
        writer.string("ownerPidNamespace", namespace.as_str());
    }
    return format!("{}\n", writer.finish());
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
    // What: every state a repair place's writer emits, asked of the one list both of them use.
    // Why: a reader that accepted only the owner's state read every claim as unreadable, so a dead claimant was
    // never cleared and every later command waited and stopped
    //  (a review of commit `986a82bbab`, its finding 3).
    if safe_integer_field(&map, "schemaVersion")? != BRANCH_MOVE_SCHEMA_VERSION
        || !REPAIR_PLACE_STATES.contains(&string_field(&map, "state")?)
    {
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
