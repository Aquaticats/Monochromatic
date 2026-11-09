//! What:
//!  Where Git keeps the administrative data of the working tree a scan walks:
//!  the index recording that working tree's tracked files,
//!  and the configuration recording the index's object format.
//! Why:
//!  `<root>/.git` is the main repository's layout only.
//!  A linked worktree holds a `.git` file whose `gitdir:` line names
//!  `<main>/.git/worktrees/<name>`,
//!  a submodule holds one naming `<host>/.git/modules/<name>`,
//!  and a scan started in a subfolder has no `.git` of its own at all,
//!  so every case finds its index the way Git's discovery does:
//!  from the walked directory upward,
//!  reading a `.git` file's `gitdir:` line,
//!  and validating the directory that line names.
//!  Joining `.git/index` onto the walked directory found none of those
//!  (issue 1232).
//!
//! What it does not do,
//!  and why the README says so:
//!  `GIT_DIR` and `GIT_WORK_TREE` name a different worktree root,
//!  `core.worktree` stores one in configuration,
//!  and Git's discovery also stops at mount points and honours
//!  `GIT_CEILING_DIRECTORIES`.
//!  Those need Git's own configuration and ownership machinery,
//!  which this package deliberately does not link,
//!  so the walked directory stays the worktree root here.
//!  `GIT_INDEX_FILE` is honoured,
//!  because it names the very file this module opens.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const discovery = discoverWorkingTree(walkRoot);
//! // const index = discovery.kind === "working-tree"
//! //   ? openIndex(discovery.tree.indexPath, discovery.tree.hashKind())
//! //   : null;
//! ```

/// Imports the path component type that lexical normalization classifies,
///  and the two path types every caller passes.
use std::path::{Component, Path, PathBuf};

/// Imports the platform-aware separator normalization,
///  so one scope prefix compares against Git's always-`/`-separated index paths.
use crate::path_name_bytes::normalized_path;

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

/// The environment variable naming the active index file.
const INDEX_FILE_ENVIRONMENT: &str = "GIT_INDEX_FILE";

/// What Git's discovery found above the walked directory.
pub(crate) enum Discovery {
    /// The walked directory is inside a working tree.
    WorkingTree(
        /// The resolved directories of that working tree.
        GitWorkingTree,
    ),
    /// No `.git` entry above the walked directory,
    ///  so nothing claims a workspace with tracked files.
    NoRepository,
    /// A `.git` entry exists above the walked directory but names nothing this
    ///  scanner can read a working tree from.
    ///
    /// Git itself stops discovery with a fatal error here;
    ///  this value exists so a scan can say the same thing while still
    ///  walking the files it can see.
    Unusable {
        /// The `.git` entry that could not be used.
        entry: PathBuf,
        /// What is wrong with it,
        /// in words.
        cause: String,
    },
}

/// The administrative directories of one working tree,
///  and the walked directory's place below its root.
pub(crate) struct GitWorkingTree {
    /// The Git directory holding this working tree's `index`.
    git_dir: PathBuf,
    /// The Git directory holding `config`:
    ///  the shared one for a linked worktree,
    ///  whose own directory holds only per-worktree state.
    common_dir: PathBuf,
    /// The walked directory's path below the working tree root,
    ///  `/`-separated with a trailing separator,
    ///  empty when the scan starts at the root.
    scope_below_root: Vec<u8>,
}

/// Finds the working tree whose root is `walk_root` or one of its ancestors.
///
/// The nearest `.git` wins,
///  which is the innermost working tree Git itself would work in.
pub(crate) fn discover(walk_root: &Path) -> Discovery {
    let Some(absolute) = absolute_path(walk_root) else {
        return Discovery::Unusable {
            entry: walk_root.to_path_buf(),
            cause: "the scan directory cannot be resolved".to_string(),
        }
    };
    for candidate in absolute.ancestors() {
        match git_directories(candidate) {
            RootEntry::None => {}
            RootEntry::Unusable { entry, cause } => return Discovery::Unusable { entry, cause },
            RootEntry::Tree { git_dir, common_dir } => {
                let below: &Path = absolute.strip_prefix(candidate).unwrap_or(Path::new(""));
                return Discovery::WorkingTree(GitWorkingTree {
                    git_dir,
                    common_dir,
                    scope_below_root: scope_bytes(below),
                })
            }
        }
    }
    return Discovery::NoRepository
}

/// What this working tree's index is asked for,
///  and how its entries are read.
impl GitWorkingTree {
    /// The path `gix-index` opens.
    ///
    /// `GIT_INDEX_FILE` names the active index when it is set,
    ///  relative to the current directory as Git reads it,
    ///  which is how a Git command that scans with a temporary index
    ///  (a commit in progress,
    /// for example) makes that index the one in use.
    pub(crate) fn index_path(&self) -> PathBuf {
        if let Some(named) = std::env::var_os(INDEX_FILE_ENVIRONMENT)
            && !named.is_empty()
        {
            return PathBuf::from(named)
        }
        return self.git_dir.join("index")
    }

