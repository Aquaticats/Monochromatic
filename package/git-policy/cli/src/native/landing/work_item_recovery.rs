//! What:
//!  The recovery half of the work-item record:
//! the claims whose lease ran out go back to `pending` (or to `failed`),
//! and a
//!  killed recovery's leftovers are removed.
//! Why:
//!  A consumer killed at any instruction is noticed by its lease,
//! never by a process id;
//! the code is apart from `work_item.rs`
//!  so each file stays within the size limit,
//! and `work_item.rs` re-exports what callers use.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const stale of claimed.filter(expired)) requeue(stale)
//! ```

use crate::claim_time::ClaimTime;
use crate::private_storage::create_private_directory_all;
use crate::service_timing::log;
use crate::work_item::{
    CLAIMED, FAILED, PENDING, RECOVERING, WorkItem, claim_file_name, entries_oldest_first, id_of_entry, is_valid_id,
    new_generation, parse_claim_name, state_directory,
};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The most a stray temporary file of a recovery may age before a recovery removes it.
/// Measured:
/// the longest wrapper command in the
///  commit timing log (1,051 commits,
/// 2026-11-02T09:01:37Z to 2026-11-03T15:10:12Z) took 750.6 s and the longest worktree add in the
///  add timing log (48 adds,
/// 2026-11-03T08:55Z to 15:06Z) 539.7 s,
/// so a day is about a hundred times the longest.
/// Kept as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const STRAY_TEMPORARY_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// The end of the name of a recovery's temporary file.
pub const TEMPORARY_SUFFIX: &str = ".retry";

/// What:
///  Return claims whose lease ran out to `pending` with one more try,
/// or to `failed` at `max_tries`.
///  Returns how many were moved.
/// Why:
///   This is how a consumer killed at any instruction is noticed when nothing says its successor started:
/// by its lease,
///  never by a process id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const stale of claimed.filter((c) => now.elapsedSince(c.stamp) > lease)) requeue(stale)
/// ```
pub fn requeue_expired(work_dir: &Path, now: &ClaimTime, lease: Duration, max_tries: u32) -> usize {
    return requeue_expired_reporting(work_dir, now, lease, max_tries).moved;
}

/// What:
///  What a recovery pass did:
/// how many claims it moved,
/// and the items it gave up on.
/// Why:
///   An item that reaches `failed` through recovery was never settled by a handler,
/// so nothing recorded a notice for it;
/// the
///  caller needs the items to tell the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Recovered = { moved: number; givenUp: WorkItem[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Recovered {
    /// How many claims this pass moved to `pending` or `failed`.
    pub moved: usize,
    /// The decoded items it moved to `failed` because they reached the try limit.
    pub given_up: Vec<WorkItem>,
}

/// What:
///  Whether a claim stamped `stamp` has expired.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The recovery takes its expiry test as a value,
/// so a control can stand in for it;
/// the state the test needs (the reading and the
///  lease) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ClaimExpiry { hasExpired(stamp: ClaimTime): boolean }
/// ```
pub(crate) trait ClaimExpiry {
    /// Whether the claim stamped `stamp` has expired.
    fn has_expired(&self, stamp: &ClaimTime) -> bool;
}

/// A claim has expired when it is of an earlier boot,
/// or older than the lease.
struct LeaseRanOut<'a> {
    /// The reading the recovery runs at.
    now: &'a ClaimTime,
    /// How long a claim lasts without a renewal.
    lease: Duration,
}

impl ClaimExpiry for LeaseRanOut<'_> {
    fn has_expired(&self, stamp: &ClaimTime) -> bool {
        return self.now.is_past_the_boot_of(stamp) || self.now.elapsed_since(stamp) > self.lease;
    }
}

/// `requeue_expired` that also gives the items it moved to `failed` for want of tries.
pub fn requeue_expired_reporting(work_dir: &Path, now: &ClaimTime, lease: Duration, max_tries: u32) -> Recovered {
    return requeue_with(work_dir, now, max_tries, &LeaseRanOut { now, lease });
}

/// What:
///  `requeue_expired_reporting` with a step told about each item that is given up before the item moves to `failed`.
/// Why:
///   The notice that names an abandoned push is the person's only remaining word about it,
/// and a record in `failed` is scanned by nothing:
/// a kill between the move and the notice would leave the branch abandoned in silence.
/// The item is told about while it is in `recovering` under this instance's own name,
/// where no other instance can touch it,
/// and a kill after the notice leaves it to be recovered once more,
/// which replaces the notice instead of losing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// requeueExpiredTelling(workDir, now, lease, maxTries, (item) => recordPushSkip(item))
/// ```
pub(crate) fn requeue_expired_reporting_telling(work_dir: &Path, now: &ClaimTime, lease: Duration, max_tries: u32, tell: &dyn GivenUpNotice) -> Recovered {
    return requeue_with_telling(work_dir, now, max_tries, &LeaseRanOut { now, lease }, tell);
}

