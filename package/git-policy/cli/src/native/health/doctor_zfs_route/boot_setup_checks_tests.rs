//! What:
//!  Controls of the boot setup's check:
//!  the healthy setup passes and names every decided line,
//!  each line the gate lost answers its own finding,
//!  each file state answers its own finding or unknown,
//!  and a dataset left `canmount`-mounted answers the command V4 decided.
//! Why:
//!  the check stands where a person learns whether the machine's boot carries
//!  the setup they were told to install,
//!  so every line and file state is asserted on its own,
//!  and the property beside them:
//!  a branch nobody tests is a machine state that reads as a pass
//!  (`AGENTS.md` rules PKG and TCV).
//!  The texts these controls judge are this build's own plan,
//!  built by the producer and then changed in the one line a control is about,
//!  so no control asserts a unit text somebody typed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = healthy(); facts.others[0].atThePath = { kind: 'absent' };
//! expect(checkServiceUnit(facts)).toMatchObject({ kind: 'finding', field: 'installed-file' });
//! ```

use super::{
    check_service_unit,
    first_difference,
    words_after,
};
use crate::checkout_boot_units::boot_unit_facts::{
    CheckoutBootFacts,
    InstalledUnitFile,
    NestedDatasetMount,
};
use crate::checkout_boot_units::boot_unit_install_plan::install_plan;
use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_PATH,
    GATE_START_BOUND,
    GATE_STOP_BOUND,
    PROVIDER_UNIT_NAME,
};
use crate::health::doctor_zfs_route::boot_setup_facts::{
    BootSetupFacts,
    FileAtInstallPath,
    NestedDatasetSetting,
    PlannedFileReading,
};
use crate::health::doctor_zfs_route::check_outcome::{
    Check,
    CheckOutcome,
    CheckReport,
};
use crate::health::doctor_zfs_route::zfs_checks_limits::{
    CANMOUNT_NOAUTO,
    CANMOUNT_ON,
};
use crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;
use std::path::{
    Path,
    PathBuf,
};

/// The dataset this machine keeps inside the checkout.
const NESTED_DATASET: &str = "zpcachyos/nosnap/vm-builder-output";

/// The route's boot facts as the design states them on this machine.
fn boot() -> CheckoutBootFacts {
    return CheckoutBootFacts {
        checkout: PathBuf::from("/home/user/Monochromatic"),
        root_file: PathBuf::from(ROOT_MODE_INSTALL_PATH),
        key_unit: None,
    };
}

/// The one dataset this machine keeps inside the checkout,
/// as the plan is given it.
fn mount() -> NestedDatasetMount {
    return NestedDatasetMount {
        dataset: String::from(NESTED_DATASET),
        mountpoint: PathBuf::from("/home/user/Monochromatic/package/vm-builder/cli/output"),
        slug: String::from("output"),
    };
}

/// The files this build's plan writes,
/// read back as a machine where a setup run installed them.
fn planned(
    boot_facts: &CheckoutBootFacts,
    nested: &[NestedDatasetMount],
) -> Vec<PlannedFileReading> {
    let plan: Vec<InstalledUnitFile> =
        install_plan(boot_facts, nested).expect("the route's own plan");
    return plan
        .into_iter()
        .map(|file| {
            return PlannedFileReading {
                at_the_path: FileAtInstallPath::Content(file.text.clone()),
                path: file.path,
                planned_text: file.text,
            };
        })
        .collect();
}

/// The gate and the files beside it,
/// from one planned list.
fn split(files: Vec<PlannedFileReading>) -> (PlannedFileReading, Vec<PlannedFileReading>) {
    let gate_path: &Path = Path::new(BOOT_GATE_UNIT_PATH);
    let mut gate: Option<PlannedFileReading> = None;
    let mut others: Vec<PlannedFileReading> = Vec::new();
    for file in files {
        if file.path == gate_path {
            gate = Some(file);
        } else {
            others.push(file);
        }
    }
    return (gate.expect("the plan writes the gate"), others);
}

/// The one dataset inside the checkout,
/// set the way V4 decides.
fn nested() -> NestedDatasetSetting {
    return NestedDatasetSetting {
        dataset: String::from(NESTED_DATASET),
        canmount: PropertyValue::Number(CANMOUNT_NOAUTO),
    };
}

/// The healthy setup:
/// every planned file installed unchanged and the dataset `noauto`.
fn healthy() -> BootSetupFacts {
    let (gate, others) = split(planned(&boot(), &[mount()]));
    return BootSetupFacts {
        gate,
        others,
        key_unit: None,
        nested: vec![nested()],
    };
}

