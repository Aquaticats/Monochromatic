//! What:
//!  Publishing a landed commit when the remote branch advanced after the commit landed:
//!       the merge of the remote tip and the local tip is computed in the object store,
//!       committed without a worktree or an index, and pushed with a plain push.
//! Why:
//!  A rejected `push` of a branch that the remote moved cannot succeed again, and an ordinary merge in the active
//!  worktree can be blocked, or made unsafe, by another consumer's uncommitted files.
//!  `git merge-tree --write-tree` and `git commit-tree` read and write only objects, so the active worktree,
//!  its index and its untracked files are never touched while merging, and both histories stay reachable from the
//!  published tip.
//!  Only after the push succeeds is the branch fast-forwarded, and only where nothing can be lost: a dirty worktree
//!  follows too when the move changes none of the files its uncommitted work touches.
//!  An amended or rebased branch is merged like any other: nothing already published is lost, and the replaced
//!  commits stay in the remote history beside the new ones.

/// The coordinator's Git, its command builder, queries and error type.
use super::auto_push_single_flight::{
    PushAttempt, PushGit, SingleFlightError, git_command, git_output, push_command, run_query,
};
/// The deadline-bound runner of network commands and the limits it applies.
use super::auto_push_dirty_paths::{diff_paths, listed, overlapping_paths, status_paths};
use super::bounded_push::{STOP_GRACE, apply_low_speed_defaults, push_deadline, run_bounded_push};
/// The diagnostic log of the wrapper.
use super::diagnostic_log::{debug, warn};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// Prefix of every line this module adds to a push's output, so the terminal filter can keep it.
pub const RECONCILE_LINE_PREFIX: &str = "cli-git: auto-push: ";

/// Start of the line that explains why a rejected push stays unpublished; the generic "run `git push`" note is then
/// wrong advice and is not printed.
pub const RECONCILE_FAILED_PREFIX: &str = "cli-git: auto-push: not published: ";

/// How many times the remote tip is fetched, merged and pushed before the push is reported as failed.
pub const MAX_RECONCILE_ATTEMPTS: u32 = 10;

/// The marker Git prints for a push the remote refused as stale; the word is not translated.
const REJECTED_MARKER: &str = "[rejected]";

/// The marker Git prints for an update that the remote's own rules refused, such as GitHub's protected branch (GH006).
/// A remote that is updated by two pushes at once refuses the slower one this way, with no word about staleness.
const REMOTE_REJECTED_MARKER: &str = "[remote rejected]";

/// Where the plain push goes: the branch's upstream remote and the ref it tracks there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Destination = { remote: string; mergeRef: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Destination {
    /// Remote name from `branch.<name>.remote`.
    pub remote: String,
    /// Full ref from `branch.<name>.merge`, for example `refs/heads/main`.
    pub merge_ref: String,
}

/// What:
///  Whether a push's output shows a refusal that a moved remote branch can explain: Git's stale `[rejected]`, or the
///  remote's own `[remote rejected]`.
/// Why:
///   Only a refusal of an update that raced another push can be cured by merging; an authentication failure or an offline
///  error must be reported as they are.
///  The message cannot tell a race from a rule that refuses every push, so the caller compares the remote tip with the
///  branch and returns the refusal unchanged when the remote has not moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wasRejectedAsStale = (output: string) => output.includes('[rejected]') || output.includes('[remote rejected]');
/// ```
pub fn was_rejected_as_stale(output: &str) -> bool {
    return output.contains(REJECTED_MARKER) || output.contains(REMOTE_REJECTED_MARKER);
}

/// A failed attempt that explains itself and keeps the push output that came before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { exitCode: 1, output: `${previous}\n${RECONCILE_FAILED_PREFIX}${reason}` }
/// ```
fn unpublished(failed_prefix: &str, previous: &str, exit_code: i64, reason: &str) -> PushAttempt {
    return PushAttempt {
        exit_code: if exit_code == 0 { 1 } else { exit_code },
        output: format!("{previous}\n{failed_prefix}{reason}"),
    };
}

/// Argument list from string literals and owned text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['merge-tree', '--write-tree', ...]
/// ```
fn argv(parts: &[&str]) -> Vec<OsString> {
    return parts.iter().map(OsString::from).collect();
}

