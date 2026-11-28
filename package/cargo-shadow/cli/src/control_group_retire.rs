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
//! (the "The cost,"
//! paragraph of "What cargo-shadow does now" in
//! `doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md`).
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
/// `use std::fs;` imports the standard library's file-system module so this file can name it unqualified.
/// Why:
/// The peak read's failure needs one more look at the group's directory,
/// and `fs::metadata` is the only read that tells a
/// directory that is there from one that is gone or unreadable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// The two reads of the scope,
/// the peak read and the directory check answer an `io::Error`,
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
    /// and why it could not,
    /// and whether the kernel's `populated 0` line was read before that.
    Unproven {
        /// Why the scope could not be read as empty or gone.
        reason: String,
        /// Whether an emptiness was read before the reason applied:
        /// the peak arm reads `populated 0` and then loses the group's directory,
        /// so the read that failed is not a read of the members.
        an_empty_line_was_read: bool,
    },
}

/// What:
/// `fn the_populated_line(group: &ControlGroup) -> io::Result<bool>` reads the kernel's `populated` line of the group,
/// or the failure a control arms in its place.
/// Why:
/// The read goes through one place,
/// so a control can answer what a group the kernel is removing answers (`ENODEV`)
/// at a moment a run cannot time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thePopulatedLine(group: ControlGroup): boolean  // throws
/// ```
fn the_populated_line(group: &ControlGroup) -> io::Result<bool> {
    // A test makes this read answer the error a group the kernel is removing answers, which a run meets only by chance:
    // the file has to be opened before the removal takes it away and read after that (`kernfs_seq_start` answers
    // `ENODEV` for a file whose group was deactivated, `fs/kernfs/file.c:166` at v7.3.5). The same read against a group
    // that is only gone answers `ENOENT`, before any file in it is opened.
    #[cfg(test)]
    if crate::failpoints::retire_reads::fires_populated_read_sees_the_gone_group() {
        return Err(io::Error::from_raw_os_error(
            rustix::io::Errno::NODEV.raw_os_error(),
        ));
    }
    return populated_at(group.directory());
}

/// What:
/// `fn the_peak_of(group: &ControlGroup) -> io::Result<u64>` reads how many processes have ever been counted in the group,
/// or the failure a control arms in its place.
/// Why:
/// The read goes through one place,
/// so a control can answer the absence a group the manager removed answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thePeakOf(group: ControlGroup): number  // throws
/// ```
fn the_peak_of(group: &ControlGroup) -> io::Result<u64> {
    // A test makes this read answer the absence of a group the manager removed, which is the window the look's two reads
    // can straddle: nothing outside holds that window open.
    #[cfg(test)]
    if crate::failpoints::retire_reads::fires_peak_read_sees_the_gone_group() {
        return Err(io::Error::from_raw_os_error(
            rustix::io::Errno::NOENT.raw_os_error(),
        ));
    }
    return pids_peak_at(group.directory());
}

