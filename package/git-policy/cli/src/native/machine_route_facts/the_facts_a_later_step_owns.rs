//! What:
//!  The four facts of `RouteFacts` that are no machine fact of this route:
//!  the checkout's covered entry,
//!  the boot setup's plan and its installed files,
//!  the cancel command's own grammar,
//!  and Sanoid's coverage comparison.
//! Why:
//!  Plan steps 19 and 20 own these facts,
//!  and their own rounds read them:
//!  the underlay entry is step 19's arm,
//!  the boot units' plan and the datasets under the checkout are its setup run's,
//!  and the coverage comparison needs a `sanoid.conf` and a measured tree
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  steps 19 and 20).
//!  One value of `RouteFacts` needs all four,
//!  so this file answers them the way a report must read a fact nobody
//!  gathered:
//!  as not read,
//!  with the reason naming the step that owns it,
//!  except the cancel command,
//!  whose one fact is the build's own grammar and is read here from the
//!  command's `usage`.
//!  A control that stood in the entry's facts by hand would claim a reading of
//!  a machine nobody made,
//!  which is why the guest arm states the same three as not read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = theFactsALaterStepOwns(); // { bootProtection, bootSetup, cancelCommand, sanoid }
//! ```

/// Where the plan writes its gate unit,
/// the plan this build installs from,
/// and the facts the plan's texts are built out of.
use crate::checkout_boot_units::{
    BOOT_GATE_UNIT_PATH,
    CheckoutBootFacts,
    InstalledUnitFile,
    install_plan,
};
/// The refusal a read of the entry answers.
use crate::checkout_underlay_entry::underlay_entry_read::EntryReadRefusal;
/// What the checkout's covered entry answered,
/// and what the route requires of it.
use crate::doctor_zfs_route::boot_protection_checks::BootProtectionFacts;
/// The boot setup's plan and the files installed from it.
use crate::doctor_zfs_route::boot_setup_facts::{
    BootSetupFacts,
    FileAtInstallPath,
    PlannedFileReading,
};
/// The cancel command's word and its usage line.
use crate::doctor_zfs_route::cancel_command_checks::CancelCommandFacts;
/// The three states a report keeps apart.
use crate::doctor_zfs_route::check_outcome::CheckOutcome;
/// The one file root's modes run as,
/// which the plan's lines name.
use crate::doctor_zfs_route::handler_checks_paths_and_modes::ROOT_MODE_EXECUTABLE_PATH;
/// Sanoid's coverage comparison as the coverage folder answered it.
use crate::doctor_zfs_route::sanoid_report::SanoidReport;
/// The command's own grammar,
/// which is the one fact its line reads.
use crate::worktree_clone_removal_cancel::cancel_arguments::{
    CANCEL_COMMAND_WORD,
    usage,
};
/// An owned path,
/// which these facts name.
use std::path::PathBuf;

/// Why the covered entry is not read here.
const THE_ENTRY_IS_A_LATER_STEPS_FACT: &str = "this gather reads the machine's own route facts, and the checkout's covered entry is plan step 19's to read";

/// Why the boot setup is not read here.
const THE_BOOT_SETUP_IS_A_LATER_STEPS_FACT: &str = "this gather reads the machine's own route facts, and the boot setup's files are plan step 19's to install and read";

/// Why the coverage comparison is not made here.
const THE_COMPARISON_IS_A_LATER_STEPS_FACT: &str = "this gather reads the machine's own route facts, and Sanoid's coverage is plan step 20's to compare";

/// What:
///  The four facts plan steps 19 and 20 own,
///  three of them stated as not read and the cancel command's read from the
///  build's own grammar.
/// Why:
///  One `RouteFacts` value is what the report takes,
///  so the gather answers these four rather than leaving a value its checks
///  cannot read,
///  and the wiring round replaces each with the read its own step builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = theFactsALaterStepOwns({ account }); // { bootProtection, bootSetup, cancelCommand, sanoid }
/// ```
pub(crate) fn the_facts_a_later_step_owns(
    the_account: u32,
    the_checkout: &std::path::Path,
) -> (
    BootProtectionFacts,
    BootSetupFacts,
    CancelCommandFacts,
    SanoidReport,
) {
    let protection: BootProtectionFacts = BootProtectionFacts {
        entry: Err(EntryReadRefusal::Call {
            what: "the entry",
            path: PathBuf::from(BOOT_GATE_UNIT_PATH),
            reason: String::from(THE_ENTRY_IS_A_LATER_STEPS_FACT),
        }),
        wanted_owner: the_account,
    };
    let setup: BootSetupFacts = BootSetupFacts {
        gate: PlannedFileReading {
            path: PathBuf::from(BOOT_GATE_UNIT_PATH),
            planned_text: the_gate_the_build_plans(the_checkout),
            at_the_path: FileAtInstallPath::Unreadable {
                why: String::from(THE_BOOT_SETUP_IS_A_LATER_STEPS_FACT),
            },
        },
        others: Vec::new(),
        key_unit: None,
        nested: Vec::new(),
    };
    let cancel: CancelCommandFacts = CancelCommandFacts {
        word: String::from(CANCEL_COMMAND_WORD),
        usage_line: String::from(usage().lines().next().unwrap_or(CANCEL_COMMAND_WORD)),
    };
    let sanoid: SanoidReport = SanoidReport::new(
        CheckOutcome::Unknown {
            reason: String::from(THE_COMPARISON_IS_A_LATER_STEPS_FACT),
        },
        Vec::new(),
    );
    return (protection, setup, cancel, sanoid);
}

/// What:
///  The gate unit's text as this build's own plan writes it,
///  or nothing when the plan refuses the facts it is given.
/// Why:
///  The boot setup check compares the file at the path with the text the plan
///  installs,
///  so the plan's text is the build's own and the path's read is the machine's;
///  a report that left the plan's text out would answer findings about a gate
///  nobody planned.
///  A plan that refuses the release's own constants is a build defect,
///  which the check reports as a finding,
///  so the refusal is not swallowed into a pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gateText = theGateTheBuildPlans('/home/user/Monochromatic'); // the ExecStart text the plan writes
/// ```
fn the_gate_the_build_plans(the_checkout: &std::path::Path) -> String {
    let facts: CheckoutBootFacts = CheckoutBootFacts {
        checkout: the_checkout.to_path_buf(),
        root_file: PathBuf::from(ROOT_MODE_EXECUTABLE_PATH),
        key_unit: None,
    };
    let plan: Vec<InstalledUnitFile> = match install_plan(&facts, &[]) {
        Ok(files) => files,
        Err(_refusal) => return String::new(),
    };
    return plan
        .iter()
        .find(|file| return file.path.to_str() == Some(BOOT_GATE_UNIT_PATH))
        .map(|file| return file.text.clone())
        .unwrap_or_default();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_facts_a_later_step_owns_tests.rs"]
mod tests;
