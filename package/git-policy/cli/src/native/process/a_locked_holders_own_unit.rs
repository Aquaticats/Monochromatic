//! What:
//!  The reads that tell whether a locked holder runs in a service unit of this
//!  account's own user manager,
//!  and so whether the forced removal may stop that unit and go on.
//! Why:
//!  The owner's answer of 2026-11-23 stops such a unit through that manager
//!  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
//!  "A holder whose copy is locked (V10)"),
//!  and the design says which four reads part the two routes:
//!  the namespace's owner,
//!  the unit's name in the task's control-group line,
//!  the manager's own answer for that name,
//!  and the caller's protected set,
//!  which the preflight reads first
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The locked holder,
//!  and the unit the wrapper may stop").
//!  A locked holder that is not such a unit,
//!  a bubblewrap sandbox or a rootless container started by hand,
//!  has no manager to stop it through and stays refused,
//!  so this module answers why instead of guessing.
//!  The reads are values,
//!  which is what lets a control show every refusal without a manager and
//!  without a lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unit = aLockedHoldersUnit(cgroupLine, ourUid, managersState); // throws NotTheAccountsUnit
//! ```

/// What:
///  The prefix of the one unified control-group line the kernel prints.
/// Why:
///  `/proc/<pid>/cgroup` prints one line per attached hierarchy and the
///  unified one alone names a unit this manager knows,
///  so a line of another hierarchy is not read at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNIFIED_HIERARCHY_PREFIX = '0::';
/// ```
pub const UNIFIED_HIERARCHY_PREFIX: &str = "0::";

/// What:
///  The start of the one segment of that line that names an account's user
///  manager:
///  `user@` followed by the account's number and `.service`.
/// Why:
///  Only a unit inside that segment is one this account's own user manager
///  runs,
///  and a holder of another account or of the system manager is not this case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ACCOUNT_MANAGER_SEGMENT = 'user@';
/// ```
pub const ACCOUNT_MANAGER_SEGMENT: &str = "user@";

/// What:
///  The suffix a segment must end with to be a service unit.
/// Why:
///  A `session-<n>.scope` is the person's own shell or a group of its
///  processes,
///  and a scope is never stopped,
///  because a scope's stop ends whatever its creator put in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SERVICE_UNIT_SUFFIX = '.service';
/// ```
pub const SERVICE_UNIT_SUFFIX: &str = ".service";

/// What:
///  Why a locked holder is not a service unit this account's own user manager
///  may stop.
/// Why:
///  Each answer names a different next act for the person:
///  a holder of another manager is left alone,
///  a scope is left alone,
///  and a unit the manager does not hold is not this account's to stop either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NotTheAccountsUnit = 'not-the-accounts-manager' | 'not-a-service-unit' | 'the-manager-does-not-hold-it';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotTheAccountsUnit {
    /// The task's control-group line runs through no `user@<our number>.service`,
    /// so the holder belongs to another account or to the system manager.
    NotTheAccountsManager,
    /// The last segment is not a service unit,
    /// such as a `session-<n>.scope`.
    NotAServiceUnit,
    /// The account's manager answered for no unit of that name.
    TheManagerDoesNotHoldIt,
}

impl NotTheAccountsUnit {
    /// What:
    ///  The word a report and the refusal's sentence name this answer by.
    /// Why:
    ///  A reader of a refusal asks which of the three it met,
    ///  and the sentence the person reads is `a_locked_holder`'s.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = answer.word(); // 'not-the-accounts-manager' | ...
    /// ```
    pub fn word(self) -> &'static str {
        return match self {
            NotTheAccountsUnit::NotTheAccountsManager => "not-the-accounts-manager",
            NotTheAccountsUnit::NotAServiceUnit => "not-a-service-unit",
            NotTheAccountsUnit::TheManagerDoesNotHoldIt => "the-manager-does-not-hold-it",
        };
    }
}

/// What:
///  One locked holder this arm may stop:
///  its mount namespace,
///  the service unit whose tasks live in it,
///  the control-group path the task's line named,
///  and the task the reads were made of.
/// Why:
///  The stop names the unit to the manager and the journal line names the
///  unit,
///  its control group and the namespace that goes with it,
///  so the three travel together and a caller cannot pair one holder's
///  namespace with another's unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ALockedHoldersUnit = { namespace: string; unit: string; cgroup: string; task: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ALockedHoldersUnit {
    /// The mount namespace's kernel identity,
    /// such as `mnt:[4026532001]`.
    pub namespace: String,
    /// The unit's name as the manager spells it,
    /// such as `watcher.service`.
    pub unit: String,
    /// The unit's control-group path,
    /// as `/proc/<pid>/cgroup` printed it.
    pub cgroup: String,
    /// The task whose line and namespace the reads were made of.
    pub task: i32,
}

