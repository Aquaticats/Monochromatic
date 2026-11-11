//! What:
//!  The kinds of work a landed object can ask for,
//! and the names that pick a consumer's handler.
//! Why:
//!  One record format carries every asynchronous job (`work_item.rs`),
//! and the kind is the first part of an item's id and
//!  the first part of a claim's file name,
//! so the names are written here once and read by the queue,
//! the claims and the service's
//!  handlers.
//! The type sits in a file of its own so the record's file stays short enough to read whole.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type WorkKind = 'review' | 'push' | 'maintenance' | 'trash' | 'prewarm' | 'relink' | 'subvolume-delete' | 'snapshot-ahead'; kind.name(); kind.fromName('push')
//! ```

/// What:
///  The kind of work an item asks for.
/// Why:
///   One record format carries every asynchronous job;
/// the kind picks the consumer's handler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorkKind = 'review' | 'push' | 'maintenance';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkKind {
    /// The standards-review shadow run.
    Review,
    /// An auto-push of the branch.
    Push,
    /// Repository maintenance.
    Maintenance,
    /// Emptying a directory of left-out files that a snapshot-made worktree set aside.
    Trash,
    /// Filling a new worktree's Cargo build directory from its source's,
    /// which the add no longer waits for.
    Prewarm,
    /// Relinking the copied package state of a new worktree,
    /// with each tool's own install,
    /// which the add no longer waits for.
    Relink,
    /// Deleting the subvolume of a removed worktree,
    /// which the removal no longer waits for.
    SubvolumeDelete,
    /// Making a snapshot of a worktree ahead of the next add,
    /// which the add would otherwise force on the person's thread.
    SnapshotAhead,
}

/// Methods over a kind.
impl WorkKind {
    /// The kind's name,
    /// which is also the start of an item's id.
    pub fn name(self) -> &'static str {
        return match self {
            WorkKind::Review => "review",
            WorkKind::Push => "push",
            WorkKind::Maintenance => "maintenance",
            WorkKind::Trash => "trash",
            WorkKind::Prewarm => "prewarm",
            WorkKind::Relink => "relink",
            WorkKind::SubvolumeDelete => "subvolume-delete",
            WorkKind::SnapshotAhead => "snapshot-ahead",
        };
    }

    /// The kind a name spells.
    pub fn from_name(name: &str) -> Option<WorkKind> {
        for kind in [WorkKind::Review, WorkKind::Push, WorkKind::Maintenance, WorkKind::Trash, WorkKind::Prewarm, WorkKind::Relink, WorkKind::SubvolumeDelete, WorkKind::SnapshotAhead] {
            if kind.name() == name {
                return Some(kind);
            }
        }
        return None;
    }
}
