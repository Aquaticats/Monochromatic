//! What:
//!  The Git steps of a landing and the text it feeds Git:
//! running one step and telling a step ended at its ceiling from other
//!  failures,
//! keeping an unlanded commit on a side ref,
//! the `fast-import` stream of a batch and the marks it exports,
//! and the
//!  tests of which requests of a batch touch the same paths.
//! Why:
//!  `service_land.rs` holds the landing itself;
//! these steps depend on nothing in it,
//! so they are a file it uses.
//! Each Git step has two faces:
//! the synchronous one a caller on a thread asks (`run`,
//! `checked`,
//! `checked_bytes`,
//! `requests_that_land`,
//! `keep_on_the_scratch_reference`),
//! and the awaited one a task of the runtime asks,
//! which awaits every
//!  child on the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runAwaiting(git, repository, args, input); importStream(scratch, old, batch, amendedParents); commitsFromMarks(text, count)
//! ```

use crate::landing_state_carried_by_its_task::{LandingState, on_the_landing_state};
use crate::service_land_follow::classify;
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, value_of, value_of_awaiting};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, NO_REFLOG_SETTING, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// How many times a side reference is written when the wait for a write ended without an answer.
///  Two:
///  a write a wait left running is often finished moments later,
///  so one write more after the reference is read costs a step and saves the commit.
///  A write Git itself refused is written no second time:
///  the same value at the same name is refused the same way.
///  Unmeasured:
///  the count is the smallest that lets one wait which ended be finished by a read and one write after it,
///  and another attempt would multiply the wait of a write the disk makes slow on every try.
const SIDE_REF_WRITE_ATTEMPTS: usize = 2;

/// What:
///  Writes the side reference `name` at `tip`,
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
///  `only_if_free` writes the reference only while its name names nothing,
///  and a name it already holds at `tip` is the answer either way.
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
/// for (let attempt = 0; attempt < 2; attempt++) { const made = await writeRef(name, tip, freeOnly); if (made.succeeded) return 'written'; if (await valueOf(name) === tip) return 'written'; if (made.answered) return { refused: wordsOf(made) }; ended = made.code } return ended ? { ended } : { refused: words }
/// ```
fn write_until_the_reference_names(git: &GitContext, repository: &Path, name: &str, tip: &str, only_if_free: bool) -> ASideRefWrite {
    let mut last_words: String = String::from("the write of the reference gave no answer");
    let mut ended: Option<FailureCode> = None;
    for _attempt in 0..SIDE_REF_WRITE_ATTEMPTS {
        let mut arguments: Vec<&str> = vec!["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", name, tip];
        if only_if_free {
            // The empty old value is Git's way to refuse when the name exists at all, so a name another command wrote is never
            //  replaced (`keep_on_a_side_ref_only_if_free`).
            arguments.push("");
        }
        let made: Result<GitOutput, FailureCode> = run(git, repository, arguments.as_slice(), None);
        if matches!(&made, Ok(output) if output.succeeded()) {
            return ASideRefWrite::Written;
        }
        // The reference says whether a write finished, whatever the wait for it said. What the caller writes is unchanged: a name
        //  that cannot be read is written again, and a name another command wrote is the caller's own decision as it always was
        //  (`keep_on_a_side_ref_only_if_free` refuses one that exists, and both other callers write the value they were given).
        if matches!(value_of(git, repository, name), RefValue::Names(named) if named == tip) {
            return ASideRefWrite::Written;
        }
        match made {
            // Git answered, so the same value at the same name is refused the same way.
            Ok(output) => return ASideRefWrite::Refused(refused_words_of(&output)),
            // The wait ended without an answer: the step is left running, and the reference did not show its write, so it is written
            //  once more. A stop or a ceiling keeps its code, which the landing answers with.
            Err(code) => {
                last_words = format!("Git could not be run ({code:?})");
                ended = Some(code);
            }
        }
    }
    return match ended {
        Some(code) => ASideRefWrite::Ended(code),
        None => ASideRefWrite::Refused(last_words),
    };
}

