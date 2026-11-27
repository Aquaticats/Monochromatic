//! What:
//!  Clearing one incomplete registration through the recipe,
//! and the names the
//!  person reads of what went.
//! Why:
//!  A kill inside Git's own add can leave a registration no worktree stands
//!  behind,
//! and the recipe is the one definition of what may be removed and in
//!  which order
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "A resume after a kill between Git's add and the registered write").
//!  `add_registration.rs` decides to clear and would go over the file-size limit
//!  carrying this,
//!  so the removal and its words live here,
//! and the decision that reaches them
//!  is the scan's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const names = clearTheEntry(scan, entry); // gitdir, commondir, HEAD, locked, ...
//! ```

use super::add_registration_reads::ScanInput;
use crate::worktree_clone_binding_record::registration_finder::EntryFacts;
use crate::worktree_clone_binding_record::registration_recipe::{
    NoSeams,
    RecipeRefusal,
    ToClear,
    clear_registration,
};
use std::ffi::OsStr;
use std::path::PathBuf;

/// Clear one incomplete entry through the recipe and answer what went.
pub(super) fn clear_the_entry(
    input: &ScanInput<'_>,
    entry: &EntryFacts,
) -> Result<Vec<String>, String> {
    let slot_parent_path: PathBuf = input
        .slot_path
        .parent()
        .ok_or_else(|| return format!("the slot {} names no parent", input.slot_path.display()))?
        .to_path_buf();
    let worktrees_parent_path: PathBuf = input.common_dir.join("worktrees");
    let slot_parent: std::fs::File = crate::fs_beneath::open_root(slot_parent_path.as_path())
        .map_err(|error| {
            return format!(
                "the worktrees folder {} cannot be opened ({error})",
                slot_parent_path.display()
            );
        })?;
    let worktrees_parent: std::fs::File =
        crate::fs_beneath::open_root(worktrees_parent_path.as_path()).map_err(|error| {
            return format!(
                "the registrations folder {} cannot be opened ({error})",
                worktrees_parent_path.display()
            );
        })?;
    let registration: PathBuf = worktrees_parent_path.join(entry.name.as_str());
    let to: ToClear<'_> = ToClear {
        slot_parent: &slot_parent,
        slot_pointer_name: input
            .slot_path
            .file_name()
            .ok_or_else(|| return String::from("the slot names no folder"))?,
        slot: input.slot_path,
        worktrees_parent: &worktrees_parent,
        registration_name: OsStr::new(entry.name.as_str()),
        registration: registration.as_path(),
    };
    clear_registration(&to, &mut NoSeams)
        .map_err(|refusal: RecipeRefusal| return refusal.message())?;
    return Ok(names_of_the_removed(entry));
}

/// The names one cleared row had,
/// in the recipe's order,
/// for the line the person reads.
fn names_of_the_removed(entry: &EntryFacts) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    if entry.slot_git_pointer.is_some() {
        names.push(String::from("the slot's `.git`"));
    }
    if entry.gitdir.is_some() {
        names.push(String::from("gitdir"));
    }
    if entry.commondir {
        names.push(String::from("commondir"));
    }
    if entry.head {
        names.push(String::from("HEAD"));
    }
    if entry.locked.is_some() {
        names.push(String::from("locked"));
    }
    names.push(format!("worktrees/{}", entry.name));
    return names;
}
