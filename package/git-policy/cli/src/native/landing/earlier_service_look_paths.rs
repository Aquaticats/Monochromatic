//! What:
//!  The questions about the file system that the look at an earlier commit service asks:
//!  the bytes that spell a path,
//!  the part of a path the file system knows,
//!  and the identity of each earlier journal that exists.
//! Why:
//!  A journal word of a command line has several spellings of one folder
//! (`/home/user/...` through the link `/home`,
//!  `/var/home/user/...`),
//! and a folder an earlier helper has not made yet has no identity to compare,
//! so the same two
//!  answers are needed wherever a word or a journal is judged,
//! and they are written once here.
//!  These answers depend on the file system only,
//! so this module uses nothing of the look.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lookPaths.ts: the spelling of a path, the part of it the file system knows, and the identity of a journal folder.
//! ```

use std::path::{Path, PathBuf};

/// The bytes of a path,
/// as a command line word spells it.
#[cfg(unix)]
pub(crate) fn path_word(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    return path.as_os_str().as_bytes().to_vec();
}

/// The bytes of a path,
/// as a command line word spells it,
/// for a platform whose paths are not bytes.
#[cfg(not(unix))]
pub(crate) fn path_word(path: &Path) -> Vec<u8> {
    return path.to_string_lossy().into_owned().into_bytes();
}

/// The identity of a folder or file:
/// the device it is on and its inode,
/// which every spelling of one folder shares.
#[cfg(unix)]
type FileIdentity = (u64, u64);

/// What:
///  The path a journal word names,
/// with the longest part of it that exists resolved and the names below that part as the word spells them.
/// Why:
///   A journal folder that the process has not made yet has no device and inode to compare,
///  and the same folder is spelled
///  differently by the process that runs this command and by the unit that started an earlier helper
///  (`/home/user/...` through the link `/home`,
///  `/var/home/user/...`).
/// The longest existing part is what the
///  file system can answer for,
/// so resolving it compares the two spellings,
/// and a name below it that does not exist yet is the
///  same name in both.
/// A word that reaches past a name the file system will not look through resolves to nothing,
/// and a
///  word that resolves to nothing proves nothing either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolvedSoFar = (word: string): string => { let at = word; const below: string[] = []; for (;;) { try { return join(realpathSync(at), ...below) } catch (e) { if (e.code !== 'ENOENT') throw e; below.unshift(basename(at)); at = dirname(at) } } }
/// ```
#[cfg(unix)]
pub(crate) fn resolved_as_far_as_the_file_system_knows(word: &Path) -> std::io::Result<PathBuf> {
    let mut below: Vec<&std::ffi::OsStr> = Vec::new();
    let mut at: &Path = word;
    loop {
        match std::fs::canonicalize(at) {
            Ok(known) => {
                let mut path: PathBuf = known;
                for name in below.iter().rev() {
                    path.push(*name);
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // A path whose last name is `..` has no name to carry down, and where it points is not known here.
                let (Some(name), Some(parent)) = (at.file_name(), at.parent()) else {
                    return Err(error);
                };
                below.push(name);
                at = parent;
            }
            Err(error) => return Err(error),
        }
    }
}

/// The identity of each earlier journal that exists,
/// or `None` when one cannot be looked at for a reason other than its absence.
#[cfg(unix)]
pub(crate) fn journal_identities(journals: &[PathBuf]) -> Option<Vec<FileIdentity>> {
    use std::os::unix::fs::MetadataExt;
    let mut identities: Vec<FileIdentity> = Vec::new();
    for journal in journals {
        match std::fs::metadata(journal) {
            Ok(found) => identities.push((found.dev(), found.ino())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return None,
        }
    }
    return Some(identities);
}

#[cfg(test)]
#[path = "earlier_service_look_paths_tests.rs"]
mod tests;
