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
use std::path::Path;

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

/// The doctor's text for the folder:
/// empty when it holds no copy and nothing went wrong,
/// otherwise one block with the size,
/// the count,
/// the oldest save and the limits that apply,
/// then a sentence for each thing that could not be read.
pub fn doctor_section(common_dir: &Path, limits: Limits) -> String {
    // The report changes nothing, so it reads the folder under the name it has now, else under the earlier one.
    let relative: String = relative_path_for_reading(common_dir, SAVED_DIRECTORY);
    let Some(found) = report_below(common_dir, relative.as_str()) else {
        return String::new();
    };
    let mut text: String = String::new();
    if found.saves > 0 {
        text = format!(
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
        );
    }
    for problem in &found.problems {
        text.push_str(problem.message().replacen("monochromatic-git-policy-cli: ", "git monochromatic-git-policy-cli doctor: discarded copies: ", 1).as_str());
    }
    return text;
}
