//! What:
//!  How long a push may show nothing before it is stopped:
//! the environment variable that sets the deadline,
//! its default,
//!  the reading of a positive whole number of seconds,
//! and the patience a push runs under.
//! Why:
//!  `bounded_push.rs` runs and stops the push;
//! the settings it reads do not use it,
//! so they sit in a file of their own and
//!  `bounded_push.rs` re-exports them for the callers that import them from there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pushDeadline = (environment) => secondsVariable(environment, 'CLI_GIT_PUSH_DEADLINE_SECONDS') ?? 30_000;
//! ```

use crate::diagnostic_log::debug;
use std::ffi::{OsStr, OsString};
use std::time::Duration;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// Environment variable that sets the push deadline in whole seconds.
pub const PUSH_DEADLINE_VARIABLE: &str = "CLI_GIT_PUSH_DEADLINE_SECONDS";

/// How long a push with no connection to a remote open may go without any sign of work (the stall window) when the variable is
///  absent or unusable.
/// Where no progress can be measured,
/// it is how long the push may run in all.
///  Citation:
/// no measurement or owner decision determines 30 seconds.
/// The owner decided the rule,
/// "Stop only on no progress
///  (Recommended)",
/// and not the number.
/// The journal of the commit service (read-only,
/// 794 pushes that landed since 2026-10-20) holds
///  whole-push times,
/// p50 3.5 s,
/// p99 26.4 s,
/// maximum 72.4 s,
/// which include connected time and so are not a measure of silence
///  without a connection;
/// no silence of a disconnected push is logged anywhere.
/// The number is an open question for the owner
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Push stall window,
/// 30 seconds").
pub const DEFAULT_PUSH_DEADLINE: Duration = Duration::from_secs(30);

/// The variable a control sets to shorten the window a run whose connection is open may stay silent for.
///  It is named as a control's because the window is fixed for every shipped run (`SPEC.md`, "Every wait has a
///  bound"),
/// and a control cannot wait the shipped 150 seconds.
pub const CONNECTED_WINDOW_VARIABLE: &str = "CLI_GIT_TEST_ONLY_CONNECTED_WINDOW_SECONDS";

/// What:
///  Read a positive whole number of seconds from one environment value.
/// Why:
///   A zero,
/// negative,
/// fractional,
/// empty or non-text value is no setting at all,
///  never "no limit",
///  so a typo cannot remove a bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seconds = Number(value); return Number.isInteger(seconds) && seconds > 0 ? seconds * 1000 : undefined;
/// ```
pub fn positive_seconds(name: &str, value: &OsStr) -> Option<Duration> {
    let parsed: Option<u64> = match value.to_str() {
        Some(text) => text.trim().parse::<u64>().ok(),
        None => None,
    };
    return match parsed {
        Some(count) if count > 0 => Some(Duration::from_secs(count)),
        Some(_) | None => {
            debug(
                TAG,
                &format!("{name} is not a positive whole number; ignoring it"),
            );
            None
        }
    };
}

/// What:
///  Read a positive whole number of seconds from the child environment.
/// Why:
///   The last of repeated entries wins,
///  as in an environment block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveSeconds(env[name])
/// ```
pub(crate) fn seconds_variable(environment: &[(OsString, OsString)], name: &str) -> Option<Duration> {
    let mut found: Option<&OsString> = None;
    for (key, value) in environment {
        if key.to_str() == Some(name) {
            found = Some(value);
        }
    }
    let value: &OsString = found?;
    return positive_seconds(name, value.as_os_str());
}

/// The variable that sets how long one command of the reconcile merge's second pass may run in all.
pub const MERGE_CEILING_VARIABLE: &str = "CLI_GIT_PUSH_MERGE_CEILING_SECONDS";

