//! What:
//!  The Git directory Git gives the Git inside a `!` alias's shell as `GIT_DIR`,
//!  and the comparison that decides whether Git gives it one at all.
//! Why:
//!  Git stores the Git directory it found in the environment of the shell it starts for a `!` alias
//!  (`setup.c:1657` to `:1662`),
//!  so the nested `git -C <elsewhere>` keeps that repository while it moves the files,
//!  and the guard has to ask its questions in that state (issue 6641).
//!  The reading sits beside `alias_run_directory` in a file of its own,
//!  because the module that holds both would otherwise pass the linter's line limit.
//!  The three readers of a Git answer it stands on are the search's own,
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
///  Git exports `GIT_DIR` from `set_git_dir` (`setup.c:1657` to `:1662`),
///  which runs for a repository or a work tree the caller named,
///  for a bare repository,
///  for a place inside a Git directory,
///  and for a working tree whose `.git` is a file that points elsewhere
///  (`setup.c:1126` to `:1129`);
///  in a plain main checkout it runs for nothing,
///  and the nested Git finds that same repository by itself.
///  The nested `git` keeps the exported directory when its own words move it with `-C`,
///  so it writes that repository's index and `HEAD` while it writes the other directory's files,
///  and a guard that asked nothing about it measured the files alone (issue 6641's second class).
///  The answer is Git's own:
///  `rev-parse --git-dir` prints the value `set_git_dir` stored,
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
///   const exported = thePlaceWasNamed || bare === 'true' || top === undefined || !sameDirectory(`${top}/.git`, absolute);
///   return exported ? printed : undefined;
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
            (Some(top_of_the_working_tree), Some(git_dir)) => !the_same_directory(
                top_of_the_working_tree.join(".git").as_path(),
                git_dir.as_path(),
            ),
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
///  Whether two paths name the directory they resolve to,
///  links and a `..` included.
/// Why:
///  Git decides whether to export `GIT_DIR` by comparing the directory it found with the plain `.git`
///  at the top of the working tree (`setup.c:1126` to `:1129`),
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
