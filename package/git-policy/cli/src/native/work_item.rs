//! What:
//!  The durable work-item record: a small file in the Git directory that says "this landed object needs this
//!  asynchronous work", written by the wrapper and drained by a separate resident consumer.
//! Why:
//!  A wrapper process exits when its command ends, and it may start no program, so work that must not delay a landing
//!  (the standards review, later auto-push and maintenance) is left as a record instead. The record is generic: a kind,
//!  the landed object and an id derived from both, so a retry cannot duplicate it. The design, its failure cases and
//!  the consumer contract are in `doc/planning/cli-git-standards-review-port.md`. Nothing calls this yet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await enqueue(workDir, { kind: 'review', oid, ... }); const item = await claimNext(workDir); await finish(workDir, item);
//! ```

use super::private_storage::{create_private_directory_all, write_private_file_unsynced};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The record format version.
pub const VERSION: u64 = 1;

/// Directory under the common Git directory that holds the state directories.
pub const WORK_DIRECTORY: &str = "cli-git/work";

/// The state directories, in the order an item moves through them.
pub const PENDING: &str = "pending";
/// Held by one consumer, with a lease.
pub const CLAIMED: &str = "claimed";
/// Finished.
pub const DONE: &str = "done";
/// Given up after too many tries.
pub const FAILED: &str = "failed";

/// What:
///  The kind of work an item asks for.
/// Why:
///   One record format carries every asynchronous job; the kind picks the consumer's handler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorkKind = 'review' | 'push' | 'maintenance';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkKind {
    /// The standards-review shadow run.
    Review,
    /// An auto-push of the branch.
    Push,
    /// Repository maintenance.
    Maintenance,
}

/// Methods over a kind.
impl WorkKind {
    /// The kind's name, which is also the start of an item's id.
    pub fn name(self) -> &'static str {
        return match self {
            WorkKind::Review => "review",
            WorkKind::Push => "push",
            WorkKind::Maintenance => "maintenance",
        };
    }

    /// The kind a name spells.
    pub fn from_name(name: &str) -> Option<WorkKind> {
        return [WorkKind::Review, WorkKind::Push, WorkKind::Maintenance].into_iter().find(|kind| return kind.name() == name);
    }
}

/// What:
///  One work item: what to do, to which landed object, and where.
/// Why:
///   The consumer needs only the object name and the places to find the repository; it reads the patch itself.
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
    pub repository: String,
    /// The common Git directory.
    pub git_dir: String,
    /// The branch reference that received the commit.
    pub branch_ref: String,
    /// The landed commit.
    pub oid: String,
    /// Its first parent, or empty for a root commit.
    pub parent: String,
    /// `toISOString` text of the enqueue time.
    pub created: String,
    /// Failed or abandoned attempts so far.
    pub tries: u32,
}

/// Methods over an item.
impl WorkItem {
    /// The id and file name: the kind and the landed object, so the same landing always yields the same id.
    pub fn id(&self) -> String {
        return format!("{}-{}", self.kind.name(), self.oid);
    }

    /// The record's one JSON line.
    pub fn encode(&self) -> String {
        let mut fields: Map<String, Value> = Map::new();
        fields.insert(String::from("v"), Value::from(VERSION));
        fields.insert(String::from("tries"), Value::from(self.tries));
        for (name, text) in [
            ("kind", self.kind.name()),
            ("id", self.id().as_str()),
            ("repository", self.repository.as_str()),
            ("gitDir", self.git_dir.as_str()),
            ("ref", self.branch_ref.as_str()),
            ("oid", self.oid.as_str()),
            ("parent", self.parent.as_str()),
            ("created", self.created.as_str()),
        ] {
            fields.insert(String::from(name), Value::from(text));
        }
        return format!("{}\n", Value::Object(fields));
    }

    /// Read a record; any other version, an unknown kind, a missing field or an id that is not derived from the kind and
    /// object is refused with a message.
    pub fn decode(text: &str) -> Result<WorkItem, String> {
        let value: Value = serde_json::from_str(text).map_err(|error| return format!("the record is not JSON: {error}"))?;
        let field = |name: &str| -> Result<String, String> {
            return value.get(name).and_then(Value::as_str).map(String::from).ok_or_else(|| return format!("the record has no text field {name}"));
        };
        if value.get("v").and_then(Value::as_u64) != Some(VERSION) {
            return Err(String::from("the record has an unsupported version"));
        }
        let kind: WorkKind = WorkKind::from_name(field("kind")?.as_str()).ok_or_else(|| return String::from("the record has an unknown kind"))?;
        let tries: u32 = value.get("tries").and_then(Value::as_u64).and_then(|count| return u32::try_from(count).ok()).ok_or_else(|| return String::from("the record has no tries"))?;
        let item: WorkItem = WorkItem {
            kind,
            repository: field("repository")?,
            git_dir: field("gitDir")?,
            branch_ref: field("ref")?,
            oid: field("oid")?,
            parent: field("parent")?,
            created: field("created")?,
            tries,
        };
        if field("id")? != item.id() {
            return Err(String::from("the record's id is not derived from its kind and object"));
        }
        return Ok(item);
    }
}

