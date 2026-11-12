//! What:
//!  The refusal of a command that would change the remote settings every worktree of a one-repository shares,
//!  and the two questions it asks about the path a `--file` write names.
//! Why:
//!  One `.git/config` is shared by the main checkout and every linked worktree,
//!  so a write from a linked worktree reaches the main checkout and every other worktree;
//!  the run stays in the linked-worktree check because it is decided before any guarded form is measured,
//!  and it sits in its own file so that check stays within its line bound
//!  (`policy_linked_worktree`,
//!  `binary_shared_remote_tests`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // refuseSharedRemoteSettings(facts, change, allowedDirs) => PolicyOutcome
//! ```

/// The facts every question is asked,
///  and the engine's findings.
use crate::policy_checks::{clean, fact_unavailable, rejected};
use crate::policy_engine::PolicyOutcome;
use crate::repository_facts::RepositoryFacts;
/// Where a command runs.
use crate::repository_location::RepositoryLocation;
/// Whether a `git remote` or `git config` command writes the settings every worktree shares.
use crate::rule_shared_remote::{SHARED_REMOTE_SETTINGS_CODE, SharedRemoteChange};
/// The text that refusal prints.
use crate::rule_shared_remote_messages::shared_remote_settings_message;
/// Which worktree a command targets.
use crate::effective_target::{EffectiveTarget, ToolCacheDirectory, classify_effective_target};
/// The common Git directory,
///  for naming the file every worktree shares.
use crate::worktree_identity::common_directory;
/// The path with every link resolved as far as it exists,
///  for comparing two spellings of one file,
///  and the directory a command runs in,
///  which `--work-tree` does not move.
use crate::repository_link_paths::{chain_directory, resolve_as_far_as_it_exists};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{Path, PathBuf};

/// What:
///  The directory the command runs in,
///  as Git resolved it:
///  this process's own directory moved by the `-C` chain the caller
///  wrote before the subcommand.
///  `&[OsString]` borrows that chain.
///  `Result<PathBuf, String>` is the directory or why it could not be named.
/// Why:
///   A relative `--file` path is read from where the command runs,
///  `--work-tree` does not move that directory for `git config`,
///  and it may name a directory the command does not run in at all,
///  so the worktree top level plus Git's own prefix is not the answer here:
///  that prefix is empty whenever the caller sits outside the `--work-tree`,
///  which is exactly when the two disagree.
///  Only the `-C` chain moves the directory Git reads a relative path from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commandDirectory = (globalPrefix) => chainDirectory(process.cwd(), globalPrefix);
/// ```
fn command_directory(global_prefix: &[OsString]) -> Result<PathBuf, String> {
    // The caller's own directory, which no option changes: Git moves it only for `-C`.
    let Ok(cwd) = std::env::current_dir() else {
        return Err(String::from(
            "monochromatic-git-policy-cli could not name the directory this command runs in, so the command was \
             not judged.",
        ));
    };
    return Ok(chain_directory(cwd.as_path(), global_prefix));
}

/// What:
///  Whether the path a `--file` write names is the configuration file every worktree
///  shares.
///  `&Path` borrows the named path,
///  the directory the command runs in and the shared file.
/// Why:
///   The shared file is one of the paths `--file` takes,
///  and naming it is not naming another file;
///  a
///       relative path,
///  one through `..` and one through a link are the same file,
///  so both spellings are
///       resolved the way the filesystem does before they are compared.
///  A path that names neither the file nor a
///       link to it is a file of the command's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesTheSharedFile = (named, directory, shared) => resolve(named.startsWith('/') ? named : join(directory, named)) === resolve(shared);
/// ```
fn names_the_shared_file(named: &Path, directory: &Path, shared_file: &Path) -> bool {
    let spelled: PathBuf = if named.is_absolute() {
        named.to_path_buf()
    } else {
        directory.join(named)
    };
    return resolve_as_far_as_it_exists(spelled.as_path()) == resolve_as_far_as_it_exists(shared_file);
}

/// What:
///  The refusal of a command that would change the remote settings every worktree of the
///  repository shares,
///  when the command runs in a linked worktree.
///  `&SharedRemoteChange` borrows the change the guard decided about.
/// Why:
///   One `.git/config` is shared by the main checkout and every linked worktree,
///  so a write from a
///       linked worktree reaches the main checkout and every other worktree;
///  the refusal names that file and
///       the main checkout,
///  and changes nothing.
///  A main checkout,
///  a place outside a worktree and an
///       exempt tool cache keep today's behaviour,
///  and every other command is left to the policies below.
///  A
///       `--file` write is this rule's only when its path resolves to that same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseSharedRemoteSettings(facts, change, allowedDirs): PolicyOutcome;
/// ```
pub(crate) fn refuse_shared_remote_settings(
    facts: &mut dyn RepositoryFacts,
    change: &SharedRemoteChange,
    allowed_worktree_dirs: &[ToolCacheDirectory],
) -> PolicyOutcome {
    let location: RepositoryLocation = match facts.location() {
        Ok(found) => found,
        Err(message) => return fact_unavailable(message),
    };
    let target: EffectiveTarget =
        classify_effective_target(&location.identity, allowed_worktree_dirs);
    // Only a linked worktree shares its configuration with another checkout.
    if target != EffectiveTarget::LinkedWorktree {
        return clean();
    }
    // The configuration file Git writes for the repository: `$GIT_COMMON_DIR/config`.
    let shared_file: PathBuf = match common_directory(&location.identity) {
        Some(directory) => directory.join("config"),
        None => {
            return fact_unavailable(String::from(
                "monochromatic-git-policy-cli could not name the configuration file this repository's worktrees \
                 share, so the command was not judged.",
            ));
        }
    };
    // A `--file` write names its own file: only the path that resolves to this repository's shared file is one.
    if let Some(named) = &change.named_file {
        let directory: PathBuf = match command_directory(facts.transaction_context().global_prefix.as_slice()) {
            Ok(found) => found,
            Err(message) => return fact_unavailable(message),
        };
        if !names_the_shared_file(named.as_path(), directory.as_path(), shared_file.as_path()) {
            return clean();
        }
    }
    let main_checkout: PathBuf = match facts.main_worktree_root() {
        Ok(found) => found,
        Err(message) => return fact_unavailable(message),
    };
    return rejected(
        SHARED_REMOTE_SETTINGS_CODE,
        shared_remote_settings_message(&change.setting, shared_file.as_path(), main_checkout.as_path()),
    );
}
