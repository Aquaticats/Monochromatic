//! What:
//!  What every command of a repository remembers about a place it already named to the person,
//!  so one directory this program may not act on is said once rather than on every command.
//! Why:
//!  The look every command makes at a removal's leftovers says a line for a place it may not touch:
//!  a directory that is not the subvolume a record names,
//!  one it cannot look at,
//!  one whose name alone shows it and whose name and kernel disagree,
//!  and one whose attempts cannot be counted.
//!  A line said on every commit is noise, and nothing in it changes until the person acts,
//!  so the first command that says it writes the line down,
//!  and the commands after it say nothing while that memory stands.
//!  The memory goes as soon as the place is proved,
//!  queued,
//!  or gone,
//!  because a state that changes must be said again if it comes back,
//!  and a memory whose place is gone is dropped by the next command
//!  (`forget_the_reports_of_places_that_moved`),
//!  so a person's directory that goes away does not keep a file here for ever.
//!  The memory of one place is dropped when the object at that path is another one,
//!  so a second, different, directory at a name already said is said again.
//!  Nothing here deletes a directory:
//!  it is one file per said line,
//!  and a repository whose leftovers were all proved pays one failed open of the folder below.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! reportOnce({ commonDir, place, line, answer });  // pushes the line only the first time
//! forgetTheReport(commonDir, place);               // the place was proved, queued or gone
//! forgetTheReportsOfPlacesThatMoved(commonDir);    // a command drops the memories of places that are not there
//! ```

use crate::diagnostic_log::warn;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use std::os::unix::fs::MetadataExt as _;
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
///  The identity of one place as the filesystem holds it now,
///  or nothing when nothing stands at the path.
/// Why:
///  A name is not an object:
///  a person who removes a directory and makes another one at the same name has a different place,
///  and the memory of a line said about the first must be dropped so the second can be said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = lstat(path) ?? undefined; // { device, inode }
/// ```
fn the_identity_of(place: &Path) -> Option<(u64, u64)> {
    let found: std::fs::Metadata = std::fs::symlink_metadata(place).ok()?;
    return Some((found.dev(), found.ino()));
}

/// What:
///  The bytes of one memory:
///  the place's own bytes in hexadecimal,
///  the identity of the object that stood there,
///  and the line the person read.
/// Why:
///   A later command must be able to tell whether the place this memory is about still stands,
///  and whether it is the same object,
///  and the path may hold bytes no line of text can carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = `${hex(pathBytes(place))}\n${device}:${inode}\n${line}`;
/// ```
fn bytes_of_the_memory(place: &Path, line: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(crate::hexadecimal_of_bytes::hexadecimal_of_bytes(crate::path_bytes::path_bytes_of(place)).as_bytes());
    bytes.push(b'\n');
    let (device, inode): (u64, u64) = the_identity_of(place).unwrap_or((0, 0));
    bytes.extend_from_slice(format!("{device}:{inode}\n").as_bytes());
    bytes.extend_from_slice(line);
    return bytes;
}

/// What:
///  The place one memory is about,
///  and the identity it wrote down,
///  or nothing when the file is not one this release wrote.
/// Why:
///   A memory written by a release before this one holds the line alone and no place,
///  and a file a person wrote holds neither;
///  both are memories of nothing this command can check,
///  so the line is said once more and the file goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [pathHex, identity] = text.split('\n', 2); return [pathFromHex(pathHex), identity];
/// ```
fn the_place_a_memory_names(bytes: &[u8]) -> Option<(PathBuf, (u64, u64))> {
    let text: &str = std::str::from_utf8(bytes).ok()?;
    let (spelled, after): (&str, &str) = text.split_once('\n')?;
    let (identity, _line): (&str, &str) = after.split_once('\n')?;
    let (device, inode): (&str, &str) = identity.split_once(':')?;
    let mut path_bytes: Vec<u8> = Vec::with_capacity(spelled.len() / 2);
    if spelled.len() % 2 != 0 {
        return None;
    }
    for pair in spelled.as_bytes().chunks(2) {
        path_bytes.push(u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?);
    }
    let place: PathBuf = PathBuf::from(crate::path_bytes::os_from_path_bytes(path_bytes.as_slice()));
    return Some((place, (device.parse().ok()?, inode.parse().ok()?)));
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
    if !remember_the_line(record.as_path(), place, line.as_bytes()) {
        // A memory that cannot be written costs the quiet: the line is said now, and the next command says it again,
        // which is what every command did before this memory existed.
        said.push(line);
        return;
    }
    said.push(line);
}

/// What:
///  Write the line a place earned into its file,
///  with the place and the object that stood there,
///  or answer that the file could not be written.
/// Why:
///   Both failures of the memory cost the same thing,
///  and neither is the person's to act on,
///  so each is logged for the diagnostic log and answered the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remembered = await rememberTheLine(record, place, bytes);
/// ```
fn remember_the_line(record: &Path, place: &Path, line: &[u8]) -> bool {
    if let Some(folder) = record.parent()
        && let Err(error) = create_private_directory_all(folder)
    {
        warn(TAG, format!("{} cannot be made, so the line a leftover earned is said again ({error})", folder.display()).as_str());
        return false;
    }
    if let Err(error) = write_private_file_unsynced(record, bytes_of_the_memory(place, line).as_slice()) {
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

/// What:
///  Drop every memory of a place that no longer stands,
///  or whose object is another one,
///  and every file in the folder this release did not write.
/// Why:
///   A memory that outlives its place is one file for ever per name a person made and removed,
///  and the next different object at a name already said is said nothing about.
///   The cost is one listing of the folder and one lookup per memory while any stands,
///  and a folder that is not there costs one failed open,
///  which is what a repository whose leftovers were all proved pays on every command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheReportsOfPlacesThatMoved(commonDir); // unlink every memory whose place is not the object it named
/// ```
pub(crate) fn forget_the_reports_of_places_that_moved(common_dir: &Path) {
    let folder: PathBuf = common_dir.join(REPORTED_FOLDER);
    let Ok(listing) = std::fs::read_dir(folder.as_path()) else { return };
    for entry in listing.flatten() {
        let record: PathBuf = entry.path();
        let Ok(bytes) = std::fs::read(record.as_path()) else { continue };
        let named: Option<(PathBuf, (u64, u64))> = the_place_a_memory_names(bytes.as_slice());
        let Some((place, wrote)) = named else {
            let _dropped = std::fs::remove_file(record.as_path());
            continue;
        };
        if the_identity_of(place.as_path()) == Some(wrote) {
            continue;
        }
        let _dropped = std::fs::remove_file(record.as_path());
    }
    let _kept = std::fs::remove_dir(folder.as_path());
}
