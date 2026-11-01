# Cli-git journal schema 2 recovery rejects a superseded index

## Symptom

On 2026-10-03,
`git status --short` in the private auto-mode qualification repository exited 2.
The PATH-shadowing `git-policy-cli` emitted JSONL with code `content-unavailable` and this message prefix:

```text
Real index no longer matches the pre-landing snapshot; recovery retained at
```

The recorded transaction was `ed7473e1-079a-406a-96e2-8fc5afb2490e` under `.git/cli-git-transactions/`.
It was later archived,
with authorization,
to `.git/cli-git-transactions-archive/` on 2026-10-03.
Its directory name did not change.

This was separate from a copy-boundary stager failing to find the installed Pi package's `LICENSE` file.
No causal relationship between those failures is asserted.

## Environment

The executable was the repo-owned `package/git-policy/cli/dist/final/node/index.mjs`.
Its observed SHA-256 was `a418ca48f202850cbda6943efe8eaaad02488c16aad5d9e7321ea10af22894cd`.
The recovery source's last commit was `8ed5daa61b819a4713b05f51a28edccaa8e54bbd`.
The journal schema was 2;
the package manifest supplied no package version field.

The transaction belonged to commit `a6736388fd261c625b56ee6ad4bc7ce0b4b61346`.
That invocation printed commit success before its command tool timed out.
The durable `ref-updated.json` named the same commit.
Later commits had advanced the private repository to `e91b58fe5e06187834211fe3b2f7c6785b57f139`.
The exact point where the interrupted invocation stopped was not instrumented.

## Root cause of the recovery refusal

`package/git-policy/cli/src/policy-engine/commit-transaction-recovery-completion.ts:209`
recognizes an installed marker or an index equal to the retained post-index.
At line 218,
it refuses to install the old post-index when the current index also differs from the retained pre-index:

```typescript
// package/git-policy/cli/src/policy-engine/commit-transaction-recovery-completion.ts:218
if (!(await snapshotFilesEqual({
  leftPath: preLanding,
  rightPath: preparing.realIndexPath,
},)))
  throw new CommitTransactionRecoveryError(`Real index no longer matches the pre-landing snapshot; recovery retained at ${directory}`,);
```

The refusal protects a different current index from being overwritten.
It does not establish that the current index is corrupt.
In this incident,
read-only forensic commands showed the current index matched the later `HEAD`,
and that the transaction commit was its ancestor.
The retained pre/post snapshots described an older landing,
not the current repository state.

No generic timeout cause or defect in native Git was established.
The historical post-commit tree-latency incident is not used as evidence for this interruption.

## Verification

The private harness is `contract/diagnostic/superseded-index-recovery/`.
Successful checks included:

-   Native read-only status and cached diff showed no staged or tracked worktree differences before recovery.
-   `merge-base --is-ancestor` confirmed the old landed commit belonged to current history.
-   The old transaction owner PID no longer existed.
-   The approved recovery preserved 21 journal files with identical bytes,
    modes,
    device IDs,
    and inodes.
-   Index bytes,
    `HEAD`,
    refs,
    and optional packed refs remained unchanged.
-   Normal wrapper `status` and `log` commands exited 0 after archival.

The failing catalog is distinct:

-   Automatic wrapper recovery rejected the superseded index with `content-unavailable`.
-   The first authored recovery program had a mismatched bracket and failed parsing with
    `SyntaxError: missing ) after argument list`.
    It executed no recovery mutations.
    Its source and `proc_4a0a` logs were retained.
-   The corrected `recover-v2.mjs` passed a syntax check and completed as `proc_aac3`,
    exit 0.

The successful recovery invocation was:

```sh
# Private contract/diagnostic/superseded-index-recovery/mise.toml
mise --no-env --no-hooks run recover-v2
```

The consumed recovery is not replayable against the now-archived source directory.
Its `before.json`,
`result.json`,
and `wrapper-reprobe.json` retain the checks and final disposition.

## Verified workaround and tradeoffs

An explicit session trust rule authorized evidence-preserving recovery without index or ref changes.
The script verified the exact transaction,
landed commit,
current `HEAD`,
clean current index,
dead owner,
and journal contents before moving the transaction out of the automatic recovery scan.
It wrote a supersession disposition beside the archive.

