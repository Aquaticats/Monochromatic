//! What:
//!  The shipped limits a judgement rests on,
//!  and the measurements each was chosen from:
//!  the longest step of a landing that updates a reference,
//!  the multiple of it a lock must outlive before it is judged,
//!  and the file system types a lock may sit on.
//! Why:
//!  A number a removal acts on is only as good as the measurement under it,
//!  so the measurement travels with the number
//!  (`doc/planning/cli-git-shipped-limits-audit.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const LONGEST_LIVE_REF_STEP = 3438; const MARGIN_MULTIPLE = 2; const MARGIN = Math.ceil((LONGEST_LIVE_REF_STEP * MARGIN_MULTIPLE) / 1000);
//! ```

/// `Duration` is a span of wall-clock time,
/// and not a bare number of milliseconds.
use std::time::Duration;

/// The longest step of a landing that updates a reference,
/// in a live service:
/// 3,438 ms,
/// the largest `ref_ms` of 1,302 `answered`
///  entries of the service journal (`journalctl --user SYSLOG_IDENTIFIER=cli-git-service`,
/// 2026-11-02T09:01:38Z to
///  2026-11-03T15:11:58Z;
/// 1 entry above 2,000 ms and 2 at or above 1,000 ms;
/// median 2 ms,
/// 99th percentile 64 ms).
/// The step starts
///  Git,
/// takes the reference lock,
/// fills it and renames it,
/// so a lock a live Git holds is gone within it.
/// Real Git on a
///  throwaway repository,
/// traced with `strace`,
/// held a lock for at most 2.8 ms (7,700 updates,
/// load average 3 to 14),
/// so this
///  is the loaded tail,
/// not the common case.
pub const LONGEST_LIVE_REF_STEP: Duration = Duration::from_millis(3438);

/// How many times the longest live reference step the margin allows:
/// twice,
/// because the longest of a sample understates the
///  tail (one entry of 1,302 reached 3,438 ms,
/// so the next sample may reach more).
/// Option B of the question on the margin,
/// "twice
///  the sample maximum,
/// seven seconds",
/// picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups").
pub const MARGIN_MULTIPLE: u64 = 2;

/// How old a lock must be before it is judged:
/// long enough for a live Git to have filled and renamed a lock it just made.
/// The
///  whole seconds at or above `MARGIN_MULTIPLE` times the longest step measured in `LONGEST_LIVE_REF_STEP` (6.876 s,
/// so 7 s),
///  because a lock younger than the longest live hold may still be live.
/// A stale lock is left that long before it is cleared,
///  which is a few seconds of a failing update,
/// against a live lock removed (`doc/planning/cli-git-shipped-limits-audit.md`,
///  "Margins for removing left-behind locks").
pub const MARGIN: Duration = Duration::from_secs(
    (LONGEST_LIVE_REF_STEP.as_millis() as u64 * MARGIN_MULTIPLE).div_ceil(1000),
);

/// The file system types a lock may sit on:
/// disks and memory of this machine.
/// A network or user-space file system can be
///  written by a Git on another machine,
/// whose process numbers mean nothing here.
pub(crate) const LOCAL_FILE_SYSTEMS: &[&str] = &[
    "btrfs", "ext4", "xfs", "f2fs", "tmpfs", "overlay", "zfs", "bcachefs",
];