/// What:
///  Whether the first commit is the second or one of its ancestors, or nothing when Git cannot tell.
/// Why:
///   Exit `0` and `1` are the answers; any other exit means an object is missing, and that is not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function isAncestor({ ancestor, descendant }): Promise<boolean | undefined>;
/// ```
fn is_ancestor(
    git: &PushGit,
    ancestor: &str,
    descendant: &str,
) -> Result<Option<bool>, SingleFlightError> {
    let output = run_query(
        git,
        &argv(&["merge-base", "--is-ancestor", ancestor, descendant]),
    )?;
    return Ok(match output.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    });
}

/// What:
///  The trimmed value of a Git setting, empty when it is not set.
/// Why:
///   An unset key exits `1` and prints nothing; the callers treat "not set" and "set to nothing" alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function configValue({ key }): Promise<string>;
/// ```
fn config_value(git: &PushGit, key: &str) -> Result<String, SingleFlightError> {
    let value = run_query(git, &argv(&["config", "--get", key]))?;
    return Ok(String::from_utf8_lossy(&value.stdout).trim().to_owned());
}

/// What:
///  The remote and the branch ref that the plain `git push` of this branch updates, when they can be named.
/// Why:
///   The merge must be made with the tip of the branch the rejected push targeted: with a push remote, a triangular
///  setup or `push.default=current` that is not the tracked branch.
///   The push remote follows Git's order (`branch.<name>.pushRemote`, then `remote.pushDefault`, then
///  `branch.<name>.remote`), and `push.default` picks the ref: `current` and a triangular `simple` push to the
///  branch's own name, `upstream` and a non-triangular `simple` push to the tracked ref.
///   `@{push}` is not used because Git refuses to resolve it for a push remote that differs from the tracked one,
///  although the push itself works.
///   The remote `.`, a `remote.<name>.push` refspec, `matching`, `nothing` and a target outside `refs/heads` are not
///  reconciled, because the rejection then came from a destination this module cannot name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readDestination({ branch }): Promise<Destination | undefined>;
/// ```
fn read_destination(
    git: &PushGit,
    branch_ref: &str,
) -> Result<Option<Destination>, SingleFlightError> {
    let Some(name) = branch_ref.strip_prefix("refs/heads/") else {
        return Ok(None);
    };
    let tracked_remote = config_value(git, &format!("branch.{name}.remote"))?;
    let mut push_remote = config_value(git, &format!("branch.{name}.pushRemote"))?;
    if push_remote.is_empty() {
        push_remote = config_value(git, "remote.pushDefault")?;
    }
    if push_remote.is_empty() {
        push_remote.clone_from(&tracked_remote);
    }
    if push_remote.is_empty() || push_remote == "." {
        return Ok(None);
    }
    let refspecs = run_query(
        git,
        &argv(&["config", "--get-all", &format!("remote.{push_remote}.push")]),
    )?;
    if !refspecs.stdout.is_empty() {
        return Ok(None);
    }
    let triangular = push_remote != tracked_remote;
    let mode = config_value(git, "push.default")?;
    let merge_ref = config_value(git, &format!("branch.{name}.merge"))?;
    let target = match (mode.as_str(), triangular) {
        ("current", _) | ("" | "simple", true) => format!("refs/heads/{name}"),
        ("" | "simple" | "upstream" | "tracking", false) => merge_ref,
        _ => return Ok(None),
    };
    if !target.starts_with("refs/heads/") || target == "refs/heads/" {
        return Ok(None);
    }
    return Ok(Some(Destination {
        remote: push_remote,
        merge_ref: target,
    }));
}

/// What:
///  The arguments of a push that names its destination, when the plain `git push` of this branch cannot.
/// Why:
///   Under `push.default=simple` Git refuses to push a branch whose upstream has another name (a linked worktree
///  branch created from `origin/main` is the usual case) before it contacts the remote, so no reconcile can start.
///  The destination `read_destination` resolves is exactly where the person tracked the branch, so the push names it.
///  A branch whose destination has its own name needs no such push, and nothing is returned for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function namedDestinationPush({ branchRef }): Promise<string[] | undefined>; // ['push', remote, `${branchRef}:${mergeRef}`]
/// ```
pub fn named_destination_push(
    git: &PushGit,
    branch_ref: &str,
) -> Result<Option<Vec<String>>, SingleFlightError> {
    let Some(destination) = read_destination(git, branch_ref)? else {
        return Ok(None);
    };
    if destination.merge_ref == branch_ref {
        return Ok(None);
    }
    return Ok(Some(vec![
        String::from("push"),
        destination.remote,
        format!("{branch_ref}:{}", destination.merge_ref),
    ]));
}

