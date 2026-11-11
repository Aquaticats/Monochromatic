//! What:
//!  The words a person reads about a Git LFS upload:
//!  the explanation after a failure,
//! a stop,
//! a refusal that kept the upload from starting,
//! and the reading that says a failure
//!  was a login nobody could be asked for.
//! Why:
//!  Both faces of the upload print them (the typed push's child with the caller's streams and the
//! bounded one the auto-push records),
//! so one file owns every sentence a person may act on,
//! and the two cannot explain the same
//!  end differently.
//! It stands beside `lfs_upload.rs` because the two together pass the package's length limit
//! (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lfs-upload-explanations.ts: failedMessage, stoppedWords, loginNeeded, captureFailed, guardFailed, explained
//! ```

/// The failure record of a push the wrapper ran.
use crate::auto_push_single_flight::PushAttempt;
/// The limit a stopped child was ended by,
/// and the status the wrapper's own stop carries.
use crate::bounded_push::{STOPPED_EXIT_CODE, StopCause};
/// The texts of a failed or refused upload,
/// and the reading that says a login was needed.
use crate::lfs_pre_push::{failed_message, login_needed_message, missing_program_message, reads_as_a_login_failure};
/// The masks and the hosts a destination or a failure names.
use crate::lfs_output_mask::{host_of, hosts_in, mask_userinfo_text};

/// The most standard error text one run keeps and relays;
/// the rest is dropped,
/// so a flood cannot fill memory.
pub const MAX_RELAYED_BYTES: u64 = 4 * 1024 * 1024;

/// What:
///  The words a run the wrapper stopped is explained with.
/// Why:
///  The program chose no status of its own,
/// so the explanation says which limit came first and what the person can do about a
///  slow line,
/// as the auto-push's upload does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cause.kind === 'stalled' ? `made no progress for ${limit} seconds and was stopped` : `...`
/// ```
pub fn stopped_upload_words(cause: &StopCause) -> String {
    return match cause {
        StopCause::Deadline => String::from("did not finish in time and was stopped"),
        StopCause::Stalled(report) => format!(
            "made no progress for {} seconds and was stopped (bytes_moved={}); {}",
            report.limit.as_secs(),
            report.moved,
            crate::push_deadline_advice::extend_clause()
        ),
        StopCause::Flood => format!("wrote more than {MAX_RELAYED_BYTES} bytes of output and was stopped"),
        // A typed upload carries no arm for the service's stop, so this ending cannot be reached here.
        StopCause::ServiceStopped => String::from("was ended because the service was stopped"),
    };
}

/// What:
///  How a refusal names the server whose upload needed a login:
///  the one the program's own words name,
/// the one the destination names,
/// or,
/// when neither does,
/// the destination as typed with any userinfo hidden.
/// Why:
///   The program's words name the server that asked,
/// which for a repository whose `lfs.url` is another host is not the push's
///  destination at all (the incident's push went to GitHub and its login question came from the Worker);
/// and no form of this may repeat a token that an `lfs.url` carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const host = hostsIn(output)[0] ?? hostOf(destination);
/// host ? `the server \`${host}\`` : `the destination \`${maskUserinfoText(destination)}\``
/// ```
fn login_subject(destination: &std::ffi::OsStr, output: &str) -> String {
    let from_program: Option<String> = hosts_in(output).into_iter().next();
    // `to_string_lossy` reads a destination that is not UTF-8 without failing; a remote name or URL is text in practice.
    let text: String = destination.to_string_lossy().into_owned();
    let from_destination: Option<String> = host_of(text.as_str());
    return match from_program.or(from_destination) {
        Some(host) => format!("the server `{host}`"),
        None => format!("the destination `{}`", mask_userinfo_text(text.as_str())),
    };
}

