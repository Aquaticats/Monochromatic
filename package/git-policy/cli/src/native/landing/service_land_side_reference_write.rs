//! What:
//!  The write of one of a landing's own side references,
//!  and what the landing does with a commit whose reference it could not prove:
//!  the write is made once more only when the wait for it ended without an answer,
//!  it never replaces a reference another command wrote,
//!  and its answer is read from the reference's own value.
//! Why:
//!   A side reference is what keeps a commit a reference lock held reachable
//!  (`held_commit_record.rs`),
//!  and the wrapper never ends such a write for being slow,
//!  because cutting a write in place risks the person's data
//!  (`transaction_git_step_bounds.rs`,
//!  `is_side_ref_write`):
//!  a wait that ends without an answer says the result is not known,
//!  never that the write failed.
//!  Reading that ending as a failed write told a commit that was made that it was not made
//!  (2026-11-11,
//!  the release gate of 0.6.1-alpha.11:
//!  the write of `refs/worktree/cli-git-service/<batch>` was still moving after its wait of 127 seconds,
//!  in quiet gaps under a second,
//!  and the landing answered `unavailable`).
//!  A commit the batch built and no reference names is one `git gc --prune=now` takes,
//!  so a batch whose own reference the write did not prove is answered as not made
//!  and the commit is kept where a commit that did not land is kept,
//!  named to the person
//!  (the owner's rule "never discard user data").
//!  It sits in a file of its own because the landing's Git steps are at the limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(await keepOnASideRef(git, repo, name, tip)).toBeUndefined()
//! ```

use crate::child_watch::{is_ceiling, is_ended_by_stop, is_not_known};
use crate::landing_state_carried_by_its_task::{LandingState, on_the_landing_state};
use crate::service_index_lock::{Announce, Announcer};
use crate::service_land_context::LandContext;
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, kept_ref, value_of, value_of_awaiting};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, NO_REFLOG_SETTING, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::path::Path;

/// Keeps the sentence of a step that was ended at its ceiling,
/// for the landing that answers it.
pub(crate) fn keep_the_ceiling_step(state: &LandingState, sentence: String) {
    *state.ceiling_step.borrow_mut() = Some(sentence);
}

/// Takes the sentence of the step that was ended at its ceiling out of the landing's state.
fn take_the_ceiling_step(state: &LandingState, (): ()) -> Option<String> {
    return state.ceiling_step.take();
}

/// The sentence of the step that was ended at its ceiling,
/// once;
/// the landing keeps it for the line the clients are told (the state `ceiling_step` of `landing_state_carried_by_its_task.rs`).
pub(crate) fn take_ceiling_step() -> Option<String> {
    return on_the_landing_state((), take_the_ceiling_step);
}

/// How many times a side reference is written when the wait for a write ended without an answer.
///  Two:
///  a write a wait left running is often finished moments later,
///  so one write more after the reference is read costs a step and saves the commit.
///  A write Git itself refused is written no second time:
///  the same value at the same name is refused the same way,
///  and a step a stop ended keeps its own code.
///  Unmeasured:
///  the count is the smallest that lets one wait which ended be finished by a read and one write after it,
///  and another attempt would multiply the wait of a write the disk makes slow on every try.
const SIDE_REF_WRITE_ATTEMPTS: usize = 2;

/// What became of one run of a Git step,
/// for a writer that tells a wait with no answer apart from an ending.
/// Why:
///  A step the wrapper never ends is not ended for being slow
///  (`transaction_git_step_bounds.rs`,
///  `is_side_ref_write`),
///  so the wait for it can end before the step does,
///  and that ending says the result is not known,
/// never that the step failed.
///  A write may be made once more when its answer is not known;
///  a write Git refused may not,
/// and a step a stop or a ceiling ended keeps the code the landing answers with.
///  The step runner of the landing reads it,
/// so a write that asks the reference itself tells the two apart
///  (`service_land_git_steps_and_import_stream.rs`,
///  `run`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StepAnswer = { answered: GitOutput } | 'left-running' | { ended: FailureCode }
/// ```
pub(crate) enum StepAnswer {
    /// Git ran and answered.
    Answered(GitOutput),
    /// The command stopped waiting for the step,
    /// which is left running,
    /// so its result is not known.
    LeftRunning,
    /// The step was ended,
    /// or Git could not be run,
    /// and the code says which.
    Ended(FailureCode),
}