/// What:
///  Fetch the remote branch into a private ref and return the commit it names.
/// Why:
///   `FETCH_HEAD` is shared by every fetch in the repository, so another process could replace it between the fetch
///  and the read; a ref named for this process cannot be replaced, and it is deleted before this returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fetchRemoteTip({ destination }): Promise<string | PushAttempt>;
/// ```
fn fetch_remote_tip(
    git: &PushGit,
    destination: &Destination,
) -> Result<Result<String, PushAttempt>, SingleFlightError> {
    let private_ref = format!("refs/cli-git/auto-push/fetch-{}", std::process::id());
    let refspec = format!("+{}:{private_ref}", destination.merge_ref);
    let arguments = argv(&[
        "fetch",
        "--no-tags",
        "--quiet",
        &destination.remote,
        &refspec,
    ]);
    let mut command = git_command(git, &arguments);
    apply_low_speed_defaults(&mut command, &git.environment);
    let fetched = run_bounded_push(command, push_deadline(&git.environment), STOP_GRACE, git.environment.as_slice())?;
    if fetched.exit_code != 0 {
        return Ok(Err(fetched));
    }
    let tip = git_output(
        git,
        &argv(&[
            "rev-parse",
            "--verify",
            &format!("{private_ref}^{{commit}}"),
        ]),
    );
    // The private ref is scaffolding; a failed deletion leaves one small ref that the next run overwrites.
    let _deleted = run_query(git, &argv(&["update-ref", "-d", &private_ref]));
    return Ok(Ok(tip?));
}

/// What:
///  Commit the merge of two tips from objects alone and return its OID, or the paths that conflict.
/// Why:
///   `merge-tree --write-tree` exits `1` for a conflicted merge and lists the paths; nothing is written to a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitMerge({ local, remote, message }): Promise<{ oid } | { conflicts }>;
/// ```
fn commit_merge(
    git: &PushGit,
    local: &str,
    remote: &str,
    message: &str,
) -> Result<Result<String, Vec<String>>, SingleFlightError> {
    let merged = run_query(
        git,
        &argv(&[
            "merge-tree",
            "--write-tree",
            "--no-messages",
            "--name-only",
            "--allow-unrelated-histories",
            local,
            remote,
        ]),
    )?;
    let text = String::from_utf8_lossy(&merged.stdout).into_owned();
    let mut lines = text.lines();
    let tree = lines.next().unwrap_or_default().trim().to_owned();
    if merged.code == Some(1) {
        let paths: Vec<String> = lines
            .map(str::trim)
            .filter(|line| {
                return !line.is_empty();
            })
            .map(str::to_owned)
            .collect();
        return Ok(Err(paths));
    }
    if merged.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("merge-tree --write-tree {local} {remote}"),
            code: merged.code,
            stderr: String::from_utf8_lossy(&merged.stderr).into_owned(),
        });
    }
    let mut commit_tree = vec!["commit-tree"];
    if signs_commits(git)? {
        commit_tree.push("-S");
    }
    commit_tree.extend(["-p", local, "-p", remote, "-m", message, &tree]);
    let oid = git_output(git, &argv(&commit_tree))?;
    return Ok(Ok(oid));
}

/// What:
///  Whether `commit.gpgsign` asks for signed commits.
/// Why:
///   `git commit-tree` signs only when given `-S` and never reads `commit.gpgsign`, so the merge would be the one
///  unsigned commit of a signed branch, which a remote that requires verified signatures rejects.
///   An unset key is `git config` exit `1`; any other failure is reported like every failed command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function signsCommits(git): Promise<boolean>;
/// ```
fn signs_commits(git: &PushGit) -> Result<bool, SingleFlightError> {
    let asked = run_query(
        git,
        &argv(&["config", "--type=bool", "--get", "commit.gpgsign"]),
    )?;
    if asked.code == Some(1) {
        return Ok(false);
    }
    if asked.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("config --type=bool --get commit.gpgsign"),
            code: asked.code,
            stderr: String::from_utf8_lossy(&asked.stderr).into_owned(),
        });
    }
    return Ok(asked.stdout.trim_ascii() == b"true");
}

/// What:
///  Whether the status read after a fast-forward differs from the one read before it.
///  `&[u8]` borrows Git's output.
/// Why:
///   The uncommitted work must read exactly as it did; a difference is the only sign that the move touched it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statusDiffers(before: Uint8Array, after: Uint8Array): boolean;
/// ```
fn status_differs(before: &[u8], after: &[u8]) -> bool {
    return before != after;
}

