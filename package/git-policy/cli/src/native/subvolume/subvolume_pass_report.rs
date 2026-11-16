//! What:
//!  The lines a tree pass says about what it could not carry into the destination,
//!  and about the entries it left
//!  aside:
//! the sockets and device nodes it did not recreate,
//! the owner,
//! permission bits,
//! times and
//!  extended attributes the kernel refused,
//! and the destination entries it kept or left out.
//! Two of those kinds are this program's own bookkeeping,
//!  and each is named with the reason the pass kept or carried it:
//! the records of the processes it started,
//!  left out,
//! and the notices it wrote for the person,
//! carried over despite the pass's cut-off.
//! Why:
//!  The migration and the conversion run the same pass over the same statistics,
//! and a person reading either
//!  command's output must read the same words for the same fact;
//! one function that both call is what keeps them
//!  from drifting apart.
//! Nothing here is silent:
//! a pass that could not carry something says so once per kind,
//!  and names the entry for each kind that has entries to name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! sayWhatAPassCouldNotCarry('copy', stats, say);
//! ```

use crate::subvolume_sync::SyncStats;
use std::path::PathBuf;

/// What:
///  One line for each kind of entry a pass could not carry,
///  or set,
///  or had to keep or leave out,
/// prefixed
///  with `label`.
/// Why:
///   See the module description.
/// The counts of a pass are the caller's line,
/// because the migration and the
///  conversion count different things;
/// what a pass does not carry is the same for both.
/// The lines are returned rather than said,
/// so a
///  caller says them through its own sink without a closure between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatAPassCouldNotCarry(label, stats): string[];
/// ```
pub fn what_a_pass_could_not_carry(label: &str, stats: &SyncStats) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for path in &stats.special_skipped {
        lines.push(format!("{label}: not recreated (a socket or device node): {}", path.display()));
    }
    if stats.losses.owner > 0 || stats.losses.extended_attributes > 0 || stats.losses.mode > 0 || stats.losses.times > 0 {
        lines.push(format!(
            "{label}: could not set the owner of {} entries, the permission bits of {}, the times of {}, and {} extended attributes (not running as the owner or as root)",
            stats.losses.owner, stats.losses.mode, stats.losses.times, stats.losses.extended_attributes
        ));
    }
    for path in &stats.conflicts {
        lines.push(format!("{label}: kept the newer or differently typed entry {} in the destination; it was not replaced", path.display()));
    }
    if !stats.left_out.is_empty() {
        lines.push(format!("{label}: {} entr(ies) absent from the destination were left out because they predate the catch-up", stats.left_out.len()));
    }
    if !stats.holder_records_left_out.is_empty() {
        lines.push(format!(
            "{label}: {} record(s) of this program's own lock holders stayed in the source, because the destination lacks them and they are older than the cut-off: {}",
            stats.holder_records_left_out.len(),
            joined(&stats.holder_records_left_out)
        ));
    }
    for path in &stats.kept_at_the_destination {
        lines.push(format!("{label}: did not place {} because the destination holds an entry there that this pass did not put there, and it stays", path.display()));
    }
    for (parked, original) in &stats.parked_copies {
        lines.push(format!(
            "{label}: kept {parked}, a copy of {original} that this command parked when a replacement was interrupted, and it did not judge it; nothing here deletes it: look at it, and delete it when nothing in it is yours",
            parked = parked.display(),
            original = original.display()
        ));
    }
    for path in &stats.changed_while_passing {
        lines.push(format!(
            "{label}: did not carry {}: it changed while the copy ran, so the destination does not hold what the source holds there",
            path.display()
        ));
    }
    if !stats.notices_carried.is_empty() {
        lines.push(format!(
            "{label}: carried {} of this program's own notice(s) although the destination lacks them and they are older than the cut-off: {}",
            stats.notices_carried.len(),
            joined(&stats.notices_carried)
        ));
    }
    if stats.recopied > 0 {
        lines.push(format!("{label}: {} file(s) changed while they were copied and were copied again", stats.recopied));
    }
    return lines;
}

/// What:
///  The paths of one kind, comma-separated for one line of a report.
/// Why:
///   Every kind that names its entries spells them the same way,
/// so a person reads one shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = paths.join(', ');
/// ```
fn joined(paths: &[PathBuf]) -> String {
    let mut shown: Vec<String> = Vec::new();
    for path in paths {
        shown.push(path.display().to_string());
    }
    return shown.join(", ");
}

/// Pass-report controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_pass_report_tests.rs"]
mod tests;
