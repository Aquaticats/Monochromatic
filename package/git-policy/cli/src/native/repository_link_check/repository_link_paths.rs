//! What:
//!  The paths an invocation names,
//! and whether one lies under the repository link.
//! Why:
//!  The test is on the path the wrapper finally uses,
//! not on the text of an argument.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const touches = paths.some((path) => underRepository(identity, resolve(path)));
//! ```

use crate::repository_link_identity::LinkIdentity;
use crate::global_arguments::VALUE_OPTIONS;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

/// The environment variables that name a Git directory,
/// a work tree or the common Git directory.
const GIT_PATH_VARIABLES: [&str; 3] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR"];

/// What:
///  The text of an option value that names a path,
/// and where in the prefix the next option starts.
/// Why:
///  `--git-dir` and `--work-tree` come as `--name=value` or `--name value`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// optionValue(prefix, index, '--git-dir') // [value, nextIndex] or undefined
/// ```
fn path_option_value(prefix: &[OsString], index: usize, name: &str) -> Option<(PathBuf, usize)> {
    let token: &[u8] = prefix[index].as_encoded_bytes();
    let inline: String = format!("{name}=");
    if let Some(value) = token.strip_prefix(inline.as_bytes()) {
        return Some((PathBuf::from(OsStr::from_bytes(value)), index + 1));
    }
    if token == name.as_bytes() && index + 1 < prefix.len() {
        return Some((PathBuf::from(prefix[index + 1].clone()), index + 2));
    }
    return None;
}

/// What:
///  The directory a command runs in and the Git directory,
///  work tree and common directory its global
///  options name,
///  before the ones its environment adds.
///  `&[OsString]` borrows the tokens before the subcommand;
///  `(PathBuf, Vec<PathBuf>)` is that directory and the named ones.
/// Why:
///   One walk answers both:
///  a `-C` moves the directory for every question after it,
///  and a
///  relative Git directory or work tree is from the directory the chain ends in.
///  A value
///  that takes an argument is stepped over,
///  so a path is never read out of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prefixPaths({ cwd, globalPrefix }) // { directory: '/a/b/c/../d', options: ['/g/dir', '/a/b/c/../d/tree'] }
/// ```
fn prefix_paths(cwd: &Path, global_prefix: &[OsString]) -> (PathBuf, Vec<PathBuf>) {
    let mut directory: PathBuf = cwd.to_path_buf();
    let mut options: Vec<PathBuf> = Vec::new();
    let mut index: usize = 0;
    while index < global_prefix.len() {
        let token: &OsString = &global_prefix[index];
        if token.as_encoded_bytes() == b"-C" && index + 1 < global_prefix.len() {
            directory = directory.join(&global_prefix[index + 1]);
            index += 2;
            continue;
        }
        if let Some((value, next)) = path_option_value(global_prefix, index, "--git-dir") {
            options.push(value);
            index = next;
            continue;
        }
        if let Some((value, next)) = path_option_value(global_prefix, index, "--work-tree") {
            options.push(value);
            index = next;
            continue;
        }
        let takes_value: bool = VALUE_OPTIONS.contains(&token.as_encoded_bytes());
        index += if takes_value { 2 } else { 1 };
    }
    return (directory, options);
}

/// What:
///  The directory a command runs in,
///  after the caller's `-C` chain:
///  the current directory when the chain is empty.
///  `&[OsString]` borrows the tokens before the subcommand;
///  `PathBuf` is the owned directory.
/// Why:
///   A relative path an option names is read from where the command runs,
///  and `--work-tree` does not move that directory;
///  Git moves it only for `-C`,
///  one link of the chain at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chainDirectory('/work/here', ['-C', '/a/b', '-C', 'c']) // '/a/b/c'
/// ```
pub fn chain_directory(cwd: &Path, global_prefix: &[OsString]) -> PathBuf {
    return prefix_paths(cwd, global_prefix).0;
}

