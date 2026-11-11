//! What:
//!  The durable work-item record:
//! a small file in the Git directory that says "this landed object needs this
//!  asynchronous work",
//! written by the wrapper and drained by a separate resident consumer.
//! Why:
//!  A wrapper process exits when its command ends,
//! and it may start no program,
//! so work that must not delay a landing
//!  (the standards review,
//! later auto-push and maintenance) is left as a record instead.
//! The record is generic:
//! a kind,
//!  the landed object and an id derived from both,
//! so a retry cannot duplicate it.
//! The design,
//! its failure cases and
//!  the consumer contract are in `doc/planning/cli-git-standards-review-port.md`.
//! Nothing calls this yet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await enqueue(workDir, { kind: 'review', oid, ... }); const item = await claimNext(workDir); await finish(workDir, item);
//! ```

use crate::claim_time::ClaimTime;
use crate::path_bytes_in_records as path_bytes;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::service_timing::log;
pub use crate::work_item_kinds_and_their_names::WorkKind;
pub use crate::work_item_state_directories_and_claim_names::{CLAIMED, DONE, FAILED, HELD, PENDING, RECOVERING, claim_file_name, id_of_entry, state_directory};
pub(crate) use crate::work_item_state_directories_and_claim_names::{entries_oldest_first, new_generation, parse_claim_name};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The record format version.
pub const VERSION: u64 = 1;

/// Directory under the worktree's own Git directory (`WorkItem::git_dir`) that holds the state directories.
pub const WORK_DIRECTORY: &str = "monochromatic-git-policy-cli/work";

/// What:
///  One work item:
/// what to do,
/// to which landed object,
/// and where.
/// Why:
///   The consumer needs only the object name and the places to find the repository;
/// it reads the patch itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorkItem = { kind; repository: string; gitDir: string; ref: string; oid: string; parent: string; created: string; tries: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkItem {
    /// What to do.
    pub kind: WorkKind,
    /// The worktree root.
    pub repository: PathBuf,
    /// The Git directory of the worktree that made the commit and whose service instance runs the item:
    /// the common directory
    ///  for the main worktree,
    /// `<common>/worktrees/<name>` for a linked one.
    /// The work records are under it,
    /// and
    ///  `is_acceptable` refuses an item whose value is not the directory it is read from.
    pub git_dir: PathBuf,
    /// The branch reference that received the commit.
    pub branch_ref: String,
    /// The landed commit.
    pub oid: String,
    /// Its first parent,
    /// or empty for a root commit.
    pub parent: String,
    /// `toISOString` text of the enqueue time.
    pub created: String,
    /// Failed or abandoned attempts so far.
    pub tries: u32,
}

/// Methods over an item.
impl WorkItem {
    /// What:
    ///  The id and file name:
    /// the kind,
    /// the landed object,
    /// and the one field of the record that tells two of them apart,
    /// so the same work always yields the same id.
    /// Why:
    ///   A push names its branch,
    /// because the same commit on two branches is two pushes to two remotes,
    /// and a subvolume delete names the directory that held it,
    /// because a
    ///  subvolume id is unique on one filesystem only.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const id = (item) =>
    ///   item.kind === 'push'
    ///     ? `${item.kind}-${item.oid}-${branchDigest(item.ref)}`
    ///     : item.kind === 'subvolume-delete'
    ///       ? `${item.kind}-${item.oid}-${parentDigest(item.parent)}`
    ///       : `${item.kind}-${item.oid}`
    /// ```
    pub fn id(&self) -> String {
        if self.kind == WorkKind::Push {
            return format!("{}-{}-{}", self.kind.name(), self.oid, crate::work_item_ids_and_names::branch_digest(self.branch_ref.as_str()));
        }
        if self.kind == WorkKind::SubvolumeDelete {
            return format!("{}-{}-{}", self.kind.name(), self.oid, crate::work_item_ids_and_names::parent_digest(self.parent.as_str()));
        }
        return format!("{}-{}", self.kind.name(), self.oid);
    }

    /// What:
    ///  The id a release that keyed a record by its kind and object alone wrote for this item.
    /// Why:
    ///   Such a record is on disk in a repository this release took over,
    /// and it is still this item's:
    /// the record's
    ///  name must not decide that a push,
    /// or a subvolume delete,
    /// left over from the release before this one is other
    ///  work.
    /// The
    ///  current id is what this release writes,
    /// so the two are asked for together wherever a name is matched against a record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `${kind}-${oid}`
    /// ```
    pub fn id_before_the_branch(&self) -> String {
        return format!("{}-{}", self.kind.name(), self.oid);
    }