/// How long one command of a commit's merge step may run in all,
/// whatever it shows.
/// Citation:
/// measured on the development host on 2026-11-09,
/// under a load average of 66 to 75 on 8 CPUs,
/// with a merge of one 7 MiB file the word pass resolves:
/// the whole step
///  (`git merge-tree`,
/// the second pass and `git commit-tree`)
/// took 396 ms and 403 ms in two runs,
/// and the word merge alone was measured at 0.73 s for 8 MiB in `SPEC.md`.
/// The pass takes no file past 8 MiB,
/// so a merge of many such files is a few seconds of measured work,
/// and 300 seconds leaves room for a slow disk and a loaded host while still bounding a merge driver that
///  keeps working without ever returning.
/// The rule is the owner's,
/// "nothing should ever wait forever or indefinitely",
/// carried out for a step the push stall window cannot bound,
///  because a driver that rewrites a scratch file for ever always shows progress.
/// The owner is invited to veto the number;
/// it is settable for a repository that needs more.
pub const DEFAULT_MERGE_CEILING: Duration = Duration::from_secs(300);

/// What:
///  The ceiling of one command of a commit's merge step:
/// the environment's value,
/// or the default.
/// Why:
///   A merge of very large files on a slow disk may need longer than the default,
/// and a control needs one it can wait for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ceilingMs = secondsOf(env.CLI_GIT_PUSH_MERGE_CEILING_SECONDS) ?? DEFAULT_MS;
/// ```
pub fn merge_ceiling(environment: &[(OsString, OsString)]) -> Duration {
    return seconds_variable(environment, MERGE_CEILING_VARIABLE).unwrap_or(DEFAULT_MERGE_CEILING);
}

/// What:
///  The push deadline:
/// the environment's value,
/// or the default.
/// Why:
///   A person may need a longer deadline on a slow link or a shorter one on a flaky one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadlineMs = secondsOf(env.CLI_GIT_PUSH_DEADLINE_SECONDS) ?? DEFAULT_MS;
/// ```
pub fn push_deadline(environment: &[(OsString, OsString)]) -> Duration {
    return seconds_variable(environment, PUSH_DEADLINE_VARIABLE).unwrap_or(DEFAULT_PUSH_DEADLINE);
}

/// What:
///  The window a run whose connection to a remote is open may show no sign of work for.
/// Why:
///   The window is fixed for every shipped run,
/// and a control that must see it fire cannot wait 150 seconds,
/// so a control shortens it here for every run it starts.
/// A value that is not a positive whole number is no setting at all,
/// as every value of this kind is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// secondsOf(env.CLI_GIT_TEST_ONLY_CONNECTED_WINDOW_SECONDS) ?? 150_000
/// ```
pub fn connected_window(environment: &[(OsString, OsString)]) -> Duration {
    return seconds_variable(environment, CONNECTED_WINDOW_VARIABLE).unwrap_or(crate::wait_ceilings::CONNECTED_PUSH_QUIET_WINDOW);
}

/// What:
///  How long a bounded child may take:
/// a fixed time,
/// or a time without any sign of work and without an open connection.
/// Why:
///   A slow push that keeps moving data is working and should finish,
/// and one whose connection to a remote is open is waiting
///  for the remote and should wait too;
/// one that shows nothing and holds no connection is stuck and should stop fast.
/// A query has
///  no transfer to measure and keeps a fixed bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Patience = { kind: 'fixed'; ms: number } | { kind: 'progress'; stallMs: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Patience {
    /// Stop after this long,
    /// whatever the child does.
    Fixed(Duration),
    /// Stop after `stall` without data moved,
    /// processor time used,
    /// or a task at work,
    /// by the child and everything it started,
    ///  unless a process of them holds an open connection to a remote:
    /// that is waited for,
    /// however long it takes.
    Progress {
        /// How long the child may show nothing while no connection is open.
        stall: Duration,
    },
}

/// The patience of a push from the environment:
/// the stall window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ kind: 'progress', stallMs: pushDeadline(env) })
/// ```
pub fn push_patience(environment: &[(OsString, OsString)]) -> Patience {
    return Patience::Progress { stall: push_deadline(environment) };
}

/// Remove exactly one trailing `\n` or `\r\n`,
/// as the captured output of the incumbent does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// output.endsWith('\n') ? output.slice(0, output.endsWith('\r\n') ? -2 : -1) : output
/// ```
pub fn strip_one_line_break(text: &str) -> &str {
    let without_lf = match text.strip_suffix('\n') {
        Some(stripped) => stripped,
        None => return text,
    };
    return without_lf.strip_suffix('\r').unwrap_or(without_lf);
}
