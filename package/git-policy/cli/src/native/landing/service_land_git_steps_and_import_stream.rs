//! What:
//!  The Git steps of a landing and the text it feeds Git:
//! running one step and telling a step ended at its ceiling from other
//!  failures,
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
//! `requests_that_land`),
//! and the awaited one a task of the runtime asks,
//! which awaits every
//!  child on the runtime.
//!  A landing's own side references,
//! and every write of one,
//! are in a file of their own
//!  (`service_land_side_reference_write.rs`),
//! so the write a step and a removal share has one home.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runAwaiting(git, repository, args, input); importStream(scratch, old, batch, amendedParents); commitsFromMarks(text, count)
//! ```

use crate::service_land_follow::classify;
use crate::service_land_side_reference_write::{StepAnswer, answering, answering_awaiting, keep_an_unlanded_commit, take_ceiling_step};
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::transaction_git::{GitContext, GitOutput, GitRequest};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

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
    return answered_step(answering(git, words_request(repository, arguments, input)));
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
    return answered_step(answering_awaiting(git, words_request(repository, arguments, input)).await);
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
/// so a step the system refused is the same answer for both of them,
/// and what the answer holds is read here for each face
///  (`answering`,
///  `answering_awaiting`).
///  A step whose wait ended is left running,
/// and nothing that reads this answer can tell:
///  `Unavailable` is the code for a step the landing cannot resolve,
///  and a write that asks the reference itself reads the answer whole
///  (`service_land_side_reference_write.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answeredStep = (answer) => answer.answered ? answer.output : { ended: answer.ended ?? 'unavailable' }
/// ```
fn answered_step(answer: StepAnswer) -> Result<GitOutput, FailureCode> {
    return match answer {
        StepAnswer::Answered(output) => Ok(output),
        StepAnswer::LeftRunning => Err(FailureCode::Unavailable),
        StepAnswer::Ended(code) => Err(code),
    };
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
///  before the landing gives its scratch branch up,
///  letting go of that branch only once another reference names the commit.
/// Why:
///   A landing that ends at a ceiling lands nothing,
/// so the commit its import built is on the scratch branch
///  and no intent will finish it;
/// the marker names why the commit was not made,
/// and the message names the ref the commit is kept
///  at.
///  A keep that could not be written leaves the scratch reference where it is,
///  and the person is told the commit's name and that reference:
///  a reference that is the only name of a built commit is removed only after another name is written
///  (`keep_an_unlanded_commit`).
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
    let Ok(tip) = checked(&git, first.repository.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()], None) else { return };
    keep_an_unlanded_commit(&git, first.repository.as_path(), seq, scratch.as_str(), tip.as_str(), announce);
}

/// Controls for the awaited Git steps of a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_git_steps_awaiting_tests.rs"]
mod awaiting_tests;
