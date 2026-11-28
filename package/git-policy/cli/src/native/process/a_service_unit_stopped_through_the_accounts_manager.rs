//! What:
//!  The stop of a locked holder's own service unit through the account's own
//!  user manager,
//!  the wait that follows it,
//!  and the verdict each read after that wait gives.
//! Why:
//!  The owner's answer of 2026-11-23 is that the removal stops that unit
//!  through the person's own manager and goes on,
//!  naming the unit so the person can start it again
//!  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
//!  "A holder whose copy is locked (V10)").
//!  The design fixes the calls and the three answers that still refuse:
//!  a stop the manager answers with a failure,
//!  a task still in the unit's control group when the wait ends,
//!  and a unit that is active again because it restarts itself
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The locked holder,
//!  and the unit the wrapper may stop").
//!  The wait itself has no value in this build,
//!  so the release cannot stop anything:
//!  `the_stop_wait` refuses,
//!  and the value a run uses is the harness's own
//!  (`forced_arm_numbers.rs`).
//!  The wait's verdict is a plain function of three reads,
//!  which is how each of the three answers is shown without a manager.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stopped = stopTheUnit('/run/user/0/bus', { unit, cgroup, tasksCgroup, namespace, identity }, stopWait);
//! ```

use crate::foundation::plain_file::read_plain_text_at;
use crate::process::a_locked_holders_own_unit::ALockedHoldersUnit;
use crate::service::systemd_dbus::{
    Bus,
    CallError,
};
use std::path::Path;
use std::time::{
    Duration,
    Instant,
};

/// What:
///  The word a report and a receipt's failed step carry for a stop that did not
///  end its holder.
/// Why:
///  Every one of the four answers ends the same way for the removal,
///  and the word is the forced removal's own vocabulary,
///  which this folder
///  spells here so a report can name it without depending on that folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HOLDER_LOCKED_WORD = 'holder-locked';
/// ```
pub const HOLDER_LOCKED_WORD: &str = "holder-locked";

/// What:
///  How many times the wait looks at the unit before it gives up.
/// Why:
///  The design gives the wait no value and says what it waits for,
///  so the wait is divided into this many looks:
///  one look would read a stop that is still going,
///  and a stop that ends inside the wait is seen on the look that follows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const POLLS_IN_ONE_STOP_WAIT = 20;
/// ```
pub const POLLS_IN_ONE_STOP_WAIT: u32 = 20;

/// What:
///  How many bytes of a task's control-group file the wait reads.
/// Why:
///  One line per attached hierarchy is what the kernel writes and the unified
///  line is the longest of them,
///  so this bound refuses a name another process swapped for a large file
///  rather than reading it into this process's memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_TASKS_CONTROL_GROUP_LIMIT = 64 * 1024;
/// ```
pub const A_TASKS_CONTROL_GROUP_LIMIT: u64 = 64 * 1024;

/// What:
///  Everything the stop of one locked holder's unit names:
///  the unit,
///  its control-group path,
///  the task's own control-group file and namespace,
///  and the namespace's kernel identity.
/// Why:
///  The manager is asked for the unit,
///  the wait reads the task's control-group file and the namespace's
///  descriptor,
///  and the journal line names the unit,
///  its control group and the namespace that goes with it,
///  so all five travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheUnitsStop = { unit; cgroup; tasksCgroup; namespace; identity };
/// ```
pub struct TheUnitsStop<'a> {
    /// The unit's name as the manager spells it.
    pub unit: &'a str,
    /// The unit's control-group path,
    /// as the task's line printed it.
    pub cgroup: &'a str,
    /// The task's own `/proc/<pid>/cgroup`.
    pub tasks_cgroup: &'a Path,
    /// The task's own `/proc/<pid>/ns/mnt`.
    pub namespace: &'a Path,
    /// The mount namespace's kernel identity,
    /// read before the stop.
    pub identity: &'a str,
}

/// What:
///  One unit this removal stopped,
///  in the three values the journal line carries.
/// Why:
///  The person has to know which unit was stopped and with which namespace the
///  copy went,
///  so the answer is those three and nothing a caller has to join.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoppedUnit = { unit; cgroup; namespace };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoppedUnit {
    /// The unit's name.
    pub unit: String,
    /// The unit's control-group path.
    pub cgroup: String,
    /// The mount namespace that went with it.
    pub namespace: String,
}

