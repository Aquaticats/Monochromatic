//! What:
//!  Controls for the namespace child:
//!  a report round-trips,
//!  every malformed report is refused by what it lacks,
//!  and a child that enters this process's own mount namespace reports what
//!  the kernel answered.
//! Why:
//!  The handler must never read a missing or rewritten report as an act that
//!  finished,
//!  and the `setns` refusal an unprivileged handler meets is exactly what the
//!  report has to carry,
//!  so both are shown here without a mount and without a guest.
//!  A privileged run on this machine answers entry, which the same control
//!  accepts and names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a child that cannot enter reports the errno', () => { expect(runIn(ownNamespace, work).outcome).toBe('failed'); });
//! ```

use super::{ActOutcome, ActReport, run_in};

/// This process's own mount namespace.
const OUR_MOUNT_NAMESPACE: &str = "/proc/self/ns/mnt";

#[test]
fn a_report_round_trips_with_its_table() {
    let report: ActReport = ActReport::done(String::from("way 8: unmounted mount 42"), String::from("42 1 0:32 / / rw - zfs a/b rw\n"));
    let read: ActReport = ActReport::decode(report.encode().as_slice()).expect("a report this build wrote is one it reads");
    assert_eq!(read, report);
}

#[test]
fn a_detail_line_with_a_newline_and_a_backslash_round_trips() {
    let report: ActReport = ActReport::failed(String::from("a line\nwith a \\ and more"), libc::EBUSY, String::new());
    let read: ActReport = ActReport::decode(report.encode().as_slice()).expect("the escapes are this build's own");
    assert_eq!(read.detail, "a line\nwith a \\ and more");
    assert_eq!(read.errno, Some(libc::EBUSY));
    assert_eq!(read.outcome, ActOutcome::Failed);
}

#[test]
fn each_malformed_report_is_refused_by_what_it_lacks() {
    for (bytes, wanted) in [
        (b"outcome done\n".to_vec(), "four lines"),
        (b"outcome done\nerrno none\ndetail x\ntable 9999999999999\n".to_vec(), "longer than the crate's own bound"),
        (b"outcome perhaps\nerrno none\ndetail x\ntable 0\n".to_vec(), "not one this build writes"),
        (b"outcome done\nerrno maybe\ndetail x\ntable 0\n".to_vec(), "not a number"),
        (b"outcome done\nerrno none\ndetail x\ntable 1\n".to_vec(), "length says 1"),
        (b"whatever done\nerrno none\ndetail x\ntable 0\n".to_vec(), "is not one this build writes"),
    ] {
        let refused: String = ActReport::decode(bytes.as_slice()).expect_err("a malformed report is refused");
        assert!(refused.contains(wanted), "the refusal says `{wanted}`, and it read {refused}");
    }
}

#[test]
fn a_child_that_enters_this_process_s_own_namespace_reports_what_the_kernel_answered() {
    let namespace: std::fs::File = std::fs::File::open(OUR_MOUNT_NAMESPACE).expect("this process's own namespace is opened");
    let report: ActReport = run_in(&namespace, || {
        let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the child reads its own table");
        return ActReport::done(String::from("the child read its own table"), table);
    })
    .expect("a child that runs one piece of the act answers with one report");
    // SAFETY: `geteuid` takes no argument and reads one number.
    let privileged: bool = unsafe { libc::geteuid() } == 0;
    if privileged {
        assert_eq!(report.outcome, ActOutcome::Done, "a privileged handler enters the namespace it names, and the report said {}", report.detail);
        assert!(report.table.contains(" / "), "the table the child read is the mount table, and it read {} bytes", report.table.len());
        return;
    }
    assert_eq!(report.outcome, ActOutcome::Failed, "entering a namespace needs a capability this account does not hold, and the report said {}", report.detail);
    assert_eq!(report.errno, Some(libc::EPERM));
    assert!(report.detail.contains("setns"), "the refusal names the call, and it read {}", report.detail);
}
