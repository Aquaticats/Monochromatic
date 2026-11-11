//! Watches the user's session bus for a process that is not the user's manager owning
//! `org.freedesktop.systemd1`,
//! and removes it.
//! Why this exists:
//! on 2026-11-11 a nested `systemd --user` that an agent's probe started asked for that name,
//! with replacement allowed,
//! and the real manager never asks for it back.
//! For hours every `systemctl --user` call and every watcher call went to the impostor,
//! and a container build inside it ran under its one-CPU,
//! 256-task scopes.
//! With the watcher calling the manager over its private socket,
//! no impostor can misdirect the watcher any more;
//! this guard keeps an impostor from misdirecting everything else on the machine,
//! which the owner asked for in the same words:
//! "Make sure no imposer can never succeed again."
//! An impostor is killed with `SIGKILL` and not `SIGTERM`,
//! because an impostor manager handles signals by stopping the units it started,
//! which would take those units with it.
//! The real manager is then asked to take the name again (`daemon-reexec`),
//! at most once in a few minutes,
//! because a re-execution is a heavy thing to repeat in a loop.
//! The whole guard runs as the watched user,
//! because a user bus refuses a connection from root,
//! and because the impostor is that user's own process.

/// The real steps the guard performs.
#[path = "impostor_guard_steps.rs"]
pub mod steps;
/// The guard as the watcher starts it.
#[path = "impostor_guard_child.rs"]
pub mod child;
/// The real steps,
/// re-exported where the program uses them.
pub use steps::RealSteps;
/// Starting the guard,
/// its command and its setup,
/// re-exported where the watcher uses them.
pub use child::{GUARD_COMMAND, GuardSetup, run_child, spawn};

/// The bus name that must belong to the user's manager and to nothing else.
pub const MANAGER_BUS_NAME: &str = "org.freedesktop.systemd1";
/// The word a loud guard line carries,
/// which the watcher looks for so the person,
/// the journal and the log all see it.
pub const LOUD_MARKER: &str = "MANAGER BUS GUARD";
/// How often the owner is checked when no event arrives.
pub const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);
/// The least time between two re-executions of the real manager.
pub const REEXEC_GAP: std::time::Duration = std::time::Duration::from_secs(5 * 60);

/// The signal the guard asks the bus daemon for.
pub fn match_rule(name: &str) -> String {
    return format!("type='signal',sender='org.freedesktop.DBus',member='NameOwnerChanged',arg0='{name}'");
}

/// What the bus says about the name's owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ownership {
    /// The name belongs to that process.
    Owner(
        /// Its pid.
        i32,
    ),
    /// No process holds the name.
    Nobody,
    /// The bus could not be asked,
    /// or answered something this guard does not read.
    Unknown,
}

/// What the guard decided to do.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// Nothing:
    /// the name belongs to the manager,
    /// or the bus could not be asked.
    Nothing,
    /// Kill the process that holds the name,
    /// and ask the manager to take it again when the gap since the last re-execution allows.
    KillImpostor {
        /// The process that holds the name.
        pid: i32,
        /// Whether the manager is asked to take the name again now.
        reexec: bool,
    },
    /// No process holds the name,
    /// so the manager is asked to take it again when the gap allows.
    Reexec {
        /// Whether the manager is asked now.
        ask: bool,
    },
}

/// The guard's memory between checks.
pub struct GuardMemory {
    /// When the manager was last asked to re-execute.
    pub last_reexec: Option<std::time::Instant>,
    /// The least time between two re-executions.
    pub gap: std::time::Duration,
}

/// Deciding,
/// without touching anything.
impl GuardMemory {
    /// Builds the memory of a guard that has not re-executed anything yet.
    pub fn new(gap: std::time::Duration) -> GuardMemory {
        return GuardMemory { last_reexec: None, gap };
    }

    /// Whether a re-execution may be asked for at `now`.
    fn may_reexec(&self, now: std::time::Instant) -> bool {
        return match self.last_reexec {
            Some(last) => now.duration_since(last) >= self.gap,
            None => true,
        };
    }

    /// What to do about the ownership the bus reported,
    /// for the manager whose main process is `expected`.
    pub fn decide(&mut self, expected: i32, ownership: Ownership, now: std::time::Instant) -> Action {
        return match ownership {
            Ownership::Owner(pid) if pid == expected => Action::Nothing,
            Ownership::Owner(pid) => {
                let reexec = self.may_reexec(now);
                if reexec {
                    self.last_reexec = Some(now);
                }
                Action::KillImpostor { pid, reexec }
            }
            Ownership::Nobody => {
                let ask = self.may_reexec(now);
                if ask {
                    self.last_reexec = Some(now);
                }
                Action::Reexec { ask }
            }
            Ownership::Unknown => Action::Nothing,
        };
    }
}