This is an incident-specific administrative resolution,
not a change to cli-git's recovery algorithm.
It preserves evidence but stops automatic replay of that superseded transaction.
It is not appropriate when current index contents,
commit ancestry,
owner liveness,
or artifact ownership are uncertain.

## What was not used

The recovery did not install an old index,
reset a ref,
delete the journal,
or fabricate an `index-installed` marker.
The printed commit-success line alone was insufficient evidence of complete wrapper finalization.
The later committed history and clean current index were independently checked.

## Separate empty index lock on 2026-10-09

The governing-domain documentation checkpoint `proc_377d` failed before `git add`.
Cli-git emitted `index-lock-unproven-owner` with exit 2.
This was an empty main-repository `.git/index.lock`,
not the historical superseded journal incident.

The native wait preserves a lock when its owner is not proven alive
and the configured unproven-owner budget expires:

```typescript
// package/git-policy/cli/src/index-lock/index-lock-wait.ts:350
if (state.unprovenMs >= timeoutMs)
  throw new IndexLockUnprovenOwnerError({
    evidence,
    verdict,
    timeoutMs,
    consequence,
  },);
```

The diagnostic reported no PID file,
no open descriptor found for the lock,
and inaccessible process-descriptor directories.
That output alone was not treated as proof of a dead owner.
Subsequent current-user process inspection found only Git filesystem-monitor daemons,
not a Git writer.
The unchanged lock was user-owned,
zero bytes,
and inode `185394298`.

After those checks,
the recovery compared the exact inode,
size,
and user ID before moving the lock without overwriting its destination
to `~/temp/agent/auto-mode-main-index-lock-185394298.empty`.
No index content,
journal,
or ref was restored.
The scoped documentation commit then passed as `proc_10e4`.
The lock creator and cause of its abandonment remain unestablished.

This is not a generic permission to delete `index.lock`.
An active or uncertain writer requires preserving its lock.
The incident demonstrates an evidence-preserving local recovery,
not a defect in Git or cli-git's refusal.

## Upstream filing decision

No external upstream issue was drafted or filed.
This is repo-owned recovery behavior,
not a demonstrated fault in native Git or the Pi process harness.

-   Upstream fault:
     not established for an external project.
-   Fixability:
     no generic recovery change was designed or qualified by this incident.
-   Supported use:
     local commit interruption is documented;
    conflicting index recovery deliberately fails closed.
-   Contribution policy:
     external contribution policy is not applicable to this repo-owned resolution.
-   Likelihood of an external fix:
     not assessed because no external defect is claimed.
-   Prototype:
     the incident-specific administrative recovery was exercised;
    no generic code fix is represented as tested.

`.out-of-scope/` was inspected;
no external filing is relevant to this local resolution.
There is no external issue draft or additive comment to publish.

## Separate owner-probe refusal on 2026-10-19

The protected `.cache/cli-git-bin/git log --max-count=3 --oneline` failed with exit 2
while a supervised rootless-container worker checkpointed package changes.
The native wrapper emitted `engine-failure`, code `content-unavailable`:

```text
# Native recovery diagnostic, distinct from superseded-index recovery.
probing transaction owner 293 failed: Operation not permitted (os error 1)
```

An explicitly read-only `/usr/bin/git log --max-count=3 --oneline` succeeded
and showed the intended five-path commit `3a21255dde` had landed.
The protected wrapper subsequently succeeded during frozen-source preparation.
No journal was deleted and no commit was repeated.

`package/git-policy/cli/src/native/recovery_inspect.rs`, `liveness`, propagates the refused signal probe.
`transaction_owner.rs` records the current process PID;
`classify_transaction_owner` probes existence before comparing birth identity.
Shared repository access across PID namespaces is a hypothesis requiring a disposable reproduction,
not an established cause of this refusal.
Denied permission does not establish that an owner is dead.

[Repository issue #1784](https://github.com/Aquaticats/Monochromatic/issues/1784)
tracks read availability and namespace-aware ownership controls.
This is separate from the unrelated LFS incident in #1770
and the missing post-commit completion-marker directory in #1781.
No shared cause is established.
