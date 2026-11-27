//! What:
//!  The fill:
//! the frozen proof,
//! the proof's phase marker,
//! the clone's leave-outs,
//!  the private index,
//! the restore,
//! the pointer's exchange,
//! and the check of the
//!  result,
//! in the order the design writes them.
//! Why:
//!  The order is the designs' and nothing under a real name is published before
//!  the finished tree is proven
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore":
//! steps 2 to 8).
//!  Step 10's clean source wrote nothing through Git,
//! so the private index was
//!  the captured index normalized and published.
//! Step 11 adds the restore:
//! the
//!  private index is the index the btrfs rung's own `restore_dirty` reads and
//!  writes,
//! so the clone is reset to the commit the caller asked for and the
//!  paths Git must write go through Git's own conversion.
//!  A resume after any kill reads the states those writes leave:
//! the marker says
//!  the proof passed,
//! an existing trash is the leave-out's own,
//! the private
//!  name's absence with the real name present says the publication ran,
//! and the
//!  pointer file with the trash's manifest says the exchange happened.
//!  What the fill reads is `add_fill_input_and_result.rs`'s,
//!  the restore is `add_fill_restore.rs`'s,
//!  the leave-out is `add_fill_leave_outs.rs`'s,
//!  and the check is `add_fill_check.rs`'s,
//!  so this file is the one order those steps hold.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const trash = fillTheClone({ source, plan, cloneRoot, capture, pointer, admin });
//! ```

use super::add_capture_store::{
    marker_is_ours,
    publish_marker,
};
use super::add_exchange::{
    PreparedExchange,
    exchange_the_git_entry,
    prepare_the_exchange,
};
use super::add_fill_check::check_the_result;
use super::add_fill_leave_outs::{
    leave_out_the_clone,
    the_existing_trash,
};
use super::add_fill_restore::{
    restore_the_clone,
    the_restore_still_runs,
};
use super::add_frozen_proof::prove_the_clone;
use super::add_index::{
    index_for_the_fill,
    pending_stands,
    publish_the_private_index,
    published_stands,
    write_the_private_index,
};
use crate::fs_beneath::{
    entry_metadata_at,
    open_root,
};
use crate::private_storage::read_regular_file;
use crate::worktree_trash::{
    PendingTrash,
    TrashManifest,
};
use std::ffi::OsStr;

/// What:
///  Everything one fill reads.
/// Why:
///  The steps behind this file read the same value,
/// so the fill's caller names
///  each fact once.
pub use super::add_fill_input_and_result::FillInput;

/// What:
///  What a filled clone hands back.
/// Why:
///  The trash the leave-out made is the disposal's work,
/// and the add hands it
///  over after the fill.
pub use super::add_fill_input_and_result::Filled;

/// The trash a filled clone holds,
/// for the resume that finds the exchange already done.
pub use super::add_fill_leave_outs::the_trash_of_a_filled_clone;

/// What:
///  Prove the mounted clone,
/// leave out what must not be carried,
/// publish the
///  index,
/// exchange the pointer,
/// and check the result.
/// Why:
///  Each step reads the state a kill at the step before it leaves,
/// so a resume
///  continues instead of repeating a write.
///
/// # Errors
/// The proof refused,
/// a leave-out refused,
/// the index could not be written or published,
/// the exchange refused,
/// or the result is not the worktree the record names.
pub fn fill_the_clone(input: &FillInput<'_>) -> Result<Filled, String> {
    let root: std::fs::File = open_root(input.clone_root).map_err(|error| {
        return format!(
            "the clone root {} cannot be opened without a link ({error})",
            input.clone_root.display()
        );
    })?;
    let already_exchanged: bool = entry_metadata_at(&root, OsStr::new(".git"))
        .map(|found| return found.is_file())
        .unwrap_or(false);
    if !already_exchanged {
        if marker_is_ours(input.home, input.slot, input.nonce)? {
            // The proof passed before the first write; the tree has been written
            // since, so proving it again would refuse the add's own work.
        } else {
            prove_the_clone(input.capture, input.clone_root)?;
            input.stops.after("proof");
            publish_marker(
                &crate::fs_beneath::open_root(input.home).map_err(|error| {
                    return format!(
                        "the record home {} cannot be opened ({error})",
                        input.home.display()
                    );
                })?,
                input.slot,
                input.nonce,
            )
            .map_err(|error| return format!("the proof's marker cannot be published ({error})"))?;
            input.nonce.as_str();
        }
    }
    let manifest: Option<TrashManifest> = match already_exchanged {
        true => the_existing_trash(input.clone_root)?,
        false => {
            let left_out: Option<TrashManifest> = leave_out_the_clone(input)?;
            input.stops.after("leave-outs");
            left_out
        }
    };
    if !published_stands(input.admin_path) && !pending_stands(input.admin_path) {
        // The seed is the clone's copied index, which D1 just proved equal to the
        // capture: a fresh read of the source's index could hold a later write.
        let seed: Vec<u8> = read_regular_file(input.clone_root.join(".git").join("index").as_path(), input.capture.index.size.saturating_add(1)).map_err(|refusal| {
            return format!("the clone's copied index cannot be read for the fill ({refusal:?}); Ways forward: run the add again with nothing writing the main checkout");
        })?;
        let bytes: Vec<u8> = index_for_the_fill(seed.as_slice(), input.hash_length)?;
        // A killed Git leaves its waiting index and its lock behind, and the write
        // below refuses a name that stands: the wrapper's own step clears both.
        crate::worktree_copy_interrupted_add_settlement::clear_the_waiting_index(input.admin_path);
        let admin: std::fs::File = open_root(input.admin_path).map_err(|error| {
            return format!(
                "the registration {} cannot be opened ({error})",
                input.admin_path.display()
            );
        })?;
        write_the_private_index(&admin, bytes.as_slice())?;
        input.stops.after("index");
    }
    if the_restore_still_runs(input.admin_path, input.clone_root, already_exchanged) {
        restore_the_clone(input)?;
        input.stops.after("restore");
    }
    let prepared: PreparedExchange =
        prepare_the_exchange(input.clone_root, input.pointer, manifest)?;
    input.stops.after("trash-entry");
    let admin: std::fs::File = open_root(input.admin_path).map_err(|error| {
        return format!(
            "the registration {} cannot be opened ({error})",
            input.admin_path.display()
        );
    })?;
    if !published_stands(input.admin_path) {
        publish_the_private_index(&admin, input.admin_path)?;
    }
    exchange_the_git_entry(input.clone_root, &prepared)?;
    input.stops.after("exchange");
    check_the_result(input)?;
    // The filled worktree is no resumable state, so the proof's records go in
    // the design's order: the marker first, then the capture. A kill between
    // them leaves a capture whose nonce the next add clears.
    super::add_capture_store::remove_capture(input.home, input.slot);
    return Ok(Filled {
        trash: Some(PendingTrash {
            destination: input.clone_root.to_path_buf(),
            common_dir: input.common_dir.to_path_buf(),
            manifest: prepared.manifest,
            taken_over: 0,
            quiet: input.quiet,
        }),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_fill_tests.rs"]
mod tests;
