//! What:
//!  The pointer's exchange with the copied Git administration:
//!  the one call that swaps the two entries,
//! and the check that each name holds
//!  what the preparation proved.
//! Why:
//!  The slot's path must hold a `.git` at every moment:
//! the copied directory
//!  until the exchange and the saved pointer after it
//!  (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
//!  "What the slot holds while the add fills it").
//!  One `renameat2(RENAME_EXCHANGE)` changes the two entries in one instant,
//! so
//!  no kill leaves the folder without one
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore",
//! step 8).
//!  What the swap is given is `add_exchange_preparation.rs`'s,
//! and the trash it
//!  swaps with is `add_exchange_trash.rs`'s,
//!  so this file is the swap and its own check.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! exchangeTheGitEntry(cloneRoot, prepared); // .git <-> trash/dN in one call
//! ```

use super::add_exchange_preparation::{
    EntryFacts,
    read_the_entry,
};
use crate::fs_beneath::{
    open_directory_at,
    open_root,
};
use crate::worktree_trash::TRASH_PARENT;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The trash entry the slot's `.git` is swapped with,
/// and what the two sides
///  held before it.
/// Why:
///  The swap reads one value that names both entries and both identities,
///  so a caller prepares once and passes the answer on.
pub use super::add_exchange_preparation::PreparedExchange;

/// Prepare the trash entry and its manifest,
/// or find that an earlier run did.
pub use super::add_exchange_preparation::prepare_the_exchange;

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

/// What:
///  Exchange the pointer file in the trash with the slot's copied `.git`.
/// Why:
///  One call changes both names in one instant,
/// so the slot holds one of them
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
