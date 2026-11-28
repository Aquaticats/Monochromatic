//! What:
//!  The Git directory Git gives the Git inside a `!` alias's shell as `GIT_DIR`,
//!  and the readings that decide whether Git gives it one at all.
//! Why:
//!  Git stores the Git directory it found in the environment of the shell it starts for a `!` alias
//!  (`setup.c:2063` to `:2071` of Git 2.56.1,
//!  through `apply_and_export_relative_gitdir`,
//!  which calls `xsetenv(GIT_DIR_ENVIRONMENT, path, 1)` at `setup.c:1102`),
//!  so the nested `git -C <elsewhere>` keeps that repository while it moves the files,
//!  and the guard has to ask its questions in that state (issue 6641).
//!  The reading sits beside `alias_run_directory` in a file of its own,
//!  because the module that holds both would otherwise pass the linter's line limit.
//!  The four readers of a Git answer it stands on are the search's own,
//!  which it shares inside the crate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { printedPath, runGit } from './external_git_command.ts';
//! ```

/// Debug diagnostics.
use crate::diagnostic_log::debug;
/// The reader that drops the newline an answer ends with.
use crate::external_git_command::without_record_terminator;
/// The printed-path reader and the record break the search beside this file holds.
use crate::external_git_command::{
    is_record_break,
    printed_path,
};
/// Running real Git,
///  for the two questions only Git answers.
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
/// `OsString` holds raw native text.
use std::ffi::OsString;
/// `Path`/`PathBuf` hold raw native paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The Git directory Git gives the Git inside a `!` alias's shell as `GIT_DIR`,
///  or nothing when it gives it none.
///  `&[OsString]` borrows the globals of the Git process that reached the alias;
///  `Option<&Path>` is the directory that process was started in,
///  the caller's `-C` options not yet applied;
///  `bool` says whether the caller's own options or environment named a Git directory or a work tree.
/// Why:
///  Git runs one export block for every repository it finds,
///  and leaves the Git directory out of the shell's environment only on one path through it
///  (`setup.c:2063` to `:2076` of Git 2.56.1):
///  `discovery.gitdir` is set for a place the caller named,
///  for a bare repository,
///  for a place inside a Git directory,
///  for a working tree whose `.git` is a file that points elsewhere,
///  and for a working tree whose configuration or environment names the work tree
///  (`setup.c:1247`),
///  and that block exports the directory it holds.
///  Only a plain main checkout whose `.git` is the directory Git found reaches that block with nothing in it,
///  and there the nested Git finds the same repository by itself.
///  The alpha.52 read's finding M1:
///  a main checkout whose own configuration sets `core.worktree`
///  takes the same path as a work tree named on the command line,
///  so Git exports the absolute Git directory from a subdirectory,
///  a nested `-C` keeps it,
///  and the nested Git writes that checkout's index and `HEAD` over the other directory's files.
///  The comparison below cannot see that one:
///  with `core.worktree` set to the checkout itself,
///  `<top>/.git` and `--absolute-git-dir` are the same directory,
///  so the shape is read by `the_repository_names_a_work_tree`.
///  The nested `git` keeps the exported directory when its own words move it with `-C`,
///  so it writes that repository's index and `HEAD` while it writes the other directory's files,
///  and a guard that asked nothing about it measured the files alone (issue 6641's second class).
///  The answer is Git's own:
///  `rev-parse --git-dir` prints the value Git stored,
///  which is what Git exports,
///  and `--absolute-git-dir` with `--show-toplevel` say whether that value is the plain `.git`
///  at the top of the working tree,
///  which discovery finds without being told.
/// A query that ends at `--show-toplevel`,
///  as it does inside a Git directory and in a bare repository,
///  still prints the answers before it (`worktree_identity.rs`'s `IDENTITY_QUERY`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDirTheAliasShellReads = ({ globals, start, thePlaceWasNamed }) => {
///   const [bare, printed, absolute, top] = git(start, [...globals, 'rev-parse', '--is-bare-repository', '--git-dir', '--absolute-git-dir', '--show-toplevel']);
///   const discovered = thePlaceWasNamed || bare === 'true' || top === undefined || !sameDirectory(`${top}/.git`, absolute);
///   return discovered || theRepositoryNamesAWorkTree(start, globals) ? printed : undefined;
/// };
/// ```
pub(crate) fn the_git_dir_the_alias_shell_reads(
    context: &GitContext,
    globals: &[OsString],
    current_directory: Option<&Path>,
    the_place_was_named: bool,
) -> Option<OsString> {
    let start: &Path = current_directory?;
    let mut arguments: Vec<OsString> = globals.to_vec();
    for option in [
        "rev-parse",
        "--is-bare-repository",
        "--git-dir",
        "--absolute-git-dir",
        "--show-toplevel",
    ] {
        arguments.push(OsString::from(option));
    }
    // Git's own `-C` options in the globals apply from the directory this process was started in and nowhere else.
    let mut request: GitRequest = GitRequest::new(start, arguments.as_slice());
    request.without_prefix = true;
    let Ok(output) = run_git(context, &request) else {
        debug(
            "gitDirTheAliasShellReads",
            "Git could not be asked what it gives an alias's shell",
        );
        return None;
    };
    let mut answers = output
        .stdout
        .split(is_record_break)
        .filter(|line: &&[u8]| return !line.is_empty());
    // A place outside every repository answers no Git directory at all, and Git gives the shell nothing.
    let bare: &[u8] = answers.next()?;
    let printed: &[u8] = answers.next()?;
    let absolute: Option<PathBuf> = answers.next().map(printed_path);
    // Git prints no top of a working tree where there is none, which is a bare repository and a place inside a Git
    //  directory, and there the alias runs where Git's options left it with the exported directory as its repository.
    let top: Option<PathBuf> = answers.next().map(printed_path);
    let exported: bool = the_place_was_named
        || bare == &b"true"[..]
        || match (top, absolute) {
            (Some(top_of_the_working_tree), Some(git_dir)) => {
                !the_same_directory(
                    top_of_the_working_tree.join(".git").as_path(),
                    git_dir.as_path(),
                ) || the_repository_names_a_work_tree(context, globals, start)
            }
            // An answer this reading cannot compare is one it cannot prove Git leaves alone, and a guard that guessed
            //  here would judge the files without the repository they belong to.
            _ => true,
        };
    if !exported {
        return None;
    }
    return Some(without_record_terminator(printed));
}

