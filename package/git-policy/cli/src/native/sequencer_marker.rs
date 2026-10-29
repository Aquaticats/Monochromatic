//! What:
//!  Proof, without a Git process, that a sequencer marker such as `MERGE_HEAD` names nothing.
//! Why:
//!  A commit asks `git rev-parse --verify --quiet <marker>` for each of three markers,
//!  and in almost every commit none exists.
//!  On the files ref storage that question reads a fixed list of places,
//!  so when every one of them is empty the answer is "absent" without a process;
//!  any entry in any place, of any kind or content,
//!  says "maybe present" and the caller asks Git,
//!  which alone decides whether the entry resolves (a dangling marker still counts as a conclusion).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const maybePresent = names.map((name) => lookupPlaces(layout, name).some(exists) || packedRefs.includes(name));
//! ```

/// Where one worktree's ref files live and what format they are in.
use super::ref_files::RefFilesLayout;
/// The ref storage backend.
use super::transaction_journal::RefFormat;
/// Borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The most `packed-refs` bytes this check reads;
///  a bigger file is left to Git.
const PACKED_REFS_LIMIT: u64 = 64 * 1024 * 1024;

/// What:
///  The paths `git rev-parse --verify <name>` looks at for a loose entry,
///  in Git's order.
/// Why:
///   Git's lookup rules try the name itself (a pseudoref in the worktree's Git directory),
///  then `refs/`, `refs/tags/`, `refs/heads/`, `refs/remotes/` and `refs/remotes/<name>/HEAD`
///  in the common directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [join(gitDir, name), join(commonDir, 'refs', name), join(commonDir, 'refs/tags', name), ...]
/// ```
fn lookup_paths(layout: &RefFilesLayout<'_>, name: &str) -> Vec<PathBuf> {
    return vec![
        layout.git_dir.join(name),
        layout.common_dir.join(format!("refs/{name}")),
        layout.common_dir.join(format!("refs/tags/{name}")),
        layout.common_dir.join(format!("refs/heads/{name}")),
        layout.common_dir.join(format!("refs/remotes/{name}")),
        layout.common_dir.join(format!("refs/remotes/{name}/HEAD")),
    ];
}

/// What:
///  Whether something exists at `path`,
///  where anything but a clean "nothing here" counts as existing.
/// Why:
///   A link, a directory, a file of any content and an unreadable parent all send the caller to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lstat(path).then(() => true, (error) => error.code !== 'ENOENT')
/// ```
fn something_is_at(path: &Path) -> bool {
    return match std::fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(error) => error.kind() != std::io::ErrorKind::NotFound,
    };
}

/// What:
///  The bytes of `packed-refs`,
///  nothing when there is no such file,
///  or `None` when the file cannot be read as a whole.
/// Why:
///   A missing `packed-refs` holds no ref;
///  an unreadable or oversized one is not proof of anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readFile(join(commonDir, 'packed-refs')).catch((e) => (e.code === 'ENOENT' ? Buffer.alloc(0) : undefined))
/// ```
fn packed_refs_bytes(common_dir: &Path) -> Option<Vec<u8>> {
    let path: PathBuf = common_dir.join("packed-refs");
    let metadata: std::fs::Metadata = match std::fs::metadata(&path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(Vec::new()),
        Err(_) => return None,
    };
    if !metadata.is_file() || metadata.len() > PACKED_REFS_LIMIT {
        return None;
    }
    return match std::fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(Vec::new()),
        Err(_) => None,
    };
}

/// What:
///  Whether `needle` occurs anywhere in `haystack`.
/// Why:
///   Any occurrence of the marker's name in `packed-refs` is a hit,
///  a superset of the lines that name one of the lookup spellings;
///  the extra hits only cost a Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// haystack.includes(needle)
/// ```
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    return haystack.windows(needle.len()).any(|window| { return window == needle; });
}

/// What:
///  For each name, whether a marker of that name may exist:
///  `false` is proof that `git rev-parse --verify --quiet <name>` exits non-zero.
/// Why:
///   The files backend resolves a name only through a loose entry at one of six paths
///  or a line of `packed-refs`; reftable storage and anything unreadable answer "maybe".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.map((name) => layout.refFormat !== 'files' || maybePresent(layout, name))
/// ```
pub fn markers_maybe_present(layout: &RefFilesLayout<'_>, names: &[&str]) -> Vec<bool> {
    if layout.ref_format != RefFormat::Files {
        return vec![true; names.len()];
    }
    let packed: Option<Vec<u8>> = packed_refs_bytes(layout.common_dir);
    let mut answers: Vec<bool> = Vec::with_capacity(names.len());
    for name in names {
        let loose: bool = lookup_paths(layout, name).iter().any(|path| { return something_is_at(path.as_path()); });
        let packed_hit: bool = match &packed {
            Some(bytes) => contains_bytes(bytes.as_slice(), name.as_bytes()),
            None => true,
        };
        answers.push(loose || packed_hit);
    }
    return answers;
}

/// Sequencer marker controls stay out of the release executable.
#[cfg(test)]
#[path = "sequencer_marker_tests.rs"]
mod tests;
