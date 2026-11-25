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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = checkCoverage(facts); // { outcome, findings: [{ field, detail, correction }], unproved }
//! ```

use super::coverage_finding::{
    CoverageAnswer, CoverageFinding, FIELD_EXCLUSION_AUTOSNAP, FIELD_EXCLUSION_DATASET, FIELD_EXCLUSION_MARKER, FIELD_EXCLUSION_MOUNT, FIELD_EXCLUSION_SECTION, FIELD_RECURSION, FIELD_ROOT_SECTION,
    FIELD_SECTION_ORDER, FIELD_STORE_COVERAGE, FIELD_TEMPLATE,
};
use super::coverage_words::{
    AUTO_SNAPSHOT_OFF, AUTO_SNAPSHOT_PROPERTY, add_recursion_correction, add_section_correction, change_autosnap_correction, change_recursion_correction, marker_correction, marker_words, mountpoint_words,
    move_section_correction, pass_detail, template_correction,
};
use super::decided_coverage::{CoverageDecision, Exclusion, ExclusionMount, exclusion_section, root_section};
use super::measured_dataset_tree::{MeasuredTree, has_path_mountpoint, is_under};
use super::sanoid_conf_reading::{ConfigRefusal, ReadConfig, SettingLookup};
use super::sanoid_section::{
    AUTOSNAP_SETTING, PATH_SETTING, PROCESS_CHILDREN_ONLY_SETTING, RECURSIVE_SETTING, RECURSIVE_ZFS, Recursion, SKIP_CHILDREN_SETTING, TEMPLATE_SETTING, is_false_word, recursion_of,
};

/// The check's stable name in a report line.
pub const COVERAGE_CHECK_NAME: &str = "sanoid-coverage";

/// What:
///  Every fact the coverage comparison reads,
///  gathered once.
/// Why:
///  One value is the whole comparison's input,
///  so a control builds the healthy set and changes one field to see exactly
///  one finding,
///  and a guest arm gathers the same facts from a real pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CoverageFacts = { decision; tree; configPath; config: Result<ReadConfig, ConfigRefusal>; storePath };
/// ```
pub struct CoverageFacts {
    /// The coverage the project decided.
    pub decision: CoverageDecision,
    /// The dataset tree the machine measured.
    pub tree: MeasuredTree,
    /// The configuration file's path,
    /// which every correction names.
    pub config_path: String,
    /// The configuration's reading,
    /// or the reason it could not be read.
    pub config: Result<ReadConfig, ConfigRefusal>,
    /// The store folder the design keeps in home and snapshotted.
    pub store_path: String,
}

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
        Err(refusal) => return CoverageAnswer::new(Vec::new(), Some(format!("{} cannot be read as sanoid reads it: {}", facts.config_path, refusal.reason())), String::new()),
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
            let covered: usize = facts.tree.datasets_under(facts.decision.recursive_root.as_str()).iter().filter(|dataset| return !facts.decision.excludes(dataset.name.as_str())).count();
            let detail: String = pass_detail(covered, facts.decision.recursive_root.as_str(), facts.decision.exclusions.len(), facts.store_path.as_str(), holder);
            return CoverageAnswer::new(findings, None, detail);
        }
        Err(reason) => return CoverageAnswer::new(findings, Some(reason), String::new()),
    }
}

/// What one relevant section says that this build does not model.
fn unproved_setting(facts: &CoverageFacts, config: &ReadConfig) -> Option<String> {
    let mut datasets: Vec<&str> = vec![facts.decision.recursive_root.as_str()];
    for exclusion in &facts.decision.exclusions {
        datasets.push(exclusion.dataset.as_str());
    }
    for dataset in datasets {
        let Some(section) = config.section(dataset) else {
            continue;
        };
        for key in [RECURSIVE_SETTING, TEMPLATE_SETTING, AUTOSNAP_SETTING, PATH_SETTING, PROCESS_CHILDREN_ONLY_SETTING, SKIP_CHILDREN_SETTING] {
            match section.setting(key) {
                SettingLookup::CaseVariant { spelling, line } => {
                    return Some(format!("the [{dataset}] section writes `{spelling}` on line {line}, which is not the key `{key}` sanoid reads, and this build cannot prove what sanoid does with it"));
                }
                SettingLookup::Found { value, line } if matches!(key, PATH_SETTING | PROCESS_CHILDREN_ONLY_SETTING | SKIP_CHILDREN_SETTING) => {
                    return Some(format!("the [{dataset}] section writes `{key} = {value}` on line {line}, and this build does not model what that changes about coverage"));
                }
                SettingLookup::Found { .. } | SettingLookup::Absent => continue,
            }
        }
    }
    return None;
}

