//! What:
//!  What every command of a repository remembers about a place it already named to the person,
//!  so one directory this program may not act on is said once rather than on every command.
//! Why:
//!  The look every command makes at a removal's leftovers says a line for a place it may not touch:
//!  a directory that is not the subvolume a record names,
//!  one it cannot look at,
//!  and one whose name alone shows it and whose name and kernel disagree.
//!  A line said on every commit is noise, and nothing in it changes until the person acts,
//!  so the first command that says it writes the line down,
//!  and the commands after it say nothing while that memory stands.
//!  The memory goes as soon as the place is proved,
//!  queued,
//!  or gone,
//!  because a state that changes must be said again if it comes back.
//!  Nothing here deletes a directory:
//!  it is one file per said line,
//!  and a repository whose leftovers were all proved pays one failed open of the folder below.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! reportOnce({ commonDir, place, line, answer }); // pushes the line only the first time
//! forgetTheReport(commonDir, place);              // the place was proved, queued or gone
//! ```

use crate::diagnostic_log::warn;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeavingsReported";

/// The folder of the common Git directory that holds one file per line already said about a place.
pub(crate) const REPORTED_FOLDER: &str = "monochromatic-git-policy-cli/leavings-reported";

/// The file one place's said line is remembered in,
/// named by a digest of the place's bytes so that one place is one file.
/// Why a digest and not the path:
/// the name a path takes in hexadecimal is twice its length,
/// and a path of the length the kernel allows is already more than a file name holds.
fn report_path(common_dir: &Path, place: &Path) -> PathBuf {
    let digest: String = crate::hexadecimal_of_bytes::hexadecimal_of_bytes(crate::subvolume_sha::sha1(crate::path_bytes::path_bytes_of(place)).as_slice());
    return common_dir.join(REPORTED_FOLDER).join(digest);
}

/// What:
///  Push `line` into `said` only when no command of this repository has said a line about `place` yet.
/// Why:
///   A place the person must act on stands until they act,
///  so a line said again on every command is noise and not news.
///  A memory that cannot be written costs the quiet:
///  the line is said again on the next command,
///  which is what every command did before this memory existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = reportOnce(commonDir, place, line); // said.push(line) only the first time
/// ```
pub(crate) fn report_once(common_dir: &Path, place: &Path, line: String, said: &mut Vec<String>) {
    let record: PathBuf = report_path(common_dir, place);
    if record.is_file() {
        return;
    }
    if !remember_the_line(record.as_path(), line.as_bytes()) {
        // A memory that cannot be written costs the quiet: the line is said now, and the next command says it again,
        // which is what every command did before this memory existed.
        said.push(line);
        return;
    }
    said.push(line);
}

/// What:
///  Write the line a place earned into its file,
///  or answer that the file could not be written.
/// Why:
///   Both failures of the memory cost the same thing,
///  and neither is the person's to act on,
///  so each is logged for the diagnostic log and answered the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remembered = await rememberTheLine(record, bytes);
/// ```
fn remember_the_line(record: &Path, bytes: &[u8]) -> bool {
    if let Some(folder) = record.parent()
        && let Err(error) = create_private_directory_all(folder)
    {
        warn(TAG, format!("{} cannot be made, so the line a leftover earned is said again ({error})", folder.display()).as_str());
        return false;
    }
    if let Err(error) = write_private_file_unsynced(record, bytes) {
        warn(TAG, format!("{} cannot be written, so the line a leftover earned is said again ({error})", record.display()).as_str());
        return false;
    }
    return true;
}

/// What:
///  Forget what was said about `place`,
///  because the place was proved,
///  queued,
///  or is gone.
/// Why:
///   A state that changes must be said again if it comes back,
///  and a memory that outlives its place costs one lookup on every command for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheReport(commonDir, place);
/// ```
pub(crate) fn forget_the_report(common_dir: &Path, place: &Path) {
    let _removed = std::fs::remove_file(report_path(common_dir, place));
}
