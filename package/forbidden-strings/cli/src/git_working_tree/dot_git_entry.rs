//! What:
//!  One directory in the walk upward,
//!  and what its `.git` entry is:
//!  a Git directory,
//!  a `.git` file naming one,
//!  nothing,
//!  or an entry Git would stop on but this scanner cannot use.
//! Why:
//!  Git's own discovery reads a `.git` file's `gitdir:` line,
//!  strips trailing line breaks there and in `commondir`,
//!  and validates the directory each one names;
//!  a scan must find the same directories to read the same index,
//!  and the constants here are Git's own bounds and prefixes,
//!  quoted from its source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // gitDirectories(root): RootEntry, reading .git (file or directory) first
//! ```

/// Imports the path types the entries and the directories they name are
///  expressed with.
use std::path::{
    Path,
    PathBuf,
};

/// The most a `.git` file may hold,
/// from `read_gitfile_raw` in Git's `setup.c`
///  (`const int max_file_size = 1 << 20;`):
///  Git refuses a file strictly larger,
///  so a larger one names no repository.
const GIT_FILE_MAX_BYTES: u64 = 1 << 20;

/// The exact first bytes of a `.git` file,
/// from Git's `read_gitfile_raw`
///  (`if (!starts_with(buf, "gitdir: "))`),
/// space included.
const GITDIR_PREFIX: &[u8] = b"gitdir: ";

/// The characters Git strips from the end of a `.git` file and of `commondir`
///  (`while (buf[len - 1] == '\n' || buf[len - 1] == '\r') len--;`).
const TRAILING_LINE_BYTES: &[u8] = b"\n\r";

/// What one directory in the walk upward holds as a `.git` entry.
pub(super) enum RootEntry {
    /// No `.git` entry,
    /// or a `.git` directory that is not a Git directory;
    ///  Git steps over both and keeps walking upward.
    None,
    /// A `.git` entry Git would stop on but this scanner cannot use.
    Unusable {
        /// The `.git` entry itself.
        entry: PathBuf,
        /// What is wrong with it,
        /// in words.
        cause: String,
    },
    /// The two Git directories of this working tree.
    Tree {
        /// The directory holding this working tree's `index`.
        git_dir: PathBuf,
        /// The directory holding the shared state,
        /// including `config`.
        common_dir: PathBuf,
    },
}

/// The Git directory of the working tree rooted at `root`,
///  with the directory holding the shared configuration,
///  read by Git's own order:
///  a `.git` file first,
/// then a `.git` directory,
///  and the directory each names validated as a Git directory.
pub(super) fn git_directories(root: &Path) -> RootEntry {
    let dot_git: PathBuf = root.join(".git");
    let metadata: std::fs::Metadata = match std::fs::metadata(&dot_git) {
        Ok(metadata) => metadata,
        // Nothing there, or a path this process may not stat:
        // `read_gitfile_raw` treats a missing entry as "keep looking" and a
        // failed stat as fatal, and an unresolvable entry can be neither
        // confirmed nor denied, so it is reported rather than stepped over.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return RootEntry::None,
        Err(error) => {
            return RootEntry::Unusable {
                entry: dot_git,
                cause: format!("it cannot be read ({})", error),
            };
        }
    };
    if metadata.is_dir() {
        // A directory named `.git` is a repository only when it holds Git's
        // state; Git steps over anything else and keeps walking upward.
        if !is_git_directory(&dot_git) {
            return RootEntry::None;
        }
        let git_dir: PathBuf = resolved_or_spelled(&dot_git);
        let common_dir: PathBuf = common_dir_of(&git_dir);
        return RootEntry::Tree {
            git_dir,
            common_dir,
        };
    }
    if !metadata.is_file() {
        return RootEntry::Unusable {
            entry: dot_git,
            cause: "it is not a regular file".to_string(),
        };
    }
    let contents: Vec<u8> = match read_git_file(&dot_git) {
        Ok(contents) => contents,
        Err(cause) => {
            return RootEntry::Unusable {
                entry: dot_git,
                cause,
            };
        }
    };
    let named: PathBuf = match gitdir_from_git_file(&contents) {
        Ok(named) => named,
        Err(cause) => {
            return RootEntry::Unusable {
                entry: dot_git,
                cause,
            };
        }
    };
    // Git builds the path by concatenation and leaves `..` to the operating
    // system, so the spelling is kept and no component is collapsed here.
    let git_dir: PathBuf = if named.is_absolute() {
        named
    } else {
        // A relative `gitdir:` is resolved against the directory holding the `.git` file, which is `root`.
        root.join(named)
    };
    if !is_git_directory(&git_dir) {
        return RootEntry::Unusable {
            entry: dot_git,
            cause: format!(
                "it names {}, which is not a Git directory",
                git_dir.display()
            ),
        };
    }
    let resolved_git_dir: PathBuf = resolved_or_spelled(&git_dir);
    let common_dir: PathBuf = common_dir_of(&resolved_git_dir);
    return RootEntry::Tree {
        git_dir: resolved_git_dir,
        common_dir,
    };
}

