//! What:
//!  Writing one producer's record,
//!  or a round of them,
//!  to the queue,
//!  and finding what is already there.
//! Why:
//!   The producer never waits for a consumer and never reads the queue back,
//!  so the id is the idempotency key and the exclusive publish makes two racing producers agree on one winner.
//!   A replayed landing and a restarted service enqueue again,
//!  and the second time must change nothing.
//! The work record itself is described beside it in `work_item.rs`,
//!  which re-exports these so a caller names one module.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await enqueueAll(workDir, items); // 'added' | 'known' for each, in the order given
//! ```

use crate::private_storage::{
    create_private_directory_all,
    write_private_file_unsynced,
};
use crate::work_item_record::WorkItem;
use crate::work_item_state_directories_and_claim_names::{
    CLAIMED,
    DONE,
    FAILED,
    PENDING,
    RECOVERING,
    entries_oldest_first,
    id_of_entry,
    state_directory,
};
use std::collections::HashSet;
use std::path::Path;

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
pub fn enqueue(
    work_dir: &Path,
    item: &WorkItem,
) -> Result<Enqueued, String> {
    let answers: Vec<Enqueued> = enqueue_all(work_dir, std::slice::from_ref(item))?;
    let Some(answer) = answers.first() else {
        return Err(format!(
            "the round of the one item {} answered nothing",
            item.id()
        ));
    };
    return Ok(*answer);
}

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
pub fn enqueue_all(
    work_dir: &Path,
    items: &[WorkItem],
) -> Result<Vec<Enqueued>, String> {
    for state in [PENDING, CLAIMED, DONE, FAILED] {
        if let Err(error) = create_private_directory_all(state_directory(work_dir, state).as_path())
        {
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
        match write_private_file_unsynced(
            state_directory(work_dir, PENDING)
                .join(id.as_str())
                .as_path(),
            item.encode().as_bytes(),
        ) {
            Ok(()) => {
                held.insert(id);
                answers.push(Enqueued::Added);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                answers.push(Enqueued::Known)
            }
            Err(error) => return Err(format!("writing the work item {id} failed: {error}")),
        }
    }
    return Ok(answers);
}
