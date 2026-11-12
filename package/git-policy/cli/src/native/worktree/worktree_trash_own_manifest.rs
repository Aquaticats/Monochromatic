//! What:
//!  The manifest a trash carries inside itself,
//! what it proves,
//! how a later pass reads it,
//! and whether anything was written into a recorded entry since it was written.
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
use crate::diagnostic_log::debug;
use crate::plain_file::write_plain_file;
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_manifest::{TrashManifest, entry_name};
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
///  with the identity of the file that carries it and the instant it was written.
/// Why:
///   The bytes say which trash the leave-out made,
/// and the identity of the file they were read through is what the emptying compares again
///  on the object it takes the name to before it removes that object,
/// so a file another process moved onto the name is never removed unread.
/// The instant is the file's own modification time:
/// everything the leave-out moved in was there when the manifest was written,
/// so a recorded entry whose directory is newer than it
///  has had something written into it since,
/// and what is there is somebody's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnManifest = { manifest: TrashManifest; file: ObjectId; written: SystemTime };
/// ```
pub struct OwnManifest {
    /// What the leave-out made and moved.
    pub manifest: TrashManifest,
    /// The object that held those bytes when they were read.
    pub file: crate::fs_beneath::ObjectId,
    /// When the wrapper wrote those bytes.
    pub written: std::time::SystemTime,
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
/// and the emptying takes the name to a name of its own and compares this identity on what it took,
/// so the file it removes
///  is the one it examined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itsOwnManifest = ({ trash, name }) => { const file = openAt(trash, '.manifest', O_NOFOLLOW | O_NONBLOCK); const manifest = decode(name, read(file)); return manifest.trash === identity(trash) ? { manifest, file: identity(file) } : undefined; };
/// ```
pub fn its_own(trash: &std::fs::File, name: &str) -> Option<OwnManifest> {
    let file: std::fs::File = open_file_at(trash, OsStr::new(MANIFEST_FILE_NAME)).ok()?;
    let found: std::fs::Metadata = file.metadata().ok()?;
    if !found.file_type().is_file() {
        return None;
    }
    let written: std::time::SystemTime = found.modified().ok()?;
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
    return Some(OwnManifest { manifest, file: carrier, written });
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

/// What:
///  The first recorded entry of `trash` that was written into after the manifest was written,
/// as the reason it stays,
///  or nothing when nothing was.
/// Why:
///   The manifest records the identity of each directory the leave-out moved,
/// and nothing about what stands inside one,
/// so a file written into a recorded entry after the manifest is a file the wrapper never read and never recorded.
/// An entry's own directory is newer than the manifest exactly when something was written directly inside it,
///  which is the check this pays for:
/// one `statx` per entry,
/// never a walk of what the entry holds,
/// so an add whose trash is emptied pays no look at the hundreds of thousands of entries a snapshot leaves out.
/// A change made below the entry's own directory leaves that directory's instant alone and is not seen;
/// `SPEC.md` states that limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writtenInto = (trash, own) => own.manifest.entries.findIndex((_, i) => mtime(join(trash, entryName(i))) > own.written);
/// ```
pub fn written_into_after(trash: &std::fs::File, own: &OwnManifest, inside: &Path) -> Option<String> {
    for (index, _recorded) in own.manifest.entries.iter().enumerate() {
        let name: String = entry_name(index);
        let Ok(found) = crate::fs_beneath::entry_metadata_at(trash, std::ffi::OsStr::new(name.as_str())) else { continue };
        let Ok(modified) = found.modified() else { continue };
        if modified > own.written {
            return Some(format!(
                "{} was written into after the wrapper wrote what it moved into the trash, so it is left alone: the emptying deletes only what the snapshot recorded putting there, and what is in it now exists nowhere else",
                inside.join(name.as_str()).display()
            ));
        }
    }
    return None;
}

/// What:
///  Move the manifest's instant forward when the emptying leaves the trash standing.
/// Why:
///   An entry the emptying partly removed has a directory newer than the manifest,
/// and the next attempt would read the emptying's own unlinks as somebody's write and leave the trash for ever.
/// The instant is only ever the reference of that check,
/// so moving it forward is how the next attempt starts where this one ended;
/// a file written into an entry *after* this instant is caught as before,
/// and one written during this attempt is not.
/// The instant is the later of the clock and the newest entry's own,
/// so a clock that stepped back while the emptying ran cannot make the emptying's own work look like a person's write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moveTheInstantForward = (trash, manifest) => utimensat(trash, MANIFEST_FILE_NAME, max(now(), ...entries.map(mtime)));
/// ```
pub fn move_the_instant_forward(trash: &std::fs::File, manifest: &TrashManifest) {
    let mut instant: std::time::SystemTime = std::time::SystemTime::now();
    for index in 0..manifest.entries.len() {
        let name: String = entry_name(index);
        if let Ok(found) = crate::fs_beneath::entry_metadata_at(trash, std::ffi::OsStr::new(name.as_str()))
            && let Ok(modified) = found.modified()
            && modified > instant
        {
            instant = modified;
        }
    }
    let since_the_epoch: std::time::Duration = instant.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let times: [libc::timespec; 2] = [
        libc::timespec { tv_sec: since_the_epoch.as_secs() as libc::time_t, tv_nsec: since_the_epoch.subsec_nanos() as libc::c_long },
        libc::timespec { tv_sec: since_the_epoch.as_secs() as libc::time_t, tv_nsec: since_the_epoch.subsec_nanos() as libc::c_long },
    ];
    if let Err(error) = crate::fs_beneath::set_times_at(trash, OsStr::new(MANIFEST_FILE_NAME), times) {
        debug("worktreeTrashOwnManifest", format!("the instant of the manifest cannot be moved forward ({error}), so a later attempt may read this one's work as somebody's write").as_str());
    }
}

/// The path a message names for the trash `name` of `root`.
fn shown(root: &Path, name: &str) -> PathBuf {
    return root.join(TRASH_PARENT).join(name);
}

/// Controls of the manifest a trash carries stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_own_manifest_tests.rs"]
mod tests;
