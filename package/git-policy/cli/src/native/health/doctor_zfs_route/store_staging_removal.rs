//! What:
//!  The one removal plan step 18 performs:
//!  the two known staging forms a killed writer left in the slot identity
//!  store.
//! Why:
//!  The project put this removal here while the store's sweep stays unlisted,
//!  so the adoption is not the only remover
//!  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
//!  "Who removes a killed writer's staging file";
//!  `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  choice 9).
//!  A staging name is never read as a record,
//!  so nothing else depends on it and no reader is affected by one standing;
//!  what makes the removal safe is that it removes only the two forms this
//!  layout's writers make and leaves every other name,
//!  including a dot name nobody accounted for,
//!  because that one may be a stranger's file.
//!  This function is reached from `doctor` alone:
//!  no command path,
//! handler mode or boot unit calls it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removal = remove_known_staging_forms({ store }); // { removed: ['.a1b2...', '..parent.3f.partial'] }
//! ```

use crate::worktree_clone_mount::{
    SlotIdentityStore,
    StoreRefusal,
};

/// What:
///  Why the store's staging names were not removed.
/// Why:
///  The removal has to say whether nothing was there or nothing could be
///  removed,
///  and the store's own refusal already names the file and the cause,
///  so its words travel rather than a second vocabulary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingRemovalRefusal = { why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagingRemovalRefusal {
    /// What the removal answered.
    pub why: String,
}

/// What:
///  What one removal took out of the store.
/// Why:
///  The names are the report:
///  a person learns which leftovers were there and that none survives,
///  and a control compares the list with the names it planted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingRemoval = { removed: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagingRemoval {
    /// The names removed,
    /// in the order the folder listed them.
    pub removed: Vec<String>,
}

/// What:
///  The store the removal acts on.
/// Why:
///  One value names the store so the function has one parameter,
///  as the repository's rules ask,
///  and the caller cannot hand in a folder and a path that disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingRemovalRequest = { store: SlotIdentityStore };
/// ```
pub struct StagingRemovalRequest<'a> {
    /// The store whose folder is listed once and cleared of the two forms.
    pub store: &'a SlotIdentityStore,
}

/// What:
///  Remove the two known staging forms a killed writer left,
///  and answer their names.
/// Why:
///  A person's report says what was there before this removed it,
///  so the removal runs after the count and age are read,
///  and a store that cannot be listed or whose entries cannot be removed
///  answers a refusal rather than a silent partial success.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = remove_known_staging_forms({ store }); // throws a refusal when the folder cannot be cleared
/// ```
///
/// # Errors
/// Everything the store's listing and removal refuse,
/// with the store's own words.
pub fn remove_known_staging_forms(
    request: StagingRemovalRequest<'_>
) -> Result<StagingRemoval, StagingRemovalRefusal> {
    return match request.store.remove_staging_entries() {
        Ok(removed) => Ok(StagingRemoval {
            removed,
        }),
        Err(refusal) => Err(StagingRemovalRefusal {
            why: refusal_words(&refusal),
        }),
    };
}

/// The store's refusal in words a person can act on.
fn refusal_words(refusal: &StoreRefusal) -> String {
    return refusal.message();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "store_staging_removal_tests.rs"]
mod tests;