/// The healthy setup with one line taken out of the gate's text,
/// which is the state an older release or a hand edit leaves.
fn without_gate_line(line: &str) -> BootSetupFacts {
    let mut facts: BootSetupFacts = healthy();
    let shortened: String = facts
        .gate
        .planned_text
        .replace(format!("{line}\n").as_str(), "");
    facts.gate.planned_text = shortened.clone();
    facts.gate.at_the_path = FileAtInstallPath::Content(shortened);
    return facts;
}

/// The answer as the line a report prints.
fn line(outcome: &CheckOutcome) -> String {
    let report: CheckReport = CheckReport::new(Check::ServiceUnit, outcome.clone());
    return report.line();
}

/// The healthy setup passes and names the provider,
/// both bounds and every file beside the gate,
/// so a person compares them with their own notes.
#[test]
fn the_healthy_boot_setup_passes_and_names_every_decided_line() {
    let facts: BootSetupFacts = healthy();
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(matches!(outcome, CheckOutcome::Pass { .. }), "{text}");
    for word in [
        PROVIDER_UNIT_NAME,
        GATE_START_BOUND,
        GATE_STOP_BOUND,
        "canmount=noauto",
        "has no key-loading unit",
        format!("the {} other file(s)", facts.others.len()).as_str(),
    ] {
        assert!(text.contains(word), "{word} is in {text}");
    }
    for file in &facts.others {
        assert!(
            text.contains(file.path.display().to_string().as_str()),
            "{} is in {text}",
            file.path.display()
        );
    }
}

/// A key-loading unit the machine has is named,
/// and the gate's own key lines are what proves it is required and ordered.
#[test]
fn a_key_loading_unit_the_machine_has_is_named() {
    let mut facts: BootSetupFacts = healthy();
    let key: String = String::from("zfs-load-key@zfsstep19-home.service");
    facts.key_unit = Some(key.clone());
    let mut gate: CheckoutBootFacts = boot();
    gate.key_unit = Some(key.clone());
    let (planned_gate, others) = split(planned(&gate, &[mount()]));
    facts.gate = planned_gate;
    facts.others = others;
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(matches!(outcome, CheckOutcome::Pass { .. }), "{text}");
    assert!(text.contains(format!("`{key}` exists").as_str()), "{text}");
}

/// A machine with a key unit whose gate does not name it is a finding,
/// because a gate can then run before the key is loaded.
#[test]
fn a_gate_that_does_not_name_the_key_unit_is_a_finding() {
    let mut facts: BootSetupFacts = healthy();
    facts.key_unit = Some(String::from("zfs-load-key@zfsstep19-home.service"));
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "key-unit"),
        "{text}"
    );
    assert!(
        text.contains("zfs-load-key@zfsstep19-home.service"),
        "{text}"
    );
}

/// A gate with no `Wants=` line is a finding about the provider,
/// because nothing then makes the boot mount the checkout first.
#[test]
fn a_gate_with_no_wants_line_is_a_finding_about_the_provider() {
    let facts: BootSetupFacts = without_gate_line(format!("Wants={PROVIDER_UNIT_NAME}").as_str());
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "provider"),
        "{text}"
    );
    assert!(text.contains("carries no `Wants=` line"), "{text}");
}

/// A gate that wants one unit and starts after another is a finding,
/// because the provider it waits for is then not the one it orders after.
#[test]
fn a_gate_ordered_after_another_unit_is_a_finding() {
    let mut facts: BootSetupFacts = healthy();
    let misplaced: String = facts.gate.planned_text.replace(
        format!("After={PROVIDER_UNIT_NAME} zfs-import.target\n").as_str(),
        "After=zfs-import.target\n",
    );
    facts.gate.planned_text = misplaced.clone();
    facts.gate.at_the_path = FileAtInstallPath::Content(misplaced);
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "provider"),
        "{text}"
    );
    assert!(text.contains("does not name that unit"), "{text}");
}

/// A gate with no start bound is a finding about the bounds,
/// because systemd's default then decides how long the boot waits.
#[test]
fn a_gate_with_no_start_bound_is_a_finding_about_the_bounds() {
    let facts: BootSetupFacts =
        without_gate_line(format!("TimeoutStartSec={GATE_START_BOUND}").as_str());
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "bounds"),
        "{text}"
    );
    assert!(
        text.contains("carries no `TimeoutStartSec=` line"),
        "{text}"
    );
}

