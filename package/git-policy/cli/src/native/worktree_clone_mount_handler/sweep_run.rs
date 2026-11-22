//! What:
//!  One whole sweep run:
//!  every immediate child of the pinned parent,
//!  one turn of `sweep_one_clone` for each,
//!  and the count and the lines a mode reports.
//! Why:
//!  The design's sweep enumerates the pinned parent's immediate children and
//!  reconciles each clone's durable intent
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!  A clone that refused does not stop the run,
//!  because one malformed clone must not leave every later clone unmounted,
//!  while a listing that stopped does end it:
//!  the clones already attached keep their mounts,
//!  and the run says where it stopped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const run = sweep(context); // { children, reached, stopped }
//! ```

use super::sweep_dataset_reader::{Listing, each_child};
use super::sweep_one_clone::{CloneOutcome, SweepContext, sweep_one_clone};

/// What:
///  What one run reached and what it answered.
/// Why:
///  A mode reports one line and a caller reads every clone's own line,
///  and a run that stopped mid-listing must say so rather than look complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepRun = { parentDataset; children; reached; stopped };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SweepRun {
    /// The parent the run enumerated.
    pub parent_dataset: String,
    /// How many children the listing gave.
    pub children: usize,
    /// What each child's turn answered,
    /// in the kernel's order.
    pub reached: Vec<CloneOutcome>,
    /// Why the listing stopped,
    /// or nothing when it reached the end.
    pub stopped: Option<String>,
}

impl SweepRun {
    /// How many clones the run attached.
    pub fn attached(&self) -> usize {
        return self.reached.iter().filter(|outcome| return matches!(outcome, CloneOutcome::Attached { .. })).count();
    }

    /// How many clones asked for nothing.
    pub fn nothing_asked(&self) -> usize {
        return self.reached.iter().filter(|outcome| return matches!(outcome, CloneOutcome::NothingAsked { .. })).count();
    }

    /// How many clones the run refused.
    pub fn refused(&self) -> usize {
        return self.reached.iter().filter(|outcome| return matches!(outcome, CloneOutcome::Refused { .. })).count();
    }

    /// How many clones failed at a step.
    pub fn failed(&self) -> usize {
        return self.reached.iter().filter(|outcome| return matches!(outcome, CloneOutcome::Failed { .. })).count();
    }

    /// How many slots another sweep held.
    pub fn held(&self) -> usize {
        return self.reached.iter().filter(|outcome| return matches!(outcome, CloneOutcome::Held { .. })).count();
    }

    /// Whether the listing reached the end of the parent's children.
    pub fn completed(&self) -> bool {
        return self.stopped.is_none();
    }

    /// The one line a mode says about the run.
    pub fn summary(&self) -> String {
        let closing: String = match &self.stopped {
            Some(reason) => format!("The listing stopped: {reason}"),
            None => String::from("The listing reached the end of the parent's children."),
        };
        return format!(
            "the clone-mount sweep reached {} of the {} children of `{}`: {} attached, {} asked for nothing, {} refused, {} failed and {} held for another sweep. {closing}",
            self.reached.len(),
            self.children,
            self.parent_dataset,
            self.attached(),
            self.nothing_asked(),
            self.refused(),
            self.failed(),
            self.held()
        );
    }

    /// The summary and every clone's own line,
    /// which is what a guest arm and an operator read.
    pub fn report(&self) -> String {
        let mut lines: Vec<String> = vec![self.summary()];
        for outcome in &self.reached {
            lines.push(outcome.line());
        }
        return lines.join("\n");
    }
}

/// What:
///  Enumerate the pinned parent's children and take one turn of each.
/// Why:
///  Every clone's turn is one call,
///  so the run's order and its counts and its stop are the only things this
///  function decides,
///  and a control drives the whole run with a scripted pool and attach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = sweep(context);
/// ```
pub fn sweep(context: &SweepContext<'_>) -> SweepRun {
    let mut reached: Vec<CloneOutcome> = Vec::new();
    // Each clone takes its turn as the listing gives it, so a listing that stops later costs the clones already reached
    // nothing: their mounts stand and the run says where it stopped.
    let listing: Listing = each_child(context.reader, context.parent_dataset, &mut |name| return reached.push(sweep_one_clone(context, name)));
    return SweepRun {
        parent_dataset: String::from(context.parent_dataset),
        children: listing.seen,
        reached,
        stopped: listing.refusal.map(|refusal| return refusal.message()),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_run_tests.rs"]
mod tests;
