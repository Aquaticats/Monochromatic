//! What:
//!  What one claim and the two records beside it say about the copy an interrupted `git worktree add` owes:
//!  the selection the add recorded,
//!  the destination it registered,
//!  and the identity that proves the worktree standing there is still that one.
//! Why:
//!  The check that makes an owed copy and the record of a copy that cannot be made read the same fields,
//!  so they are parsed once here and named for both.
//!  How one such copy is made is in `worktree_copy_owed_copy_making_it.rs`,
//!  and the pass that finds the adds that owe one is in `worktree_copy_owed_copy.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owedAdd = owedAddOf(claimPath, container); // { claim, container, record, destination, owed }
//! ```

use crate::worktree_add_binding::{
    OwedCopy,
    OwedSelection,
    Recorded,
    decode_record,
    owed_copy_of,
};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage_claim::DESTINATION_SUFFIX;
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  The selection an add's record names,
/// as the copy takes it.
/// Why:
///   The record writes the two listings a source can have,
/// and a reader takes only those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selectionOf = (owed) => owed.selection === 'main-checkout' ? CopySelection.MainCheckout : CopySelection.Ignored;
/// ```
pub(super) fn selection_of(owed: &OwedCopy) -> CopySelection {
    return match owed.selection {
        OwedSelection::Ignored => CopySelection::Ignored,
        OwedSelection::MainCheckout => CopySelection::MainCheckout,
    };
}

/// What:
///  One add's unfinished state,
/// as the claim and its two records say it.
/// Why:
///   Every check below reads the same fields,
/// so they are parsed once and named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwedAdd = { claim: string; container: string; record: string; destination: string; owed: OwedCopy };
/// ```
pub(super) struct OwedAdd {
    /// The claim file,
    /// whose stage name the copy fills.
    pub(super) claim: PathBuf,
    /// The stage container the claim names.
    pub(super) container: PathBuf,
    /// The record of the destination written beside the claim.
    pub(super) record: PathBuf,
    /// The worktree the add registered.
    pub(super) destination: PathBuf,
    /// The identity the record holds for that worktree,
    /// when it was bound before the command died.
    pub(super) destination_identity: Option<crate::fs_beneath::ObjectId>,
    /// The ignored-state copy the add owes.
    pub(super) owed: OwedCopy,
}

/// What:
///  The add's unfinished state,
/// or nothing when the claim is not one an add made or its record names no copy.
/// Why:
///   A copy's own claim records no destination,
/// and a record written before the source was written names none:
/// both settle exactly as they did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owedAdd = owedAddOf(claimPath, container);
/// ```
pub(super) fn owed_add_of(
    claim: &Path,
    container: &Path,
) -> Option<OwedAdd> {
    let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
    let text: String = std::fs::read_to_string(record.as_path()).ok()?;
    let (destination, destination_identity): (PathBuf, Option<crate::fs_beneath::ObjectId>) =
        match decode_record(text.as_str())? {
            Recorded::Planned(named) => (named, None),
            Recorded::Registered(binding) => (binding.destination, Some(binding.destination_id)),
        };
    let owed: OwedCopy = owed_copy_of(text.as_str())?;
    return Some(OwedAdd {
        claim: claim.to_path_buf(),
        container: container.to_path_buf(),
        record,
        destination,
        destination_identity,
        owed,
    });
}
