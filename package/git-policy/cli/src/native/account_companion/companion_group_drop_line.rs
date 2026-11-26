//! What:
//!  The line `doctor` prints about the account companion's group drop:
//!  the skip,
//!  the settings the host read,
//!  why Ubuntu 24.04 answers that way,
//!  and what a person can do about it.
//! Why:
//!  The owner asked for it with the skip itself:
//!  "Add the skip and make the doctor tell user what to do."
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "The account companion's group drop on Ubuntu 24.04").
//!  An add that skips prints one line of its own and goes ahead,
//!  so a host that loses the companion's check could pass for a host that
//!  passed it;
//!  this line is where a person learns otherwise without running an add.
//!  It prints only for the skip.
//!  A host that wrote the `deny` needs no line,
//!  and a host that answered anywhere else has that answer in the add's own
//!  line,
//!  so `doctor` neither repeats it nor predicts it.
//!  The remedies are the troubleshooting record's,
//!  each measured in a guest before it was written here
//!  (`doc/troubleshooting/ubuntu-apparmor-userns-restriction-refuses-the-zfs-account-companion.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = groupDropLine(); // '' when the host wrote the deny, or a report of four lines
//! ```

use super::companion_calls::CompanionCall;
use super::companion_deadline::{
    COMPANION_DEADLINE,
    COMPANION_DEADLINE_MARGIN,
};
use super::companion_group_drop_probe::{
    GroupDrop,
    the_group_drop,
};
use super::companion_settings::{
    self,
    Settings,
};
use crate::work_awaited_within_a_ceiling::work_within;
use std::time::Duration;

/// What every line of this report starts with.
/// Why:
///  The doctor's other sections start with their own prefix,
///  so a reader and a control can tell which part of the report spoke.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PREFIX = 'account companion:';
/// ```
pub const GROUP_DROP_PREFIX: &str = "account companion:";

/// What:
///  How long `doctor` waits for the probe's thread before it says the answer
///  could not be read.
/// Why:
///  The probe bounds its own child by the companion's measured deadline,
///  and this ceiling adds the margin that deadline was measured with,
///  so the probe's own bound is the one that decides and the thread's ceiling
///  only covers a thread that never started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CEILING = COMPANION_DEADLINE + COMPANION_DEADLINE_MARGIN;
/// ```
pub const GROUP_DROP_CEILING: Duration =
    COMPANION_DEADLINE.saturating_add(COMPANION_DEADLINE_MARGIN);

/// What:
///  One read of this host's group drop,
///  as a function pointer,
///  so a control gives its own answer and forks nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GroupDropRead = () => GroupDrop;
/// ```
pub type GroupDropRead = fn() -> GroupDrop;

/// What:
///  The doctor's report about the group drop on this host.
/// Why:
///  This is the read the doctor's section makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = groupDropLine();
/// ```
pub fn group_drop_line() -> String {
    return group_drop_line_with(the_group_drop, GROUP_DROP_CEILING);
}

/// What:
///  The same report with the read and the ceiling given.
/// Why:
///  The read runs on a thread of its own,
///  because `doctor`'s report is built inside the process's runtime,
///  and an answer that does not arrive within the ceiling becomes one line that
///  says so rather than a report that silently left the section out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = workWithin(ceiling, read) ?? `${PREFIX} the group drop could not be checked within ...`;
/// ```
pub fn group_drop_line_with(
    read: GroupDropRead,
    ceiling: Duration,
) -> String {
    let Some(group_drop) = work_within(ceiling, read) else {
        return format!(
            "{GROUP_DROP_PREFIX} the group drop could not be checked: its probe did not answer within {ceiling:?}\n"
        );
    };
    let settings: Settings = companion_settings::read();
    return report_of(group_drop, &settings);
}

/// What:
///  The words one answer prints,
///  which are none for every answer but the skip and a probe that could not be
///  read.
/// Why:
///  The wording is one function of the answer and the settings,
///  so a control holds each of the four answers without this host's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = reportOf({ kind: 'skipped', errno: 13 }, settings); // four lines
/// ```
pub fn report_of(
    group_drop: GroupDrop,
    settings: &Settings,
) -> String {
    return match group_drop {
        GroupDrop::Written => String::new(),
        GroupDrop::Answered {
            ..
        } => String::new(),
        GroupDrop::Skipped {
            errno,
        } => skipped_report(errno, settings),
        GroupDrop::Unread {
            reason,
        } => format!("{GROUP_DROP_PREFIX} the group drop could not be checked: {reason}\n"),
    };
}

/// What:
///  The four lines the skip prints:
///  what happened,
///  the settings this host read,
///  why Ubuntu 24.04's profile answers there,
///  and what a person can do.
/// Why:
///  The remedy line is the troubleshooting record's measured one,
///  and the sentence about doing nothing is the shipped skip itself,
///  so both are facts and neither is a maybe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${PREFIX} this host skips ... EACCES ...` + `${PREFIX} ${settings.line()}` + why + remedy
/// ```
fn skipped_report(
    errno: i32,
    settings: &Settings,
) -> String {
    let name: &str = CompanionCall::OpenSetgroups.name();
    let word: String = CompanionCall::errno_word(errno);
    return format!(
        "{prefix} this host skips the account companion's group drop: {name} answered {word} (errno {errno}), so every managed ZFS add goes ahead unproved by that check.\n\
{prefix} {settings}\n\
{prefix} the open asks CAP_SYS_ADMIN inside the new user namespace, and Ubuntu 24.04's default AppArmor profile for unprivileged user namespaces (unprivileged_userns) denies every capability inside one, so the kernel answers before the deny write.\n\
{prefix} nothing needs doing, or turn that restriction off for the whole machine with sudo sysctl --write kernel.apparmor_restrict_unprivileged_userns=0, which lasts until the next boot and removes a distribution security default from every unprivileged program.\n",
        prefix = GROUP_DROP_PREFIX,
        settings = settings.line()
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_group_drop_line_tests.rs"]
mod tests;