    /// The configuration file recording this working tree's object format.
    pub(crate) fn config_path(&self) -> PathBuf {
        return self.common_dir.join("config")
    }

    /// One index entry's path expressed below the walked directory,
    ///  or `None` when the entry lies outside it.
    ///
    /// Index paths are recorded from the working tree root,
    ///  so a scan of a subfolder drops both the prefix above the walked
    ///  directory and every entry outside it;
    ///  reporting them unchanged would scan files the caller did not ask for.
    pub(crate) fn below_walk_root<'entry>(&self, entry: &'entry [u8]) -> Option<&'entry [u8]> {
        if self.scope_below_root.is_empty() {
            return Some(entry)
        }
        return entry.strip_prefix(self.scope_below_root.as_slice())
    }
}

/// The object format the repository's configuration records:
///  `Sha256` only when `extensions.objectformat` names it,
///  `Sha1` otherwise.
///
/// `gix_index::File::at` parses entries by that format,
///  so a Sha1 kind on a Sha256 index lands every entry's flags and path
///  offsets twelve bytes off and the paths come out silently corrupted.
///  Reading the wrong configuration therefore corrupts paths rather than
///  failing.
pub(crate) fn index_hash_kind(config_path: &Path) -> gix_hash::Kind {
    let Ok(config) = std::fs::read_to_string(config_path) else {
        return gix_hash::Kind::Sha1
    };
    let mut in_extensions: bool = false;
    for raw in config.lines() {
        // Strip line comments (`# ...`, `; ...`) and surrounding whitespace; only the value before them is a signal.
        let line: &str = raw.split(['#', ';']).next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        // Section headers are `[section]` or `[section "subsection"]`; only the section name selects `objectformat`.
        if let Some(rest) = line.strip_prefix('[') {
            let section: &str = rest.trim_end_matches(']').trim();
            let name: &str = section.split_whitespace().next().unwrap_or("");
            in_extensions = name.eq_ignore_ascii_case("extensions");
            continue;
        }
        if !in_extensions {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("objectformat") && value.trim().eq_ignore_ascii_case("sha256") {
            return gix_hash::Kind::Sha256
        }
    }
    return gix_hash::Kind::Sha1
}

/// What one directory in the walk upward holds as a `.git` entry.
enum RootEntry {
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
fn git_directories(root: &Path) -> RootEntry {
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
            }
        }
    };
    if metadata.is_dir() {
        // A directory named `.git` is a repository only when it holds Git's
        // state; Git steps over anything else and keeps walking upward.
        if !is_git_directory(&dot_git) {
            return RootEntry::None
        }
        let git_dir: PathBuf = resolved_or_spelled(&dot_git);
        let common_dir: PathBuf = common_dir_of(&git_dir);
        return RootEntry::Tree { git_dir, common_dir }
    }
    if !metadata.is_file() {
        return RootEntry::Unusable {
            entry: dot_git,
            cause: "it is not a regular file".to_string(),
        }
    }
    let contents: Vec<u8> = match read_git_file(&dot_git) {
        Ok(contents) => contents,
        Err(cause) => return RootEntry::Unusable { entry: dot_git, cause },
    };
    let named: PathBuf = match gitdir_from_git_file(&contents) {
        Ok(named) => named,
        Err(cause) => return RootEntry::Unusable { entry: dot_git, cause },
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
            cause: format!("it names {}, which is not a Git directory", git_dir.display()),
        }
    }
    let resolved_git_dir: PathBuf = resolved_or_spelled(&git_dir);
    let common_dir: PathBuf = common_dir_of(&resolved_git_dir);
    return RootEntry::Tree { git_dir: resolved_git_dir, common_dir }
}

/// Whether `suspect` holds what Git's `is_git_directory` requires:
///  a `HEAD` naming a ref or an object id,
///  and `objects` and `refs` beside the shared state.
fn is_git_directory(suspect: &Path) -> bool {
    if !head_names_a_ref_or_object_id(&suspect.join("HEAD")) {
        return false
    }
    let common: PathBuf = common_dir_of(suspect);
    return common.join("objects").is_dir() && common.join("refs").is_dir()
}