/// What:
///  Fast-forward the pushed branch and this worktree to the published merge, or the reason that was not done.
///  The success value says whether uncommitted work was left in place.
/// Why:
///   A branch left behind the remote makes the next commit rejected again, and the person at the keyboard would have
///  to merge by hand; the merge contains the branch tip, so the move is a pure fast-forward.
///   It is attempted only where nothing can be lost or surprised: the worktree has the branch checked out at the tip
///  that was pushed, and no uncommitted change (tracked file, staged entry, or either name of a staged rename)
///  lies in a file the move writes.
///   Another session's work in progress therefore never holds the branch behind the remote, while a move that
///  would write over such a file is left for the person.
///   The status read takes no optional lock, and `git merge --ff-only` refuses, without changing anything, a move that
///  would overwrite an untracked file or a change made after the status read.
///   Every refusal is a reason for the report, never a failure: the merge is already published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advanceLocalBranch({ git, branchRef, local, mergeOid }): Promise<{ reason?: string; keptChanges?: boolean }>;
/// ```
fn advance_local_branch(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Result<bool, String>, SingleFlightError> {
    let checked_out = run_query(git, &argv(&["symbolic-ref", "--quiet", "HEAD"]))?;
    if checked_out.code != Some(0) {
        return Ok(Err(String::from("HEAD is detached")));
    }
    if String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(Err(String::from(
            "this worktree has another branch checked out",
        )));
    }
    let head = run_query(git, &argv(&["rev-parse", "--verify", "HEAD^{commit}"]))?;
    if String::from_utf8_lossy(&head.stdout).trim() != local {
        return Ok(Err(String::from(
            "the branch moved after the commit landed",
        )));
    }
    let status_arguments = argv(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=no",
    ]);
    let status = run_query(git, &status_arguments)?;
    if status.code != Some(0) {
        return Ok(Err(String::from("the worktree status could not be read")));
    }
    let dirty = status_paths(&status.stdout);
    let kept: bool = !dirty.is_empty();
    if kept {
        let incoming = run_query(
            git,
            &argv(&["diff", "--name-only", "-z", "--no-renames", local, merge_oid]),
        )?;
        if incoming.code != Some(0) {
            return Ok(Err(String::from(
                "the files the merge changes could not be read",
            )));
        }
        let overlap = overlapping_paths(&dirty, &diff_paths(&incoming.stdout));
        if !overlap.is_empty() {
            return Ok(Err(format!(
                "the worktree has uncommitted changes to files the merge changes: {}",
                listed(&overlap)
            )));
        }
    }
    let moved = run_query(git, &argv(&["merge", "--ff-only", "--quiet", merge_oid]))?;
    if moved.code != Some(0) {
        let detail = String::from_utf8_lossy(&moved.stderr);
        return Ok(Err(format!(
            "Git refused the fast-forward: {}",
            detail.lines().next().unwrap_or_default().trim()
        )));
    }
    if kept {
        // The uncommitted work must read exactly as it did; the move is already done, so a difference is reported.
        let after = run_query(git, &status_arguments)?;
        if status_differs(&status.stdout, &after.stdout) {
            warn(
                TAG,
                "the worktree status differs after the fast-forward; compare it with the status before the commit",
            );
        }
    }
    return Ok(Ok(kept));
}

/// What:
///  Merge the remote tip into the pushed branch's tip and publish the result, a bounded number of times.
/// Why:
///   Another worktree advancing the remote makes the plain push fail for good, while both histories are valid and
///  must both be kept; the merge is published as a new commit above the local tip, so the landed commit stays
///  reachable and nothing is forced.
///   Once the merge is published, the branch and its worktree are fast-forwarded to it when that cannot lose or
///  overwrite anything; otherwise they are left as they are and the report says why.
///   A push destination Git cannot name, a conflict, an exhausted retry budget and every failure of a command are
///  reported, and only the first leaves the original rejection unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reconcileDivergentPush({ git, branchRef, rejected }): Promise<PushAttempt>;
/// ```
pub fn reconcile_divergent_push(
    git: &PushGit,
    branch_ref: &str,
    rejected: PushAttempt,
) -> Result<PushAttempt, SingleFlightError> {
    if !was_rejected_as_stale(&rejected.output) {
        return Ok(rejected);
    }
    let Some(destination) = read_destination(git, branch_ref)? else {
        debug(
            TAG,
            "the push destination cannot be resolved to a remote branch; no reconcile",
        );
        return Ok(rejected);
    };
    let local = git_output(
        git,
        &argv(&["rev-parse", "--verify", &format!("{branch_ref}^{{commit}}")]),
    )?;
    let plan = Reconcile {
        destination,
        local,
        branch_ref: Some(branch_ref),
        label: "auto-push",
        explicit: false,
    };
    let mut push_merge = |target: &Destination, merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        let arguments = argv(&[
            "push",
            &target.remote,
            &format!("{merge_oid}:{}", target.merge_ref),
        ]);
        let mut command = push_command(git, &arguments);
        apply_low_speed_defaults(&mut command, &git.environment);
        return run_bounded_push(command, push_deadline(&git.environment), STOP_GRACE, git.environment.as_slice());
    };
    return Ok(match integrate_and_push(git, &plan, rejected, &mut push_merge)? {
        Integrated::NotDivergent(attempt) | Integrated::Finished(attempt) => attempt,
    });
}

