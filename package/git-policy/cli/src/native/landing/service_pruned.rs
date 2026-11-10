//! What:
//!  How a commit headed for the service notices that the content it prepared was removed from the repository before it
//!  landed:
//! the policy read that fails on a missing object,
//! the check of the request's objects after the service could not
//!  land it,
//! and the numbered phase markers a test uses to remove content at a chosen instant.
//! Why:
//!  A commit's blobs,
//! trees and commit object are unreferenced until the branch moves,
//! and `git gc --prune=now` deletes every
//!  unreferenced object at once.
//! The Git manual for `git gc` (the installed 2.56.0 manual and
//!  <https://git-scm.com/docs/git-gc>,
//! section NOTES,
//! checked 2026-11-01) says:
//! "when git gc runs concurrently with another
//!  process,
//! there is a risk of it deleting an object that the other process is using but hasn't created a reference to.
//!  This may just cause the other process to fail or may corrupt the repository if the other process later adds a
//!  reference to the deleted object",
//! and for the option itself:
//! "--prune=now prunes loose objects regardless of their age
//!  and increases the risk of corruption if another process is writing to the repository concurrently".
//! Git's own
//!  mitigation is the two-week grace of the default `--prune` date,
//! which `--prune=now` removes.
//! This commit path therefore
//!  says plainly what happened and what to do (`ClientFailure::ContentPruned`),
//! instead of showing a policy event for a
//!  machine or a failure of the service.
//! It does not prepare the commit again:
//! the bytes the command captured are what is
//!  gone,
//! and reading the worktree again would commit whatever it holds by then.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (contentWasRemoved(events) || preparedContentMissing(request)) fail(CONTENT_PRUNED);
//! ```

use crate::candidate_reader::OBJECT_MISSING_PREFIX;
use crate::diagnostic_log::warn;
use crate::diagnostics::EngineFailureCode;
use crate::policy_events::PolicyEvent;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{ObjectKind, from_hex, name_length};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{MODE_GITLINK, is_tree_mode, parse_tree};
use crate::service_protocol::{Change, CommitRequest};
use crate::transaction_phase::{Phase, PhaseMarkerError, reach_transaction_phase};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// The layout of the captured invocation a request was prepared from.
use crate::capture_invocation::InvocationLayout;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli-service";

/// Counts how often this process reached `ServiceStaged`,
/// which numbers the marker files of a commit prepared again.
static STAGED_REACHED: AtomicU64 = AtomicU64::new(0);

/// Counts how often this process reached `ServicePrepared`.
static PREPARED_REACHED: AtomicU64 = AtomicU64::new(0);

/// Counts how often this process reached `ServiceSentBack`.
static SENT_BACK_REACHED: AtomicU64 = AtomicU64::new(0);

/// What:
///  Whether an event says the content to check was removed:
/// an engine failure that the content is unavailable,
/// naming a missing object.
/// Why:
///  `content_was_removed` asks it of every event of a check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRemovedContent = (event: PolicyEvent) => event.code === 'content-unavailable' && event.message.startsWith(OBJECT_MISSING_PREFIX);
/// ```
fn is_removed_content(event: &PolicyEvent) -> bool {
    return matches!(
        event,
        PolicyEvent::EngineFailure { code: EngineFailureCode::ContentUnavailable, message, .. } if message.starts_with(OBJECT_MISSING_PREFIX)
    );
}

/// What:
///  Whether a policy run stopped because an object it had to read is missing from the repository.
/// Why:
///   That is the instant a pruning `git gc` is seen from the policies;
/// preparing the commit again writes the content anew.
///   Other content failures (a Git that did not start,
/// an object of the wrong kind) are not this and keep their event.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// events.some((event) => event.type === 'engine-failure' && event.message.startsWith(OBJECT_MISSING_PREFIX))
/// ```
pub fn content_was_removed(events: &[PolicyEvent]) -> bool {
    return events.iter().any(is_removed_content);
}

/// How many tree entries (and trees) the walk of one batch examines before it stops asking and leaves the rest to Git.
pub const CLOSURE_ENTRY_BUDGET: usize = 5_000_000;

/// How the walk over the trees of a batch ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClosureEnd = 'intact' | 'missing' | 'out-of-budget';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClosureEnd {
    /// Every tree and blob below every root is present,
    /// or is left to Git as unreadable.
    Intact,
    /// An object below a root is gone from the object store.
    Missing,
    /// The budget ran out first,
    /// so nothing is known about the rest.
    OutOfBudget,
}

/// What a walk found and what it cost.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClosureWalk {
    /// How it ended.
    end: ClosureEnd,
    /// Entries and trees examined,
    /// which is what the budget counts.
    examined: usize,
}

