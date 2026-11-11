//! What:
//!  What one add does with the snapshots the commit service made ahead of it:
//! prove the newest candidate equal to the
//!  source's index,
//! claim it by one rename into the stage name,
//! wait for one when there is none,
//! and hand the service what it
//!  proved stale or keeps no longer.
//! Why:
//!  One `BTRFS_IOC_SNAP_CREATE_V2` forces a btrfs transaction commit of the whole filesystem on the thread that calls
//!  it,
//! which took 5,413.0 ms at the median on this volume
//! (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
//! "A worktree add"),
//! so the copy an add uses is usually one the commit
//!  service made before the add arrived,
//! and the add's own cost is a walk of it against the source's index and one rename.
//! A copy may not be trusted
//!  because it is there:
//! the files the new worktree receives must equal the source's index,
//! and a copy that fell behind
//!  it would give a worktree whose tracked files carry an older state beside freshly recorded stat data,
//! which Git
//!  would then report as clean.
//! The walk is the same one the index step makes over a copy the add took itself,
//! so a copy
//!  proven before it was claimed is never walked twice.
//! The names,
//! the work items and the service seam this file uses are in `worktree_snapshot_ahead.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const look = lookForASnapshotAhead({ source, plan, stage, environment, service });
//! // 'claimed' with the proof, 'none' (take one here), or 'timedOut' with the words for the person
//! ```

use crate::wait_ceilings::waiting_notice_after;
use crate::worktree_snapshot_ahead::{AHEAD_KEPT, AheadLook, AheadService, POLL, ahead_key, candidates, kept_in, made_at, over_kept, pool_folder};
use crate::add_timing::{begin_step, end_step, note as note_timing};
use crate::diagnostic_log::debug;
use crate::fs_beneath::{ObjectId, entry_metadata_at, identity_at, open_root, rename_at};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_snapshot_index_step::{Proof, prove_the_copy};
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSnapshotAheadClaim";

/// What:
///  The identity of what a path's name leads to now,
/// or nothing when its parent cannot be opened or the name cannot be looked up.
/// Why:
///   The claimed snapshot's identity is what the add records for a later removal to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityAt(openRoot(dirname(path)), basename(path)) ?? undefined
/// ```
pub(crate) fn identity_of_name(path: &Path) -> Option<ObjectId> {
    let parent: File = open_root(path.parent().unwrap_or(path)).ok()?;
    return identity_at(&parent, path.file_name()?).ok();
}

/// What:
///  Whether the entry `name` below `folder` is a directory,
/// which is the only thing that can hold a copy of a source.
/// Why:
///   The proof opens the entry as a tree,
/// and an entry that is a link is refused rather than renamed,
/// because the rename
///  would move whatever stands at the link's target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isADirectory = (dir, name) => lstatSync(join(handlePath(dir), name)).isDirectory();
/// ```
fn is_a_directory(folder: &File, name: &OsStr) -> bool {
    return match entry_metadata_at(folder, name) {
        Ok(found) => found.is_dir(),
        Err(_unreadable) => false,
    };
}

/// What:
///  Claim the newest snapshot ahead whose files equal the source's index,
/// by renaming it into the stage name.
/// Why:
///   The proof runs first,
/// because an unproven snapshot must never reach the stage:
/// the add would install the
///  source's index over that snapshot's files and Git would report the difference as clean.
/// The rename is what makes one
///  claim out of racing adds:
/// the loser finds the name gone and tries the next candidate,
/// and the winner's stage
///  name holds the object this process proved.
/// Every candidate the proof refused is answered so the caller hands it to
///  the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = (source, stage, environment, folder, key) => { for (const name of candidates(folder, key)) { if (!isADirectory(folder, name)) continue; if (!prove(source, join(folderPath, name))) { stale.push(name); continue; } if (rename(folder, name, folder, stageName)) return proof; } };
/// ```
fn claim_the_newest_proven(source: &SnapshotSource, stage: &Path, environment: &ConversionEnvironment, folder: &File, folder_path: &Path, key: &str, stale: &mut Vec<OsString>, stages: &File) -> Option<(Proof, u128)> {
    let stage_name: &OsStr = stage.file_name()?;
    for name in candidates(folder, key) {
        // The millisecond in the name is the instant the copy was made, and the ignored state of the new worktree is
        //  that instant's; the caller names every ignored entry the source wrote after it.
        let Some(copied_at_millis) = made_at(name.as_os_str()) else { continue };
        if !is_a_directory(folder, name.as_os_str()) {
            continue;
        }
        let entry: PathBuf = folder_path.join(&name);
        let proving: bool = begin_step("snapshotAheadProof");
        let proven: Result<Proof, String> = prove_the_copy(source, entry.as_path(), environment);
        end_step(proving);
        match proven {
            Err(refusal) => {
                debug(TAG, format!("{} does not equal the source's index ({refusal}), so it goes to the service and the next name is tried", entry.display()).as_str());
                // The wait looks again every `POLL`, so one name is proven stale once per pass: it is handed over once.
                if !stale.contains(&name) {
                    stale.push(name);
                }
            }
            // The pool is inside the Git directory and the stage is beside the destination, so the claim is one rename across two
            //  folders of one filesystem.
            Ok(proof) => match rename_at(folder, name.as_os_str(), stages, stage_name, false) {
                Ok(()) => {
                    debug(TAG, format!("{} was claimed as {} after proving it equal to the source's index", entry.display(), stage.display()).as_str());
                    return Some((proof, copied_at_millis));
                }
                // Another add claimed it between the proof and the rename; the next candidate is this add's chance.
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    debug(TAG, format!("{} was claimed by another add ({error})", entry.display()).as_str());
                }
                Err(error) => {
                    debug(TAG, format!("{} proved equal to the source's index and cannot be claimed ({error})", entry.display()).as_str());
                    return None;
                }
            },
        }
    }
    return None;
}

