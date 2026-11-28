//! What:
//!  The reading every unit text's control shares:
//!  one INI section's directive lines,
//!  with comments and blank lines dropped,
//!  and the shape every file of this folder must have.
//! Why:
//!  an assertion that searched the whole file would find `[Unit]`'s
//!  `After=` inside `[Service]`'s text or a line a comment carries,
//!  so each control names the section it read.
//!  The shape check is one place because a file missing its last newline,
//!  or one still carrying a `<placeholder>`,
//!  is wrong whichever unit it is.
//!  The classification of a plan's files is here for the guest arm's sake:
//!  `systemd-analyze verify` refuses a drop-in path by name
//!  (`Failed to prepare filename .../checkout.service.d/x.conf: Invalid argument`,
//!  measured in the step 19 guest of 2026-11-27),
//!  so the arm verifies the plan's unit files and reads the drop-ins back from
//!  the manager,
//!  and this answer is what keeps the two apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesOf(text, '[Unit]')).toContain('Before=umount.target');
//! ```

use crate::checkout_boot_units::boot_unit_facts::{
    CheckoutBootFacts,
    InstalledUnitFile,
    NestedDatasetMount,
};
use crate::checkout_boot_units::boot_unit_install_plan::install_plan;
use std::path::{
    Path,
    PathBuf,
};

/// The directive lines of one section,
/// trimmed,
/// with comments and blank lines dropped.
///
/// Why:
///  the caller compares whole `key=value` lines,
///  so whitespace around them must not decide the comparison.
pub(crate) fn lines_of(
    text: &str,
    section: &str,
) -> Vec<String> {
    let mut inside: bool = false;
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            inside = trimmed == section;
            continue;
        }
        if !inside || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        lines.push(String::from(trimmed));
    }
    return lines;
}

/// Whether one section carries a line that begins with the given words.
///
/// Why:
///  a line such as `Last=` names an argument a control does not want to
///  repeat,
/// so the assertion reads the beginning and the rest is the text's.
pub(crate) fn carries(
    text: &str,
    section: &str,
    beginning: &str,
) -> bool {
    return lines_of(text, section)
        .iter()
        .any(|line| return line.starts_with(beginning));
}

/// What:
///  Which of a plan's paths `systemd-analyze verify` can read at all:
///  the unit files,
///  and never the drop-ins under a `<unit>.d` folder.
/// Why:
///  the verifier's first argument is a unit file,
/// and a drop-in path answers
///  `Failed to prepare filename ...: Invalid argument` on every systemd tested,
///  so an arm that handed it one proved nothing about that file and could not
///  pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verifiable = plan.filter((file) => !file.path.parent.endsWith('.d'));
/// ```
pub(crate) fn unit_files_of(plan: &[InstalledUnitFile]) -> Vec<PathBuf> {
    return plan
        .iter()
        .filter(|file| return !is_drop_in(file.path.as_path()))
        .map(|file| return file.path.clone())
        .collect();
}

/// Whether one plan path is a drop-in,
/// which is a file inside a folder named `<unit>.d`.
///
/// Why:
///  the folder is the one fact that separates a file the manager merges into a
///  unit from a unit of its own,
/// and the install paths are built from it.
pub(crate) fn is_drop_in(path: &Path) -> bool {
    return path
        .parent()
        .and_then(|folder| return folder.file_name())
        .and_then(|name| return name.to_str())
        .is_some_and(|name| return name.ends_with(".d"));
}

/// Whether one section carries the whole line.
pub(crate) fn holds(
    text: &str,
    section: &str,
    wanted: &str,
) -> bool {
    return lines_of(text, section)
        .iter()
        .any(|line| return line == wanted);
}

/// Assert one file of this folder has the shape every one of them must have:
/// one final newline,
/// no second one,
/// no carriage return,
/// no tab,
/// and no `<placeholder>` left in it.
///
/// Why:
///  a leftover placeholder is a unit line naming a file nothing installed,
///  and a tab or carriage return is a byte systemd and a reader disagree about.
pub(crate) fn assert_file_shape(text: &str) {
    assert!(text.ends_with('\n'), "the file ends with one newline");
    assert!(!text.ends_with("\n\n"), "the file carries no blank tail");
    assert!(!text.contains('\r'), "the file carries no carriage return");
    assert!(!text.contains('\t'), "the file carries no tab");
    assert!(
        !text.contains('<'),
        "the file carries no placeholder: {text}"
    );
    assert!(
        !text.contains('>'),
        "the file carries no placeholder: {text}"
    );
}

#[test]
fn the_plan_s_units_and_drop_ins_are_told_apart() {
    let facts: CheckoutBootFacts = crate::checkout_boot_units::boot_unit_facts::tests::facts_of(
        "/home/user/Monochromatic",
        "/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets",
    );
    let nested: NestedDatasetMount = crate::checkout_boot_units::boot_unit_facts::tests::nested_of(
        "zpcachyos/nosnap/vm-builder-output",
        "/home/user/Monochromatic/package/vm-builder/cli/output",
        "output",
    );
    let plan: Vec<InstalledUnitFile> =
        install_plan(&facts, &[nested]).unwrap_or_else(|refusal| panic!("{}", refusal.message()));
    let units: Vec<PathBuf> = unit_files_of(&plan);
    for path in &units {
        assert!(
            !is_drop_in(path.as_path()),
            "the verifier's list holds no drop-in: {}",
            path.display()
        );
    }
    assert_eq!(
        units.len(),
        3,
        "the gate, the propagation service and the nested dataset's unit are the plan's unit files; the list read {:?}",
        units
    );
    let drop_ins: usize = plan.len() - units.len();
    assert_eq!(drop_ins, 3, "the plan writes three drop-ins");
}

#[test]
fn the_reader_names_a_section_and_drops_comments_and_blanks() {
    let text: &str =
        "# a comment\n[Unit]\nDescription=x\n\n# inside\nWants=y\n[Service]\nType=oneshot\n";
    assert_eq!(
        lines_of(text, "[Unit]"),
        vec![String::from("Description=x"), String::from("Wants=y")]
    );
    assert_eq!(
        lines_of(text, "[Service]"),
        vec![String::from("Type=oneshot")]
    );
    assert!(carries(text, "[Unit]", "Wants="));
    assert!(
        !holds(text, "[Unit]", "Wants=y "),
        "a line is compared whole"
    );
    assert!(holds(text, "[Unit]", "Wants=y"));
}