    /// The record's one JSON line.
    pub fn encode(&self) -> String {
        let mut fields: Map<String, Value> = Map::new();
        fields.insert(String::from("v"), Value::from(VERSION));
        fields.insert(String::from("tries"), Value::from(self.tries));
        // A directory goes in as plain text and as its bytes, so a reader of either form finds the one the wrapper named.
        path_bytes::write_field(&mut fields, "repository", self.repository.as_path());
        path_bytes::write_field(&mut fields, "gitDir", self.git_dir.as_path());
        for (name, text) in [
            ("kind", self.kind.name()),
            ("id", self.id().as_str()),
            ("ref", self.branch_ref.as_str()),
            ("oid", self.oid.as_str()),
            ("parent", self.parent.as_str()),
            ("created", self.created.as_str()),
        ] {
            fields.insert(String::from(name), Value::from(text));
        }
        return format!("{}\n", Value::Object(fields));
    }

    /// Read a record;
    /// any other version,
    /// an unknown kind,
    /// a missing field or an id that is not derived from the kind and
    /// object is refused with a message.
    pub fn decode(text: &str) -> Result<WorkItem, String> {
        let value: Value = match serde_json::from_str(text) {
            Ok(parsed) => parsed,
            Err(error) => return Err(format!("the record is not JSON: {error}")),
        };
        if value.get("v").and_then(Value::as_u64) != Some(VERSION) {
            return Err(String::from("the record has an unsupported version"));
        }
        let kind: WorkKind = match WorkKind::from_name(text_field(&value, "kind")?.as_str()) {
            Some(known) => known,
            None => return Err(String::from("the record has an unknown kind")),
        };
        // A count that is missing, or too large for a count of tries, is no count.
        let Some(count) = value.get("tries").and_then(Value::as_u64) else {
            return Err(String::from("the record has no tries"));
        };
        let Ok(tries) = u32::try_from(count) else {
            return Err(String::from("the record has no tries"));
        };
        let item: WorkItem = WorkItem {
            kind,
            repository: path_bytes::read_field(&value, "repository")?,
            git_dir: path_bytes::read_field(&value, "gitDir")?,
            branch_ref: text_field(&value, "ref")?,
            oid: text_field(&value, "oid")?,
            parent: text_field(&value, "parent")?,
            created: text_field(&value, "created")?,
            tries,
        };
        let named: String = text_field(&value, "id")?;
        if named != item.id() && named != item.id_before_the_branch() {
            return Err(String::from("the record's id is not derived from its kind, object and branch"));
        }
        return Ok(item);
    }
}

/// What:
///  Whether an enqueue added the item or found it already known.
/// Why:
///   A landing that is replayed,
/// or a daemon that restarts,
/// enqueues again;
/// the second time must change nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Enqueued = 'added' | 'known';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Enqueued {
    /// A new pending item was written.
    Added,
    /// The id already exists in some state directory.
    Known,
}

/// What:
///  A named field of a work record as text.
/// Why:
///  `WorkItem::decode` reads every text field of the record,
/// and a field that is missing or not text refuses the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const textField = (value: object, name: string): string => { if (typeof value[name] !== 'string') throw new Error(`the record has no text field ${name}`); return value[name]; };
/// ```
fn text_field(value: &Value, name: &str) -> Result<String, String> {
    return match value.get(name).and_then(Value::as_str) {
        Some(text) => Ok(String::from(text)),
        None => Err(format!("the record has no text field {name}")),
    };
}

/// What:
///  The ids of the items that hold a claim file in `claimed` or `recovering`,
/// from one listing of each.
/// Why:
///  `enqueue_all` treats an item that is being worked on as already known,
/// and a round of items must answer that
///  for all of them from one listing per directory rather than one per item (`RG2`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Set([CLAIMED, RECOVERING].flatMap((state) => entriesOldestFirst(workDir, state).map(([name]) => idOfEntry(name))))
/// ```
fn claimed_or_recovering_ids(work_dir: &Path) -> HashSet<String> {
    let mut held: HashSet<String> = HashSet::new();
    for state in [CLAIMED, RECOVERING] {
        for (name, _modified) in entries_oldest_first(work_dir, state) {
            held.insert(String::from(id_of_entry(name.as_str())));
        }
    }
    return held;
}

