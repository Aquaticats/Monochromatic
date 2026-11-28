//! What:
//!  The clone's trash:
//!  the folder one add makes beside its slot,
//!  the manifest that records what the trash holds,
//!  and the small reads inside it.
//! Why:
//!  The exchange keeps the copied Git administration here after the slot gets
//!  the saved pointer back,
//!  and a resume after a kill proves the entry it finds
//!  by this manifest
//!  (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
//!  "What the slot holds while the add fills it").
//!  The manifest takes its name in one rename because no stage claim inside the
//!  mounted clone removes the trash,
//!  so a kill between two writes leaves either
//!  the record that stood or the new one
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore",
//!  step 8).
//!  `add_exchange.rs` swaps the slot's `.git` with one entry of this trash and
//!  would go over the file-size limit carrying the trash's own writes,
//!  so the trash's folder,
//!  manifest and reads live here,
//!  and the checks that pair them with the swap are
//!  `add_exchange_preparation.rs`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const manifest = makeATrash(root, cloneRoot); // .claude/cli-git-trash/<random>
//! writeTheManifestAtomically(trash, updated); // .manifest.pending -> .manifest
//! const bytes = readSmallFile(trash, entry); // one entry's own bytes
//! ```

use crate::fs_beneath::{
    ObjectId,
    create_file_at,
    identity_at,
    make_directory_at,
    open_directory_at,
    open_file_at,
    remove_at,
};
use crate::worktree_trash::TrashManifest;
use std::ffi::OsStr;
use std::io::Write as _;
use std::path::Path;

/// The name the trash's own manifest holds.
const MANIFEST_NAME: &str = ".manifest";

/// The staging name the atomic manifest rewrite fills.
const MANIFEST_STAGING_NAME: &str = ".manifest.pending";

/// The mode of the trash directory and its files.
const TRASH_MODE: u32 = 0o700;

/// The bytes of the random part of a trash's name,
/// the width the wrapper's trash names use.
const TRASH_RANDOM_BYTES: usize = 20;

/// Make one trash directory in the clone and answer its manifest.
pub(super) fn make_a_trash(
    root: &std::fs::File,
    clone_root: &Path,
) -> Result<TrashManifest, String> {
    let parent: std::fs::File = trash_parent(root)?;
    let mut random: [u8; TRASH_RANDOM_BYTES] = [0; TRASH_RANDOM_BYTES];
    getrandom::fill(&mut random).map_err(|error| {
        return format!("the trash's name cannot be taken from the random source ({error})");
    })?;
    let name: String = random
        .iter()
        .map(|byte| return format!("{byte:02x}"))
        .collect();
    let made: bool = make_directory_at(&parent, OsStr::new(name.as_str()), TRASH_MODE)
        .map_err(|error| return format!("the trash {name} cannot be made ({error})"))?;
    if !made {
        return Err(format!(
            "the trash {name} already stands in {}, so a name another command made is never reused",
            clone_root.display()
        ));
    }
    let identity: ObjectId = identity_at(&parent, OsStr::new(name.as_str()))
        .map_err(|error| return format!("the trash {name} cannot be identified ({error})"))?;
    return Ok(TrashManifest {
        name,
        trash: identity,
        entries: Vec::new(),
    });
}

/// Open `.claude/cli-git-trash`,
/// making either folder when it is absent.
fn trash_parent(root: &std::fs::File) -> Result<std::fs::File, String> {
    let claude: std::fs::File = open_or_make(root, OsStr::new(".claude"))?;
    return open_or_make(&claude, OsStr::new("cli-git-trash"));
}

/// Open one folder below `parent`,
/// making it when it is absent.
fn open_or_make(
    parent: &std::fs::File,
    name: &OsStr,
) -> Result<std::fs::File, String> {
    return match open_directory_at(parent, name) {
        Ok(found) => Ok(found),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let made: bool = make_directory_at(parent, name, TRASH_MODE).map_err(|refused| {
                return format!(
                    "the folder {} cannot be made ({refused})",
                    name.to_string_lossy()
                );
            })?;
            if !made {
                return Err(format!(
                    "the folder {} was made by somebody else, so it is not this add's",
                    name.to_string_lossy()
                ));
            }
            open_directory_at(parent, name).map_err(|refused| {
                return format!(
                    "the folder {} cannot be opened ({refused})",
                    name.to_string_lossy()
                );
            })
        }
        Err(refused) => Err(format!(
            "the folder {} cannot be opened ({refused})",
            name.to_string_lossy()
        )),
    };
}

/// Read one small file of the trash,
/// refusing a link or a directory.
pub(super) fn read_small_file(
    trash: &std::fs::File,
    name: &str,
) -> Result<Vec<u8>, String> {
    return read_file_at(trash, OsStr::new(name), Path::new("the trash"));
}

/// Read one regular file below a held folder,
/// refusing a link.
pub(super) fn read_file_at(
    parent: &std::fs::File,
    name: &OsStr,
    shown: &Path,
) -> Result<Vec<u8>, String> {
    let mut file: std::fs::File = open_file_at(parent, name).map_err(|error| {
        return format!("{}/{name:?} cannot be opened ({error})", shown.display());
    })?;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut bytes)
        .map_err(|error| return format!("{}/{name:?} cannot be read ({error})", shown.display()))?;
    return Ok(bytes);
}

/// Write the manifest inside the trash in one rename,
/// replacing nothing else.
pub(super) fn write_the_manifest_atomically(
    trash: &std::fs::File,
    manifest: &TrashManifest,
) -> Result<(), String> {
    let _stale: std::io::Result<()> = remove_at(trash, OsStr::new(MANIFEST_STAGING_NAME), false);
    let mut file: std::fs::File =
        create_file_at(trash, OsStr::new(MANIFEST_STAGING_NAME), 0o600)
            .map_err(|error| return format!("the trash's manifest cannot be staged ({error})"))?;
    file.write_all(manifest.encode().as_bytes())
        .map_err(|error| return format!("the trash's manifest cannot be written ({error})"))?;
    file.sync_all()
        .map_err(|error| return format!("the trash's manifest cannot be flushed ({error})"))?;
    drop(file);
    crate::fs_beneath::rename_at(
        trash,
        OsStr::new(MANIFEST_STAGING_NAME),
        trash,
        OsStr::new(MANIFEST_NAME),
        true,
    )
    .map_err(|error| return format!("the trash's manifest cannot take its name ({error})"))?;
    trash.sync_all().map_err(|error| {
        return format!("the trash cannot be flushed after its manifest ({error})");
    })?;
    return Ok(());
}
