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
use std::path::{Path, PathBuf};

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
///  The refusal for a resumed rollback whose old tree's name holds something other than the installed checkout,
///  or nothing when
///  it holds that one,
/// when the rename already happened,
/// or when the journal records no installed identity.
/// Why:
///   The resumed path renames that object to the new subvolume's name and deletes it,
/// and a kill between the exchange and
///  the rename is enough to put the command in that state:
/// no second process has to run,
/// so the object is checked before
///  anything is renamed.
/// A name with nothing at it is not a refusal:
/// there is nothing to rename or delete,
/// and the run says so
///  by doing nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = resumedOldTreeRefusal(run, oldPath, newPath); // undefined when the object is the recorded one
/// ```
pub(crate) fn resumed_old_tree_refusal(run: &Run<'_>, old_path: &Path, new_path: &Path) -> Option<Refusal> {
    let installed: &str = run.journal.value(INSTALLED_ROOT)?;
    // The rename already happened, so the directory the resumed run deletes is the one at the new name.
    if object_form(new_path).is_some() {
        let found: String = object_form(new_path)?;
        if found == installed {
            return None;
        }
        return Some(Refusal {
            cause: format!("{} holds {found}, and the journal records the installed checkout as {installed}", new_path.display()),
            remedy: String::from("If the installed checkout was moved or replaced, move it back to that path and run the command again. Nothing was renamed or deleted."),
        });
    }
    let found: String = object_form(old_path)?;
    if found == installed {
        return None;
    }
    return Some(Refusal {
        cause: format!("{} holds {found}, and the journal records the installed checkout as {installed}", old_path.display()),
        remedy: String::from("If the checkout was moved or replaced, move the installed checkout back to that path and run the command again. Nothing was renamed or deleted."),
    });
}

/// What:
///  The refusal for a deletion whose object is not the checkout the migration installed,
///  or nothing when it is,
///  when the name leads nowhere,
///  or when the journal records no installed identity.
/// Why:
///   The rollback deletes the subvolume the migration made,
/// and a name holds whatever a later process put there.
/// The recorded exchange makes a resumed run skip the swap,
/// so the pair check that guards the exchange does not run on that path,
/// and the deletion would judge by the name alone;
/// the identity is therefore compared on every path that reaches it.
/// A journal written by an earlier release records no identity,
/// and without a record the command cannot tell,
/// so it does not refuse on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = installedIdentityRefusal(run, newPath); // undefined when the object is the recorded one
/// ```
pub(crate) fn installed_identity_refusal(run: &Run<'_>, path: &Path) -> Option<Refusal> {
    let installed: &str = run.journal.value(INSTALLED_ROOT)?;
    let found: String = object_form(path)?;
    if found == installed {
        return None;
    }
    return Some(Refusal {
        cause: format!("{} holds {found}, and the journal records the installed checkout as {installed}", path.display()),
        remedy: String::from("If the installed checkout was moved or replaced, move it back to that path and run the command again. Nothing was deleted."),
    });
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

/// What:
///  The cause of the refusal for the entries the carry did not put into the checkout,
/// naming each kind and the paths the list
///  holds.
/// Why:
///   Two kinds exist and they are kept for different reasons:
/// an entry absent from the checkout that predates the catch-up
///  may be a file deleted there on purpose,
/// and one whose path holds another kind of entry is there because the checkout's
///  entry may be newer.
/// Both leave the old tree's copy as the only one at that path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cause = uncarriedCause(run);
/// ```
pub(crate) fn uncarried_cause(run: &Run<'_>) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !run.left_out.is_empty() {
        parts.push(format!(
            "{} entr(ies) absent from {} and older than the catch-up ({})",
            run.left_out.len(),
            run.layout.root.display(),
            shown_left_out(run.left_out.iter())
        ));
    }
    if !run.conflicts.is_empty() {
        parts.push(format!(
            "{} entr(ies) whose path in {} holds another kind of entry, kept as the newer one ({})",
            run.conflicts.len(),
            run.layout.root.display(),
            shown_left_out(run.conflicts.iter())
        ));
    }
    return format!("{} did not reach {}: {} (the whole list is in {})", run.layout.old_path.display(), run.layout.root.display(), parts.join("; "), run.layout.left_out_path.display());
}

/// The remedy of that refusal:
/// copy what is wanted,
/// then accept the loss with the flag,
/// or delete the old tree by hand.
pub(crate) fn uncarried_remedy() -> String {
    return String::from(
        "Copy anything you want to keep out of the old tree first. If what it keeps is there on purpose, pass --discard-left-out together with --remove-old to delete the old tree with it; you can also delete the old tree yourself.",
    );
}

/// How many left-out entries a refusal spells out before it names the file that lists them all.
const SHOWN_LEFT_OUT: usize = 10;

/// The first few left-out entries,
/// joined for a refusal,
/// and how many more there are.
pub(crate) fn shown_left_out<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> String {
    let mut shown: Vec<String> = Vec::new();
    let mut more: usize = 0;
    for path in paths {
        if shown.len() < SHOWN_LEFT_OUT {
            shown.push(path.display().to_string());
        } else {
            more += 1;
        }
    }
    if more > 0 {
        shown.push(format!("and {more} more"));
    }
    return shown.join(", ");
}