/// The recursive ancestor's section,
/// its recursion and its template.
fn root_findings(facts: &CoverageFacts, config: &ReadConfig, findings: &mut Vec<CoverageFinding>) {
    let root: &str = facts.decision.recursive_root.as_str();
    let Some(section) = config.section(root) else {
        let detail: String = format!("the configuration has no [{root}] section, so no dataset under {root} is covered");
        findings.push(CoverageFinding::new(FIELD_ROOT_SECTION, detail, Some(add_section_correction(facts.config_path.as_str(), &root_section(&facts.decision)))));
        return;
    };
    match recursion_of(section.written_value(RECURSIVE_SETTING)) {
        Recursion::Yes => {}
        Recursion::Zfs => {
            let detail: String = format!("the [{root}] section sets `{RECURSIVE_SETTING} = {RECURSIVE_ZFS}`, so sanoid takes one recursive snapshot of {root} and every child dataset is in it, which no exclusion can be left out of");
            findings.push(CoverageFinding::new(FIELD_RECURSION, detail, Some(change_recursion_correction(facts.config_path.as_str(), root, RECURSIVE_ZFS))));
        }
        Recursion::Off { value: None } => {
            let detail: String = format!("the [{root}] section writes no `{RECURSIVE_SETTING}` value, so only {root} itself is snapshotted and its child datasets are not");
            findings.push(CoverageFinding::new(FIELD_RECURSION, detail, Some(add_recursion_correction(facts.config_path.as_str(), root))));
        }
        Recursion::Off { value: Some(written) } => {
            let detail: String = format!("the [{root}] section writes `{RECURSIVE_SETTING} = {written}`, which sanoid does not read as true, so its child datasets are not covered");
            findings.push(CoverageFinding::new(FIELD_RECURSION, detail, Some(change_recursion_correction(facts.config_path.as_str(), root, written.as_str()))));
        }
    }
    match section.setting(TEMPLATE_SETTING) {
        SettingLookup::Found { value, .. } => {
            if !value.split(',').any(|name| return name.trim() == facts.decision.template) {
                let detail: String = format!("the [{root}] section keeps the template `{value}`, which does not name `{}`", facts.decision.template);
                findings.push(CoverageFinding::new(FIELD_TEMPLATE, detail, Some(template_correction(facts.config_path.as_str(), root, facts.decision.template.as_str(), Some(value.as_str())))));
            }
        }
        SettingLookup::Absent => {
            let detail: String = format!("the [{root}] section names no template, so the retention this route decided for it does not apply");
            findings.push(CoverageFinding::new(FIELD_TEMPLATE, detail, Some(template_correction(facts.config_path.as_str(), root, facts.decision.template.as_str(), None))));
        }
        SettingLookup::CaseVariant { .. } => {}
    }
}

/// One excluded dataset:
/// its presence,
/// its mount kind,
/// its section and its
/// marker.
fn exclusion_findings(facts: &CoverageFacts, config: &ReadConfig, exclusion: &Exclusion, findings: &mut Vec<CoverageFinding>) {
    let dataset: &str = exclusion.dataset.as_str();
    let found = facts.tree.dataset(dataset);
    match found {
        None => {
            let detail: String = format!("the measured dataset tree holds no {dataset}, and a folder inside another dataset cannot be excluded, so {dataset} must be its own dataset before a section can leave it out");
            findings.push(CoverageFinding::new(FIELD_EXCLUSION_DATASET, detail, None));
        }
        Some(measured) => mount_finding(exclusion, measured.mountpoint.as_deref(), findings),
    }
    section_findings(facts, config, dataset, findings);
    if let Some(measured) = found
        && measured.auto_snapshot.as_deref() != Some(AUTO_SNAPSHOT_OFF)
    {
        let detail: String = format!("`{AUTO_SNAPSHOT_PROPERTY}` {} on {dataset}, and the decision sets it to {AUTO_SNAPSHOT_OFF} on every exclusion", marker_words(measured.auto_snapshot.as_deref()));
        findings.push(CoverageFinding::new(FIELD_EXCLUSION_MARKER, detail, Some(marker_correction(dataset))));
    }
}

/// One exclusion's mount point against the kind the decision gives it.
fn mount_finding(exclusion: &Exclusion, mountpoint: Option<&str>, findings: &mut Vec<CoverageFinding>) {
    let dataset: &str = exclusion.dataset.as_str();
    let words: String = mountpoint_words(mountpoint);
    match exclusion.mount {
        ExclusionMount::Path if !has_path_mountpoint(mountpoint) => {
            let detail: String = format!(
                "the decision mounts {dataset} at a folder, and the measured mount point of {dataset} is {words}, so the exclusion covers no folder while the dataset that holds it keeps snapshotting that folder, and no configuration line or property ends this"
            );
            findings.push(CoverageFinding::new(FIELD_EXCLUSION_MOUNT, detail, None));
        }
        ExclusionMount::Unmounted if has_path_mountpoint(mountpoint) => {
            let detail: String = format!("the decision leaves {dataset} unmounted, and the measured mount point of {dataset} is {words}, and no configuration line or property ends this");
            findings.push(CoverageFinding::new(FIELD_EXCLUSION_MOUNT, detail, None));
        }
        ExclusionMount::Path | ExclusionMount::Unmounted => {}
    }
}