/// The steps the guard performs,
/// so a test can watch them without a bus or a manager.
pub trait GuardSteps {
    /// The main process of the user's manager.
    fn expected_pid(&self, uid: u32) -> Option<i32>;
    /// Who holds the bus name now.
    fn ownership(&mut self, uid: u32) -> Ownership;
    /// Waits up to `timeout` for the bus to say that the name's owner changed,
    /// and reports whether it did.
    fn wait_for_change(&mut self, uid: u32, timeout: std::time::Duration) -> bool;
    /// Ends the process that holds the name.
    fn kill(&self, pid: i32) -> Result<(), String>;
    /// Asks the manager to take the bus name again.
    fn reexec(&self, uid: u32) -> Result<(), String>;
}

/// One line the guard says about what it did,
/// which the watcher writes to its log,
/// the journal and the person's desktop.
#[derive(Debug, Eq, PartialEq)]
pub struct GuardReport {
    /// Whether this line says that what the person expects is not in force.
    pub loud: bool,
    /// What was done,
    /// without the prefix the report line carries.
    pub text: String,
}

/// The guard that performs the steps.
pub struct Guard<'a> {
    /// The steps.
    pub steps: &'a mut dyn GuardSteps,
    /// What it remembers between checks.
    pub memory: GuardMemory,
}

/// Running one check.
impl Guard<'_> {
    /// Checks the name once at `now` and does what the answer requires.
    /// Every action is reported,
    /// and so is a bus that could not be asked.
    pub fn check(&mut self, uid: u32, now: std::time::Instant) -> Vec<GuardReport> {
        let Some(expected) = self.steps.expected_pid(uid) else {
            return vec![GuardReport {
                loud: true,
                text: format!(
                    "the guard of {MANAGER_BUS_NAME} cannot act: the system manager named no main process of user@{uid}.service, so an impostor that holds the name is not removed"
                ),
            }];
        };
        let ownership = self.steps.ownership(uid);
        let mut reports: Vec<GuardReport> = Vec::new();
        match self.memory.decide(expected, ownership, now) {
            Action::Nothing => {}
            Action::Reexec { ask: false } => {}
            Action::Reexec { ask: true } => {
                reports.push(GuardReport {
                    loud: true,
                    text: format!(
                        "no process holds {MANAGER_BUS_NAME} on the session bus, so every `systemctl --user` call there fails; asking the user's manager (pid {expected}) to take the name again"
                    ),
                });
                match self.steps.reexec(uid) {
                    Ok(()) => reports.push(GuardReport {
                        loud: false,
                        text: format!("the user's manager (pid {expected}) re-executed and takes {MANAGER_BUS_NAME} again"),
                    }),
                    Err(error) => reports.push(GuardReport {
                        loud: true,
                        text: format!("the user's manager (pid {expected}) did not re-execute: {error}; {MANAGER_BUS_NAME} still has no owner"),
                    }),
                }
            }
            Action::KillImpostor { pid, reexec } => {
                reports.push(GuardReport {
                    loud: true,
                    text: format!(
                        "{MANAGER_BUS_NAME} on the session bus is held by pid {pid}, which is not the user's manager (pid {expected}); killing it, because every call to that name, `systemctl --user` included, would go to it instead"
                    ),
                });
                match self.steps.kill(pid) {
                    Ok(()) => reports.push(GuardReport { loud: false, text: format!("killed pid {pid}, which was not the user's manager") }),
                    Err(error) => reports.push(GuardReport { loud: true, text: format!("pid {pid} holds {MANAGER_BUS_NAME} and could not be killed: {error}") }),
                }
                if reexec {
                    match self.steps.reexec(uid) {
                        Ok(()) => reports.push(GuardReport {
                            loud: false,
                            text: format!("the user's manager (pid {expected}) re-executed and takes {MANAGER_BUS_NAME} again"),
                        }),
                        Err(error) => reports.push(GuardReport {
                            loud: true,
                            text: format!("the user's manager (pid {expected}) did not re-execute after pid {pid} was killed: {error}"),
                        }),
                    }
                } else {
                    reports.push(GuardReport {
                        loud: false,
                        text: format!("the user's manager is not re-executed yet: it was re-executed under {REEXEC_GAP:?} ago"),
                    });
                }
            }
        }
        return reports;
    }
}
