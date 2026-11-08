//! What:
//! The numbers of the lean-folder tools:
//! grace periods,
//! idle periods,
//! retention and size caps,
//! each with the measurement or decision it rests on.
//! Why:
//! The owner decided on 2026-11-07 that these are measured and recorded as coordinator picks with the owner's veto open
//! (`doc/handover/questions-for-the-owner.md`);
//! a shipped limit cites its measurement beside the value.
//! The measurements and their dates are in `doc/planning/lean-build-dirs.md`,
//! section "Measured picks,
//! 2026-11-07".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const GRACE_SECONDS = 7 * 86_400;
//! ```

/// Seconds in a day.
pub const DAY_SECONDS: i64 = 86_400;

/// One gibibyte.
pub const GIB: u64 = 1 << 30;

/// The grace period of units,
/// toolchains,
/// images and volumes:
/// seven days.
/// Measured 2026-11-07:
/// the 342 existing worktrees were idle (newest write among their top-level entries) a median of 5.4 days and 6.9 days at the
/// 75th percentile,
/// and dated nightlies were installed at most seven days apart,
/// so a worktree or a second toolchain left alone for a week is still kept.
pub const GRACE_SECONDS: i64 = 7 * DAY_SECONDS;

/// The idle period of a tree whose workspace is gone (or whose toolchain is retired):
/// seven days,
/// the same measurement as the grace period.
pub const TREE_IDLE_SECONDS: i64 = 7 * DAY_SECONDS;

/// The idle period of a tree nothing identifies:
/// twenty-one days.
/// Measured 2026-11-07:
/// worktree idleness was 20.3 days at the 90th percentile,
/// so a workspace dormant for three weeks is beyond nearly every worktree that is still being used.
pub const UNIDENTIFIED_TREE_IDLE_SECONDS: i64 = 21 * DAY_SECONDS;

/// The retention age of verification run records:
/// fourteen days,
/// the age of the accepted logger decision (`doc/decision/module-logger-log-accumulation.md`).
/// Measured 2026-11-07:
/// 372 records in six packages,
/// the newest 6.9 days old and the oldest 34.5 days.
pub const RECORD_RETENTION_SECONDS: i64 = 14 * DAY_SECONDS;

/// The size cap of the run records of one verification directory:
/// two gibibytes.
/// Measured 2026-11-07:
/// the fuzz campaign records average about 95 MiB (2.46 GiB in 26),
/// so the cap holds about twenty of the largest kind.
pub const RECORD_CAP_BYTES: u64 = 2 * GIB;

/// The protection window of a run record:
/// one day,
/// so a run in progress is never removed even over the cap.
pub const RECORD_PROTECT_SECONDS: i64 = DAY_SECONDS;

/// The size cap of one profile directory of an in-tree build (a container build in the checkout's `target/`):
/// twelve gibibytes,
/// one and a half times the largest newest-compiler group measured.
/// Measured 2026-11-07 (units of the newest compiler plus the incremental state written since it first ran):
/// `package/nested-wayland-session/cli/target/debug` 7.35 GiB,
/// `package/ide/app/target/debug` 4.23 GiB,
/// `package/ide/app/target/release` 3.98 GiB;
/// 1.5 times 7.35 GiB is 11.0 GiB,
/// rounded up.
pub const IN_TREE_PROFILE_CAP_BYTES: u64 = 12 * GIB;

/// The size cap of one profile directory of a host build tree:
/// twenty-four gibibytes,
/// one and a half times the largest newest-compiler group measured.
/// Measured 2026-11-07:
/// `0b/ab24900891e355/debug` 15.83 GiB (13.17 GiB of units and 2.66 GiB of incremental state),
/// `75/3315c269a08379/debug` 8.44 GiB,
/// `74/8a90f0615defdd/debug` 4.82 GiB;
/// 1.5 times 15.83 GiB is 23.7 GiB,
/// rounded up.
pub const HOST_PROFILE_CAP_BYTES: u64 = 24 * GIB;

/// The longest a whole `cargo-shadow lean run` may take:
/// three hours.
/// Measured 2026-11-07:
/// listing 4313 images alone took 209 s,
/// and the first run removes terabytes of build trees,
/// so the bound is generous and every step also obeys it.
pub const RUN_DEADLINE_SECONDS: u64 = 3 * 3_600;
