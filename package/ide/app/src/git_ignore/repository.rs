//! Find the git work tree above a directory without running git.

/// Only metadata calls and one small text read per `.git` file.
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

/// What:
///  The nearest work tree above a directory:
///  its top directory,
///  and the directory that holds its `info/exclude`.
/// Why:
///  Git applies `.gitignore` files from the top of the work tree down,
///  and the shared `info/exclude` of a linked work tree lives in the common directory,
///  not in the work tree's own git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Repository = { top: string; commonDir: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    /// The directory that holds `.git`:
    ///  `.gitignore` files above it do not apply,
    ///  as in git.
    pub top: PathBuf,
    /// The directory whose `info/exclude` applies;
    ///  `top/.git` for an ordinary repository.
    pub common_dir: PathBuf,
}

/// The first line of `text` without surrounding blanks,
///  or `None` for empty text.
fn trimmed_first_line(text: &str) -> Option<String> {
    let line = text.lines().next()?;
    return Some(line.trim().to_string());
}

/// First line of a text file without its line break,
///  or `None` when the file is missing or unreadable (logged unless it is missing).
fn first_line(path: &Path) -> Option<String> {
    return match fs::read_to_string(path) {
        Ok(text) => trimmed_first_line(&text),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => {
            tracing::debug!(path = %path.display(), %error, "cannot read a git pointer file");
            None
        }
    };
}

/// Resolve the directory that holds the shared `info/exclude`,
///  given the work tree's `.git` entry.
/// A `.git` directory is its own common directory;
///  a `.git` file (linked work tree or submodule) names the git directory with a `gitdir:` line,
///  resolved against the file's directory,
///  and that git directory may name a common directory in `commondir`,
///  resolved against itself.
fn common_dir(top: &Path, dot_git: &Path, is_file: bool) -> PathBuf {
    if !is_file {
        return dot_git.to_path_buf();
    }
    let Some(line) = first_line(dot_git) else {
        return dot_git.to_path_buf();
    };
    let Some(target) = line.strip_prefix("gitdir:") else {
        tracing::debug!(path = %dot_git.display(), "the .git file names no gitdir; its info/exclude is not read");
        return dot_git.to_path_buf();
    };
    // join keeps an absolute target as it is and resolves a relative one against the work tree.
    let git_dir = top.join(target.trim());
    return match first_line(&git_dir.join("commondir")) {
        Some(common) => git_dir.join(common),
        None => git_dir,
    };
}

/// Walk up from `directory` (included) to the nearest directory holding `.git`,
///  a directory or a file,
///  as git's own discovery does without a ceiling.
/// `None` means there is no work tree above:
///  git would report no repository.
pub fn find(directory: &Path) -> Option<Repository> {
    for candidate in directory.ancestors() {
        let dot_git = candidate.join(".git");
        match fs::metadata(&dot_git) {
            Ok(metadata) => {
                return Some(Repository {
                    top: candidate.to_path_buf(),
                    common_dir: common_dir(candidate, &dot_git, metadata.is_file()),
                });
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                tracing::debug!(path = %dot_git.display(), %error, "cannot inspect .git while looking for the work tree");
            }
        }
    }
    return None;
}