/// What:
///  `write_until_the_reference_names` with every Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and a side reference is written from the landing once it is a task;
/// the two attempts,
/// the read of the reference after a write that gave no answer and the three answers a caller tells
///  apart are the synchronous face's,
/// so a commit one face keeps is kept by the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let attempt = 0; attempt < 2; attempt++) { const made = await writeRefAwaiting(...); if (made.succeeded) return 'written'; if (await valueOfAwaiting(name) === tip) return 'written'; if (made.answered) return { refused: wordsOf(made) }; ended = made.code } return ended ? { ended } : { refused: words }
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
async fn write_until_the_reference_names_awaiting(git: &GitContext, repository: &Path, name: &str, tip: &str, only_if_free: bool) -> ASideRefWrite {
    let mut last_words: String = String::from("the write of the reference gave no answer");
    let mut ended: Option<FailureCode> = None;
    for _attempt in 0..SIDE_REF_WRITE_ATTEMPTS {
        let mut arguments: Vec<&str> = vec!["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", name, tip];
        if only_if_free {
            // The empty old value is Git's way to refuse when the name exists at all, so a name another command wrote is never
            //  replaced (`keep_on_a_side_ref_only_if_free`).
            arguments.push("");
        }
        let made: Result<GitOutput, FailureCode> = run_awaiting(git, repository, arguments.as_slice(), None).await;
        if matches!(&made, Ok(output) if output.succeeded()) {
            return ASideRefWrite::Written;
        }
        // The reference says whether a write finished, whatever the wait for it said. What the caller writes is unchanged: a name
        //  that cannot be read is written again, and a name another command wrote is the caller's own decision as it always was
        //  (`keep_on_a_side_ref_only_if_free` refuses one that exists, and both other callers write the value they were given).
        if matches!(value_of_awaiting(git, repository, name).await, RefValue::Names(named) if named == tip) {
            return ASideRefWrite::Written;
        }
        match made {
            // Git answered, so the same value at the same name is refused the same way.
            Ok(output) => return ASideRefWrite::Refused(refused_words_of(&output)),
            // The wait ended without an answer: the step is left running, and the reference did not show its write, so it is written
            //  once more. A stop or a ceiling keeps its code, which the landing answers with.
            Err(code) => {
                last_words = format!("Git could not be run ({code:?})");
                ended = Some(code);
            }
        }
    }
    return match ended {
        Some(code) => ASideRefWrite::Ended(code),
        None => ASideRefWrite::Refused(last_words),
    };
}

/// What became of one write of one of the wrapper's own side references.
/// Why:
///  The write answers three different things,
/// and a caller that keeps a commit or answers a commit tells them apart:
///  a reference that names the commit,
///  a write Git refused,
///  and a step that was ended or could not be run,
///  whose code is the landing's own answer for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ASideRefWrite = 'written' | { refused: string } | { ended: FailureCode }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ASideRefWrite {
    /// The reference names the object the write was for.
    Written,
    /// Git answered that it refused the write,
    /// in its own words.
    Refused(String),
    /// The write was ended or could not be run,
    /// and the code says what ended it.
    Ended(FailureCode),
}

impl ASideRefWrite {
    /// What the person is told when the write did not put the reference where it was wanted,
    /// or nothing when it did.
    /// Why:
    ///  The three answers are one sentence for a caller that only says where a commit was kept;
    ///  the walk that decides a commit keeps the codes instead (`service_land_objects_of_batch.rs`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const refusal = (write: ASideRefWrite) => write === 'written' ? undefined : write.refused ?? `Git could not be run (${write.ended})`
    /// ```
    pub(crate) fn refusal(&self) -> Option<String> {
        return match self {
            Self::Written => None,
            Self::Refused(words) => Some(words.clone()),
            Self::Ended(code) => Some(format!("Git could not be run ({code:?})")),
        };
    }
}

/// What:
///  Writes `refs/cli-git/kept/<number>` at `tip`,
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
/// expect(await keepOnASideRef(git, repo, seq, tip)).toBeUndefined()
/// ```
pub(crate) fn keep_on_a_side_ref(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> Option<String> {
    let name: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq);
    return write_until_the_reference_names(git, repository, name.as_str(), tip, false).refusal();
}

/// What:
///  `keep_on_a_side_ref` with every Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and a commit a landing could not put on a branch is kept on its
///  side reference once the landing is a task;
/// the name,
/// the write and the words a refusal gives are the synchronous face's,
/// so a commit is kept at one name either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = await keepOnASideRefAwaiting(git, repo, seq, tip);
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn keep_on_a_side_ref_awaiting(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> Option<String> {
    let name: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq);
    return write_until_the_reference_names_awaiting(git, repository, name.as_str(), tip, false).await.refusal();
}