/// What:
///  Whether a tree,
/// a tree below it or a blob in any of them is gone from the object store,
/// for every root of a batch.
/// Why:
///   An amendment carries a whole tree and no list of blobs,
/// and a prune can remove a blob from under a tree that survives
///  until the branch moves.
/// One work stack and one set of examined objects serve the whole batch,
/// so a tree that several
///  requests share (the usual amendment batch changes few files of one large tree) is walked once,
/// and a blob that several
///  trees list is looked up once:
/// the cost is linear in the batch's distinct objects,
/// with no recursion.
/// A found loss ends
///  the whole walk,
/// so a shared set never hides one.
/// The budget counts every tree and entry examined,
/// not only distinct
///  trees;
/// when it runs out the answer is "not known",
/// as for an object that cannot be read here,
/// because holding the
///  landing for an unbounded walk delays every later commit of the worktree.
/// A submodule entry names a commit of another
///  repository and is skipped,
/// and an object that is present but cannot be read here (unreadable,
/// beyond the reader's size
///  limit) is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// walkTrees(roots, { shared: true, budget }).some((entry) => !objectStore.has(entry.id))
/// ```
fn walk_tree_closures(database: &ObjectDatabase, roots: &[&str], format: ObjectFormat, budget: usize) -> ClosureWalk {
    // A root that is not an object name of the repository's format is left out.
    let mut pending: Vec<Vec<u8>> = Vec::new();
    for root in roots {
        if let Some(bytes) = from_hex(root.as_bytes(), format) {
            pending.push(bytes);
        }
    }
    let mut visited: HashSet<Vec<u8>> = HashSet::new();
    let mut examined: usize = 0;
    while let Some(name) = pending.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        if examined >= budget {
            return ClosureWalk { end: ClosureEnd::OutOfBudget, examined };
        }
        examined += 1;
        let Some(object) = database.read(name.as_slice()) else {
            if database.contains(name.as_slice()) {
                continue;
            }
            return ClosureWalk { end: ClosureEnd::Missing, examined };
        };
        if object.kind != ObjectKind::Tree {
            continue;
        }
        let Some(entries) = parse_tree(object.bytes.as_slice(), name_length(format)) else { continue };
        if entries.len() > budget - examined {
            return ClosureWalk { end: ClosureEnd::OutOfBudget, examined };
        }
        examined += entries.len();
        for entry in entries {
            if entry.mode == MODE_GITLINK {
                continue;
            }
            if is_tree_mode(entry.mode) {
                pending.push(entry.id);
            } else if visited.insert(entry.id.clone()) && !database.contains(entry.id.as_slice()) {
                return ClosureWalk { end: ClosureEnd::Missing, examined };
            }
        }
    }
    return ClosureWalk { end: ClosureEnd::Intact, examined };
}

/// Whether the walk found a loss;
/// an exhausted budget is logged and counts as "not known",
/// which is `false`.
fn tree_closures_missing(database: &ObjectDatabase, roots: &[&str], format: ObjectFormat) -> bool {
    let walk: ClosureWalk = walk_tree_closures(database, roots, format, CLOSURE_ENTRY_BUDGET);
    if walk.end == ClosureEnd::OutOfBudget {
        warn(TAG, &format!("the trees of the batch hold more than {CLOSURE_ENTRY_BUDGET} entries; their objects are not checked here and are left to Git"));
    }
    return walk.end == ClosureEnd::Missing;
}