/// What:
///  How long until the first claim that is still held runs out its lease,
/// or nothing when no claim is held.
/// Why:
///   An instance that finds claims it may not take yet has nothing to wake it when they run out,
/// so the service asks when to
///  look again;
/// without this,
/// work whose holder died would wait for the next commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.min(...claimed.map((c) => lease - now.elapsedSince(c.stamp)))
/// ```
pub fn next_claim_expiry(work_dir: &Path, now: &ClaimTime, lease: Duration) -> Option<Duration> {
    // The least of what every held claim has left; the temporary files of a recovery are not claims.
    let mut least: Option<Duration> = None;
    for state in [CLAIMED, RECOVERING] {
        for (name, modified) in entries_oldest_first(work_dir, state) {
            if name.ends_with(TEMPORARY_SUFFIX) {
                continue;
            }
            let left: Duration = lease_left(now, &stamp_of_entry(name.as_str(), modified), lease);
            least = Some(match least {
                Some(known) => known.min(left),
                None => left,
            });
        }
    }
    return least;
}

/// How much of `lease` a claim stamped `stamp` has left at `now`:
/// none for a claim of an earlier boot,
/// whose holder is dead.
fn lease_left(now: &ClaimTime, stamp: &ClaimTime, lease: Duration) -> Duration {
    if now.is_past_the_boot_of(stamp) {
        return Duration::ZERO;
    }
    return lease.saturating_sub(now.elapsed_since(stamp));
}

/// The reading in a claim's file name,
/// or for a claim by an earlier release the wall-clock time of its last stamp.
fn stamp_of_entry(name: &str, modified: SystemTime) -> ClaimTime {
    return match parse_claim_name(name) {
        Some((_token, stamp)) => stamp,
        None => ClaimTime::from_wall(modified),
    };
}

/// What:
///  Take each claim whose reading `expired` accepts,
/// by renaming the exact file name that was looked at into `recovering`
///  under a generation of its own,
/// then return it to `pending` with one more try,
/// or to `failed` at `max_tries`.
/// Returns how
///  many were moved.
/// Why:
///   The look and the take are separate moments,
/// and in between another instance may have returned the same claim and claimed
///  it again,
/// or its holder may have renewed it.
/// Both change the name,
/// so the take of the old name finds nothing and takes
///  nothing:
/// the rename is the only check and it is atomic.
/// The file in `recovering` is this instance's alone,
/// so its record
///  is rewritten there whole (a temporary file renamed over it) and the item reaches `pending` by one rename;
/// a recovery
///  killed anywhere leaves the item in exactly one place,
/// and an item left in `recovering` runs out its lease and is
///  recovered like a claim,
/// which costs it one more try.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const seen of claimed.filter(expired)) { if (await tryRename(seen.name, owned)) { await rewrite(owned); await rename(owned, pending) } }
/// ```
pub(crate) fn requeue_with(work_dir: &Path, now: &ClaimTime, max_tries: u32, expired: &dyn ClaimExpiry) -> Recovered {
    return requeue_with_telling(work_dir, now, max_tries, expired, &NoNotice);
}

/// What:
///  Who a recovery tells about the items it gives up on.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The notice that names an abandoned push needs the Git directory and writes a file,
/// and a control stands in for it to see when it is called;
/// a value keeps that step out of the recovery itself.
/// The answer says whether the person was told:
/// a
///  push is not settled while its notice could not be written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface GivenUpNotice { tell(givenUp: WorkItem): boolean }
/// ```
pub(crate) trait GivenUpNotice {
    /// Tell the person about `given_up`,
    /// before the record moves to `failed`,
    /// and answer whether the words are where a command will show them.
    fn tell(&self, given_up: &WorkItem) -> bool;
}

/// The step of a recovery that tells nobody about the items it gives up,
/// which is every caller that writes no notice.
struct NoNotice;

impl GivenUpNotice for NoNotice {
    fn tell(&self, _given_up: &WorkItem) -> bool {
        return true;
    }
}