/// What:
///  Write `item` as a pending record,
/// whole or absent,
/// unless its id is already in any state,
/// and answer whether it was added.
///  The producer never waits for a consumer and never reads anything back.
/// Why:
///   The id is the idempotency key;
/// the exclusive publish makes two racing producers agree on one winner.
/// This is `enqueue_all` for one item.
///
/// # Errors
///
/// As `enqueue_all`.
pub fn enqueue(work_dir: &Path, item: &WorkItem) -> Result<Enqueued, String> {
    let answers: Vec<Enqueued> = enqueue_all(work_dir, std::slice::from_ref(item))?;
    let Some(answer) = answers.first() else {
        return Err(format!("the round of the one item {} answered nothing", item.id()));
    };
    return Ok(*answer);
}

pub use crate::work_item_holding::{block, forget_held, hold, promote, record_pass, record_refusal, record_verdict, requeue_after_giving_up};
pub use crate::work_item_queued_ids::queued_ids;

/// What:
///  Write every item of one producer's round as a pending record,
/// whole or absent,
/// unless its id is already in any state,
/// answering what happened to each in the order given.
/// Why:
///   The id is the idempotency key;
/// the exclusive publish makes two racing producers agree on one winner.
/// A round that asked `claimed` and `recovering` once per item cost one listing
///  per item,
/// so a producer of N items cost N listings (`RG2`);
/// the round takes each of those listings once.
/// The price is the window the listing opens:
/// an item that a consumer claims
///  after the round listed the claims is queued again,
/// because nothing reads the queue back.
///
/// # Errors
///
/// A state directory that cannot be made,
/// or a pending record that cannot be written for a reason other than its id
/// being there already.
pub fn enqueue_all(work_dir: &Path, items: &[WorkItem]) -> Result<Vec<Enqueued>, String> {
    for state in [PENDING, CLAIMED, DONE, FAILED] {
        if let Err(error) = create_private_directory_all(state_directory(work_dir, state).as_path()) {
            return Err(format!("creating the {state} directory failed: {error}"));
        }
    }
    // The claims are listed once for the whole round; `done` and `failed` are named by the id alone, so each item asks
    //  about them by name and no listing sees them.
    let mut held: HashSet<String> = claimed_or_recovering_ids(work_dir);
    let mut answers: Vec<Enqueued> = Vec::with_capacity(items.len());
    for item in items {
        let id: String = item.id();
        // A round holds every id once: a second item of one id finds the first one's record where the round put it.
        if held.contains(id.as_str())
            || state_directory(work_dir, DONE).join(id.as_str()).exists()
            || state_directory(work_dir, FAILED).join(id.as_str()).exists()
        {
            answers.push(Enqueued::Known);
            continue;
        }
        match write_private_file_unsynced(state_directory(work_dir, PENDING).join(id.as_str()).as_path(), item.encode().as_bytes()) {
            Ok(()) => {
                held.insert(id);
                answers.push(Enqueued::Added);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => answers.push(Enqueued::Known),
            Err(error) => return Err(format!("writing the work item {id} failed: {error}")),
        }
    }
    return Ok(answers);
}

/// What:
///  A held item:
/// the decoded record,
/// its id,
/// and the name of the file that holds it.
/// Why:
///   Finishing needs the claim's file name,
/// not the id:
/// the name carries the generation of the claim,
/// so a settlement or a
///  renewal reaches only the claim it was made for.
/// The consumer needs the item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Claimed = { id: string; name: string; item: WorkItem };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claimed {
    /// File name in every state directory but `claimed` and `recovering`,
    /// where the file name is `name`.
    pub id: String,
    /// File name in `claimed` as of the claim or the last renewal:
    /// the id,
    /// the claim's generation and the clock reading.
    pub name: String,
    /// The decoded record.
    pub item: WorkItem,
}

/// The shape of an id,
/// and the length an object name may have,
/// beside the digest they work with.
pub(crate) use crate::work_item_ids_and_names::is_valid_id;
#[cfg(test)]
pub(crate) use crate::work_item_ids_and_names::MAX_OBJECT_NAME_LENGTH;

