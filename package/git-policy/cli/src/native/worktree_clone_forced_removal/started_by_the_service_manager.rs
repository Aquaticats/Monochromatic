//! What:
//!  The refusal of a `--force` removal the service manager started:
//!  how the manager is recognised as a parent,
//!  the two places it is looked for,
//!  and the words the refusal prints.
//! Why:
//!  The project's fourth answer refuses `--force` outright when the command's
//!  parent is the service manager,
//!  and its "One more answer" refuses it also when the command's session was
//!  started directly by the manager,
//!  so a unit whose main process is a script is caught too
//!  (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
//!  "Which processes a forced removal never ends" and
//!  "One more answer: a unit whose main process is a script").
//!  The reason is that nothing tells the process that asked for the removal from
//!  the manager that started it,
//!  and an ending there would reach whatever the unit holds.
//!  The recognition reads the kernel's own data and not a name an account could
//!  satisfy with a look-alike,
//!  and a session leader whose own record cannot be read refuses too,
//!  because the fourth answer's words do not cover an orphan and refusing is the
//!  safe reading (the design's "A command whose parent is the service manager, or
//!  whose session was started by it").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = startedByTheServiceManager(reads);  // undefined when nothing matched
//! ```

use crate::worktree_clone_forced_removal::process_session_record::CommandWord;

/// The first process of the machine.
pub const THE_FIRST_PROCESS: i32 = 1;

/// The name the service manager answers to in `/proc/<pid>/comm`,
/// both as the system manager and as a user manager.
pub const MANAGER_COMMAND: &str = "systemd";

/// What:
///  Which manager started the command,
///  which the refusal's sentence names.
/// Why:
///  A person reading the sentence has to know whether the process above the
///  command is the system manager or their own,
///  because the remedy (run it from the process that wants it) is the same and
///  the thing to stop is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagerKind = 'system' | 'user';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagerKind {
    /// The machine's service manager,
    /// the process with number one.
    System,
    /// The account's own manager,
    /// the process the account's `user@<uid>.service` runs.
    User,
}

impl ManagerKind {
    /// The words the refusal's sentence uses.
    pub fn words(self) -> &'static str {
        return match self {
            ManagerKind::System => "systemd",
            ManagerKind::User => "systemd --user",
        };
    }
}

/// What:
///  Which of the two places the manager was found in,
///  and what the sentence says about it.
/// Why:
///  A command run as a service unit has the manager above it,
///  and a script the manager started has the manager above the session leader
///  and not above the command,
///  so the two cases read differently to the person who has to fix it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagerPlace = { kind: 'parent' | 'session-leader-parent' | 'orphan'; pid?; manager? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagerPlace {
    /// The command's own parent is the manager.
    Parent {
        /// The manager's process number.
        pid: i32,
        /// Which manager it is.
        manager: ManagerKind,
    },
    /// The command's session leader's parent is the manager.
    SessionLeaderParent {
        /// The manager's process number.
        pid: i32,
        /// Which manager it is.
        manager: ManagerKind,
    },
    /// The session leader's own record could not be read,
    /// or its parent has gone,
    /// so nothing proves what started the session.
    Orphan {
        /// The session leader's process number.
        leader: i32,
    },
}

/// What:
///  The refusal of a command the service manager started.
/// Why:
///  The wrapper exits before it writes either property,
///  so what the value carries is the manager and the place,
///  which is what the sentence prints,
///  and the exit status the caller answers with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagerRefusal = { place: ManagerPlace };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagerRefusal {
    /// Where the manager was found,
    /// or that the session leader could not be proved.
    pub place: ManagerPlace,
}

/// What:
///  The reads the check needs,
///  already made,
///  so the decision is one function a control can drive.
/// Why:
///  A parent, its command and its cgroup are three reads of two files,
///  and keeping them as values lets the control show every shape the fourth
///  answer names without starting a manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagerReads = { uid; parentPid; parentCommand?; parentCgroup?; sessionLeaderPid; leaderRecord?; leaderParentCommand?; leaderParentCgroup? };
/// ```
pub struct ManagerReads<'a> {
    /// The account whose user manager is asked about.
    pub uid: u32,
    /// The command's recorded parent.
    pub parent_pid: i32,
    /// The parent's own `comm`,
    /// or nothing when its record could not be read.
    pub parent_command: Option<&'a str>,
    /// The parent's cgroup path from `/proc/<pid>/cgroup`,
    /// or nothing when it could not be read.
    pub parent_cgroup: Option<&'a str>,
    /// The command's session leader,
    /// from `getsid(0)`.
    pub session_leader_pid: i32,
    /// The leader's own recorded parent,
    /// or nothing when the leader's record could not be read.
    pub leader_parent: Option<i32>,
    /// The leader's parent's `comm`.
    pub leader_parent_command: Option<&'a str>,
    /// The leader's parent's cgroup path.
    pub leader_parent_cgroup: Option<&'a str>,
}