/// What:
/// `fn the_group_s_directory_is_gone(group: &ControlGroup) -> bool` says whether the group's own directory is not there.
/// Why:
/// Both reads of the look can answer an absence,
/// and only a directory that is gone makes that absence the manager's collection:
/// a read of a file that is not there inside a directory that is means the kernel has no such file,
/// not that the group went away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theGroupDirectoryIsGone(group: ControlGroup): boolean
/// ```
fn the_group_s_directory_is_gone(group: &ControlGroup) -> bool {
    // What:     `fs::metadata(group.directory())` examines the directory itself: `Ok` is one that is there (a file counts),
    //           and `Err` is the reason it could not be examined.
    // Why:      A read of a file inside the directory cannot tell a missing file from a missing directory, and this is the
    //           one read that can.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { statSync(group.directory) } catch (error) { return isGone(error) }
    // ```
    match fs::metadata(group.directory()) {
        Ok(_there) => {}
        Err(error) => return is_the_group_being_gone(&error),
    }
    // A test makes this check answer the absence of a group the manager removed between the look's two reads: a directory
    // that is there answers no such thing.
    #[cfg(test)]
    if crate::failpoints::retire_reads::fires_directory_check_sees_the_gone_group() {
        return true;
    }
    return false;
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
    // What:     `the_populated_line(...)` reads the kernel's `populated` line: `Ok(true)` is a member, `Ok(false)` is none,
    //           and `Err(error)` is a read that failed.
    // Why:      A group whose absence cannot be verified is not read as empty, and every other failure is a group that may
    //           still have a member.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { populated = populatedAt(group.directory) } catch (error) { ... }
    // ```
    let populated: bool = match the_populated_line(group) {
        Ok(found) => found,
        // What:     `the_absence_is_the_scope_going_away(group, &error)` says whether the failed read is the group going
        //           away rather than a group that may still have a member.
        // Why:      `ENODEV` is the kernel taking the group's own files away on its way to removing the group, which it
        //           does only for a group that holds no task (`kernel/cgroup/cgroup.c:6333` at v7.3.5), so the unit is the
        //           manager's to free and the step says nothing about it. `ENOENT` is only the absence of a name, so it is
        //           read as the group's absence when the group's directory is gone as well.
        //           The directory check is not made for `ENODEV`: measured on this host on 2026-11-27, a group being removed
        //           is read as `ENODEV` while its directory is still there (300 removals raced by this step's own read:
        //           24 answered `ENODEV`, and a directory check right after each found the directory there in 20 of them),
        //           so that check would turn a right silence into a warning about a unit the manager is freeing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (theAbsenceIsTheScopeGoingAway(group, error)) return collectedOrUnproven(group, false)
        // ```
        Err(error) if the_absence_is_the_scope_going_away(group, &error) => {
            // What:     `collected_or_unproven(group, false, &error)` says whether the absence can be read as emptiness, and that no
            //           emptiness was read before it.
            // Why:      The stop accepts an absence as emptiness only under that check, so the retirement does too: an
            //           absence it cannot verify keeps the unit and says so.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return collectedOrUnproven(group, false, error)
            // ```
            return collected_or_unproven(group, false, &error);
        }
        Err(error) => {
            return Emptiness::Unproven {
                reason: format!("the scope's `populated` line could not be read ({error})"),
                an_empty_line_was_read: false,
            };
        }
    };
    if populated {
        return Emptiness::Member;
    }
    // What:     `the_peak_of(...)` reads how many processes the group has ever held. `Ok(0)` means none was ever counted in
    //           it, which is the stuck scope; any other answer means the manager was told the group emptied.
    // Why:      Asking the manager costs a program start, and this read costs a file read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return thePeakOf(group) > 0 ? 'emptied after a process' : 'emptied without one'
    // ```
    return match the_peak_of(group) {
        Ok(peak) if peak > 0 => Emptiness::EmptiedAfterAProcess,
        // What:     `group_gone_at_the_peak(group, &error)` says whether this failure means the group's directory went away
        //           between the two reads, which is a collected unit, or something else.
        // Why:      A missing `pids.peak` and a group that went away are different facts about the unit, and only the second
        //           one needs no question.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const gone = groupGoneAtThePeak(group, error)
        // if (gone) return gone
        // return 'emptied without one'
        // ```
        Err(error) => match group_gone_at_the_peak(group, &error) {
            Some(answer) => answer,
            // A host whose group has no `pids.peak` (the controller is not enabled for it) shows nothing, so the question is
            // asked: the safe side is one program start, never a unit left loaded without a word.
            None => Emptiness::EmptiedWithoutAProcess,
        },
        Ok(_) => Emptiness::EmptiedWithoutAProcess,
    };
}

/// What:
/// `fn is_the_group_being_gone(error: &io::Error) -> bool` says whether a read of the scope failed because the group is not
/// there any more,
/// either as a name (`ENOENT`) or as a file the kernel has taken away (`ENODEV`).
/// Why:
/// Those two are the only failures that say anything about the group's own existence,
/// and the stop of a containment reads `ENODEV` the same way
/// (`populated_through` answers the empty line for it);
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
/// `fn the_kernel_is_removing_the_group(error: &io::Error) -> bool` says whether a read failed because the kernel has already
/// deactivated the group's own files,
/// which it does on its way to removing the group.
/// Why:
/// `ENODEV` is the kernel's own statement that the group behind the file is being taken apart:
/// `kernfs_seq_start` answers it where it can no longer take an active reference to the file's node
/// (`fs/kernfs/file.c:166` at v7.3.5),
/// and `kernfs_fop_open` answers it where the node was deactivated before the file was opened (`:612`).
/// That is a stronger fact than `ENOENT`,
/// which is only the absence of a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theKernelIsRemovingTheGroup(error: Error): boolean
/// ```
fn the_kernel_is_removing_the_group(error: &io::Error) -> bool {
    return error.raw_os_error() == Some(rustix::io::Errno::NODEV.raw_os_error());
}

