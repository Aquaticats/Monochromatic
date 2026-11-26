//! What:
//!  What one request's timing is made of:
//!  when its connection was accepted and how long its request line took to arrive,
//!  the phases of its landing,
//!  how the reads a landing spent time on split,
//!  and everything the line for one answered request states.
//! Why:
//!   The figures are recorded by the landing and read by the line the service writes,
//!   so one file holds the shape both sides agree on.
//!  Every phase is a `Duration` of this process's own clock rather than a wall time,
//!  because a journal line states its own time separately (`service_timing_line_text.rs`)
//!  and a landing's phases must add up to what it measured.
//!  The line that writes these figures is `service_timing_answered_line.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LandTiming = { lockWait: number; objects: number; journal: number; refUpdate: number; indexUpdate: number; attempts: number; readsSplit: ReadSplit };
//! ```
use crate::service_durable_syncs::SyncFigure;
use crate::service_index_found::WaitNotes;
use crate::transaction_step_durations::StepFigure;
use std::path::Path;
use std::time::{
    Duration,
    Instant,
};

/// When a connection was accepted and how long its request line took to arrive.
#[derive(Clone, Copy, Debug)]
pub struct Arrival {
    /// The moment the connection was accepted.
    pub accepted: Instant,
    /// Time from the acceptance to the whole request line being read;
    /// a client that is slow to send shows up here and not as a
    ///  wait in the queue.
    pub read: Duration,
}

/// What:
///  The phases of one landing the service itself spent time in,
/// summed over the attempts a moving branch made it repeat.
/// Why:
///   The wait for another program's hold on the index,
/// the objects,
/// the journal write,
/// the branch move and the index update are
///  the stretches a slow landing can lose time in;
/// whatever else a landing did is reported as the reads that remain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandTiming = { lockWait: number; lockHolder?: string; objects: number; journal: number; refUpdate: number; indexUpdate: number; attempts: number };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LandTiming {
    /// Time to take the index lock,
    /// which is the whole wait for a foreign holder.
    pub lock_wait: Duration,
    /// The holder the evidence named while the lock was waited for,
    /// when it named one.
    pub lock_holder: Option<String>,
    /// What the acquisition noted about its wait:
    /// what the first and the last look at the held lock found,
    /// how many looks there were,
    /// and how the time split between flushing the marker and waiting for the holder.
    pub lock_notes: WaitNotes,
    /// Time to make the commits:
    /// one import for the batch,
    /// or the checks of a commit Git built.
    pub objects: Duration,
    /// Time to write the intent to the journal.
    pub journal: Duration,
    /// Time of the compare-and-swap that moves the branch.
    pub ref_update: Duration,
    /// Time to remove the conclusion state and to build and install the new index.
    pub index_update: Duration,
    /// How long the index lock was held,
    /// from its placement to its release:
    /// the branch move and the index build.
    /// Not a phase of its own (it overlaps `ref_update` and `index_update`),
    /// so it is not part of what `reads` subtracts.
    pub hold: Duration,
    /// Passes over the batch:
    /// one,
    /// and one more each time the branch moved under it.
    pub attempts: usize,
    /// Which of the residual reads spent the time,
    /// named one by one;
    /// their sum is never more than `reads`.
    pub reads_split: ReadSplit,
    /// Every sync this landing made or handed to the background,
    /// in the order it asked for them.
    pub syncs: Vec<SyncFigure>,
}

impl LandTiming {
    /// What `landing` spent outside the named phases:
    /// the branch reads,
    /// the comparisons with the branch,
    /// the check for removed
    ///  content,
    /// and the rest of what `reads_split` names one by one.
    pub fn reads(
        &self,
        landing: Duration,
    ) -> Duration {
        return landing.saturating_sub(
            self.lock_wait + self.objects + self.journal + self.ref_update + self.index_update,
        );
    }
}

