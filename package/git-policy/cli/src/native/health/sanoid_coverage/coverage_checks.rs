//! What:
//!  The comparison of one machine's measured dataset tree and its
//!  `sanoid.conf` against the coverage the project decided:
//!  the recursive ancestor,
//!  the template it keeps,
//!  each exclusion's section and its order,
//!  each exclusion's marker,
//!  and the store's place under a snapshotted dataset.
//! Why:
//!  Plan step 20 asks `doctor` to "compare the measured dataset tree and store
//!  location against recursive coverage" and to verify the sections,
//!  the ancestor order,
//!  the mount paths and `com.sun:auto-snapshot=false`,
//!  printing the concrete correction for confirmed drift
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 20;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints").
//!  Three kinds of silence are why every one of these is a check:
//!  a wrongly ordered exclusion section is snapshotted,
//!  `recursive = zfs` cannot leave any child out,
//!  and a marker that does not implement sanoid's exclusion is still what the
//!  other snapshotting tools read.
//!  The comparison reads values only,
//!  so no control here touches this machine's configuration,
//!  its datasets or `/dev/zfs`,
//!  and nothing on a command path reaches it until step 20 is wired.
//!  The comparison is split across siblings because this file is over the line
//!  limit:
//!  this one decides the order of the checks,
//!  `coverage_facts.rs` holds what they are asked with,
//!  `coverage_settings_not_modelled.rs` finds the settings this build cannot
//!  judge,
//!  `root_section_findings.rs` judges the recursive ancestor,
//!  `exclusion_findings.rs` judges one exclusion,
//!  and `store_coverage_finding.rs` judges where the store is held.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = checkCoverage(facts); // { outcome, findings: [{ field, detail, correction }], unproved }
//! ```

/// The facts the comparison reads,
/// re-exported so a caller and a control name them here.
pub use super::coverage_facts::{
    COVERAGE_CHECK_NAME,
    CoverageFacts,
};
/// The answer one comparison gives,
/// and the finding each step of it builds.
use super::coverage_finding::{
    CoverageAnswer,
    CoverageFinding,
};
/// The settings this build cannot judge.
use super::coverage_settings_not_modelled::unproved_setting;
/// The sentence a pass prints for a store the coverage reaches.
use super::coverage_words::pass_detail;
/// One excluded dataset's findings.
use super::exclusion_findings::exclusion_findings;
/// The recursive ancestor's findings.
use super::root_section_findings::root_findings;
/// One configuration text,
/// read into the sections every check looks up.
use super::sanoid_conf_reading::ReadConfig;
/// The dataset that holds the store,
/// and the finding when the coverage does not reach it.
use super::store_coverage_finding::store_holder;
/// What:
///  The comparison's answer for one machine.
/// Why:
///  It is the one entry step 20's wiring calls,
///  so the check's order of decisions cannot differ between a report and a
///  control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkCoverage(facts); // CoverageAnswer
/// ```
pub fn check_coverage(facts: &CoverageFacts) -> CoverageAnswer {
    let config: &ReadConfig = match &facts.config {
        Ok(read) => read,
        Err(refusal) => {
            return CoverageAnswer::new(
                Vec::new(),
                Some(format!(
                    "{} cannot be read as sanoid reads it: {}",
                    facts.config_path,
                    refusal.reason()
                )),
                String::new(),
            );
        }
    };
    if let Some(reason) = unproved_setting(facts, config) {
        return CoverageAnswer::new(Vec::new(), Some(reason), String::new());
    }
    let mut findings: Vec<CoverageFinding> = Vec::new();
    root_findings(facts, config, &mut findings);
    for exclusion in &facts.decision.exclusions {
        exclusion_findings(facts, config, exclusion, &mut findings);
    }
    match store_holder(facts, &mut findings) {
        Ok(holder) => {
            let covered: usize = facts
                .tree
                .datasets_under(facts.decision.recursive_root.as_str())
                .iter()
                .filter(|dataset| return !facts.decision.excludes(dataset.name.as_str()))
                .count();
            let detail: String = pass_detail(
                covered,
                facts.decision.recursive_root.as_str(),
                facts.decision.exclusions.len(),
                facts.store_path.as_str(),
                holder,
            );
            return CoverageAnswer::new(findings, None, detail);
        }
        Err(reason) => return CoverageAnswer::new(findings, Some(reason), String::new()),
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "coverage_checks_tests.rs"]
mod tests;
