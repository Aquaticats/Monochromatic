//! What:
//!  One slot as a wrapper command meets it:
//!  the state the slot is in,
//!  the report of the slot,
//!  its dataset and the next safe action,
//!  its link's repair report when the pointer or the registration needs one,
//!  and the paths one `git worktree repair` names.
//! Why:
//!  The design answers a command that meets an unmounted clone or a pending
//!  add with the slot,
//!  its dataset and the one next safe action rather than with a guess between
//!  them,
//!  and a damaged link gets the report of what a repair can and cannot restore
//!  beside it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  The live read fills the sources the classification takes,
//!  so the command path and the controls read the same rules.
//!  Nothing here writes a file:
//!  the restore is the prune guard's arm.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = theSlotAt(commonDir, 'one', slotPath); // { state, report, link }
//! ```

use crate::worktree_clone_binding_record::record_publication::read_binding;
use crate::worktree_clone_binding_record::registration_home::{home_folder, slot_of};
use crate::worktree_clone_link_repair::repair_report::{LinkRepair, LinkRepairReport, class_of, link_report_of};
use crate::worktree_clone_prune_pointer::link_facts::{LinkFacts, link_facts_of, recorded_link_of};
use crate::worktree_clone_slot_state::slot_state_classification::{SlotRouteState, classify_slot};
use crate::worktree_clone_slot_state::slot_state_live_read::{GatedCloneIntentReads, SlotReadingSources, read_slot};
use crate::worktree_clone_slot_state::slot_state_mounts::{CloneMount, MountRows, clone_mount_of, read_mount_rows, slot_mount_of};
use crate::worktree_clone_slot_state::slot_state_readings::SlotReadings;
use crate::worktree_clone_slot_state::slot_state_report::{SlotStateReport, report_of as slot_report_of};
use crate::zfs_compatibility_gate::gate_verdict::LiveGate;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::path::Path;

/// What:
///  One slot as a command meets it:
///  the state,
///  the slot's report,
///  and the link's report when the pointer or the registration is not intact.
/// Why:
///  The state decides what the command does,
///  and the two reports are what the person reads,
///  from one reading so they cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotAtTheCommand = { state, report, link? };
/// ```
#[derive(Clone, Debug)]
pub struct SlotAtTheCommand {
    /// The state the slot is in.
    pub state: SlotRouteState,
    /// The slot,
    /// its dataset and the next safe action.
    pub report: SlotStateReport,
    /// The link's report,
    /// and nothing while both pointer copies are the add's own.
    pub link: Option<LinkRepairReport>,
}

impl SlotAtTheCommand {
    /// The lines a command prints for this slot.
    ///
    /// Why:
    ///  One shape for every command,
    ///  and the link's lines follow the slot's only when there is something
    ///  to say about the link.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lines = slot.lines();
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.report.lines();
        if let Some(link) = &self.link {
            lines.extend(link.lines());
        }
        return lines;
    }
}

/// What:
///  One slot's state and reports,
///  read from the machine.
/// Why:
///  The command path asks one question and prints one answer,
///  and the live sources are the same pair the add route already holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = theSlotAt(commonDir, slot, slotPath);
/// ```
pub fn the_slot_at(common_dir: &Path, slot: &str, slot_path: &Path) -> SlotAtTheCommand {
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    let gate: LiveGate = LiveGate;
    let intents: GatedCloneIntentReads<'_> = GatedCloneIntentReads::new(&gate);
    let sources: SlotReadingSources<'_> = SlotReadingSources { mounts: &mounts, intents: &intents };
    let readings: SlotReadings = read_slot(common_dir, slot, slot_path, &sources);
    let state: SlotRouteState = classify_slot(&readings);
    let report: SlotStateReport = slot_report_of(&readings);
    let link: Option<LinkRepairReport> = the_links_report_of(common_dir, slot, slot_path, &mounts);
    return SlotAtTheCommand { state, report, link };
}

/// What:
///  The link's report for one slot,
///  and nothing when the record names no registration yet or the pointer is
///  the add's own.
/// Why:
///  A slot with no record is not this route's to report on,
///  and an intact link has nothing to repair,
///  so a command does not print a repair section for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const link = theLinksReportOf(commonDir, slot, slotPath, mounts, readings);
/// ```
fn the_links_report_of(common_dir: &Path, slot: &str, slot_path: &Path, mounts: &LiveMountedDatasets) -> Option<LinkRepairReport> {
    let record = read_binding(home_folder(common_dir).as_path(), slot).ok()?;
    let rows: MountRows = read_mount_rows(mounts);
    let mount: CloneMount = clone_mount_of(&rows, &slot_mount_of(mounts, slot_path), record.clone_dataset.as_str(), slot_path);
    let link = recorded_link_of(&record);
    let facts: LinkFacts = link_facts_of(slot, &link, &mount);
    // An intact link is nothing a person must read.
    if class_of(&facts) == LinkRepair::Intact {
        return None;
    }
    return Some(link_report_of(&facts));
}

/// What:
///  The link report of one slot the record home knows,
///  or nothing when no record stands there.
/// Why:
///  A path with no record is not this route's to report on,
///  and a record that cannot be read refuses its own sentence elsewhere,
///  so the caller falls back to Git's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = theLinksReport(commonDir, slotPath, mounts);
/// ```
pub fn the_links_report(common_dir: &Path, slot_path: &Path, mounts: &dyn crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets) -> Option<LinkRepairReport> {
    let slot: String = slot_of(slot_path).ok()?;
    let record = read_binding(home_folder(common_dir).as_path(), slot.as_str()).ok()?;
    let rows: MountRows = read_mount_rows(mounts);
    let mount: CloneMount = clone_mount_of(&rows, &slot_mount_of(mounts, slot_path), record.clone_dataset.as_str(), slot_path);
    let link = recorded_link_of(&record);
    let facts: LinkFacts = link_facts_of(slot.as_str(), &link, &mount);
    return Some(link_report_of(&facts));
}

/// What:
///  The paths one `git worktree repair <path>...` names,
///  with each read the way Git reads an operand.
/// Why:
///  The notice must be about the worktree the person named,
///  and a path that is not a directory is Git's to refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paths = repairPaths(['worktree', 'repair', 'topic'], 0, cwd, commonDir);
/// ```
pub fn repair_paths(arguments: &[std::ffi::OsString], subcommand_index: usize, start: &Path, common_dir: Option<&Path>) -> Vec<std::path::PathBuf> {
    let Some(word) = arguments.get(subcommand_index) else {
        return Vec::new();
    };
    let Some(subcommand) = arguments.get(subcommand_index + 1) else {
        return Vec::new();
    };
    if word.as_encoded_bytes() != b"worktree" || subcommand.as_encoded_bytes() != b"repair" {
        return Vec::new();
    }
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    let mut options_end: bool = false;
    for token in &arguments[subcommand_index + 2..] {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !options_end && bytes == b"--" {
            options_end = true;
            continue;
        }
        if !options_end && bytes.starts_with(b"-") {
            continue;
        }
        if let Some(common) = common_dir
            && let Some(found) = crate::worktree_remove_registry::path_git_picks_for_operand(common, token.as_os_str())
        {
            paths.push(found);
            continue;
        }
        if let Ok(found) = std::fs::canonicalize(start.join(token))
            && found.is_dir()
        {
            paths.push(found);
        }
    }
    return paths;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "live_link_report_tests.rs"]
mod tests;
