//! What:
//!  The merge device of plan step 19:
//!  the one root-owned folder under `/run`,
//!  the route-wide run claim of one event-driven pass,
//!  the mark a trigger writes when it asked for a pass,
//!  the one extra pass a mark buys,
//!  and every line the device answers with.
//! Why:
//!  Every event-driven trigger would otherwise start its own pass over every
//!  managed clone,
//!  and two triggers about one clone need one pass that reads the clone after
//!  both writes
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "Merging the nudges pending for one clone";
//!  veto entries V4,
//!  V6,
//!  V7 and V12).
//!  The device decides how many passes run and who runs them;
//!  it never decides what a pass reads or writes,
//!  so its whole dependency on the sweep is the closure a caller hands it.
//!  The pass budget and its number are plan step 22's:
//!  the design's own choice 17 is "The merge device is built in step 19,
//!  the budget's number set in step 22"
//!  (`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
//!  "The seventeen choices as the project was shown them",
//!  and "What this record does not decide").
//!  Nothing here is reached by a command yet:
//!  step 19's wiring calls the taker at the entry of the default sweep,
//!  after the filter and the gate and after the run's own prerequisite checks,
//!  and that wiring is step 19's own round
//!  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`,
//!  "A piece built ahead merges as code no person's command reaches").
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `sweeps_folder.rs`:
//!   the one root-owned folder the three records live in,
//!   made when a fresh boot has none and proved through its own descriptor.
//! - `pass_mark.rs`:
//!   the mark one trigger writes for a pass no pass has covered,
//!   whose presence is its whole meaning.
//! - `run_claim.rs`:
//!   the route-wide run claim of one event-driven pass,
//!   taken without waiting.
//! - `merge_lines.rs`:
//!   every line the device answers with,
//!   each naming what it did and who covers a request it left standing.
//! - `merge_pass.rs`:
//!   the taker:
//!   one pass under the claim,
//!   one more for a mark,
//!   and the second claim attempt.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = runAMergePass(SweepsFolder.live(), () => sweep());
//! ```

/// The one root-owned folder the device's records live in.
pub mod sweeps_folder;

/// The mark a trigger writes for a pass no pass has covered.
pub mod pass_mark;

/// The route-wide run claim of one event-driven pass.
pub mod run_claim;

/// Every line the device answers with.
pub mod merge_lines;

/// The taker:
/// one pass,
/// one more for a mark,
/// and the second claim attempt.
pub mod merge_pass;

/// The taker and the most passes one invocation runs.
pub use merge_pass::{
    MOST_PASSES,
    run_a_merge_pass,
};
/// The look for the mark.
pub use pass_mark::{
    MarkReading,
    the_mark_is_there,
};
/// The claim one event-driven pass holds.
pub use run_claim::{
    ClaimAnswer,
    HeldRunClaim,
    take_the_run_claim,
};
/// The folder the device's records live in,
/// and the mark's name and the one line it holds.
pub use sweeps_folder::{
    PASS_MARK_MODE,
    PASS_MARK_NAME,
    PASS_MARK_TEXT,
    RUN_CLAIM_PREFIX,
    SWEEPS_FOLDER,
    SWEEPS_FOLDER_MODE,
    SweepsFolder,
};
