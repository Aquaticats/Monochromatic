//! What:
//!  The healthy facts the coverage and scope controls change one field at a
//!  time.
//! Why:
//!  One builder keeps every control's starting point the same,
//!  so a test that sees one finding knows its own change made it,
//!  and no control reads this machine's configuration or datasets.
//!  The names are fixture names:
//!  nothing here is a dataset,
//!  a pool or a path of any machine.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = healthyFacts(); facts.tree.datasets[0].autoSnapshot = 'true'; // one field moved
//! ```

use crate::health::sanoid_coverage::{ConfigRefusal, CoverageDecision, CoverageFacts, Exclusion, ExclusionMount, MeasuredDataset, MeasuredTree, ReadConfig, SpaceGuardFacts, read_sanoid_conf};

/// The recursive ancestor the controls use.
pub(crate) const ROOT: &str = "tank/home";

/// The excluded dataset the controls mount at a folder.
pub(crate) const CACHE: &str = "tank/varcache";

/// The excluded container the controls leave unmounted.
pub(crate) const CONTAINER: &str = "tank/nosnap";

/// The excluded child the controls mount at a folder.
pub(crate) const NPM: &str = "tank/nosnap/npm";

/// The excluded source the controls leave unmounted.
pub(crate) const PROBE: &str = "tank/nosnap/probe";

/// The covered child dataset no exclusion names.
pub(crate) const WORKTREES: &str = "tank/home/worktrees";

/// The store path under the recursive ancestor's mount point.
pub(crate) const STORE: &str = "/fixture/home/.local/share/pnpm/store";

/// The configuration path every correction names.
pub(crate) const CONFIG_PATH: &str = "/fixture/etc/sanoid/sanoid.conf";

/// The coverage the controls decide:
/// the same shape as the host's,
/// with fixture names.
pub(crate) fn test_decision() -> CoverageDecision {
    let exclusions: Vec<Exclusion> = vec![
        Exclusion { dataset: String::from(CACHE), mount: ExclusionMount::Path },
        Exclusion { dataset: String::from(CONTAINER), mount: ExclusionMount::Unmounted },
        Exclusion { dataset: String::from(NPM), mount: ExclusionMount::Path },
        Exclusion { dataset: String::from(PROBE), mount: ExclusionMount::Unmounted },
    ];
    return CoverageDecision { recursive_root: String::from(ROOT), template: String::from("home"), exclusions };
}

/// One dataset's measured facts,
/// with the marker the decision writes unless a control moves it.
pub(crate) fn a_dataset(name: &str, mountpoint: Option<&str>, marker: Option<&str>) -> MeasuredDataset {
    return MeasuredDataset { name: String::from(name), mountpoint: mountpoint.map(String::from), auto_snapshot: marker.map(String::from) };
}

/// The measured tree of the decisions above,
/// every exclusion carrying `com.sun:auto-snapshot=false`.
pub(crate) fn healthy_tree() -> MeasuredTree {
    return MeasuredTree {
        datasets: vec![
            a_dataset(ROOT, Some("/fixture/home"), None),
            a_dataset(WORKTREES, Some("/fixture/home/.claude/worktrees"), None),
            a_dataset(CACHE, Some("/fixture/cache"), Some("false")),
            a_dataset(CONTAINER, None, Some("false")),
            a_dataset(NPM, Some("/fixture/home/.npm"), Some("false")),
            a_dataset(PROBE, Some("legacy"), Some("false")),
        ],
    };
}

/// One dataset of a tree,
/// so a control moves exactly that field.
pub(crate) fn dataset_in<'a>(tree: &'a mut MeasuredTree, name: &str) -> &'a mut MeasuredDataset {
    return tree.datasets.iter_mut().find(|dataset| return dataset.name == name).expect("the fixture holds this dataset");
}

/// A configuration text from section names and their settings,
/// in the order given.
pub(crate) fn config_text(sections: &[(&str, &[(&str, &str)])]) -> String {
    let mut text: String = String::new();
    for (dataset, settings) in sections {
        text.push_str(format!("[{dataset}]\n").as_str());
        for (key, value) in *settings {
            text.push_str(format!("  {key} = {value}\n").as_str());
        }
        text.push('\n');
    }
    return text;
}

/// The configuration the decision writes:
/// the ancestor first and every exclusion after it.
pub(crate) fn healthy_config_text() -> String {
    let decision: CoverageDecision = test_decision();
    let mut sections: Vec<(&str, Vec<(&str, &str)>)> = vec![(ROOT, vec![("use_template", "home"), ("recursive", "yes")])];
    for exclusion in &decision.exclusions {
        sections.push((exclusion.dataset.as_str(), vec![("autosnap", "no")]));
    }
    let borrowed: Vec<(&str, &[(&str, &str)])> = sections.iter().map(|(dataset, settings)| return (*dataset, settings.as_slice())).collect();
    return config_text(borrowed.as_slice());
}

/// The facts of a healthy machine.
pub(crate) fn healthy_facts() -> CoverageFacts {
    return facts_of(healthy_tree(), healthy_config_text().as_str());
}

/// The healthy tree with one configuration text.
pub(crate) fn facts_of(tree: MeasuredTree, text: &str) -> CoverageFacts {
    let config: Result<ReadConfig, ConfigRefusal> = read_sanoid_conf(text);
    return CoverageFacts { decision: test_decision(), tree, config_path: String::from(CONFIG_PATH), config, store_path: String::from(STORE) };
}

/// What a space guard reads on the healthy machine.
pub(crate) fn decided_guard() -> SpaceGuardFacts {
    return SpaceGuardFacts { configured_dataset: String::from(ROOT), pruned_prefix: String::from("autosnap_"), recursive: false, decision: test_decision(), tree: healthy_tree() };
}