/// What:
///  Claim whatever a warm pool offers this add right now,
/// answering the names it proved stale.
/// Why:
///   This is the whole saving when the service is ahead:
/// one listing,
/// one proof and one rename,
/// and no btrfs
///  commit on the person's thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimNow = (source, stage, environment, stale) => claimed ?? undefined;
/// ```
fn claim_now(source: &SnapshotSource, stage: &Path, environment: &ConversionEnvironment, stale: &mut Vec<OsString>) -> Option<(ObjectId, Proof, u128)> {
    let folder_path: PathBuf = pool_folder(source.git_dir.as_path());
    let folder: File = open_root(folder_path.as_path()).ok()?;
    let stages: File = open_root(stage.parent()?).ok()?;
    let key: String = ahead_key(source.root.as_path());
    let (proof, copied_at_millis): (Proof, u128) = claim_the_newest_proven(source, stage, environment, &folder, folder_path.as_path(), key.as_str(), stale, &stages)?;
    let made: ObjectId = identity_of_name(stage)?;
    return Some((made, proof, copied_at_millis));
}

/// What:
///  Hand the names this add settled to the service,
/// and forget them,
/// so no name is handed over twice.
/// Why:
///   The wait looks again and again,
/// so one name is settled once per pass,
/// and every hand-over is a wake and a
///  record;
/// `handed` is what this add already said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handOver = (service, stale) => { if (stale.length === 0) return; service.handOver(stale); stale.length = 0; };
/// ```
fn hand_over(service: &AheadService<'_>, stale: &mut Vec<OsString>, handed: &mut Vec<OsString>) {
    let mut unsaid: Vec<OsString> = Vec::with_capacity(stale.len());
    for name in stale.iter() {
        if !handed.contains(name) {
            unsaid.push(name.clone());
        }
    }
    *stale = unsaid;
    if stale.is_empty() {
        return;
    }
    let _told = (service.hand_over)(stale.as_slice());
    handed.extend_from_slice(stale.as_slice());
    stale.clear();
}

/// What:
///  A wait in the unit a person reads it in:
/// whole seconds from one second up,
/// milliseconds below that.
/// Why:
///   A control shortens the ceiling to milliseconds,
/// and a person's sentence must state the bound it actually
///  reached rather than rounding it to "0 s".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inWords = (d: number) => d >= 1000 ? `${Math.round(d / 1000)} s` : `${d} ms`;
/// ```
fn in_words(ceiling: Duration) -> String {
    return if ceiling.as_secs() >= 1 { format!("{} s", ceiling.as_secs()) } else { format!("{} ms", ceiling.as_millis()) };
}

/// What:
///  Wait,
/// bounded,
/// for a snapshot ahead this add can claim.
/// Why:
///   The service takes the btrfs commit on its own thread,
/// so the add waits for an answer instead of for the kernel,
/// and
///  the ceiling is what keeps that wait finite when the service is busy or stopped.
/// A wait longer than
///  `waiting_notice_after` says on standard error what it waits for and until when,
/// so a person who sees a
///  slow add learns which step it is on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (Date.now() < deadline) { const claim = claimNow(); if (claim) return claim; await sleep(POLL); } return undefined;
/// ```
fn wait_for_one(source: &SnapshotSource, stage: &Path, environment: &ConversionEnvironment, stale: &mut Vec<OsString>, ceiling: Duration) -> Option<(ObjectId, Proof, u128)> {
    let began: Instant = Instant::now();
    let notice_after: Duration = waiting_notice_after();
    let mut noticed: bool = false;
    loop {
        if let Some(claimed) = claim_now(source, stage, environment, stale) {
            return Some(claimed);
        }
        let spent: Duration = began.elapsed();
        let left: Duration = ceiling.saturating_sub(spent);
        if left.is_zero() {
            return None;
        }
        if !noticed && spent >= notice_after {
            noticed = true;
            // A closed standard error leaves nowhere to print; the timing record below still says how the wait ended.
            let _told = writeln!(
                std::io::stderr(),
                "monochromatic-git-policy-cli: still waiting for the commit service to make a snapshot ahead for this add ({} s so far, at most {}, after which Git's own checkout makes the worktree)",
                spent.as_secs(),
                in_words(ceiling)
            );
        }
        std::thread::sleep(POLL.min(left));
    }
}

