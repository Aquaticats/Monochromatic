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
//! const leaving = itsOwnManifest({ trash, name });      // { manifest, file } when the emptying may remove it
//! ```

use crate::fs_beneath::{handle_path, identity_of, open_directory_at, open_file_at, open_root, plain_components};
use crate::plain_file::write_plain_file;
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
/// The name is created and never replaces what is at it,
/// so a link another process put there is
///  refused instead of written through,
/// a file it put there keeps its bytes,
/// and a pipe is refused instead of waited on:
/// none is a file the wrapper made,
/// and no bytes of the manifest's reach anything outside the trash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeTrashManifest = ({ root, manifest }) => writePlainFile(join(root, TRASH_PARENT, manifest.name, '.manifest'), encodeManifest(manifest));
/// ```
pub fn write(root: &Path, manifest: &TrashManifest) -> Result<(), String> {
    let trash: std::fs::File = opened_trash(root, manifest.name.as_str())?;
    let path: PathBuf = handle_path(&trash).join(MANIFEST_FILE_NAME);
    return match write_plain_file(path.as_path(), manifest.encode().as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("the manifest inside the trash {} cannot be written ({error})", shown(root, manifest.name.as_str()).display())),
    };
}

/// What:
///  The manifest a trash carries inside itself,
///  with the identity of the file that carries it.
/// Why:
///   The bytes say which trash the leave-out made,
/// and the identity of the file they were read through is what the emptying compares again
///  before it unlinks that file,
/// so a file another process moved onto the name is never removed unread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnManifest = { manifest: TrashManifest; file: ObjectId };
/// ```
pub struct OwnManifest {
    /// What the leave-out made and moved.
    pub manifest: TrashManifest,
    /// The object that held those bytes when they were read.
    pub file: crate::fs_beneath::ObjectId,
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
    let own: OwnManifest = its_own(&trash, name)?;
    return Some(own.manifest);
}

/// What:
///  The manifest the wrapper wrote inside `trash`,
/// and the identity of the file that holds it,
///  or nothing when that file is not this trash's own.
/// Why:
///   The emptying removes that file before it removes the trash directory,
/// so a file it cannot read as this
///  trash's manifest must keep the whole directory,
/// as every other thing the snapshot did not put there does.
/// The identity is taken from the handle the bytes were read through:
/// a file moved onto the name after the read is
///  another object,
/// and the emptying compares this identity again immediately before it unlinks,
/// so the file it removes
///  is the one it examined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itsOwnManifest = ({ trash, name }) => { const file = openAt(trash, '.manifest', O_NOFOLLOW | O_NONBLOCK); const manifest = decode(name, read(file)); return manifest.trash === identity(trash) ? { manifest, file: identity(file) } : undefined; };
/// ```
pub fn its_own(trash: &std::fs::File, name: &str) -> Option<OwnManifest> {
    let file: std::fs::File = open_file_at(trash, OsStr::new(MANIFEST_FILE_NAME)).ok()?;
    if !file.metadata().ok()?.file_type().is_file() {
        return None;
    }
    let carrier: crate::fs_beneath::ObjectId = identity_of(&file).ok()?;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(&file, MANIFEST_LIMIT + 1), &mut bytes).ok()?;
    if bytes.len() as u64 > MANIFEST_LIMIT {
        return None;
    }
    let text: String = String::from_utf8(bytes).ok()?;
    let manifest: TrashManifest = TrashManifest::decode(name, text.as_str()).ok()?;
    if manifest.trash != identity_of(trash).ok()? {
        return None;
    }
    return Some(OwnManifest { manifest, file: carrier });
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