/// `requeue_with` with the step told about each item it gives up,
/// before the item moves to `failed`.
pub(crate) fn requeue_with_telling(work_dir: &Path, now: &ClaimTime, max_tries: u32, expired: &dyn ClaimExpiry, tell: &dyn GivenUpNotice) -> Recovered {
    let mut recovered: Recovered = Recovered::default();
    for state in [CLAIMED, RECOVERING] {
        for (name, modified) in entries_oldest_first(work_dir, state) {
            if name.ends_with(TEMPORARY_SUFFIX) {
                remove_stray_temporary(work_dir, state, name.as_str(), modified, now);
                continue;
            }
            if !expired.has_expired(&stamp_of_entry(name.as_str(), modified)) {
                continue;
            }
            if let Some(given_up) = recover(work_dir, state, name.as_str(), now, max_tries, tell) {
                recovered.moved += 1;
                recovered.given_up.extend(given_up);
            }
        }
    }
    return recovered;
}

/// Removes a recovery's temporary file that a killed recovery left,
/// once it is old enough that nobody can still be using it.
fn remove_stray_temporary(work_dir: &Path, state: &str, name: &str, modified: SystemTime, now: &ClaimTime) {
    if now.elapsed_since(&ClaimTime::from_wall(modified)) > STRAY_TEMPORARY_AGE {
        let _removed = std::fs::remove_file(state_directory(work_dir, state).join(name));
    }
}

/// Returns the claim file `name` of `state` to `pending` or `failed`;
/// nothing when another instance moved it first,
/// otherwise
/// the item when it was given up on (moved to `failed` for want of tries).
fn recover(work_dir: &Path, state: &str, name: &str, now: &ClaimTime, max_tries: u32, tell: &dyn GivenUpNotice) -> Option<Option<WorkItem>> {
    let id: &str = id_of_entry(name);
    // A claim by an earlier release was not checked when it was queued;
    // an id whose names would not fit goes to `failed` by its
    //  own name,
    // since the longer name of this release's claim or of its temporary file could not be made.
    if !is_valid_id(id) {
        let moved: bool = create_private_directory_all(state_directory(work_dir, FAILED).as_path()).is_ok()
            && std::fs::rename(state_directory(work_dir, state).join(name), state_directory(work_dir, FAILED).join(id)).is_ok();
        log(format!(
            "the claim {name:?} is not of a kind and a suffix of at most {} characters, so it was {}",
            crate::work_item_ids_and_names::MAX_ID_SUFFIX_LENGTH,
            if moved { "moved to failed" } else { "left where it is because moving it failed" }
        )
        .as_str());
        return moved.then_some(None);
    }
    let Ok(token) = new_generation() else {
        return None;
    };
    let owned_name: String = claim_file_name(id, token.as_str(), now);
    let recovering: PathBuf = state_directory(work_dir, RECOVERING);
    if create_private_directory_all(recovering.as_path()).is_err() {
        return None;
    }
    let owned: PathBuf = recovering.join(owned_name.as_str());
    // The take: only one instance can rename this exact name, and a claim that was renewed or claimed again is not this name.
    if std::fs::rename(state_directory(work_dir, state).join(name), owned.as_path()).is_err() {
        return None;
    }
    // A record that cannot be read after it was taken is settled like one that cannot be decoded: leaving it in `recovering`
    //  would have every later pass take it, fail the same way, and keep the service waiting on it for good.
    let Ok(text) = std::fs::read_to_string(owned.as_path()) else {
        return std::fs::rename(owned.as_path(), state_directory(work_dir, FAILED).join(id)).is_ok().then_some(None);
    };
    let (target, given_up): (&str, Option<WorkItem>) = match WorkItem::decode(text.as_str()) {
        Ok(mut item) if item.id() == id || item.id_before_the_branch() == id => {
            item.tries += 1;
            let rewritten: PathBuf = recovering.join(format!("{owned_name}{TEMPORARY_SUFFIX}"));
            let destination: &str = if item.tries >= max_tries { FAILED } else { PENDING };
            if std::fs::write(rewritten.as_path(), item.encode()).is_err() || std::fs::rename(rewritten.as_path(), owned.as_path()).is_err() {
                return None;
            }
            // The word about a push given up on comes before the move: nothing scans `failed`, so a kill in between would
            //  abandon the branch in silence. The item is this instance's alone in `recovering` here.
            //  A word that cannot be written does not settle it either: the item stays in `recovering`, where a later pass
            //  tries the notice again and replaces it rather than losing it.
            if destination == FAILED && !tell.tell(&item) {
                return None;
            }
            (destination, (destination == FAILED).then_some(item))
        }
        // The record never changes once it is pending,
        // so this is a record that was never readable,
        // or one of other work than
        //  its name says,
        // which a handler would run and settle under the wrong id.
        _ => (FAILED, None),
    };
    return std::fs::rename(owned.as_path(), state_directory(work_dir, target).join(id)).is_ok().then_some(given_up);
}
