//! What:
//!  The report `doctor` prints about the account companion's group drop:
//!  that this host skipped it,
//!  the settings this host read,
//!  why the kernel answered that way,
//!  and what a person can do about it.
//! Why:
//!  The owner asked for it with the skip itself:
//!  "Add the skip and make the doctor tell user what to do."
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "The account companion's group drop on Ubuntu 24.04").
//!  An add that skips prints one line of its own and goes ahead
//!  (`account_companion/companion_lines.rs`),
//!  so a host that lost the companion's check could pass for a host that passed
//!  it;
//!  this report is where a person learns otherwise without running an add.
//!  It prints only for the skip.
//!  A host that wrote the `deny` needs no report,
//!  and a host that answered anywhere else has that answer in the add's own
//!  line,
//!  so `doctor` neither repeats it nor predicts it.
//!  The remedies are the measured ones of
//!  `doc/troubleshooting/ubuntu-apparmor-userns-restriction-refuses-the-zfs-account-companion.md`,
//!  each checked in a throwaway Ubuntu 24.04 guest before it was written here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = groupDropReport(); // '' on a host that wrote the deny, else the four lines
//! ```

use super::group_drop_probe::{
    GroupDrop,
    ProbeCall,
    the_group_drop,
};

/// What:
///  What every line of this report starts with.
/// Why:
///  A reader and a control can tell which part of the doctor's report spoke,
///  as they can for the sections beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PREFIX = 'account companion:';
/// ```
pub const GROUP_DROP_PREFIX: &str = "account companion:";

/// What:
///  The path that names how many user namespaces this host gives.
/// Why:
///  A person seeing the skip needs the three settings that decide it,
///  and this is the first the design names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_USER_NAMESPACES = '/proc/sys/user/max_user_namespaces';
/// ```
const MAX_USER_NAMESPACES: &str = "/proc/sys/user/max_user_namespaces";

/// What:
///  The path a distribution patches in to disable user namespaces wholesale.
/// Why:
///  Where it exists, its value is part of why the companion skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNPRIVILEGED_USERS_CLONE = '/proc/sys/kernel/unprivileged_userns_clone';
/// ```
const UNPRIVILEGED_USERS_CLONE: &str = "/proc/sys/kernel/unprivileged_userns_clone";

/// What:
///  The path Ubuntu 24.04's AppArmor restriction is set through.
/// Why:
///  It is the setting a person can change,
///  and the report names it as the one to look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const APPARMOR_RESTRICTION = '/proc/sys/kernel/apparmor_restrict_unprivileged_userns';
/// ```
const APPARMOR_RESTRICTION: &str = "/proc/sys/kernel/apparmor_restrict_unprivileged_userns";

/// What:
///  The word a setting the running kernel does not have prints.
/// Why:
///  A missing setting must never pass for a satisfied one,
///  so its absence is said rather than left out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ABSENT = 'absent';
/// ```
const ABSENT: &str = "absent";

/// What:
///  The three settings the skip's cause is read from,
///  as this host answered them.
/// Why:
///  The add's own skip line prints them too,
///  and a person comparing the two lines must see the same values.
///  Each is a `String` rather than a `&str`
///  because each is read at run time and outlives the call that read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HostSettings = { maxUserNamespaces: string; unprivilegedUsernsClone: string; apparmorRestriction: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HostSettings {
    /// How many user namespaces this host gives.
    pub max_user_namespaces: String,
    /// The distribution's own switch,
    /// where it has one.
    pub unprivileged_userns_clone: String,
    /// The AppArmor restriction,
    /// where the kernel carries it.
    pub apparmor_restriction: String,
}

/// What:
///  The doctor's report about the group drop on this host.
/// Why:
///  This is what the executable's doctor section calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = groupDropReport();
/// ```
pub fn group_drop_report() -> String {
    return report_of(the_group_drop(), &read_settings());
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
    settings: &HostSettings,
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
///  The four lines a skipped group drop prints:
///  what happened,
///  the settings this host read,
///  why the kernel answers there,
///  and what a person can do.
/// Why:
///  The remedies are the troubleshooting record's measured ones,
///  and the sentence about doing nothing is the shipped skip itself,
///  so both are facts and neither is a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${PREFIX} this host skips the group drop: ...` + settings + why + remedies
/// ```
fn skipped_report(
    errno: i32,
    settings: &HostSettings,
) -> String {
    let name: &str = ProbeCall::OpenSetgroups.name();
    let word: &str = errno_word(errno);
    return format!(
        "{prefix} this host skips the account companion's group drop: {name} answered {word} (errno {errno}), so every managed ZFS add goes ahead unproved by that check.\n\
{prefix} user.max_user_namespaces={max}, kernel.unprivileged_userns_clone={clone}, kernel.apparmor_restrict_unprivileged_userns={restriction}\n\
{prefix} the open asks the kernel for CAP_SYS_ADMIN inside the new user namespace, and Ubuntu 24.04's default AppArmor profile for unprivileged user namespaces (unprivileged_userns) denies every capability inside one, so the kernel refuses the open and the deny write is never reached.\n\
{prefix} nothing needs doing, because every managed ZFS add goes ahead with this skip and prints this same call and number on its own line as it runs; or an administrator turns that restriction off for the whole machine with `sysctl --write kernel.apparmor_restrict_unprivileged_userns=0`, which lasts until the next boot, gives every unprivileged program on the machine the capabilities it asks for inside its own user namespace, and so removes a security default the distribution ships.\n",
        prefix = GROUP_DROP_PREFIX,
        max = settings.max_user_namespaces,
        clone = settings.unprivileged_userns_clone,
        restriction = settings.apparmor_restriction
    );
}

/// What:
///  The kernel's own word for one error number,
///  where this build knows it.
/// Why:
///  The add's skip line names the number by that word too,
///  and a person comparing the two lines must read the same word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = errno === 13 ? 'EACCES' : errno === 1 ? 'EPERM' : 'errno';
/// ```
fn errno_word(errno: i32) -> &'static str {
    return match errno {
        libc::EACCES => "EACCES",
        libc::EPERM => "EPERM",
        libc::ENOSPC => "ENOSPC",
        libc::EINVAL => "EINVAL",
        _other => "errno",
    };
}

/// What:
///  The three settings as this host answers them.
/// Why:
///  `doctor` prints them beside the skip,
///  so the person sees the switch the report tells them about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settings = { maxUserNamespaces: read(MAX) ?? 'absent', ... };
/// ```
fn read_settings() -> HostSettings {
    return HostSettings {
        max_user_namespaces: value_of(MAX_USER_NAMESPACES),
        unprivileged_userns_clone: value_of(UNPRIVILEGED_USERS_CLONE),
        apparmor_restriction: value_of(APPARMOR_RESTRICTION),
    };
}

/// What:
///  One setting's value,
///  trimmed,
///  or the word for a setting the kernel does not have.
/// Why:
///  A kernel without the switch prints the word rather than an empty field,
///  which a person would read as a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = tryReadFile(path)?.trim() ?? 'absent';
/// ```
fn value_of(path: &str) -> String {
    return match std::fs::read_to_string(path) {
        Ok(text) => String::from(text.trim()),
        Err(_missing) => String::from(ABSENT),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "group_drop_report_line_tests.rs"]
mod tests;
