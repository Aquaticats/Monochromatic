//! What:
//!  Controls of the incomplete reads:
//!  each cause names what could not be read,
//!  a namespace with no live member says its mounts are unknown and never
//!  clear,
//!  and the two bounds say the scan stopped rather than that it found nothing.
//! Why:
//!  A removal reads these causes to report the residual the destroy-anyway
//!  decision accepts,
//!  so a cause that could be mistaken for a count would hide exactly what the
//!  decision has to say out loud.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(read.kind()).toBe('no-live-member'); expect(read.message()).toContain('cannot say it does not');
//! ```

use super::IncompleteRead;
use std::path::PathBuf;

/// A namespace with no live member names the namespace,
/// how many tasks were tried and says the mounts are unknown.
#[test]
fn a_namespace_with_no_live_member_is_unknown_and_never_clear() {
    let read: IncompleteRead = IncompleteRead::NoLiveMember {
        namespace: String::from("mnt:[9]"),
        tasks_tried: 2,
        reason: String::from("No such file or directory (os error 2)"),
    };
    assert_eq!(read.kind(), super::NO_LIVE_MEMBER_WORD);
    assert_eq!(read.namespace(), Some("mnt:[9]"));
    assert_eq!(read.task(), None);
    let said: String = read.message();
    assert!(said.contains("mnt:[9]"), "{said}");
    assert!(said.contains("every one of its 2 tasks"), "{said}");
    assert!(said.contains("cannot say it does not"), "{said}");
    assert!(said.contains("invisible to every scan"), "{said}");
}

/// A task that ended names the task,
/// and its sentence says the namespace is reported by whichever other task was
/// read.
#[test]
fn a_task_that_ended_names_the_task_and_the_retry() {
    let read: IncompleteRead = IncompleteRead::TaskEnded {
        namespace: String::from("mnt:[7]"),
        task: PathBuf::from("/proc/11"),
        reason: String::from("No such file or directory (os error 2)"),
    };
    assert_eq!(read.kind(), super::TASK_ENDED_WORD);
    assert_eq!(read.task(), Some(std::path::Path::new("/proc/11")));
    let said: String = read.message();
    assert!(said.contains("/proc/11"), "{said}");
    assert!(said.contains("ordinary churn"), "{said}");
}

/// The two bounds say the scan stopped,
/// and neither sentence lets a reader call the result clear.
#[test]
fn the_bounds_say_the_scan_stopped() {
    let namespaces: IncompleteRead = IncompleteRead::NamespacesPastTheBound {
        bound: 2,
    };
    assert_eq!(namespaces.kind(), super::NAMESPACES_PAST_THE_BOUND_WORD);
    assert!(
        namespaces
            .message()
            .contains("no scan of it can call itself clear")
    );
    let details: IncompleteRead = IncompleteRead::DetailsPastTheBound {
        bound: 3,
    };
    assert_eq!(details.kind(), super::DETAILS_PAST_THE_BOUND_WORD);
    assert!(details.message().contains("stopped listing and reading"));
    assert_eq!(details.namespace(), None);
    assert_eq!(details.task(), None);
}

/// A listing that failed names the `/proc` root and says no task was read.
#[test]
fn a_listing_that_failed_names_the_proc_root() {
    let read: IncompleteRead = IncompleteRead::Listing {
        proc_root: PathBuf::from("/no/such/proc"),
        reason: String::from("No such file or directory (os error 2)"),
    };
    assert_eq!(read.kind(), super::LISTING_WORD);
    assert!(read.message().contains("/no/such/proc"));
    assert!(read.message().contains("without showing there is none"));
}