/// What:
///  The service unit one task's control-group line names,
///  when that unit runs in this account's own user manager and is a service.
/// Why:
///  A unit outside the account's own manager cannot be stopped by this removal,
///  and a scope must never be stopped,
///  so the read answers a reason rather than a name it may not act on.
///
/// # Errors
/// The line runs through no `user@<our number>.service`,
/// its last segment is not a `.service` unit,
/// or it holds no unified line at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unit = await theUnitOfTheAccountsManager(cgroupLine, 0); // 'watcher.service'
/// ```
pub fn the_unit_of_the_accounts_manager(
    cgroup_line: &str,
    our_uid: u32,
) -> Result<String, NotTheAccountsUnit> {
    // What: the one unified line, found by its prefix; the others name hierarchies this manager does not run.
    // Why:  a line of another hierarchy would name a path no `user@<n>.service` segment begins.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const unified = cgroupLine.split('\n').find((line) => line.startsWith(UNIFIED_HIERARCHY_PREFIX));
    // ```
    let Some(unified) = cgroup_line
        .lines()
        .find(|line| return line.starts_with(UNIFIED_HIERARCHY_PREFIX))
    else {
        return Err(NotTheAccountsUnit::NotTheAccountsManager);
    };
    // What: the control-group path itself, with the prefix taken off.
    // Why:  only the segments after it name mounts' units.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const path = unified.slice(UNIFIED_HIERARCHY_PREFIX.length);
    // ```
    let path: &str = unified
        .strip_prefix(UNIFIED_HIERARCHY_PREFIX)
        .unwrap_or(unified);
    // What: the segment this account's own user manager is named by, built from the number the caller read.
    // Why:  a holder of another account runs under another number's manager and is not this removal's to stop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ours = `${ACCOUNT_MANAGER_SEGMENT}${ourUid}${SERVICE_UNIT_SUFFIX}`;
    // ```
    let ours: String = format!("{ACCOUNT_MANAGER_SEGMENT}{our_uid}{SERVICE_UNIT_SUFFIX}");
    // What: whether any segment of the path is that manager's own unit.
    // Why:  the design refuses a locked holder of another manager before any signal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!path.split('/').includes(ours)) throw 'not-the-accounts-manager';
    // ```
    if !path.split('/').any(|segment| return segment == ours) {
        return Err(NotTheAccountsUnit::NotTheAccountsManager);
    }
    // What: the last segment that holds any text, which is the innermost unit or scope the task lives in.
    // Why:  that is the slice's own name, and only it says whether the task is a service unit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const last = path.split('/').filter((s) => s !== '').at(-1);
    // ```
    let Some(last) = path
        .split('/')
        .filter(|segment| return !segment.is_empty())
        .next_back()
    else {
        return Err(NotTheAccountsUnit::NotTheAccountsManager);
    };
    // What: a scope, a mount unit or a slice answers this, so it is never stopped.
    // Why:  a scope's stop ends whatever its creator put in it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!last.endsWith(SERVICE_UNIT_SUFFIX)) throw 'not-a-service-unit';
    // ```
    if !last.ends_with(SERVICE_UNIT_SUFFIX) {
        return Err(NotTheAccountsUnit::NotAServiceUnit);
    }
    // What: the unit's own name, as the manager spells it.
    // Why:  the manager is asked for exactly this name, so the two reads cannot disagree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return last;
    // ```
    return Ok(String::from(last));
}

/// What:
///  The unit's name,
///  when the account's own manager answered for it.
/// Why:
///  `GetUnit` fails for a name a manager has not loaded,
///  and only a unit that manager holds is one it may stop,
///  so a name it does not hold is refused rather than stopped.
///
/// # Errors
/// The manager answered for no unit of that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unit = theUnitTheManagerHolds('watcher.service', 'active'); // 'watcher.service'
/// ```
pub fn the_unit_the_manager_holds(
    unit: &str,
    its_state: Option<&str>,
) -> Result<String, NotTheAccountsUnit> {
    return match its_state {
        Some(_state) => Ok(String::from(unit)),
        None => Err(NotTheAccountsUnit::TheManagerDoesNotHoldIt),
    };
}

/// What:
///  The whole read from a task's control-group line and the manager's answer
///  to the unit this removal may stop.
/// Why:
///  The preflight makes one decision per locked holder,
///  and one call answering the unit or the reason keeps the two reads in the
///  design's order.
///
/// # Errors
/// The line names no unit of this account's own manager,
/// or that manager answered for no unit of the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unit = aLockedHoldersUnit(line, 0, managerState); // 'watcher.service'
/// ```
pub fn a_locked_holders_unit(
    cgroup_line: &str,
    our_uid: u32,
    its_state: Option<&str>,
) -> Result<String, NotTheAccountsUnit> {
    let unit: String = the_unit_of_the_accounts_manager(cgroup_line, our_uid)?;
    return the_unit_the_manager_holds(unit.as_str(), its_state);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "a_locked_holders_own_unit_tests.rs"]
mod tests;