/// What:
///  Runs one Git step and says what its answer was,
/// keeping the sentence of a step that was ended.
/// Why:
///  `run` answers a failure code and nothing more,
///  and a writer that may be made again needs the difference between a wait that ended without an answer and a write Git
///  refused (`StepAnswer`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function answering(git, request): StepAnswer { try { return { answered: await runGit(git, request) } } catch (error) { if (isNotKnown(error)) return 'left-running'; if (isCeiling(error) || isEndedByStop(error)) { rememberCeilingStep(error.message); return { ended: 'StepTooSlow' } } return { ended: 'Unavailable' } } }
/// ```
pub(crate) fn answering(git: &GitContext, request: GitRequest) -> StepAnswer {
    return match run_git(git, &request) {
        Ok(output) => StepAnswer::Answered(output),
        Err(error) if is_not_known(&error) => StepAnswer::LeftRunning,
        Err(error) => {
            // A step ended at its ceiling, or by the service's stop, is told apart from every other failure, and its sentence is kept
            //  for the answer; the commit it belongs to is kept on a side ref and not made (`land_batch_timed`).
            if is_ceiling(&error) || is_ended_by_stop(&error) {
                on_the_landing_state(error.to_string(), keep_the_ceiling_step);
                return StepAnswer::Ended(FailureCode::StepTooSlow);
            }
            StepAnswer::Ended(FailureCode::Unavailable)
        }
    };
}

/// What:
///  `answering` with the step's Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
///  and a side reference is written from the landing once the landing is a task;
/// the three answers and the sentence kept for a step that was ended
///  are the synchronous face's,
/// so a commit one face keeps is kept by the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function answeringAwaiting(git, request): StepAnswer { try { return { answered: await runGitAwaiting(git, request) } } catch (error) { if (isNotKnown(error)) return 'left-running'; if (isCeiling(error) || isEndedByStop(error)) { rememberCeilingStep(error.message); return { ended: 'StepTooSlow' } } return { ended: 'Unavailable' } } }
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn answering_awaiting(git: &GitContext, request: GitRequest) -> StepAnswer {
    return match run_git_awaiting(git, &request).await {
        Ok(output) => StepAnswer::Answered(output),
        Err(error) if is_not_known(&error) => StepAnswer::LeftRunning,
        Err(error) => {
            if is_ceiling(&error) || is_ended_by_stop(&error) {
                on_the_landing_state(error.to_string(), keep_the_ceiling_step);
                return StepAnswer::Ended(FailureCode::StepTooSlow);
            }
            StepAnswer::Ended(FailureCode::Unavailable)
        }
    };
}

/// What:
///  Whether the reference's own value names `tip`,
/// never what a symbolic reference leads to.
/// Why:
///  Every write of a side reference is `--no-deref`,
///  so a reference whose own value is the name of another reference was not written by one of them,
///  and the commit it leads to is held by that other reference alone:
///  the command that moves it takes the commit with it
///  (the owner's rule "never discard user data").
///  A read that followed the symbolic reference answered a refused write with a success that had not happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theReferenceItselfNames = (git, repository, name, tip) => valueOf(git, repository, name).names === tip
/// ```
fn the_reference_itself_names(git: &GitContext, repository: &Path, name: &str, tip: &str) -> bool {
    return matches!(value_of(git, repository, name), RefValue::Names(named) if named == tip);
}

