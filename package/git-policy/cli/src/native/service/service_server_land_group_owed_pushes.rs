//! What:
//!  What a batch of landings owes,
//!  and takes before its branch moves:
//!  the record of its index update and the held push of every commit it would land.
//! Why:
//!  The service knows a commit landed before the wrapper does,
//!  so the push record is held here where nothing runs it,
//!  and a batch that does not land gives back exactly what it took.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await holdThePush(request, commit);
//! ```

use crate::service_protocol::CommitRequest;

/// What:
///  Takes everything a batch owes before its branch moves:
/// the record of the index update it will owe,
/// and the held push of every commit it would land.
/// Why:
///   One call for the landing,
/// so a kill after the branch move leaves both obligations and neither is forgotten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whatABatchOwes(intent) // owed-index record, held pushes
/// ```
pub(crate) fn what_a_batch_owes(intent: &crate::service_journal::Intent) {
    crate::owed_index_update::write(&crate::owed_index_update::of_intent(intent));
    hold_the_pushes_of(intent);
}

/// What:
///  Gives back everything a batch that does not land owes.
/// Why:
///   One call for every path that removes the intent,
/// so a record or a held push of a batch that never landed survives nowhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetWhatABatchOwes(intent)
/// ```
pub(crate) fn forget_what_a_batch_owes(intent: &crate::service_journal::Intent) {
    crate::owed_index_update::remove(intent.git_dir.as_path(), intent.seq);
    forget_the_pushes_of(intent);
}

pub(crate) fn hold_the_pushes_of(intent: &crate::service_journal::Intent) {
    for (at, commit) in intent.commits.iter().enumerate() {
        // The command that will decide this commit's checks is the one the request named, and the batch recorded it: a restart's
        //  hold says who will answer, so a later command can prove that one gone instead of saying nothing will ever answer.
        let holder: &str = intent.holders.get(at).map_or("", String::as_str);
        hold_one(intent.repository.as_path(), intent.git_dir.as_path(), intent.reference.as_str(), commit.as_str(), intent.old.as_str(), holder);
    }
}

/// The push record of one landed commit of a worktree,
/// named by the item's own id:
/// the kind,
/// the commit and the digest of the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const item = { kind: 'push', repository, gitDir, branchRef: reference, oid: commit, parent, created: nowIso(), tries: 0 };
/// ```
fn push_item_of(repository: &std::path::Path, git_dir: &std::path::Path, reference: &str, commit: &str, parent: &str) -> crate::work_item::WorkItem {
    return crate::work_item::WorkItem {
        kind: crate::work_item::WorkKind::Push,
        repository: repository.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: reference.to_string(),
        oid: commit.to_string(),
        parent: parent.to_string(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
}

/// Forgets the pushes of a batch that does not land,
/// under the name this release writes and under the name a release before it wrote.
pub(crate) fn forget_the_pushes_of(intent: &crate::service_journal::Intent) {
    let work_dir: std::path::PathBuf = crate::service_work::work_directory(intent.git_dir.as_path());
    for commit in &intent.commits {
        let item: crate::work_item::WorkItem = push_item_of(intent.repository.as_path(), intent.git_dir.as_path(), intent.reference.as_str(), commit.as_str(), intent.old.as_str());
        for name in [item.id(), item.id_before_the_branch()] {
            crate::work_item::forget_held(work_dir.as_path(), name.as_str());
        }
    }
}

/// Holds one push record for a commit a batch lands,
/// naming the command that will decide the commit's checks.
///
/// The holder is the request's own,
/// recorded with the batch:
/// the answer loop writes it into the same record with the item's own words once the branch has moved (`hold_the_push`),
/// and a batch a restart finishes names the command the request named,
/// because no process table of the restart can tell which command owes the gate.
fn hold_one(repository: &std::path::Path, git_dir: &std::path::Path, reference: &str, commit: &str, parent: &str, holder: &str) {
    let item: crate::work_item::WorkItem = push_item_of(repository, git_dir, reference, commit, parent);
    let work_dir: std::path::PathBuf = crate::service_work::work_directory(git_dir);
    if let Err(reason) = crate::work_item::hold(work_dir.as_path(), &item, holder) {
        crate::service_error!("the push of the landed commit {commit} could not be held for its gate ({reason})");
    }
}

/// What:
///  Holds the push record of a commit this service landed,
/// where nothing runs it until the wrapper's post-commit gate approves it.
/// Why:
///   The wrapper writes its own record after the answer,
/// so a wrapper killed between the two would lose the push
///  (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
/// finding 3 of the first review).
/// The service knows the commit landed,
/// and a held record runs nothing:
/// only the wrapper's promotion lets the queue see
///  it,
/// so a blocking gate still keeps the commit local.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await holdThePush(request, commit);
/// ```
pub(crate) fn hold_the_push(request: &CommitRequest, commit: &str) {
    let item: crate::work_item::WorkItem = crate::work_item::WorkItem {
        kind: crate::work_item::WorkKind::Push,
        repository: request.repository.clone(),
        git_dir: request.git_dir.clone(),
        branch_ref: request.reference.clone(),
        oid: commit.to_string(),
        parent: request.base.clone(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    let work_dir: std::path::PathBuf = crate::service_work::work_directory(request.git_dir.as_path());
    // The record this batch already held names no command; naming this request's command here is what lets a later command
    //  tell a wait that has a command behind it from one whose command is gone (`held_push_holder_and_verdict.rs`).
    let named: bool = crate::work_item::record_verdict(work_dir.as_path(), item.id().as_str(), request.holder.as_str(), crate::held_push_holder_and_verdict::Verdict::Undecided, "");
    if named {
        return;
    }
    if let Err(reason) = crate::work_item::hold(work_dir.as_path(), &item, request.holder.as_str()) {
        crate::service_error!("the push of the landed commit {commit} could not be held for its gate ({reason})");
    }
}
