//! What:
//!  The coverage the project decided:
//!  the home dataset snapshotted recursively,
//!  the template it keeps,
//!  and the nine exclusion datasets with whether the decision mounts each.
//! Why:
//!  Step 20 owns the `sanoid.conf` text,
//!  and the excluded datasets are a decision rather than a reading:
//!  eight from the owner's answer of 2026-11-18
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Snapshots and setup reporting")
//!  and the start probe's own dataset as the ninth
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "Amendments made with this record",
//!  which amends the exclusion count).
//!  Each exclusion stands after the recursive section because sanoid copies a
//!  recursive section's settings onto every child without checking for a child
//!  section of its own,
//!  and the probe dataset is named although it is outside the home subtree,
//!  so a later recursive ancestor cannot silently include it.
//!  Nothing applies this text to a machine:
//!  a step prints it and the owner writes it
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 20,
//!  "What it builds").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const decision = decidedCoverage();
//! generatedText(decision); // the header and every section, in order
//! ```

use super::sanoid_section::{AUTOSNAP_NO, AUTOSNAP_SETTING, RECURSIVE_SETTING, RECURSIVE_YES, SanoidSection, SanoidSetting, TEMPLATE_SETTING, render_sections};

/// The dataset the decided recursive section names.
pub const RECURSIVE_ROOT: &str = "zpcachyos/ROOT/cos/home";

/// The retention template the decided recursive section keeps.
pub const TEMPLATE: &str = "home";

/// What:
///  Whether the decision mounts an excluded dataset at a path.
/// Why:
///  A dataset mounted at a path covers that folder,
///  while `none`,
/// `-` and `legacy` cover none,
///  so the two kinds drift in opposite directions and a check that ignored
///  the difference would pass an exclusion that protects nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExclusionMount = 'path' | 'unmounted';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExclusionMount {
    /// The decision mounts the dataset at a folder.
    Path,
    /// The decision leaves the dataset unmounted,
    /// as a container or a source another process attaches on demand.
    Unmounted,
}

/// What:
///  One excluded dataset and whether the decision mounts it.
/// Why:
///  The dataset is what a section names,
///  and the mount kind is what tells a folder exclusion from a container,
///  so both travel together rather than in parallel lists nothing keeps in
///  step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Exclusion = { dataset: string; mount: ExclusionMount };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Exclusion {
    /// The excluded dataset's whole name.
    pub dataset: String,
    /// Whether the decision mounts it at a folder.
    pub mount: ExclusionMount,
}

/// What:
///  The coverage one machine's configuration should hold.
/// Why:
///  The check compares a reading with a decision,
///  and the decision carries every name it needs,
///  so the same check runs on the guest's layout with the guest's names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CoverageDecision = { recursiveRoot: string; template: string; exclusions: Exclusion[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageDecision {
    /// The dataset the recursive section names.
    pub recursive_root: String,
    /// The retention template that section keeps.
    pub template: String,
    /// The excluded datasets in the order the sections are written.
    pub exclusions: Vec<Exclusion>,
}

/// What:
///  This host's decided coverage,
///  as the design and the start probe's decision name it.
/// Why:
///  A generated text and a printed correction have to be the decision itself,
///  not one writer's recollection of it,
///  so the names stand in one function a control can assert whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decidedCoverage(); // { recursiveRoot: 'zpcachyos/ROOT/cos/home', template: 'home', exclusions: [...] }
/// ```
pub fn decided_coverage() -> CoverageDecision {
    let exclusions: Vec<Exclusion> = [
        ("zpcachyos/ROOT/cos/varcache", ExclusionMount::Path),
        ("zpcachyos/nosnap", ExclusionMount::Unmounted),
        ("zpcachyos/nosnap/cargo-build", ExclusionMount::Path),
        ("zpcachyos/nosnap/cargo-registry", ExclusionMount::Path),
        ("zpcachyos/nosnap/npm", ExclusionMount::Path),
        ("zpcachyos/nosnap/user-cache", ExclusionMount::Path),
        ("zpcachyos/nosnap/vm-builder-output", ExclusionMount::Path),
        ("zpcachyos/nosnap/libvirt-images", ExclusionMount::Path),
        ("zpcachyos/nosnap/git-worktree-probe", ExclusionMount::Unmounted),
    ]
    .into_iter()
    .map(|(dataset, mount)| return Exclusion { dataset: String::from(dataset), mount })
    .collect();
    return CoverageDecision { recursive_root: String::from(RECURSIVE_ROOT), template: String::from(TEMPLATE), exclusions };
}

impl CoverageDecision {
    /// Whether one dataset is one of the decision's exclusions.
    pub fn excludes(&self, dataset: &str) -> bool {
        return self.exclusions.iter().any(|exclusion| return exclusion.dataset == dataset);
    }
}

/// The comment lines a generated block starts with,
/// which the design's own sample carries above its sections.
pub const GENERATED_HEADER: &str = "# /etc/sanoid/sanoid.conf (the sections this route decides)\n# Use yes, not zfs. Every exclusion stands after the section of each dataset above it.\n";

/// What:
///  The recursive ancestor's section,
///  which is also the correction a person adds for it.
/// Why:
///  The ancestor section is the one section the exclusions' order is measured
///  against,
///  so the check names exactly the text it wanted to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rootSection(decision); // { dataset: root, settings: [{ key: 'use_template', value: 'home' }, { key: 'recursive', value: 'yes' }] }
/// ```
pub fn root_section(decision: &CoverageDecision) -> SanoidSection {
    let settings: Vec<SanoidSetting> = vec![
        SanoidSetting { key: String::from(TEMPLATE_SETTING), value: decision.template.clone() },
        SanoidSetting { key: String::from(RECURSIVE_SETTING), value: String::from(RECURSIVE_YES) },
    ];
    return SanoidSection::new(decision.recursive_root.as_str(), settings);
}

/// What:
///  The sections the decision writes,
///  in the order they take effect.
/// Why:
///  The recursive ancestor stands first and each exclusion after it,
///  which is the one order a child section can win in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sectionsOf(decision); // [root, ...exclusions]
/// ```
pub fn sections_of(decision: &CoverageDecision) -> Vec<SanoidSection> {
    let mut sections: Vec<SanoidSection> = vec![root_section(decision)];
    for exclusion in &decision.exclusions {
        sections.push(exclusion_section(exclusion.dataset.as_str()));
    }
    return sections;
}

/// What:
///  One exclusion's section,
///  which is also the correction a person adds for it.
/// Why:
///  The check names the section it wanted to read,
///  and a person pastes exactly that text rather than inferring it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exclusionSection('tank/home/scratch'); // { dataset, settings: [{ key: 'autosnap', value: 'no' }] }
/// ```
pub fn exclusion_section(dataset: &str) -> SanoidSection {
    let settings: Vec<SanoidSetting> = vec![SanoidSetting { key: String::from(AUTOSNAP_SETTING), value: String::from(AUTOSNAP_NO) }];
    return SanoidSection::new(dataset, settings);
}

/// What:
///  The whole configuration text the decision writes,
///  ready to paste.
/// Why:
///  The design's sample is a text,
///  and a person setting the route up needs it as one block with the header
///  that says where it goes and which order the sections keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// generatedText(decision); // '# /etc/sanoid/sanoid.conf ...\n[tank/home]\n...'
/// ```
pub fn generated_text(decision: &CoverageDecision) -> String {
    let mut text: String = String::from(GENERATED_HEADER);
    text.push('\n');
    text.push_str(render_sections(sections_of(decision).as_slice()).as_str());
    return text;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "decided_coverage_tests.rs"]
mod tests;
