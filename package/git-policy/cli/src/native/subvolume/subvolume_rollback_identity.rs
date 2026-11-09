//! What:
//!  Which directory a name holds,
//! by identity,
//! for the two steps that write into and exchange directories:
//! the checkout
//!  `--remove-old` carries into and `--rollback` exchanges,
//! and the pair the rollback swaps.
//! Why:
//!  A name is resolved again by every call,
//! so a check that a checkout is the directory the journal records is worth nothing unless
//!  the step after it is about the same object.
//! This file holds the reader,
//! the pinned descriptor the carry goes through,
//! and
//!  the pair check the rollback takes before and after its exchange,
//! so the steps read as steps and the identity rule
//!  is written once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pinned = await pinRoot(run, layout);                                     // the checkout, held open
//! const refusal = pairRefusal({ root, oldPath, atRoot, atOld, installed, what }); // undefined when the pair is right
//! ```

use crate::fs_beneath::{ObjectId, identity_at, identity_of, open_directory_at, open_root};
use crate::subvolume_error::MigrateError;
use crate::subvolume_preflight::{INSTALLED_ROOT, Refusal};
use crate::subvolume_run::Run;
use crate::worktree_convert_phases::identity_of_path;
use std::path::Path;

/// The identity of the object at `path` in the journal's form,
/// or nothing when the name leads nowhere.
pub(crate) fn object_form(path: &Path) -> Option<String> {
    return identity_of_path(path).map(as_recorded);
}

/// One identity in the journal's form,
/// for `Option::map`.
pub(crate) fn as_recorded(identity: ObjectId) -> String {
    return recorded_form(&identity);
}

/// What:
///  The refusal for a pair of directories that is not the pair the rollback expects,
/// or nothing when it is.
/// Why:
///   The exchange and the deletion that follows it take names,
/// not descriptors,
/// so what the names hold is read
///  immediately before and immediately after the exchange:
/// the checkout must be the installed directory (a record the
///  migration writes,
/// compared only when it is there)
/// and the two names must hold two different directories,
/// and after the
///  exchange each must hold what the other held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = pairRefusal({ root, oldPath, atRoot, atOld, installed, what }); // undefined when the pair is right
/// ```
pub(crate) fn pair_refusal(pair: PairLook<'_>) -> Option<Refusal> {
    if let Some(installed) = pair.installed
        && pair.what == "before"
        && pair.at_root.as_deref() != Some(installed)
    {
        return Some(Refusal {
            cause: format!("{} holds {}, and the journal records the installed checkout as {installed}", pair.root.display(), pair.at_root.as_deref().unwrap_or("a name that leads nowhere")),
            remedy: String::from("If the checkout was moved or replaced, move the directory the migration installed back to that path and run the command again. Nothing was changed."),
        });
    }
    if pair.at_root.is_none() || pair.at_old.is_none() || pair.at_root == pair.at_old {
        return Some(Refusal {
            cause: format!(
                "{} and {} are not two directories that are different from each other (they hold {} and {})",
                pair.root.display(),
                pair.old_path.display(),
                pair.at_root.as_deref().unwrap_or("a name that leads nowhere"),
                pair.at_old.as_deref().unwrap_or("a name that leads nowhere")
            ),
            remedy: String::from("Check both paths and put the directories the migration made back at their names, then run the command again. Nothing was changed."),
        });
    }
    return None;
}

/// What the two names held when they were read,
/// and what the journal says about one of them.
pub(crate) struct PairLook<'a> {
    /// The checkout's path.
    pub(crate) root: &'a Path,
    /// The original tree's path.
    pub(crate) old_path: &'a Path,
    /// The identity at the checkout's path.
    pub(crate) at_root: Option<String>,
    /// The identity at the original tree's path.
    pub(crate) at_old: Option<String>,
    /// The identity of the installed checkout the journal records,
    /// when it records one.
    pub(crate) installed: Option<&'a str>,
    /// Which look this is,
    /// `before` or `after` the exchange.
    pub(crate) what: &'a str,
}