/// What:
/// `fn the_absence_is_the_scope_going_away(group: &ControlGroup, error: &io::Error) -> bool` says whether a failed read of the
/// scope is the scope going away,
/// rather than a read that failed for a reason of its own.
/// Why:
/// A group the kernel is removing is empty,
/// because the kernel refuses to remove a group that still holds a task,
/// so `ENODEV` alone is the scope's absence and needs no directory check;
/// and `ENOENT` is that absence only when the group's directory is gone as well,
/// because a file that is not there inside a directory that is means the kernel has no such file.
/// The peak read is where that second half is live:
/// a group whose parent does not enable the pids controller has no `pids.peak` at all
/// (`README.md`,
/// "`pids.peak` is a non-root group's file").
/// It is not live for the `populated` read of a group:
/// measured on this host on 2026-11-27,
/// 3090 reads raced against 300 creations never answered `ENOENT` while the directory was there,
/// and every first sight of a new directory held `cgroup.events`,
/// in a listing of the directory in 88 of the 300 iterations and in the successful read itself in the other 212.
/// That is because the control group root is created deactivated
/// (`KERNFS_ROOT_CREATE_DEACTIVATED` at `kernel/cgroup/cgroup.c:2184` at v7.3.5),
/// and `cgroup_mkdir` activates the directory and its files under one lock
/// (`kernfs_activate` at `:6157` after `css_populate_dir` at `:6146`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theAbsenceIsTheScopeGoingAway(group: ControlGroup, error: Error): boolean
/// ```
fn the_absence_is_the_scope_going_away(
    group: &ControlGroup,
    error: &io::Error,
) -> bool {
    if !is_the_group_being_gone(error) {
        return false;
    }
    // What:     `the_kernel_is_removing_the_group(error)` is the stronger of the two answers, and it needs no directory to
    //           confirm it.
    // Why:      The directory is measurably still there while the kernel is removing the group, so asking it would refuse the
    //           answer the kernel just gave.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (theKernelIsRemovingTheGroup(error)) return true
    // ```
    if the_kernel_is_removing_the_group(error) {
        return true;
    }
    // What:     `the_group_s_directory_is_gone(group)` is the check `ENOENT` needs: a name that is not there is the group's
    //           absence only when the group's own directory is gone too.
    // Why:      A kernel that has the file and not the directory says nothing about a member, so that state must not be read
    //           as the manager's collection.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return theGroupDirectoryIsGone(group)
    // ```
    return the_group_s_directory_is_gone(group);
}

/// What:
/// `fn group_gone_at_the_peak(group: &ControlGroup, error: &io::Error) -> Option<Emptiness>` says what a failed `pids.peak`
/// read means for the unit,
/// or `None` when it says nothing about the group's absence.
/// Why:
/// The causes of that failure want different answers:
/// a file that is not there in a directory that is means the kernel tells nothing about a process ever counted in the
/// group,
/// which only a question to the manager settles;
/// a group that went away or that the kernel is removing is a group the manager is freeing,
/// which no question settles better.
/// A control drives both,
/// because the race this comes from (the removal of the group between the two reads) cannot be
/// staged from outside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupGoneAtThePeak(group: ControlGroup, error: Error): Emptiness | undefined
/// ```
fn group_gone_at_the_peak(
    group: &ControlGroup,
    error: &io::Error,
) -> Option<Emptiness> {
    // What:     `if !the_absence_is_the_scope_going_away(group, error)` leaves every failure that is not the scope's absence
    //           to the caller, which asks the manager.
    // Why:      A read that was refused for another reason says nothing about the group's existence, and a `pids.peak` that
    //           is not there in a directory that is means a host without the file (before Linux 6.1, or a group whose parent
    //           does not enable the pids controller), which only the manager can be asked about.
    //           A directory that cannot be read for another reason is not a directory that went away either: `Path::is_dir`
    //           answers `false` for every failure, so it would read `EACCES` and `EIO` as an absence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!theAbsenceIsTheScopeGoingAway(group, error)) return undefined
    // ```
    if !the_absence_is_the_scope_going_away(group, error) {
        return None;
    }
    // What:     `Some(collected_or_unproven(group, true, error))` is the answer the absent `populated` read gets, and it says that
    //           an empty `populated` line was read before the peak read.
    // Why:      The two reads are of the same group, so an absence seen at the second is the absence the first would have seen,
    //           and the sentence built from it may say what the first read showed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return collectedOrUnproven(group, true, error)
    // ```
    return Some(collected_or_unproven(group, true, error));
}

