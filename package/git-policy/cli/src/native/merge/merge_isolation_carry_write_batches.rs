//! What:
//!  Which step of a carry a batch of writes is,
//!  and the test markers each of its steps reaches.
//! Why:
//!   The clear before a merge,
//!  the carry of each layer after it and a putting back differ in what a path that moved after the judgement does,
//!  and a control needs the instant between the index records and the files;
//!  one type names the three steps,
//!  and its markers are the only place that instant exists.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const batch = Batch::Clear; // its marks, and whether a moved path stops the whole batch
//! ```

/// The test-only markers the transaction phases kill or pause the process at.
use crate::transaction_phase::Phase;

/// What:
///  The test-only markers one write step reaches.
/// Why:
///   A step writes its index records in one call and its worktree files one at a time after it,
/// so a control needs the instant
///  between those parts and between two files;
///  `transaction_phase` kills or pauses the process there.
pub(crate) struct StepMarks {
    /// Reached once every index record of this step is written.
    pub(crate) index: Phase,
    /// Reached after each worktree file of this step,
    /// numbered by occurrence.
    pub(crate) file: Phase,
}

/// The markers of the clear's writes.
pub(crate) const CLEAR_MARKS: StepMarks = StepMarks {
    index: Phase::MergeClearIndex,
    file: Phase::MergeClearFile,
};

/// The markers of the carry back's writes.
pub(crate) const CARRY_MARKS: StepMarks = StepMarks {
    index: Phase::MergeCarryIndex,
    file: Phase::MergeCarryFile,
};

/// What:
///  Which step of a carry these writes are,
///  and what that step does when a path moved after it was judged.
/// Why:
///   The clear hands the paths to Git's own two-way update,
/// so a path that moved after the judgement must stop it with
///  nothing written and the round made again;
/// the carry back and a putting back write each path on its own,
///  so one that moved keeps its bytes and the others still go.
///  The test-only markers follow the same three steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Batch = 'clear' | 'carry' | 'restore';
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Batch {
    /// The clear before the branch moves.
    Clear,
    /// The merge of each layer with the merge.
    Carry,
    /// The putting back of a merge that did not land.
    Restore,
}

impl Batch {
    /// The markers this batch reaches,
    /// when a control watches them.
    pub(crate) fn marks(self) -> Option<&'static StepMarks> {
        return match self {
            Batch::Clear => Some(&CLEAR_MARKS),
            Batch::Carry => Some(&CARRY_MARKS),
            Batch::Restore => None,
        };
    }

    /// Whether a path that moved after the judgement stops the whole batch,
    /// rather than keeping its own bytes while the rest go on.
    pub(crate) fn stops_when_a_path_moved(self) -> bool {
        return self == Batch::Clear;
    }
}
