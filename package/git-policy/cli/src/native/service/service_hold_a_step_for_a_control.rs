//! What:
//!  A step of the service that a control can hold open,
//! in the process that runs it,
//! by naming the step in an environment variable:
//! the step says it was reached by making a file,
//! and goes on when the control makes another.
//! Why:
//!   A control of the service has to put the service in a state and not wait for one:
//! "the service is inside a trash while the commit is made" is a state,
//! and a duration would measure the machine.
//! The holds a control of one process sets are not this:
//! the service is a child process,
//! so what reaches it is a variable and a directory
//! (`service_fault.rs` names the same reason for the fault points it ends the service at).
//! Two variables, both inert unless a control sets them:
//! `CLI_GIT_TEST_ONLY_HOLD` is a comma-separated list of the steps to hold,
//! and `CLI_GIT_TEST_ONLY_HOLD_DIR` is where the reached and let-go files go.
//! A hold that is not let go ends at `PHASE_PATIENCE`,
//! so a control that dies cannot leave the service of a fixture held for ever
//! (nothing may wait without an end).
//! The hold awaited frees whatever polled it:
//! a step that waits on a child, a remote or a timer holds no thread of the process,
//! which is how the shipped steps of those kinds wait,
//! so a held one costs a control nothing the work it stands in for would not cost.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (holds.includes(point)) { await writeFile(`${dir}/${point}.reached`, ''); while (!existsSync(`${dir}/${point}.let-go`)) await sleep(20) }
//! ```

use crate::wait_ceilings::PHASE_PATIENCE;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// The steps a control may hold,
/// named as the call sites pass them.
pub const INSIDE_A_TRASH: &str = "inside-a-trash";
/// A trash's one entry being removed,
/// which is the step of that kind that holds a place of the pool.
pub const REMOVING_A_TRASH_ENTRY: &str = "removing-a-trash-entry";
/// The push of a landed commit,
/// waiting on the remote.
pub const INSIDE_A_PUSH: &str = "inside-a-push";
/// A new worktree's package state being relinked,
/// waiting on a tool.
pub const INSIDE_A_RELINK: &str = "inside-a-relink";
/// A new worktree's Cargo build directory being filled,
/// waiting on a tool.
pub const INSIDE_A_PREWARM: &str = "inside-a-prewarm";
/// One step of a slow clone of a Cargo build directory,
/// which is the step of a prewarm that holds a place of the pool.
pub const INSIDE_A_CLONE_STEP: &str = "inside-a-clone-step";
/// The review of a landed commit,
/// waiting on its endpoint.
pub const INSIDE_A_REVIEW: &str = "inside-a-review";
/// The automatic upkeep of a repository,
/// waiting on Git.
pub const INSIDE_THE_UPKEEP: &str = "inside-the-upkeep";
/// The deletion of a removed worktree's subvolume.
pub const INSIDE_A_SUBVOLUME_DELETE: &str = "inside-a-subvolume-delete";
/// The background sync of a landing's records.
pub const INSIDE_A_SYNC: &str = "inside-a-sync";
/// The removal of a landed request's scratch tree.
pub const INSIDE_A_SCRATCH_REMOVAL: &str = "inside-a-scratch-removal";
/// The look that says whose a request is.
pub const IN_THE_ROUTING_LOOK: &str = "in-the-routing-look";
/// One trash's emptying, between its bound's check and the wave of entries it is about to hand over.
pub const BEFORE_A_TRASH_WAVE: &str = "before-a-trash-wave";

/// The variable that names the steps to hold,
/// comma separated.
pub const HOLD_VARIABLE: &str = "CLI_GIT_TEST_ONLY_HOLD";

/// The variable that names the directory the reached and let-go files go in.
pub const HOLD_DIRECTORY_VARIABLE: &str = "CLI_GIT_TEST_ONLY_HOLD_DIR";

/// How often a held step looks for the file that lets it go,
/// which is the resolution of a control and not a wait of the service.
const THE_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// The steps a control named,
/// read once for the life of the process.
static NAMED: OnceLock<Vec<String>> = OnceLock::new();

/// The directory a control named,
/// read once for the life of the process.
static DIRECTORY: OnceLock<Option<PathBuf>> = OnceLock::new();

/// What:
///  The point and the directory a control of this process states,
/// for a control whose hold is in the process itself rather than in a fixture's service.
/// Why:
///  The variables are read once for the life of a process and a test process runs one control,
/// so a control of the package's own code can state the same hold here instead of in its environment.
/// Compiled only into a test build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdForAControl('before-a-trash-wave', holds); // this process holds that step
/// ```
#[cfg(test)]
static IN_THIS_PROCESS: std::sync::Mutex<Option<(String, PathBuf)>> = std::sync::Mutex::new(None);