/// What:
///  Writes the side reference `name` at `tip`,
///  only while that name names nothing,
///  and answers what became of it,
///  asking the reference itself when the wait for a write ended without an answer.
/// Why:
///   A side reference is what keeps a commit a reference lock held reachable
///  (`held_commit_record.rs`),
///  and the wrapper never ends such a write for being slow,
///  because cutting a write in place risks the person's data
///  (`transaction_git_step_bounds.rs`,
///  `is_side_ref_write`):
///  a wait that ends without an answer says the result is not known,
///  never that the write failed.
///  Reading that ending as a failed write told a commit that was made that it was not made
///  (2026-11-11,
///  the release gate of 0.6.1-alpha.11:
///  the write of `refs/worktree/cli-git-service/<batch>` was still moving after its wait of 127 seconds,
///  in quiet gaps under a second,
///  and the landing answered `unavailable`).
///  The write names one value for one name,
///  so it may be written again,
///  and the reference is the answer:
///  it is read,
///  never guessed at from a wait that ended.
///  The empty old value makes every attempt a create:
///  a name another command writes is never replaced,
///  whether it wrote it before this command started or between two attempts,
///  because the commit that name holds may be the only thing naming it
///  (the owner's rule "never discard user data").
///  A reference whose own value is the name of another reference is not this write's either,
///  because every attempt follows none (`the_reference_itself_names`).
///  An ending that is a stop or a ceiling keeps its own code,
///  because the landing's answer for such a step is the step's own sentence and the commit it keeps beside it.
///  Limit:
///  a write a wait left running is not ended,
///  so it can finish after the branch write removed the reference a landing no longer needs,
///  and the reference it recreates names the commit its branch already has;
///  that reference stays,
///  because the record that would let a restart sweep it goes when the landing ends.
///  It costs space once that commit leaves the branch,
///  never the person's data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let attempt = 0; attempt < 2; attempt++) { const made = await writeRefOnlyIfFree(name, tip); if (made.succeeded) return 'written'; if (await valueOf(name).names === tip) return 'written'; if (made.answered) return { refused: wordsOf(made) }; leftRunning = !made.answered } return leftRunning ? 'left-running' : { refused: words }
/// ```
fn write_until_the_reference_names(git: &GitContext, repository: &Path, name: &str, tip: &str) -> ASideRefWrite {
    for _attempt in 0..SIDE_REF_WRITE_ATTEMPTS {
        // The empty old value is Git's way to refuse when the name exists at all, so a reference another command wrote is never
        //  replaced (`keep_on_a_side_ref`).
        let made: StepAnswer = answering(git, GitRequest::new(repository, &["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", name, tip, ""]));
        if matches!(&made, StepAnswer::Answered(output) if output.succeeded()) {
            return ASideRefWrite::Written;
        }
        // The reference says whether a write finished, whatever the wait for it said, and its own value is what is asked for: what a
        //  symbolic reference leads to is another command's to move.
        if the_reference_itself_names(git, repository, name, tip) {
            return ASideRefWrite::Written;
        }
        match made {
            // Git answered, so the same value at the same name is refused the same way.
            StepAnswer::Answered(output) => return ASideRefWrite::Refused(refused_words_of(&output)),
            // The wait ended without an answer: the step is left running, and the reference did not show its write, so it is written
            //  once more.
            StepAnswer::LeftRunning => {}
            // A stop or a ceiling ended the step, or Git could not be run.
            StepAnswer::Ended(code) => return ASideRefWrite::Ended(code),
        }
    }
    return ASideRefWrite::LeftRunning;
}

