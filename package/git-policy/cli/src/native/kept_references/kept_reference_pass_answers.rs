//! What:
//!  The answers one pass gives about a repository's kept references:
//!  the reasons a reference is pruned,
//!  the reasons it is kept,
//!  the entry a person reads for each,
//!  and the small readings they are built from.
//! Why:
//!  The pass and the listing both speak in these types,
//!  and they sit in a file of their own so the pass that judges and removes
//!  stays inside the line bound the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Pass = { pruned: Pruned[]; kept: Kept[]; bytesBefore: number; bytesAfter: number };
//! ```

use crate::kept_reference_limits::KeptReferenceLimits;
use crate::kept_reference_record::{FIRST_SEEN, KeptReferenceRecord};
use crate::kept_references_reading::OneKeptReference;
use std::path::Path;
use std::time::SystemTime;

/// What:
///  Why one kept reference was pruned.
/// Why:
///  The three answers are what the person reads and what the log line carries,
///  and they are the only reasons the pass has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhyPruned = 'reached-a-branch' | 'too-old' | 'too-much-kept';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhyPruned {
    /// A branch,
    /// a remote-tracking reference or a tag reaches the commit,
    /// so the reference holds nothing.
    ReachedABranch,
    /// The reference is older than the age limit and no branch reaches its commit.
    TooOld,
    /// The objects only kept references reach weighed more than the size limit,
    /// and this was the oldest reference the floor allowed.
    TooMuchKept,
}

impl WhyPruned {
    /// The word the log line and the listing use.
    pub fn word(self) -> &'static str {
        match self {
            WhyPruned::ReachedABranch => return "reached-a-branch",
            WhyPruned::TooOld => return "too-old",
            WhyPruned::TooMuchKept => return "too-much-kept",
        }
    }
}

/// What:
///  Why one kept reference was left where it is.
/// Why:
///   A person who reads the listing needs the reason a reference the wrapper keeps is still there,
///  and the reason is the rule that spared it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhyNotPruned = 'younger-than-the-limit' | 'under-the-size-limit' | 'this-pass-reached-its-cap';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhyNotPruned {
    /// It is younger than the age limit,
    /// and the objects only kept references reach weigh less than the size limit.
    YoungerThanTheLimit,
    /// The objects only kept references reach weigh less than the size limit,
    /// or this reference was spared by the floor that keeps a young one.
    UnderTheSizeLimit,
    /// This pass had already planned `THE_MOST_A_PASS_PRUNES` removals,
    /// so what the limits allow is left to a later pass.
    ThisPassReachedItsCap,
}

/// What:
///  One kept reference the pass removed or would remove,
///  with what the person needs to find out afterwards what went.
/// Why:
///   Removing the reference destroys the only handle on a commit no branch reaches,
///  so the log line and the listing carry the commit,
/// the batch,
/// the worktree it was built in,
///  its age and the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pruned = { reference; commit; batch; rescued; worktree?; keptAtMs; ageMs; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pruned {
    /// The reference that went.
    pub reference: String,
    /// The commit it named.
    pub commit: String,
    /// The batch its name carries.
    pub batch: u64,
    /// Whether it was the name beside the batch's own.
    pub rescued: bool,
    /// The worktree the commit was built in,
    /// or nothing when the release that kept it recorded none.
    pub worktree: Option<String>,
    /// When the reference was written,
    /// in milliseconds since the Unix epoch.
    pub kept_at_ms: u128,
    /// How old it was when this pass read it.
    pub age_ms: u128,
    /// Why it went.
    pub why: WhyPruned,
}

/// What:
///  One kept reference the pass left where it is,
///  with the rule that spared it.
/// Why:
///  The listing says what is kept and what the next pass would take,
///  and both halves are one reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Kept = { reference; commit; batch; rescued; worktree?; keptAtMs; ageMs; whyNot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kept {
    /// The reference that stayed.
    pub reference: String,
    /// The commit it names.
    pub commit: String,
    /// The batch its name carries.
    pub batch: u64,
    /// Whether it is the name beside the batch's own.
    pub rescued: bool,
    /// The worktree the commit was built in,
    /// or nothing when it was not recorded.
    pub worktree: Option<String>,
    /// When it was written,
    /// in milliseconds since the Unix epoch.
    pub kept_at_ms: u128,
    /// How old it was when this pass read it.
    pub age_ms: u128,
    /// The rule that spared it.
    pub why_not: WhyNotPruned,
}