/// What:
///  Hold `point` in this process at `directory`, as the variables would.
/// Why:
///  A control of the package's own code (a unit test of the emptying) cannot start a service of its own,
///  so it states the hold here and the step waits for the let-go file a control writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdForAControl(BEFORE_A_TRASH_WAVE, holds)
/// ```
#[cfg(test)]
pub(crate) fn hold_for_a_control(point: &str, directory: &std::path::Path) {
    *IN_THIS_PROCESS.lock().expect("the hold this process states") = Some((String::from(point), directory.to_path_buf()));
}

/// The steps a control named,
/// which is nothing in every run that sets no such variable.
fn the_steps_a_control_named() -> &'static [String] {
    return NAMED.get_or_init(|| {
        let named: Option<String> = std::env::var(HOLD_VARIABLE).ok();
        return named.map_or_else(Vec::new, |text| {
            return text.split(',').map(str::trim).filter(|step| { return !step.is_empty(); }).map(str::to_string).collect();
        });
    });
}

/// The directory a control named,
/// which is nothing in every run that sets no such variable.
fn the_directory_a_control_named() -> Option<&'static PathBuf> {
    return DIRECTORY.get_or_init(|| { return std::env::var_os(HOLD_DIRECTORY_VARIABLE).map(PathBuf::from); }).as_ref();
}

/// What:
///  The file that says the step `point` was reached.
/// Why:
///   A control waits for a state and not for a duration,
/// so it reads this file instead of sleeping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reached = `${dir}/${point}.reached`;
/// ```
pub fn reached_file(directory: &std::path::Path, point: &str) -> PathBuf {
    return directory.join(format!("{point}.reached"));
}

/// The file that lets the step `point` go on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const letGo = `${dir}/${point}.let-go`;
/// ```
pub fn let_go_file(directory: &std::path::Path, point: &str) -> PathBuf {
    return directory.join(format!("{point}.let-go"));
}

/// What:
///  Whether a control asked for `point` to be held.
/// Why:
///   A caller that must not hold a thread asks this before it awaits the hold,
/// so a shipped run pays one scan of a list that is empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!holds.includes(point)) return;
/// ```
fn a_control_asked(point: &str) -> Option<PathBuf> {
    // A control of this process states its hold in memory, and the variables are for a process it cannot reach.
    #[cfg(test)]
    if let Some((named, directory)) = IN_THIS_PROCESS.lock().expect("the hold this process states").as_ref()
        && named == point
    {
        return Some(directory.clone());
    }
    let directory: &PathBuf = the_directory_a_control_named()?;
    if !the_steps_a_control_named().iter().any(|named| { return named == point; }) {
        return None;
    }
    return Some(directory.clone());
}

/// What:
///  Holds `point` until the control lets it go,
/// awaited,
/// so the thread that reached it is free.
/// Why:
///   The steps that wait on a child, a remote or a timer hold no thread,
/// so a held one must not either:
/// a hold that blocked a worker would be a state the shipped code cannot reach,
/// and a service with four workers would stop serving.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await holdAwaiting(point);
/// ```
pub async fn hold_awaiting(point: &str) {
    let Some(directory) = a_control_asked(point) else { return };
    if !reach(&directory, point) {
        return;
    }
    let deadline: Instant = Instant::now() + PHASE_PATIENCE;
    while Instant::now() < deadline {
        if let_go_file(directory.as_path(), point).exists() {
            return;
        }
        tokio::time::sleep(THE_POLL_INTERVAL).await;
    }
    crate::service_warn!("the step {point} was held by a control that never let it go, so it went on after {PHASE_PATIENCE:?}");
}

/// What:
///  Holds `point` until the control lets it go,
/// on this thread,
/// for a step that already holds a thread of the pool.
/// Why:
///   A trash's entry removal holds a place of the pool for as long as it runs,
/// and a control that holds it open must hold what the removal holds:
/// the thread is the resource the shipped removal takes,
/// so a held removal is the state that starves the pool if anything can starve it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdBlocking(point);
/// ```
pub fn hold_blocking(point: &str) {
    let Some(directory) = a_control_asked(point) else { return };
    if !reach(&directory, point) {
        return;
    }
    let deadline: Instant = Instant::now() + PHASE_PATIENCE;
    while Instant::now() < deadline {
        if let_go_file(directory.as_path(), point).exists() {
            return;
        }
        std::thread::sleep(THE_POLL_INTERVAL);
    }
    crate::service_warn!("the step {point} was held by a control that never let it go, so it went on after {PHASE_PATIENCE:?}");
}

/// What:
///  Say the step was reached,
/// and whether a control may hold it.
/// Why:
///   A control learns the state from the file,
/// and a hold whose file cannot be made is one the control cannot wait for,
/// so the step is not held and says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { writeFileSync(`${dir}/${point}.reached`, '') ; return true } catch { warn(); return false }
/// ```
fn reach(directory: &std::path::Path, point: &str) -> bool {
    let reached: PathBuf = reached_file(directory, point);
    if let Err(error) = std::fs::write(reached.as_path(), b"") {
        crate::service_warn!("the step {point} cannot be held, because {} cannot be made: {error}", reached.display());
        return false;
    }
    return true;
}
