//! What:
//!  The bounded loop that repeats a whole policy pass after corrections changed the
//!       candidate content,
//!  until the content is stable or the loop gives up.
//! Why:
//!  A correction by one policy can create a finding for another,
//!  so every changed
//!      state restarts the complete order.
//!  The loop must end:
//!  at most eight passes may
//!      change the content,
//!  and returning to an earlier state is a cycle.
//!  This module owns
//!      only that control flow;
//!  running policies,
//!  applying corrections and storing exact
//!      states stay behind a small interface,
//!  so the loop is testable without Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const convergence = await convergeCommitPolicies({ firstPass, firstSnapshot, ... });
//! ```

/// What:
///  The most passes that may change candidate content.
///  `usize` is the unsigned
///       integer of list positions and counts (siblings `u32`,
///  `u64`).
/// Why:
///   The incumbent stops after eight changed passes;
///  `usize` is the type the snapshot
///       numbers below are counted in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAXIMUM_CHANGED_PASSES = 8;
/// ```
pub const MAXIMUM_CHANGED_PASSES: usize = 8;

/// The engine-failure message of a pass that still proposes corrections after the limit.
pub const FIX_PASS_LIMIT_MESSAGE: &str =
    "Policy patches did not converge within eight changed passes.";

/// The engine-failure message of corrections that returned to an earlier state.
pub const FIX_CYCLE_MESSAGE: &str = "Policy patches repeated an exact prior candidate state.";

/// What:
///  What one whole policy pass over the current state ended in.
///  An `enum` is a closed
///       set of named alternatives.
///  `#[derive(...)]` asks the compiler to generate copying,
///       debug printing and `==`.
/// Why:
///   Only a pass that proposes corrections continues the loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PassResult = 'stable' | 'proposed' | 'failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassResult {
    /// No policy proposed a correction:
    ///  the findings of this pass are final.
    Stable,
    /// At least one policy proposed a correction.
    Proposed,
    /// The pass ended in an engine failure.
    Failed,
}

/// What:
///  The work the loop drives.
///  A `trait` is a named set of methods a type promises to
///       provide,
///  like a TS `interface`;
///  `&mut self` lends the implementer for writing.
/// Why:
///   Exact candidate states can be large,
///  so the implementer stores them (the
///       incumbent used private files) and answers equality by snapshot number;
///  the loop
///       never holds content.
/// Gotcha:
///  Snapshot 0 is the state before the first pass and must exist before the loop starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface FixPasses { runPass(version: number): PassResult; apply(snapshot: number): boolean; snapshotsEqual(a: number, b: number): boolean }
/// ```
pub trait FixPasses {
    /// Run every policy against the current state;
    ///  `candidate_version` counts earlier changes.
    fn run_pass(&mut self, candidate_version: usize) -> PassResult;
    /// Apply the last pass's corrections and store the new exact state under `snapshot`.
    /// Returns false when the corrections could not be applied.
    fn apply(&mut self, snapshot: usize) -> bool;
    /// Whether two stored states are equal byte for byte.
    fn snapshots_equal(&mut self, left: usize, right: usize) -> bool;
}

/// What:
///  How the loop ended.
///  `Settled` carries the number of passes that changed content.
/// Why:
///   The caller reports the stable pass,
///  or one engine failure for each other ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Convergence = { kind: 'settled'; changedPasses: number } | { kind: 'blocked' | 'fix-cycle' | 'fix-pass-limit' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Convergence {
    /// The last pass proposed nothing,
    ///  or its corrections left the state exactly as it was.
    Settled {
        /// Passes whose corrections changed the state.
        changed_passes: usize,
    },
    /// A pass failed,
    ///  or its corrections could not be applied.
    Blocked,
    /// Corrections reproduced a state from before the preceding one.
    FixCycle,
    /// A pass still proposed corrections after eight changed passes.
    FixPassLimit,
}

/// What:
///  The ending forced by the state just stored under `snapshot`,
///  or nothing when it
///       differs from every earlier state.
///  `&mut dyn FixPasses` lends "any implementer" for
///       writing (`dyn` means the concrete type is chosen at run time,
///  like a TS interface
///       value);
///  `Option<Convergence>` is "an ending or nothing".
/// Why:
///   Which earlier state the new one equals decides the ending:
///  the preceding state
///       means "nothing changed",
///  an older one means the corrections are going in circles.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const earlier = visited.findIndex(state => equal(state, current));
/// if (earlier === visited.length - 1) return settled; if (earlier !== -1) return cycle;
/// ```
fn repeated_state_ending(passes: &mut dyn FixPasses, snapshot: usize) -> Option<Convergence> {
    // `0..snapshot` counts every state stored before this one.
    for earlier in 0..snapshot {
        if passes.snapshots_equal(earlier, snapshot) {
            // The preceding state's number is also the number of changes made before it.
            if earlier + 1 == snapshot {
                // `Some(x)` is the "present" case of `Option`.
                return Some(Convergence::Settled {
                    changed_passes: earlier,
                });
            }
            return Some(Convergence::FixCycle);
        }
    }
    // `None` is the "absent" case of `Option`.
    return None;
}

/// What:
///  Drive passes until the state is stable or the loop must give up.
/// Why:
///   The bound is structural:
///  the `for` loop runs at most eight times,
///  each round
///       ending or storing exactly one new state,
///  and one last pass may only confirm
///       stability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function converge(passes: FixPasses): Convergence;
/// ```
pub fn converge(passes: &mut dyn FixPasses) -> Convergence {
    // `changed_passes` counts the rounds that stored a new state so far.
    for changed_passes in 0..MAXIMUM_CHANGED_PASSES {
        // `match` picks one arm per variant; `{}` does nothing and goes on.
        match passes.run_pass(changed_passes) {
            PassResult::Stable => return Convergence::Settled { changed_passes },
            PassResult::Failed => return Convergence::Blocked,
            PassResult::Proposed => {}
        }
        let snapshot: usize = changed_passes + 1;
        if !passes.apply(snapshot) {
            return Convergence::Blocked;
        }
        // `if let Some(ending) = ...` runs only when the new state equals a stored one.
        if let Some(ending) = repeated_state_ending(passes, snapshot) {
            return ending;
        }
    }
    match passes.run_pass(MAXIMUM_CHANGED_PASSES) {
        PassResult::Stable => {
            return Convergence::Settled {
                changed_passes: MAXIMUM_CHANGED_PASSES,
            };
        }
        PassResult::Failed => return Convergence::Blocked,
        PassResult::Proposed => return Convergence::FixPassLimit,
    }
}

/// Stability,
///  limit and cycle controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_convergence_tests.rs"]
mod tests;