/// What:
///  The directories an invocation names:
/// the directory its `-C` chain ends in (the current directory without one),
/// and the Git directory,
/// work tree and common directory its options and environment name.
/// Why:
///  The check is on the path the wrapper finally uses,
/// so a command that spells the repository by its physical path,
/// through `GIT_DIR`,
/// or through `--work-tree` is checked as well.
/// `-C` paths chain the way Git chains them (a relative one from the one before),
/// and a relative Git directory or work tree is from the directory the chain ends in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namedPaths({ cwd, globalPrefix, environment }) // ['/work/here', '/g/dir']
/// ```
pub fn named_paths(cwd: &Path, global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> Vec<PathBuf> {
    let (directory, mut options): (PathBuf, Vec<PathBuf>) = prefix_paths(cwd, global_prefix);
    for (name, value) in environment {
        if GIT_PATH_VARIABLES.contains(&name.to_string_lossy().as_ref()) && !value.is_empty() {
            options.push(PathBuf::from(value));
        }
    }
    let mut paths: Vec<PathBuf> = vec![directory.clone()];
    for option in options {
        paths.push(directory.join(option));
    }
    return paths;
}

/// What:
///  The path with every link resolved as far as it exists,
/// and the part that does not exist appended with its `.` and `..` segments applied.
/// Why:
///  A link to a drive that is not mounted does not resolve,
/// but a path through it still lies under the repository;
/// its longest part that exists is resolved and the rest is kept as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolveAsFarAsItExists('/home/u/link/sub') // '/var/home/u/link/sub'
/// ```
pub(crate) fn resolve_as_far_as_it_exists(path: &Path) -> PathBuf {
    for candidate in path.ancestors() {
        let Ok(real) = std::fs::canonicalize(candidate) else {
            continue;
        };
        let Ok(rest) = path.strip_prefix(candidate) else {
            continue;
        };
        let mut resolved: PathBuf = real;
        for component in rest.components() {
            match component {
                Component::Normal(name) => resolved.push(name),
                Component::ParentDir => {
                    resolved.pop();
                }
                Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
            }
        }
        return resolved;
    }
    return path.to_path_buf();
}

/// What:
///  Whether a path lies under the repository:
/// under the link path (as the link is spelled,
/// with its parent resolved),
/// under the approved canonical root,
/// or under a sibling named like the link plus `.nvme-before-860-` (the old copy the move renames).
/// Why:
///  Decides whether the check runs for a command.
/// The identity file records no path of the old copy,
/// so its name is the one the cut-over gives it
/// (`doc/planning/repository-on-the-860-drive.md`,
/// phase 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// touchesRepository(identity, path)
/// ```
pub fn touches_repository(identity: &LinkIdentity, path: &Path) -> bool {
    if identity.link_path.is_empty() {
        return false;
    }
    let resolved: PathBuf = resolve_as_far_as_it_exists(path);
    let link: &Path = Path::new(identity.link_path.as_str());
    let (Some(parent), Some(name)) = (link.parent(), link.file_name()) else {
        return false;
    };
    let parent_resolved: PathBuf = resolve_as_far_as_it_exists(parent);
    let spelled: PathBuf = parent_resolved.join(name);
    if resolved.starts_with(spelled.as_path()) {
        return true;
    }
    if !identity.canonical_root.is_empty() && resolved.starts_with(resolve_as_far_as_it_exists(Path::new(identity.canonical_root.as_str()))) {
        return true;
    }
    let old_copy_prefix: String = format!("{}.nvme-before-860-", name.to_string_lossy());
    return match resolved.strip_prefix(parent_resolved.as_path()) {
        Ok(below) => match below.components().next() {
            Some(Component::Normal(first)) => first.to_string_lossy().starts_with(old_copy_prefix.as_str()),
            _ => false,
        },
        Err(_elsewhere) => false,
    };
}

/// Path controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_link_paths_tests.rs"]
mod tests;