/// What:
///  Whether one process is a service manager of the account,
///  and which one.
/// Why:
///  The system manager is the process with number one and needs no second read,
///  and the user manager is recognised by its own cgroup path,
///  which a user process cannot make because that path lies in a slice only the
///  manager's units run under,
///  and by its own `comm`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (pid === 1) return 'system';
/// if (command === 'systemd' && cgroup === `/user.slice/user-${uid}.slice/user@${uid}.service/init.scope`) return 'user';
/// return undefined;
/// ```
pub fn manager_of(
    pid: i32,
    command: Option<&str>,
    cgroup: Option<&str>,
    uid: u32,
) -> Option<ManagerKind> {
    if pid == THE_FIRST_PROCESS {
        return Some(ManagerKind::System);
    }
    if command != Some(MANAGER_COMMAND) {
        return None;
    }
    let path: &str = cgroup?;
    let user_manager_init: String =
        format!("/user.slice/user-{uid}.slice/user@{uid}.service/init.scope");
    if path == user_manager_init {
        return Some(ManagerKind::User);
    }
    return None;
}

/// What:
///  Read one `/proc/<pid>/cgroup` text as the unified hierarchy's own path.
/// Why:
///  The file prints one line per hierarchy,
///  and the unified one is the line whose hierarchy number is `0` and whose
///  controllers are empty:
///  `0::<path>`.
///  A line of another shape names another hierarchy and proves nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').find(line => line.startsWith('0::'))?.slice(3)
/// ```
pub fn unified_cgroup_path(text: &str) -> Option<String> {
    for line in text.split('\n') {
        let Some(path) = line.strip_prefix("0::") else {
            continue;
        };
        if path.is_empty() {
            return None;
        }
        return Some(String::from(path));
    }
    return None;
}

/// What:
///  Decide whether a `--force` removal must be refused because the service
///  manager started the command,
///  or the session the command runs in.
/// Why:
///  The check runs in the wrapper before it writes the force property,
///  so a refusal here changes nothing at all:
///  no property,
///  no intent,
///  no request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = managerOf(reads.parentPid, reads.parentCommand, reads.parentCgroup, reads.uid);
/// if (parent) return refusalOf({ kind: 'parent', pid: reads.parentPid, manager: parent });
/// const leader = managerOf(reads.leaderParent, ...);
/// if (leader) return refusalOf({ kind: 'session-leader-parent', ... });
/// if (reads.leaderParent === undefined) return refusalOf({ kind: 'orphan', leader: reads.sessionLeaderPid });
/// ```
pub fn started_by_the_service_manager(reads: &ManagerReads<'_>) -> Option<ManagerRefusal> {
    if let Some(manager) = manager_of(
        reads.parent_pid,
        reads.parent_command,
        reads.parent_cgroup,
        reads.uid,
    ) {
        return Some(ManagerRefusal {
            place: ManagerPlace::Parent {
                pid: reads.parent_pid,
                manager,
            },
        });
    }
    let Some(leader_parent) = reads.leader_parent else {
        // The session leader's own record was refused or has gone: nothing proves what started the session.
        return Some(ManagerRefusal {
            place: ManagerPlace::Orphan {
                leader: reads.session_leader_pid,
            },
        });
    };
    if let Some(manager) = manager_of(
        leader_parent,
        reads.leader_parent_command,
        reads.leader_parent_cgroup,
        reads.uid,
    ) {
        return Some(ManagerRefusal {
            place: ManagerPlace::SessionLeaderParent {
                pid: leader_parent,
                manager,
            },
        });
    }
    return None;
}

/// What:
///  The refusal's own sentence,
///  with the manager, the place and the remedy filled in.
/// Why:
///  The design fixes this text for the command and the session leader's parent
///  and for an orphan,
///  and a person reads it to learn that nothing changed and what to run instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.error(theRefusalText(slot, refusal));
/// ```
pub fn the_refusal_text(
    slot: &str,
    refusal: &ManagerRefusal,
) -> String {
    let place: String = match &refusal.place {
        ManagerPlace::Parent {
            pid,
            manager,
        } => {
            format!("its parent is the manager (pid {pid}, {})", manager.words())
        }
        ManagerPlace::SessionLeaderParent {
            pid,
            manager,
        } => {
            format!(
                "its session leader's parent is the manager (pid {pid}, {})",
                manager.words()
            )
        }
        ManagerPlace::Orphan {
            leader,
        } => {
            format!(
                "its session leader (pid {leader}) has no record this command could read, so nothing proves what started the session"
            )
        }
    };
    return format!(
        "the removal of {slot} was asked to end the holders, and this command was started by the service manager: {place}, which cannot be told apart from the process that asked for it. If you did not start this removal as a service unit, the command was reparented to the manager because its own parent exited, or it runs in a console login's session; nothing was changed and nothing was ended: the clone, its snapshots and the Git registration are as they were. Run the removal as a child of the process that wants it: from that process's own shell, or under `systemd-run --user --scope` started by that shell; in a console login, start it under `setsid --wait` so its own session begins at the command. Do not run it as a service unit.\n"
    );
}

/// What:
///  The command name a refusal prints,
///  cut to the kernel's bound,
///  or the sentence's own word for a record it could not read.
/// Why:
///  The sentence prints `(<command>)` beside a number,
///  and the escape for that parenthesis belongs to the sentence that writes it,
///  so this hands the sentence the kernel's bytes and no escape of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = commandWordOf(comm).text();
/// ```
pub fn command_word(comm: Option<&str>) -> CommandWord {
    let Some(name) = comm else {
        return CommandWord::saying("a command this wrapper could not read");
    };
    return CommandWord::of(name);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "started_by_the_service_manager_tests.rs"]
mod tests;