/// The journal's form of an identity:
/// device and inode,
/// without the birth time the descriptor's identity also carries.
pub(crate) fn recorded_form(identity: &ObjectId) -> String {
    return format!("{}:{}", identity.device, identity.inode);
}

/// What:
///  The checkout the carry writes into,
/// held open,
/// with the identity it had when it was opened.
/// Why:
///   A path is resolved again by every call inside the carry,
/// so a checkout moved aside and another put at the same
///  path between the check and the carry would receive the old tree's newer files.
/// The directory is opened below its parent without
///  following a link,
/// its identity is read from that descriptor,
/// and every write of the carry goes through the descriptor,
/// so
///  the files land in the directory that was checked whatever happens to its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PinnedRoot = { parent: FileHandle; name: string; directory: FileHandle; identity: ObjectId };
/// ```
pub(crate) struct PinnedRoot {
    /// The directory that holds the checkout.
    pub(crate) parent: std::fs::File,
    /// The checkout's name in `parent`.
    pub(crate) name: std::ffi::OsString,
    /// The checkout itself,
    /// held open.
    pub(crate) directory: std::fs::File,
    /// The identity of the held directory.
    pub(crate) identity: ObjectId,
}

/// What:
///  Open the checkout and prove it is the directory the migration installed,
/// or say why it is not.
///  `Ok(Err(refusal))` means nothing was written and nothing was deleted.
/// Why:
///   See `PinnedRoot`.
/// The recorded identity is compared on the descriptor,
/// so the check and the use are the same
///  object,
/// and a journal of an earlier release that records no identity still gets a pinned directory,
/// its
///  identity being the one the check after the carry compares against.
/// A checkout moved aside after this moment receives the files at
///  the directory it became,
/// which is the directory the migration installed,
/// and the old tree is kept because its name no
///  longer leads there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pinned = await pinRoot(run, layout);
/// ```
pub(crate) fn pin_root(run: &Run<'_>, after_pinned: &mut dyn FnMut()) -> Result<Result<PinnedRoot, Refusal>, MigrateError> {
    let root: &Path = run.layout.root.as_path();
    let Some(parent_path) = root.parent() else {
        return Ok(Err(Refusal { cause: format!("{} has no parent directory", root.display()), remedy: String::from("Run the command for a checkout that is a directory.") }));
    };
    let name: std::ffi::OsString = root.file_name().unwrap_or_default().to_os_string();
    let parent: std::fs::File = MigrateError::from_io_result(open_root(parent_path), "hold the directory of the checkout", parent_path)?;
    let directory: std::fs::File = MigrateError::from_io_result(open_directory_at(&parent, name.as_os_str()), "hold the checkout", root)?;
    let identity: ObjectId = MigrateError::from_io_result(identity_of(&directory), "examine", root)?;
    let now: String = recorded_form(&identity);
    if let Some(recorded) = run.journal.value(INSTALLED_ROOT)
        && now != recorded
    {
        return Ok(Err(Refusal {
            cause: format!("{} is not the directory installed by the migration there: it is {now} and the journal records {recorded}", root.display()),
            remedy: String::from("If the checkout was moved or replaced, move the directory the migration installed back to that path and run the command again. Nothing was changed."),
        }));
    }
    after_pinned();
    return Ok(Ok(PinnedRoot { parent, name, directory, identity }));
}

/// Whether the name of the pinned checkout still leads to it;
/// a refusal naming both identities when it does not.
pub(crate) fn pinned_root_moved(run: &Run<'_>, pinned: &PinnedRoot) -> Option<Refusal> {
    let root: &Path = run.layout.root.as_path();
    let held: String = recorded_form(&pinned.identity);
    let now: String = match identity_at(&pinned.parent, pinned.name.as_os_str()) {
        Ok(found) => recorded_form(&found),
        Err(_gone) => String::from("a name that leads nowhere"),
    };
    if now == held {
        return None;
    }
    return Some(Refusal {
        cause: format!("{} was the directory {held} while the old tree's newer files were carried into it, and it is {now} now", root.display()),
        remedy: String::from("Nothing was deleted. Check what is at that path, move the directory the migration installed back, and run the command again."),
    });
}