/// What:
///  Why a locked holder's unit was not stopped.
/// Why:
///  A person's next act differs by cause:
///  a bus with no manager is repaired by starting the manager,
///  a refused stop by reading the manager's own words,
///  and a unit that stayed or came back by stopping it themselves,
///  so each answer keeps its own fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnitNotStopped = { kind: 'the-bus-refused'; said } | { kind: 'the-stop-refused'; unit; said } | { kind: 'the-unit-stayed'; unit; said } | { kind: 'the-unit-came-back'; unit; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnitNotStopped {
    /// The account's own bus could not be reached or carries no user manager.
    TheBusRefused {
        /// What the bus answered.
        said: String,
    },
    /// The manager answered the stop with a failure.
    TheStopRefused {
        /// The unit it was asked for.
        unit: String,
        /// What the manager answered.
        said: String,
    },
    /// A task is still in the unit's control group,
    /// or the namespace still exists,
    /// when the wait ends.
    TheUnitStayed {
        /// The unit it was asked for.
        unit: String,
        /// What the wait still found.
        said: String,
    },
    /// The unit is active again,
    /// which a unit with `Restart=` gives.
    TheUnitCameBack {
        /// The unit it was asked for.
        unit: String,
        /// The state the manager reported.
        said: String,
    },
}

impl UnitNotStopped {
    /// What:
    ///  The receipt's failed-step word and the report's word for this answer.
    /// Why:
    ///  Every one of the four ends the same way for the removal:
    ///  nothing was unmounted and the clone stays where it was,
    ///  which is the design's word for a locked holder.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = answer.word(); // 'holder-locked'
    /// ```
    pub fn word(&self) -> &'static str {
        return HOLDER_LOCKED_WORD;
    }

    /// What:
    ///  What the manager or the wait answered,
    ///  as the refusal's sentence carries it.
    /// Why:
    ///  The sentence `a_locked_holder` repeats the read's own words after the
    ///  clause,
    ///  so a person reads why their manager did not end the holder.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const said = answer.said(); // 'Unit forced-act-holder.service not found.'
    /// ```
    pub fn said(&self) -> String {
        return match self {
            UnitNotStopped::TheBusRefused {
                said,
            }
            | UnitNotStopped::TheStopRefused {
                said,
                ..
            }
            | UnitNotStopped::TheUnitStayed {
                said,
                ..
            }
            | UnitNotStopped::TheUnitCameBack {
                said,
                ..
            } => said.clone(),
        };
    }
}

/// What:
///  What the reads after the wait found:
///  the manager's own state for the unit,
///  the task's control-group line,
///  and whether the mount namespace still exists.
/// Why:
///  The three are the design's whole answer to "did the stop end the holder",
///  and taking them as values is what lets a control show each verdict without
///  a manager and without a mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LeftBehind = { state?: string; tasksCgroup?: string; namespace: boolean };
/// ```
pub struct LeftBehind<'a> {
    /// The manager's `ActiveState`,
    /// or nothing when it holds no unit of that name.
    pub state: Option<&'a str>,
    /// The task's whole control-group file,
    /// or nothing when the task has gone.
    pub tasks_cgroup: Option<&'a str>,
    /// Whether the mount namespace's path still names something.
    pub namespace: bool,
}

/// What:
///  Whether the reads after the wait say the unit and its copy are gone.
/// Why:
///  The design names three answers that still refuse and one that goes on,
///  and the order here is the order a person reads them in:
///  a unit that is active again is named as that,
///  because a restarted unit's control group is back and the state says why.
///
/// # Errors
/// The unit is active again,
/// a task is still in its control group,
/// or the namespace still exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theStopsVerdict('watcher.service', { state: undefined, tasksCgroup: undefined, namespace: false }); // ok
/// ```
pub fn the_stops_verdict(
    unit: &str,
    left: &LeftBehind<'_>,
) -> Result<(), UnitNotStopped> {
    // What: the states the manager reports for a unit that is running or becoming so.
    // Why:  a unit with `Restart=` is back in one of these, and that is not a stop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const running = ['active', 'activating', 'reloading'];
    // ```
    const RUNNING: &[&str] = &["active", "activating", "reloading"];
    if let Some(state) = left.state
        && RUNNING.contains(&state)
    {
        return Err(UnitNotStopped::TheUnitCameBack {
            unit: String::from(unit),
            said: format!("the manager reports the unit {state} again"),
        });
    }
    // What: a line that still names this unit, which only a task in it prints.
    // Why:  that is exactly what the design waits for the line to stop saying.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (left.tasksCgroup?.split('\n').some((line) => line.endsWith(`/${unit}`))) throw ...
    // ```
    if let Some(text) = left.tasks_cgroup
        && text
            .lines()
            .any(|line| return line.ends_with(&format!("/{unit}")))
    {
        return Err(UnitNotStopped::TheUnitStayed {
            unit: String::from(unit),
            said: String::from("a task is still in the unit's control group when the wait ends"),
        });
    }
    // What: whether the namespace's own path still names something.
    // Why:  the copy goes with the namespace, so a namespace that is still there is a holder this stop did not end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (left.namespace) throw 'the unit stayed';
    // ```
    if left.namespace {
        return Err(UnitNotStopped::TheUnitStayed {
            unit: String::from(unit),
            said: String::from("the holder's mount namespace is still there when the wait ends"),
        });
    }
    return Ok(());
}