/// What:
///  Writes the per-worktree reference `refs/worktree/cli-git-service/<seq>` at `tip`,
///  the reference that keeps the objects of a batch reachable until its branch write,
///  and answers nothing when the reference names it or Git's own words when it does not.
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
    return write_until_the_reference_names(git, repository, scratch, tip, false);
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
    return write_until_the_reference_names_awaiting(git, repository, scratch, tip, false).await;
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
///  whether this command read it first or another one wrote it in between.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnASideRefOnlyIfFree(git, repo, name, tip)).toBeUndefined()
/// ```
pub(crate) fn keep_on_a_side_ref_only_if_free(git: &GitContext, repository: &Path, name: &str, tip: &str) -> Option<String> {
    return write_until_the_reference_names(git, repository, name, tip, true).refusal();
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
    let name: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq);
    if keep_on_a_side_ref(git, repository, seq, tip).is_some() {
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
    let name: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq);
    if keep_on_a_side_ref_awaiting(git, repository, seq, tip).await.is_some() {
        return String::from("monochromatic-git-policy-cli: the commit built for you could not be kept on a side ref; your staged changes and your worktree are as they were, so run the commit again when the program above is done.\n");
    }
    return format!(
        "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
    );
}

pub(crate) fn run<S: AsRef<std::ffi::OsStr>>(git: &GitContext, repository: &Path, arguments: &[S], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    return run_words(git, repository, owned_words(arguments).as_slice(), input);
}

/// What:
///  One command line as the owned words a byte-native start takes.
/// Why:
///   The words may hold bytes that are not text,
/// so each one is taken as the operating system has it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ownedWords = (arguments) => arguments.map((word) => Buffer.from(word))
/// ```
fn owned_words<S: AsRef<std::ffi::OsStr>>(arguments: &[S]) -> Vec<std::ffi::OsString> {
    let mut words: Vec<std::ffi::OsString> = Vec::with_capacity(arguments.len());
    for argument in arguments {
        words.push(argument.as_ref().to_os_string());
    }
    return words;
}

/// What:
///  `run` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and every step of a landing runs from
///  the landing once that landing is a task.
/// The question and the failure a step stands for are the synchronous face's,
/// so the
///  two cannot run another command or answer it differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runAwaiting(git, repository, args, input);
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn run_awaiting(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    let words: Vec<std::ffi::OsString> = arguments.iter().map(std::ffi::OsString::from).collect();
    return run_words_awaiting(git, repository, words.as_slice(), input).await;
}

/// What:
///  The failure one Git step of a landing stands for.
/// Why:
///  A step ended at its ceiling,
/// or by the service's stop,
/// is told apart from every other failure,
/// and its
///  sentence is kept for the answer;
/// the commit it belongs to is kept on a side ref and not made
///  (`land_batch_timed`).
/// Both faces read the failure through this one function,
/// so neither can answer a step Git could
///  not run with another code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failureOf(error) { if (isCeiling(error) || isEndedByStop(error)) { onTheLandingState(error.message, keepTheCeilingStep); return 'stepTooSlow'; } return 'unavailable'; }
/// ```
fn failure_of(error: std::io::Error) -> FailureCode {
    if crate::child_watch::is_ceiling(&error) || crate::child_watch::is_ended_by_stop(&error) {
        on_the_landing_state(error.to_string(), keep_the_ceiling_step);
        return FailureCode::StepTooSlow;
    }
    return FailureCode::Unavailable;
}

/// What:
///  `run` for a command line that is not all UTF-8,
///  which the reflog message of a commit whose message is not UTF-8 needs.
/// Why:
///   The message is one argument of `git update-ref -m`,
/// so its bytes reach Git as they are and no text conversion can change them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(['update-ref', '-m', message, '--stdin'], { input: transaction })
/// ```
pub(crate) fn run_words(git: &GitContext, repository: &Path, arguments: &[std::ffi::OsString], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    return run_words_answered(run_git(git, &words_request(repository, arguments, input)));
}

/// What:
///  `run_words` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the branch move of a landing is a step of that\n///  landing once it is a task;
/// the message of a commit whose text is not UTF-8 reaches Git as its own bytes on
///  both faces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runWordsAwaiting(git, repository, arguments, input);
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn run_words_awaiting(
    git: &GitContext,
    repository: &Path,
    arguments: &[std::ffi::OsString],
    input: Option<Vec<u8>>,
) -> Result<GitOutput, FailureCode> {
    return run_words_answered(run_git_awaiting(git, &words_request(repository, arguments, input)).await);
}

