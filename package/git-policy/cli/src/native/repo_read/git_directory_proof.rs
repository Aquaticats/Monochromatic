//! What:
//!  The one proof that a folder is the Git directory of the command,
//!  resolved so that no component of its path is a symbolic link.
//! Why:
//!  The lock pass opens and unlinks files,
//!  and it may do that only inside a Git directory Git itself would accept:
//!  a folder an option named,
//!  or one a walk reached through a link,
//!  is no repository,
//!  and cleaning it deleted a person's file from a folder that is not one
//!  (a review of `551084db3c`,
//! findings 2 and 3).
//!  So the proof is taken once,
//!  before anything is judged,
//!  and handed to the pass as an argument:
//!  a caller with no proof has no pass.
//!  A link above the Git directory is resolved rather than refused,
//!  because that is where the repository really is;
//!  what the proof guarantees is that the path it gives has no link left in it,
//!  so every folder below it can be opened one component at a time without one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proven = proveGitDirectory(gitDir, commonDir); // realpath both, then Git's own signatures, or undefined
//! ```

/// Git's own test of a Git directory:
///  a `HEAD` that names a reference or an object,
///  and either an object store and a reference folder or a `commondir` file.
use std::path::{Path, PathBuf};

/// The longest `HEAD` this reads.
const HEAD_LIMIT: u64 = 1 << 16;

/// What:
///  A Git directory this command may have its locks cleared in,
///  and the common Git directory its records and shared references live in,
///  each with no link in its path.
/// Why:
///  Every root the pass derives is built from these two folders,
///  so a proof here is what makes a candidate reachable only inside a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProvenGitDirectory = { gitDir: string; commonDir: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenGitDirectory {
    /// The Git directory,
    ///  resolved so that no component is a link.
    git_dir: PathBuf,
    /// The common Git directory,
    ///  resolved the same way.
    common_dir: PathBuf,
}

/// What:
///  Prove that `git_dir` is a Git directory of this machine's Git,
///  or answer nothing when it is not one.
/// Why:
///   The folder is resolved first,
///  so a link anywhere in the path is followed once,
///  and from then on the path has none;
///  then Git's own signatures decide:
///  a `HEAD` that names a reference or an object,
///  and an object store with a reference folder,
///  or the `commondir` file a linked worktree's Git directory carries.
///  A folder that fails this is no repository,
///  and the pass and the record folder both stay out of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proven = (gitDir, commonDir) => { const real = realpathSync(gitDir); const shared = realpathSync(commonDir); return isGitDirectory(real) && isGitDirectory(shared) ? { gitDir: real, commonDir: shared } : undefined };
/// ```
pub fn prove_git_directory(git_dir: &Path, common_dir: &Path) -> Option<ProvenGitDirectory> {
    // What: Both folders are the ones the command's one resolver answered, resolved to real paths, and both are Git
    //       directories by Git's own test.
    // Why:  The common directory is never derived a second time here: with `GIT_COMMON_DIR` the resolver's answer is the
    //       one the records were written in, and a proof that read `commondir` itself would answer a folder the writer
    //       never used (a review of `4e9b9fff1d`, finding 1).
    let resolved: PathBuf = std::fs::canonicalize(git_dir).ok()?;
    let common: PathBuf = std::fs::canonicalize(common_dir).ok()?;
    if !is_a_git_directory(resolved.as_path()) || !is_a_git_directory(common.as_path()) {
        return None;
    }
    return Some(ProvenGitDirectory { git_dir: resolved, common_dir: common });
}

/// What:
///  Whether Git would find a repository at `directory`:
///  a `HEAD` that names a reference or an object,
///  and an object store with a reference folder,
///  or a `commondir` file.
/// Why:
///   This is `setup.c`'s own list (`is_git_directory`),
///  read from the files and never from a name,
///  so a folder of a person's that merely sits where a Git directory would cannot be proven a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isGitDirectory = (dir) => validHead(dir) && ((isDir(dir + '/objects') && isDir(dir + '/refs')) || isFile(dir + '/commondir'));
/// ```
pub fn is_a_git_directory(directory: &Path) -> bool {
    if !head_names_a_reference_or_object(directory.join("HEAD").as_path()) {
        return false;
    }
    // What: The object store and the reference folder are read as Git reads them, a link followed once at the last
    //       component.
    // Why:  This answers whether Git accepts the folder, and Git follows those two names;
    //       that no link is ever opened belongs to the walk of the roots
    //       (`abandoned_locks.rs`,
    // `folder_below_proven`), and that no root of the proof's own path is a link belongs to the resolution above.
    let shared: bool = directory.join("objects").is_dir() && directory.join("refs").is_dir();
    let linked: bool = matches!(std::fs::symlink_metadata(directory.join("commondir")).map(|metadata| { return metadata.file_type(); }), Ok(kind) if kind.is_file());
    return shared || linked;
}

/// What:
///  Whether `HEAD` names a reference or an object,
///  which is the one file every Git directory has.
/// Why:
///   `HEAD` is either `ref: <name>` or an object's hexadecimal text,
///  and a folder with no such file is no repository to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const validHead = (path) => { const text = readText(path, HEAD_LIMIT).trim(); return /^ref: \S+$/.test(text) || /^[0-9a-f]{40}([0-9a-f]{24})?$/.test(text) };
/// ```
fn head_names_a_reference_or_object(path: &Path) -> bool {
    let bytes: Vec<u8> = match crate::plain_file::read_plain_file(path, HEAD_LIMIT) {
        Ok(read) => read,
        Err(_unreadable) => return false,
    };
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    let head: &str = text.trim();
    if let Some(named) = head.strip_prefix("ref: ") {
        return !named.trim().is_empty();
    }
    let is_hex: bool = matches!(head.len(), 40 | 64) && head.bytes().all(|byte| { return byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase(); });
    return is_hex;
}

/// What:
///  The Git directory and the common one, for a caller that proves nothing itself.
/// Why:
///   A control reads the two folders a proof gives,
///  and nothing else may read them from here:
///  the pass takes the proof, not the paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proven.path(); proven.common()
/// ```
impl ProvenGitDirectory {
    /// The Git directory, with no link in its path.
    pub fn path(&self) -> &Path {
        return self.git_dir.as_path();
    }

    /// The common Git directory, with no link in its path.
    pub fn common(&self) -> &Path {
        return self.common_dir.as_path();
    }
}
