//! What:
//!  The global options that select the repository one invocation works on,
//!  as the arguments that name it by absolute paths.
//! Why:
//!  A commit is built and landed by several processes:
//!  the wrapper,
//!  the Git children it starts,
//!  and the resident service,
//!  which inherits neither the caller's options nor the caller's variables.
//!  Git finds `<worktree>/.git` by itself,
//!  so every other layout has to be named:
//!  a bare repository beside its work tree,
//!  a linked worktree's own directory,
//!  a Git directory an environment variable sends elsewhere.
//!  The selection keeps the caller's own `--git-dir` and `--work-tree`,
//!  resolved from the directory the `-C` chain reached,
//!  as Git resolves them:
//!  `git.c` sets the variables while it reads the options,
//!  and a relative one is read when Git sets the repository up,
//!  after every `-C`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const selection = repositorySelection({ directory, globalPrefix }); // ['--git-dir', '/abs/cfg.git', '--work-tree', '/abs/wt']
//! ```

use crate::global_arguments::takes_value;
use crate::worktree_identity::{WorktreeIdentity, git_directory, worktree_root};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The spelling of the option that names the Git directory.
const GIT_DIR: &str = "--git-dir";
/// The spelling of the option that names the work tree.
const WORK_TREE: &str = "--work-tree";
/// The option that makes the directory Git runs in the Git directory,
/// and names no work tree.
const BARE: &[u8] = b"--bare";

/// What:
///  The value of an inline option (`--git-dir=<value>`),
///  or nothing when the token is not that option.
/// Why:
///   Git 2.56.0 takes both spellings,
///  and a rule that read only the separated one would miss the one people write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inlineValue = (token: string, spelling: string) => token.startsWith(spelling + '=') ? token.slice(spelling.length + 1) : undefined;
/// ```
fn inline_value(token: &[u8], spelling: &str) -> Option<OsString> {
    let value: &[u8] = token.strip_prefix(spelling.as_bytes())?.strip_prefix(b"=")?;
    #[cfg(unix)]
    {
        /// The Unix byte-to-native-string conversion for a configured path.
        use std::os::unix::ffi::OsStrExt;
        return Some(OsStr::from_bytes(value).to_os_string());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return Some(OsString::from(String::from_utf8_lossy(value).into_owned()));
    }
}

/// What:
///  The repository a global prefix selects,
///  as `--git-dir <absolute>` and `--work-tree <absolute>`,
///  or nothing when the prefix names no repository.
/// Why:
///   A commit's own Git children run somewhere other than the caller's directory,
///  and a request that keeps none of the caller's other options still needs the repository the caller named.
///  The last `--git-dir` (or `--bare`) wins and so does the last `--work-tree`,
///  as with Git,
///  and both are read from the directory the `-C` chain ends in,
///  because Git resolves them after its last `-C`.
///  `directory` must therefore be the directory the command runs in,
///  before any `-C`:
///  handing this function a directory Git discovered instead applies the chain a second time
///  and names a repository that does not exist
///  (`of_identity`,
///  which is what a rule that starts Git elsewhere wants).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fromGlobalOptions({ directory, globalPrefix }): string[] {
///   let ends = directory; let gitDir; let workTree;
///   for (let i = 0; i < globalPrefix.length; i += 1) {
///     const token = globalPrefix[i];
///     if (token === '-C') { if (globalPrefix[i + 1] && globalPrefix[i + 1] !== '') ends = join(ends, globalPrefix[i + 1]); i += 1; continue; }
///     if (token === '--bare') { gitDir = '.'; continue; }
///     if (token === '--git-dir' || token === '--work-tree') { if (token === '--git-dir') gitDir = globalPrefix[i + 1]; else workTree = globalPrefix[i + 1]; i += 1; continue; }
///     if (token.startsWith('--git-dir=')) { gitDir = token.slice('--git-dir='.length); continue; }
///     if (token.startsWith('--work-tree=')) { workTree = token.slice('--work-tree='.length); continue; }
///     if (takesValue(token)) i += 1;
///   }
///   const named = [['--git-dir', gitDir], ['--work-tree', workTree]].filter(([, value]) => value !== undefined);
///   return named.flatMap(([spelling, value]) => [spelling, join(ends, value)]);
/// }
/// ```
pub fn from_global_options(directory: &Path, global_prefix: &[OsString]) -> Vec<OsString> {
    let mut ends: PathBuf = directory.to_path_buf();
    let mut git_dir: Option<OsString> = None;
    let mut work_tree: Option<OsString> = None;
    let mut index: usize = 0;
    while index < global_prefix.len() {
        let token: &OsString = &global_prefix[index];
        let argument: &[u8] = token.as_encoded_bytes();
        if argument == b"-C" {
            // An empty `-C` moves Git nowhere, and a `-C` with no value is Git's own error to word.
            if let Some(value) = global_prefix.get(index + 1)
                && !value.is_empty()
            {
                ends = ends.join(value);
            }
            index += 2;
            continue;
        }
        if argument == BARE {
            // `--bare` is Git's `GIT_DIR=.` (`git.c`, `handle_options`), read from the directory the chain ends in.
            git_dir = Some(OsString::from("."));
            index += 1;
            continue;
        }
        if let Some(value) = inline_value(argument, GIT_DIR) {
            git_dir = Some(value);
            index += 1;
            continue;
        }
        if let Some(value) = inline_value(argument, WORK_TREE) {
            work_tree = Some(value);
            index += 1;
            continue;
        }
        if (argument == GIT_DIR.as_bytes() || argument == WORK_TREE.as_bytes()) && index + 1 < global_prefix.len() {
            let value: OsString = global_prefix[index + 1].clone();
            if argument == GIT_DIR.as_bytes() {
                git_dir = Some(value);
            } else {
                work_tree = Some(value);
            }
            index += 2;
            continue;
        }
        index += if takes_value(argument) { 2 } else { 1 };
    }
    let mut selection: Vec<OsString> = Vec::new();
    for (spelling, named) in [(GIT_DIR, git_dir), (WORK_TREE, work_tree)] {
        let Some(value) = named else { continue };
        selection.push(OsString::from(spelling));
        selection.push(ends.join(value).into_os_string());
    }
    return selection;
}

