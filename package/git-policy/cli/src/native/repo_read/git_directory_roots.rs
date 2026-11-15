//! What:
//!  The repository a command works on,
//! resolved the way Git resolves it and without starting Git:
//! the directory its `-C` chain ends in,
//! the Git directory its options or environment name,
//! and the common Git directory the locks and the record folder live in.
//! Why:
//!  The lock pass, the record of the processes a command starts, the reference-lock clearing before a forwarded
//!  `fetch` or `push` and the sweep of a stopped child's locks all ask one question:
//!  which repository does this command work on.
//!  Two answers to it disagree, and a command then clears a lock in one repository and records its Git in another,
//!  or looks in a directory Git never uses (a review of `2af2ecb879`, finding 1).
//!  Git is not asked,
//! because the answer is needed when Git is about to be started or has just failed,
//! and reading a few small files costs nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const repository = repositoryACommandWorksOn({ cwd, globalPrefix, environment }); const roots = repository.roots();
//! ```

/// Owned OS strings of a command's environment.
use std::ffi::{OsStr, OsString};
/// Paths of the repository.
use std::path::{Path, PathBuf};
/// The command whose repository is wanted.
use std::process::Command;
/// The one walk of the global options that answers the directory a `-C` chain ends in and the Git directory a
///  `--git-dir` names, so neither is resolved twice.
use crate::repository_link_paths::{GIT_COMMON_DIRECTORY_VARIABLE, GIT_DIRECTORY_VARIABLE, chain_directory, last_git_dir_option};

/// The common Git directory a linked worktree's Git directory names in its `commondir` file,
/// or the directory itself.
///  The few lines are repeated here because the module that reads it belongs to the service run,
/// which the client path
///  may not name (`standards_review_boundary_tests.rs`);
/// a path that cannot be resolved is used as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync(join(gitDir, 'commondir')) ? realpath(resolve(gitDir, readFile(...).trimEnd())) : gitDir
/// ```
pub fn common_dir_for(git_dir: &Path) -> PathBuf {
    let Ok(text) = std::fs::read_to_string(git_dir.join("commondir")) else {
        return git_dir.to_path_buf();
    };
    let named: &str = text.trim_end_matches(['\r', '\n']);
    if named.is_empty() {
        return git_dir.to_path_buf();
    }
    let joined: PathBuf = git_dir.join(named);
    return std::fs::canonicalize(&joined).unwrap_or(joined);
}

/// Where the Git directory of a working directory is:
/// `.git` as a directory,
/// a `gitdir:` file,
/// or a bare repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitDirectoryOf(start: string): string | undefined;
/// ```
pub fn git_directory_of(start: &Path) -> Option<PathBuf> {
    for directory in start.ancestors() {
        let dot_git: PathBuf = directory.join(".git");
        if let Ok(metadata) = std::fs::symlink_metadata(dot_git.as_path()) {
            if metadata.is_dir() {
                return Some(dot_git);
            }
            if metadata.is_file() {
                let text: String = std::fs::read_to_string(dot_git.as_path()).ok()?;
                let named: &str = text.lines().next()?.strip_prefix("gitdir:")?.trim();
                return Some(directory.join(named));
            }
        }
        if directory.join("HEAD").is_file() && directory.join("objects").is_dir() {
            return Some(directory.to_path_buf());
        }
    }
    return None;
}

/// The Git directory and the common Git directory (when it differs) of a working directory,
/// or of `GIT_DIR` when one is
///  named;
/// nothing when no repository is found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsFor(working: string, named?: string): string[];
/// ```
pub fn roots_for(working: &Path, named: Option<PathBuf>) -> Vec<PathBuf> {
    let git_dir: Option<PathBuf> = match named {
        Some(explicit) => Some(explicit),
        None => git_directory_of(working),
    };
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(directory) = git_dir {
        let common: PathBuf = common_dir_for(directory.as_path());
        roots.push(directory.clone());
        if common != directory {
            roots.push(common);
        }
    }
    return roots;
}

/// What:
///  The repository a command works on:
///  the directory it runs in after its `-C` chain,
///  the Git directory it works in,
///  and the common Git directory its locks and its record folder live in.
/// Why:
///  One value answers every reader that needs to know which repository an invocation works on,
///  so no two of them can disagree about a relative `-C`,
///  a named Git directory or a named common directory.
///  `git_dir` and `common_dir` are `None` when the command works outside a repository,
///  which a pass then costs one failed look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommandRepository = { start: string; gitDir?: string; commonDir?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandRepository {
    /// The directory the command runs in,
    ///  after every `-C` it was given.
    pub start: PathBuf,
    /// The Git directory it works in,
    ///  when one was named or found.
    pub git_dir: Option<PathBuf>,
    /// The common Git directory,
    ///  where a linked worktree's shared references and the record folder live.
    pub common_dir: Option<PathBuf>,
}