/// What:
///  What one pass found and did.
/// Why:
///  The service logs the figure and the listing prints the whole of it,
///  so one type carries what a person sees and what the log keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ThePass = { pruned: Pruned[]; kept: Kept[]; bytesBefore: number; bytesAfter: number; dryRun: boolean; stoppedEarly: boolean };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ThePass {
    /// The references the pass removed,
    /// or would remove.
    pub pruned: Vec<Pruned>,
    /// The references it left,
    /// with the rule that spared each.
    pub kept: Vec<Kept>,
    /// The bytes the objects only kept references reached before it read them.
    pub bytes_before: u64,
    /// The bytes the pass left,
    /// or would leave.
    pub bytes_after: u64,
    /// Whether the pass removed nothing because a person asked what it would do.
    pub dry_run: bool,
    /// Whether the pass stopped at its cap or its bound with work the limits allowed still to do.
    pub stopped_early: bool,
}

/// What:
///  The current time in milliseconds since the Unix epoch.
/// Why:
///  The age of a kept reference is a subtraction of two readings,
///  and the pass needs one reading for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nowMs = () => Date.now();
/// ```
pub fn now_ms() -> u128 {
    return match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_before_the_epoch) => 0,
    };
}

/// What:
///  How old `record` was at `now`,
///  never a negative age.
/// Why:
///   A clock that stepped back leaves a record from the future,
///  and an age of nothing keeps that reference,
///  which is the safe side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ageOf = (record, now) => Math.max(0, now - record.keptAtMs);
/// ```
pub fn age_of(record: &KeptReferenceRecord, now: u128) -> u128 {
    return now.saturating_sub(record.kept_at_ms);
}

/// What:
///  When one kept reference was written,
///  as its record says,
///  and the record written for it when the repository holds none.
/// Why:
///   A reference written by an earlier release has no age anywhere,
///  and its commit's date is older than the moment it was kept,
///  so dating it from the first pass that lists it is the one reading that never prunes early.
///  Its worktree is unknown then,
///  and the record says so by leaving it out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = readRecord(commonDir, name) ?? writeIfAbsent(commonDir, firstSeen(one, now));
/// ```
pub fn recorded_or_first_seen(common_dir: &Path, one: &OneKeptReference, now: u128) -> Option<KeptReferenceRecord> {
    if let Some(found) = crate::kept_reference_record::read_record(common_dir, one.reference.as_str()) {
        return Some(found);
    }
    let candidate: KeptReferenceRecord = KeptReferenceRecord {
        reference: one.reference.clone(),
        commit: one.commit.clone(),
        batch: one.batch,
        rescued: one.rescued,
        worktree: None,
        kept_at_ms: now,
        why: String::from(FIRST_SEEN),
    };
    return crate::kept_reference_record::write_if_absent(common_dir, &candidate);
}

/// What:
///  The commits of the references that would stay,
/// as a size question asks about them.
/// Why:
///  The size limit is about what the references that stay still hold,
///  so every question about it is asked of the survivors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitsOf = (ones, planned) => ones.filter((one, at) => !planned[at]).map((one) => one.commit);
/// ```
pub fn commits_of_survivors(kept: &[OneKeptReference], planned: &[bool]) -> Vec<String> {
    let mut commits: Vec<String> = Vec::new();
    for (at, one) in kept.iter().enumerate() {
        if !planned[at] {
            commits.push(one.commit.clone());
        }
    }
    return commits;
}

/// What:
///  The time one record says its reference was written,
///  or the fallback a caller gives for a reference the repository records none of.
/// Why:
///   A reference an earlier release wrote has no record until a pass writes one,
///  and a time of nothing would make it the oldest thing in the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptAtOr = (record, fallback) => record ? record.keptAtMs : fallback;
/// ```
fn kept_at_or(record: Option<&KeptReferenceRecord>, fallback: u128) -> u128 {
    return match record {
        Some(held) => held.kept_at_ms,
        None => fallback,
    };
}

