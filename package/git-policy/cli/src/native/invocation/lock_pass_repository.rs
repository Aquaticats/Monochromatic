//! What:
//!  The Git directory the lock pass searches:
//!  the repository the command itself will work in.
//! Why:
//!  The pass that clears the locks a killed Git left runs before the command reaches Git,
//!  so it must find the repository Git is about to use.
//!  Git selects it in one order:
//!  the leading `-C` chain moves the directory the command runs in,
//!  then `--git-dir` (or `--bare`) names the Git directory,
//!  then `GIT_DIR` does,
//!  and otherwise discovery walks up from that directory looking for `.git`.
//!  A command that names another repository with `-C` or `--git-dir` therefore leaves the locks of the
//!  directory it was started from alone,
//!  and the repository it names is repaired before its own Git starts.
//!  Nothing here starts Git:
//!  every answer is a file the file system already holds,
//!  read with the bounds `private_storage` gives.
//!
//!  A spelling this cannot resolve answers nothing,
//!  which leaves the pass exactly where it was before this resolution:
//!  a directory this account may not enter,
//!  a path that is no repository,
//!  and a Git directory the files at hand do not describe.
//!  So does a bare repository,
//!  which the pass reached by none of the routes it searched before,
//!  because such a repository keeps no index a commit would take and the pass leaves it as native Git does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gitDir = gitDirectoryOfThisCommand(args, env); // GIT_DIR, else the -C target's repository, else the cwd's
//! ```

/// The repository a command line selects,
///  read from its global options and its environment.
use crate::repository_selection::{SelectedRepository, selected_repository};
/// The global prefix of the command Git will receive.
use crate::global_arguments::{GlobalLayout, global_layout};
/// The wrapper's own controls,
///  removed before the prefix is read,
///  so none is read as a path.
use crate::wrapper_controls::{Controls, no_controls, strip_global_controls};
/// The Git directory a directory is in,
///  found by reading `.git` and walking up.
use crate::git_directory_roots::git_directory_of;
/// The repository's own configuration,
///  read as plain text with the reader's bound.
use crate::private_storage::read_regular_file;
use crate::repo_read_config_text::{CONFIG_TEXT_LIMIT, ConfigEntry, last_entry, parse_bool, parse_config_text};
/// The name every main worktree gives its Git directory,
///  and the operating-system text of the command's arguments and environment.
use std::ffi::{OsStr, OsString};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The name every main worktree gives its Git directory.
const DOT_GIT: &str = ".git";

/// What:
///  The Git directory the command run from `current` will work in,
///  or nothing when that cannot be told without starting Git.
/// Why:
///  One function answers the pass's question for the command Git is about to receive,
///  and `current` is a parameter so a control drives it without changing its own directory.
///  A route this pass did not search before,
///  such as `-C` or `--git-dir`,
///  is searched only when the directory is one a command may enter and is a work tree's Git directory,
///  so a bare repository,
///  a path this account cannot read and a path that is no repository keep the lock they kept before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitDirectoryOfCommandIn(current, args, env): string | undefined {
///   const prefix = prefixOf(args);
///   const selected = selectedRepository(current, prefix, env);
///   if (selected.directory !== current && !canEnter(selected.directory)) return undefined;
///   const gitDir = selected.gitDirectory ?? discovery(selected.directory);
///   if ((selected.directory !== current || selected.namedByOption) && hasNoWorkTree(gitDir)) return undefined;
///   return canEnter(gitDir) ? gitDir : undefined;
/// }
/// ```
pub fn git_directory_of_command_in(current: &Path, arguments: &[OsString], environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    let prefix: Vec<OsString> = global_prefix(arguments);
    let selected: SelectedRepository = selected_repository(current, prefix.as_slice(), environment);
    let moved: bool = selected.directory != current;
    // Git changes into the directory its `-C` chain names before it reads anything else, and refuses the command
    // when it cannot: one such command reaches no repository, so the pass has nothing it may clear.
    if moved && !this_account_can_enter(selected.directory.as_path()) {
        return None;
    }
    let (git_dir, reached_only_now): (PathBuf, bool) = match selected.git_directory {
        Some(named) => (named, moved || selected.named_by_option),
        None => (git_directory_of(selected.directory.as_path())?, moved),
    };
    if reached_only_now && has_no_work_tree(git_dir.as_path()) {
        return None;
    }
    if !this_account_can_enter(git_dir.as_path()) {
        return None;
    }
    return Some(git_dir);
}

