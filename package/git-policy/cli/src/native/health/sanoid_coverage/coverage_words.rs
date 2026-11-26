//! What:
//!  The sentences a coverage finding's correction prints:
//!  the text to add,
//!  the value to change,
//!  the section to move,
//!  and the `zfs set` that writes an exclusion's marker,
//!  plus how a measured mount point reads.
//! Why:
//!  A correction is pasted into a file or run as a command,
//!  so each one is built from the same names the check compared and never
//!  from a recollection of the design
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints":
//!  "print the concrete INI correction or `zfs set …` for confirmed drift").
//!  The mount-point sentence keeps `none`,
//!  `-` and `legacy` apart from a path and from a value nobody read,
//!  because the finding that prints it is about which of those the machine
//!  said.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! addSectionCorrection('/etc/sanoid/sanoid.conf', section); // 'add these lines to ...:\n[tank/home]\n  autosnap = no\n'
//! ```

use super::measured_dataset_tree::has_path_mountpoint;
use super::sanoid_section::{
    AUTOSNAP_NO,
    AUTOSNAP_SETTING,
    RECURSIVE_SETTING,
    RECURSIVE_YES,
    SanoidSection,
    TEMPLATE_SETTING,
    render_section,
};

/// The property that marks an exclusion for the snapshotting tools.
pub const AUTO_SNAPSHOT_PROPERTY: &str = "com.sun:auto-snapshot";

/// The value the decision writes to that property on every exclusion.
pub const AUTO_SNAPSHOT_OFF: &str = "false";

/// How a measured mount point reads in a finding.
pub fn mountpoint_words(mountpoint: Option<&str>) -> String {
    let Some(written) = mountpoint else {
        return String::from("no mount point this build read");
    };
    if has_path_mountpoint(Some(written)) {
        return format!("the path {written}");
    }
    return format!("the value {written}, which covers no folder");
}

/// The correction that adds one section and where it goes.
pub fn add_section_correction(
    config_path: &str,
    section: &SanoidSection,
) -> String {
    return format!(
        "add these lines to {config_path}:\n{}",
        render_section(section)
    );
}

/// The correction that changes the recursive ancestor's recursion value.
pub fn change_recursion_correction(
    config_path: &str,
    dataset: &str,
    written: &str,
) -> String {
    return format!(
        "in the [{dataset}] section of {config_path}, change `{RECURSIVE_SETTING} = {written}` to `{RECURSIVE_SETTING} = {RECURSIVE_YES}`"
    );
}

/// The correction that adds a recursion value to a section that writes none.
pub fn add_recursion_correction(
    config_path: &str,
    dataset: &str,
) -> String {
    return format!(
        "add `{RECURSIVE_SETTING} = {RECURSIVE_YES}` to the [{dataset}] section of {config_path}"
    );
}

/// The correction that names the retention template in the ancestor's section.
pub fn template_correction(
    config_path: &str,
    dataset: &str,
    template: &str,
    written: Option<&str>,
) -> String {
    let Some(value) = written else {
        return format!(
            "add `{TEMPLATE_SETTING} = {template}` to the [{dataset}] section of {config_path}"
        );
    };
    return format!(
        "in the [{dataset}] section of {config_path}, change `{TEMPLATE_SETTING} = {value}` to name `{template}`"
    );
}

/// The correction that turns an exclusion's snapshots off.
pub fn change_autosnap_correction(
    config_path: &str,
    dataset: &str,
    written: &str,
) -> String {
    return format!(
        "in the [{dataset}] section of {config_path}, change `{AUTOSNAP_SETTING} = {written}` to `{AUTOSNAP_SETTING} = {AUTOSNAP_NO}`"
    );
}

/// How a measured `com.sun:auto-snapshot` value reads in a finding.
pub fn marker_words(marker: Option<&str>) -> String {
    let Some(written) = marker else {
        return String::from("is not set");
    };
    return format!("reads {written}");
}

/// The correction that writes an exclusion's marker on the machine.
pub fn marker_correction(dataset: &str) -> String {
    return format!("zfs set {AUTO_SNAPSHOT_PROPERTY}={AUTO_SNAPSHOT_OFF} {dataset}");
}

/// The correction that moves one section below the section whose recursion
/// reaches it.
pub fn move_section_correction(
    dataset: &str,
    ancestor: &str,
) -> String {
    return format!("move the [{dataset}] section below the [{ancestor}] section");
}

/// The pass line's account of what the comparison read.
pub fn pass_detail(
    covered: usize,
    root: &str,
    exclusions: usize,
    store_path: &str,
    holder: &str,
) -> String {
    return format!(
        "{covered} dataset(s) under {root} are covered by its recursive section, {exclusions} exclusion(s) stand after every recursive ancestor with {AUTO_SNAPSHOT_PROPERTY}={AUTO_SNAPSHOT_OFF}, and the store at {store_path} is held by {holder}"
    );
}