/// What:
///  `write_until_the_reference_names` with every Git child awaited on the runtime,
///  and the same create-only attempts.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and a side reference is written from the landing once the landing is a task;
/// the attempts,
/// the read of the reference after a write that gave no answer and the four answers a caller tells
///  apart are the synchronous face's,
/// so a commit one face keeps is kept by the other,
/// and neither can replace a name another command wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let attempt = 0; attempt < 2; attempt++) { const made = await writeRefOnlyIfFreeAwaiting(name, tip); if (made.succeeded) return 'written'; if (await valueOfAwaiting(name).names === tip) return 'written'; if (made.answered) return { refused: wordsOf(made) }; leftRunning = !made.answered } return leftRunning ? 'left-running' : { refused: words }
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
async fn write_until_the_reference_names_awaiting(git: &GitContext, repository: &Path, name: &str, tip: &str) -> ASideRefWrite {
    for _attempt in 0..SIDE_REF_WRITE_ATTEMPTS {
        let made: StepAnswer = answering_awaiting(git, GitRequest::new(repository, &["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", name, tip, ""])).await;
        if matches!(&made, StepAnswer::Answered(output) if output.succeeded()) {
            return ASideRefWrite::Written;
        }
        if the_reference_itself_names_awaiting(git, repository, name, tip).await {
            return ASideRefWrite::Written;
        }
        match made {
            StepAnswer::Answered(output) => return ASideRefWrite::Refused(refused_words_of(&output)),
            StepAnswer::LeftRunning => {}
            StepAnswer::Ended(code) => return ASideRefWrite::Ended(code),
        }
    }
    return ASideRefWrite::LeftRunning;
}

/// What:
///  `the_reference_itself_names` with its read awaited on the runtime.
/// Why:
///  A task of the runtime cannot enter it to wait for a child,
/// and the answer a write rests on is the reference's own value,
/// which is read from that task;
/// the question and what counts as the answer are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theReferenceItselfNamesAwaiting = async (git, repository, name, tip) => (await valueOfAwaiting(git, repository, name)).names === tip
/// ```
async fn the_reference_itself_names_awaiting(git: &GitContext, repository: &Path, name: &str, tip: &str) -> bool {
    return matches!(value_of_awaiting(git, repository, name).await, RefValue::Names(named) if named == tip);
}

/// What became of one write of one of the wrapper's own side references.
/// Why:
///  The write answers four different things,
/// and a caller that keeps a commit or answers a commit tells them apart:
///  a reference that names the commit,
///  a write Git refused,
///  a write the command stopped waiting for,
///  and a step that was ended or could not be run,
///  whose code is the landing's own answer for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ASideRefWrite = 'written' | { refused: string } | 'left-running' | { ended: FailureCode }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ASideRefWrite {
    /// The reference names the object the write was for.
    Written,
    /// Git answered that it refused the write,
    /// in its own words.
    Refused(String),
    /// The command stopped waiting for the write,
    /// which is left running,
    /// so whether the reference names the object is not known.
    LeftRunning,
    /// The write was ended or could not be run,
    /// and the code says what ended it.
    Ended(FailureCode),
}

impl ASideRefWrite {
    /// What the person is told when the write did not put the reference where it was wanted,
    /// or nothing when it did.
    /// Why:
    ///  The answers are one sentence for a caller that only says where a commit was kept;
    ///  the walk that decides a commit keeps the codes instead (`service_land_objects_of_batch.rs`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const refusal = (write: ASideRefWrite) => write === 'written' ? undefined : write.refused ?? (write === 'left-running' ? 'the write was left running' : `Git could not be run (${write.ended})`)
    /// ```
    pub(crate) fn refusal(&self) -> Option<String> {
        return match self {
            Self::Written => None,
            Self::Refused(words) => Some(words.clone()),
            Self::LeftRunning => Some(String::from("the write was still running when this command stopped waiting for it, so whether the reference names the commit is not known")),
            Self::Ended(code) => Some(format!("Git could not be run ({code:?})")),
        };
    }
}

