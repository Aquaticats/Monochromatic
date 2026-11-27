//! What:
//! The step that ends a run's scope unit after the run has shown its control group empty:
//! the one last look at the scope,
//! and the one question to the manager that the look may call for.
//! Why:
//! Issue 6578:
//! a transient scope whose process was already exiting when the manager wrote its process id into
//! the scope's `cgroup.procs` is counted as attached without being moved
//! (`cg_attach` returns success for a task that is leaving).
//! The scope's control group therefore never becomes populated,
//! the kernel sends no empty notification for it,
//! and the unit stays `active (running)` with `populated 0` and no member.
//! A run cannot make the manager re-read that state,
//! but it can ask the manager to stop the unit it started:
//! the stop job ends the unit,
//! and `CollectMode=inactive-or-failed` (the `--collect` of the launcher)
//! then frees it.
//! The end of a run calls it after the stop of the containment,
//! in `bounded_run_end.rs`.
//!
//! The cost (2026-11-27,
//! the alpha.12 release pass):
//! a question to the manager is a program start,
//! and asking it on every run's end cost a scoped run 15 to 36 ms of the caller's time
//! (`doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md`,
//! "The cost of the retirement").
//! So the look answers the question from the kernel alone:
//! `pids.peak` says whether a process was ever counted in the scope's group,
//! and only a group that never held one can be the stuck scope this step exists for.
//! A group that held one changed its `populated` line,
//! which is the change the manager ends the unit on.
//! A group that is gone was already collected.
//! Neither asks the manager anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-retire.ts
//! export function retire(group: ControlGroup, systemctl: string, boundMs: number): string | undefined
//! export function retireUnit(systemctl: string, unit: string, boundMs: number): void  // throws the reason
//! ```

/// What:
/// `use crate::control_group::{ControlGroup, pids_peak_at, populated_at};` imports the scope the step acts on,
/// the kernel's count of how many processes it has held,
/// and the kernel's `populated` line.
/// Why:
/// The look needs the scope's own directory,
/// and the two reads are of that directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ControlGroup, pidsPeakAt, populatedAt } from 'crate/control_group';
/// ```
use crate::control_group::{
    ControlGroup,
    pids_peak_at,
    populated_at,
};
/// What:
/// `use crate::control_group_route_child_runs::run_asking;` imports the short program runner of the route discovery.
/// Why:
/// The one question the end of a run may ask the manager is the same kind of bounded question as the discovery asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runAsking } from 'crate/control_group_route_child_runs';
/// ```
use crate::control_group_route_child_runs::run_asking;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// The two reads of the scope answer an `io::Error`,
/// and the absence of the scope is one of those errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;
/// What:
/// `use std::time::Duration;` imports a span of time (its sibling `Instant` is a point on the clock).
/// Why:
/// The step that asks the manager gets one bound for the whole question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds
/// ```
use std::time::Duration;

/// How long the manager is given to answer the one stop request this step makes.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// measurement beside it:
/// on 2026-11-27 a `systemctl --user stop` of an empty `cargo-shadow-*.scope` answered in 3.8 ms
/// (median of twelve calls on this host,
/// systemd 262.2-1-arch),
/// so five seconds only ends a manager that does not answer.
/// One bound covers the whole step,
/// because the step asks once;
/// a killed request adds the reap grace of the short program runner (100 ms) and nothing else.
pub(crate) const RETIRE_BOUND: Duration = Duration::from_secs(5);

/// The exit status `systemctl` answers for a unit the manager does not load (`EXIT_NOTINSTALLED`,
/// `systemctl(1)` "Exit status").
/// Why:
/// A unit the manager has already collected is exactly what a successful retirement leaves behind,
/// so its stop request's refusal is the answer this step wants,
/// and no second question is needed to tell it from a manager that refused for another reason.
const UNIT_NOT_LOADED: i32 = 5;

/// What the run's last look at its scope says about the unit behind it.
#[derive(Debug)]
pub(crate) enum Emptiness {
    /// The kernel's `populated 0` line was read,
    /// and `pids.peak` says a process has been counted in the group:
    /// the kernel changed the `populated` line,
    /// so the manager's watch on it fired and it collects the unit.
    EmptiedAfterAProcess,
    /// The kernel's `populated 0` line was read,
    /// and no process is known to have been counted in the group:
    /// the shape of the stuck scope of issue 6578,
    /// so the manager is asked to stop the unit.
    EmptiedWithoutAProcess,
    /// The kernel's `populated 1` line was read:
    /// a member may still write.
    Member,
    /// The group's directory is gone,
    /// and its absence is verified:
    /// the manager collected the unit.
    Collected,
    /// Nothing could be read as emptiness,
    /// and why it could not.
    Unproven {
        /// Why the scope could not be read as empty or gone.
        reason: String,
    },
}