/// What:
///  Set the modification time of the file at `path` to now.
/// Why:
///   A release that cannot read the reading in a claim's name ages the claim by the file's modification time,
/// which a rename
///  keeps.
/// Claims and renewals are renames,
/// so without this such a reader sees the time the item was queued,
/// not the last
///  renewal,
/// and takes a live holder's claim.
/// A failure is returned,
/// because a claim or renewal that went on without the stamp
///  would be one that such a reader takes (a file another user owns cannot be stamped).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await utimes(path, new Date(), new Date())
/// ```
fn stamp_modification_time(path: &Path) -> std::io::Result<()> {
    return std::fs::File::open(path)?.set_modified(SystemTime::now());
}

/// How a claim or renewal stamps the file it holds.
type Stamp<'a> = &'a dyn Fn(&Path) -> std::io::Result<()>;

/// The path of the file that holds the claim of `id` now,
/// or nothing when it is not claimed or being recovered.
pub fn find_claim(work_dir: &Path, id: &str) -> Option<PathBuf> {
    // `claimed` first, then `recovering`, each oldest first; the first entry that is the claim of `id`.
    for state in [CLAIMED, RECOVERING] {
        for (name, _modified) in entries_oldest_first(work_dir, state) {
            if id_of_entry(name.as_str()) == id {
                return Some(state_directory(work_dir, state).join(name));
            }
        }
    }
    return None;
}

/// What:
///  Claim the oldest pending item by renaming it into `claimed` under a name that carries a fresh generation and the
///  reading `now`.
/// A record that does not decode is moved to `failed` so it cannot block the queue.
/// Why:
///   A rename is atomic:
/// of two consumers racing for one item,
/// the loser sees the file gone and takes the next one.
/// The
///  generation and the reading are in the name,
/// so the claim is whole from the instant it exists;
/// nothing is written to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'pending', id), join(workDir, 'claimed', claimFileName(id, newGeneration(), now)))
/// ```
pub fn claim_next(work_dir: &Path, now: &ClaimTime) -> Result<Option<Claimed>, String> {
    return claim_next_of(work_dir, now, &[WorkKind::Review, WorkKind::Push, WorkKind::Maintenance, WorkKind::Trash, WorkKind::Prewarm, WorkKind::Relink, WorkKind::SubvolumeDelete, WorkKind::SnapshotAhead]);
}

/// What:
///  Claim the oldest pending item of the given kinds,
/// leaving every other kind where it is.
/// Why:
///   Consumers of different kinds share one queue;
/// the kind starts the file name,
/// so a consumer never opens,
/// moves or
///   stamps an item that belongs to another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'pending', id), join(workDir, 'claimed', name)) // only ids that start with a handled kind
/// ```
pub fn claim_next_of(work_dir: &Path, now: &ClaimTime, kinds: &[WorkKind]) -> Result<Option<Claimed>, String> {
    return claim_next_stamped(work_dir, now, kinds, &stamp_modification_time);
}

/// What:
///  Claims every pending item of the given kinds,
/// from one listing of `pending`,
/// oldest first.
/// Why:
///   A drain that claims its work with `claim_next_of` lists and sorts `pending` for each claim,
/// so a queue of N items
///  costs N listings and N sorts (`RG2`).
/// The round takes the listing once and tries every entry of it,
/// so a drain of N
///  items costs one listing.
/// Each claim still carries the reading of the moment it is taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimed = []; for (const name of entriesOldestFirst(workDir, 'pending')) { const item = await take(name, now()); if (item) claimed.push(item) }; return claimed
/// ```
pub fn claim_all_of(work_dir: &Path, kinds: &[WorkKind]) -> Result<Vec<Claimed>, String> {
    let mut claimed: Vec<Claimed> = Vec::new();
    for (id, queued) in entries_oldest_first(work_dir, PENDING) {
        // Each claim carries the reading of the moment it is taken, not of the start of the round, which a long round makes
        //  most of a lease old.
        if let Some(taken) = take_entry(work_dir, id.as_str(), queued, kinds, &ClaimTime::read(), &stamp_modification_time)? {
            claimed.push(taken);
        }
    }
    return Ok(claimed);
}