/// What:
///  Keeps the commit of batch `seq` on `refs/cli-git/kept/<number>`,
/// the ref the wrapper keeps a commit it did not land on,
/// and returns nothing when the reference names it or Git's own words when it does not.
/// Why:
///   One place names that ref and writes it,
/// so the side ref of a landing that gave up at a step and the side ref the restart writes for a batch its branch does
/// not show cannot drift apart;
/// the write follows no symbolic ref,
/// and it passes the setting that keeps Git from making a reflog for a ref that has none
/// (`SPEC.md`,
/// a setting that does not stop an append to a log another Git made).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepTheBatchesCommit(git, repo, 1, tip)).toBeUndefined()
/// ```
pub(crate) fn keep_the_batches_commit(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> Option<String> {
    return keep_on_a_side_ref(git, repository, kept_ref(seq).as_str(), tip);
}

/// What:
///  `keep_the_batches_commit` with every Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and a commit a landing could not put on a branch is kept on its
///  own kept reference once the landing is a task;
/// the name and the write are the synchronous face's,
/// so a commit one face keeps is kept where the other keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepTheBatchesCommitAwaiting(git, repo, 1, tip)).toBeUndefined()
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn keep_the_batches_commit_awaiting(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> Option<String> {
    return keep_on_a_side_ref_awaiting(git, repository, kept_ref(seq).as_str(), tip).await;
}

/// What:
///  Writes the per-worktree reference `refs/worktree/cli-git-service/<seq>` at `tip`,
///  the reference that keeps the objects of a batch reachable until its branch write,
///  and answers what became of the write.
/// Why:
///  This is the write that keeps a commit Git built reachable while its branch write meets a lock
///  (`service_land_objects_of_batch.rs`),
///  and it is the one the release gate of 0.6.1-alpha.11 read as a failed write when its wait ran out;
///  it is written here so that reading is answered by the reference
///  (`write_until_the_reference_names`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnTheScratchReference(git, repo, scratchRef(seq), tip)).toEqual('written')
/// ```
pub(crate) fn keep_on_the_scratch_reference(git: &GitContext, repository: &Path, scratch: &str, tip: &str) -> ASideRefWrite {
    return write_until_the_reference_names(git, repository, scratch, tip);
}

/// What:
///  `keep_on_the_scratch_reference` with every Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the reference that keeps a batch's objects reachable is written
///  from the landing once it is a task;
/// the write,
/// its two attempts and the read that answers a wait which ended are the
///  synchronous face's,
/// so the two cannot disagree about what the reference names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnTheScratchReferenceAwaiting(git, repo, scratchRef(seq), tip)).toEqual('written')
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn keep_on_the_scratch_reference_awaiting(git: &GitContext, repository: &Path, scratch: &str, tip: &str) -> ASideRefWrite {
    return write_until_the_reference_names_awaiting(git, repository, scratch, tip).await;
}

/// What:
///  Writes the reference `name` at `tip`
/// only while that name names nothing,
/// and answers nothing when the reference names `tip`
///  or Git's own words when it does not.
/// Why:
///   A command that removes a linked worktree keeps a held commit on a reference of the common Git directory
///  before Git removes the only other ones (`worktree_removal_held_commits.rs`),
///  and a name that another command already wrote must never be replaced:
///  replacing it would forget the commit that reference was made for,
///  which is the owner's rule "never discard user data" again.
///  The empty old value is Git's `--stdin`-free way to ask for exactly that:
///  `update-ref <ref> <new> ""` refuses when the name exists at all,
///  whether this command read it first or another one wrote it in between,
///  and it is what every attempt of `write_until_the_reference_names` asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnASideRef(git, repo, name, tip)).toBeUndefined()
/// ```
pub(crate) fn keep_on_a_side_ref(git: &GitContext, repository: &Path, name: &str, tip: &str) -> Option<String> {
    return write_until_the_reference_names(git, repository, name, tip).refusal();
}

/// What:
///  `keep_on_a_side_ref` with every Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and a commit a landing could not put on a branch is kept on its
///  side reference once the landing is a task;
/// the name,
/// the create-only write and the words a refusal gives are the synchronous face's,
/// so a commit is kept at one name either way,
///  and a name another command wrote is replaced by neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = await keepOnASideRefAwaiting(git, repo, name, tip);
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn keep_on_a_side_ref_awaiting(git: &GitContext, repository: &Path, name: &str, tip: &str) -> Option<String> {
    return write_until_the_reference_names_awaiting(git, repository, name, tip).await.refusal();
}

/// What:
///  Git's own words for a write it refused,
/// in one sentence.
/// Why:
///  A refusal Git explained is explained with its words,
/// and a refusal it did not explain is said with the status it ended with,
/// so every caller names the same thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusedWordsOf = (output) => output.stderr.trim() || `git update-ref ended with ${output.code ?? 'no exit status'}`
/// ```
fn refused_words_of(output: &GitOutput) -> String {
    let words: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_string();
    if !words.is_empty() {
        return words;
    }
    return match output.code {
        Some(code) => format!("git update-ref ended with {code}"),
        None => String::from("git update-ref ended with no exit status"),
    };
}

/// What:
///  Keeps the commit built for a batch that was withdrawn on a side ref,
/// and returns the line that tells the person where it is.
/// Why:
///   The owner's answer for a wait that reaches its bound is that nothing lands later by itself and the commit is kept for a retry;
///  the person's staged changes are untouched,
/// so running the commit again builds the same commit,
/// and the side ref lets them
///  recover this one if the worktree changed meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await keepOnASideRef(git, repository, seq, tip); return `kept at refs/cli-git/kept/${seq}`
/// ```
pub(crate) fn keep_unlanded(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> String {
    let name: String = kept_ref(seq);
    if keep_the_batches_commit(git, repository, seq, tip).is_some() {
        return String::from("monochromatic-git-policy-cli: the commit built for you could not be kept on a side ref; your staged changes and your worktree are as they were, so run the commit again when the program above is done.\n");
    }
    return format!(
        "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
    );
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
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn keep_unlanded_awaiting(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> String {
    let name: String = kept_ref(seq);
    if keep_the_batches_commit_awaiting(git, repository, seq, tip).await.is_some() {
        return String::from("monochromatic-git-policy-cli: the commit built for you could not be kept on a side ref; your staged changes and your worktree are as they were, so run the commit again when the program above is done.\n");
    }
    return format!(
        "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
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
pub(crate) fn remember_the_tip_a_side_reference_does_not_hold(tip: &str, why: String) {
    on_the_landing_state(UnheldTip { tip: tip.to_string(), why }, keep_the_unheld_tip);
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

fn keep_the_unheld_tip(state: &LandingState, unheld: UnheldTip) {
    *state.unproven_side_ref_tip.borrow_mut() = Some((unheld.tip, unheld.why));
}

/// Takes the tip a batch built whose side reference did not keep it,
/// once.
fn take_the_tip_a_side_reference_does_not_hold() -> Option<(String, String)> {
    return on_the_landing_state((), take_the_unheld_tip);
}

fn take_the_unheld_tip(state: &LandingState, (): ()) -> Option<(String, String)> {
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
pub(crate) fn keep_what_a_side_reference_write_left(context: &LandContext, announce: &dyn Announcer, requests: &[CommitRequest], seq: u64) {
    let Some((tip, why)) = take_the_tip_a_side_reference_does_not_hold() else { return };
    let Some(first) = requests.first() else { return };
    announce.announce(Announce::Waiting(format!(
        "monochromatic-git-policy-cli: the write of this batch's own side reference did not keep the commit Git built ({why}), so this commit was not made.\n"
    )));
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    announce.announce(Announce::Waiting(keep_the_tip_a_side_reference_did_not_hold(&git, first.repository.as_path(), seq, tip.as_str())));
}

/// What:
///  Keeps the tip a batch built on the reference a commit that did not land is kept on,
/// and answers the sentence the person reads.
/// Why:
///  The batch is answered as not made,
///  and the commit must still be findable:
///  the name is the one the ceiling of an ended step keeps its commit on,
///  so a person who looks for a commit the wrapper kept looks in one family.
fn keep_the_tip_a_side_reference_did_not_hold(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> String {
    let name: String = kept_ref(seq);
    let refused: Option<String> = keep_the_batches_commit(git, repository, seq, tip);
    let Some(words) = refused else {
        return format!(
            "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again.\n"
        );
    };
    return format!(
        "monochromatic-git-policy-cli: the commit built for you could not be kept on a side reference either ({words}); your staged changes and your worktree are as they were, so run the commit again.\n"
    );
}