/// What:
///  Stop one locked holder's unit through the account's own manager and wait
///  for its tasks,
///  its control group and its namespace to go.
/// Why:
///  The manager's stop call returns when the job is queued,
///  so only the reads that follow it say the holder is gone,
///  and every one of them is bounded by the wait the caller hands in.
///
/// # Errors
/// The bus refused,
/// the manager refused the stop,
/// or the unit stayed or came back inside the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopped = stopTheUnit(busPath, the, wait);  // throws UnitNotStopped
/// ```
pub fn stop_the_unit(
    bus_socket: &Path,
    the: &TheUnitsStop<'_>,
    wait: Duration,
) -> Result<StoppedUnit, UnitNotStopped> {
    // What: the connection to the account's own session bus, bounded by the wait.
    // Why:  a bus that accepts and says nothing must not hold the removal past its own bound.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bus = Bus.connect(busSocket, wait);
    // ```
    let mut bus: Bus = Bus::connect(bus_socket, wait).map_err(|error| {
        return UnitNotStopped::TheBusRefused {
            said: said_of(&error),
        };
    })?;
    // What: the manager's own stop of that unit, answered once the job is queued.
    // Why:  this is the one act the owner's answer allows: the manager ends the unit's jobs its own way.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // bus.stopUnit(the.unit);
    // ```
    bus.stop_unit(the.unit).map_err(|error| {
        return UnitNotStopped::TheStopRefused {
            unit: String::from(the.unit),
            said: said_of(&error),
        };
    })?;
    // What: the time the reads that follow may take.
    // Why:  the connection spent part of the wait, and the wait is about the unit, not about the connect.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // bus.within(wait);
    // ```
    bus.within(wait);
    // What: when the wait ends.
    // Why:  a stop that has not finished by then is the design's own refusal, not an endless wait.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const deadline = Date.now() + wait;
    // ```
    let deadline: Instant = Instant::now() + wait;
    // What: how long one look is followed by the next.
    // Why:  the wait is divided evenly, so the last look lands inside it whatever the host's own speed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pause = wait / POLLS_IN_ONE_STOP_WAIT;
    // ```
    let pause: Duration = wait / POLLS_IN_ONE_STOP_WAIT;
    // What: the last answer the reads gave, which the refusal names when the wait ends.
    // Why:  the wait's end must not lose the reason it waited.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let notYet: UnitNotStopped;
    // ```
    let not_yet: UnitNotStopped = loop {
        // What: the manager's own state for the unit, which answers nothing for one it unloaded.
        // Why:  the design waits for the manager to report the unit inactive.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const state = bus.activeState(the.unit); // string | undefined
        // ```
        let state: Option<String> = match bus.active_state(the.unit) {
            Ok(answer) => answer,
            Err(error) => {
                return Err(UnitNotStopped::TheUnitStayed {
                    unit: String::from(the.unit),
                    said: format!(
                        "the manager did not answer for the unit inside the wait: {}",
                        said_of(&error)
                    ),
                });
            }
        };
        // What: the task's whole control-group file, or nothing when the task has gone.
        // Why:  the design's wait is for the unit's tasks to leave this file, and the read is the bounded one a path
        //       another process controls is read through (`AGENTS.md`'s rule for a file a person or a program
        //       controls: opened so it cannot block, with a size bound).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const tasksCgroup = readPlainTextAt(the.tasksCgroup, A_TASKS_CONTROL_GROUP_LIMIT);  // ENOENT → undefined
        // ```
        let tasks_cgroup: Option<String> =
            match read_plain_text_at(the.tasks_cgroup, A_TASKS_CONTROL_GROUP_LIMIT) {
                Ok(text) => Some(text),
                Err(_gone) => None,
            };
        // What: the three reads the verdict is made of.
        // Why:  one value keeps the verdict a plain function of what was found.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const left = { state, tasksCgroup, namespace: pathExists(the.namespace) };
        // ```
        let left: LeftBehind<'_> = LeftBehind {
            state: state.as_deref(),
            tasks_cgroup: tasks_cgroup.as_deref(),
            namespace: the.namespace.exists(),
        };
        match the_stops_verdict(the.unit, &left) {
            Ok(()) => {
                return Ok(StoppedUnit {
                    unit: String::from(the.unit),
                    cgroup: String::from(the.cgroup),
                    namespace: String::from(the.identity),
                });
            }
            Err(not_yet) => {
                // What: the wait's end, which hands the last answer to the caller.
                // Why:  a stop that has not finished inside the wait is the design's own refusal.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (Date.now() >= deadline) break notYet;
                // ```
                if Instant::now() >= deadline {
                    break not_yet;
                }
            }
        }
        std::thread::sleep(pause);
    };
    return Err(not_yet);
}