/// Whether `suspect` holds what Git's `is_git_directory` requires:
///  a `HEAD` naming a ref or an object id,
///  and `objects` and `refs` beside the shared state.
fn is_git_directory(suspect: &Path) -> bool {
    if !head_names_a_ref_or_object_id(&suspect.join("HEAD")) {
        return false;
    }
    let common: PathBuf = common_dir_of(suspect);
    return common.join("objects").is_dir() && common.join("refs").is_dir();
}

/// Whether `head` is the file Git accepts as a repository's `HEAD`:
///  a symbolic ref into `refs/`,
/// or an object id.
fn head_names_a_ref_or_object_id(head: &Path) -> bool {
    let Ok(contents) = std::fs::read_to_string(head) else {
        return false;
    };
    let line: &str = contents.lines().next().unwrap_or("").trim();
    if let Some(target) = line.strip_prefix("ref:") {
        return target.trim_start().starts_with("refs/");
    }
    // A detached HEAD holds one object id, forty bytes for Sha1 and sixty-four for Sha256.
    if !matches!(line.len(), 40 | 64) {
        return false;
    }
    for byte in line.bytes() {
        if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    return true;
}

/// What an operating system refusal of a `.git` file means for a scan,
///  in one phrase.
fn git_file_unreadable(error: std::io::Error) -> String {
    return format!("it cannot be read ({})", error);
}

/// The bytes of a `.git` file,
/// bounded by Git's own ceiling.
fn read_git_file(path: &Path) -> Result<Vec<u8>, String> {
    let metadata: std::fs::Metadata = std::fs::metadata(path).map_err(git_file_unreadable)?;
    if metadata.len() > GIT_FILE_MAX_BYTES {
        return Err("it is too large to be a .git file".to_string());
    }
    return std::fs::read(path).map_err(git_file_unreadable);
}

/// The Git directory a `.git` file's own contents name,
///  before a relative one is resolved against its working tree.
///
/// Git strips every trailing line break and requires the `gitdir: `
///  prefix,
///  so a newline inside the named path stays part of that path,
///  and a file without the prefix is not a Git file at all.
pub(crate) fn gitdir_from_git_file(contents: &[u8]) -> Result<PathBuf, String> {
    let mut line: &[u8] = contents;
    while let [rest @ .., last] = line {
        if !TRAILING_LINE_BYTES.contains(last) {
            break;
        }
        line = rest;
    }
    let Some(named) = line.strip_prefix(GITDIR_PREFIX) else {
        return Err("it does not start with `gitdir: `".to_string());
    };
    if named.is_empty() {
        return Err("it names no path".to_string());
    }
    let Ok(text) = std::str::from_utf8(named) else {
        return Err("the path it names is not UTF-8".to_string());
    };
    return Ok(PathBuf::from(text));
}

/// The directory holding the repository's shared state,
///  from `<git_dir>/commondir` when a linked worktree has one,
///  and `git_dir` itself otherwise.
///
/// Git strips every trailing line break there too,
///  and resolves a relative entry against the Git directory.
fn common_dir_of(git_dir: &Path) -> PathBuf {
    let Ok(contents) = std::fs::read(git_dir.join("commondir")) else {
        return git_dir.to_path_buf();
    };
    let mut line: &[u8] = &contents;
    while let [rest @ .., last] = line {
        if !TRAILING_LINE_BYTES.contains(last) {
            break;
        }
        line = rest;
    }
    let Ok(text) = std::str::from_utf8(line) else {
        return git_dir.to_path_buf();
    };
    return resolved_or_spelled(&git_dir.join(text));
}

/// `path` with symlinks followed and navigation components settled by the
///  filesystem,
///  or the path as spelled when it cannot be resolved.
///
/// Git realpaths the Git directory it discovered
///  (`strbuf_realpath` in `read_gitfile_gently`)
///  and the common directory (`strbuf_add_real_path`),
///  so a `gitdir:` line holding `..` addresses the same directory here as it
///  does there,
///  including when a `..` follows a symlink.
fn resolved_or_spelled(path: &Path) -> PathBuf {
    return match std::fs::canonicalize(path) {
        Ok(resolved) => resolved,
        Err(_) => path.to_path_buf(),
    };
}
