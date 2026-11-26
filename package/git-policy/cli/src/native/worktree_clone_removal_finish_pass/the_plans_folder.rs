//! What:
//!  The one look the finish pass takes at the destruction plan folder:
//!  one `stat` while the folder is absent,
//!  and one listing when it stands,
//!  with each entry's name in the order a person would read them.
//! Why:
//!  Plans stand in one folder of the wrapper's own home,
//!  and the pass runs before every forwarded command,
//!  so the healthy state costs one `stat` and no listing
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass that settles a registration").
//!  The look is a seam,
//!  so a control counts the one call the healthy state makes
//!  and answers with a folder it planted without a repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const look = folder.look(plansFolder(commonDir)); // absent | not-a-folder | names
//! ```

use std::ffi::OsString;
use std::path::Path;

/// What:
///  What one look at the plan folder answered.
/// Why:
///  The absent folder is the healthy state and is not an error,
///  and a folder that is not one is reported rather than read as empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderLook = { kind: 'no-folder' } | { kind: 'not-a-folder'; said } | { kind: 'entries'; names };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FolderLook {
    /// Nothing stands at the folder's path.
    NoFolder,
    /// Something else stands there,
    /// or the look could not list it.
    NotAFolder {
        /// What the call answered,
        /// or that the entry is not a folder.
        said: String,
    },
    /// The folder stands,
    /// holding these entries,
    /// each name spelled as the file system gives it.
    Entries(Vec<OsString>),
}

/// What:
///  The look the finish pass takes,
///  as a value a control answers itself.
/// Why:
///  The healthy cost is one `stat` and no listing,
///  and a control that counts the calls is what proves the pass asks once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ThePlansFolder { look(folder: Path): FolderLook }
/// ```
pub trait ThePlansFolder {
    /// The one look at `folder`.
    fn look(
        &self,
        folder: &Path,
    ) -> FolderLook;
}

/// What:
///  The release answer:
///  one `stat`,
///  then one listing only when a folder stands.
/// Why:
///  A repository that never removed a ZFS clone has no folder,
///  and a listing of a folder that is not there is the cost this rule removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// TheLivePlansFolder.look(folder);
/// ```
pub struct TheLivePlansFolder;

impl ThePlansFolder for TheLivePlansFolder {
    /// The folder's entries,
    /// or what stands in its place.
    fn look(
        &self,
        folder: &Path,
    ) -> FolderLook {
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(folder) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return FolderLook::NoFolder;
            }
            Err(error) => {
                return FolderLook::NotAFolder {
                    said: error.to_string(),
                };
            }
        };
        if !metadata.is_dir() {
            return FolderLook::NotAFolder {
                said: String::from("the entry at the plan folder's path is not a folder"),
            };
        }
        let listing: std::fs::ReadDir = match std::fs::read_dir(folder) {
            Ok(opened) => opened,
            Err(error) => {
                return FolderLook::NotAFolder {
                    said: error.to_string(),
                };
            }
        };
        let mut names: Vec<OsString> = Vec::new();
        for listed in listing {
            let opened: std::fs::DirEntry = match listed {
                Ok(found) => found,
                Err(error) => {
                    return FolderLook::NotAFolder {
                        said: error.to_string(),
                    };
                }
            };
            names.push(opened.file_name());
        }
        // The order a listing returns is the file system's, so the pass reads one order on every machine.
        names.sort();
        return FolderLook::Entries(names);
    }
}

/// What:
///  Whether an empty plan folder went.
/// Why:
///  A kill between a plan's unlink and the folder's removal leaves an empty folder,
///  and the next command removes it rather than listing it for the clone's whole life.
///  An unlink that finds entries is the file system's own refusal,
///  so a folder still holding a record is never removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = removeTheFolderWhenEmpty(folder);
/// ```
pub fn remove_the_folder_when_empty(folder: &Path) -> bool {
    return std::fs::remove_dir(folder).is_ok();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_plans_folder_tests.rs"]
mod tests;
