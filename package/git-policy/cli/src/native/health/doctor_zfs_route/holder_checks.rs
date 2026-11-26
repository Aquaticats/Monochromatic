//! What:
//!  The namespaces that hold a clone,
//!  read from the all-namespace scan's result.
//! Why:
//!  "Unmounted" means absent in every mount namespace holding the clone's ZFS
//!  source,
//!  so a removal asks the root handler for the scan before each destructive
//!  step,
//!  and `doctor` names the holders the same result holds
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces";
//!  plan step 18,
//!  "the holders of a clone by namespace ID and representative PID,
//!  read from step 9's all-namespace scan result";
//!  `doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`).
//!  A scan that could not read every task proves no absence,
//!  so incomplete coverage is unknown even when it found no holder,
//!  and a holder is a finding that names the namespace,
//!  the representative task and the path.
//!  The result is a value handed in,
//!  so a control writes the holders it chose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = check_holders({ clone, holders, incomplete }); // pass | finding('holder') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use std::path::PathBuf;

/// What:
///  One native mount namespace and one task in it that holds the clone.
/// Why:
///  A namespace a person can name is the one place the mount lives,
///  and a task is what another run can look at,
///  so both travel with the matched source and path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceHold = { namespace: string; representativePid: number; source: string; mountPoint: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceHold {
    /// The mount namespace's own identifier,
    /// as the kernel prints it (`mnt:[4026531840]`).
    pub namespace: String,
    /// One task in that namespace,
    /// which a person can look at.
    pub representative_pid: u32,
    /// The ZFS source the row names,
    /// which is the clone's dataset.
    pub source: String,
    /// Where the mount stands in that namespace.
    pub mount_point: PathBuf,
}

/// What:
///  The scan's whole answer for one clone:
///  the holders it found and every read it could not complete.
/// Why:
///  A holder and an incomplete read are both answers,
///  and a report that dropped either would claim more than the scan showed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HolderFacts = { clone: string; holders: NamespaceHold[]; incomplete: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HolderFacts {
    /// The clone the scan asked about.
    pub clone: String,
    /// Every namespace the scan saw holding it.
    pub holders: Vec<NamespaceHold>,
    /// Every read the scan could not complete,
    /// in the scan's own words.
    pub incomplete: Vec<String>,
}

/// What:
///  Whether any namespace holds the clone,
///  and whether the scan could read every task.
/// Why:
///  A holder is what a removal must end or refuse on,
///  so the finding names each namespace and task,
///  and a scan that read nothing complete leaves the answer unknown rather
///  than passing for a clean one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_holders(facts); // pass | finding('holder') | unknown
/// ```
pub fn check_holders(facts: &HolderFacts) -> CheckOutcome {
    let clone: &str = facts.clone.as_str();
    if !facts.holders.is_empty() {
        let described: Vec<String> = facts
            .holders
            .iter()
            .map(|holder| {
                return format!(
                    "{} (task {}) mounts {} at {}",
                    holder.namespace,
                    holder.representative_pid,
                    holder.source,
                    holder.mount_point.display()
                );
            })
            .collect();
        let mut detail: String = format!(
            "{} namespace(s) hold `{clone}`: {}",
            facts.holders.len(),
            described.join("; ")
        );
        if !facts.incomplete.is_empty() {
            detail.push_str(&format!(
                "; {} further read(s) could not be completed ({})",
                facts.incomplete.len(),
                facts.incomplete.join("; ")
            ));
        }
        return finding("holder", detail);
    }
    if !facts.incomplete.is_empty() {
        return unknown(format!(
            "the scan found no holder of `{clone}` and could not read {} task(s) or table(s) ({}), so absence in every namespace is unproved",
            facts.incomplete.len(),
            facts.incomplete.join("; ")
        ));
    }
    return pass(format!(
        "no namespace holds `{clone}`, and the scan completed every read"
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_checks_tests.rs"]
mod tests;
