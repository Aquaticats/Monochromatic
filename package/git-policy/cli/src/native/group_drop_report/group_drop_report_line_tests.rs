//! Controls of `doctor`'s group-drop report:
//!  nothing for the host that wrote the `deny` or answered elsewhere,
//!  four lines for the skip,
//!  and one line for a probe that could not be read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(reportOf({ kind: 'written' }, settings)).toBe('');
//! expect(reportOf({ kind: 'skipped', errno: 13 }, settings).split('\n').length).toBe(5);
//! ```

use super::{
    GROUP_DROP_PREFIX,
    HostSettings,
    group_drop_report,
    report_of,
};
use crate::group_drop_report::group_drop_probe::{
    GroupDrop,
    ProbeCall,
};

/// The namespaces a host with Ubuntu 24.04's restriction read.
const NAMESPACES: &str = "15000";
/// The value a kernel without the distribution's own switch reads.
const ABSENT: &str = "absent";
/// The value the AppArmor restriction reads where it is on.
const RESTRICTION_ON: &str = "1";

/// One host's three settings,
/// as the skip's report prints them.
fn settings() -> HostSettings {
    return HostSettings {
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
                call: ProbeCall::UnshareUserNamespace,
                errno: libc::EPERM,
            },
            &settings()
        ),
        ""
    );
    assert_eq!(
        report_of(
            GroupDrop::Answered {
                call: ProbeCall::OpenSetgroups,
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
///  so the remedies are part of the report and not a pointer to another file.
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
    assert_eq!(
        lines[1],
        format!(
            "{GROUP_DROP_PREFIX} user.max_user_namespaces={NAMESPACES}, kernel.unprivileged_userns_clone={ABSENT}, kernel.apparmor_restrict_unprivileged_userns={RESTRICTION_ON}"
        )
    );
    assert!(
        lines[2].contains("CAP_SYS_ADMIN") && lines[2].contains("unprivileged_userns"),
        "{}",
        lines[2]
    );
    assert!(lines[3].contains("nothing needs doing"), "{}", lines[3]);
    assert!(
        lines[3].contains("sysctl --write kernel.apparmor_restrict_unprivileged_userns=0"),
        "the remedy is the measured command: {}",
        lines[3]
    );
    assert!(
        lines[3].contains("removes a security default the distribution ships"),
        "the remedy names its cost: {}",
        lines[3]
    );
}

/// The value a host whose AppArmor restriction is off reads.
const RESTRICTION_OFF: &str = "0";

/// What:
///  A skip whose own setting line reads anything but `1` prints no Ubuntu cause and no
///  command that turns that restriction off,
///  and says instead what the open asks for and where the kernel wrote its own answer.
/// Why:
///  The third line named Ubuntu 24.04's AppArmor restriction for every `EACCES` of the
///  open,
///  and the second line printed the setting that decides it (the alpha.47 release
///  pass's Low 9):
///  a host whose setting reads `0` or `absent` read a cause its own report
///  contradicted,
///  and a remedy for a rule that host does not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const found of ['0', 'absent']) {
///   const lines = reportOf({ kind: 'skipped', errno: 13 }, { ...settings, apparmorRestriction: found }).split('\n');
///   expect(lines[2]).not.toContain("Ubuntu 24.04's default AppArmor profile");
///   expect(lines[2]).toContain(found);
///   expect(lines[3]).not.toContain('sysctl --write kernel.apparmor_restrict_unprivileged_userns=0');
/// }
/// ```
#[test]
fn a_skip_whose_setting_reads_off_names_no_ubuntu_cause() {
    for found in [RESTRICTION_OFF, ABSENT] {
        let mut other: HostSettings = settings();
        other.apparmor_restriction = String::from(found);
        let report: String = report_of(
            GroupDrop::Skipped {
                errno: libc::EACCES,
            },
            &other,
        );
        let lines: Vec<&str> = report.lines().collect();
        assert_eq!(lines.len(), 4, "{report}");
        assert!(
            lines[1]
                .contains(format!("kernel.apparmor_restrict_unprivileged_userns={found}").as_str()),
            "the settings line prints what this host read: {}",
            lines[1]
        );
        assert!(
            !lines[2].contains("Ubuntu 24.04's default AppArmor profile"),
            "the Ubuntu cause is not named where its own setting reads {found}: {}",
            lines[2]
        );
        assert!(
            lines[2].contains(format!("reads `{found}`").as_str()),
            "the cause names the value this host read: {}",
            lines[2]
        );
        assert!(
            lines[2].contains("audit log"),
            "the cause names where the kernel wrote which profile answered: {}",
            lines[2]
        );
        assert!(
            !lines[3].contains("sysctl --write kernel.apparmor_restrict_unprivileged_userns=0"),
            "no remedy turns off a rule this host reads {found}: {}",
            lines[3]
        );
        assert!(
            lines[3].contains("nothing needs doing"),
            "the shipped skip needs nothing: {}",
            lines[3]
        );
    }
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

/// What:
///  The report the doctor makes on this host answers in this module's shape:
///  nothing,
///  or its own four lines,
///  or the one line that says why not.
#[test]
fn the_doctors_own_report_answers_this_host() {
    let report: String = group_drop_report();
    assert!(
        report.is_empty() || report.starts_with(GROUP_DROP_PREFIX),
        "the doctor's report is empty or its own: {report}"
    );
    assert!(
        !report.ends_with("\n\n"),
        "no blank line ends the report: {report}"
    );
}
