//! What:
//!  The capture-order decision for one shared path of a replay.
//! Why:
//!  When the prepared commit and every landed commit that changed a path captured it from the same worktree,
//!       the later capture's bytes land, which is what sequential native commits record from the shared disk.
//!       Every other path keeps subsumption and then the three-way merge. The service decides it from the capture stamps it
//!       keeps (`service_captures.rs`); this is the answer's shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CaptureOrderDecision = 'prepared' | 'landed' | 'unordered';
//! ```

/// What:
///  The capture-order decision for one shared path.
/// Why:
///   Replay acts on the three answers differently: the prepared entry lands, the landed entry stays,
///       or subsumption and the merge decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CaptureOrderDecision = 'prepared' | 'landed' | 'unordered';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureOrderDecision {
    /// The prepared commit captured later: its entry lands.
    Prepared,
    /// A landed commit captured later: the landed entry stays.
    Landed,
    /// Capture order does not apply: subsumption, then the three-way merge.
    Unordered,
}
