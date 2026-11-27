//! What:
//!  What the landing does with a commit whose own side reference it could not prove:
//!  it keeps the commit on the reference a commit that did not land is kept on,
//!  it lets go of the batch's own reference only once another reference names the commit,
//!  and it says to the person where the commit is.
//! Why:
//!   A commit the batch built and no reference names is one `git gc --prune=now` takes,
//!  so a batch whose own reference the write did not prove is answered as not made
//!  and the commit is kept where a commit that did not land is kept,
//!  named to the person
//!  (the owner's rule "never discard user data").
//!  A reference that is the only name of a commit is deleted only after another name for that commit is known to be
//!  written,
//!  and never when the name it deleted was not the value that was read
//!  (a compare-and-delete,
//!  `git update-ref -d <ref> <old>`).
//!  A keep Git refused leaves the batch's own scratch reference as the only name the commit has,
//!  so the person is told the commit's name and that reference:
//!  a sentence that named a recoverable commit without saying where it is would read as a commit that was lost.
//!  It sits in a file of its own because the landing's side reference file was over the limit the repository linter
//!  enforces (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = keepAnUnlandedCommit(git, repository, seq, held, tip, announce);
//! ```

use crate::landing_state_carried_by_its_task::{
    LandingState,
    on_the_landing_state,
};
use crate::service_index_lock::{
    Announce,
    Announcer,
};
use crate::service_land_context::LandContext;
use crate::service_protocol::CommitRequest;
use crate::service_scratch_ref_of_a_dropped_batch::kept_ref;
use crate::the_answer_of_one_git_step_of_a_side_reference::{
    StepAnswer,
    answering,
};
use crate::the_side_references_a_commit_is_kept_on::{
    keep_the_batches_commit,
    keep_the_batches_commit_awaiting,
};
use crate::the_write_of_a_side_reference::{
    the_batch_reference_names,
    the_reference_itself_names_awaiting,
};
use crate::transaction_git::{
    GitContext,
    GitRequest,
    NO_REFLOG_SETTING,
};
use std::path::Path;

/// What became of keeping the commit a batch built on the reference a commit that did not land is kept on,
/// and what the person is told about it.
/// Why:
///  A caller that lets go of the reference the batch held the commit on must know whether another reference names it:
///  a reference that is the only name of a commit is one `git gc --prune=now` takes,
///  so it is removed only after another name is known to be written
///  (the owner's rule "never discard user data").
///  The sentence and the answer come from one read of the reference after the write,
/// so no caller can decide from a keep it did not prove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AKeptCommit = { named: boolean; line: string }
/// ```
pub(crate) struct AKeptCommit {
    /// Whether a reference of this repository names the commit now.
    pub(crate) named: bool,
    /// The line the person reads,
    /// naming the reference the commit is kept at and,
    /// when no reference could hold it,
    /// the commit and the reference it stays on.
    pub(crate) line: String,
}

/// What:
///  Keeps the commit built for a batch that was withdrawn on a side ref,
/// and answers whether a reference of this repository names it and the line that tells the person where it is.
/// Why:
///   The owner's answer for a wait that reaches its bound is that nothing lands later by itself and the commit is kept for a retry;
///  the person's staged changes are untouched,
/// so running the commit again builds the same commit,
/// and the side ref lets them
///  recover this one if the worktree changed meanwhile.
///  A keep Git refused leaves the batch's own scratch reference as the only name the commit has,
///  so the person is told the commit's name and that reference:
///  a sentence that named a recoverable commit without saying where it is would read as a commit that was lost.
///  A batch whose own reference was never written has no name either,
///  and then the person is told the object's own name and how to make a reference of it
///  (`the_words_for_a_commit_no_reference_names`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = keepOnASideRef(git, repository, seq, tip); return { named: !kept, line: kept ? `could not be kept on a side ref (${kept}), so it stays at ${scratchRef(seq)}` : `kept at refs/cli-git/kept/${seq}` }
/// ```
pub(crate) fn keep_unlanded(
    git: &GitContext,
    repository: &Path,
    seq: u64,
    held: &str,
    tip: &str,
) -> AKeptCommit {
    let name: String = kept_ref(seq);
    let Some(words) = keep_the_batches_commit(git, repository, seq, tip) else {
        return AKeptCommit {
            named: true,
            line: format!(
                "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
            ),
        };
    };
    let stays: String = match the_batch_reference_names(git, repository, held, tip) {
        true => format!("so it stays at {held} (see it with `git log {held}`)"),
        false => crate::the_clause_a_commit_no_reference_proves::the_words_for_a_commit_no_reference_names(tip),
    };
    return AKeptCommit {
        named: false,
        line: format!(
            "monochromatic-git-policy-cli: the commit built for you, {tip}, could not be kept on a side ref ({words}), {stays}; nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
        ),
    };
}

