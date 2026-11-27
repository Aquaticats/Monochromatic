//! The mark each cgroup holds,
//! kept attached only while the scan still selects that cgroup.

/// The watcher whose marks these are.
use super::ApplicationWatch;
/// The links of one target,
/// and how to tell whether one is held.
use super::watch_support::{
    attach_target,
    is_attached,
};
/// Which marks stay attached after one scan,
/// and what to say about a name that holds no program of its application.
use crate::application_marks::{
    decide_marks,
    reports_that_still_apply,
};
/// Target discovery across named app-slice services and executable-owned processes.
use crate::application_targets::scan_application_targets;
/// Polling and syscall failures.
use std::io;
/// Cgroup paths of the marks the watcher holds.
use std::path::PathBuf;

/// Reconciles discovered targets,
/// and keeps a mark attached only while the scan still selects its cgroup.
///
/// A name-selected cgroup that holds no program of the application the name claims is not marked,
/// and the watcher says so once for each such cgroup,
/// because an application whose program the scan cannot recognize would otherwise lose its exemption in silence.
/// The marks the watcher holds are also what the process watch treats as covered,
/// so the two can never disagree about which cgroup carries the mark.
pub(super) fn reconcile(watch: &mut ApplicationWatch<'_>) -> io::Result<()> {
    let scan = scan_application_targets(&watch.roots)?;
    // The cgroups whose mark the watcher holds now,
    // and which of them stay after this scan reported what it found.
    let attached: Vec<PathBuf> = watch
        .active
        .iter()
        .map(|attachment| return attachment.path.clone())
        .collect();
    let decision = decide_marks(&attached, &scan, &watch.reported_without_application);
    // A released mark is detached by dropping its links,
    // which is what the kernel unregisters the hooks with.
    watch
        .active
        .retain(|attachment| return decision.keep.contains(&attachment.path));
    watch.reported_without_application =
        reports_that_still_apply(&watch.reported_without_application, &scan);
    for target in &scan.targets {
        if is_attached(&watch.active, target) {
            continue;
        }
        if let Some(attachment) = attach_target(target, watch.mark)? {
            watch.active.push(attachment);
        }
    }
    // The cgroup paths of the marks the watcher holds.
    let marked: Vec<PathBuf> = watch
        .active
        .iter()
        .map(|attachment| return attachment.path.clone())
        .collect();
    watch.routing.note_marked(&marked);
    for cgroup in decision.report {
        watch.reported_without_application.push(cgroup.clone());
        watch.reporter.report_failure(
            "application unit without its program",
            &format!(
                "wg-quicker-exempt: POLICY NOT IN FORCE for {:?}: its name claims a desktop application, and no process in it runs that application's program, so the socket mark is not attached to it and the traffic of every process in it stays in the tunnel. A program of the application may be spelled in a way this build does not know, and `list-targets` shows the cgroups it does mark",
                cgroup.display(),
            ),
        );
    }
    return Ok(());
}
