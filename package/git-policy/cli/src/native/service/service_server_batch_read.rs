//! What:
//!  One arrived batch of requests,
//! read:
//!  the withdrawals their clients sent answered before anything lands,
//!  the identities of withdrawn requests remembered so they can never land,
//!  and the commits grouped by branch in the order of their first arrival.
//! Why:
//!   Everything the loop does with a batch before it lands one of its groups is reading it:
//!   a request whose client withdrew must never land,
//!   not even when it reaches the queue after the withdrawal,
//!   a request that asks the service to retire ends the loop after this batch,
//!   and what may land together is one branch,
//!   with an amend a group of its own.
//!  Reading the batch in the order the queue had is what makes the order of the landings the order of arrival,
//!  and the count of commits is what tells a service retired before it served anything that it must stop its own
//!  socket (`service_own_socket_of_a_retire_that_served_nothing.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! answerTheWithdrawals(jobs, landed, withdrawn); const batch = groupTheBatch(jobs, withdrawn);
//! ```
use crate::service_journal::DONE_RETAINED;
use crate::service_protocol::ServiceMessage;
use crate::service_server_queue_types_and_send::{
    GroupKey,
    Job,
    Queued,
    place_in_groups,
};
use std::collections::{
    HashMap,
    HashSet,
};

/// How many withdrawn identities the service remembers before it forgets them all:
/// the same count as the finished batches it
///  remembers (`DONE_RETAINED`),
/// since a client that withdrew sends nothing more for its request.
const WITHDRAWN_RETAINED: usize = DONE_RETAINED;

/// What:
///  Answers every withdrawal of one arrived batch,
///  and remembers the identity of each request it drops.
/// Why:
///   The answer is the landing's own verdict when the commit is already on the branch,
///   and nothing else can be said for a request that has not landed yet,
///   so the client is told it was withdrawn and the identity is kept against a later arrival of that request.
///  The memory never grows past `WITHDRAWN_RETAINED`:
///  it is emptied before more identities are added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// answerTheWithdrawals(jobs, landed, withdrawn)
/// ```
pub(super) fn answer_the_withdrawals(
    jobs: &[Job],
    landed: &HashMap<String, String>,
    withdrawn: &mut HashSet<String>,
) {
    // Withdrawals are answered before anything lands, on this one thread, so each is either after a landing that is over (the
    //  commit is on the branch, and that is the answer) or before the landing of its request begins (the request is dropped
    //  and remembered, so it can never land, not even when it reaches the queue after the withdrawal).
    if withdrawn.len() > WITHDRAWN_RETAINED {
        withdrawn.clear();
    }
    for job in jobs {
        if let Job::Withdraw(id, reply) = job {
            match landed.get(id) {
                Some(commit) => reply.send(&ServiceMessage::Landed {
                    id: id.clone(),
                    commit: commit.clone(),
                    already: true,
                    landing: None,
                }),
                None => {
                    withdrawn.insert(id.clone());
                    reply.send(&ServiceMessage::Withdrawn {
                        id: id.clone(),
                    });
                }
            }
        }
    }
}

/// What one arrived batch is once its jobs are read.
/// Why:
///  The loop needs the groups,
/// whether the service was asked to retire,
/// and how many commits it was asked to land.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadBatch = { groups: [GroupKey, Queued[]][]; retiring: boolean; commits: number };
/// ```
pub(super) struct ReadBatch {
    /// The groups to land,
    /// each with the requests that land as it does.
    pub groups: Vec<(GroupKey, Vec<Queued>)>,
    /// Whether one request of the batch asked the service to retire.
    pub retiring: bool,
    /// How many commit requests the batch held.
    pub commits: usize,
}

/// What:
///  Reads one arrived batch into the groups the service lands,
///  dropping the requests their clients withdrew.
/// Why:
///   An amend is a group of its own,
///   and what arrives after it starts another group,
///   so the order on one branch is the order of arrival.
///  `HEAD` is a head of one worktree,
///  so two detached worktrees of one repository are two branches.
///  The key is exact for a name that is not text too,
///  so two branches never share a group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// groupTheBatch(jobs, withdrawn) // { groups, retiring, commits }
/// ```
pub(super) fn group_the_batch(
    jobs: Vec<Job>,
    withdrawn: &HashSet<String>,
) -> ReadBatch {
    let mut retiring: bool = false;
    let mut commits: usize = 0;
    // Branches in order of first arrival; within a branch, arrival order.
    let mut groups: Vec<(GroupKey, Vec<Queued>)> = Vec::new();
    for job in jobs {
        match job {
            Job::Withdraw(..) => {}
            Job::Commit(request, _dropped, _arrival) if withdrawn.contains(&request.id) => {
                // The client took its request back, so nothing lands on the tree it named and its tree goes back
                //  to the age rule with it (`service_scratch_trees_of_queued_requests.rs`).
                crate::service_scratch_trees_of_queued_requests::release(request.id.as_str());
                crate::service_log!(
                    "dropped the request {} that its client withdrew",
                    request.id
                );
            }
            Job::Retire => retiring = true,
            Job::Activity => {}
            Job::Commit(request, stream, arrival) => {
                commits += 1;
                // An amend is a group of its own, and what arrives after it starts another group, so the order on
                //  one branch is the order of arrival.
                // `HEAD` is a head of one worktree, so two detached worktrees of one repository are two branches.
                // The key is exact for a name that is not text too, so two branches never share a group.
                let queue_reference: String = if request.reference.is_head() {
                    format!("HEAD@{}", request.repository.display())
                } else {
                    request.reference.hex()
                };
                let key: GroupKey = (
                    request.common_dir.clone(),
                    queue_reference,
                    request.lands_alone(),
                );
                place_in_groups(&mut groups, key, (request, stream, arrival));
            }
        }
    }
    // What: the three answers the loop reads, each already in the order the queue had.
    // Why:  the loop lands the groups, ends when it was told to retire, and counts what it served.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { groups, retiring, commits };
    // ```
    return ReadBatch {
        groups,
        retiring,
        commits,
    };
}