/// What:
///  Whether an object the request depends on (a blob it names,
/// the tree of an amend with everything below it,
/// the commit
///  Git built) is gone from the object store the client wrote it to.
/// Why:
///   When the service cannot land a request it answers with a general failure;
/// whether the cause is removed content is
///  something only the objects can say,
/// and reading them here starts no Git.
/// A store the in-process reader cannot open
///  answers `false`,
/// which leaves the general failure as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesOf(request).some((name) => !objectStore.has(name))
/// ```
pub fn prepared_content_missing(common_dir: &Path, objects_dir: &Path, object_format: &str, request: &CommitRequest) -> bool {
    let format: ObjectFormat = if object_format == ObjectFormat::Sha256.name() { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let Some(database) = ObjectDatabase::open(common_dir, Some(objects_dir), format) else { return false };
    let named = request
        .changes
        .iter()
        .filter_map(blob_of_change)
        .chain(request.tree.as_deref())
        .chain(request.commit.as_deref());
    for name in named {
        match from_hex(name.as_bytes(), format) {
            Some(bytes) if !database.contains(bytes.as_slice()) => return true,
            Some(_) | None => {}
        }
    }
    return match request.tree.as_deref() {
        Some(tree) => tree_closures_missing(&database, &[tree], format),
        None => false,
    };
}

/// What:
///  Whether the objects the request names are gone from the repository the capture wrote them to.
/// Why:
///   The service answers a request it cannot land with a general failure,
/// and removed content is the one such answer that must say what happened instead of naming the service;
/// the three locations it reads are the capture's,
/// so a caller asks this question and never assembles them itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (theObjectsItNamesAreGone(request, layout)) fail(CONTENT_PRUNED);
/// ```
pub(crate) fn the_objects_it_names_are_gone(request: &CommitRequest, layout: &InvocationLayout) -> bool {
    return prepared_content_missing(
        layout.common_dir.as_path(),
        layout.object_directory.as_path(),
        layout.object_format.as_str(),
        request,
    );
}

/// What:
///  The blob a change names,
/// nothing for a deletion.
/// Why:
///  The objects a request depends on are its blobs,
/// its tree and its commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blobOfChange = (change: Change) => change.blob ?? undefined;
/// ```
fn blob_of_change(change: &Change) -> Option<&str> {
    return change.blob.as_deref();
}

/// What:
///  Whether the loose object file of a name is in the objects directory.
/// Why:
///  A loose object needs no reader of the object store;
/// only the others are looked up in the packs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLoose = (objects: string, name: string) => name.length > 2 && existsSync(join(objects, name.slice(0, 2), name.slice(2)))
/// ```
fn is_loose(objects: &std::path::Path, name: &str) -> bool {
    return name.len() > 2 && objects.join(&name[..2]).join(&name[2..]).is_file();
}

/// What:
///  Whether any object the requests of a batch depend on is gone from the repository's object store.
/// Why:
///   The landing asks this before it imports anything or moves the branch:
/// `fast-import` dies on a missing blob and leaves a
///  crash report in the Git directory,
/// and Git moves a branch to a commit whose objects are gone without looking.
/// Loose
///  objects are found with one lookup each;
/// the packs are opened only when an object is not loose.
/// A store the in-process
///  reader cannot open answers `false`,
/// which leaves the decision to Git,
/// as for the client's own check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// requests.flatMap(namesOf).some((name) => !loose(name) && !(await packs()).has(name))
/// ```
pub fn batch_content_missing(common_dir: &Path, requests: &[&CommitRequest]) -> bool {
    let mut names: Vec<&str> = Vec::new();
    for request in requests {
        names.extend(request.changes.iter().filter_map(blob_of_change).chain(request.tree.as_deref()).chain(request.commit.as_deref()));
    }
    let objects: std::path::PathBuf = common_dir.join("objects");
    let mut absent: Vec<&str> = Vec::new();
    for name in names {
        if !is_loose(objects.as_path(), name) {
            absent.push(name);
        }
    }
    // An embedded tree is walked whether or not its own object is loose, so a request with one always needs the reader.
    let mut trees: Vec<&str> = Vec::new();
    for request in requests {
        if let Some(tree) = request.tree.as_deref() {
            trees.push(tree);
        }
    }
    let Some(sample) = absent.first().or(trees.first()) else { return false };
    let format: ObjectFormat = if sample.len() == ObjectFormat::Sha256.hex_length() { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let Some(database) = ObjectDatabase::open(common_dir, None, format) else { return false };
    for name in &absent {
        if let Some(bytes) = from_hex(name.as_bytes(), format)
            && !database.contains(bytes.as_slice())
        {
            return true;
        }
    }
    return tree_closures_missing(&database, &trees, format);
}

/// What:
///  Reaches a service-commit phase marker,
/// numbering the files by how often this process reached the phase.
/// Why:
///   A commit prepared again reaches each phase again,
/// and a test that removes content at the phase must be able to answer
///  each time separately (`<phase>-<n>.reached`,
/// `<phase>-<n>.release`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachServicePhase('service-staged')
/// ```
pub fn reach_service_phase(phase: Phase) -> Result<(), PhaseMarkerError> {
    // One counter per phase, so a number always counts the arrivals at that phase and nothing else.
    let counter: &AtomicU64 = match phase {
        Phase::ServiceStaged => &STAGED_REACHED,
        Phase::ServiceSentBack => &SENT_BACK_REACHED,
        Phase::MergeCarried
        | Phase::MergeCleared
        | Phase::MergeIntent
        | Phase::MergeApplied
        | Phase::MergeCarriedBack
        | Phase::ServicePrepared => &PREPARED_REACHED,
    };
    let number: u64 = counter.fetch_add(1, Ordering::SeqCst) + 1;
    return reach_transaction_phase(phase, Some(number));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_pruned_tests.rs"]
mod tests;
