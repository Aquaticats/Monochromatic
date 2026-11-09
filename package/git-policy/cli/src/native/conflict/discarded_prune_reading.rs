//! What:
//!  The folder of saved conflict sides as a command that only reports finds it:
//!  under the data folder that stands in the common Git directory,
//!  whatever spelling an earlier release gave it.
//! Why:
//!  `git monochromatic-git-policy-cli doctor` reports that folder and must change nothing,
//! so it cannot settle an earlier
//!  release's names to their current spelling first
//! (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
//! finding 6).
//!  The walk from the common directory keeps the reading of an absent folder,
//! a link and a refusal the same as the
//!  writer's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listing = await listStampedForReading(commonDir); const relative = savedRelativeForReading(commonDir);
//! ```

use crate::conflict_sides::SAVED_DIRECTORY;
use crate::discarded_prune::{Listing, list_in, unusable};
use crate::discarded_store::Store;
use std::path::{Path, PathBuf};

/// What:
///  The saved copies a reader finds,
/// under the current data folder or,
/// when only that exists,
/// under the earlier spelling an earlier release wrote.
/// Why:
///   A command that only reports must not move an earlier release's folder to its current name,
/// so it reads whichever
///  spelling stands there
/// (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 6).
/// The program that owns the
///  directory settles the names;
/// the doctor only looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = dataFolderForReading(commonDir); const store = await Store.open(dirname(root), 'discarded');
/// ```
pub fn list_stamped_for_reading(common_dir: &Path) -> Listing {
    let relative: String = saved_relative_for_reading(common_dir);
    let saved_root: PathBuf = common_dir.join(relative.as_str());
    return match Store::open(common_dir, relative.as_str()) {
        Ok(Some(store)) => list_in(&store, saved_root.as_path()),
        Ok(None) => Listing::default(),
        Err(failure) => Listing { problems: vec![unusable(saved_root.as_path(), &failure)], ..Listing::default() },
    };
}

/// The text of a file name,
/// for a folder this program names itself.
fn text_of_name(found: &std::ffi::OsStr) -> String {
    return found.to_string_lossy().into_owned();
}

/// The current spelling of the data folder,
/// used when the resolved path has no file name at all.
fn the_current_data_folder_name() -> String {
    return String::from(crate::wrapper_names_old_and_new::DATA_FOLDER.current);
}

/// What:
///  The folder of saved copies below a common Git directory,
/// as the spelling of the data folder that stands there names it,
/// relative to the
///  common directory.
/// Why:
///   The walk from the common directory keeps the reading of an absent folder,
/// a link and a refusal the same,
///  whichever spelling is there;
/// naming the folder in the report needs the same answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function savedRelativeForReading(commonDir: string): string; // `${dataFolderForReading(commonDir)}/discarded`
/// ```
pub fn saved_relative_for_reading(common_dir: &Path) -> String {
    let folder: PathBuf = crate::earlier_name_moved_to_current_in_place::existing_kept_name(
        common_dir,
        crate::wrapper_names_old_and_new::DATA_FOLDER,
    );
    let name: String = folder.file_name().map(text_of_name).unwrap_or_else(the_current_data_folder_name);
    let Some(below) = SAVED_DIRECTORY.split('/').next_back() else {
        return String::from(SAVED_DIRECTORY);
    };
    return format!("{name}/{below}");
}
