//! What:
//!  What one fill reads and what a filled clone hands back.
//! Why:
//!  The fill touches the clone,
//!  the registration and the records,
//!  and each of
//!  them is named once by the caller that proved it,
//!  so one value carries every input and a reader of any step sees what it may
//!  read without opening the fill itself.
//!  `add_fill.rs` runs the steps in order and would go over the file-size limit
//!  carrying the two types,
//!  the restore is `add_fill_restore.rs`'s,
//!  the leave-out is `add_fill_leave_outs.rs`'s,
//!  and the check is `add_fill_check.rs`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type FillInput = { source; plan; cloneRoot; slot; capture; pointer; nonce; adminPath; quiet };
//! type Filled = { trash?: PendingTrash };
//! ```

use super::add_capture::Capture;
use crate::transaction_git::GitContext;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_trash::PendingTrash;
use std::ffi::OsString;
use std::path::Path;

/// What:
///  Everything one fill reads.
/// Why:
///  The fill touches the clone,
///  the registration and the records,
///  and each of
///  them is named once by the caller that proved it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FillInput = { source; plan; cloneRoot; slot; capture; pointer; nonce; adminPath; quiet };
/// ```
pub struct FillInput<'a> {
    /// The proven clean source.
    pub source: &'a SnapshotSource,
    /// The add's plan,
    /// which names the destination the stand-ins compare with.
    pub plan: &'a AddPlan,
    /// The slot,
    /// which the trash's record names.
    pub slot: &'a str,
    /// The mounted clone's root,
    /// which is the slot.
    pub clone_root: &'a Path,
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The capture the proof compares with.
    pub capture: &'a Capture,
    /// The repository's object name length in bytes.
    pub hash_length: usize,
    /// The bytes Git wrote into the slot's `.git`.
    pub pointer: &'a [u8],
    /// This add's nonce,
    /// which the marker and the record home are keyed by.
    pub nonce: &'a crate::worktree_clone_mount_protocol::RequestNonce,
    /// The wrapper's record home.
    pub home: &'a Path,
    /// The registration's directory,
    /// which receives the private index.
    pub admin_path: &'a Path,
    /// The repository's Git context,
    /// which the restore's own Git runs start through.
    pub context: &'a GitContext,
    /// The real Git this command resolved.
    pub real_git: &'a Path,
    /// The variables every Git child of this command receives.
    pub overlay: &'a [(OsString, OsString)],
    /// Whether the person asked for no lines.
    pub quiet: bool,
    /// The test-only switch that ends a control run at one step.
    pub stops: &'a super::add_route_plan::Stops,
}

/// What:
///  What a filled clone hands back.
/// Why:
///  The trash the leave-out made and the exchange extended is the disposal's
///  work,
///  and the add hands it over after the fill.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Filled = { trash?: PendingTrash };
/// ```
#[derive(Debug)]
pub struct Filled {
    /// The trash the add leaves for the service.
    pub trash: Option<PendingTrash>,
}
