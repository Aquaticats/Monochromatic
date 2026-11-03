//! What:
//!  The restore of a snapshot of a main checkout that had uncommitted changes: make the copy's index and files equal the
//!  tree of the new worktree's `HEAD` with `git read-tree --reset -u` and `git clean -f -d`, then prove it in the stage.
//! Why:
//!  `git worktree add` from a checkout with changes makes a worktree that holds the committed files and nothing of the changes.
//!  The snapshot carries the changes, so the copy is reset the way Git's own `reset --hard` would reset it, from the index the
//!  snapshot carries: that index still records the files as they were before they were edited, so the reset rewrites exactly the
//!  edited, deleted and staged paths and leaves the rest. Untracked files that no ignore rule covers go, ignored files stay.
//!  Both Git runs are the ladder's own steps on the new worktree (its Git directory and its stage), started as real Git
//!  children by the lifecycle's runner, so no policy of the wrapper that guards a person's `reset` or `clean` applies to
//!  them. The proof runs before the stage is swapped in; a proof that fails ends the rung and Git's own checkout makes the
//!  worktree. The main checkout is read only to be snapshotted and listed, and is never written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await restoreDirtySnapshot({ source, target, stage, token, runGit, context }); // throws the reason it cannot be proven
//! ```

use super::transaction_git::{GitContext, GitRequest, run_git};
use super::worktree_copy::GitRunner;
use super::worktree_ladder_target::{Target, checkout_environment, run_step};
use super::worktree_snapshot_source::{ChangedPaths, SnapshotSource};
use std::ffi::OsString;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeDirtyRestore";

/// What:
///  The line that tells the person a snapshot of a checkout with changes left the changes out.
/// Why:
///   The new worktree matches a clean `git worktree add`, which the person may not expect from a copy of a dirty checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: the uncommitted changes of the main checkout (${count} paths) were left out of the new worktree.\n`
/// ```
pub fn left_out_note(changed: &ChangedPaths) -> String {
    let paths: &str = if changed.count == 1 { "path" } else { "paths" };
    return format!("cli-git: the uncommitted changes of the main checkout ({} {paths}) were left out of the new worktree, which holds what Git's own checkout holds.\n", changed.count);
}

/// What:
///  Run one captured Git read in the stage under the new worktree's Git directory.
/// Why:
///   The proof reads the stage the way the restore wrote it, never the main checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, stdout } = await runIn(stage, admin, ['diff-files', '--quiet']);
/// ```
fn ask(context: &GitContext, target: &Target, work_tree: &Path, arguments: &[&str]) -> Result<(Option<i32>, Vec<u8>), String> {
    let request: GitRequest = GitRequest {
        cwd: work_tree.to_path_buf(),
        arguments: arguments.iter().map(|word: &&str| { return OsString::from(*word); }).collect(),
        without_prefix: true,
        environment: checkout_environment(target.admin.as_path(), work_tree),
        ..GitRequest::default()
    };
    let output = run_git(context, &request).map_err(|error| { return format!("`git {}` could not start ({error})", arguments.join(" ")); })?;
    return Ok((output.code, output.stdout));
}

/// What:
///  Prove that the stage equals what Git's checkout of the target commit makes: the index equals the commit's tree, Git shows
///  no untracked file, and no tracked file differs from the index.
/// Why:
///   The restore trusts Git's reset and clean; the proof asks Git again from the other side, so a state either of them left
///  (a path the reset skipped, a directory the clean could not remove) ends the rung instead of reaching the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await proveRestored({ context, target, workTree }); // throws the first thing that is not as Git's checkout leaves it
/// ```
pub fn prove_restored(context: &GitContext, target: &Target, work_tree: &Path) -> Result<(), String> {
    let (cached, _out) = ask(context, target, work_tree, &["diff-index", "--cached", "--quiet", target.commit.as_str()])?;
    if cached != Some(0) {
        return Err(String::from("the restored index does not equal the tree of the new HEAD"));
    }
    let (listed, untracked) = ask(context, target, work_tree, &["ls-files", "--others", "--exclude-standard", "--directory", "--no-empty-directory", "-z"])?;
    if listed != Some(0) {
        return Err(String::from("Git could not list the untracked files of the restored worktree"));
    }
    if let Some(name) = untracked.split(|byte: &u8| { return *byte == 0; }).find(|name: &&[u8]| { return !name.is_empty(); }) {
        return Err(format!("{} is still untracked in the restored worktree", super::worktree_snapshot_source::plain(String::from_utf8_lossy(name).as_ref())));
    }
    let (files, _out) = ask(context, target, work_tree, &["diff-files", "--quiet"])?;
    if files != Some(0) {
        return Err(String::from("a tracked file of the restored worktree differs from its index entry"));
    }
    return Ok(());
}

/// What:
///  Restore the stage, which is a snapshot of `source` made with its own index, to the tree of the target commit, and prove it.
/// Why:
///   See the module description. The stage is never the source's own directory, and the Git directory is the new worktree's,
///  so neither run can reach the main checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await restoreDirty({ source, target, stage, token, runGit, context }); // throws the reason
/// ```
pub fn restore_dirty(source: &SnapshotSource, target: &Target, stage: &Path, token: Option<&str>, run_git_step: &mut GitRunner, context: &GitContext) -> Result<(), String> {
    if stage == source.root.as_path() || target.admin == source.git_dir || target.admin == source.common_dir {
        return Err(String::from("the restore would run in the source's own directory"));
    }
    let environment = checkout_environment(target.admin.as_path(), stage);
    let in_stage = |words: &[&str]| -> Vec<OsString> {
        let mut arguments: Vec<OsString> = vec![OsString::from("-C"), stage.as_os_str().to_os_string()];
        arguments.extend(words.iter().map(|word: &&str| { return OsString::from(*word); }));
        return arguments;
    };
    let reset = run_step(run_git_step, token, in_stage(&["read-tree", "--reset", "-u", target.commit.as_str()]).as_slice(), environment.as_slice())?;
    if !reset.succeeded() {
        return Err(format!("`git read-tree --reset -u` ended with {:?}", reset.code));
    }
    let clean = run_step(run_git_step, token, in_stage(&["clean", "--force", "-d", "--quiet"]).as_slice(), environment.as_slice())?;
    if !clean.succeeded() {
        return Err(format!("`git clean --force -d` ended with {:?}", clean.code));
    }
    prove_restored(context, target, stage)?;
    super::diagnostic_log::debug(TAG, format!("restored {} to {} and proved it", stage.display(), target.commit).as_str());
    return Ok(());
}

/// Dirty restore controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_dirty_restore_tests.rs"]
mod tests;