/// What:
///  `claim_next_of` with the stamp of the claim's modification time given,
/// so a test can make the stamp fail.
/// Why:
///   The entry is taken into `recovering` under its claim name first,
/// where no reader of `claimed` looks and where the
///  entry is this instance's alone;
/// only there is it stamped,
/// and only then published into `claimed`.
/// A claim that cannot be
///  stamped or published goes back to `pending` with the modification time it had,
/// so a queue that cannot be drained keeps
///  its true age and the backlog notice still fires.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(pending, staged);
/// await stamp(staged);
/// await rename(staged, join(workDir, 'claimed', name))
/// ```
fn claim_next_stamped(work_dir: &Path, now: &ClaimTime, kinds: &[WorkKind], stamp: Stamp<'_>) -> Result<Option<Claimed>, String> {
    for (id, queued) in entries_oldest_first(work_dir, PENDING) {
        if let Some(taken) = take_entry(work_dir, id.as_str(), queued, kinds, now, stamp)? {
            return Ok(Some(taken));
        }
    }
    return Ok(None);
}

/// What:
///  Claims the pending entry `id` of one of `kinds`,
/// or answers nothing when it is no entry to claim.
/// Why:
///   The two ways of claiming,
/// one item at a time and a whole round,
/// differ only in the listing they walk,
/// so one function
///  takes an entry and each caller drives it through its own listing.
/// `queued` is the entry's modification time,
///  which a claim that cannot be taken puts back.
///
/// # Errors
///
/// A generation that cannot be made,
/// or a claim that cannot be read.
fn take_entry(work_dir: &Path, id: &str, queued: SystemTime, kinds: &[WorkKind], now: &ClaimTime, stamp: Stamp<'_>) -> Result<Option<Claimed>, String> {
    let mut found_kind: Option<&WorkKind> = None;
    for candidate in kinds {
        if id.starts_with(format!("{}-", candidate.name()).as_str()) {
            found_kind = Some(candidate);
            break;
        }
    }
    let Some(kind) = found_kind else {
        return Ok(None);
    };
    // An id that is not a kind and what that kind names could make names that do not fit; it never becomes a claim.
    if !crate::work_item_ids_and_names::is_id_suffix(kind, &id[kind.name().len() + 1..]) {
        quarantine_pending(
            work_dir,
            id,
            format!(
                "is not a kind and a suffix of at most {} characters: an object name, and for a push a branch digest after it",
                crate::work_item_ids_and_names::MAX_ID_SUFFIX_LENGTH
            )
            .as_str(),
        );
        return Ok(None);
    }
    let name: String = claim_file_name(id, new_generation()?.as_str(), now);
    let pending_path: PathBuf = state_directory(work_dir, PENDING).join(id);
    let staged: PathBuf = state_directory(work_dir, RECOVERING).join(name.as_str());
    let claimed_path: PathBuf = state_directory(work_dir, CLAIMED).join(name.as_str());
    if create_private_directory_all(state_directory(work_dir, RECOVERING).as_path()).is_err() || std::fs::rename(pending_path.as_path(), staged.as_path()).is_err() {
        return Ok(None);
    }
    let stamped: std::io::Result<()> = stamp(staged.as_path());
    let published: std::io::Result<()> = match stamped {
        Ok(()) => std::fs::rename(staged.as_path(), claimed_path.as_path()),
        Err(error) => Err(error),
    };
    if let Err(error) = published {
        log(format!("the claim of {id} could not be taken, so the item stays queued: {error}").as_str());
        // The staged file is this instance's alone:
        // the age it had is put back before it is queued again,
        // whatever the stamp did.
        let opened: std::io::Result<std::fs::File> = std::fs::File::open(staged.as_path());
        let restored: std::io::Result<()> = match opened {
            Ok(file) => file.set_modified(queued),
            Err(open_error) => Err(open_error),
        };
        let requeued: std::io::Result<()> = match restored {
            Ok(()) => std::fs::rename(staged.as_path(), pending_path.as_path()),
            Err(restore_error) => Err(restore_error),
        };
        if let Err(requeue_error) = requeued {
            log(format!("the staged claim of {id} could not be queued again; its lease will run out and recover it: {requeue_error}").as_str());
        }
        return Ok(None);
    }
    let text: String = match std::fs::read_to_string(claimed_path.as_path()) {
        Ok(read) => read,
        Err(error) => return Err(format!("reading the claim of {id} failed: {error}")),
    };
    return match WorkItem::decode(text.as_str()) {
        // The record is the work the file name says;
        // a record of other work would be settled as done under this id.
        Ok(item) if item.id() == id || item.id_before_the_branch() == id => Ok(Some(Claimed { id: id.to_owned(), name, item })),
        _ => {
            let _ = std::fs::rename(claimed_path.as_path(), state_directory(work_dir, FAILED).join(id));
            Ok(None)
        }
    };
}