/// What:
/// `fn collected_or_unproven(group: &ControlGroup, an_empty_line_was_read: bool, absence: &io::Error) -> Emptiness` says what a group
/// whose directory is gone means for its unit:
/// `Collected` when this process still sees the directory the scope was planned in,
/// and `Unproven` with the reason when it does not,
/// carrying whether an emptiness had been read before the absence.
/// Why:
/// Both reads of the group can find it gone,
/// and an absence the stop accepts as emptiness is the only absence the retirement may read as a free unit;
/// the warning that follows must not deny an emptiness the same look read,
/// and it must say what the read saw rather than what the reader expected:
/// measured on this host on 2026-11-27,
/// 24 of the removals that answered `ENODEV` still had the group's directory there when it was checked right after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectedOrUnproven(group: ControlGroup, anEmptyLineWasRead: boolean, absence: Error): Emptiness
/// ```
fn collected_or_unproven(
    group: &ControlGroup,
    an_empty_line_was_read: bool,
    absence: &io::Error,
) -> Emptiness {
    // What:     `group.absence_is_verified()` asks whether this process still sees the directory the scope was planned in.
    // Why:      The stop accepts an absence as emptiness only under that check, so the retirement does too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (absenceIsVerified(group)) return 'collected' else return { unproven: reason }
    // ```
    return match group.absence_is_verified() {
        Ok(()) => Emptiness::Collected,
        Err(reason) => {
            // What:     `what_was_read` names the failure the look met: the kernel taking the group's own files away
            //           (`ENODEV`), or the group's directory not being there (`ENOENT`).
            // Why:      `ENODEV` is the kernel's statement about the group's files, and the directory is measurably still
            //           there while it is made, so a sentence saying the directory is gone would describe a state the read did
            //           not see.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const whatWasRead = theKernelIsRemovingTheGroup(absence)
            //   ? `the kernel has taken the group's own files away, so the scope's directory ${group.directory()} may still be there`
            //   : `the scope's directory ${group.directory()} is not there`
            // ```
            let what_was_read: String = match the_kernel_is_removing_the_group(absence) {
                true => format!(
                    "the kernel has taken the group's own files away, so the scope's directory {} may still be there",
                    group.directory().display()
                ),
                false => format!(
                    "the scope's directory {} is not there",
                    group.directory().display()
                ),
            };
            // What:     `Emptiness::Unproven { .. }` is the answer that keeps the unit, with the sentence the run prints.
            // Why:      A unit whose emptiness was not shown is kept, and the warning says what was read and what could not be.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { unproven: `${whatWasRead}, and its absence could not be read as emptiness (${reason})`, anEmptyLineWasRead }
            // ```
            Emptiness::Unproven {
                reason: format!(
                    "{what_was_read}, and its absence could not be read as emptiness ({reason})"
                ),
                an_empty_line_was_read,
            }
        }
    };
}

/// What:
/// `pub(crate) fn retire(group: &ControlGroup, systemctl: &str, bound: Duration) -> Option<String>` ends the scope unit of a run
/// that has ended,
/// and answers the one warning a person has to read,
/// or `None`.
/// Why:
/// The step runs after the stop of the containment whatever that stop answered,
/// a refusal included,
/// so the scope may still hold a member and this is never a claim about a writer:
/// a failure does not turn the run into "not quiescent".
/// A unit that may still have a member behind it keeps the unit,
/// because the refusal's report names that member and a manager stop would kill it;
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
            "the scope unit {unit} of the run that just ended was left loaded: the kernel's `populated 1` line was read for its control group, so this step asked for no stop"
        )),
        // What:     `an_empty_line_was_read` decides the tail of the warning: the peak arm read `populated 0` before the group's
        //           directory went away, and the arm that failed to read that line read nothing about the members.
        // Why:      A warning that ended "so it is not shown that no member is left" would deny the emptiness the same look
        //           read (`doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md`, "A `populated 0`
        //           line is the proof that no member is left"), and a tail that only said an empty line had been read, after a
        //           clause saying emptiness could not be read, read as both claims at once. So the tail says what was read
        //           first and what happened after it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const tail = anEmptyLineWasRead
        //   ? `the kernel's \`populated 0\` line was read for it, and then ${reason}`
        //   : `${reason}, so it is not shown that no member is left`
        // ```
        Emptiness::Unproven {
            reason,
            an_empty_line_was_read,
        } => Some(match an_empty_line_was_read {
            true => format!(
                "the scope unit {unit} of the run that just ended may be left loaded: the kernel's `populated 0` line was read for it, and then {reason}"
            ),
            false => format!(
                "the scope unit {unit} of the run that just ended may be left loaded: {reason}, so it is not shown that no member is left"
            ),
        }),
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
        Err(reason) => Err(format!(
            "`{text}` {}",
            without_the_program(systemctl, reason)
        )),
    };
}

/// What:
/// `fn without_the_program(program: &str, reason: String) -> String` is the short program runner's reason with the program's
/// own name taken off its front,
/// when it starts with it.
/// Why:
/// The question's text already names the program and its flags,
/// and the runner's reason opens with the program again,
/// so a warning that carried both would read
/// ```text
/// `systemctl --user --no-ask-password stop X` systemctl did not answer within 5s
/// ```
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withoutTheProgram(program: string, reason: string): string
/// ```
fn without_the_program(
    program: &str,
    reason: String,
) -> String {
    let prefix: String = format!("{program} ");
    return match reason.strip_prefix(&prefix) {
        Some(rest) => rest.to_string(),
        None => reason,
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

/// The control of what a run's last look at its scope answers when the scope's directory was never made,
/// which is the state a launcher that exits before the manager makes the group leaves.
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "control_group_retire_scope_not_made_yet_tests.rs"]
mod control_group_retire_scope_not_made_yet_tests;
