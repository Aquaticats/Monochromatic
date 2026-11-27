//! What:
//!  The names one add's route chooses before anything is mutated:
//! the slot's
//!  record name,
//! the add's nonce,
//! the origin snapshot and the clone dataset,
//!  the request the add was given,
//! and the test-only switch that ends the add
//!  after a named step for the guest arms.
//! Why:
//!  The design writes the proposed clone and snapshot names in the binding before
//!  any ZFS object exists,
//! so a resume can find what the add made
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//! A2 and
//!  "The binding").
//!  The request is recorded because a resumed add proves it is the same request
//!  (`T21a` of the design's tests),
//! and it is read from the person's own words
//!  rather than guessed from what Git later did.
//!  The stop switch is the seam the guest arms kill the add at:
//! a name from a
//!  closed list makes the add send itself `SIGKILL` at that step's boundary,
//!  which is the shape the step 8 adoption arm already uses for a killed
//!  writer (`binary_zfs_adoption_guest_tests.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planTheAdd(facts, args); // { nonce, snapshot, cloneDataset, request }
//! stops.after('clone');                // a control run dies here
//! ```

use super::add_route_gate::RouteFacts;
use crate::worktree_clone_binding_record::binding_encoding::Request;
use crate::worktree_clone_binding_record::binding_words::RequestForm;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_command_facts::AddPlan;
use std::ffi::{
    OsStr,
    OsString,
};
use std::os::unix::ffi::OsStrExt;

/// The word an origin snapshot's name begins with,
/// before the nonce.
pub const ORIGIN_SNAPSHOT_PREFIX: &str = "git-worktree-clone-origin-";

/// The variable that makes a control add die after one named step.
pub const STOP_AFTER_VARIABLE: &str = "CLI_GIT_TEST_ONLY_ZFS_ADD_STOP_AFTER";

/// The variable naming the step a control add holds at until a file stands.
pub const HOLD_STEP_VARIABLE: &str = "CLI_GIT_TEST_ONLY_ZFS_ADD_HOLD_STEP";

/// The variable naming the file whose appearance releases a held control add.
pub const HOLD_UNTIL_VARIABLE: &str = "CLI_GIT_TEST_ONLY_ZFS_ADD_HOLD_UNTIL";

/// The bound on one control's hold,
/// so a test that dies before releasing it ends the add rather than the suite.
pub const HOLD_BOUND: std::time::Duration = std::time::Duration::from_secs(120);

/// Every step the stop switch knows,
/// in the add's order.
///
/// Why:
///  A closed list refuses a typo instead of running a kill arm that silently
///  killed nothing,
/// which the plan's guest arms would report as a pass.
pub const STOP_STEPS: &[&str] = &[
    "claim",
    "binding",
    "slot",
    "slot-identity",
    "git-add",
    "registered",
    "snapshot",
    "origin-inventory",
    "clone",
    "clone-inventory",
    "mount-intent",
    "mount",
    "proof",
    "leave-outs",
    "index",
    "publish",
    "trash-entry",
    "exchange",
    "filled",
];

/// What:
///  The names and request one add chose before its first ZFS write.
/// Why:
///  The binding's first write records all of them,
/// and every later step reads
///  them from there instead of choosing again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddNames = { nonce: RequestNonce; originSnapshot: string; cloneDataset: string; request: Request };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddNames {
    /// The random number the lock's sentence and the record's names are recomputed from.
    pub nonce: RequestNonce,
    /// The origin snapshot's whole name,
    /// `dataset@git-worktree-clone-origin-<nonce>`.
    pub origin_snapshot: String,
    /// The clone's dataset name.
    pub clone_dataset: String,
    /// How the person asked for the worktree.
    pub request: Request,
}

/// What:
///  The names one add chooses,
/// from the facts its gate proved and the words the
///  person typed.
/// Why:
///  Nothing here mutates or reads the pool,
/// so a control covers every request
///  form and both name forms on any machine.
///
/// # Errors
/// The random source failed,
/// or the person's words are not a form the add's plan read.
pub fn name_the_add(
    facts: &RouteFacts,
    plan: &AddPlan,
    arguments: &[OsString],
) -> Result<AddNames, String> {
    let nonce: RequestNonce = RequestNonce::fresh().map_err(|refusal| return format!("the add cannot take a random number for its lock and records ({refusal:?}), so nothing was changed"))?;
    let origin_snapshot: String = format!(
        "{}@{ORIGIN_SNAPSHOT_PREFIX}{}",
        facts.checkout_dataset,
        nonce.as_str()
    );
    let request: Request = request_of(plan, arguments)?;
    return Ok(AddNames {
        nonce,
        origin_snapshot,
        clone_dataset: facts.clone_dataset(),
        request,
    });
}