/// What:
///  Stop every locked holder's unit the preflight recorded,
///  in the order it recorded them.
/// Why:
///  The design puts the locked units first in the structural stage,
///  each stopped and its namespace waited for before another namespace is
///  changed,
///  so one call answers what the act does with the plan's whole list.
///
/// # Errors
/// The first unit that was not stopped,
/// with what the manager or the wait answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopped = stopEveryLockedUnit(busPath, plan.unitsToStop, wait);
/// ```
pub fn stop_every_locked_unit(
    bus_socket: &Path,
    units: &[ALockedHoldersUnit],
    wait: Duration,
) -> Result<Vec<StoppedUnit>, UnitNotStopped> {
    // What: the units stopped so far, one value per unit the manager ended.
    // Why:  the journal writes one line per entry, and a stop that refuses keeps the ones before it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stopped: StoppedUnit[] = [];
    // ```
    let mut stopped: Vec<StoppedUnit> = Vec::new();
    for one in units {
        // What: the task's own control-group file, which the wait reads.
        // Why:  the design waits for the unit's tasks to leave this file.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const tasksCgroup = `/proc/${one.task}/cgroup`;
        // ```
        let tasks_cgroup: String = format!("/proc/{}/cgroup", one.task);
        // What: the task's own namespace descriptor, which the wait reads.
        // Why:  a namespace that is still there is a copy that did not go with the unit.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const namespace = `/proc/${one.task}/ns/mnt`;
        // ```
        let namespace: String = format!("/proc/{}/ns/mnt", one.task);
        // What: everything that one stop names, in one value.
        // Why:  the stop takes one holder at a time and cannot pair two holders' fields.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const the = { unit: one.unit, cgroup: one.cgroup, tasksCgroup, namespace, identity: one.namespace };
        // ```
        let the: TheUnitsStop<'_> = TheUnitsStop {
            unit: one.unit.as_str(),
            cgroup: one.cgroup.as_str(),
            tasks_cgroup: Path::new(tasks_cgroup.as_str()),
            namespace: Path::new(namespace.as_str()),
            identity: one.namespace.as_str(),
        };
        stopped.push(stop_the_unit(bus_socket, &the, wait)?);
    }
    // What: every unit stopped, in the plan's order.
    // Why:  the caller writes one journal line per entry and tells the person to start each again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return stopped;
    // ```
    return Ok(stopped);
}

/// What:
///  What a failed exchange with the bus or the manager said,
///  in the words a person reads.
/// Why:
///  A refusal that said only "the stop failed" would leave the person guessing
///  whether the manager,
///  the bus or the unit answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = saidOf(error); // 'org.freedesktop.systemd1.NoSuchUnit: ...'
/// ```
fn said_of(error: &CallError) -> String {
    return match error {
        CallError::Connection(said) => format!("the session bus could not be reached: {said}"),
        CallError::Authentication(said) => {
            format!("the session bus refused this account's login: {said}")
        }
        CallError::TimedOut => String::from("the bus did not answer in the time it was given"),
        CallError::Manager {
            name,
            text,
        } => format!("the manager answered {name}: {text}"),
        CallError::Malformed(said) => format!("the bus answered something else: {said}"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "a_service_unit_stopped_through_the_accounts_manager_tests.rs"]
mod tests;