/// What:
///  The arguments that name one repository by absolute paths,
///  or nothing when Git finds it from `work_tree` itself.
/// Why:
///   A landing runs in the work tree the request names,
///  and the resident service keeps none of the caller's options.
///  A plain layout needs no argument:
///  naming it would make every in-process read of the landing start Git instead of answering,
///  which is time the person waits for.
///  Every other layout,
///  `<work_tree>/.git` not being the Git directory itself,
///  is named so the landing reads and writes the repository the commit was prepared in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ofPaths = ({ gitDir, workTree }) => dirname(gitDir) === workTree && basename(gitDir) === '.git' ? [] : ['--git-dir', gitDir, '--work-tree', workTree];
/// ```
pub fn of_paths(git_dir: &Path, work_tree: &Path) -> Vec<OsString> {
    let plain: bool = git_dir.parent() == Some(work_tree) && git_dir.file_name() == Some(OsStr::new(".git"));
    if plain {
        return Vec::new();
    }
    return vec![
        OsString::from(GIT_DIR),
        git_dir.as_os_str().to_os_string(),
        OsString::from(WORK_TREE),
        work_tree.as_os_str().to_os_string(),
    ];
}

/// What:
///  The repository a located invocation is in,
///  as `--git-dir <absolute>` and `--work-tree <absolute>`,
///  or nothing for a plain `<worktree>/.git` layout Git finds by itself
///  and for an invocation in no work tree.
/// Why:
///   A rule that starts Git somewhere other than the caller's own directory cannot re-read the caller's options:
///  Git resolves a relative `--git-dir` after its last `-C`,
///  so resolving that value again from a directory Git discovered applies the directory change a second time
///  and names another repository.
///  The location Git answered already holds both directory names,
///  resolved once,
///  so this names the repository the command line selected without re-deriving it from tokens whose reading is Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ofIdentity = (identity) => 'worktreeRoot' in identity ? ofPaths(identity.gitDir, identity.worktreeRoot) : [];
/// ```
pub fn of_identity(identity: &WorktreeIdentity) -> Vec<OsString> {
    let (Some(git_dir), Some(work_tree)) = (git_directory(identity), worktree_root(identity)) else {
        return Vec::new();
    };
    return of_paths(git_dir, work_tree);
}

/// What:
///  The tokens of a global prefix without the options that select a repository or move Git through directories.
/// Why:
///   Native `git commit` in the shadow repository is given the caller's kept options,
///  and the shadow is a repository the wrapper names itself:
///  a `-C` would move it after the wrapper placed it,
///  and the caller's `--git-dir`,
///  `--work-tree` or `--bare` would name the real repository beside the shadow's own.
///  Configuration options (`-c`,
///  `--config-env`) stay:
///  they are the person's settings for the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutRepositorySelection = (prefix: string[]) => prefix.filter((token, index) => !['-C', '--bare', '--git-dir', '--work-tree'].includes(token) && !['-C', '--git-dir', '--work-tree'].includes(prefix[index - 1]) && !token.startsWith('--git-dir=') && !token.startsWith('--work-tree='));
/// ```
pub fn without_repository_selection(global_prefix: &[OsString]) -> Vec<OsString> {
    let mut kept: Vec<OsString> = Vec::new();
    let mut index: usize = 0;
    while index < global_prefix.len() {
        let token: &OsString = &global_prefix[index];
        let argument: &[u8] = token.as_encoded_bytes();
        // The options that name a repository, with the value Git reads after them: Git would read them instead of the shadow.
        if argument == b"-C" || argument == GIT_DIR.as_bytes() || argument == WORK_TREE.as_bytes() {
            index += 2;
            continue;
        }
        if argument == BARE || inline_value(argument, GIT_DIR).is_some() || inline_value(argument, WORK_TREE).is_some() {
            index += 1;
            continue;
        }
        kept.push(token.clone());
        index += 1;
        // A value-taking option that stays keeps its value.
        if takes_value(argument)
            && let Some(value) = global_prefix.get(index)
        {
            kept.push(value.clone());
            index += 1;
        }
    }
    return kept;
}

/// Selection controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_selection_tests.rs"]
mod tests;