/// What:
///  `keep_unlanded` with the write of the kept reference awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the line that tells the person where their commit is kept is
///  written when a landing that ended at a step ends;
/// the reference,
/// the two sentences and the reason each of them is
///  said are the synchronous face's,
/// so the person is told the same thing on both faces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = await keepUnlandedAwaiting(git, repository, seq, tip);
/// ```
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"
    )
)]
pub(crate) async fn keep_unlanded_awaiting(
    git: &GitContext,
    repository: &Path,
    seq: u64,
    held: &str,
    tip: &str,
) -> AKeptCommit {
    let name: String = kept_ref(seq);
    let Some(words) = keep_the_batches_commit_awaiting(git, repository, seq, tip).await else {
        return AKeptCommit {
            named: true,
            line: format!(
                "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
            ),
        };
    };
    let stays: String = match the_reference_itself_names_awaiting(git, repository, held, tip).await {
        true => format!("so it stays at {held} (see it with `git log {held}`)"),
        false => crate::the_clause_a_commit_no_reference_proves::the_words_for_a_commit_no_reference_names(tip),
    };
    return AKeptCommit {
        named: false,
        line: format!(
            "monochromatic-git-policy-cli: the commit built for you, {tip}, could not be kept on a side ref ({words}), {stays}; nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
        ),
    };
}

/// What:
///  Keeps the commit a batch built that is not on its branch,
///  tells the person where it is kept,
///  and lets go of the reference the batch held it on only once another reference names it.
/// Why:
///   Every path that answers a batch as not made after Git built its commit ends here,
///  and the rule is one rule:
///  a reference that is the only name of a built commit is deleted only after another name for that commit is known to be written,
///  and never when the name it deleted was not the value that was read
///  (a compare-and-delete,
///  `git update-ref -d <ref> <old>`):
///  a commit no reference names is one `git gc --prune=now` takes,
///  and what a person made is never discarded
///  (the owner's rule).
///  A keep the wrapper could not write leaves the scratch reference where it is,
///  and the person is told the commit's name and that reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = keepUnlanded(git, repository, seq, tip); announce(kept.line); if (kept.named) await updateRefDelete(scratchRef(seq), tip);
/// ```
pub(crate) fn keep_an_unlanded_commit(
    git: &GitContext,
    repository: &Path,
    seq: u64,
    held: &str,
    tip: &str,
    announce: &dyn Announcer,
) -> AKeptCommit {
    let kept: AKeptCommit = keep_unlanded(git, repository, seq, held, tip);
    announce.announce(Announce::Waiting(kept.line.clone()));
    if !kept.named {
        return kept;
    }
    if let_go_of_a_reference_only_while_it_names(git, repository, held, tip) {
        return kept;
    }
    crate::service_warn!(
        "batch {seq} {}: the commit {tip} is kept, and its scratch reference {held} was left as it is: it does not name that commit any more, or it could not be removed",
        crate::service_timing::in_the_repository_at(repository)
    );
    return kept;
}

