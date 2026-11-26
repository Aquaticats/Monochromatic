//! What:
//!  The pointer's exchange with the copied Git administration, and the trash
//!  entry that holds the pointer bytes until then.
//! Why:
//!  The slot's path must hold a `.git` at every moment: the copied directory
//!  until the exchange and the saved pointer after it
//!  (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
//!  "What the slot holds while the add fills it").
//!  One `renameat2(RENAME_EXCHANGE)` changes the two entries in one instant, so
//!  no kill leaves the folder without one
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore", step 8).
//!  The copied administration lands in the new worktree's trash under the name
//!  its manifest records, and the disposal empties it after the add returns;
//!  the manifest is written atomically because no stage claim inside the
//!  mounted clone removes the trash.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prepared = prepareTheExchange({ cloneRoot, pointer, carried });
//! exchangeTheGitEntry(prepared); // .git <-> trash/dN in one call
//! ```

use crate::fs_beneath::{
    ObjectId,
    create_file_at,
    entry_metadata_at,
    identity_at,
    make_directory_at,
    open_directory_at,
    open_file_at,
    open_root,
    remove_at,
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

/// The name the trash's own manifest holds.
const MANIFEST_NAME: &str = ".manifest";

/// The staging name the atomic manifest rewrite fills.
const MANIFEST_STAGING_NAME: &str = ".manifest.pending";

/// The mode of the trash directory and its files.
const TRASH_MODE: u32 = 0o700;

/// The bytes of the random part of a trash's name,
/// the width the wrapper's trash names use.
const TRASH_RANDOM_BYTES: usize = 20;

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
struct EntryFacts {
    /// Whether it is a regular file.
    is_file: bool,
    /// Its identity.
    identity: ObjectId,
}

/// Read one entry of the root, or nothing when it is absent.
fn read_the_entry(
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

/// Make one trash directory in the clone and answer its manifest.
fn make_a_trash(
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

/// Open `.claude/cli-git-trash`, making either folder when it is absent.
fn trash_parent(root: &std::fs::File) -> Result<std::fs::File, String> {
    let claude: std::fs::File = open_or_make(root, OsStr::new(".claude"))?;
    return open_or_make(&claude, OsStr::new("cli-git-trash"));
}

/// Open one folder below `parent`, making it when it is absent.
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

/// Read one small file of the trash, refusing a link or a directory.
fn read_small_file(
    trash: &std::fs::File,
    name: &str,
) -> Result<Vec<u8>, String> {
    return read_file_at(trash, OsStr::new(name), Path::new("the trash"));
}

/// Read one regular file below a held folder, refusing a link.
fn read_file_at(
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

/// Exchange two entries that live in different held folders,
/// in one `RENAME_EXCHANGE` call.
fn exchange_between(
    first_dir: &std::fs::File,
    first: &OsStr,
    second_dir: &std::fs::File,
    second: &OsStr,
) -> std::io::Result<()> {
    use std::os::fd::AsRawFd as _;
    use std::os::unix::ffi::OsStrExt as _;
    let first_name: std::ffi::CString =
        std::ffi::CString::new(first.as_bytes()).map_err(|_nul| {
            return std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the first name holds a NUL",
            );
        })?;
    let second_name: std::ffi::CString =
        std::ffi::CString::new(second.as_bytes()).map_err(|_nul| {
            return std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the second name holds a NUL",
            );
        })?;
    // SAFETY: both names outlive the call and both folders are open descriptors.
    let answer: libc::c_int = unsafe {
        libc::renameat2(
            first_dir.as_raw_fd(),
            first_name.as_ptr(),
            second_dir.as_raw_fd(),
            second_name.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    };
    if answer != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// Write the manifest inside the trash in one rename, replacing nothing else.
fn write_the_manifest_atomically(
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

/// What:
///  Exchange the pointer file in the trash with the slot's copied `.git`.
/// Why:
///  One call changes both names in one instant, so the slot holds one of them
///  and never neither.
///
/// # Errors
/// The two entries are not what the preparation found,
/// the exchange is refused,
/// or either name does not hold what the swap should have put there.
pub fn exchange_the_git_entry(
    clone_root: &Path,
    prepared: &PreparedExchange,
) -> Result<(), String> {
    if prepared.already_swapped {
        return Ok(());
    }
    let root: std::fs::File = open_root(clone_root).map_err(|error| {
        return format!(
            "the clone root {} cannot be opened without a link ({error})",
            clone_root.display()
        );
    })?;
    let trash_path: PathBuf = clone_root
        .join(TRASH_PARENT)
        .join(prepared.manifest.name.as_str());
    let trash: std::fs::File = open_directory_at(&root, OsStr::new(".claude"))
        .and_then(|claude| return open_directory_at(&claude, OsStr::new("cli-git-trash")))
        .and_then(|parent| {
            return open_directory_at(&parent, OsStr::new(prepared.manifest.name.as_str()));
        })
        .map_err(|error| {
            return format!(
                "the trash {} cannot be opened ({error})",
                trash_path.display()
            );
        })?;
    let copied: EntryFacts =
        read_the_entry(&root, OsStr::new(".git"), clone_root)?.ok_or_else(|| {
            return String::from("the slot holds no .git, so the exchange has nothing to swap");
        })?;
    let pointer_file: EntryFacts =
        read_the_entry(&trash, OsStr::new(prepared.entry.as_str()), &trash_path)?.ok_or_else(
            || {
                return format!(
                    "the trash entry {} does not stand, so nothing was exchanged",
                    prepared.entry
                );
            },
        )?;
    if copied.is_file
        || copied.identity != prepared.copied
        || !pointer_file.is_file
        || pointer_file.identity != prepared.pointer_file
    {
        return Err(String::from(
            "the slot's .git or the trash entry is not what the preparation found, so nothing was exchanged",
        ));
    }
    exchange_between(
        &root,
        OsStr::new(".git"),
        &trash,
        OsStr::new(prepared.entry.as_str()),
    )
    .map_err(|error| {
        return format!(
            "the exchange of .git with the trash entry {} was refused ({error})",
            prepared.entry
        );
    })?;
    let after_git: EntryFacts = read_the_entry(&root, OsStr::new(".git"), clone_root)?
        .ok_or_else(|| return String::from("the slot holds no .git after the exchange"))?;
    let after_entry: EntryFacts =
        read_the_entry(&trash, OsStr::new(prepared.entry.as_str()), &trash_path)?.ok_or_else(
            || {
                return format!(
                    "the trash entry {} does not stand after the exchange",
                    prepared.entry
                );
            },
        )?;
    if !after_git.is_file
        || after_git.identity != prepared.pointer_file
        || after_entry.is_file
        || after_entry.identity != prepared.copied
    {
        return Err(format!(
            "the exchange left .git as {} and {} as {}, so neither name holds the object the preparation proved",
            if after_git.is_file {
                "a file"
            } else {
                "a folder"
            },
            prepared.entry,
            if after_entry.is_file {
                "a file"
            } else {
                "a folder"
            }
        ));
    }
    root.sync_all().map_err(|error| {
        return format!("the clone root cannot be flushed after the exchange ({error})");
    })?;
    trash.sync_all().map_err(|error| {
        return format!("the trash cannot be flushed after the exchange ({error})");
    })?;
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_exchange_tests.rs"]
mod tests;