impl CommandRepository {
    /// What:
    ///  The Git directories a lock pass searches:
    ///  the Git directory,
    ///  and the common one when it differs.
    /// Why:
    ///  A lock of the worktree's state sits in the first,
    ///  and the shared references and a linked worktree's locks in the second,
    ///  so a pass that read only one would miss what the other holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// repository.roots() // ['/repo/.git'] or ['/repo/.git', '/repo/.git']
    /// ```
    pub fn roots(&self) -> Vec<PathBuf> {
        let Some(git_dir) = self.git_dir.as_ref() else {
            return Vec::new();
        };
        let mut roots: Vec<PathBuf> = vec![git_dir.clone()];
        if let Some(common) = self.common_dir.as_ref()
            && common != git_dir
        {
            roots.push(common.clone());
        }
        return roots;
    }
}

/// What:
///  The repository a command works on,
///  from the directory it runs in,
///  the global options it was given,
///  and its environment.
///  `&[OsString]` borrows the tokens before the subcommand.
/// Why:
///  This is the one answer to "which repository does this invocation work on",
///  resolved the way Git resolves it:
///
///  - the `-C` chain moves the directory,
///    each link from the one before,
///    and only it moves that directory (`--work-tree` does not);
///  - the last `--git-dir` wins over `GIT_DIR`,
///    and a relative one is read from the directory the chain ends in;
///  - `GIT_COMMON_DIR` names the common directory,
///    read from that same directory when it is relative,
///    and it wins over the `commondir` file a linked worktree carries;
///  - without a named Git directory the repository is found by walking up from that directory,
///    which reads a few small files and starts no Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryACommandWorksOn({ cwd, globalPrefix: ['-C', 'sub', '--git-dir=../cfg.git'], environment })
/// // { start: '/w/sub', gitDir: '/w/sub/../cfg.git', commonDir: '/w/sub/../cfg.git' }
/// ```
pub fn repository_a_command_works_on(cwd: &Path, global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> CommandRepository {
    let start: PathBuf = chain_directory(cwd, global_prefix);
    let named: Option<PathBuf> = last_git_dir_option(global_prefix)
        .or_else(|| { return environment_value(environment, GIT_DIRECTORY_VARIABLE).map(PathBuf::from); });
    let git_dir: Option<PathBuf> = match named {
        Some(value) => Some(start.join(value)),
        None => git_directory_of(start.as_path()),
    };
    let Some(git_dir_found) = git_dir.as_ref() else {
        return CommandRepository { start, git_dir: None, common_dir: None };
    };
    let common_dir: PathBuf = match environment_value(environment, GIT_COMMON_DIRECTORY_VARIABLE) {
        Some(value) => start.join(value),
        None => common_dir_for(git_dir_found.as_path()),
    };
    return CommandRepository { start, git_dir: Some(git_dir_found.clone()), common_dir: Some(common_dir) };
}

/// What:
///  The last value an environment list carries for `name`,
///  or nothing when it carries none or an empty one.
/// Why:
///   A command's environment is its own list,
///  and Git reads the last assignment of a variable it is started with;
///  an empty value names nothing, as an empty `GIT_DIR` names no Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// environmentValue([['GIT_DIR', '/g']], 'GIT_DIR') // '/g'
/// ```
fn environment_value<'a>(environment: &'a [(OsString, OsString)], name: &str) -> Option<&'a OsStr> {
    let mut found: Option<&'a OsStr> = None;
    for (variable, value) in environment {
        if variable == OsStr::new(name) && !value.is_empty() {
            found = Some(value.as_os_str());
        }
    }
    return found;
}

/// What:
///  The roots of a command that has not started yet:
///  the Git directories of the repository it will work in.
/// Why:
///   Such a command names its repository the way any invocation does,
///  through its working directory and its environment,
///  so it is resolved by the one resolver every other reader uses:
///  `GIT_DIR` names its Git directory and `GIT_COMMON_DIR` its common one,
///  and a variable the command removes names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsOfCommand(command: Command): string[];
/// ```
pub fn roots_of_command(command: &Command) -> Vec<PathBuf> {
    let working: PathBuf = match command.get_current_dir() {
        Some(directory) => directory.to_path_buf(),
        None => std::env::current_dir().unwrap_or_default(),
    };
    // Only what the command sets: a removal carries no value and names nothing.
    let mut environment: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in command.get_envs() {
        let Some(value) = value else {
            continue;
        };
        environment.push((name.to_os_string(), value.to_os_string()));
    }
    return repository_a_command_works_on(working.as_path(), &[], environment.as_slice()).roots();
}
