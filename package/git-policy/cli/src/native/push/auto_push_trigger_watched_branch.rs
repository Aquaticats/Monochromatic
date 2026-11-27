//! What:
//!  The branch a command started on,
//!  watched before the command runs:
//!  where the branch and its repository are,
//!  the tip the branch held,
//!  and the environment Git would see.
//! Why:
//!  The command may be a merge or a pull,
//!  which move the tip;
//!  what the tip was is what a rebase is told apart by.
//!  The watch holds the state a decision is made from,
//!  and the decision itself is made in `auto_push_trigger_ahead_of_upstream.rs`,
//!  so a reader can hold the two apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type TipWatch = { layout: RepoLayout; variables: [string, string][]; realGit: string; branchRef: string; before: string };
//! function tryWatchTip(worktreeRoot, realGit, overlay, environment): TipWatch | Declined;
//! ```

/// Import the reader that learns the system configuration file from the real Git the child would run,
/// the record's shape for the watched worktree,
/// the repository's layout and the refs it holds.
use crate::git_system_config::with_real_git;
use crate::push_trigger_log::Origin;
use crate::repo_read_layout::{
    RepoLayout,
    read_layout_after_success,
};
use crate::repo_read_refs::{
    RefValue,
    Refs,
};
/// Import the owned string type an environment entry is made of,
/// and the borrowed path the watch hands out.
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What the branch looked like before the command:
/// where it pointed and where its repository is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TipWatch = { layout: RepoLayout; variables: [string, string][]; realGit: string; branchRef: string; before: string };
/// ```
#[derive(Clone, Debug)]
pub struct TipWatch {
    /// Where the worktree and its repository are.
    layout: RepoLayout,
    /// The environment Git would see,
    /// which the configuration reader needs for its scopes.
    variables: Vec<(OsString, OsString)>,
    /// The real Git the service will push with.
    real_git: PathBuf,
    /// The branch `HEAD` named.
    branch_ref: String,
    /// The tip of that branch before the command.
    before: String,
}

/// Why the trigger did nothing:
/// the step that declined,
/// as a stable word,
/// and the facts a person needs to see why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Declined = { reason: string; detail: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declined {
    /// The step,
    /// in words that do not change (`layout-declined`,
    /// `head-detached`,
    /// `upstream-equals-tip`,
    /// and the others).
    pub reason: &'static str,
    /// What the step saw.
    pub detail: String,
}

/// A decline of `reason` with what was seen.
///
/// Why `pub(super)`:
///  the decision that declines lives beside this file,
///  and a reader of that file reads this one first.
pub(super) fn declined(
    reason: &'static str,
    detail: impl Into<String>,
) -> Declined {
    return Declined {
        reason,
        detail: detail.into(),
    };
}

impl TipWatch {
    /// The common Git directory of the watched repository,
    /// where the trigger's records are kept.
    pub fn common_dir(&self) -> &Path {
        return self.layout.common_dir.as_path();
    }

    /// The worktree the command runs in,
    /// as a record names it.
    pub fn origin(&self) -> Origin<'_> {
        return Origin {
            worktree: self.layout.worktree_root.as_path(),
            git_dir: self.layout.git_dir.as_path(),
        };
    }

    /// The branch `HEAD` named before the command.
    pub fn branch_ref(&self) -> &str {
        return self.branch_ref.as_str();
    }

    /// The tip of that branch before the command.
    pub fn tip_before(&self) -> &str {
        return self.before.as_str();
    }

    /// Where the branch and its repository are,
    /// as the readers of refs and objects take them.
    pub(super) fn layout(&self) -> &RepoLayout {
        return &self.layout;
    }

    /// The environment Git would see,
    /// which the configuration reader reads the branch's upstream through.
    pub(super) fn variables(&self) -> &[(OsString, OsString)] {
        return self.variables.as_slice();
    }

    /// The real Git the service pushes with,
    /// which the route to the service is read through.
    pub(super) fn real_git(&self) -> &Path {
        return self.real_git.as_path();
    }
}

/// The branch `HEAD` names and its tip,
/// or the step that found none:
/// a layout the reader declines,
/// a detached `HEAD`,
/// an unborn branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentBranch(layout): { branchRef: string; tip: string } | Declined;
/// ```
pub(super) fn current_branch(layout: &RepoLayout) -> Result<(String, String), Declined> {
    let refs: Refs = Refs::new(
        layout.common_dir.as_path(),
        layout.object_format.hex_length(),
    );
    let Some(head) = refs.lookup_head(layout.git_dir.as_path()) else {
        return Err(declined(
            "head-unreadable",
            format!(
                "HEAD in {} is not a plain symbolic or direct ref",
                layout.git_dir.display()
            ),
        ));
    };
    let RefValue::Symbolic(target) = head else {
        return Err(declined(
            "head-detached",
            "HEAD names a commit, not a branch",
        ));
    };
    if !target.starts_with("refs/heads/") {
        return Err(declined(
            "head-not-a-branch",
            format!("HEAD names {target}"),
        ));
    }
    let Some(tip) = refs.resolve(target.as_str()) else {
        return Err(declined(
            "branch-has-no-tip",
            format!("{target} does not resolve"),
        ));
    };
    return Ok((target, tip));
}

/// What:
///  Start watching the branch of a worktree before a command runs.
/// Why:
///   The command may be a merge or a pull,
/// which move the tip;
/// what the tip was is what a rebase is told apart by.
///   The layout is read as for a caller that follows a Git run that succeeds (`read_layout_after_success`):
/// a configuration value the reader cannot prove is no reason not to push once the command that moved the tip succeeded
/// (the files are read again,
/// not the snapshot that command saw,
/// and the final push is Git's).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchTip(worktreeRoot, realGit, overlay, environment): TipWatch | Declined;
/// ```
pub fn try_watch_tip(
    worktree_root: &Path,
    real_git: &Path,
    overlay: &[(OsString, OsString)],
    environment: &[(OsString, OsString)],
) -> Result<TipWatch, Declined> {
    // The readers learn the system configuration file from the real Git the child would run.
    let named: Vec<(OsString, OsString)> = with_real_git(overlay, real_git);
    let Some(layout) = read_layout_after_success(worktree_root, named.as_slice()) else {
        return Err(declined(
            "layout-declined",
            format!(
                "the files of {} are not a layout the reader takes",
                worktree_root.display()
            ),
        ));
    };
    let (branch_ref, before): (String, String) = current_branch(&layout)?;
    let mut variables: Vec<(OsString, OsString)> = environment.to_vec();
    variables.extend(named);
    return Ok(TipWatch {
        layout,
        variables,
        real_git: real_git.to_path_buf(),
        branch_ref,
        before,
    });
}

/// `try_watch_tip` without the reason:
/// nothing when the watch could not start.
pub fn watch_tip(
    worktree_root: &Path,
    real_git: &Path,
    overlay: &[(OsString, OsString)],
    environment: &[(OsString, OsString)],
) -> Option<TipWatch> {
    return try_watch_tip(worktree_root, real_git, overlay, environment).ok();
}