/// What:
/// `fn emptiness_now(group: &ControlGroup) -> Emptiness` reads the kernel's last word about the scope,
/// and says whether the unit behind it can be shown to need nothing.
/// Why:
/// This is the gate the retirement hangs on,
/// and the stop of the containment accepts exactly two proofs that a scope has no member:
/// a `populated` line that was read,
/// and an absence that is verified.
/// The retirement reads the same two,
/// so it never asks the manager to stop a unit whose member it has not shown gone;
/// and it needs one more fact to skip a question:
/// whether the group ever held a process,
/// because that is what makes the manager end the unit by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emptinessNow(group: ControlGroup): Emptiness
/// ```
fn emptiness_now(group: &ControlGroup) -> Emptiness {
    // A test makes the look answer the state of the stuck scope of issue 6578, which a run of a real program cannot leave:
    // its scope has held a process.
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::RetireSeesStuckScope) {
        return Emptiness::EmptiedWithoutAProcess;
    }
    // What:     `populated_at(...)` reads the kernel's `populated` line: `Ok(true)` is a member, `Ok(false)` is none, and
    //           `Err(error)` is a read that failed. `NotFound` and `NODEV` are a group whose directory is gone.
    // Why:      A group whose absence cannot be verified is not read as empty, and every other failure is a group that may
    //           still have a member.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { populated = populatedAt(group.directory) } catch (error) { ... }
    // ```
    let populated: bool = match populated_at(group.directory()) {
        Ok(found) => found,
        Err(error) if is_the_group_being_gone(&error) => {
            // What:     `absence_is_verified()` asks whether this process still sees the directory the scope was planned in.
            // Why:      The stop accepts an absence as emptiness only under that check, so the retirement does too: an
            //           absence it cannot verify keeps the unit and says so.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (absenceIsVerified(group)) return 'collected' else return { unproven: reason }
            // ```
            return match group.absence_is_verified() {
                Ok(()) => Emptiness::Collected,
                Err(reason) => Emptiness::Unproven {
                    reason: format!(
                        "the scope's directory {} is gone, and its absence could not be read as emptiness ({reason})",
                        group.directory().display()
                    ),
                },
            };
        }
        Err(error) => {
            return Emptiness::Unproven {
                reason: format!("the scope's `populated` line could not be read ({error})"),
            };
        }
    };
    if populated {
        return Emptiness::Member;
    }
    // What:     `pids_peak_at(...)` reads how many processes the group has ever held. `Ok(0)` means none was ever counted in
    //           it, which is the stuck scope; any other answer means the manager was told the group emptied.
    // Why:      Asking the manager costs a program start, and this read costs a file read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return pidsPeakAt(group.directory) > 0 ? 'emptied after a process' : 'emptied without one'
    // ```
    return match pids_peak_at(group.directory()) {
        Ok(peak) if peak > 0 => Emptiness::EmptiedAfterAProcess,
        // A host whose group has no `pids.peak` (the controller is not enabled for it) shows nothing, so the question is
        // asked: the safe side is one program start, never a unit left loaded without a word.
        Ok(_) | Err(_) => Emptiness::EmptiedWithoutAProcess,
    };
}

/// What:
/// `fn is_the_group_being_gone(error: &io::Error) -> bool` says whether a read of the scope failed because the group's
/// directory is not there any more.
/// Why:
/// A removed group answers `ENOENT` and its files answer `ENODEV` (the stop reads the same two as one thing);
/// every other failure is a group that may still have a member.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isTheGroupBeingGone(error: Error): boolean
/// ```
fn is_the_group_being_gone(error: &io::Error) -> bool {
    return error.kind() == io::ErrorKind::NotFound
        || error.raw_os_error() == Some(rustix::io::Errno::NODEV.raw_os_error());
}

