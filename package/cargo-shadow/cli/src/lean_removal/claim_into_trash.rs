//! What:
//! The claim of one item for removal:
//! read its identity,
//! move it into the trash by a rename that never replaces,
//! and read the identity again.
//! Why:
//! The rename is what makes one owner per item:
//! a second tool that races finds the item gone,
//! and a path swapped between the check and the rename is left in the trash for a person to look at.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function itemOf(path: string): Item
//! function trashName(trash: string, item: string, serial: number): string
//! export function claimIntoTrash(item: Item, trash: string, serial: number): string  // throws Refusal
//! ```
/// Imports the item as it was judged.
use super::item_and_deletion_record::Item;
/// Imports the refusals the claim answers with.
use super::refusal_text::Refusal;
/// Imports the device and inode the claim compares and the call that reads them.
use crate::lean_identity::{
    Identity,
    identity_of,
};
/// Imports the final component of a path as text.
use crate::lean_text::name_of;
/// Imports the rename that fails instead of replacing.
use rustix::fs::{
    CWD,
    RenameFlags,
    renameat_with,
};
/// Imports the borrowed and owned path types the claim works with.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the directory creation the trash needs and the error kinds the claim reads.
use std::{
    fs,
    io,
};
/// What:
/// `pub fn item_of(path: &Path) -> io::Result<Item>` reads the identity an item has now.
/// Why:
/// Classification records it,
/// and the removal compares against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const itemOf = (path: string): Item => ({ path, identity: identityOf(path) });
/// ```
pub fn item_of(path: &Path) -> io::Result<Item> {
    return Ok(Item {
        path: path.to_path_buf(),
        identity: identity_of(path)?,
    });
}

/// What:
/// `fn trash_name(trash: &Path, item: &Path, serial: u64) -> PathBuf` is a name in the trash no other claim can share.
/// Why:
/// The process id,
/// the clock and a serial make each claim unique,
/// so the rename never has to replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trashName = (trash: string, item: string, serial: number) => join(trash, `${process.pid}-${Date.now()}-${serial}-${basename(item)}`);
/// ```
fn trash_name(
    trash: &Path,
    item: &Path,
    serial: u64,
) -> PathBuf {
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_) => 0,
    };
    let base: String = name_of(item);
    return trash.join(format!("{}-{nanos}-{serial}-{base}", std::process::id()));
}

/// What:
/// `pub fn claim_into_trash(item: &Item, trash: &Path, serial: u64) -> Result<PathBuf, Refusal>` moves the item into the trash
/// when it still is the judged entry.
/// Why:
/// It is the claim.
/// The trash must be on the item's file system (a sibling directory),
/// so the move is one atomic rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function claimIntoTrash(item: Item, trash: string, serial: number): string  // throws Refusal
/// ```
pub fn claim_into_trash(
    item: &Item,
    trash: &Path,
    serial: u64,
) -> Result<PathBuf, Refusal> {
    let before: Identity = match identity_of(&item.path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(Refusal::Gone(item.path.clone()));
        }
        Err(error) => {
            return Err(Refusal::Io {
                path: item.path.clone(),
                error,
            });
        }
    };
    if before != item.identity {
        return Err(Refusal::Changed {
            path: item.path.clone(),
            judged: item.identity,
            found: before,
        });
    }
    if let Err(error) = fs::create_dir_all(trash) {
        return Err(Refusal::Io {
            path: trash.to_path_buf(),
            error,
        });
    }
    let target: PathBuf = trash_name(trash, &item.path, serial);
    match renameat_with(CWD, &item.path, CWD, &target, RenameFlags::NOREPLACE) {
        Ok(()) => {}
        Err(code) if code == rustix::io::Errno::NOENT => {
            return Err(Refusal::Gone(item.path.clone()));
        }
        Err(code) => {
            return Err(Refusal::Io {
                path: item.path.clone(),
                error: io::Error::from(code),
            });
        }
    }
    let after: Identity = match identity_of(&target) {
        Ok(found) => found,
        Err(error) => {
            return Err(Refusal::Io {
                path: target,
                error,
            });
        }
    };
    if after != item.identity {
        // The path was swapped between the check and the rename: what moved is not what was judged. It stays in the trash.
        return Err(Refusal::Changed {
            path: target,
            judged: item.identity,
            found: after,
        });
    }
    return Ok(target);
}