/// The question both faces of `run_words` ask Git.
fn words_request(repository: &Path, arguments: &[std::ffi::OsString], input: Option<Vec<u8>>) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    return request;
}

/// What:
///  Git's output of one step,
///  and the failure a step that could not run stands for.
/// Why:
///  Both faces read the run through this one function,
/// so a step the system refused is the same answer for both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runWordsAnswered = (answered) => answered.error ? failureOf(answered.error) : answered;
/// ```
fn run_words_answered(answered: std::io::Result<GitOutput>) -> Result<GitOutput, FailureCode> {
    return match answered {
        Ok(output) => Ok(output),
        Err(error) => Err(failure_of(error)),
    };
}

/// Keeps the sentence of a step that was ended at its ceiling.
fn keep_the_ceiling_step(state: &LandingState, sentence: String) {
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

pub(crate) fn checked<S: AsRef<std::ffi::OsStr>>(git: &GitContext, repository: &Path, arguments: &[S], input: Option<Vec<u8>>) -> Result<String, FailureCode> {
    return checked_text(run(git, repository, arguments, input)?);
}

/// What:
///  `checked` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the landing asks its questions from one
///  once it is a task.
/// The step and the reading of its answer are the synchronous face's,
/// so the two cannot
///  answer different text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = await checkedAwaiting(git, repository, args, input);
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn checked_awaiting(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<String, FailureCode> {
    return checked_text(run_awaiting(git, repository, arguments, input).await?);
}

/// What:
///  The text a step that must succeed answered,
///  and the failure code of one that did not.
/// Why:
///  Both faces read a step's answer through this one function,
/// so a step Git refused is classified the same whichever
///  face ran it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checkedText = (output) => output.succeeded ? output.out.trim() : classify(output);
/// ```
fn checked_text(output: GitOutput) -> Result<String, FailureCode> {
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  The standard output of one Git step that must succeed,
/// as the bytes Git wrote.
/// Why:
///   `checked` answers with trimmed text,
/// which is right for an object name and wrong for a listing of paths:
/// a listing with a name that is not text has to reach the caller as the name Git wrote.
pub(crate) fn checked_bytes<S: AsRef<std::ffi::OsStr>>(git: &GitContext, repository: &Path, arguments: &[S], input: Option<Vec<u8>>) -> Result<Vec<u8>, FailureCode> {
    return checked_stdout(run(git, repository, arguments, input)?);
}

/// What:
///  `checked_bytes` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the listing a batch's paths are read from is asked
///  from the landing once it is a task;
/// the listing reaches the caller as the bytes Git wrote on both faces,
/// so
///  a name that is not text survives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listing = await checkedBytesAwaiting(git, repository, args, input);
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn checked_bytes_awaiting(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<Vec<u8>, FailureCode> {
    return checked_stdout(run_awaiting(git, repository, arguments, input).await?);
}

/// What:
///  The bytes a step that must succeed wrote,
///  and the failure code of one that did not.
/// Why:
///  Both faces read a listing through this one function,
/// so a step Git refused classifies the same way whichever face ran it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checkedStdout = (output) => output.succeeded ? output.out : classify(output);
/// ```
fn checked_stdout(output: GitOutput) -> Result<Vec<u8>, FailureCode> {
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(output.stdout);
}

/// The `fast-import` stream of a batch and the tests of which requests touch the same paths,
/// kept in a file of their own to stay under the file-size limit.
#[path = "service_land_batch_import_stream_and_overlaps.rs"]
mod batch_text;
pub(crate) use self::batch_text::{commits_from_marks, ids_of, import_stream};
/// The forms a control of the batch's paths reads the stream and the comparisons by.
#[cfg(test)]
pub(crate) use self::batch_text::{overlaps, quoted};
use self::batch_text::{ends_a_field, landing_of};

/// What:
///  The command each request of a batch says will decide its commit's post-commit checks,
///  in the order of the commits.
/// Why:
///   The intent of a batch is the only record a restart has of the command that will promote the landed commits'
///  held pushes,
/// and a hold that names no command says a published commit may still be held (`held_push_holder_and_verdict.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// batch.map((request) => request.holder)
/// ```
pub(crate) fn holders_of(batch: &[&CommitRequest]) -> Vec<String> {
    let mut holders: Vec<String> = Vec::new();
    for request in batch {
        holders.push(request.holder.clone());
    }
    return holders;
}


/// What:
///  The requests of a batch that land against the tip `old`,
/// with their positions:
/// a request whose paths changed on the branch
///  since its base is left out,
/// and so is one that an earlier request of the batch overlaps unless it captured the worktree later.
/// Why:
///  The requests left out are sent back to be prepared again and merged against what landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// requests.map((r, i) => [i, r]).filter(([, r]) => !conflictsSince(r, old) && !shadowedByEarlier(r))
/// ```
pub(crate) fn requests_that_land<'a>(git: &GitContext, repository: &Path, requests: &'a [CommitRequest], old: &str) -> Result<Vec<(usize, &'a CommitRequest)>, FailureCode> {
    let mut changed_since: HashMap<&str, BTreeSet<Vec<u8>>> = HashMap::new();
    for request in requests {
        if request.base != old && !changed_since.contains_key(request.base.as_str()) {
            let listing: Vec<u8> = checked_bytes(git, repository, &names_since_request(request.base.as_str(), old), None)?;
            changed_since.insert(request.base.as_str(), names_of(listing.as_slice()));
        }
    }
    return Ok(landing_of(requests, old, &changed_since));
}

/// What:
///  `requests_that_land` with every listing awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the requests one batch lands are chosen from the landing
///  once it is a task.
/// The questions,
/// the fold and the answers are the synchronous face's,
/// so the two cannot land another
///  set of requests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landing = await requestsThatLandAwaiting(git, repository, requests, old);
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn requests_that_land_awaiting<'a>(
    git: &GitContext,
    repository: &Path,
    requests: &'a [CommitRequest],
    old: &str,
) -> Result<Vec<(usize, &'a CommitRequest)>, FailureCode> {
    let mut changed_since: HashMap<&str, BTreeSet<Vec<u8>>> = HashMap::new();
    for request in requests {
        if request.base != old && !changed_since.contains_key(request.base.as_str()) {
            let listing: Vec<u8> = checked_bytes_awaiting(git, repository, &names_since_request(request.base.as_str(), old), None).await?;
            changed_since.insert(request.base.as_str(), names_of(listing.as_slice()));
        }
    }
    return Ok(landing_of(requests, old, &changed_since));
}

/// The question that lists the paths a request's base changed since `old`.
fn names_since_request<'a>(base: &'a str, old: &'a str) -> Vec<&'a str> {
    return vec!["diff-tree", "-r", "--name-only", "-z", base, old];
}

/// The paths one listing of Git's `-z` output names.
fn names_of(listing: &[u8]) -> BTreeSet<Vec<u8>> {
    let mut names: BTreeSet<Vec<u8>> = BTreeSet::new();
    for name in listing.split(ends_a_field) {
        if !name.is_empty() {
            names.insert(name.to_vec());
        }
    }
    return names;
}

/// What:
///  Tells the clients of a batch which step was ended at its ceiling,
/// and keeps the commit the batch built on a side ref
///  before the landing gives its scratch branch up.
/// Why:
///   A landing that ends at a ceiling lands nothing,
/// so the commit its import built is on the scratch branch
///  and no intent will finish it;
/// the marker names why the commit was not made,
/// and the message names the ref the commit is kept
///  at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepWhatAnEndedStepBuilt(context, announce, requests, seq)
/// ```
pub fn keep_what_an_ended_step_built(context: &crate::service_land_context::LandContext, announce: &dyn crate::service_index_lock::Announcer, requests: &[CommitRequest], seq: u64) {
    let Some(step) = take_ceiling_step() else { return };
    let Some(first) = requests.first() else { return };
    announce.announce(crate::service_index_lock::Announce::Waiting(format!("monochromatic-git-policy-cli: {step}, so this commit was not made.\n")));
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    let scratch: String = crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(seq);
    if let Ok(tip) = checked(&git, first.repository.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()], None) {
        announce.announce(crate::service_index_lock::Announce::Waiting(keep_unlanded(&git, first.repository.as_path(), seq, tip.as_str())));
        let _deleted = run(&git, first.repository.as_path(), &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch.as_str()], None);
    }
}

/// Controls for the awaited Git steps of a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_git_steps_awaiting_tests.rs"]
mod awaiting_tests;