/// Moves the entry `id` of `pending` to `failed` and says why,
/// where nothing can be done about a failure of either.
fn quarantine_pending(work_dir: &Path, id: &str, reason: &str) {
    let failed: PathBuf = state_directory(work_dir, FAILED);
    let moved: bool = create_private_directory_all(failed.as_path()).is_ok() && std::fs::rename(state_directory(work_dir, PENDING).join(id), failed.join(id)).is_ok();
    log(format!("the queued entry {id:?} {reason}, so it was {}", if moved { "moved to failed" } else { "left in pending because moving it failed" }).as_str());
}

/// What:
///  Move a claimed item to `done` (or to `failed`),
/// after the consumer's result is durable.
/// Why:
///   A crash before this rename leaves the item claimed,
/// and its lease runs out;
/// the work is then done again,
/// so a
///   handler must tolerate a repeat.
/// The rename names the claim's generation,
/// so a holder whose claim was taken over and
///  claimed again by another settles nothing of the other's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'claimed', claimed.name), join(workDir, 'done', claimed.id))
/// ```
pub fn settle(work_dir: &Path, claimed: &Claimed, state: &str) -> Result<(), String> {
    return settle_named(work_dir, claimed.name.as_str(), state);
}

/// `settle` for the claim file `name`,
/// whose id is the part of the name before the generation.
pub fn settle_named(work_dir: &Path, name: &str, state: &str) -> Result<(), String> {
    let id: &str = id_of_entry(name);
    return match std::fs::rename(state_directory(work_dir, CLAIMED).join(name), state_directory(work_dir, state).join(id)) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("moving {id} to {state} failed: {error}")),
    };
}

/// What:
///  Start the lease of the claim file `name` again at `now`,
/// returning its new name;
/// nothing when the claim is no longer in
///  `claimed` under that name or its modification time could not be set.
/// Why:
///   A holder that is still working must not look abandoned:
/// every consumer of a directory that several consumers drain reads
///   the reading in the name as the proof of life.
/// The renewal is a rename of the exact name the holder last wrote,
/// so a
///  claim that was taken over,
/// settled,
/// or taken over and claimed again by someone else is not found and is never brought
///  back or renewed on the other's behalf;
/// and an instance that looked at the old name and means to take it finds nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'claimed', name), join(workDir, 'claimed', claimFileName(id, token, now)))
/// ```
pub fn renew_claim(work_dir: &Path, name: &str, now: &ClaimTime) -> Option<String> {
    return renew_claim_stamped(work_dir, name, now, &stamp_modification_time);
}

/// `renew_claim` with the stamp of the claim's modification time given,
/// so a test can make the stamp fail.
/// A renewal whose
///  stamp failed is no renewal:
/// an older reader would still see the old modification time and take the claim.
fn renew_claim_stamped(work_dir: &Path, name: &str, now: &ClaimTime, stamp: Stamp<'_>) -> Option<String> {
    let id: &str = id_of_entry(name);
    // A claim by an earlier release has no generation; its renewal gives it one.
    let token: String = match parse_claim_name(name) {
        Some((token, _)) => token.to_owned(),
        None => new_generation().ok()?,
    };
    let renewed: String = claim_file_name(id, token.as_str(), now);
    let directory: PathBuf = state_directory(work_dir, CLAIMED);
    if let Err(error) = stamp(directory.join(name).as_path()) {
        log(format!("the claim {name} could not be renewed: {error}").as_str());
        return None;
    }
    return match std::fs::rename(directory.join(name), directory.join(renewed.as_str())) {
        Ok(()) => Some(renewed),
        Err(_refused) => None,
    };
}

/// What:
///  How long the oldest pending item has waited,
/// or `None` when nothing is pending.
/// Why:
///   The wrapper's one-line notice that no consumer is draining the queue is this age past a bound;
/// a process id in a
///   file would be meaningless across process-id namespaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// now - oldestPending.mtime
/// ```
pub fn oldest_pending_age(work_dir: &Path, now: SystemTime) -> Option<Duration> {
    let pending: Vec<(String, SystemTime)> = entries_oldest_first(work_dir, PENDING);
    let (_name, created) = pending.first()?;
    return Some(now.duration_since(*created).unwrap_or(Duration::ZERO));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "work_item_tests.rs"]
mod tests;
