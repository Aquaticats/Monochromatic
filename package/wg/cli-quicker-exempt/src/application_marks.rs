//! What one reconciliation of the socket marks decides,
//! given the cgroups the watcher holds and what the scan selected.
//!
//! A mark is exemption from the tunnel,
//! so it stays attached only while the scan still selects its cgroup:
//! a name-selected cgroup whose application has gone is no evidence of an application,
//! and its traffic belongs in the tunnel like every other cgroup's.
//! Keeping the mark because the cgroup directory still exists
//! left the bypass on a cgroup whose application had exited
//! and the report that its mark was not attached said the opposite of the truth.

/// The scan's answer about which cgroups carry the mark.
use crate::application_targets::ApplicationScan;
/// Cgroup paths of the marks.
use std::path::PathBuf;

/// What one reconciliation decides.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct MarkDecision {
    /// The cgroups whose mark stays,
    /// which the scan selected.
    pub keep: Vec<PathBuf>,
    /// The cgroups whose mark comes off,
    /// which the scan no longer selects.
    pub release: Vec<PathBuf>,
    /// The cgroups to report,
    /// once each,
    /// in the order to report them.
    pub report: Vec<PathBuf>,
}

/// Decides which marks stay,
/// and what to say about the cgroups the name claims without its program.
///
/// The decision reads only the scan and the attached paths,
/// so it can be tested without a cgroup filesystem and without the kernel's mark.
pub fn decide_marks(attached: &[PathBuf], scan: &ApplicationScan, reported: &[PathBuf]) -> MarkDecision {
    let mut decision = MarkDecision::default();
    for path in attached {
        if scan.targets.contains(path) {
            decision.keep.push(path.clone());
        } else {
            decision.release.push(path.clone());
        }
    }
    for cgroup in &scan.without_application {
        if reported.contains(cgroup) {
            continue;
        }
        decision.report.push(cgroup.clone());
    }
    return decision;
}

/// The reports that still apply:
/// a cgroup the scan selects again is no longer a name without its program,
/// so a later loss of the application is reported again instead of staying silent.
pub fn reports_that_still_apply(reported: &[PathBuf], scan: &ApplicationScan) -> Vec<PathBuf> {
    let mut still: Vec<PathBuf> = Vec::new();
    for cgroup in reported {
        if scan.without_application.contains(cgroup) {
            still.push(cgroup.clone());
        }
    }
    return still;
}
