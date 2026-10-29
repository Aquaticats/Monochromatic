# Stack concurrent pushes in the git wrapper (issue 1804)

Status: accepted for implementation on 2026-10-29 (the user chose "A and B together in one release").
Design findings made while implementing are recorded in the "Implementation findings" section.
Issue: <https://github.com/Aquaticats/Monochromatic/issues/1804>.

## Evidence

Reproduced 2026-10-29 in a capped podman container (2 GiB, 2 CPUs, repositories on the container's tmpfs) with the
installed wrapper `cli-git-native` linked as `git` ahead of the real git 2.56.0 on `PATH`.
The harness checks that `git` resolves to the wrapper before it starts.
Harness: `~/temp/agent/podman-repro-1804.ts`; log: `~/temp/agent/podman-1804/repro.log`.
A first run was invalid, because the link step failed and every call ran plain git; it was discarded and rerun.

The issue bundles three separate failures.

### Failure 1: an explicit push is never integrated

A linked worktree on a detached HEAD, one commit above `origin/main`, while another clone pushed to `main` in between:

```text
$ git push origin HEAD:main
error: atomic push failed for ref refs/heads/main. status: 5
 ! [rejected]        HEAD -> main (fetch first)
```

Fetch, rebase, and push again was rejected a second time after another clone pushed in between.
The same rejection happened from a linked worktree branch.
The wrapper only inserts `--atomic` into an explicit push (`src/native/rule_atomic_push.rs`) and runs the push-time
policies over the pushed range (`src/native/manual_push_lifecycle.rs`).
It has no integration step for an explicit push.

### Failure 2: a detached HEAD never auto-pushes

```text
cli-git: HEAD is detached (mid-rebase/cherry-pick or a detached checkout); skipping auto-push.
```

This is deliberate (`DETACHED_HEAD_NOTE`, `src/native/auto_push.rs`).
It stays a separate case: auto-push needs a branch name, and failure 1 covers the explicit push from a detached HEAD.

### Failure 3: a branch whose name differs from its upstream can never auto-push

A linked worktree created with `git worktree add -b feature <path> origin/main` tracks `origin/main`.
With `push.default` unset (git's `simple`), every commit's auto-push fails, with no race needed:

```text
fatal: The upstream branch of your current branch does not match
the name of your current branch.  To push to the upstream branch
on the remote, use

    git push origin HEAD:main
cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.
```

Git refuses the plain `git push` before contacting the remote, so the output has no `[rejected]` marker, and
`reconcile_divergent_push` (`src/native/auto_push_reconcile.rs`) is never reached.
`read_destination` in the same file already names this destination: for a non-triangular `simple` setup it is the
tracked ref (`refs/heads/main`).

### Control: auto-push on a checked-out `main` already stacks

A commit in the main worktree on `main`, with the remote moved after the clone, published a merge and fast-forwarded:

```text
cli-git: auto-push: origin/main had new commits, so the merge d4f13a2f... of main and origin/main was published.
main and the worktree were fast-forwarded to it.
```

The mechanism (`reconcile_divergent_push`) fetches into a private ref, builds the merge with `git merge-tree
--write-tree` and `git commit-tree`, pushes it, and retries up to `MAX_RECONCILE_ATTEMPTS` (10).
It never touches the worktree before the push, and stops with the conflicting paths on a real conflict.

## Proposal

Reuse the existing reconcile loop instead of writing a second one.

### Change A: auto-push to the tracked ref when the names differ

When the branch has an upstream and git would refuse the plain `git push` for a name mismatch, push explicitly to the
destination `read_destination` already computes (`git push <remote> HEAD:<merge-ref>`) and then reconcile as today.
This fixes failure 3 and needs no policy decision beyond the one the user already made by tracking `origin/main`.

### Change B: reconcile an explicit push that was rejected as stale

When the wrapper forwarded an explicit `git push` and the result matches `was_rejected_as_stale`:
- Take the destination from the push's own refspec (`<src>:<dst>`, or a branch name) and the remote argument, not from
  branch configuration.
- Resolve `<src>` to a commit (works for `HEAD`, including detached).
- Run the existing loop: fetch the remote tip into a private ref, merge in the object store, push the merge, retry up to
  10 times, report each integration.
- Re-run the push-time policies on what the merge adds (the range from the remote tip to the merge) before each push,
  so a merge that brings in refused content stops.
- On a real conflict, stop and name the paths without retrying.
- Leave `--dry-run`, force (`--force`, `--force-with-lease`, `+refspec`), deletes, tags, and multi-ref pushes unchanged.
- After success, fast-forward the checked-out branch only when `<src>` is that branch and the worktree allows it, as
  `advance_local_branch` does; a detached HEAD or a differently named source stays where it is, and the report says so.

## Decisions

The user answered the three questions below on 2026-10-29, taking the recommended option each time:
merge, any non-force single-branch update, and leave a detached HEAD where it is while printing the merge commit.
The user then accepted the proposal for implementation, as one release for both changes.

## Decision questions as asked

1.  Integration style. Recommended: merge (as auto-push does today), because it never rewrites a commit the caller may
    already have shared, and the loop exists and is mutation-tested.
    Rebase would give linear history but rewrites local commits and needs a worktree.
2.  Scope of explicit pushes. The issue says the default branch.
    Recommended: any non-force, single-ref, non-delete update of a branch, because the rejection mechanics are the same
    and a default-branch-only rule needs the default branch name resolved from the remote.
3.  Detached HEAD after a successful explicit push. Recommended: leave HEAD where it is and print the merge commit.

## Implementation findings

Read from source on 2026-10-29, before any code.

- A plain forwarded `push` replaces the wrapper process with real git (`replace_process_with_real_git`,
  `src/native/forwarding.rs`), so the wrapper cannot see the rejection.
  Commands that need the wrapper alive already run git as a child with inherited streams
  (`Action::ForwardSettled`, `Action::ForwardCoordinated`, `run_forwarded_git`), including pushes from linked worktrees
  that need the worktree copy.
  Change B therefore runs an eligible push as a child with inherited streams, so the terminal output is unchanged.
- No output parsing is needed for an explicit push: after a non-zero exit, the loop fetches the destination tip and
  compares ancestry, and it already returns the original result when the remote is not a divergence
  (authentication, offline and hook refusals fetch-fail or find the remote behind).
  The explicit path must stay silent in those cases, unlike auto-push, which reports a failed fetch.
- `probe_manual_push_updates` (`src/native/manual_push_probe.rs`) already resolves, per update, the remote name, the
  destination ref, the local object and the remote's current object via `ls-remote`.
  Change B reuses it after a failure; no refspec parser is written.
- Policies on the merge: the merge is pushed by the wrapper invoking itself (`push <remote> <merge>:<ref>`) with an
  environment marker that disables a second reconcile, so the normal pre-forward and manual-push gates run on the
  integrated result with no second policy path.
  Auto-push's own merge push stays as it is in this release (it pushes with real git and runs no push-time policy);
  that gap is recorded in the handover and not changed here, because a gate on it would stop merges that publish today.
- Locking: the reconcile for an explicit push is keyed by the destination ref, since a detached HEAD has no branch.

## Risks

- The push-time policy run over a merge is the least understood part: its input range and the interaction with the
  forbidden-strings scan (`doc/troubleshooting/forbidden-strings-builtin-rule-fixture-tokens.md`) must be read from
  `manual_push_probe.rs` and `manual_push_range.rs` before the design is final.
- Explicit pushes would gain network round trips on a rejection (a fetch per attempt); only a rejected push pays them.
- Published merge commits from explicit pushes add history; this is the same trade-off auto-push already made.

## Verification plan

- Unit tests for the refspec destination parser, and for each excluded form staying unchanged.
- Binary tests in a bare-remote fixture for: detached-HEAD push, branch-name mismatch auto-push, a conflict, a policy
  refusal on the merge, the 10-attempt exhaustion, and `--dry-run`/force unchanged.
- The podman harness above, rerun after the change, expected to show the three failures gone.
- Mutation campaign over every touched file, not blocking the release.