/// What:
/// `pub(crate) fn retire(group: &ControlGroup, systemctl: &str, bound: Duration) -> Option<String>` ends the scope unit of a run
/// that has ended,
/// and answers the one warning a person has to read,
/// or `None`.
/// Why:
/// The scope is already shown empty or gone,
/// so this is housekeeping for the unit and never a claim about a writer:
/// a failure does not turn the run into "not quiescent".
/// A unit that may still have a member behind it keeps the unit,
/// because the refusal's report names that member and a manager stop would kill it behind the report;
/// a unit whose emptiness was not shown keeps it too,
/// and the run says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retire(group: ControlGroup, systemctl: string, boundMs: number): string | undefined
/// ```
pub(crate) fn retire(
    group: &ControlGroup,
    systemctl: &str,
    bound: Duration,
) -> Option<String> {
    let unit: String = group.unit();
    return match emptiness_now(group) {
        // The manager collected the unit when the scope stopped, or the kernel told it the group emptied and it will: a
        // retire has nothing to add, and the ordinary run ends without a program start here.
        Emptiness::Collected | Emptiness::EmptiedAfterAProcess => None,
        Emptiness::EmptiedWithoutAProcess => match retire_unit(systemctl, &unit, bound) {
            Ok(()) => None,
            Err(reason) => Some(format!(
                "the scope unit {unit} of the run that just ended may still be loaded ({reason})"
            )),
        },
        Emptiness::Member => Some(format!(
            "the scope unit {unit} of the run that just ended was left loaded: its control group still has a member, and a stop would kill it behind the report that names it"
        )),
        Emptiness::Unproven {
            reason,
        } => Some(format!(
            "the scope unit {unit} of the run that just ended was left loaded: {reason}, so it is not shown that no member is left"
        )),
    };
}

/// What:
/// `pub(crate) fn retire_unit(systemctl: &str, unit: &str, bound: Duration) -> Result<(), String>` asks the manager to stop
/// `unit` once,
/// and answers `Ok` when the unit is gone afterwards or was already gone,
/// or the reason it may still be loaded.
/// Why:
/// The run's scope is already shown empty,
/// so this is housekeeping and never a claim about a writer;
/// the caller reports a failure as a warning and does not turn it into "not quiescent".
/// A unit the manager collected before the request is not a failure:
/// `systemctl` answers exit status 5 (`EXIT_NOTINSTALLED`) for a unit it does not load,
/// which is what a collected unit is (measured:
/// twelve such stops answered 5 on 2026-11-27),
/// so one question settles both cases and the second question the release asked is gone.
/// The reason names the command as it ran,
/// built from the same arguments,
/// so a person who copies it runs what the run ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retireUnit(systemctl: string, unit: string, boundMs: number): void  // throws the reason
/// ```
pub(crate) fn retire_unit(
    systemctl: &str,
    unit: &str,
    bound: Duration,
) -> Result<(), String> {
    // What:     `run_asking(...)` starts `systemctl` with these arguments under `bound`, and answers its status and both streams
    //           or the reason it could not be read. The flags are the ones the machine's rules require of every `systemctl`
    //           call: `--user` keeps it on this account's manager, and `--no-ask-password` never opens a password dialog on a
    //           desktop.
    // Why:      The unit this run made has to be ended by the manager that made it, and the status is what tells a collected
    //           unit from a refusal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const answered = runAsking(systemctl, ['--user', '--no-ask-password', 'stop', unit], [], boundMs)
    // ```
    let arguments: [&str; 4] = ["--user", "--no-ask-password", "stop", unit];
    let text: String = command_text(systemctl, &arguments);
    return match run_asking(systemctl, &arguments, &[], bound) {
        // The manager ran the stop job, or it does not load the unit: either way the unit is gone.
        Ok(answered)
            if answered.status.success() || answered.status.code() == Some(UNIT_NOT_LOADED) =>
        {
            Ok(())
        }
        Ok(answered) => Err(format!("`{text}` failed ({})", first_line(&answered.err))),
        Err(reason) => Err(format!("`{text}` {reason}")),
    };
}

/// What:
/// `fn command_text(program: &str, arguments: &[&str]) -> String` is the command as a person would type it.
/// Why:
/// A warning that names a command has to name the one that ran,
/// with the flags it carried (`--no-ask-password` among them).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandText(program: string, args: string[]): string
/// ```
fn command_text(
    program: &str,
    arguments: &[&str],
) -> String {
    return format!("{program} {}", arguments.join(" "));
}

/// What:
/// `fn first_line(text: &[u8]) -> &str` is the first line of what a program wrote on standard error,
/// or a word saying it wrote none.
/// Why:
/// A refusal is read by a person,
/// and the manager's own sentence is the part that names the unit and the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstLine(bytes: Uint8Array): string
/// ```
fn first_line(text: &[u8]) -> String {
    return String::from_utf8_lossy(text)
        .lines()
        .next()
        .unwrap_or("no message")
        .trim()
        .to_string();
}

/// The controls of this module (issue 6578).
#[cfg(test)]
#[path = "control_group_retire_tests.rs"]
mod tests;