/// What:
///  Whether an enqueue added the item or found it already known.
/// Why:
///   A landing that is replayed, or a daemon that restarts, enqueues again; the second time must change nothing.
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

/// The directory of one state.
pub fn state_directory(work_dir: &Path, state: &str) -> PathBuf {
    return work_dir.join(state);
}

/// What:
///  Write the item as a pending record, whole or absent, unless its id is already in any state.
///  The producer never waits for a consumer and never reads anything back.
/// Why:
///   The id is the idempotency key; the exclusive publish makes two racing producers agree on one winner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeExclusive(join(workDir, 'pending', item.id), item.encode())
/// ```
pub fn enqueue(work_dir: &Path, item: &WorkItem) -> Result<Enqueued, String> {
    for state in [PENDING, CLAIMED, DONE, FAILED] {
        create_private_directory_all(state_directory(work_dir, state).as_path()).map_err(|error| return format!("creating the {state} directory failed: {error}"))?;
    }
    let id: String = item.id();
    for state in [CLAIMED, DONE, FAILED] {
        if state_directory(work_dir, state).join(id.as_str()).exists() {
            return Ok(Enqueued::Known);
        }
    }
    return match write_private_file_unsynced(state_directory(work_dir, PENDING).join(id.as_str()).as_path(), item.encode().as_bytes()) {
        Ok(()) => Ok(Enqueued::Added),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(Enqueued::Known),
        Err(error) => Err(format!("writing the work item {id} failed: {error}")),
    };
}

/// What:
///  A held item: the decoded record and its id.
/// Why:
///   Finishing needs only the id; the consumer needs the item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Claimed = { id: string; item: WorkItem };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claimed {
    /// File name in every state directory.
    pub id: String,
    /// The decoded record.
    pub item: WorkItem,
}