/// What:
///  Removes the reference `name` only while it still names `tip`,
///  which is Git's own compare-and-delete (`update-ref -d <ref> <old>`).
/// Why:
///   A command that reads a reference and then removes it by name alone removes whatever another command wrote in between,
///  and that may be the only name of another commit
///  (the owner's rule "never discard user data").
///  The expected old value is the value that was read,
///  so the removal happens only when the reference still holds what this command proved safe to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = await git(['-c', 'core.logAllRefUpdates=false', 'update-ref', '-d', '--no-deref', name, tip]); return removed.ok
/// ```
pub(crate) fn let_go_of_a_reference_only_while_it_names(
    git: &GitContext,
    repository: &Path,
    name: &str,
    tip: &str,
) -> bool {
    return matches!(
        answering(git, GitRequest::new(repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", name, tip])),
        StepAnswer::Answered(output) if output.succeeded()
    );
}

/// What:
///  Remembers the tip a batch built whose own side reference was not proven to name it,
/// and why,
/// for the landing that answers the batch.
/// Why:
///  The landing answers with a failure,
///  and nothing else names the commit it built,
///  so the answer must keep it and tell the person where
///  (`keep_what_a_side_reference_write_left`).
///  The landing's state carries it because the step that built the commit answers with a code alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landingState.unprovenSideRef = { tip, why }
/// ```
pub(crate) fn remember_the_tip_a_side_reference_does_not_hold(
    tip: &str,
    why: String,
) {
    on_the_landing_state(
        UnheldTip {
            tip: tip.to_string(),
            why,
        },
        keep_the_unheld_tip,
    );
}

/// One tip a batch built whose side reference did not keep it,
/// and why.
struct UnheldTip {
    /// The commit the batch built.
    tip: String,
    /// What the write of the batch's own side reference came to,
    /// in a phrase the person reads.
    why: String,
}

fn keep_the_unheld_tip(
    state: &LandingState,
    unheld: UnheldTip,
) {
    *state.unproven_side_ref_tip.borrow_mut() = Some((unheld.tip, unheld.why));
}

/// Takes the tip a batch built whose side reference did not keep it,
/// once.
fn take_the_tip_a_side_reference_does_not_hold() -> Option<(String, String)> {
    return on_the_landing_state((), take_the_unheld_tip);
}

fn take_the_unheld_tip(
    state: &LandingState,
    (): (),
) -> Option<(String, String)> {
    return state.unproven_side_ref_tip.take();
}

/// What:
///  Keeps the commit a batch built when the write of the batch's own side reference did not keep it,
/// on `refs/cli-git/kept/<seq>`,
/// and tells the person where it is.
/// Why:
///   The landing answers such a batch with a failure,
///  and the commit it built is on no branch:
///  a commit that no reference names is one `git gc --prune=now` takes,
///  so the landing that could not hold it on its own reference holds it where a commit that did not land is kept
///  (the owner's rule "never discard user data").
///  Nothing was changed yet when this runs,
/// because it runs before the branch write,
///  so the person's staged changes and their worktree are as they were.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unheld = takeTheUnheldTip(); if (unheld) { announce(`the write of this batch's own side reference did not keep the commit Git built (${unheld.why}), so this commit was not made.`); announce(keepTheBatchesCommit(git, repo, seq, unheld.tip)) }
/// ```
pub(crate) fn keep_what_a_side_reference_write_left(
    context: &LandContext,
    announce: &dyn Announcer,
    requests: &[CommitRequest],
    seq: u64,
) {
    let Some((tip, why)) = take_the_tip_a_side_reference_does_not_hold() else {
        return;
    };
    let Some(first) = requests.first() else {
        return;
    };
    announce.announce(Announce::Waiting(format!(
        "monochromatic-git-policy-cli: the write of this batch's own side reference did not keep the commit Git built ({why}), so this commit was not made.\n"
    )));
    let git: GitContext = GitContext {
        common_dir: Some(first.common_dir.clone()),
        ..context.git.clone()
    };
    announce.announce(Announce::Waiting(
        keep_the_tip_a_side_reference_did_not_hold(
            &git,
            first.repository.as_path(),
            seq,
            tip.as_str(),
        ),
    ));
}

/// What:
///  Keeps the tip a batch built on the reference a commit that did not land is kept on,
/// and answers the sentence the person reads.
/// Why:
///  The batch is answered as not made,
///  and the commit must still be findable:
///  the name is the one the ceiling of an ended step keeps its commit on,
///  so a person who looks for a commit the wrapper kept looks in one family.
///  A kept write that is refused too leaves the commit named by nothing at all,
///  and the one thing this landing can still give the person is the object's own name,
///  with what to do with it before a maintenance that drops what no reference names takes it
///  (the owner's rule "never discard user data").
fn keep_the_tip_a_side_reference_did_not_hold(
    git: &GitContext,
    repository: &Path,
    seq: u64,
    tip: &str,
) -> String {
    let name: String = kept_ref(seq);
    let refused: Option<String> = keep_the_batches_commit(git, repository, seq, tip);
    let Some(words) = refused else {
        return format!(
            "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again.\n"
        );
    };
    return format!(
        "monochromatic-git-policy-cli: the commit built for you, {tip}, could not be kept on a side reference either ({words}), and no reference is proven to name it, so a maintenance that drops what no reference names would take it; keep it with `git update-ref refs/heads/<the name you want> {tip}`, and your staged changes and your worktree are as they were. Run the commit again.\n"
    );
}
