//! What:
//!  The preparation the exchange needs: the trash's manifest, the entry name the
//!  copied administration will take, and the identities on both sides of the
//!  swap.
//! Why:
//!  A resume after a kill reuses the entry the manifest names instead of taking
//!  the next one, and the identities prove each side after the swap
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore", step 8).
//!  `add_exchange.rs` performs the one swap and would go over the file-size
//!  limit carrying this beside it,
//!  and `add_exchange_trash.rs` holds the trash's own folder, manifest and reads,
//!  so this file is the record written before the swap and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prepared = prepareTheExchange({ cloneRoot, pointer, carried });
//! // prepared.alreadySwapped: an earlier run swapped the two
//! ```

use super::add_exchange_trash::{
    make_a_trash,
    read_file_at,
    read_small_file,
    write_the_manifest_atomically,
};
use crate::fs_beneath::{
    ObjectId,
    create_file_at,
    entry_metadata_at,
    identity_at,
    open_directory_at,
    open_root,
};
use crate::worktree_trash::{
    TRASH_PARENT,
    TrashManifest,
    entry_name,
};
use std::ffi::OsStr;
use std::io::Write as _;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  What the exchange needs from before the swap:
///  the trash's manifest,
///  the entry name the copied administration will take,
///  and the identities on both sides.
/// Why:
///  A resume after the manifest is written reuses the entry the manifest names
///  instead of taking the next one, and the identities prove each side after
///  the swap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedExchange = { manifest; entry; copied; pointerFile; alreadySwapped };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedExchange {
    /// The trash's manifest,
    /// with the copied administration's entry in it.
    pub manifest: TrashManifest,
    /// The entry name the pointer file took before the swap.
    pub entry: String,
    /// The copied administration's identity,
    /// which the manifest records for that entry.
    pub copied: ObjectId,
    /// The pointer file's identity before the swap.
    pub pointer_file: ObjectId,
    /// Whether an earlier run already swapped the two.
    pub already_swapped: bool,
}

/// What:
///  Prepare the trash entry and its manifest, or find that an earlier run did.
/// Why:
///  The pointer bytes are written to the trash's next entry, the copied
///  administration's identity is recorded against that entry, and the manifest
///  takes the record in one rename before the swap, so a kill after the swap
///  names what the trash holds.
///
/// # Errors
/// The slot's `.git` is neither the copied directory nor the pointer the trash
/// names,
/// the trash cannot be made or written,
/// or a name the manifest would use already stands.
pub fn prepare_the_exchange(
    clone_root: &Path,
    pointer: &[u8],
    carried: Option<TrashManifest>,
) -> Result<PreparedExchange, String> {
    let root: std::fs::File = open_root(clone_root).map_err(|error| {
        return format!(
            "the clone root {} cannot be opened without a link ({error})",
            clone_root.display()
        );
    })?;
    let manifest: TrashManifest = match carried {
        Some(found) => found,
        None => make_a_trash(&root, clone_root)?,
    };
    let trash_path: PathBuf = clone_root.join(TRASH_PARENT).join(manifest.name.as_str());
    let trash: std::fs::File = open_directory_at(&root, OsStr::new(".claude"))
        .and_then(|claude| return open_directory_at(&claude, OsStr::new("cli-git-trash")))
        .and_then(|parent| return open_directory_at(&parent, OsStr::new(manifest.name.as_str())))
        .map_err(|error| {
            return format!(
                "the trash {} cannot be opened ({error})",
                trash_path.display()
            );
        })?;
    let Some(git) = read_the_entry(&root, OsStr::new(".git"), clone_root)? else {
        return Err(String::from(
            "the slot holds no .git, so the exchange has nothing to swap",
        ));
    };
    if git.is_file {
        return already_the_pointer(&root, &trash, &manifest, pointer, git, clone_root);
    }
    return write_the_pointer_entry(&root, &trash, &manifest, pointer, git.identity, clone_root);
}

/// What one entry of the clone's root is.
pub(super) struct EntryFacts {
    /// Whether it is a regular file.
    pub(super) is_file: bool,
    /// Its identity.
    pub(super) identity: ObjectId,
}

/// Read one entry of the root, or nothing when it is absent.
pub(super) fn read_the_entry(
    root: &std::fs::File,
    name: &OsStr,
    clone_root: &Path,
) -> Result<Option<EntryFacts>, String> {
    let metadata: std::fs::Metadata = match entry_metadata_at(root, name) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "{}/{} cannot be examined ({error})",
                clone_root.display(),
                name.to_string_lossy()
            ));
        }
    };
    let identity: ObjectId = identity_at(root, name).map_err(|error| {
        return format!(
            "{}/{} cannot be identified ({error})",
            clone_root.display(),
            name.to_string_lossy()
        );
    })?;
    return Ok(Some(EntryFacts {
        is_file: metadata.is_file(),
        identity,
    }));
}

