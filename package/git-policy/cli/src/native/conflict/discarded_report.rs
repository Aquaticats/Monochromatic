//! What:
//!  What the folder of saved conflict copies holds (`Report`) and the text the doctor prints for it,
//!  for a reader that moves nothing.
//! Why:
//!  The pruning pass owns the counting and the removal (`discarded_prune.rs`);
//! the report only reads,
//!  and it reads the folder that exists under whichever of its two names it has,
//! so a repository that still holds
//!  the earlier name is reported instead of being migrated by a report.
//!  The limits that apply are the ones the settings file gives,
//! and each one is named in the text beside
//!  what it does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = report(commonDir);        // undefined when the folder holds no copy and nothing went wrong
//! stderr.write(doctorSection(commonDir, limits)); // '' when there is nothing to say
//! ```

/// The folder of saved copies inside a Git directory.
use crate::conflict_sides::SAVED_DIRECTORY;
/// The listing of that folder and its limits.
use crate::discarded_prune::{Limits, list_stamped_below};
/// The listing's shape and what could not be read,
/// which are the pruning pass's.
use crate::discarded_prune_problems::{Listing, Problem};
/// The relative spelling a reader uses for the folder that exists.
use crate::kept_names_in_the_git_directories::relative_path_for_reading;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Duration` is how long the survey of the folders may take.
use std::time::Duration;
/// The ceiling of a file system read of the report.
use crate::wait_ceilings::REPORT_SCAN_CEILING;
/// The job of such a read,
/// run on a thread of its own.
use crate::work_awaited_within_a_ceiling::work_within;

/// What the folder holds,
/// for `git monochromatic-git-policy-cli doctor`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    /// Bytes of every copy.
    pub bytes: u128,
    /// How many files the copies are.
    pub copies: usize,
    /// How many saves (directories) they came from.
    pub saves: usize,
    /// The name of the oldest save,
    /// empty when no save could be read.
    pub oldest: String,
    /// What could not be read.
    pub problems: Vec<Problem>,
}

/// The folder's size,
/// copy count and oldest save and what could not be read,
/// or nothing when it holds no copy and nothing
/// went wrong.
pub fn report(common_dir: &Path) -> Option<Report> {
    return report_below(common_dir, relative_path_for_reading(common_dir, SAVED_DIRECTORY).as_str());
}

/// What:
///  `report` for the folder at `relative` below the Git directory.
/// Why:
///  A report reads the folder that exists,
/// under whichever of its two names it has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reportBelow(commonDir, relative);
/// ```
fn report_below(common_dir: &Path, relative: &str) -> Option<Report> {
    let listing: Listing = list_stamped_below(common_dir, relative);
    if listing.entries.is_empty() && listing.problems.is_empty() {
        return None;
    }
    let mut total_bytes: u128 = 0;
    for entry in &listing.entries {
        total_bytes += entry.bytes;
    }
    return Some(Report {
        bytes: total_bytes,
        copies: listing.files,
        saves: listing.entries.len(),
        oldest: match listing.entries.first() {
            Some(entry) => entry.name.clone(),
            None => String::new(),
        },
        problems: listing.problems,
    });
}

/// What:
///  One folder the report read,
/// as its relative path below the Git directory,
/// with what it holds.
///  `Vec<(String, Report)>` is every folder the report reads,
/// in the order it names them.
/// Why:
///   The text names each folder it describes,
/// and a repository can hold the saved copies under more than one folder name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SavedCopies = Array<[string, Report]>;
/// ```
pub type SavedCopies = Vec<(String, Report)>;

/// What:
///  What one report reads,
/// as a function pointer,
/// so a control can stand a survey that never answers where a mount that
///  stopped answering stands.
/// Why:
///   The survey opens and lists directories,
/// which waits in the kernel on a mount whose server has stopped;
/// this is the
///  read the ceiling is placed on,
/// and it is a value the report is given rather than a call it makes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SavedCopiesRead = (commonDir: string) => SavedCopies;
/// ```
pub type SavedCopiesRead = fn(&Path) -> SavedCopies;

/// What:
///  The folder of saved copies that exists,
/// with what it holds.
/// Why:
///   The report changes nothing,
/// so it reads the folder under the name it has now,
/// else under the earlier one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folderThatExists = (commonDir) => [[relativePathForReading(commonDir, SAVED_DIRECTORY), report]];
/// ```
fn the_folder_that_exists(common_dir: &Path) -> SavedCopies {
    let relative: String = relative_path_for_reading(common_dir, SAVED_DIRECTORY);
    return match report_below(common_dir, relative.as_str()) {
        Some(found) => vec![(relative, found)],
        None => Vec::new(),
    };
}

/// What:
///  The doctor's text for the saved copies of `common_dir`:
/// empty when no folder holds a copy and nothing went wrong,
/// otherwise
///  one block per folder with the size,
/// the count,
/// the oldest save and the limits that apply,
/// then a sentence for each thing
///  that could not be read.
/// Why:
///   The survey runs on a thread of its own and is awaited no longer than `REPORT_SCAN_CEILING`,
/// because
///  "nothing should ever wait forever or indefinitely":
/// a folder that gives no answer is said in one line rather than
///  holding the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = doctorSection(commonDir, limits);
/// ```
pub fn doctor_section(common_dir: &Path, limits: Limits) -> String {
    return doctor_section_with(the_folder_that_exists, REPORT_SCAN_CEILING, common_dir, limits);
}

/// What:
///  `doctor_section` with the survey and the ceiling given.
/// Why:
///   The survey and the ceiling are the two things a control varies;
/// `REPORT_SCAN_CEILING` says why the production value is
///  what it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// doctorSectionWith(theFolderThatExists, REPORT_SCAN_CEILING, commonDir, limits);
/// ```
pub fn doctor_section_with(read: SavedCopiesRead, ceiling: Duration, common_dir: &Path, limits: Limits) -> String {
    let named: PathBuf = common_dir.to_path_buf();
    let surveyed: Option<SavedCopies> = work_within(ceiling, move || { return read(named.as_path()); });
    let Some(folders) = surveyed else {
        return format!(
            "git monochromatic-git-policy-cli doctor: discarded copies: the saved copies of {} could not be read within {ceiling:?}\n",
            common_dir.display()
        );
    };
    let mut text: String = String::new();
    for (relative, found) in &folders {
        if found.saves > 0 {
            text.push_str(
                format!(
                    "git monochromatic-git-policy-cli doctor: discarded copies: {} holds {} bytes in {} copies from {} saves; the oldest save is {}. Copies older than {} days \
                     are removed, and the oldest are removed past {} bytes, never one younger than a day (`discardedCopies.maxAgeDays` and \
                     `discardedCopies.maxSize` in monochromatic-git-policy-cli.config.jsonc change the limits).\n",
                    common_dir.join(relative.as_str()).display(),
                    found.bytes,
                    found.copies,
                    found.saves,
                    found.oldest,
                    limits.max_age_days,
                    limits.max_bytes,
                )
                .as_str(),
            );
        }
        for problem in &found.problems {
            text.push_str(problem.message().replacen("monochromatic-git-policy-cli: ", "git monochromatic-git-policy-cli doctor: discarded copies: ", 1).as_str());
        }
    }
    return text;
}

/// The report's own controls are in the test module.
#[cfg(test)]
#[path = "discarded_report_tests.rs"]
mod tests;
