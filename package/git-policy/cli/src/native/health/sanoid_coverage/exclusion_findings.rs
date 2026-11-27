//! What:
//!  One excluded dataset's findings:
//!  whether the machine has it,
//!  its mount point,
//!  its section and its `autosnap` value,
//!  and where that section stands in file order.
//! Why:
//!  An exclusion costs space or history only when one of these is wrong,
//!  and each one needs a different correction:
//!  a dataset of its own,
//!  a mount point the decision names,
//!  a section the template reaches through,
//!  and a section written after every recursive ancestor above it.
//!  One ancestor's descendant grant or one section's place is read from the
//!  same facts,
//!  so all of it lives beside the one finding it feeds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! exclusionFindings(facts, config, exclusion, findings);
//! ```

/// The facts the comparison reads.
use super::coverage_facts::CoverageFacts;
/// One finding,
/// and the field names its report lines carry.
use super::coverage_finding::{
    CoverageFinding,
    FIELD_EXCLUSION_AUTOSNAP,
    FIELD_EXCLUSION_DATASET,
    FIELD_EXCLUSION_MARKER,
    FIELD_EXCLUSION_MOUNT,
    FIELD_EXCLUSION_SECTION,
    FIELD_SECTION_ORDER,
};
/// The marker's words and the corrections a person applies.
use super::coverage_words::{
    AUTO_SNAPSHOT_OFF,
    AUTO_SNAPSHOT_PROPERTY,
    add_section_correction,
    change_autosnap_correction,
    marker_correction,
    marker_words,
    mountpoint_words,
    move_section_correction,
};
/// One exclusion the decision names,
/// where it is mounted,
/// and the text its section is written as.
use super::decided_coverage::{
    Exclusion,
    ExclusionMount,
    exclusion_section,
};
/// Whether a measured mount point is a folder.
use super::measured_dataset_tree::has_path_mountpoint;
/// One configuration text read into sections,
/// and the lookup a section's setting answers.
use super::sanoid_conf_reading::{
    ReadConfig,
    SettingLookup,
};
/// The `autosnap` setting and how a section recurses.
use super::sanoid_section::{
    AUTOSNAP_SETTING,
    RECURSIVE_SETTING,
    Recursion,
    is_false_word,
    recursion_of,
};
/// One excluded dataset:
/// its presence,
/// its mount kind,
/// its section and its
/// marker.
pub(super) fn exclusion_findings(
    facts: &CoverageFacts,
    config: &ReadConfig,
    exclusion: &Exclusion,
    findings: &mut Vec<CoverageFinding>,
) {
    let dataset: &str = exclusion.dataset.as_str();
    let found = facts.tree.dataset(dataset);
    match found {
        None => {
            let detail: String = format!(
                "the measured dataset tree holds no {dataset}, and a folder inside another dataset cannot be excluded, so {dataset} must be its own dataset before a section can leave it out"
            );
            findings.push(CoverageFinding::new(FIELD_EXCLUSION_DATASET, detail, None));
        }
        Some(measured) => mount_finding(exclusion, measured.mountpoint.as_deref(), findings),
    }
    section_findings(facts, config, dataset, findings);
    if let Some(measured) = found
        && measured.auto_snapshot.as_deref() != Some(AUTO_SNAPSHOT_OFF)
    {
        let detail: String = format!(
            "`{AUTO_SNAPSHOT_PROPERTY}` {} on {dataset}, and the decision sets it to {AUTO_SNAPSHOT_OFF} on every exclusion",
            marker_words(measured.auto_snapshot.as_deref())
        );
        findings.push(CoverageFinding::new(
            FIELD_EXCLUSION_MARKER,
            detail,
            Some(marker_correction(dataset)),
        ));
    }
}

/// One exclusion's mount point against the kind the decision gives it.
fn mount_finding(
    exclusion: &Exclusion,
    mountpoint: Option<&str>,
    findings: &mut Vec<CoverageFinding>,
) {
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
            let detail: String = format!(
                "the decision leaves {dataset} unmounted, and the measured mount point of {dataset} is {words}, and no configuration line or property ends this"
            );
            findings.push(CoverageFinding::new(FIELD_EXCLUSION_MOUNT, detail, None));
        }
        ExclusionMount::Path | ExclusionMount::Unmounted => {}
    }
}

/// One exclusion's section,
/// its `autosnap` value and its place in file order.
fn section_findings(
    facts: &CoverageFacts,
    config: &ReadConfig,
    dataset: &str,
    findings: &mut Vec<CoverageFinding>,
) {
    let Some(section) = config.section(dataset) else {
        let detail: String = format!(
            "the configuration has no [{dataset}] section, so the recursive section's settings reach {dataset} and it is snapshotted"
        );
        findings.push(CoverageFinding::new(
            FIELD_EXCLUSION_SECTION,
            detail,
            Some(add_section_correction(
                facts.config_path.as_str(),
                &exclusion_section(dataset),
            )),
        ));
        return;
    };
    match section.setting(AUTOSNAP_SETTING) {
        SettingLookup::Found {
            value,
            line,
        } => {
            if !is_false_word(value.as_str()) {
                let detail: String = format!(
                    "the [{dataset}] section writes `{AUTOSNAP_SETTING} = {value}` on line {line}, which sanoid does not read as false, so {dataset} is snapshotted"
                );
                findings.push(CoverageFinding::new(
                    FIELD_EXCLUSION_AUTOSNAP,
                    detail,
                    Some(change_autosnap_correction(
                        facts.config_path.as_str(),
                        dataset,
                        value.as_str(),
                    )),
                ));
            }
        }
        SettingLookup::Absent => {
            let detail: String = format!(
                "the [{dataset}] section writes no `{AUTOSNAP_SETTING}` value, so the template's own value applies and {dataset} is snapshotted"
            );
            findings.push(CoverageFinding::new(
                FIELD_EXCLUSION_AUTOSNAP,
                detail,
                Some(add_section_correction(
                    facts.config_path.as_str(),
                    &exclusion_section(dataset),
                )),
            ));
        }
        SettingLookup::CaseVariant {
            ..
        } => {}
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
fn later_recursive_section(
    config: &ReadConfig,
    dataset: &str,
    last: &mut Option<(usize, String)>,
) {
    let Some(section) = config.section(dataset) else {
        return;
    };
    if matches!(
        recursion_of(section.written_value(RECURSIVE_SETTING)),
        Recursion::Off { .. }
    ) {
        return;
    }
    let Some(position) = config.position(dataset) else {
        return;
    };
    if last
        .as_ref()
        .is_none_or(|(held, _)| return *held < position)
    {
        *last = Some((position, String::from(dataset)));
    }
}

/// The last section whose recursion reaches one dataset,
/// and that dataset's
/// section standing before it.
fn order_finding(
    facts: &CoverageFacts,
    config: &ReadConfig,
    dataset: &str,
    section_line: usize,
    findings: &mut Vec<CoverageFinding>,
) {
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
        findings.push(CoverageFinding::new(
            FIELD_SECTION_ORDER,
            detail,
            Some(move_section_correction(dataset, ancestor.as_str())),
        ));
    }
}
