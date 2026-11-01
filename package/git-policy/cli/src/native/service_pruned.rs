//! What:
//!  How a commit headed for the service notices that the content it prepared was removed from the repository before it
//!  landed: the policy read that fails on a missing object, the check of the request's objects after the service could not
//!  land it, and the numbered phase markers a test uses to remove content at a chosen instant.
//! Why:
//!  A commit's blobs, trees and commit object are unreferenced until the branch moves, and `git gc --prune=now` deletes every
//!  unreferenced object at once. The Git manual for `git gc` (the installed 2.56.0 manual and
//!  <https://git-scm.com/docs/git-gc>, section NOTES, checked 2026-11-01) says: "when git gc runs concurrently with another
//!  process, there is a risk of it deleting an object that the other process is using but hasn't created a reference to.
//!  This may just cause the other process to fail or may corrupt the repository if the other process later adds a
//!  reference to the deleted object", and for the option itself: "--prune=now prunes loose objects regardless of their age
//!  and increases the risk of corruption if another process is writing to the repository concurrently". Git's own
//!  mitigation is the two-week grace of the default `--prune` date, which `--prune=now` removes. This commit path therefore
//!  says plainly what happened and what to do (`ClientFailure::ContentPruned`), instead of showing a policy event for a
//!  machine or a failure of the service. It does not prepare the commit again: the bytes the command captured are what is
//!  gone, and reading the worktree again would commit whatever it holds by then.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (contentWasRemoved(events) || preparedContentMissing(request)) fail(CONTENT_PRUNED);
//! ```

use super::candidate_reader::OBJECT_MISSING_PREFIX;
use super::diagnostics::EngineFailureCode;
use super::policy_events::PolicyEvent;
use super::repo_read_layout::ObjectFormat;
use super::repo_read_object::from_hex;
use super::repo_read_odb::ObjectDatabase;
use super::service_protocol::CommitRequest;
use super::transaction_phase::{Phase, PhaseMarkerError, reach_transaction_phase};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counts how often this process reached `ServiceStaged`, which numbers the marker files of a commit prepared again.
static STAGED_REACHED: AtomicU64 = AtomicU64::new(0);

/// Counts how often this process reached `ServicePrepared`.
static PREPARED_REACHED: AtomicU64 = AtomicU64::new(0);

/// What:
///  Whether a policy run stopped because an object it had to read is missing from the repository.
/// Why:
///   That is the instant a pruning `git gc` is seen from the policies; preparing the commit again writes the content anew.
///   Other content failures (a Git that did not start, an object of the wrong kind) are not this and keep their event.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// events.some((event) => event.type === 'engine-failure' && event.message.startsWith(OBJECT_MISSING_PREFIX))
/// ```
pub fn content_was_removed(events: &[PolicyEvent]) -> bool {
    return events.iter().any(|event| {
        return matches!(
            event,
            PolicyEvent::EngineFailure { code: EngineFailureCode::ContentUnavailable, message, .. } if message.starts_with(OBJECT_MISSING_PREFIX)
        );
    });
}

/// What:
///  Whether an object the request depends on (a blob it names, the tree of an amend, the commit Git built) is gone from the
///  object store the client wrote it to.
/// Why:
///   When the service cannot land a request it answers with a general failure; whether the cause is removed content is
///  something only the objects can say, and reading them here starts no Git. A store the in-process reader cannot open
///  answers `false`, which leaves the general failure as it is.
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
        .filter_map(|change| { return change.blob.as_deref(); })
        .chain(request.tree.as_deref())
        .chain(request.commit.as_deref());
    for name in named {
        match from_hex(name.as_bytes(), format) {
            Some(bytes) if !database.contains(bytes.as_slice()) => return true,
            Some(_) | None => {}
        }
    }
    return false;
}

/// What:
///  Whether any object the requests of a batch depend on is gone from the repository's object store.
/// Why:
///   The landing asks this before it imports anything or moves the branch: `fast-import` dies on a missing blob and leaves a
///  crash report in the Git directory, and Git moves a branch to a commit whose objects are gone without looking. Loose
///  objects are found with one lookup each; the packs are opened only when an object is not loose. A store the in-process
///  reader cannot open answers `false`, which leaves the decision to Git, as for the client's own check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// requests.flatMap(namesOf).some((name) => !loose(name) && !(await packs()).has(name))
/// ```
pub fn batch_content_missing(common_dir: &Path, requests: &[&CommitRequest]) -> bool {
    let names: Vec<&str> = requests
        .iter()
        .flat_map(|request| {
            return request.changes.iter().filter_map(|change| { return change.blob.as_deref(); }).chain(request.tree.as_deref()).chain(request.commit.as_deref());
        })
        .collect();
    let objects: std::path::PathBuf = common_dir.join("objects");
    let is_loose = |name: &str| -> bool { return name.len() > 2 && objects.join(&name[..2]).join(&name[2..]).is_file(); };
    let absent: Vec<&str> = names.into_iter().filter(|name| { return !is_loose(name); }).collect();
    let Some(sample) = absent.first() else { return false };
    let format: ObjectFormat = if sample.len() == ObjectFormat::Sha256.hex_length() { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let Some(database) = ObjectDatabase::open(common_dir, None, format) else { return false };
    return absent.iter().any(|name| { return from_hex(name.as_bytes(), format).is_some_and(|bytes| { return !database.contains(bytes.as_slice()); }); });
}

/// What:
///  Reaches a service-commit phase marker, numbering the files by how often this process reached the phase.
/// Why:
///   A commit prepared again reaches each phase again, and a test that removes content at the phase must be able to answer
///  each time separately (`<phase>-<n>.reached`, `<phase>-<n>.release`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachServicePhase('service-staged')
/// ```
pub fn reach_service_phase(phase: Phase) -> Result<(), PhaseMarkerError> {
    let counter: &AtomicU64 = if phase == Phase::ServiceStaged { &STAGED_REACHED } else { &PREPARED_REACHED };
    let number: u64 = counter.fetch_add(1, Ordering::SeqCst) + 1;
    return reach_transaction_phase(phase, Some(number));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_pruned_tests.rs"]
mod tests;
