//! Controls of `doctor`'s group-drop line:
//!  nothing for the host that wrote the `deny` or answered elsewhere,
//!  four lines for the skip,
//!  one line for a probe that could not be read,
//!  and the ceiling that ends a read which never answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(reportOf({ kind: 'written' }, settings)).toBe('');
//! expect(reportOf({ kind: 'skipped', errno: 13 }, settings).split('\n').length).toBe(5);
//! ```

use super::{
    GROUP_DROP_CEILING,
    GROUP_DROP_PREFIX,
    group_drop_line,
    group_drop_line_with,
    report_of,
};
use crate::account_companion::companion_calls::CompanionCall;
use crate::account_companion::companion_deadline::COMPANION_DEADLINE;
use crate::account_companion::companion_group_drop_probe::{
    GroupDrop,
    the_group_drop,
};
use crate::account_companion::companion_settings::Settings;
use std::time::Duration;

/// The namespaces a host with Ubuntu 24.04's restriction read,
/// which the line prints as they came.
const NAMESPACES: &str = "15000";
/// The value a kernel without the distribution's own switch reads.
const ABSENT: &str = "absent";
/// The value the AppArmor restriction reads where it is on.
const RESTRICTION_ON: &str = "1";

/// One host's three settings, as the skip's line prints them.
fn settings() -> Settings {
    return Settings {
        max_user_namespaces: String::from(NAMESPACES),
        unprivileged_userns_clone: String::from(ABSENT),
        apparmor_restriction: String::from(RESTRICTION_ON),
    };
}

/// What:
///  A host that wrote the `deny` has nothing to say,
///  and neither has one that answered at another call,
///  because the add's own line reports that answer.
#[test]
fn a_host_that_wrote_the_deny_or_answered_elsewhere_prints_nothing() {
    assert_eq!(report_of(GroupDrop::Written, &settings()), "");
    assert_eq!(
        report_of(
            GroupDrop::Answered {
                call: CompanionCall::UnshareUserNamespace,
                errno: libc::EPERM,
            },
            &settings()
        ),
        ""
    );
    assert_eq!(
        report_of(
            GroupDrop::Answered {
                call: CompanionCall::OpenSetgroups,
                errno: libc::EPERM,
            },
            &settings()
        ),
        ""
    );
}

/// What:
///  The skip prints four lines:
///  the call and the number it answered,
///  the settings this host read,
///  the reason the kernel answers there,
///  and the remedies a person has.
/// Why:
///  The owner asked that `doctor` tell the user what to do,
///  so the remedy is part of the line and not a pointer to another file.
#[test]
fn the_skip_prints_the_call_the_settings_the_reason_and_the_remedies() {
    let report: String = report_of(
        GroupDrop::Skipped {
            errno: libc::EACCES,
        },
        &settings(),
    );
    let lines: Vec<&str> = report.lines().collect();
    assert_eq!(lines.len(), 4, "{report}");
    for line in &lines {
        assert!(
            line.starts_with(GROUP_DROP_PREFIX),
            "every line says which section spoke: {line}"
        );
    }
    assert!(
        lines[0].contains("open /proc/self/setgroups answered EACCES (errno 13)"),
        "{}",
        lines[0]
    );
    assert!(
        lines[0].contains("goes ahead unproved by that check"),
        "the skip is not a pass: {}",
        lines[0]
    );
    assert_eq!(lines[1], format!("{GROUP_DROP_PREFIX} {}", settings().line()));
    assert!(
        lines[2].contains("CAP_SYS_ADMIN") && lines[2].contains("unprivileged_userns"),
        "{}",
        lines[2]
    );
    assert!(
        lines[3].contains("nothing needs doing"),
        "{}",
        lines[3]
    );
    assert!(
        lines[3].contains("sysctl --write kernel.apparmor_restrict_unprivileged_userns=0"),
        "the remedy is the measured command: {}",
        lines[3]
    );
    assert!(
        lines[3].contains("removes a distribution security default"),
        "the remedy names its cost: {}",
        lines[3]
    );
}

/// What:
///  A probe that could not be run or read prints one line with its reason,
///  so a section that silently said nothing never passes for a passing host.
#[test]
fn a_probe_that_could_not_be_read_is_said() {
    let report: String = report_of(
        GroupDrop::Unread {
            reason: String::from("its child could not be started (errno 11)"),
        },
        &settings(),
    );
    assert_eq!(report.lines().count(), 1, "{report}");
    assert!(report.starts_with(GROUP_DROP_PREFIX), "{report}");
    assert!(
        report.contains("the group drop could not be checked"),
        "{report}"
    );
    assert!(report.contains("errno 11"), "{report}");
    assert!(report.ends_with('\n'), "{report}");
}

/// A read of the group drop that never answers,
/// as a probe whose child is stuck in the kernel does not.
///
/// The thread parks,
/// so it spends no processor time while the ceiling runs out:
/// this is a stall,
/// not a load.
fn stalled_group_drop() -> GroupDrop {
    loop {
        std::thread::park();
    }
}

/// What:
///  A read that never answers ends at its ceiling with one line that says so.
/// Why:
///  `doctor` may not wait with no end,
///  and the ceiling must sit above the probe's own bound,
///  or the report would give up on a probe that is still inside it.
#[test]
fn a_read_that_never_answers_gives_up_at_its_ceiling() {
    assert!(
        GROUP_DROP_CEILING > COMPANION_DEADLINE,
        "the thread's ceiling sits above the bound the probe itself enforces"
    );
    let line: String = group_drop_line_with(stalled_group_drop, Duration::from_millis(50));
    assert!(line.starts_with(GROUP_DROP_PREFIX), "{line}");
    assert!(line.contains("did not answer within"), "{line}");
    assert!(line.ends_with('\n'), "{line}");
    assert_eq!(line.lines().count(), 1, "{line}");
}

/// What:
///  The read `doctor` makes answers in the shape of this module's report,
///  and the entry point the doctor calls answers what that read does.
#[test]
fn the_doctors_own_read_answers_this_host() {
    let line: String = group_drop_line_with(the_group_drop, GROUP_DROP_CEILING);
    assert!(
        line.is_empty() || line.starts_with(GROUP_DROP_PREFIX),
        "the doctor's line is empty or its own report: {line}"
    );
    let through_the_doctor: String = group_drop_line();
    assert_eq!(
        through_the_doctor.is_empty(),
        line.is_empty(),
        "the doctor's own entry point answers what the read answers"
    );
}