/// What:
///  What one reconcile run integrates, where, and how it words and limits itself.
///  `Option<&str>` is "the branch, or nothing for a detached HEAD".
/// Why:
///   Auto-push and an explicit push share the loop; they differ in the line prefix they print, in whether the branch
///  follows the published merge, and in how much of a failure they keep quiet about.
///   An explicit push already printed Git's own refusal, so a remote that cannot be fetched, a remote that did not
///  diverge, and a failure that is not a stale update are left exactly as the push reported them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reconcile = { destination: Destination; local: string; branchRef?: string; label: string; explicit: boolean };
/// ```
pub struct Reconcile<'a> {
    /// The remote and the ref the rejected push updated.
    pub destination: Destination,
    /// The commit the rejected push sent.
    pub local: String,
    /// The branch that follows the published merge when it can, or nothing for a detached HEAD.
    pub branch_ref: Option<&'a str>,
    /// The word after `cli-git:` in every line the run adds.
    pub label: &'a str,
    /// Whether the rejected push was typed by a person and has already shown its own output.
    pub explicit: bool,
}

/// How a reconcile run ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Integrated = { kind: 'not-divergent'; attempt } | { kind: 'finished'; attempt };
/// ```
pub enum Integrated {
    /// The remote did not diverge, or the run could not tell; the rejected attempt stands as it was.
    NotDivergent(PushAttempt),
    /// The run published the merge, or says why it did not.
    Finished(PushAttempt),
}

