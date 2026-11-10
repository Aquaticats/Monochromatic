//! What:
//!  The manifest a trash carries inside itself,
//! what it proves,
//! and how a later pass reads it.
//! Why:
//!  The work item that empties a trash is written after the snapshot is published,
//! so a command killed between the
//!  two leaves a trash no record names.
//! The manifest written inside the trash before the swap travels with it,
//! so whatever a
//!  later pass finds already says which directories the wrapper moved there,
//! and the next pass can record that trash for the
//!  service without a guess.
//!   The proof is the one the handler already uses:
//! the identity of the trash directory and of each entry in it,
//! which
//!  only a rename performed by the leave-out produces.
//! A directory below the trash parent that cannot show this is the
//!  person's own and is left untouched.
//!   A trash made by a release before this manifest existed carries none,
//! so it is carried and named rather than deleted:
//! the
//!  wrapper only removes what it can prove it made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const manifest = writeTrashManifest({ root, trash }); // root/.claude/cli-git-trash/<name>/.manifest
//! const proved = readTrashManifest({ root, name });     // TrashManifest | undefined, identities compared
//! ```

use crate::fs_beneath::{handle_path, identity_of, open_directory_at, open_root, plain_components};
use crate::plain_file::read_plain_file;
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_manifest::TrashManifest;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  The name of the file a trash carries inside itself,
/// below its own name in a worktree.
/// Why:
///   The trash directory is the wrapper's own,
/// named with forty hexadecimal digits,
/// so one plain name inside it
///  cannot be mistaken for a directory the leave-out set aside,
/// which are all named `d<index>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TRASH_MANIFEST_FILE = '.manifest';
/// ```
pub const MANIFEST_FILE_NAME: &str = ".manifest";

/// The most bytes a trash's own manifest may take:
/// the encoded manifest of the thousands of directories one snapshot can set aside.
///  An identity is a few bytes of digits,
/// and the measured leave-out of this repository sets aside hundreds of
///  directories,
/// so a megabyte is far past any real manifest and still bounds the read.
const MANIFEST_LIMIT: u64 = 1024 * 1024;

/// What:
///  Write the manifest of `manifest` inside the trash directory it names,
/// below `root`,
/// which holds the trash parent.
/// Why:
///   The file is written before the snapshot is swapped into place,
/// so a kill after the swap leaves a trash that says what it
///  is.
/// A kill during this write is harmless:
/// the swap has not happened,
/// so the trash is still inside the stage,
///  and the claim of that stage removes it.
/// That is why the write needs no staging name of its own,
/// and leaves no
///  half-written file beside the manifest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeTrashManifest = ({ root, manifest }) => writeFileSync(join(root, TRASH_PARENT, manifest.name, '.manifest'), encodeManifest(manifest));
/// ```
pub fn write(root: &Path, manifest: &TrashManifest) -> Result<(), String> {
    let trash: std::fs::File = opened_trash(root, manifest.name.as_str())?;
    let path: PathBuf = handle_path(&trash).join(MANIFEST_FILE_NAME);
    return match std::fs::write(path.as_path(), manifest.encode().as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("the manifest inside the trash {} cannot be written ({error})", shown(root, manifest.name.as_str()).display())),
    };
}

/// What:
///  The manifest a trash below `root` wrote about itself,
/// when its identities are that trash's own,
/// and nothing otherwise.
/// Why:
///   Only a manifest whose trash identity is the directory's proves the leave-out made it,
/// and every entry it
///  names is compared with the object found there before it is removed.
/// A missing,
/// unreadable or damaged manifest
///  proves nothing,
/// and a trash nothing proves is left where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readTrashManifest = ({ root, name }) => { const trash = openTrash(root, name); const manifest = decode(name, readFileSync(join(trash, '.manifest'))); return identity(trash) === manifest.trash ? manifest : undefined; };
/// ```
pub fn read(root: &Path, name: &str) -> Option<TrashManifest> {
    let trash: std::fs::File = match opened_trash(root, name) {
        Ok(found) => found,
        Err(_unreadable) => return None,
    };
    let manifest: TrashManifest = read_its_own(&trash, name)?;
    let found: crate::fs_beneath::ObjectId = identity_of(&trash).ok()?;
    if manifest.trash != found {
        return None;
    }
    return Some(manifest);
}

/// What:
///  Whether the leftover file of a trash is the manifest the wrapper wrote,
/// and not a person's file that shares the name.
/// Why:
///   The emptying removes the manifest before it removes the trash directory,
/// so a file it cannot read as this trash's manifest
///  must keep the whole directory,
/// as every other thing the snapshot did not put there does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOwnManifest = ({ trash, name }) => decode(name, readFileSync(join(trash, '.manifest'))) !== undefined;
/// ```
pub fn is_its_own(trash: &std::fs::File, name: &str) -> bool {
    return read_its_own(trash, name).is_some();
}

/// The encoded manifest inside `trash`,
/// read as the manifest of the trash `name`,
/// and nothing when the file is missing,
/// unreadable or not that manifest.
fn read_its_own(trash: &std::fs::File, name: &str) -> Option<TrashManifest> {
    let path: PathBuf = handle_path(trash).join(MANIFEST_FILE_NAME);
    let bytes: Vec<u8> = read_plain_file(path.as_path(), MANIFEST_LIMIT).ok()?;
    let text: String = String::from_utf8(bytes).ok()?;
    return TrashManifest::decode(name, text.as_str()).ok();
}

/// What:
///  The trash directory `name` below the trash parent of `root`,
/// opened one plain name at a time,
/// and the reason when that middle step is not a plain directory.
/// Why:
///   Every reader of a trash's own manifest opens the trash the same way the emptying does,
/// so no link is
///  ever followed and a path that cannot be read is an error rather than a proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const openedTrash = (root, name) => openDirectoryAt(openDirectoryBeneath(openRoot(root), TRASH_PARENT.split('/')), name);
/// ```
fn opened_trash(root: &Path, name: &str) -> Result<std::fs::File, String> {
    let held: std::fs::File = match open_root(root) {
        Ok(file) => file,
        Err(error) => return Err(format!("{} cannot be opened ({error})", root.display())),
    };
    let names: Vec<std::ffi::OsString> = match plain_components(Path::new(TRASH_PARENT)) {
        Ok(found) => found,
        Err(error) => return Err(format!("the trash parent is not a plain path ({error})")),
    };
    let mut parent: std::fs::File = held;
    for part in names {
        parent = match open_directory_at(&parent, part.as_os_str()) {
            Ok(next) => next,
            Err(error) => return Err(format!("{} cannot be opened ({error})", shown(root, name).display())),
        };
    }
    return match open_directory_at(&parent, OsStr::new(name)) {
        Ok(trash) => Ok(trash),
        Err(error) => Err(format!("{} is not a plain directory ({error})", shown(root, name).display())),
    };
}

/// The path a message names for the trash `name` of `root`.
fn shown(root: &Path, name: &str) -> PathBuf {
    return root.join(TRASH_PARENT).join(name);
}

/// Controls of the manifest a trash carries stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_own_manifest_tests.rs"]
mod tests;