/// What:
///  The age of one record at `now`,
///  or nothing when there is no record.
/// Why:
///   Every rule asks a reference's age,
///  and a reference the repository recorded none of has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ageOrNothing = (record, now) => record ? Math.max(0, now - record.keptAtMs) : 0;
/// ```
fn age_or_nothing(record: Option<&KeptReferenceRecord>, now: u128) -> u128 {
    return match record {
        Some(held) => age_of(held, now),
        None => 0,
    };
}

/// What:
///  The worktree one record names,
///  or nothing when it names none.
/// Why:
///  The listing and the log line print the worktree a commit was built in,
///  and a record written by a release that knew none says so by leaving it out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktreeOf = (record) => record?.worktree;
/// ```
fn worktree_of(record: Option<&KeptReferenceRecord>) -> Option<String> {
    return match record {
        Some(held) => held.worktree.clone(),
        None => None,
    };
}

/// What:
///  The age of one kept reference,
///  as the record says,
///  and nothing when the repository holds no record of it.
/// Why:
///  The three rules ask the same age three times,
///  and a reference with no record has none to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ageAt = (records, at, now) => records[at] ? ageOf(records[at], now) : 0;
/// ```
pub fn age_at(records: &[Option<KeptReferenceRecord>], at: usize, now: u128) -> u128 {
    return age_or_nothing(records[at].as_ref(), now);
}

/// What:
///  The oldest kept reference the floor allows a pass to remove for its size,
///  or nothing when every one that is left is younger than the floor.
/// Why:
///  "The oldest go first" is what makes the size rule predictable,
///  and the floor is what keeps a commit kept a moment ago out of its reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oldest = (ones, planned) => ones.filter((one, at) => !planned[at] && age(at) > floor).sortBy(keptAt)[0];
/// ```
pub fn oldest_the_floor_allows(reading: &[OneKeptReference], records: &[Option<KeptReferenceRecord>], planned: &[bool], limits: KeptReferenceLimits, now: u128) -> Option<usize> {
    let mut chosen: Option<usize> = None;
    for at in 0..reading.len() {
        if planned[at] || age_at(records, at, now) <= limits.floor.as_millis() {
            continue;
        }
        let candidate: u128 = kept_at_or(records[at].as_ref(), u128::MAX);
        // The earliest time kept among those already passed over is the one to beat.
        let held_at: u128 = match chosen {
            Some(held) => kept_at_or(records[held].as_ref(), u128::MAX),
            None => u128::MAX,
        };
        if chosen.is_none() || candidate < held_at {
            chosen = Some(at);
        }
    }
    return chosen;
}

/// What:
///  One kept reference the pass left,
///  with the rule that spared it.
/// Why:
///  The listing of a reference that stayed says which of the rules spared it,
///  and a reference the pass planned to take but Git refused says so as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptOne = (one, record, now) => ({ ...one, keptAtMs, ageMs, whyNot });
/// ```
pub fn kept_one(one: &OneKeptReference, record: Option<&KeptReferenceRecord>, now: u128, limits: KeptReferenceLimits, planned: bool, cap_reached: bool) -> Kept {
    let kept_at_ms: u128 = kept_at_or(record, now);
    let age_ms: u128 = age_or_nothing(record, now);
    let why_not: WhyNotPruned = if planned && cap_reached {
        WhyNotPruned::ThisPassReachedItsCap
    } else if age_ms <= limits.max_age.as_millis() {
        WhyNotPruned::YoungerThanTheLimit
    } else {
        WhyNotPruned::UnderTheSizeLimit
    };
    return Kept {
        reference: one.reference.clone(),
        commit: one.commit.clone(),
        batch: one.batch,
        rescued: one.rescued,
        worktree: worktree_of(record),
        kept_at_ms,
        age_ms,
        why_not,
    };
}

/// What:
///  One planned reference as the person and the log read it.
/// Why:
///  The removal and the record are two steps of one decision,
///  and the entry is built once so the log line and the listing cannot differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const prunedOne = (one, record, now, why) => ({ ...one, keptAtMs, ageMs, why });
/// ```
pub fn pruned_one(one: &OneKeptReference, record: Option<&KeptReferenceRecord>, now: u128, why: WhyPruned) -> Pruned {
    return Pruned {
        reference: one.reference.clone(),
        commit: one.commit.clone(),
        batch: one.batch,
        rescued: one.rescued,
        worktree: worktree_of(record),
        kept_at_ms: kept_at_or(record, now),
        age_ms: age_or_nothing(record, now),
        why,
    };
}