/// What:
///  Merge the remote tip into the pushed commit and publish the result, a bounded number of times.
/// Why:
///   Another worktree advancing the remote makes the plain push fail for good, while both histories are valid and
///  must both be kept; the merge is published as a new commit above the local tip, so the landed commit stays
///  reachable and nothing is forced.
///   `push_merge` publishes one merge, and is told the destination and the merge commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function integrateAndPush({ git, plan, rejected, pushMerge }): Promise<Integrated>;
/// ```
pub fn integrate_and_push(
    git: &PushGit,
    plan: &Reconcile<'_>,
    rejected: PushAttempt,
    push_merge: &mut dyn FnMut(&Destination, &str) -> Result<PushAttempt, SingleFlightError>,
) -> Result<Integrated, SingleFlightError> {
    let destination = plan.destination.clone();
    let local = plan.local.clone();
    let label = plan.label;
    let line_prefix = format!("cli-git: {label}: ");
    let failed_prefix = format!("{line_prefix}not published: ");
    let short = destination
        .merge_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&destination.merge_ref)
        .to_owned();
    let message = format!(
        "Merge {}/{short} into {short}\n\nCreated by cli-git {label} because {}/{short} advanced after the commit landed.",
        destination.remote, destination.remote
    );
    let mut previous = rejected.output.clone();
    // The remote tip a merge was pushed against, with the refusal it got: a remote that still shows that tip did not
    // move, so the refusal was a rule and not a race.
    let mut refused_against: Option<(String, PushAttempt)> = None;
    for attempt in 1..=MAX_RECONCILE_ATTEMPTS {
        let remote_tip = match fetch_remote_tip(git, &destination)? {
            Ok(tip) => tip,
            Err(failed) => {
                if plan.explicit && refused_against.is_none() {
                    debug(
                        TAG,
                        "the remote could not be fetched after an explicit push failed; its own report stands",
                    );
                    return Ok(Integrated::NotDivergent(rejected));
                }
                return Ok(Integrated::Finished(unpublished(
                    &failed_prefix,
                    &format!("{previous}\n{}", failed.output),
                    failed.exit_code,
                    &format!(
                        "{}/{short} has commits this branch lacks, and fetching them failed; nothing was pushed and the worktree was not touched.",
                        destination.remote
                    ),
                )));
            }
        };
        if let Some((tip, refusal)) = refused_against.take()
            && tip == remote_tip
        {
            debug(
                TAG,
                "the remote did not move after it refused the merge; the refusal is final",
            );
            return Ok(Integrated::Finished(refusal));
        }
        match (
            is_ancestor(git, &remote_tip, &local)?,
            is_ancestor(git, &local, &remote_tip)?,
        ) {
            (Some(true), _) => {
                debug(
                    TAG,
                    "the remote tip is behind the branch; the rejection was not a divergence",
                );
                return Ok(Integrated::NotDivergent(rejected));
            }
            (_, Some(true)) if plan.explicit => {
                debug(
                    TAG,
                    "the remote already contains the pushed commit; the explicit push failed for another reason",
                );
                return Ok(Integrated::NotDivergent(rejected));
            }
            (_, Some(true)) => {
                return Ok(Integrated::Finished(PushAttempt {
                    exit_code: 0,
                    output: format!(
                        "{line_prefix}{}/{short} already contains {local}; nothing to push.",
                        destination.remote
                    ),
                }));
            }
            (Some(false), Some(false)) => {}
            _ => {
                if plan.explicit {
                    return Ok(Integrated::NotDivergent(rejected));
                }
                return Ok(Integrated::Finished(unpublished(
                    &failed_prefix,
                    &previous,
                    rejected.exit_code,
                    "Git could not relate the branch to the remote tip; nothing was pushed and the worktree was not touched.",
                )));
            }
        }
        let merge_oid = match commit_merge(git, &local, &remote_tip, &message)? {
            Ok(oid) => oid,
            Err(paths) => {
                return Ok(Integrated::Finished(unpublished(
                    &failed_prefix,
                    &previous,
                    rejected.exit_code,
                    &format!(
                        "merging {}/{short} into {short} conflicts in: {}. Nothing was pushed and the worktree was not touched; \
                         merge {}/{short} in a separate worktree, resolve the conflicts and push.",
                        destination.remote,
                        paths.join(", "),
                        destination.remote
                    ),
                )));
            }
        };
        debug(
            TAG,
            &format!(
                "attempt {attempt}: pushing the merge {merge_oid} of {local} and {remote_tip}"
            ),
        );
        let pushed = push_merge(&destination, &merge_oid)?;
        if pushed.exit_code == 0 {
            let branch = plan
                .branch_ref
                .map(|reference| {
                    return reference.strip_prefix("refs/heads/").unwrap_or(reference);
                })
                .unwrap_or("HEAD");
            let advanced = match plan.branch_ref {
                Some(branch_ref) => advance_local_branch(git, branch_ref, &local, &merge_oid)?,
                None => Err(String::from("HEAD is detached")),
            };
            let followed = match advanced {
                Ok(false) => format!("{branch} and the worktree were fast-forwarded to it."),
                Ok(true) => format!(
                    "{branch} and the worktree were fast-forwarded to it; your uncommitted changes were left as they are."
                ),
                Err(reason) => {
                    debug(TAG, &format!("{branch} was not fast-forwarded: {reason}"));
                    format!(
                        "{branch} itself and the worktree are unchanged ({reason}); update {branch} with `git merge --ff-only {}/{short}` \
                         when the worktree allows.",
                        destination.remote
                    )
                }
            };
            return Ok(Integrated::Finished(PushAttempt {
                exit_code: 0,
                output: format!(
                    "{}\n{line_prefix}{}/{short} had new commits, so the merge {merge_oid} of {branch} and {}/{short} \
                     was published. {followed}",
                    pushed.output, destination.remote, destination.remote
                )
                .trim_start_matches('\n')
                .to_owned(),
            }));
        }
        if !plan.explicit && !was_rejected_as_stale(&pushed.output) {
            return Ok(Integrated::Finished(pushed));
        }
        previous = pushed.output.clone();
        refused_against = Some((remote_tip, pushed));
    }
    return Ok(Integrated::Finished(unpublished(
        &failed_prefix,
        &previous,
        rejected.exit_code,
        &format!(
            "{}/{short} advanced again during each of {MAX_RECONCILE_ATTEMPTS} attempts, so nothing was pushed and the worktree was not touched. \
             Push again when the remote is quiet.",
            destination.remote
        ),
    )));
}

/// Reconcile behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_reconcile_tests.rs"]
mod tests;