/// What:
///  The Git directory this command will work in,
///  or nothing when that cannot be told without starting Git.
/// Why:
///  The executable calls this with its own arguments and environment,
///  and the pass then searches the one repository the command will use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDir = gitDirectoryOfThisCommand(args, env);
/// ```
pub fn git_directory_of_this_command(arguments: &[OsString], environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    let current: PathBuf = std::env::current_dir().ok()?;
    return git_directory_of_command_in(current.as_path(), arguments, environment);
}

/// What:
///  The tokens of the command Git will receive before its subcommand,
///  with the wrapper's own controls removed,
///  so a control is never read as one of Git's path options.
/// Why:
///  Git reads its global options up to the subcommand,
///  and a wrapper control written there is removed before the command reaches Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const prefix = globalPrefix(withoutWrapperControls(args));
/// ```
fn global_prefix(arguments: &[OsString]) -> Vec<OsString> {
    let mut controls: Controls = no_controls();
    let cleaned: Vec<OsString> = strip_global_controls(arguments, &mut controls);
    let layout: GlobalLayout = global_layout(cleaned.as_slice());
    return cleaned[..layout.prefix_len].to_vec();
}

/// What:
///  Whether this account may enter `directory`,
///  which is what Git does before it reads anything of a directory it works in.
/// Why:
///  Search permission on a directory is what reaching `directory/.` needs,
///  so a directory without it holds no repository this command can reach,
///  and a path that only some other account may enter is left as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const canEnter = (directory) => statSync(join(directory, '.')).isDirectory();
/// ```
fn this_account_can_enter(directory: &Path) -> bool {
    return match std::fs::symlink_metadata(directory.join(".")) {
        Ok(found) => found.is_dir(),
        Err(_refused) => false,
    };
}

/// What:
///  Whether the Git directory `git_dir` has no work tree,
///  which is what Git calls a bare repository,
///  so the pass leaves it alone.
/// Why:
///  A work tree's Git directory is the `.git` of a directory,
///  or a linked worktree's own directory,
///  which is the only kind that names a common directory.
///  Every other layout keeps the answer in the repository's own configuration:
///  `core.bare` decides it,
///  and a Git directory without that key is bare unless it names a work tree.
///  A configuration this read cannot prove keeps the repository as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hasNoWorkTree = (gitDir) => !exists(join(gitDir, 'commondir')) && basename(gitDir) !== '.git' && (bareOf(config(gitDir)) ?? !worktreeOf(config(gitDir)));
/// ```
fn has_no_work_tree(git_dir: &Path) -> bool {
    if git_dir.join("commondir").exists() {
        return false;
    }
    if git_dir.file_name() == Some(OsStr::new(DOT_GIT)) {
        return false;
    }
    let Some(entries) = configuration_entries(git_dir) else {
        return true;
    };
    if let Some(bare) = last_entry(entries.as_slice(), "core.bare") {
        // A value this reader cannot interpret decides nothing, so the repository keeps what it had.
        return parse_bool(bare.value.as_deref()) != Some(false);
    }
    return last_entry(entries.as_slice(), "core.worktree").is_none();
}

/// What:
///  The entries of the repository's own configuration file,
///  or nothing when it cannot be read or parsed.
/// Why:
///  The bare question is asked of the repository's configuration and of nothing else:
///  no system or global scope can make a repository bare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = parse(readFile(join(gitDir, 'config')));
/// ```
fn configuration_entries(git_dir: &Path) -> Option<Vec<ConfigEntry>> {
    let bytes: Vec<u8> = read_regular_file(git_dir.join("config").as_path(), CONFIG_TEXT_LIMIT as u64).ok()?;
    return parse_config_text(bytes.as_slice());
}

/// Controls of the pass's repository stay out of the release executable.
#[cfg(test)]
#[path = "lock_pass_repository_tests.rs"]
mod tests;
