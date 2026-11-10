# cli-git 0.0.1: linked-worktree diff can time out behind a live worktree-copy owner

Note 2026-11-09T16:36:36-05:00:
`package/git-policy/cli/src/worktree-copy/` is not in the tree;
its files were deleted on 2026-10-13 in `642b6d72cb`.

## Symptom

During a debug-only Fold Search prototype,
`git -C /home/user/temp/agent/music-player-theme-compose-prototype diff --check`
exited `2` and emitted:

```text
cli-git: timed out waiting for active worktree-copy settlement under "/var/home/user/Monochromatic/.git".
```

The linked prototype's `--git-dir` resolved to
`/var/home/user/Monochromatic/.git/worktrees/music-player-theme-compose-prototype`;
its common directory was `/var/home/user/Monochromatic/.git`.
This is **not** the older main-worktree applicability bug described in
[cli-git-worktree-copy-settlement-timeout.md](cli-git-worktree-copy-settlement-timeout.md).

## Root cause and boundary

`package/git-policy/cli/src/worktree-copy/git-observer.ts:110-146` excludes
main worktrees but returns a worktree-copy observation for linked sources:

```ts
if (identity.kind === 'main-worktree') {
  rl.debug('effective invocation targets main worktree; worktree copy observation is not applicable',);
  return WORKTREE_COPY_NOT_APPLICABLE;
}
```

The paths this record names under `package/git-policy/cli/src/worktree-copy/` were removed on
2026-10-13 in `642b6d72cb` (refactor(*):
delete the TypeScript Git policy wrapper,
its tasks,
bins
and trust workflow);
no file in the working tree carries them now.

The linked source reaches the returned observation at
`git-observer.ts:138-146`:

```ts
return {
  adminRoot,
  beforeAdminIds,
  commonDir: identity.commonDir,
  effectiveCwd: identity.effectiveCwd,
  ...(identity.kind === 'linked-worktree'
    ? { sourceRoot: identity.worktreeRoot, }
    : {}),
};
```

`package/git-policy/cli/src/worktree-copy/lifecycle.ts:290` takes the
common-directory settlement lock before forwarding even a read-only Git
command from such a source:

```ts
await using settlementLock = await acquireWorktreeCopyLock(initialObservation.commonDir,);
```

`package/git-policy/cli/src/worktree-copy/journal-lock.ts:25-30,497-515`
uses 100 attempts separated by 10ms and then emits the exact timeout:

```ts
const LOCK_RETRY_DELAY_MS = 10;
const LOCK_RETRY_ATTEMPTS = 100;
```

At `journal-lock.ts:508-515` the busy result waits and eventually throws:

```ts
if (result !== LOCK_BUSY)
  return result;
// oxlint-disable-next-line no-await-in-loop -- retry delay prevents active-owner spin
await wait(LOCK_RETRY_DELAY_MS,);
}
throw new WorktreeCopyError(
  `cli-git: timed out waiting for active worktree-copy settlement under ${JSON.stringify(commonDir,)}.`,
);
```

At this incident's inspection,
`settlement.lock/owner.json` named PID `970928` and a birth identity.
`ps --pid 970928 --format pid,ppid,etime,stat,args` showed a live cli-git
`worktree add --detach /var/home/user/worktrees/ccc-premerge 5f513eb64`
process.
The lock therefore had a **live concurrent owner**;
no stale-lock inference or manual cleanup is justified.
Whether that worktree-add took longer than expected and why it was running
are separate questions not diagnosed here.

## Verification

The source revision under examination was the current repository-owned
`@monochromatic-dev/git-policy-cli` `0.0.1` implementation.
Use the following read-only checks while a different applicable linked
worktree command demonstrably owns the same settlement lock:

```sh
/usr/bin/git -C /home/user/temp/agent/music-player-theme-compose-prototype rev-parse --show-toplevel --git-dir --git-common-dir
git -C /home/user/temp/agent/music-player-theme-compose-prototype diff --check
/usr/bin/git -C /home/user/temp/agent/music-player-theme-compose-prototype diff --check
```

The wrapped linked `diff --check` failed with the quoted diagnostic and exit
`2`;
the absolute real-Git read-only diff exited `0` on the same source files.
The observer's main-worktree gate avoids this linked-source observation path;
this is a source-class distinction,
not evidence that all Git invocations time out.
The check isolates inspection capability,
not commit or worktree-copy correctness.

## Verified workaround

For **read-only inspection while the owner remains live**,
use `/usr/bin/git -C <linked-worktree> diff --check`.
The observed invocation exited `0` without waiting for worktree-copy
settlement.
Its tradeoff is that absolute real Git bypasses cli-git policies,
transaction recovery and automation,
so do **not** generalize this bypass to staging,
commits or worktree mutations.
Defer those wrapped mutations until the concurrent owner finishes.
No lock file or unrelated process was removed.

## What does not work

- Retrying the wrapped linked command while the same live owner still holds
  the lock only repeats a bounded timeout;
  this was not used as a recovery strategy.
- Deleting `settlement.lock` would race the live owner and risk its
  transaction;
  this was **not** attempted.
- Treating the main-worktree applicability fix as a linked-worktree bypass
  misclassifies this invocation.

## Upstream filing decision

No external upstream filing is applicable.
Cli-git is owned by this repository,
`.out-of-scope/` has no matching exemption,
and this run has not established a tool defect:

1.  Upstream fault is unproven;
    serialization with a live linked-worktree owner matches the inspected path.
2.  The implementation could change its read-only applicability rule,
    but no fix is prescribed from this single contention observation.
3.  Linked-worktree copy is an intentionally supported path.
4.  The code belongs to this repository,
    so no third-party contribution policy applies.
5.  No maintainer rejection or remediation request was established.
6.  No compatible patch was prototyped because the diagnosed state was live
    contention,
    not a reproduced fault.

No external issue or comment draft is prepared.
