//! What:
//!  The one read that lets a fast rung step aside before it claims an add's record or copies a file:
//! which paths the tree of the commit the add starts at changes,
//! and whether one of them is a path the rung cannot serve.
//! Why:
//!  A rung installs its index with the stat data of every path Git must write zeroed
//! (`doc/planning/cli-git-clone-rung-conversion-fence.md`),
//! and `git read-tree -m -u` verifies the entries of the paths it is about to change,
//!  so a zeroed entry the target tree changes makes it refuse the whole tree difference.
//! The rung then falls back to Git's checkout after its copies,
//!  which is correct and pays the copy twice:
//! a 10,002-file fixture measured 508 ms against `main`'s 378 ms for one such add.
//! The tree the add starts at can differ from the source's `HEAD` in three ways,
//!  and all three are read here:
//! a commit-ish the add names,
//!  the local branch Git checks out in that commit-ish's place,
//!  and the branch `git worktree add <path>` names after the destination when the add names none
//! (`Documentation/git-worktree.adoc`,
//! "COMMANDS",
//!  and `add_worktree` in `builtin/worktree.c`).
//! The fence's own answer already says which paths it cannot serve,
//!  so the rung can answer before the claim and before the first copy,
//!  and a skipped add is Git's own checkout and nothing else.
//! The read is one `git diff-tree --name-only` between the source's frozen commit and that commit:
//! names only,
//!  no blobs,
//!  no worktree.
//! It is the same set `git read-tree -m -u` is about to change,
//!  because a source on either rung holds no staged change
//!  (a source with changes is snapshotted and restored,
//!  never tree-differenced),
//!  so its index is the source's tree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reason = skipReasonForTheTargetTree(source, context, plan, theRungHandsGitAPath, first); // undefined: the rung runs
//! ```

