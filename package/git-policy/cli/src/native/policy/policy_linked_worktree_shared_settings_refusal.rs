//! What:
//!  The refusal of a command that would write the settings every worktree of one repository shares,
//!  with the two readings it needs:
//!  the directory a command of the caller's own runs in,
//!  and whether the path a `--file` write names is that shared file.
//! Why:
//!  One `.git/config` is shared by the main checkout and every linked worktree,
//!  so a write from a linked worktree reaches every checkout of the repository,
//!  and that answer is asked of every command before a guarded form is measured;
//!  it sits beside the check it answers for,
//!  `policy_linked_worktree.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const outcome = refuseSharedRemoteSettings({ facts, change, shellDirectory, allowedDirs, start });
//! ```

/// Which worktree a command targets,
///  and the tool caches the environment exempts,
///  for the rule that keeps a main checkout and a linked one apart.
use crate::effective_target::{
    EffectiveTarget,
    ToolCacheDirectory,
    classify_effective_target,
};
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// The path with every link resolved as far as it exists,
///  for comparing two spellings of one file,
///  and the directory a command runs in,
///  which `--work-tree` does not move.
use crate::repository_link_paths::{
    chain_directory,
    resolve_as_far_as_it_exists,
};
/// Where a command runs.
use crate::repository_location::RepositoryLocation;
/// Whether a `git remote` or `git config` command writes the settings every worktree shares.
use crate::rule_shared_remote::{
    SHARED_REMOTE_SETTINGS_CODE,
    SharedRemoteChange,
};
/// The text that refusal prints.
use crate::rule_shared_remote_messages::shared_remote_settings_message;
/// Two of the three endings every check shares.
use crate::the_answers_a_policy_gives::{
    clean,
    fact_unavailable,
    rejected,
};
/// The common Git directory,
///  for naming the file every worktree shares.
use crate::worktree_identity::common_directory;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The directory a command of the caller's own runs in,
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
///  The command inside the shell of a `!` alias runs somewhere else,
///  which the guard carries as `GuardedForwarded::shell_directory`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commandDirectory = (globalPrefix) => chainDirectory(process.cwd(), globalPrefix);
/// ```
fn command_directory(
    global_prefix: &[OsString],
    start: Option<&Path>,
) -> Result<PathBuf, String> {
    // The caller's own directory, which no option changes: Git moves it only for `-C`.
    let Some(cwd) = start else {
        return Err(String::from(
            "monochromatic-git-policy-cli could not name the directory this command runs in, so the command was \
             not judged.",
        ));
    };
    return Ok(chain_directory(cwd, global_prefix));
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
fn names_the_shared_file(
    named: &Path,
    directory: &Path,
    shared_file: &Path,
) -> bool {
    let spelled: PathBuf = if named.is_absolute() {
        named.to_path_buf()
    } else {
        directory.join(named)
    };
    return resolve_as_far_as_it_exists(spelled.as_path())
        == resolve_as_far_as_it_exists(shared_file);
}

/// What:
///  The refusal of a command that would change the remote settings every worktree of the
///  repository shares,
///  when the command runs in a linked worktree.
///  `&SharedRemoteChange` borrows the change the guard decided about,
///  and `Option<&Path>` the directory Git runs the shell of a `!` alias in,
///  when the command that writes is the one that shell runs.
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
///  and every other command is left to the linked-worktree check that asks this answer.
///  A
///       `--file` write is this rule's only when its path resolves to that same file,
///  read from
///       the directory the writing command runs in:
///  the shell's directory for a command a `!` alias runs,
///  and the
///       caller's `-C` chain for every other one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseSharedRemoteSettings(facts, change, shellDirectory, allowedDirs): PolicyOutcome;
/// ```
pub(crate) fn refuse_shared_remote_settings(
    facts: &mut dyn RepositoryFacts,
    change: &SharedRemoteChange,
    shell_directory: Option<&Path>,
    allowed_worktree_dirs: &[ToolCacheDirectory],
    start: Option<&Path>,
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
        // Where the writing command runs: a `!` alias's shell starts at the top of the working tree Git changed to,
        //  while every other command starts where the caller's `-C` chain ends.
        let directory: PathBuf = match shell_directory {
            Some(found) => found.to_path_buf(),
            None => {
                match command_directory(facts.transaction_context().global_prefix.as_slice(), start)
                {
                    Ok(found) => found,
                    Err(message) => return fact_unavailable(message),
                }
            }
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
        shared_remote_settings_message(
            &change.setting,
            shared_file.as_path(),
            main_checkout.as_path(),
        ),
    );
}
