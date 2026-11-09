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
    if stats.left_out > 0 {
        lines.push(format!("{label}: {} entr(ies) absent from the destination were left out because they predate the catch-up", stats.left_out));
    }
    if stats.recopied > 0 {
        lines.push(format!("{label}: {} file(s) changed while they were copied and were copied again", stats.recopied));
    }
    return lines;
}

/// Pass-report controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_pass_report_tests.rs"]
mod tests;
