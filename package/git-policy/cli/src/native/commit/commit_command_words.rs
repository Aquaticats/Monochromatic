//! What:
//!  The words one commit command says when it stops:
//!  a message,
//!  a refused form,
//!  or a finding with its exit status,
//!  each handed to the thread that writes standard error.
//! Why:
//!  The exit status already carries the result,
//!  and a note of the commit is a line a person can live without,
//!  so a reader that stopped draining costs the command the hand-over's bound and no more
//!  (`standard_error_off_the_calling_thread.rs`,
//!  issue 6421).
//!  A refusal Git printed is bytes,
//!  and a path or a message the person typed may hold a byte that is not UTF-8,
//!  so decoding it would change what they see.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.stderr.write(text); // handed to the thread that writes it, waited for at most one bound
//! ```

/// How the command ended,
///  which a stop names the route of.
use crate::commit_command_input::CommitEnding;
/// Why the commit did not land,
/// by the route that stopped it.
use crate::commit_stop_reason::reason_of_stop;
/// The record of where the commit spent its time,
///  which a stop states its reason in.
use crate::commit_timing::CommitTiming;
/// What a transaction stopped with.
use crate::commit_transaction::TransactionStop;
use crate::policy_events::{
    EngineFailureHint,
    PolicyEvent,
    render_policy_events,
};
use crate::policy_trigger::Trigger;
/// The sentences a refused request is rendered with.
use crate::service_messages::render;
/// Standard error as a stream whose whole lines a thread of their own writes,
/// and the writer that counts the time they took.
use crate::standard_error_off_the_calling_thread::{
    StandardErrorHand,
    StandardErrorOffTheCallingThread,
};
use crate::time_spent_outside_the_phases::MeasuredStandardError;
/// `Write` is the trait whose `write_all` and `flush` the counted stream answers.
use std::io::Write;

/// What:
///  Write text to standard error,
/// ignoring a closed stream,
///  and counting the wait for the thread that writes it.
/// Why:
///   The exit status already carries the result,
///  and a note of the commit is a line a person can live without,
///  so a reader that stopped draining costs the command the hand-over's bound and no more
///  (`standard_error_off_the_calling_thread.rs`,
/// issue 6421).
///  The write is still the one place a slow reader becomes time the commit spends,
///  which the timing record states as `stderrMs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stderr.write(text); // handed to the thread that writes it, waited for at most one bound
/// ```
pub(crate) fn write_stderr(text: &str) {
    let mut stream: MeasuredStandardError<StandardErrorOffTheCallingThread<StandardErrorHand>> =
        MeasuredStandardError::over(StandardErrorOffTheCallingThread::new());
    let _ = stream.write_all(text.as_bytes());
    let _ = stream.flush();
}

/// What:
///  Write bytes to standard error,
/// ignoring a closed stream.
/// Why:
///   A refusal Git printed is bytes,
///  and a path or a message the person typed may hold a byte that is not UTF-8;
///  decoding it would change what they see.
///  The bytes go to the thread that writes standard error,
///  so a reader that stopped draining holds nothing of this command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stderr.write(bytes); // handed to the thread that writes it
/// ```
pub(crate) fn write_stderr_bytes(bytes: &[u8]) {
    let mut stream: MeasuredStandardError<StandardErrorOffTheCallingThread<StandardErrorHand>> =
        MeasuredStandardError::over(StandardErrorOffTheCallingThread::new());
    let _ = stream.write_all(bytes);
    let _ = stream.flush();
}

/// What:
///  Print what the transaction stopped with and name the exit status.
/// Why:
///   Each stop ends differently:
/// Git's own status with nothing printed,
/// a message with status 1,
/// or a finding with
///   status 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// catch (error) { ... process.exitCode = ... }
/// ```
pub(crate) fn report_stop(
    stop: TransactionStop,
    timing: &mut CommitTiming,
) -> CommitEnding {
    // Why the command stopped is written down here, before the process is told how it ended, so a record of a commit that
    //  did not land names the route that stopped it and not only the status it returned (`commit_stop_reason.rs`).
    timing.stop = Some(reason_of_stop(
        &stop,
        crate::git_step_failure::drain_failed_step(),
    ));
    match stop {
        TransactionStop::NativeExit(code) => {
            return CommitEnding {
                code,
                signal: None,
            };
        }
        TransactionStop::Message(message) => {
            write_stderr(format!("{message}\n").as_str());
            return CommitEnding {
                code: 1,
                signal: None,
            };
        }
        // A stop the command decided itself: the words the failure renders are what the person reads, and the failure is what
        //  the record named (`commit_stop_reason.rs`), so both ends of the command carry the same cause.
        TransactionStop::Client(failure) => {
            write_stderr(format!("{}\n", render(&failure)).as_str());
            return CommitEnding {
                code: 1,
                signal: None,
            };
        }
        // The person stopped this command with a signal, and the commit it waited for is not made: the words say so, and the
        //  command ends as that signal ends a program, which is what a shell reports for an interrupted native Git (`128 + the
        //  signal`). The returned status is reached only when the signal cannot end this process.
        // What: The person is told in one line, and the command ends by that signal once the record is written.
        // Why:  The record is the only account of a stopped commit that outlives the terminal;
        //       a raise here ended the process before `run_commit_command` wrote it, so an interrupted commit left none.
        TransactionStop::EndedBySignal {
            signal,
            message,
            ..
        } => {
            write_stderr(format!("{message}\n").as_str());
            return CommitEnding {
                code: crate::forwarding::SIGNAL_EXIT_BASE + signal,
                signal: Some(signal),
            };
        }
        TransactionStop::Failed {
            code,
            message,
        } => {
            let event: PolicyEvent = PolicyEvent::EngineFailure {
                code,
                hint: EngineFailureHint::OfCode,
                message,
                trigger: Some(Trigger::PreForward),
                policy: None,
                path: None,
            };
            write_stderr(render_policy_events(0, std::slice::from_ref(&event)).as_str());
            return CommitEnding {
                code: 2,
                signal: None,
            };
        }
    }
}
