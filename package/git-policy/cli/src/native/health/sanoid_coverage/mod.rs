//! What:
//!  Plan step 20 of the ZFS route,
//!  built ahead and unwired:
//!  the `sanoid.conf` sections that cover home recursively and name the nine
//!  exclusions in the order that takes effect,
//!  the `com.sun:auto-snapshot=false` marker every exclusion carries,
//!  `doctor`'s comparison of the measured dataset tree and store against that
//!  coverage with the concrete correction for confirmed drift,
//!  and the report of what the space guard can actually prune.
//! Why:
//!  Three silences cost space or history:
//!  an exclusion section written before its recursive ancestor is snapshotted
//!  anyway,
//!  `recursive = zfs` cannot leave any child out,
//!  and the guard prunes one dataset's `autosnap_*` names only,
//!  so recursive home snapshots outgrow what it can free
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Snapshots and setup reporting";
//!  `doc/planning/cli-git-zfs-worktree.md`,
//!  lines 355 to 385).
//!  Every check takes its facts as a value,
//!  so the wording and the decision are tested on built facts and a control
//!  never opens this machine's `/dev/zfs`,
//!  its configuration or its datasets,
//!  and nothing here is applied to a machine:
//!  writing `sanoid.conf`,
//!  setting the marker or changing a property is the owner's act on the
//!  owner's authorisation,
//!  printed by `doctor` and never run by this code.
//!  Nothing on a command path,
//!  the mount handler or a boot unit reaches this folder until step 20 is
//!  wired,
//!  which is the project's answer of 2026-11-23
//!  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`,
//!  "Building later pieces ahead of the plan's order").
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `sanoid_section.rs`:
//!   one section and one setting,
//!   the words sanoid reads a toggle as,
//!   and the text a person pastes.
//! - `sanoid_conf_reading.rs`:
//!   a configuration text's sections in file order and the reasons one cannot
//!   be proved.
//! - `measured_dataset_tree.rs`:
//!   the datasets one machine measured,
//!   their ancestry,
//!   their mount points and the dataset that holds a path.
//! - `decided_coverage.rs`:
//!   the recursive ancestor,
//!   the template and the nine exclusions the project decided,
//!   and the text that writes them.
//! - `coverage_finding.rs`:
//!   one drift,
//!   the check's answer and the lines a report prints.
//! - `coverage_words.rs`:
//!   the corrections and the sentences that read a measured value.
//! - `coverage_facts.rs`:
//!   what one coverage comparison is asked with,
//!   and the check's stable name in a report line.
//! - `coverage_settings_not_modelled.rs`:
//!   the settings a relevant section writes that this build cannot judge,
//!   which leave the coverage unproved rather than passed.
//! - `root_section_findings.rs`:
//!   the recursive ancestor's section,
//!   its recursion and its template.
//! - `exclusion_findings.rs`:
//!   one excluded dataset:
//!   its presence,
//!   its mount point,
//!   its section,
//!   its `autosnap` value and its place in file order.
//! - `store_coverage_finding.rs`:
//!   the dataset whose mount point holds the store,
//!   and the finding when the coverage does not reach it.
//! - `coverage_checks.rs`:
//!   the comparison itself:
//!   the order of the checks and the answer they build.
//! - `space_guard_scope.rs`:
//!   what the space guard can prune and what it cannot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const generated = generatedText(decidedCoverage());
//! const coverage = checkCoverage(facts);   // { outcome, findings, unproved }
//! const scope = checkSpaceGuardScope(guard); // { outcome, lines }
//! ```

/// One `sanoid.conf` section,
/// one setting,
/// the toggle words and the text a
/// correction adds.
pub mod sanoid_section;

/// A configuration text's sections in file order and its refusals.
pub mod sanoid_conf_reading;

/// The datasets one machine measured and their ancestry.
pub mod measured_dataset_tree;

/// The coverage the project decided and the text that writes it.
pub mod decided_coverage;

/// One drift,
/// the check's answer and its report lines.
pub mod coverage_finding;

/// The corrections and the sentences that read a measured value.
pub mod coverage_words;

/// What one coverage comparison is asked with,
/// and the check's stable name.
pub mod coverage_facts;

/// The settings a relevant section writes that this build cannot judge.
pub mod coverage_settings_not_modelled;

/// The recursive ancestor's section,
/// its recursion and its template.
pub mod root_section_findings;

/// One excluded dataset's findings.
pub mod exclusion_findings;

/// The dataset whose mount point holds the store.
pub mod store_coverage_finding;

/// The comparison of a measured tree and configuration with the decision.
pub mod coverage_checks;

/// What the space guard can prune and what it cannot.
pub mod space_guard_scope;

/// The healthy facts every control in this folder changes one field at a time,
/// never part of the release executable.
#[cfg(test)]
#[path = "coverage_test_facts.rs"]
pub(crate) mod coverage_test_facts;

/// The comparison of a measured tree and configuration with the decision.
pub use coverage_checks::{
    COVERAGE_CHECK_NAME,
    CoverageFacts,
    check_coverage,
};
/// One drift and the check's answer.
pub use coverage_finding::{
    CoverageAnswer,
    CoverageFinding,
    FIELD_EXCLUSION_AUTOSNAP,
    FIELD_EXCLUSION_DATASET,
    FIELD_EXCLUSION_MARKER,
    FIELD_EXCLUSION_MOUNT,
    FIELD_EXCLUSION_SECTION,
    FIELD_RECURSION,
    FIELD_ROOT_SECTION,
    FIELD_SECTION_ORDER,
    FIELD_STORE_COVERAGE,
    FIELD_TEMPLATE,
};
/// The corrections and the sentences that read a measured value.
pub use coverage_words::{
    AUTO_SNAPSHOT_OFF,
    AUTO_SNAPSHOT_PROPERTY,
    marker_correction,
    mountpoint_words,
};
/// The decided coverage and the text that writes it.
pub use decided_coverage::{
    CoverageDecision,
    Exclusion,
    ExclusionMount,
    GENERATED_HEADER,
    RECURSIVE_ROOT,
    TEMPLATE,
    decided_coverage,
    exclusion_section,
    generated_text,
    root_section,
    sections_of,
};
/// The datasets one machine measured and their ancestry.
pub use measured_dataset_tree::{
    MeasuredDataset,
    MeasuredTree,
    has_path_mountpoint,
    is_under,
};
/// A configuration text's sections,
/// its duplicates and its refusals.
pub use sanoid_conf_reading::{
    ConfigRefusal,
    DuplicateKey,
    ReadConfig,
    ReadSection,
    ReadSetting,
    SettingLookup,
    read_sanoid_conf,
};
/// One section,
/// one setting and the words sanoid reads a toggle as.
pub use sanoid_section::{
    AUTOSNAP_NO,
    AUTOSNAP_SETTING,
    FALSE_WORDS,
    PATH_SETTING,
    PROCESS_CHILDREN_ONLY_SETTING,
    RECURSIVE_SETTING,
    RECURSIVE_YES,
    RECURSIVE_ZFS,
    Recursion,
    SKIP_CHILDREN_SETTING,
    SanoidSection,
    SanoidSetting,
    TEMPLATE_SETTING,
    TRUE_WORDS,
    is_false_word,
    is_true_word,
    recursion_of,
    render_section,
    render_sections,
};
/// What the space guard can prune and what it cannot.
pub use space_guard_scope::{
    FIELD_GUARD_DATASET,
    FIELD_GUARD_RECURSION,
    SCOPE_CHECK_NAME,
    ScopeAnswer,
    SpaceGuardFacts,
    check_space_guard_scope,
};
