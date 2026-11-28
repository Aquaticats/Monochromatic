//! What:
//!  The staging folders a removal leaves under the account's cache,
//!  each with its owner,
//!  its mode,
//!  whether its name is one a removal derives and whether it is empty.
//! Why:
//!  A removal moves a clone to a staging folder under `~/.cache` and unmounts
//!  it from there,
//!  so a folder another account could write,
//!  one holding content,
//!  or one whose name no removal derives is a state a person must see
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staging target and who creates it";
//!  plan step 18,
//!  "the staging folder and its live bind's propagation").
//!  The account is trusted with its own cache by the accepted threat model,
//!  so these are reports rather than refusals,
//!  and the folder's own descriptor is not held across the listing:
//!  a folder an account makes while the listing runs costs the report one
//!  entry and never a mount.
//!  A cache that could not be listed leaves the check unable to say a staging
//!  folder is absent,
//!  so the cache's own look carries that answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const staging = theStagingFacts({ places, account }); // { cache, account, folders: [{ name, state, derived, empty }] }
//! ```

/// The look at one path.
use super::one_look_at_a_path::{
    the_path_facts,
    the_path_state,
};
/// The places the account's cache is named by.
use super::the_route_places::TheRoutePlaces;
/// One path and what the look at it answered.
use crate::doctor_zfs_route::handler_checks::{
    PathFacts,
    PathState,
};
/// One cache and the staging-shaped names under it.
use crate::doctor_zfs_route::staging_folder_checks::{
    StagedFolder,
    StagingFolderFacts,
    is_derived_folder_name,
};
/// The prefix every staging folder's name begins with.
use crate::worktree_clone_unmount::staging_folder_name::STAGING_FOLDER_PREFIX;

/// The folder a cache is reported under when the environment names no home,
/// which is the one name the design would have used.
const THE_CACHE_WITHOUT_A_HOME: &str = ".cache";

/// What:
///  The account's cache folder and every staging-shaped name under it.
/// Why:
///  One value carries the account,
///  the cache and the folders,
///  so a report cannot compare one account's folder with another's record,
///  and a machine whose environment names no home is told so rather than read
///  as if its cache were the file system's root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = theStagingFacts({ places, account });
/// ```
pub(crate) fn the_staging_facts(
    places: &TheRoutePlaces,
    account: u32,
) -> StagingFolderFacts {
    let cache: std::path::PathBuf = match places.the_cache_or_why() {
        Ok(held) => held,
        Err(why) => {
            // What:     `Unreadable` is the state a check reports as unknown with its reason, which is what a cache
            //           nobody named is.
            // Why:      A pass would say no staging folder stands under `.cache`, which nobody looked at.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { cache: { path: '.cache', state: { kind: 'unreadable', why } }, account, folders: [] };
            // ```
            return StagingFolderFacts {
                cache: PathFacts {
                    path: std::path::PathBuf::from(THE_CACHE_WITHOUT_A_HOME),
                    state: PathState::Unreadable {
                        why,
                    },
                },
                account,
                folders: Vec::new(),
            };
        }
    };
    let looked: PathFacts = the_path_facts(cache.as_path());
    let PathState::Directory {
        ..
    } = looked.state
    else {
        return StagingFolderFacts {
            cache: looked,
            account,
            folders: Vec::new(),
        };
    };
    let folders: Vec<StagedFolder> = match the_staging_names(cache.as_path()) {
        Ok(names) => names
            .into_iter()
            .map(|name| return the_staged_folder(cache.as_path(), name))
            .collect(),
        Err(why) => {
            return StagingFolderFacts {
                cache: PathFacts {
                    path: cache,
                    state: PathState::Unreadable {
                        why,
                    },
                },
                account,
                folders: Vec::new(),
            };
        }
    };
    return StagingFolderFacts {
        cache: looked,
        account,
        folders,
    };
}

/// What:
///  Every name under the cache that begins with the staging prefix.
/// Why:
///  A folder whose name carries no prefix is the account's own and none of the
///  route's business,
///  and the prefix is the one word every writer of a staging folder uses.
///  A cache that is not there holds none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = theStagingNames('/home/someone/.cache'); // ['monochromatic-git-policy-cli-staging-1234']
/// ```
fn the_staging_names(cache: &std::path::Path) -> Result<Vec<String>, String> {
    let entries: std::fs::ReadDir = match std::fs::read_dir(cache) {
        Ok(listed) => listed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "the cache folder in {} could not be listed ({error}), so no staging folder is proved absent",
                cache.display()
            ));
        }
    };
    let mut names: Vec<String> = Vec::new();
    for listed in entries {
        let one_entry: std::fs::DirEntry = listed.map_err(|error| {
            return format!(
                "the cache folder in {} could not be listed ({error}), so no staging folder is proved absent",
                cache.display()
            );
        })?;
        let name: String = one_entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(STAGING_FOLDER_PREFIX) {
            names.push(name);
        }
    }
    return Ok(names);
}

/// What:
///  One staging-shaped name with the four facts the check judges.
/// Why:
///  The name's derivation,
///  its look,
///  and whether it holds an entry are three separate readings,
///  and a folder that is gone between the listing and the look keeps a name
///  and answers absent rather than ending the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = theStagedFolder('/home/someone/.cache', 'monochromatic-git-policy-cli-staging-1234');
/// ```
fn the_staged_folder(
    cache: &std::path::Path,
    name: String,
) -> StagedFolder {
    let path: std::path::PathBuf = cache.join(name.as_str());
    return StagedFolder {
        derived: is_derived_folder_name(name.as_str()),
        state: the_path_state(path.as_path()),
        empty: holds_no_entry(path.as_path()),
        name,
    };
}

/// What:
///  Whether a staging folder holds no entry,
///  as the folder's own listing answers.
/// Why:
///  The design calls a folder holding content the account's substitution,
///  which only a report can surface,
///  and a folder a look could not list is not an empty one,
///  so the answer is false where the listing refused and the folder's own look
///  says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const empty = holdsNoEntry('/home/someone/.cache/monochromatic-git-policy-cli-staging-1234');
/// ```
fn holds_no_entry(path: &std::path::Path) -> bool {
    let listed: std::fs::ReadDir = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_error) => return false,
    };
    return listed.into_iter().next().is_none();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_staging_folders_read_tests.rs"]
mod tests;