/// What:
///  The explanation printed after a failed upload:
/// the general one,
/// and,
/// when the program's own words say it needed a login,
/// the one that names the server and every way to give it one.
/// Why:
///   The wrapper is what refused to ask a person,
/// so that failure is the one whose way forward only the wrapper can explain
///  (`doc/troubleshooting/lfs-credential-in-url.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// failedMessage(status) + (readsAsALoginFailure(output) ? loginNeededMessage(loginSubject(destination)) : '')
/// ```
pub fn explanation_after_failure(status: &str, destination: &std::ffi::OsStr, output: &[u8]) -> String {
    let mut text: String = failed_message(status);
    // `from_utf8_lossy` reads the program's words whatever their encoding; only the search matters here.
    let shown: String = String::from_utf8_lossy(output).into_owned();
    if reads_as_a_login_failure(shown.as_str()) {
        text.push_str(login_needed_message(login_subject(destination, shown.as_str()).as_str()).as_str());
    }
    return text;
}

/// The refusal printed when the standard error of the upload cannot be captured.
pub fn capture_failed_message(why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: this push was stopped before it sent anything. The wrapper reads the output of `git-lfs push` through \
         a private temporary file, to hide the credentials in the URLs it names, and that file could not be made ({why}). Ways forward: fix the cause named in the parentheses (the \
         wrapper needs a writable `XDG_STATE_HOME`, or a `HOME`, for its private scratch directory) and push again; or \
         skip the upload for this push with `GIT_LFS_SKIP_PUSH=1` or `git push --no-verify`, which publishes pointer \
         files without their objects.\n"
    );
}

/// What:
///  The refusal printed when no signal handler could be installed for the upload's wait.
/// Why:
///  Without a guard the wait would have no bound and no relay,
/// so the push is stopped before it sends anything rather than left to wait
///  where nothing bounds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: this push was stopped before it sent anything. ... (${why}) ...`
/// ```
pub fn guard_failed_message(why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: this push was stopped before it sent anything. The wrapper bounds the run of `git-lfs push` \
         and relays a signal sent to it to that program, and it could not begin that wait ({why}). Ways forward: push again, because \
         the cause may be transient; or upload by hand with `git lfs push --all <remote>` and push with `git push --no-verify`.\n"
    );
}

/// What:
///  What a person is told when no `git-lfs` is on the caller's `PATH`.
/// Why:
///   Both faces refuse the upload with the same words,
/// and the push that would have followed it is never started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ exitCode: 1, output: missingProgramMessage().trimEnd() })
/// ```
pub fn missing_program_attempt() -> PushAttempt {
    return PushAttempt { exit_code: 1, output: missing_program_message().trim_end().to_owned() };
}

/// What:
///  The failed attempt of one finished upload,
/// with the credentials in its output hidden,
/// or nothing when it succeeded.
/// Why:
///  A server error is not run again here,
/// for the reason given in `run_upload_interactive`:
/// the service's push retry plan
///  tries the push again by itself.
/// Both faces read one finished attempt here,
/// so the two cannot report a failure differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// attempt.exitCode === 0 ? undefined : explained({ exitCode, output: maskUserinfoText(attempt.output) });
/// ```
pub fn failed_upload(attempt: &PushAttempt, destination: &std::ffi::OsStr) -> Option<PushAttempt> {
    if attempt.exit_code == 0 {
        return None;
    }
    let shown: String = mask_userinfo_text(attempt.output.as_str());
    return Some(explained(PushAttempt { exit_code: attempt.exit_code, output: shown }, destination));
}

/// The failed attempt with the explanation after the program's own output;
/// a stop by deadline says so,
/// and a login the program needed says how to give it one.
pub fn explained(attempt: PushAttempt, destination: &std::ffi::OsStr) -> PushAttempt {
    let description: String = if attempt.exit_code == STOPPED_EXIT_CODE {
        String::from("did not finish and was stopped")
    } else {
        format!("exited with status {}", attempt.exit_code)
    };
    let explanation: String = explanation_after_failure(description.as_str(), destination, attempt.output.as_bytes());
    let shown: &str = explanation.trim_end();
    let output: String = if attempt.output.is_empty() { shown.to_owned() } else { format!("{}\n{shown}", attempt.output) };
    return PushAttempt { exit_code: attempt.exit_code, output };
}
