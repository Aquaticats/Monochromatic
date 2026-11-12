//! What:
//!  The Git steps of a landing as faces a task of the runtime may await:
//! one step's child awaited on the runtime,
//! the side reference a withdrawn landing keeps its commit on,
//! and the words that tell the person where that commit is.
//! Why:
//!  A task cannot call the synchronous runner:
//! a step that may be ended at its ceiling registers its pipes by entering the runtime
//! (`step_output_read_whole_through_pipes.rs`,
//! `attach_pipes`),
//! and entering the runtime from inside it is refused,
//! so the synchronous face of such a step answers `Unavailable` on a task.
//! The steps themselves are `service_land_git_steps_and_import_stream.rs`'s,
//! one implementation each with the synchronous faces:
//! the sentence of a step that was ended,
//! the refusal Git gave,
//! and the line that names a kept commit are all read from there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = await runAwaiting(git, repo, args, input); const line = await keepUnlandedAwaiting(git, repo, seq, tip);
//! ```

use crate::service_land_git_steps_and_import_stream::{ASideRefWrite, SIDE_REF_WRITE_ATTEMPTS, refused_words_of, the_failure_of_a_step, the_line_of_an_unlanded_commit};
use crate::service_protocol::FailureCode;
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, value_of_awaiting};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, NO_REFLOG_SETTING};
use std::path::Path;

/// What:
///  `keep_unlanded` for a caller that is a task of the runtime:
/// the same side reference and the same line,
/// with the write and the look at the reference awaited.
/// Why:
///  A withdrawn landing keeps the commit it built before it gives its scratch reference up,
/// and a task cannot start that Git (`transaction_git_awaiting.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = await keepUnlandedAwaiting(git, repository, seq, tip)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio-landing-batch-body-storage-steps-awaited.md)"))]
pub(crate) async fn keep_unlanded_awaiting(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> String {
    return the_line_of_an_unlanded_commit(seq, keep_on_a_side_ref_awaiting(git, repository, seq, tip).await.is_none());
}

/// What:
///  `keep_on_a_side_ref` for a caller that is a task of the runtime.
/// Why:
///   One place names the reference a commit is kept on,
/// and both faces write it there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = await keepOnASideRefAwaiting(git, repo, seq, tip)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio-landing-batch-body-storage-steps-awaited.md)"))]
pub(crate) async fn keep_on_a_side_ref_awaiting(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> Option<String> {
    let name: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq);
    return write_until_the_reference_names_awaiting(git, repository, name.as_str(), tip, false).await.refusal();
}

/// What:
///  `write_until_the_reference_names` with its Git awaited on the runtime,
///  for the same reason:
/// a write a wait ended is answered by the reference itself.
/// Why:
///  The two faces must read an ending that gave no answer the same way,
/// or one of them tells a commit that was made that it was not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let attempt = 0; attempt < 2; attempt++) { const made = await runAwaiting(...); if (made.succeeded) return 'written'; if (await valueOfAwaiting(name) === tip) return 'written'; ... }
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio-landing-batch-body-storage-steps-awaited.md)"))]
async fn write_until_the_reference_names_awaiting(git: &GitContext, repository: &Path, name: &str, tip: &str, only_if_free: bool) -> ASideRefWrite {
    let mut last_words: String = String::from("the write of the reference gave no answer");
    let mut ended: Option<FailureCode> = None;
    for _attempt in 0..SIDE_REF_WRITE_ATTEMPTS {
        let mut arguments: Vec<&str> = vec!["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", name, tip];
        if only_if_free {
            arguments.push("");
        }
        let made: Result<GitOutput, FailureCode> = run_awaiting(git, repository, arguments.as_slice(), None).await;
        if matches!(&made, Ok(output) if output.succeeded()) {
            return ASideRefWrite::Written;
        }
        if matches!(value_of_awaiting(git, repository, name).await, RefValue::Names(named) if named == tip) {
            return ASideRefWrite::Written;
        }
        match made {
            Ok(output) => return ASideRefWrite::Refused(refused_words_of(&output)),
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
///  `run` for a caller that is a task of the runtime:
/// the same step,
/// with its child awaited on the runtime.
/// Why:
///  A task cannot call the synchronous runner,
/// which enters the runtime to wait for its child
/// (`transaction_git_awaiting.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runAwaiting(git, repository, args, input)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio-landing-batch-body-storage-steps-awaited.md)"))]
pub(crate) async fn run_awaiting(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    let words: Vec<std::ffi::OsString> = arguments.iter().map(std::ffi::OsString::from).collect();
    return run_words_awaiting(git, repository, words.as_slice(), input).await;
}

/// What:
///  `run_words` for a caller that is a task of the runtime,
/// answering what ended a step the same way the synchronous face does.
/// Why:
///  The sentence of a step ended at its ceiling is kept for the clients' answer,
/// and a stop of the service is told apart from every other failure by both faces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runWordsAwaiting(git, repository, words, input)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio-landing-batch-body-storage-steps-awaited.md)"))]
pub(crate) async fn run_words_awaiting(git: &GitContext, repository: &Path, arguments: &[std::ffi::OsString], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    return match crate::transaction_git_awaiting::run_git_awaiting(git, &request).await {
        Ok(output) => Ok(output),
        Err(error) => Err(the_failure_of_a_step(&error)),
    };
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
/// Controls for the awaited Git steps of a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_git_steps_awaited_tests.rs"]
mod tests;