/// A file nobody installed is a finding that names the path and the repair.
#[test]
fn a_file_nobody_installed_is_a_finding() {
    let mut facts: BootSetupFacts = healthy();
    facts.others[0].at_the_path = FileAtInstallPath::Absent;
    let path: String = facts.others[0].path.display().to_string();
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "installed-file"),
        "{text}"
    );
    assert!(text.contains(path.as_str()), "{text}");
    assert!(text.contains("is not installed"), "{text}");
    assert!(text.contains("which a setup run installs"), "{text}");
}

/// A file an older release or a hand edit wrote names the first line that
/// differs,
/// with what stands there and what this build writes.
#[test]
fn a_file_whose_text_differs_names_the_line() {
    let mut facts: BootSetupFacts = healthy();
    let written: String = facts.gate.planned_text.clone();
    let wanted: &str = "RemainAfterExit=yes";
    let stands_line: &str = "RemainAfterExit=no";
    let line_number: usize = written
        .lines()
        .position(|one| return one == wanted)
        .expect("the gate stays after it exits")
        + 1;
    facts.gate.at_the_path = FileAtInstallPath::Content(written.replace(wanted, stands_line));
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "installed-file"),
        "{text}"
    );
    assert!(
        text.contains(format!("line {line_number}").as_str()),
        "{text}"
    );
    assert!(
        text.contains(
            format!("`{stands_line}` stands where this build writes `{wanted}`").as_str()
        ),
        "{text}"
    );
}

/// A path that is not a regular file is a finding,
/// because systemd reads no text out of it.
#[test]
fn a_path_that_is_not_a_regular_file_is_a_finding() {
    let mut facts: BootSetupFacts = healthy();
    facts.gate.at_the_path = FileAtInstallPath::WrongKind {
        found: String::from("a folder"),
    };
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "installed-file"),
        "{text}"
    );
    assert!(
        text.contains("is a folder rather than the regular file"),
        "{text}"
    );
}

/// A path nobody could read proves nothing,
/// so the check answers unknown with the path and the reason.
#[test]
fn a_file_nobody_could_read_is_unknown() {
    let mut facts: BootSetupFacts = healthy();
    facts.gate.at_the_path = FileAtInstallPath::Unreadable {
        why: String::from("permission denied"),
    };
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(matches!(outcome, CheckOutcome::Unknown { .. }), "{text}");
    assert!(text.contains(BOOT_GATE_UNIT_PATH), "{text}");
    assert!(text.contains("permission denied"), "{text}");
}

/// A dataset left `canmount=on` is a finding whose correction is the command
/// the owner's answer V4 decided.
#[test]
fn a_dataset_left_canmount_mounted_is_a_finding() {
    let mut facts: BootSetupFacts = healthy();
    facts.nested[0].canmount = PropertyValue::Number(CANMOUNT_ON);
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "nested-canmount"),
        "{text}"
    );
    assert!(text.contains("is `canmount=on`"), "{text}");
    assert!(
        text.contains(
            format!("correction: `sudo zfs set canmount=noauto {NESTED_DATASET}`").as_str()
        ),
        "{text}"
    );
}

/// A `canmount` nobody read leaves the answer unknown rather than passed,
/// because a value nobody read could be the one V4 refuses.
#[test]
fn a_canmount_nobody_read_is_unknown() {
    let mut facts: BootSetupFacts = healthy();
    facts.nested[0].canmount = PropertyValue::Absent;
    let outcome: CheckOutcome = check_service_unit(&facts);
    let text: String = line(&outcome);
    assert!(matches!(outcome, CheckOutcome::Unknown { .. }), "{text}");
    assert!(text.contains(NESTED_DATASET), "{text}");
}

/// The first line two texts disagree on is the one a person edits,
/// and nothing when they agree.
#[test]
fn the_first_differing_line_is_the_one_a_person_edits() {
    assert_eq!(first_difference("a\nb\nc\n", "a\nb\nc\n"), None);
    assert_eq!(
        first_difference("a\nb\nc\n", "a\nB\nc\n"),
        Some((2, String::from("b"), String::from("B")))
    );
    assert_eq!(
        first_difference("a\nb\n", "a\nb\nc\n"),
        Some((3, String::new(), String::from("c")))
    );
}

/// One unit line's words are read after its prefix,
/// and a line the text does not hold answers nothing.
#[test]
fn a_lines_words_are_read_after_its_prefix() {
    assert_eq!(
        words_after("[Unit]\nAfter=a.service b.service\n", "After="),
        Some(String::from("a.service b.service"))
    );
    assert_eq!(words_after("[Unit]\nWants=x\n", "After="), None);
}
