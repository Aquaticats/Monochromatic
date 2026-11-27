//! What:
//!  The settings a relevant section writes that this build does not model,
//!  which leave the coverage unproved rather than passed.
//! Why:
//!  sanoid reads keys this build never judges,
//!  and a section that writes `skip_children` or a key in another spelling may
//!  change what is snapshotted,
//!  so the comparison says so instead of reporting a pass it cannot stand
//!  behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reason: string | undefined = unprovedSetting(facts, config);
//! ```

/// The facts the comparison reads.
use super::coverage_facts::CoverageFacts;
/// One configuration text read into sections,
/// and the lookup a section's setting answers.
use super::sanoid_conf_reading::{
    ReadConfig,
    SettingLookup,
};
/// The settings this build reads and the ones it only recognizes.
use super::sanoid_section::{
    AUTOSNAP_SETTING,
    PATH_SETTING,
    PROCESS_CHILDREN_ONLY_SETTING,
    RECURSIVE_SETTING,
    SKIP_CHILDREN_SETTING,
    TEMPLATE_SETTING,
};
/// What one relevant section says that this build does not model.
pub(super) fn unproved_setting(
    facts: &CoverageFacts,
    config: &ReadConfig,
) -> Option<String> {
    let mut datasets: Vec<&str> = vec![facts.decision.recursive_root.as_str()];
    for exclusion in &facts.decision.exclusions {
        datasets.push(exclusion.dataset.as_str());
    }
    for dataset in datasets {
        let Some(section) = config.section(dataset) else {
            continue;
        };
        for key in [
            RECURSIVE_SETTING,
            TEMPLATE_SETTING,
            AUTOSNAP_SETTING,
            PATH_SETTING,
            PROCESS_CHILDREN_ONLY_SETTING,
            SKIP_CHILDREN_SETTING,
        ] {
            match section.setting(key) {
                SettingLookup::CaseVariant {
                    spelling,
                    line,
                } => {
                    return Some(format!(
                        "the [{dataset}] section writes `{spelling}` on line {line}, which is not the key `{key}` sanoid reads, and this build cannot prove what sanoid does with it"
                    ));
                }
                SettingLookup::Found {
                    value,
                    line,
                } if matches!(
                    key,
                    PATH_SETTING | PROCESS_CHILDREN_ONLY_SETTING | SKIP_CHILDREN_SETTING
                ) =>
                {
                    return Some(format!(
                        "the [{dataset}] section writes `{key} = {value}` on line {line}, and this build does not model what that changes about coverage"
                    ));
                }
                SettingLookup::Found {
                    ..
                }
                | SettingLookup::Absent => continue,
            }
        }
    }
    return None;
}