/// What:
///  How the residual reads of a landing split,
///  one named stretch each.
/// Why:
///   `reads` was one number for the branch reads,
///  the comparisons with the branch,
///  the check for removed content,
///  the index witness,
///  the conclusion copy,
///  the records a batch owes and the head question under the lock;
///  a landing that spent a second there
///  could not say which one did.
///  Every named stretch is a part of what `reads` subtracts,
///  so the sum of them never exceeds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadSplit = { config: number; branch: number; parents: number; changed: number; content: number; witness: number; conclusion: number; owed: number; follow: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReadSplit {
    /// Reading the configuration snapshot and deciding whether the history is plain,
    /// once for the batch,
    /// before it starts.
    pub config: Duration,
    /// Reading the branch's commit and asking whether the worktree still has it checked out,
    /// before the index lock is asked for.
    pub branch: Duration,
    /// Listing the parents of the commit an amend replaces.
    pub parents: Duration,
    /// Comparing the requests with the branch since their base,
    /// one listing per distinct base.
    pub changed: Duration,
    /// Looking for the content of the batch in the object store before anything is imported.
    pub content: Duration,
    /// Recording the index file the landing saw.
    pub witness: Duration,
    /// Keeping the copy of the conclusion state a cleanup removes.
    pub conclusion: Duration,
    /// Taking the index update and the held push a batch owes before its branch moves.
    pub owed: Duration,
    /// Reading the worktree's head again after the branch moved.
    pub follow: Duration,
}

impl ReadSplit {
    /// Every named read added together.
    pub fn total(&self) -> Duration {
        return self.config
            + self.branch
            + self.parents
            + self.changed
            + self.content
            + self.witness
            + self.conclusion
            + self.owed
            + self.follow;
    }
}

/// Everything the line for one answered request states.
pub struct Answered<'record> {
    /// The request's identity.
    pub request: &'record str,
    /// The reference the request names.
    pub reference: &'record str,
    /// What became of it:
    /// a word such as `landed`,
    /// `already`,
    /// `retry-prepare`,
    /// `branch-moved` or `failed:<code>`.
    pub outcome: &'record str,
    /// The batch number,
    /// shared by every request landed together.
    pub batch: u64,
    /// The worktree root the batch landed in,
    /// which the line names:
    /// one journal covers every worktree of the machine.
    pub repository: &'record Path,
    /// How many requests the batch of this branch held.
    pub size: usize,
    /// This request's place among them,
    /// from one.
    pub position: usize,
    /// How many other commits were still waiting for their answer when this landing finished,
    /// which is how many commits queued behind it.
    pub queued_behind: usize,
    /// How the connection and its request line arrived.
    pub arrival: Arrival,
    /// Time from the request being read to its batch starting to land.
    pub queued: Duration,
    /// Time to settle the batches a killed service left open,
    /// which runs before each batch.
    pub settle: Duration,
    /// The phases of the landing.
    pub landing: &'record LandTiming,
    /// Time the landing took,
    /// from its start to its return.
    pub land_total: Duration,
    /// Time from the landing's end to this request being answered.
    pub answer: Duration,
    /// Time from the acceptance of the connection to the answer.
    pub total: Duration,
    /// The longest quiet gap of every Git step and tool the service ran since the previous answered line,
    /// by step name,
    /// taken
    ///  from the ledger (`quiet_ledger.rs`).
    /// The landings of one service share the ledger,
    /// so the first answer of a batch holds
    ///  what the batch's steps showed and the others of that batch hold nothing.
    pub quiet: &'record [(String, Duration)],
    /// Every Git step the service started since the previous answered line,
    /// as program and subcommand with the offset from the
    ///  connection's acceptance and the duration (`transaction_step_durations.rs`).
    /// Taken like `quiet`:
    /// the first answer of a batch
    ///  holds the batch's steps.
    pub steps: &'record [StepFigure],
    /// How many steps came after the list reached its bound.
    pub steps_dropped: usize,
}