/// The names in a state directory with their modification times, oldest first.
fn entries_oldest_first(work_dir: &Path, state: &str) -> Vec<(String, SystemTime)> {
    let mut found: Vec<(String, SystemTime)> = Vec::new();
    if let Ok(listing) = std::fs::read_dir(state_directory(work_dir, state)) {
        for entry in listing.flatten() {
            let modified: SystemTime = entry.metadata().and_then(|meta| return meta.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
            found.push((entry.file_name().to_string_lossy().into_owned(), modified));
        }
    }
    found.sort_by(|left, right| return left.1.cmp(&right.1).then_with(|| return left.0.cmp(&right.0)));
    return found;
}

/// What:
///  Claim the oldest pending item by renaming it into `claimed`, stamping the claim time as its lease start.
///  A record that does not decode is moved to `failed` so it cannot block the queue.
/// Why:
///   A rename is atomic: of two consumers racing for one item, the loser sees the file gone and takes the next one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'pending', id), join(workDir, 'claimed', id))
/// ```
pub fn claim_next(work_dir: &Path, now: SystemTime) -> Result<Option<Claimed>, String> {
    return claim_next_of(work_dir, now, &[WorkKind::Review, WorkKind::Push, WorkKind::Maintenance]);
}

/// What:
///  Claim the oldest pending item of the given kinds, leaving every other kind where it is.
/// Why:
///   Consumers of different kinds share one queue; the kind starts the file name, so a consumer never opens, moves or
///   stamps an item that belongs to another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'pending', id), join(workDir, 'claimed', id)) // only ids that start with a handled kind
/// ```
pub fn claim_next_of(work_dir: &Path, now: SystemTime, kinds: &[WorkKind]) -> Result<Option<Claimed>, String> {
    for (id, _) in entries_oldest_first(work_dir, PENDING) {
        if !kinds.iter().any(|kind| return id.starts_with(format!("{}-", kind.name()).as_str())) {
            continue;
        }
        let claimed_path: PathBuf = state_directory(work_dir, CLAIMED).join(id.as_str());
        if std::fs::rename(state_directory(work_dir, PENDING).join(id.as_str()), claimed_path.as_path()).is_err() {
            continue;
        }
        let stamped: std::io::Result<()> = std::fs::File::options().append(true).open(claimed_path.as_path()).and_then(|file| return file.set_modified(now));
        stamped.map_err(|error| return format!("stamping the claim of {id} failed: {error}"))?;
        let text: String = std::fs::read_to_string(claimed_path.as_path()).map_err(|error| return format!("reading the claim of {id} failed: {error}"))?;
        match WorkItem::decode(text.as_str()) {
            Ok(item) => return Ok(Some(Claimed { id, item })),
            Err(_) => {
                let _ = std::fs::rename(claimed_path.as_path(), state_directory(work_dir, FAILED).join(id.as_str()));
            }
        }
    }
    return Ok(None);
}

/// What:
///  Move a claimed item to `done` (or to `failed`), after the consumer's result is durable.
/// Why:
///   A crash before this rename leaves the item claimed, and its lease runs out; the work is then done again, so a
///   handler must tolerate a repeat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'claimed', id), join(workDir, 'done', id))
/// ```
pub fn settle(work_dir: &Path, claimed: &Claimed, state: &str) -> Result<(), String> {
    return std::fs::rename(state_directory(work_dir, CLAIMED).join(claimed.id.as_str()), state_directory(work_dir, state).join(claimed.id.as_str()))
        .map_err(|error| return format!("moving {} to {state} failed: {error}", claimed.id));
}

/// What:
///  Return claims whose lease ran out to `pending` with one more try, or to `failed` at `max_tries`.
///  Returns how many were moved.
/// Why:
///   This is how a consumer killed at any instruction is noticed when nothing says its successor started: by its lease,
///   never by a process id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const stale of claimed.filter((c) => now - c.mtime > lease)) requeue(stale)
/// ```
pub fn requeue_expired(work_dir: &Path, now: SystemTime, lease: Duration, max_tries: u32) -> usize {
    return requeue_where(work_dir, max_tries, &|started| { return now.duration_since(started).unwrap_or(Duration::ZERO) > lease; });
}

/// What:
///  Return every claim to `pending` with one more try, or to `failed` at `max_tries`, whatever its age. Returns how many
///  were moved.
/// Why:
///   A consumer that starts holds no claim yet, and the manager runs one consumer at a time, so a claim it finds was left by a
///   consumer that died or was replaced; waiting out the lease would strand the work for as long as the lease lasts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const left of claimed) requeue(left)
/// ```
pub fn requeue_claimed(work_dir: &Path, max_tries: u32) -> usize {
    return requeue_where(work_dir, max_tries, &|_started| { return true; });
}

/// Returns the claims whose stamp satisfies `due` to `pending`, or to `failed` at `max_tries`; how many were moved.
fn requeue_where(work_dir: &Path, max_tries: u32, due: &dyn Fn(SystemTime) -> bool) -> usize {
    let mut moved: usize = 0;
    for (id, started) in entries_oldest_first(work_dir, CLAIMED) {
        if !due(started) {
            continue;
        }
        let from: PathBuf = state_directory(work_dir, CLAIMED).join(id.as_str());
        let Ok(text) = std::fs::read_to_string(from.as_path()) else {
            continue;
        };
        let target: &str = match WorkItem::decode(text.as_str()) {
            Ok(mut item) => {
                item.tries += 1;
                let rewritten: PathBuf = from.with_extension("retry");
                let state: &str = if item.tries >= max_tries { FAILED } else { PENDING };
                if std::fs::write(rewritten.as_path(), item.encode()).is_err() || std::fs::rename(rewritten.as_path(), from.as_path()).is_err() {
                    continue;
                }
                state
            }
            Err(_) => FAILED,
        };
        if std::fs::rename(from.as_path(), state_directory(work_dir, target).join(id.as_str())).is_ok() {
            moved += 1;
        }
    }
    return moved;
}

/// What:
///  How long the oldest pending item has waited, or `None` when nothing is pending.
/// Why:
///   The wrapper's one-line notice that no consumer is draining the queue is this age past a bound; a process id in a
///   file would be meaningless across process-id namespaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// now - oldestPending.mtime
/// ```
pub fn oldest_pending_age(work_dir: &Path, now: SystemTime) -> Option<Duration> {
    return entries_oldest_first(work_dir, PENDING).first().map(|(_, created)| return now.duration_since(*created).unwrap_or(Duration::ZERO));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "work_item_tests.rs"]
mod tests;