/// What:
///  How the person's words asked for the worktree,
/// in the record's three forms.
/// Why:
///  The recorded request must survive a resume unchanged,
/// so it is read from
///  the words alone:
/// a detached add carries its commit-ish or none,
/// a named new
///  branch carries its name,
/// and an add with neither carries the target it
///  named or the slot's own name,
/// which is the branch Git derives from the
///  destination.
///
/// # Errors
/// A `-b` or `-B` with no value,
/// or a word after the options that is not text.
pub fn request_of(
    plan: &AddPlan,
    arguments: &[OsString],
) -> Result<Request, String> {
    let mut detached: bool = false;
    let mut new_branch: Option<Vec<u8>> = None;
    let mut index: usize = plan.insert_at;
    while let Some(word) = arguments.get(index) {
        let bytes: &[u8] = word.as_bytes();
        if bytes == b"--" || bytes == b"--end-of-options" {
            break;
        }
        if let Some(long) = bytes.strip_prefix(b"--") {
            if long == b"detach" {
                detached = true;
            }
            index += 1;
            continue;
        }
        if bytes.len() > 1 && bytes[0] == b'-' {
            let mut at: usize = 1;
            while let Some(letter) = bytes.get(at) {
                if *letter == b'd' {
                    detached = true;
                    at += 1;
                    continue;
                }
                if *letter == b'b' || *letter == b'B' {
                    let attached: &[u8] = &bytes[at + 1..];
                    if !attached.is_empty() {
                        new_branch = Some(attached.to_vec());
                    } else {
                        let value: &OsString = arguments.get(index + 1).ok_or_else(|| return format!("the add's `-{}` names no branch, so its request cannot be recorded", char::from(*letter)))?;
                        new_branch = Some(value.as_bytes().to_vec());
                        index += 1;
                    }
                    break;
                }
                at += 1;
            }
            index += 1;
            continue;
        }
        break;
    }
    if detached {
        return Ok(match plan.target.as_deref() {
            Some(target) => Request::with_argument(RequestForm::Detach, target.as_bytes()),
            None => Request::detached(),
        });
    }
    if let Some(branch) = new_branch {
        return Ok(Request::with_argument(
            RequestForm::Branch,
            branch.as_slice(),
        ));
    }
    return Ok(match plan.target.as_deref() {
        Some(target) => Request::with_argument(RequestForm::Commit, target.as_bytes()),
        None => Request::with_argument(
            RequestForm::Branch,
            OsStr::new(plan.destination.file_name().unwrap_or_default()).as_bytes(),
        ),
    });
}

/// What:
///  The step names a control add stops after,
/// read once.
/// Why:
///  A release build reads the variable too and finds nothing,
/// so the switch is
///  one optional word and never a mode a shipped add enters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stops = { after?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stops {
    /// The step whose boundary kills this run,
    /// when the variable named one.
    step: Option<String>,
    /// The step this run waits at,
    /// when a control named one.
    hold_step: Option<String>,
    /// The file whose appearance releases the wait.
    hold_until: Option<std::path::PathBuf>,
}

impl Stops {
    /// The switch this process's environment sets,
    /// or no switch.
    ///
    /// # Errors
    /// The variable names a step the add does not know.
    pub fn from_environment() -> Result<Stops, String> {
        // The three switches a control of the add arms, which a release build never reads.
        let hold_step: Option<String> = crate::test_only_switches::value(HOLD_STEP_VARIABLE)
            .and_then(|value| return value.into_string().ok());
        let hold_until: Option<std::path::PathBuf> =
            crate::test_only_switches::value(HOLD_UNTIL_VARIABLE).map(std::path::PathBuf::from);
        let Some(value) = crate::test_only_switches::value(STOP_AFTER_VARIABLE) else {
            return Ok(Stops {
                step: None,
                hold_step,
                hold_until,
            });
        };
        let text: String = value
            .into_string()
            .map_err(|_raw| return format!("{STOP_AFTER_VARIABLE} is not text"))?;
        let mut stops: Stops = Stops::named(text.as_str())?;
        stops.hold_step = hold_step;
        stops.hold_until = hold_until;
        return Ok(stops);
    }

    /// A switch that stops after `step`.
    ///
    /// # Errors
    /// The name is not one of the add's steps.
    pub fn named(step: &str) -> Result<Stops, String> {
        if !STOP_STEPS.contains(&step) {
            return Err(format!(
                "{STOP_AFTER_VARIABLE} names `{step}`, and the add's steps are {}",
                STOP_STEPS.join(", ")
            ));
        }
        return Ok(Stops {
            step: Some(String::from(step)),
            hold_step: None,
            hold_until: None,
        });
    }

    /// No switch at all.
    pub fn none() -> Stops {
        return Stops {
            step: None,
            hold_step: None,
            hold_until: None,
        };
    }

    /// What:
    ///  Wait at `step` until the control's release file stands,
    ///  when a control named this step.
    /// Why:
    ///  The full-pool arm has to fill the pool between the capture and the
    ///  snapshot,
    /// where no other state can tell it the moment arrived;
    /// the wait
    ///  is bounded so a dead test ends the add.
    ///  The pause holds the add's own thread,
    /// which runs no task,
    /// so no worker of the runtime
    ///  waits here;
    /// the guard's exception list names it beside the mount's wait of the same route.
    ///
    /// # Errors
    /// The bound passed with no release file,
    /// or the waiting marker cannot be written.
    pub fn hold(
        &self,
        step: &str,
    ) -> Result<(), String> {
        if self.hold_step.as_deref() != Some(step) {
            return Ok(());
        }
        let Some(until) = self.hold_until.as_ref() else {
            return Ok(());
        };
        let waiting: std::path::PathBuf = until.with_extension("waiting");
        std::fs::write(waiting.as_path(), b"waiting\n").map_err(|error| {
            return format!(
                "the control's waiting marker {} cannot be written ({error})",
                waiting.display()
            );
        })?;
        let started: std::time::Instant = std::time::Instant::now();
        while !until.exists() {
            if started.elapsed() > HOLD_BOUND {
                return Err(format!(
                    "the control never released the add at `{step}` ({} did not appear), so it stopped before the step it was meant to measure",
                    until.display()
                ));
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        return Ok(());
    }

    /// End this process by `SIGKILL` when the switch names `step`,
    /// exactly as a
    /// kill from outside would.
    pub fn after(
        &self,
        step: &str,
    ) {
        if self.step.as_deref() != Some(step) {
            return;
        }
        // SAFETY: the call sends this process a signal and touches nothing else.
        unsafe { libc::raise(libc::SIGKILL) };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_route_plan_tests.rs"]
mod tests;
