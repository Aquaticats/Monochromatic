//! What:
//!  The recursive ancestor's section:
//!  whether it is there,
//!  how it recurses and which template it keeps.
//! Why:
//!  Every exclusion is a child of this dataset,
//!  so a section that is missing,
//!  that takes one recursive snapshot,
//!  or that keeps another template leaves either every child or none of them
//!  covered,
//!  which is the drift a person fixes in the configuration text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! rootFindings(facts, config, findings);
//! ```

/// The facts the comparison reads.
use super::coverage_facts::CoverageFacts;
/// One finding,
/// and the field names its report lines carry.
use super::coverage_finding::{
    CoverageFinding,
    FIELD_RECURSION,
    FIELD_ROOT_SECTION,
    FIELD_TEMPLATE,
};
/// The corrections a person applies to the configuration text.
use super::coverage_words::{
    add_recursion_correction,
    add_section_correction,
    change_recursion_correction,
    template_correction,
};
/// The text the decision writes for the recursive ancestor.
use super::decided_coverage::root_section;
/// One configuration text read into sections,
/// and the lookup a section's setting answers.
use super::sanoid_conf_reading::{
    ReadConfig,
    SettingLookup,
};
/// How a section recurses and the settings it is read for.
use super::sanoid_section::{
    RECURSIVE_SETTING,
    RECURSIVE_ZFS,
    Recursion,
    TEMPLATE_SETTING,
    recursion_of,
};
/// The recursive ancestor's section,
/// its recursion and its template.
pub(super) fn root_findings(
    facts: &CoverageFacts,
    config: &ReadConfig,
    findings: &mut Vec<CoverageFinding>,
) {
    let root: &str = facts.decision.recursive_root.as_str();
    let Some(section) = config.section(root) else {
        let detail: String = format!(
            "the configuration has no [{root}] section, so no dataset under {root} is covered"
        );
        findings.push(CoverageFinding::new(
            FIELD_ROOT_SECTION,
            detail,
            Some(add_section_correction(
                facts.config_path.as_str(),
                &root_section(&facts.decision),
            )),
        ));
        return;
    };
    match recursion_of(section.written_value(RECURSIVE_SETTING)) {
        Recursion::Yes => {}
        Recursion::Zfs => {
            let detail: String = format!(
                "the [{root}] section sets `{RECURSIVE_SETTING} = {RECURSIVE_ZFS}`, so sanoid takes one recursive snapshot of {root} and every child dataset is in it, which no exclusion can be left out of"
            );
            findings.push(CoverageFinding::new(
                FIELD_RECURSION,
                detail,
                Some(change_recursion_correction(
                    facts.config_path.as_str(),
                    root,
                    RECURSIVE_ZFS,
                )),
            ));
        }
        Recursion::Off {
            value: None,
        } => {
            let detail: String = format!(
                "the [{root}] section writes no `{RECURSIVE_SETTING}` value, so only {root} itself is snapshotted and its child datasets are not"
            );
            findings.push(CoverageFinding::new(
                FIELD_RECURSION,
                detail,
                Some(add_recursion_correction(facts.config_path.as_str(), root)),
            ));
        }
        Recursion::Off {
            value: Some(written),
        } => {
            let detail: String = format!(
                "the [{root}] section writes `{RECURSIVE_SETTING} = {written}`, which sanoid does not read as true, so its child datasets are not covered"
            );
            findings.push(CoverageFinding::new(
                FIELD_RECURSION,
                detail,
                Some(change_recursion_correction(
                    facts.config_path.as_str(),
                    root,
                    written.as_str(),
                )),
            ));
        }
    }
    match section.setting(TEMPLATE_SETTING) {
        SettingLookup::Found {
            value,
            ..
        } => {
            if !value
                .split(',')
                .any(|name| return name.trim() == facts.decision.template)
            {
                let detail: String = format!(
                    "the [{root}] section keeps the template `{value}`, which does not name `{}`",
                    facts.decision.template
                );
                findings.push(CoverageFinding::new(
                    FIELD_TEMPLATE,
                    detail,
                    Some(template_correction(
                        facts.config_path.as_str(),
                        root,
                        facts.decision.template.as_str(),
                        Some(value.as_str()),
                    )),
                ));
            }
        }
        SettingLookup::Absent => {
            let detail: String = format!(
                "the [{root}] section names no template, so the retention this route decided for it does not apply"
            );
            findings.push(CoverageFinding::new(
                FIELD_TEMPLATE,
                detail,
                Some(template_correction(
                    facts.config_path.as_str(),
                    root,
                    facts.decision.template.as_str(),
                    None,
                )),
            ));
        }
        SettingLookup::CaseVariant {
            ..
        } => {}
    }
}