use crate::diagnostic_log::debug;
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::worktree_clone_conversion_fence::{
    ChangeNotServed,
    WhyNotServed,
};
use crate::worktree_command_facts::AddPlan;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::{
    OsStr,
    OsString,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadderTargetTree";

/// The branch namespace every name this module reads is looked for in.
const BRANCH_NAMESPACE: &str = "refs/heads/";

/// The two names Git never takes for a branch,
/// however the refs of a repository look
/// (`check_branch_ref` of `refs.c` rejects a name that starts with `-` and the name `HEAD`).
/// Why they are here:
///  a hand-made ref of such a name would otherwise be read as the branch Git checked out,
///  and `-` is Git's own synonym for `@{-1}`.
const NAMES_NEVER_A_BRANCH: &[&str] = &["HEAD"];

/// What:
///  The commit the add's worktree starts at,
/// as the name the person is told and as the commit-ish a read resolves.
/// Why:
///   `git worktree add` checks the local branch a name is out when the add has neither `-b` nor `--detach`,
/// and an add that names no commit-ish starts at the branch its destination is named after when that branch exists;
///  a read of the tag of the same name would answer about another tree,
///  and the person is told the name they wrote,
///  or the branch Git derived for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TargetTree = { shown: string; read: string };
/// ```
struct TargetTree {
    /// The name the person reads in the sentence:
    /// the commit-ish they wrote,
    /// or the branch Git named after the destination.
    shown: OsString,
    /// The commit-ish the read resolves.
    read: OsString,
}

/// What:
///  The sentence that ends the rung when the tree the add starts at changes a path the rung cannot serve,
///  or nothing when the rung may run.
/// Why:
///   The answer is taken before the add's record is claimed and before anything is copied,
/// so a skipped add pays Git's own checkout and one name-only read,
///  and nothing of the rung's.
/// `the_rung_hands_a_path_to_git` is the caller's own answer to whether any changed path could change the rung's
///  decision;
///  when it is false the read buys nothing and is not paid at all.
/// A read that fails decides nothing:
///  the rung then runs as it did and its fallback still keeps every byte guarantee.
/// `first` is the caller's own answer to which of the changed paths the rung cannot serve,
///  because each rung knows its own path set:
/// the snapshot rung has the whole fence before its snapshot,
///  and the clone rung has the attributes half and its links before its copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = (source, context, plan, handsAPath, first) => {
///   if (!handsAPath) return undefined;
///   const tree = theTreeTheAddStartsAt(source, context, plan); if (!tree) return undefined;
///   const changed = changedPaths(source, context, tree.read); if (!changed) return undefined;
///   const served = first(changed); return served && theSentence(tree.shown, served);
/// };
/// ```
pub fn skip_reason_for_the_target_tree(
    source: &SnapshotSource,
    context: &GitContext,
    plan: &AddPlan,
    the_rung_hands_a_path_to_git: bool,
    first: &dyn Fn(&[Vec<u8>]) -> Option<ChangeNotServed>,
) -> Option<String> {
    if !the_rung_hands_a_path_to_git {
        return None;
    }
    let tree: TargetTree = match the_tree_the_add_starts_at(source, context, plan) {
        Ok(Some(found)) => found,
        Ok(None) => return None,
        Err(reason) => {
            debug(TAG, format!("the commit the add starts at cannot be read ({reason}); the rung runs as it did").as_str());
            return None;
        }
    };
    let changed: Vec<Vec<u8>> = match changed_paths(context, source, tree.read.as_os_str()) {
        Ok(found) => found,
        Err(reason) => {
            debug(
                TAG,
                format!(
                    "the tree of {} cannot be read ({reason}); the rung runs as it did",
                    String::from_utf8_lossy(tree.shown.as_encoded_bytes())
                )
                .as_str(),
            );
            return None;
        }
    };
    let served: ChangeNotServed = first(changed.as_slice())?;
    return Some(the_sentence(tree.shown.as_os_str(), &served));
}

/// What:
///  The commit `git worktree add` checks out for this plan,
///  or nothing when it is the source's `HEAD`.
/// Why:
///   `add()` of `builtin/worktree.c` gives an add that names no commit-ish the branch named after the destination,
/// creates that branch at `HEAD` when it does not exist,
///  and `add_worktree` sets the new worktree's `HEAD` to `refs/heads/<name>` whenever that ref exists and the add
///  has neither `-b` nor `--detach`;
///  a tag of the same name is what `rev-parse` would reach first,
///  and never what Git checks out.
/// A branch at the source's `HEAD` names the same tree,
/// so nothing can differ there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theTreeTheAddStartsAt(source, context, plan): TargetTree | undefined | Error;
/// ```
fn the_tree_the_add_starts_at(
    source: &SnapshotSource,
    context: &GitContext,
    plan: &AddPlan,
) -> Result<Option<TargetTree>, String> {
    let Some(named) = plan.target.as_deref() else {
        // Git starts at `HEAD` itself when there is no commit-ish and it creates a branch: `-b` names the branch.
        if !plan.checks_out_a_branch {
            return Ok(None);
        }
        let Some(branch_name) = plan.destination.file_name() else {
            return Ok(None);
        };
        let Some((branch, commit)) = the_local_branch(context, source, branch_name)? else {
            return Ok(None);
        };
        return Ok(branch_tree(
            source,
            branch_name.to_os_string(),
            branch,
            commit.as_str(),
        ));
    };
    if plan.checks_out_a_branch
        && let Some((branch, commit)) = the_local_branch(context, source, named)?
    {
        return Ok(branch_tree(
            source,
            named.to_os_string(),
            branch,
            commit.as_str(),
        ));
    }
    return Ok(Some(TargetTree {
        shown: named.to_os_string(),
        read: named.to_os_string(),
    }));
}

/// What:
///  The tree of the branch Git checks out,
///  or nothing when that branch names the source's `HEAD`.
/// Why:
///   The person is told the name they wrote while the read asks for the branch itself,
/// and a branch at the source's `HEAD` cannot change a path against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const branchTree = (source, shown, branch, commit) => commit === source.headCommit ? undefined : { shown, read: branch };
/// ```
fn branch_tree(
    source: &SnapshotSource,
    shown: OsString,
    branch: OsString,
    commit: &str,
) -> Option<TargetTree> {
    if commit == source.clean.head_commit.as_str() {
        return None;
    }
    return Some(TargetTree {
        shown,
        read: branch,
    });
}

/// What:
///  The local branch `name` is,
///  as its full ref and the object it names,
///  or nothing when no such branch exists.
/// Why:
///   This is the one question `check_branch_ref` and `refs_ref_exists` answer for `add_worktree`:
/// a ref of exactly that name in `refs/heads/`.
/// The answer is read with one `git for-each-ref` rather than by looking into the Git directory,
///  so the packed refs and whatever ref backend the repository keeps are Git's own to read.
/// A name `check_branch_ref` rejects is no branch,
///  and a record whose ref is another name is no branch of this one,
///  so the name is read as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const localBranch = (context, source, name) => git(context, source.root, ['for-each-ref', '--format=%(objectname) %(refname)', '--end-of-options', `refs/heads/${name}`]).split('\n');
/// ```
fn the_local_branch(
    context: &GitContext,
    source: &SnapshotSource,
    name: &OsStr,
) -> Result<Option<(OsString, String)>, String> {
    if name.as_encoded_bytes().starts_with(b"-")
        || NAMES_NEVER_A_BRANCH
            .iter()
            .any(|rejected| return name == OsStr::new(rejected))
    {
        return Ok(None);
    }
    let mut wanted: OsString = OsString::from(BRANCH_NAMESPACE);
    wanted.push(name);
    let arguments: Vec<OsString> = vec![
        OsString::from("for-each-ref"),
        OsString::from("--format=%(objectname) %(refname)"),
        OsString::from("--end-of-options"),
        wanted.clone(),
    ];
    let request: GitRequest = GitRequest {
        cwd: source.root.clone(),
        arguments,
        without_prefix: true,
        ..GitRequest::default()
    };
    let output: GitOutput = run_git(context, &request).map_err(|error| {
        return format!("a Git run of the local branch could not start ({error})");
    })?;
    if output.code != Some(0) {
        return Err(format!(
            "`git for-each-ref` ended with {:?} ({})",
            output.code,
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    return Ok(the_one_branch(output.stdout.as_slice(), wanted));
}

/// What:
///  The record that names exactly the wanted ref,
///  as the ref and the object it holds,
///  or nothing when no record does.
/// Why:
///   A pattern without a wildcard also matches every ref that begins with it up to a slash,
/// so a tag `rel` beside a branch `rel/one` answers `refs/heads/rel/one`;
///  only a record whose ref is exactly the wanted one is the branch Git checks out,
///  and other records are no branch of that name rather than a failed read,
///  so the name is read as written and the rung declines exactly where `main` skips.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theOneBranch = (stdout, wanted) => { for (const line of stdout.split('\n').filter(Boolean)) { const space = line.indexOf(' '); if (space >= 0 && line.slice(space + 1) === wanted) return [line.slice(0, space), wanted]; } return undefined; };
/// ```
fn the_one_branch(
    stdout: &[u8],
    wanted: OsString,
) -> Option<(OsString, String)> {
    for record in stdout
        .split(|byte| return *byte == b'\n')
        .filter(|line| return !line.is_empty())
    {
        let Some(space) = record.iter().position(|byte| return *byte == b' ') else {
            continue;
        };
        if &record[space + 1..] == wanted.as_encoded_bytes() {
            return Some((
                wanted,
                String::from_utf8_lossy(&record[..space]).into_owned(),
            ));
        }
    }
    return None;
}

/// What:
///  The whole line the person reads,
///  naming the commit,
///  the path,
///  and why the rung cannot serve it.
/// Why:
///   One sentence per reason says what this rung does with the path,
/// and the tape of the decline ends with what happens instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theSentence = (shown, served) => `the tree of ${shown} differs from the source's index at ${served.path}, ${clause(served.why)}, so this rung declines and Git's checkout writes the worktree`;
/// ```
fn the_sentence(
    shown: &OsStr,
    served: &ChangeNotServed,
) -> String {
    let clause: &str = match served.why {
        WhyNotServed::TheFenceWritesIt => "which this rung must leave for Git to write",
        WhyNotServed::ALinkIsCopied => "which this rung makes again as a link it cannot vouch for",
    };
    return format!(
        "the tree of {} differs from the source's index at {}, {clause}, so this rung declines and Git's checkout writes the worktree",
        String::from_utf8_lossy(shown.as_encoded_bytes()),
        String::from_utf8_lossy(served.path.as_slice())
    );
}

/// What:
///  Every path the tree of `target` changes against the source's `HEAD`,
/// as one `git diff-tree -r -z --name-only` reports them.
/// Why:
///   The names are the whole answer,
/// so no blob and no worktree is read,
///  and a rename is two names rather than a pair (`--no-renames`),
///  which is what the tree update changes.
/// A target that does not resolve is Git's own error later;
/// this read answers it as a reason nothing is decided from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = (context, source, target) => git(context, source.root, ['diff-tree', '-r', '-z', '--name-only', '--no-renames', '--end-of-options', source.clean.headCommit, `${target}^{commit}`]).split('\0');
/// ```
fn changed_paths(
    context: &GitContext,
    source: &SnapshotSource,
    target: &OsStr,
) -> Result<Vec<Vec<u8>>, String> {
    let mut peeled: OsString = target.to_os_string();
    peeled.push("^{commit}");
    let mut arguments: Vec<OsString> = [
        "diff-tree",
        "-r",
        "-z",
        "--name-only",
        "--no-renames",
        "--end-of-options",
    ]
    .iter()
    .map(|word: &&str| return OsString::from(word))
    .collect();
    arguments.push(OsString::from(source.clean.head_commit.as_str()));
    arguments.push(peeled);
    let request: GitRequest = GitRequest {
        cwd: source.root.clone(),
        arguments,
        without_prefix: true,
        ..GitRequest::default()
    };
    let output: GitOutput = run_git(context, &request).map_err(|error| {
        return format!("a Git run of the target tree could not start ({error})");
    })?;
    if output.code != Some(0) {
        return Err(format!(
            "`git diff-tree` ended with {:?} ({})",
            output.code,
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    let mut changed: Vec<Vec<u8>> = Vec::new();
    for path in output.stdout.split(|byte| return *byte == 0) {
        if path.is_empty() {
            continue;
        }
        changed.push(path.to_vec());
    }
    return Ok(changed);
}

/// Skip controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_target_tree_skip_tests.rs"]
mod tests;