/// What:
///  Whether the repository's own configuration names the work tree it uses,
///  which makes Git store the Git directory it found and export it.
///  `&[OsString]` borrows the global options of the Git process that reached the alias,
///  and `&Path` the directory it was started in.
/// Why:
///  `repo_discover_implicit_gitdir` reads a discovered `.git` through the explicit path when the
///  repository's configuration sets `core.worktree` (`setup.c:1247` to `:1254`),
///  and that path is the one the export block names.
///  The value is asked of Git rather than read from a file,
///  because three files can set it:
///  the repository's `config` (`setup.c:585` to `:597`),
///  its `config.worktree` when `extensions.worktreeConfig` is on (`setup.c:786` to `:793`),
///  and neither is a place this crate reads for an answer Git already gives.
///  Only the two scopes that reach discovery count:
///  `-c core.worktree=...` on the command line is Git's `command` scope,
///  which discovery never reads (`read_and_verify_repository_format` reads the repository's own files),
///  so a reading that asked for the value alone would export a Git directory where Git exports none,
///  and pin a repository the nested Git's own `-C` would have moved away from.
///  A read that fails answers nothing to add:
///  the shape this asks about is one the comparison above has already read as the plain `.git` at the top,
///  and pinning a repository on an answer nobody read would judge a place the command may not act in.
/// Gotcha:
///  A `core.worktree` in a repository whose Git directory holds a `commondir` file is ignored by Git
///  (`setup.c:805` to `:812`),
///  and this reading answers that it is set.
///  The pinned Git directory is then the one discovery found,
///  so the facts name that same repository,
///  but a nested `-C` no longer moves it.
///  A `.git` that is a file never reaches this question:
///  its answer is never the plain `.git` directory the comparison looks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theRepositoryNamesAWorkTree = (start, globals) => git(start, [...globals, 'config', '--show-scope', '--get-all', 'core.worktree'])
///   .split('\n').some((line) => line.startsWith('local\t') || line.startsWith('worktree\t'));
/// ```
fn the_repository_names_a_work_tree(
    context: &GitContext,
    globals: &[OsString],
    start: &Path,
) -> bool {
    let mut arguments: Vec<OsString> = globals.to_vec();
    for option in ["config", "--show-scope", "--get-all", "core.worktree"] {
        arguments.push(OsString::from(option));
    }
    let mut request: GitRequest = GitRequest::new(start, arguments.as_slice());
    request.without_prefix = true;
    let output = match run_git(context, &request) {
        Ok(answered) => answered,
        Err(_) => {
            debug(
                "theRepositoryNamesAWorkTree",
                "Git could not be asked whether this repository names its own work tree",
            );
            return false;
        }
    };
    // A value the repository's own configuration does not set exits non-zero and prints nothing, which is no export.
    return output.stdout.split(is_record_break).any(|line: &[u8]| {
        return line.starts_with(LOCAL_SCOPE) || line.starts_with(WORKTREE_SCOPE);
    });
}

/// The scope `git config --show-scope` prints for a value the repository's own `config` sets,
///  before the TAB that separates it from the value.
const LOCAL_SCOPE: &[u8] = b"local\t";

/// The scope `git config --show-scope` prints for a value `config.worktree` sets,
///  before the TAB that separates it from the value.
const WORKTREE_SCOPE: &[u8] = b"worktree\t";

/// What:
///  Whether two paths name the directory they resolve to,
///  links and a `..` included.
/// Why:
///  Git decides whether to export `GIT_DIR` by comparing the directory it found with the plain `.git`
///  at the top of the working tree (`setup.c:1238` to `:1276`),
///  and `.git` is a directory in a main checkout and a file pointing elsewhere in a linked worktree,
///  so the two answers have to be resolved before they are compared.
///  A path that does not resolve is compared as it stands:
///  it is not the `.git` directory Git would have found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameDirectory = (left, right) => realpath(left) === realpath(right);
/// ```
fn the_same_directory(
    left: &Path,
    right: &Path,
) -> bool {
    let resolved_left: PathBuf =
        std::fs::canonicalize(left).unwrap_or_else(|_| return left.to_path_buf());
    let resolved_right: PathBuf =
        std::fs::canonicalize(right).unwrap_or_else(|_| return right.to_path_buf());
    return resolved_left == resolved_right;
}
