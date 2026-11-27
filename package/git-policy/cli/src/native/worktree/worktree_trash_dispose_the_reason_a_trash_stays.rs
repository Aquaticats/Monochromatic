//! What:
//!  The reason a trash the emptying leaves gives for every step it would not take:
//!  the answer every step of the emptying, and the service's own line, is worded from.
//! Why:
//!  A directory under the recorded name that the snapshot did not make,
//!  an entry that is no longer the object the leave-out moved there,
//!  and a file somebody put in the trash all stay,
//!  so the reason names what is there and the service reports what it did not delete.
//!  A step that could not finish leaves what it did not examine where it is,
//!  and the reason says so.
//!  Every reason is the same shape,
//!  so the service's line and the records cannot disagree about one.
//!  `worktree_trash_dispose.rs` re-exports the type, so every caller names it there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type TrashLeft = { reason: string };
//! const kept = (reason: string): TrashLeft => ({ reason });
//! ```

/// What:
///  A trash the emptying left where it is,
/// with the reason it gives for every step it would not take.
/// Why:
///   A directory under the recorded name that the snapshot did not make,
/// an entry that is no longer the object
///  the leave-out moved there,
/// and a file somebody put in the trash all stay,
/// so the reason names what is there and the
///  service reports what it did not delete.
/// A step that could not finish leaves what it did not examine where it is,
///  and the reason says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrashLeft = { reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrashLeft {
    /// Why the trash stays,
    /// naming what is in it.
    pub reason: String,
}

impl TrashLeft {
    /// What:
    ///  A refusal with the reason it gives.
    /// Why:
    ///   Every reason the emptying answers with is the same shape,
    /// so the service's line and the records cannot disagree
    ///  about one.
    /// It is visible to this folder because the steps that word a reason
    ///  are two files, `worktree_trash_dispose_after_the_entries.rs`
    /// and `worktree_trash_dispose.rs`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kept = (reason) => ({ reason });
    /// ```
    pub(crate) fn kept(reason: String) -> TrashLeft {
        return TrashLeft {
            reason,
        };
    }
}
