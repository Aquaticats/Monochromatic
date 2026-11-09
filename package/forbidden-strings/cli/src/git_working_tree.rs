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
//!  so every case finds its index by asking Git's own discovery rules from
//!  the walked directory,
//! never by joining `.git/index` onto it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const tree = discoverWorkingTree(walkRoot); // null when there is no repository
//! // const index = openIndex(`${tree.gitDir}/index`, tree.hashKind());
//! ```

/// Imports the path component type that lexical normalization classifies,
///  and the two path types every caller passes.
use std::path::{Component, Path, PathBuf};

/// Imports the platform-aware separator normalization,
///  so one scope prefix compares against Git's always-`/`-separated index paths.
use crate::path_name_bytes::normalized_path;

/// The most a `.git` file may hold,
/// from Git's own `read_gitfile_gently`
///  (`max_file_size = 1 << 20` in `setup.c`):
///  Git refuses a bigger one,
///  so a bigger one is not a Git file and no repository is named by it.
const GIT_FILE_MAX_BYTES: u64 = 1 << 20;

/// The exact first bytes of a `.git` file,
/// from Git's `read_gitfile_gently`
///  (`memcmp(url, "gitdir: ", 8)`),
/// space included.
const GITDIR_PREFIX: &[u8] = b"gitdir: ";

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
///  A directory with no such ancestor is not inside a repository and
///  returns `None`.
pub(crate) fn discover(walk_root: &Path) -> Option<GitWorkingTree> {
    let absolute: PathBuf = absolute_path(walk_root)?;
    for candidate in absolute.ancestors() {
        let Some((git_dir, common_dir)) = git_directories(candidate) else {
            continue;
        };
        let below: &Path = absolute.strip_prefix(candidate).unwrap_or(Path::new(""));
        return Some(GitWorkingTree {
            git_dir,
            common_dir,
            scope_below_root: scope_bytes(below),
        })
    }
    return None
}

/// What this working tree's index is asked for,
///  and how its entries are read.
impl GitWorkingTree {
    /// The path `gix-index` opens.
    pub(crate) fn index_path(&self) -> PathBuf {
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

/// The Git directory of the working tree rooted at `root`,
///  with the directory holding the shared configuration,
///  or `None` when `root` itself is not a working tree root.
fn git_directories(root: &Path) -> Option<(PathBuf, PathBuf)> {
    let dot_git: PathBuf = root.join(".git");
    let metadata: std::fs::Metadata = std::fs::metadata(&dot_git).ok()?;
    if metadata.is_dir() {
        let git_dir: PathBuf = lexically_normalized(&dot_git);
        return Some((git_dir.clone(), git_dir))
    }
    if !metadata.is_file() {
        return None
    }
    let contents: Vec<u8> = read_git_file(&dot_git)?;
    let named: PathBuf = gitdir_from_git_file(&contents)?;
    let git_dir: PathBuf = if named.is_absolute() {
        lexically_normalized(&named)
    } else {
        // Git resolves a relative `gitdir:` against the directory holding the `.git` file, which is `root`.
        lexically_normalized(&root.join(named))
    };
    let common_dir: PathBuf = common_dir_of(&git_dir);
    return Some((git_dir, common_dir))
}

/// The bytes of a `.git` file,
/// bounded by Git's own ceiling.
fn read_git_file(path: &Path) -> Option<Vec<u8>> {
    let metadata: std::fs::Metadata = std::fs::metadata(path).ok()?;
    if metadata.len() >= GIT_FILE_MAX_BYTES {
        return None
    }
    return std::fs::read(path).ok()
}

/// The Git directory a `.git` file's own contents name,
///  before a relative one is resolved against its working tree.
///
/// Git strips exactly one trailing newline and requires the `gitdir: `
///  prefix,
///  so a newline inside the named path stays part of that path,
///  and a file without the prefix is not a Git file at all.
pub(crate) fn gitdir_from_git_file(contents: &[u8]) -> Option<PathBuf> {
    let line: &[u8] = contents.strip_suffix(b"\n").unwrap_or(contents);
    let named: &[u8] = line.strip_prefix(GITDIR_PREFIX)?;
    if named.is_empty() {
        return None
    }
    let text: &str = std::str::from_utf8(named).ok()?;
    return Some(PathBuf::from(text))
}

/// The directory holding the repository's shared state,
///  from `<git_dir>/commondir` when a linked worktree has one,
///  and `git_dir` itself otherwise.
fn common_dir_of(git_dir: &Path) -> PathBuf {
    let Ok(contents) = std::fs::read(git_dir.join("commondir")) else {
        return git_dir.to_path_buf()
    };
    let line: &[u8] = contents.strip_suffix(b"\n").unwrap_or(&contents);
    let Ok(text) = std::str::from_utf8(line) else {
        return git_dir.to_path_buf()
    };
    return lexically_normalized(&git_dir.join(text))
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
/// Every input here is absolute,
///  so a `..` that climbs past the filesystem root is dropped,
///  which is what Git and the kernel do with such a path.
pub(crate) fn lexically_normalized(path: &Path) -> PathBuf {
    let mut normalized: PathBuf = PathBuf::new();
    for part in path.components() {
        match part {
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
