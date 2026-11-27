//! What:
//!  The bytes one repository's drain holds at its peak,
//!  and the measurement those bytes are rounded up from.
//! Why:
//!  Two files read the figure and neither owns the other:
//! the round that builds a plan with it,
//!  and the file that runs a batch of a round side by side.
//!  Held at the bottom of the folder,
//! it is what both name instead of naming each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const perItemMemory = repositoryMemory; // measuredRepositoryPeak rounded up to a power of two
//! ```

/// The 90th percentile of `MemoryPeak` over the 28 commit-service units of this host that `systemctl --user show 'cli-git@*.service'
///  --property=MemoryPeak` reported on 2026-11-03 (the units' peaks since each one's last start;
/// the Git processes a service
///  starts are inside the unit):
/// 130,482,176 bytes.
/// The median was 33,000,000 bytes,
/// the 75th percentile 49,000,000 bytes,
/// and the
///  largest 1,073,766,400 bytes,
/// a unit that had run under the one-gibibyte cap this build no longer sets.
/// A unit serves one
///  worktree for its whole life,
/// so a unit's peak is an upper bound of what one repository's drain holds at once.
pub const MEASURED_REPOSITORY_PEAK: u64 = 130_482_176;

/// The bytes the drain of one repository holds at its peak:
/// `MEASURED_REPOSITORY_PEAK` rounded up to a power of two,
/// as
///  `CONVERSION_MEMORY` rounds its measurement.
/// The coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Parallelism caps and memory estimates").
pub const REPOSITORY_MEMORY: u64 = MEASURED_REPOSITORY_PEAK.next_power_of_two();
