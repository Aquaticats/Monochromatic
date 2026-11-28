//! What:
//!  The place one of the two words runs in:
//!  the checkout whose worktrees hold the slots,
//!  the repository's common Git directory that holds the records,
//!  and the account's home that holds the cache the staging folder stands in.
//! Why:
//!  Both commands name a slot below `.claude/worktrees` of a checkout and read
//!  records under one common directory
//!  (`worktree_clone_removal_cancel::the_proof_inputs`,
//!  `worktree_clone_record_settling::settling_command`),
//!  so a caller in no worktree,
//!  or in a worktree whose home this process cannot name,
//!  is refused before any record or mount is read.
//!  Git answers where the invocation stands,
//!  in one query,
//!  with every `-C` and symbolic link already applied,
//!  so this file re-implements no discovery of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = theRoute('cancel-worktree-removal', globalArguments, overlay, realGit, home);
//! // { checkout, commonDir, home } or an answer that refuses
//! ```

use crate::repo_read::repository_facts::{
    RepositoryFacts,
    git_facts,
};
use crate::repo_read::repository_location::RepositoryLocation;
use crate::worktree::worktree_identity::{
    common_directory,
    worktree_root,
};
use crate::worktree_clone_cancel_and_settle_commands::the_answer_one_word_prints::AnAnswer;
use crate::worktree_clone_removal_cancel::cancel_refusal::CALLERS_MISTAKE;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The three places both commands read and write.
/// Why:
///  Every seam the two commands take is made from one of them,
///  so one value carries them together and a command cannot be handed a slot
///  below one checkout and a record home below another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRoute = { checkout: string; commonDir: string; home?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TheRoute {
    /// The checkout whose `.claude/worktrees` holds the slots.
    pub(crate) checkout: PathBuf,
    /// The common Git directory the wrapper's records live under.
    pub(crate) common_dir: PathBuf,
    /// The account's home,
    /// which the cache holding the staging folder stands in,
    /// where this process can name it.
    pub(crate) home: Option<PathBuf>,
}

impl TheRoute {
    /// What:
    ///  The place one invocation selects,
    ///  or the answer that refuses it.
    /// Why:
    ///  A slot is a folder of one checkout and a record is a file of one common
    ///  directory,
    ///  so a command that could not name either would read another repository's
    ///  slot,
    ///  which is the one mistake a person never means.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// TheRoute.of('cancel-worktree-removal', globalArguments, overlay, realGit, home);
    /// ```
    pub(crate) fn of(
        word: &str,
        global_arguments: &[OsString],
        overlay: &[(OsString, OsString)],
        real_git: &Path,
        home: Option<PathBuf>,
    ) -> Result<TheRoute, AnAnswer> {
        let mut facts = git_facts(real_git, global_arguments, overlay);
        let location: RepositoryLocation = facts.location().map_err(|reason| {
            return AnAnswer::on_standard_error(
                CALLERS_MISTAKE,
                format!(
                    "git monochromatic-git-policy-cli {word}: this invocation selects no repository, and Git answered: {reason}"
                ),
            );
        })?;
        let Some(checkout) = worktree_root(&location.identity) else {
            return Err(AnAnswer::on_standard_error(
                CALLERS_MISTAKE,
                format!(
                    "git monochromatic-git-policy-cli {word}: this invocation stands in no worktree of a repository, and a slot is a folder below one worktree's `.claude/worktrees`. Run it inside the checkout whose slot is named."
                ),
            ));
        };
        let Some(common_dir) = common_directory(&location.identity) else {
            return Err(AnAnswer::on_standard_error(
                CALLERS_MISTAKE,
                format!(
                    "git monochromatic-git-policy-cli {word}: this invocation's repository answers no common Git directory, and the wrapper's records of a slot live under one."
                ),
            ));
        };
        return Ok(TheRoute {
            checkout: checkout.to_path_buf(),
            common_dir: common_dir.to_path_buf(),
            home,
        });
    }

    /// What:
    ///  The account's home,
    ///  or the answer that refuses a command which needs one.
    /// Why:
    ///  The cancellation settles the staging folder that stands in the account's
    ///  own cache,
    ///  so a process that cannot name the home cannot tell what it left.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const home = route.theHome('cancel-worktree-removal');
    /// ```
    pub(crate) fn the_home(
        &self,
        word: &str,
    ) -> Result<&Path, AnAnswer> {
        return match self.home.as_deref() {
            Some(home) => Ok(home),
            None => Err(AnAnswer::on_standard_error(
                CALLERS_MISTAKE,
                format!(
                    "git monochromatic-git-policy-cli {word}: this process names no account home, and the staging folder a cancellation settles stands in the account's own cache."
                ),
            )),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_place_the_two_commands_run_in_tests.rs"]
mod tests;