/// What:
///  Look for a snapshot the wrapper made ahead of this add,
/// ask the service for one when there is none,
/// wait for it no longer than the ceiling,
/// and hand the service everything this add proved stale and keeps no longer.
/// Why:
///   The add's own snapshot forces a commit on the person's thread,
/// so it is taken only when nothing came;
/// a
///  ceiling that passed ends the snapshot rung with the reason,
/// and the caller makes the worktree the way Git does and tells
///  the person why.
/// The request goes out before the wait,
/// so the wait is for work this add asked for,
/// and never
///  for whatever a busy service happened to be doing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = ({ source, stage, environment, service }) => { if (source.dirty) return none; const warm = claimNow(); if (warm) { handOver(stale); return claimed(warm); } if (handOver(stale) is err) return none; const waited = waitForOne(); handOver(stale + overKept(keptIn(pool, source.root))); return waited ? claimed(waited) : timedOut; };
/// ```
pub fn look_for_a_snapshot_ahead(source: &SnapshotSource, plan: &AddPlan, stage: &Path, environment: &ConversionEnvironment, service: &AheadService<'_>) -> AheadLook {
    // A source with uncommitted changes carries its own index, which is one instant with its files, so its snapshot has to be
    //  taken on this add's own path: nothing proves a copy of an earlier instant equal to what the add must carry.
    if source.dirty.is_some() {
        note_timing("snapshotAhead", "not-for-a-source-with-changes");
        return AheadLook::None;
    }
    let pool: PathBuf = pool_folder(source.git_dir.as_path());
    let mut stale: Vec<OsString> = Vec::new();
    let mut handed: Vec<OsString> = Vec::new();
    if let Some((made, proof, copied_at_millis)) = claim_now(source, stage, environment, &mut stale) {
        note_timing("snapshotAhead", "claimed");
        // The add after this one needs one, and whatever this look proved stale goes with the same wake.
        hand_over(service, &mut stale, &mut handed);
        let _asked = (service.ask)();
        return AheadLook::Claimed { made, proof, copied_at_millis };
    }
    if let Err(reason) = (service.ask)() {
        debug(TAG, format!("{}: no snapshot ahead was asked for ({reason}), so this add takes its own", plan.destination.display()).as_str());
        note_timing("snapshotAhead", "no-service");
        hand_over(service, &mut stale, &mut handed);
        return AheadLook::None;
    }
    hand_over(service, &mut stale, &mut handed);
    let waiting: bool = begin_step("snapshotAheadWait");
    let claimed: Option<(ObjectId, Proof, u128)> = wait_for_one(source, stage, environment, &mut stale, service.ceiling);
    end_step(waiting);
    // What this add proved stale while it waited, and what it keeps no longer, goes to the service in its own time.
    let now: u128 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis();
    for name in over_kept(kept_in(pool.as_path(), source.root.as_path()).as_slice(), now) {
        // A name the wait already proved stale is handed over once.
        if !stale.contains(&name) {
            stale.push(name);
        }
    }
    hand_over(service, &mut stale, &mut handed);
    // The add after this one needs a copy of its own, so one is asked for while the pool holds fewer than an add keeps.
    //  This is what makes a repository's pool stand ahead of its adds rather than answer them one at a time.
    if kept_in(pool.as_path(), source.root.as_path()).len() < AHEAD_KEPT {
        let _asked = (service.ask)();
    }
    return match claimed {
        Some((made, proof, copied_at_millis)) => {
            note_timing("snapshotAhead", "waited");
            AheadLook::Claimed { made, proof, copied_at_millis }
        }
        None => {
            note_timing("snapshotAhead", "timed-out");
            AheadLook::TimedOut(format!(
                "no snapshot the wrapper makes ahead of an add came within {}, and nothing here waits on one indefinitely",
                in_words(service.ceiling)
            ))
        }
    };
}