/// One exclusion's section,
/// its `autosnap` value and its place in file order.
fn section_findings(facts: &CoverageFacts, config: &ReadConfig, dataset: &str, findings: &mut Vec<CoverageFinding>) {
    let Some(section) = config.section(dataset) else {
        let detail: String = format!("the configuration has no [{dataset}] section, so the recursive section's settings reach {dataset} and it is snapshotted");
        findings.push(CoverageFinding::new(FIELD_EXCLUSION_SECTION, detail, Some(add_section_correction(facts.config_path.as_str(), &exclusion_section(dataset)))));
        return;
    };
    match section.setting(AUTOSNAP_SETTING) {
        SettingLookup::Found { value, line } => {
            if !is_false_word(value.as_str()) {
                let detail: String = format!("the [{dataset}] section writes `{AUTOSNAP_SETTING} = {value}` on line {line}, which sanoid does not read as false, so {dataset} is snapshotted");
                findings.push(CoverageFinding::new(FIELD_EXCLUSION_AUTOSNAP, detail, Some(change_autosnap_correction(facts.config_path.as_str(), dataset, value.as_str()))));
            }
        }
        SettingLookup::Absent => {
            let detail: String = format!("the [{dataset}] section writes no `{AUTOSNAP_SETTING}` value, so the template's own value applies and {dataset} is snapshotted");
            findings.push(CoverageFinding::new(FIELD_EXCLUSION_AUTOSNAP, detail, Some(add_section_correction(facts.config_path.as_str(), &exclusion_section(dataset)))));
        }
        SettingLookup::CaseVariant { .. } => {}
    }
    order_finding(facts, config, dataset, section.line, findings);
}

/// What:
///  How late the last section whose recursion reaches one dataset stands,
///  when one does.
/// Why:
///  Two sections reach an exclusion:
///  the decided recursive ancestor,
///  which the design puts every exclusion after so a later recursive ancestor
///  cannot silently include it,
///  and any dataset ancestor of the exclusion that recurses in this tree.
///  The latest of them is the one a moved section has to pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// laterRecursiveSection(config, 'tank/home', last); // seeds the ancestor's position
/// ```
fn later_recursive_section(config: &ReadConfig, dataset: &str, last: &mut Option<(usize, String)>) {
    let Some(section) = config.section(dataset) else {
        return;
    };
    if matches!(recursion_of(section.written_value(RECURSIVE_SETTING)), Recursion::Off { .. }) {
        return;
    }
    let Some(position) = config.position(dataset) else {
        return;
    };
    if last.as_ref().is_none_or(|(held, _)| return *held < position) {
        *last = Some((position, String::from(dataset)));
    }
}

/// The last section whose recursion reaches one dataset,
/// and that dataset's
/// section standing before it.
fn order_finding(facts: &CoverageFacts, config: &ReadConfig, dataset: &str, section_line: usize, findings: &mut Vec<CoverageFinding>) {
    let mut last: Option<(usize, String)> = None;
    later_recursive_section(config, facts.decision.recursive_root.as_str(), &mut last);
    for ancestor in facts.tree.ancestors(dataset) {
        later_recursive_section(config, ancestor.name.as_str(), &mut last);
    }
    let Some((position, ancestor)) = last else {
        return;
    };
    if config.position(dataset).unwrap_or(section_line) < position {
        let ancestor_section = config.section(ancestor.as_str());
        let ancestor_line: usize = ancestor_section.map_or(section_line, |held| return held.line);
        let detail: String = format!(
            "the [{dataset}] section stands on line {section_line} and the [{ancestor}] section on line {ancestor_line}, and sanoid copies a recursive section's settings onto every child without checking for a child section, so {dataset} is snapshotted"
        );
        findings.push(CoverageFinding::new(FIELD_SECTION_ORDER, detail, Some(move_section_correction(dataset, ancestor.as_str()))));
    }
}

/// What:
///  The dataset whose mount point holds the store,
///  and the finding when recursive coverage does not reach it.
/// Why:
///  The design keeps the pnpm store in home and snapshotted,
///  so a store held by another dataset is a drift no INI line or property
///  ends,
///  and a store no measured mount point holds is a fact this build could not
///  read rather than a pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// storeHolder(facts, config, findings); // Ok('tank/home') | Err('no measured dataset ...')
/// ```
fn store_holder<'a>(facts: &'a CoverageFacts, findings: &mut Vec<CoverageFinding>) -> Result<&'a str, String> {
    let store: &str = facts.store_path.as_str();
    let Some(holder) = facts.tree.holding_dataset(store) else {
        return Err(format!("no measured dataset's mount point covers the store at {store}, so this build cannot prove the store is snapshotted"));
    };
    let root: &str = facts.decision.recursive_root.as_str();
    let covered: bool = (holder == root || is_under(holder, root)) && !facts.decision.excludes(holder);
    if !covered {
        let detail: String = format!("the store at {store} is held by {holder}, which the recursive section of {root} does not reach, so the store is not snapshotted");
        findings.push(CoverageFinding::new(FIELD_STORE_COVERAGE, detail, None));
    }
    return Ok(holder);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "coverage_checks_tests.rs"]
mod tests;