/// Whether `head` is the file Git accepts as a repository's `HEAD`:
///  a symbolic ref into `refs/`,
/// or an object id.
fn head_names_a_ref_or_object_id(head: &Path) -> bool {
    let Ok(contents) = std::fs::read_to_string(head) else {
        return false
    };
    let line: &str = contents.lines().next().unwrap_or("").trim();
    if let Some(target) = line.strip_prefix("ref:") {
        return target.trim_start().starts_with("refs/")
    }
    // A detached HEAD holds one object id, forty bytes for Sha1 and sixty-four for Sha256.
    if !matches!(line.len(), 40 | 64) {
        return false
    }
    for byte in line.bytes() {
        if !byte.is_ascii_hexdigit() {
            return false
        }
    }
    return true
}

/// What an operating system refusal of a `.git` file means for a scan,
///  in one phrase.
fn git_file_unreadable(error: std::io::Error) -> String {
    return format!("it cannot be read ({})", error)
}

/// The bytes of a `.git` file,
/// bounded by Git's own ceiling.
fn read_git_file(path: &Path) -> Result<Vec<u8>, String> {
    let metadata: std::fs::Metadata = std::fs::metadata(path).map_err(git_file_unreadable)?;
    if metadata.len() > GIT_FILE_MAX_BYTES {
        return Err("it is too large to be a .git file".to_string())
    }
    return std::fs::read(path).map_err(git_file_unreadable)
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
        return Err("it does not start with `gitdir: `".to_string())
    };
    if named.is_empty() {
        return Err("it names no path".to_string())
    }
    let Ok(text) = std::str::from_utf8(named) else {
        return Err("the path it names is not UTF-8".to_string())
    };
    return Ok(PathBuf::from(text))
}

/// The directory holding the repository's shared state,
///  from `<git_dir>/commondir` when a linked worktree has one,
///  and `git_dir` itself otherwise.
///
/// Git strips every trailing line break there too,
///  and resolves a relative entry against the Git directory.
fn common_dir_of(git_dir: &Path) -> PathBuf {
    let Ok(contents) = std::fs::read(git_dir.join("commondir")) else {
        return git_dir.to_path_buf()
    };
    let mut line: &[u8] = &contents;
    while let [rest @ .., last] = line {
        if !TRAILING_LINE_BYTES.contains(last) {
            break;
        }
        line = rest;
    }
    let Ok(text) = std::str::from_utf8(line) else {
        return git_dir.to_path_buf()
    };
    return resolved_or_spelled(&git_dir.join(text))
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
    }
}

/// The walked directory's path below the working tree root,
///  `/`-separated with a trailing separator,
///  empty when the two are the same directory.
fn scope_bytes(below: &Path) -> Vec<u8> {
    if below.as_os_str().is_empty() {
        return Vec::new()
    }
    let mut bytes: Vec<u8> = normalized_path(below);
    bytes.push(b'/');
    return bytes
}

/// The absolute form of `path`,
///  resolved against the current directory when relative,
///  with `.` and `..` dealt with lexically and no symlink followed.
fn absolute_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return Some(lexically_normalized(path))
    }
    let current: PathBuf = std::env::current_dir().ok()?;
    return Some(lexically_normalized(&current.join(path)))
}

/// `path` with `.` components dropped and `..` resolved against the
///  component before it,
///  touching no filesystem entry.
///
/// Every input is absolute,
///  so a `..` that climbs past the filesystem root is dropped,
///  which is what Git and the kernel do with such a path.
///  This resolves spelling only:
///  a `..` after a symlink names a different directory on disk,
///  so nothing that must match the kernel's own resolution is normalized
///  here,
/// and `gitdir:` and `commondir` paths keep their spelling.
pub(crate) fn lexically_normalized(path: &Path) -> PathBuf {
    let mut parts = path.components().peekable();
    // A volume prefix is the base a root component keeps: `PathBuf::push` of a
    // rooted path replaces everything but the prefix, so the prefix starts the
    // result and the root is pushed onto it.
    let mut normalized: PathBuf = match parts.peek() {
        Some(Component::Prefix(prefix)) => {
            let base: PathBuf = PathBuf::from(prefix.as_os_str());
            parts.next();
            base
        }
        _ => PathBuf::new(),
    };
    for part in parts {
        match part {
            Component::Prefix(_) => {}
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    return normalized
}

/// Registers the `tests` child module.
// What:     `#[cfg(test)] #[path = "git_working_tree_tests.rs"] mod tests;`
//           declares a test-only submodule whose code lives in the sibling
//           file, so it compiles only under `cargo nextest run` and still
//           reaches this file's private items through `super::`.
// Why:      Keeps the production file inside its line budget while the
//           fixtures stay beside the code they cover.
//
// In TS you'd write (pseudocode):
// ```ts
// // git-working-tree.unit.test.ts, run only by the test runner
// ```
#[cfg(test)]
#[path = "git_working_tree_tests.rs"]
mod tests;
