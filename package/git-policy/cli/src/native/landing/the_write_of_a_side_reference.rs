//! What:
//!  The write of one of a landing's own side references:
//!  it is made once more only when the wait for it ended without an answer,
//!  it never replaces a reference another command wrote,
//!  its answer is read from the reference's own value,
//!  and the words a person reads say what Git refused.
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
//!  The write names one value for one name,
//!  so it may be written again,
//!  and the reference is the answer:
//!  it is read,
//!  never guessed at from a wait that ended.
//!  It sits in a file of its own because the landing's side reference file was over the limit the repository linter
//!  enforces (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const made: ASideRefWrite = keepOnTheScratchReference(git, repository, scratchRef(seq), tip);
//! ```

use crate::service_protocol::FailureCode;
use crate::service_scratch_ref_of_a_dropped_batch::{
    RefValue,
    value_of,
    value_of_awaiting,
};
use crate::the_answer_of_one_git_step_of_a_side_reference::{
    StepAnswer,
    answering,
    answering_awaiting,
};
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    NO_REFLOG_SETTING,
};
use std::path::Path;

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
pub(crate) fn the_batch_reference_names(
    git: &GitContext,
    repository: &Path,
    name: &str,
    tip: &str,
) -> bool {
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
///  because every attempt follows none (`the_batch_reference_names`).
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
pub(crate) fn write_until_the_reference_names(
    git: &GitContext,
    repository: &Path,
    name: &str,
    tip: &str,
) -> ASideRefWrite {
    for _attempt in 0..SIDE_REF_WRITE_ATTEMPTS {
        // The empty old value is Git's way to refuse when the name exists at all, so a reference another command wrote is never
        //  replaced (`keep_on_a_side_ref`).
        let made: StepAnswer = answering(
            git,
            GitRequest::new(
                repository,
                &[
                    "-c",
                    NO_REFLOG_SETTING,
                    "update-ref",
                    "--no-deref",
                    name,
                    tip,
                    "",
                ],
            ),
        );
        if matches!(&made, StepAnswer::Answered(output) if output.succeeded()) {
            return ASideRefWrite::Written;
        }
        // The reference says whether a write finished, whatever the wait for it said, and its own value is what is asked for: what a
        //  symbolic reference leads to is another command's to move.
        if the_batch_reference_names(git, repository, name, tip) {
            return ASideRefWrite::Written;
        }
        match made {
            // Git answered, so the same value at the same name is refused the same way.
            StepAnswer::Answered(output) => {
                return ASideRefWrite::Refused(refused_words_of(&output));
            }
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
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"
    )
)]
pub(crate) async fn write_until_the_reference_names_awaiting(
    git: &GitContext,
    repository: &Path,
    name: &str,
    tip: &str,
) -> ASideRefWrite {
    for _attempt in 0..SIDE_REF_WRITE_ATTEMPTS {
        let made: StepAnswer = answering_awaiting(
            git,
            GitRequest::new(
                repository,
                &[
                    "-c",
                    NO_REFLOG_SETTING,
                    "update-ref",
                    "--no-deref",
                    name,
                    tip,
                    "",
                ],
            ),
        )
        .await;
        if matches!(&made, StepAnswer::Answered(output) if output.succeeded()) {
            return ASideRefWrite::Written;
        }
        if the_reference_itself_names_awaiting(git, repository, name, tip).await {
            return ASideRefWrite::Written;
        }
        match made {
            StepAnswer::Answered(output) => {
                return ASideRefWrite::Refused(refused_words_of(&output));
            }
            StepAnswer::LeftRunning => {}
            StepAnswer::Ended(code) => return ASideRefWrite::Ended(code),
        }
    }
    return ASideRefWrite::LeftRunning;
}

/// What:
///  `the_batch_reference_names` with its read awaited on the runtime.
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
pub(crate) async fn the_reference_itself_names_awaiting(
    git: &GitContext,
    repository: &Path,
    name: &str,
    tip: &str,
) -> bool {
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
            Self::LeftRunning => Some(String::from(
                "the write was still running when this command stopped waiting for it, so whether the reference names the commit is not known",
            )),
            Self::Ended(code) => Some(format!("Git could not be run ({code:?})")),
        };
    }
}

/// What:
///  The one line of what Git said when a step refused,
///  for a sentence a person reads.
/// Why:
///  `fast-import` follows its refusal with its whole statistics report,
///  and a line that told the person the reference was not written would carry twenty lines of it:
///  the first line is the one that names the refusal,
///  and a step that said nothing falls back to the status it ended with
///  (`refused_words_of`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theRefusalLineOf = (output) => output.stderr.split('\n').map((line) => line.trim()).find((line) => line.length > 0) ?? refusedWordsOf(output)
/// ```
pub(crate) fn the_refusal_line_of(output: &GitOutput) -> String {
    let words: String = String::from_utf8_lossy(output.stderr.as_slice()).to_string();
    for line in words.lines() {
        let trimmed: &str = line.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    return refused_words_of(output);
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
pub(crate) fn refused_words_of(output: &GitOutput) -> String {
    let words: String = String::from_utf8_lossy(output.stderr.as_slice())
        .trim()
        .to_string();
    if !words.is_empty() {
        return words;
    }
    return match output.code {
        Some(code) => format!("git update-ref ended with {code}"),
        None => String::from("git update-ref ended with no exit status"),
    };
}