/// The exchange already happened: the slot holds the pointer and the manifest
/// names the copied administration at its entry.
fn already_the_pointer(
    root: &std::fs::File,
    trash: &std::fs::File,
    manifest: &TrashManifest,
    pointer: &[u8],
    git: EntryFacts,
    clone_root: &Path,
) -> Result<PreparedExchange, String> {
    let Some(wanted) = manifest.entries.last() else {
        return Err(format!(
            "the slot holds a `.git` file the trash {} does not name, so nothing proves whose it is; Ways forward: read the file and remove the worktree by hand",
            clone_root.display()
        ));
    };
    let entry: String = entry_name(manifest.entries.len() - 1);
    let found: ObjectId = identity_at(trash, OsStr::new(entry.as_str())).map_err(|error| {
        return format!("the trash entry {entry} cannot be identified ({error})");
    })?;
    if found != *wanted {
        return Err(format!(
            "the trash entry {entry} is {} and the manifest records {}, so the sunk copy is not the one the add made",
            found.encode(),
            wanted.encode()
        ));
    }
    let sunk: std::fs::Metadata = entry_metadata_at(trash, OsStr::new(entry.as_str()))
        .map_err(|error| return format!("the trash entry {entry} cannot be examined ({error})"))?;
    if !sunk.is_dir() {
        return Err(format!(
            "the trash entry {entry} is no folder, and the copied administration should stand there"
        ));
    }
    let slot: Vec<u8> = read_file_at(root, OsStr::new(".git"), clone_root)?;
    if slot != pointer {
        return Err(String::from(
            "the slot's `.git` is not the pointer this add saved, so the exchange is not this add's to finish",
        ));
    }
    return Ok(PreparedExchange {
        manifest: manifest.clone(),
        entry,
        copied: *wanted,
        pointer_file: git.identity,
        already_swapped: true,
    });
}

/// Write the pointer bytes to the trash's next entry and record the copied
/// administration against it.
fn write_the_pointer_entry(
    root: &std::fs::File,
    trash: &std::fs::File,
    manifest: &TrashManifest,
    pointer: &[u8],
    copied: ObjectId,
    clone_root: &Path,
) -> Result<PreparedExchange, String> {
    if let Some(last) = manifest.entries.last()
        && *last == copied
    {
        // A kill after the manifest was written and before the swap leaves the
        // pointer entry at the manifest's last name; it is reused, never added
        // again, so the trash holds one entry per object.
        let previous: String = entry_name(manifest.entries.len() - 1);
        let pointer_file: ObjectId =
            identity_at(trash, OsStr::new(previous.as_str())).map_err(|error| {
                return format!("the trash entry {previous} cannot be identified ({error})");
            })?;
        if read_small_file(trash, previous.as_str())? == pointer {
            return Ok(PreparedExchange {
                manifest: manifest.clone(),
                entry: previous,
                copied,
                pointer_file,
                already_swapped: false,
            });
        }
    }
    let next: usize = manifest.entries.len();
    let entry: String = entry_name(next);
    let existing: bool = entry_metadata_at(trash, OsStr::new(entry.as_str())).is_ok();
    let pointer_file: ObjectId = if existing {
        // A kill after the entry was written and before the manifest was
        // rewritten leaves it here; it is reused only when it holds the pointer.
        if read_small_file(trash, entry.as_str())? != pointer {
            return Err(format!(
                "the trash {} already holds {entry}, and it is not this add's pointer; Ways forward: inspect the trash and remove it if it is not the add's",
                clone_root.display()
            ));
        }
        identity_at(trash, OsStr::new(entry.as_str())).map_err(|error| {
            return format!("the trash entry {entry} cannot be identified ({error})");
        })?
    } else {
        let mut file: std::fs::File = create_file_at(trash, OsStr::new(entry.as_str()), 0o600)
            .map_err(|error| return format!("the trash entry {entry} cannot be made ({error})"))?;
        file.write_all(pointer).map_err(|error| {
            return format!("the trash entry {entry} cannot be written ({error})");
        })?;
        file.sync_all().map_err(|error| {
            return format!("the trash entry {entry} cannot be flushed ({error})");
        })?;
        crate::fs_beneath::identity_of(&file).map_err(|error| {
            return format!("the trash entry {entry} cannot be identified ({error})");
        })?
    };
    let mut updated: TrashManifest = manifest.clone();
    updated.entries.push(copied);
    write_the_manifest_atomically(trash, &updated)?;
    root.sync_all().map_err(|error| {
        return format!("the clone root cannot be flushed after the trash record ({error})");
    })?;
    return Ok(PreparedExchange {
        manifest: updated,
        entry,
        copied,
        pointer_file,
        already_swapped: false,
    });
}
