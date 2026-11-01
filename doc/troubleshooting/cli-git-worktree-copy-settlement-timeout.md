# cli-git 0.0.1: main-worktree status wrongly entered worktree-copy settlement and could exit 2

## Symptom

This command ran in the main worktree:

```sh
git status --short && git log -6 --oneline --decorate
```

Cli-git emitted:

```text
cli-git: timed out waiting for active worktree-copy settlement under
"/var/home/user/Monochromatic/.git".
```

The main-worktree identity is measurable:

```text
$ /usr/bin/git rev-parse --show-toplevel --git-dir --git-common-dir
/var/home/user/Monochromatic
.git
.git
```

Equal canonical Git and common directories identify the main worktree.
That target should not enter linked-worktree administrative observation,
journal recovery,
settlement locking,
or ignored-state synchronization.

The exact diagnostic comes from cli-git,
not real Git and not the command harness.
The harness's `(timeout 30s)` annotation is an outer limit;
cli-git exited first with its own code `2`.
Because `git status --short` failed,
shell `&&` did not start `git log`.

## Root cause

The earlier diagnosis in this file treated live lock contention as the problem.
That was incomplete.
Contention explained how the message was emitted,
but not why main-worktree `status` tried to acquire this lock.
The applicability bug was the cause at the user boundary.

### Pre-fix observer treated every effective repository as applicable

At pre-fix commit `8efcf4538799f792976ef0d761c60cd0f248a032`,
`package/git-policy/cli/src/worktree-copy/git-observer.ts:302-344` resolved only the common directory,
then immediately read linked-worktree administration and returned an observation:

```ts
const commonDir = await resolveCommonDir({
  gitPath,
  preSubcommandArgs,
  invocationCwd,
},);
if ((typeof commonDir) === 'symbol') {
  return WORKTREE_COPY_NOT_APPLICABLE;
}
const adminRoot = join(
  commonDir,
  'worktrees',
);
const beforeAdminIds = await readAdminIds(adminRoot,);
const sourceRoot = await resolveSourceRoot({
  gitPath,
  preSubcommandArgs,
  invocationCwd,
},);
return {
  adminRoot,
  beforeAdminIds,
  commonDir,
  effectiveCwd,
  ...((typeof sourceRoot) === 'symbol' ? {} : { sourceRoot, }),
};
```

No invocation-specific Git directory was resolved.
The observer therefore could not distinguish:

- main worktree:
   canonical Git directory equals canonical common directory;
- linked worktree:
   canonical Git directory differs from canonical common directory;
- bare repository:
   no source worktree root.

### Lifecycle locked every observation

`package/git-policy/cli/src/worktree-copy/lifecycle.ts:269` acquires settlement for every returned observation:

```ts
await using settlementLock = await acquireWorktreeCopyLock(initialObservation.commonDir,);
```

A plain main-worktree `status` therefore created
`.git/cli-git-worktree-copy/v1`,
acquired `settlement.lock`,
and became vulnerable to another invocation retaining that lock through the bounded acquisition loop.

### Initial fix established behavior but duplicated package knowledge

Commit `230f78959153195cbe01b0497d977dcac84fab71` first changed
`package/git-policy/cli/src/worktree-copy/git-observer.ts` to resolve both source and invocation-specific Git
administration before reading linked identities:

```ts
const commonDir = await resolveCommonDir({
  gitPath,
  preSubcommandArgs,
  invocationCwd,
},);
if ((typeof commonDir) === 'symbol')
  return WORKTREE_COPY_NOT_APPLICABLE;

const sourceRoot = await resolveSourceRoot({
  gitPath,
  preSubcommandArgs,
  invocationCwd,
},);
if ((typeof sourceRoot) === 'string') {
  const gitDir = await resolveGitDir({
    gitPath,
    preSubcommandArgs,
    invocationCwd,
  },);
  if (gitDir === commonDir)
    return WORKTREE_COPY_NOT_APPLICABLE;
}
```

This established correct behavior,
but keeping `gitDir === commonDir` inside the worktree-copy observer duplicated identity knowledge already needed by
linked-worktree policy classification.
That shape was rejected as a shallow caller-local fix.

### Shared worktree identity module owns classification

Commit `3e14beaba75a0d99e01a19cfe53bccce4dd9a3cc` moves raw argument parsing,
exact Git target selection,
canonical path resolution,
and outside,
bare,
main,
and linked classification behind
`resolveGitWorktreeIdentity` in `package/git-policy/cli/src/git-worktree-identity.ts:290-370`.
Commit `4b7d8422f9845d85563d0f7a146d928dfc4b8816` also recognizes an explicit `--git-dir` plus `--work-tree` target even
though Git reports `--is-inside-work-tree=false` from an unrelated launch directory.

The shared module owns the identity decision:

```ts
return {
  kind: gitDir === commonDir
    ? 'main-worktree'
    : 'linked-worktree',
  commonDir,
  effectiveCwd,
  gitDir,
  worktreeRoot,
};
```

Worktree copy now only maps shared identity to its lifecycle decision at
`package/git-policy/cli/src/worktree-copy/git-observer.ts:105-117`:

```ts
const identity = await resolveGitWorktreeIdentity({
  args,
  gitPath,
},);
if (identity.kind === 'outside-worktree')
  return WORKTREE_COPY_NOT_APPLICABLE;
if (identity.kind === 'main-worktree')
  return WORKTREE_COPY_NOT_APPLICABLE;
```

`package/git-policy/cli/src/effective-target.ts:73-87` consumes the same identity and adds only policy-specific
allowlisting.
Only after the worktree-copy gate does the observer read linked administrative identities.
The lifecycle receives the not-applicable sentinel for a main worktree and forwards real Git without recovery,
locking,
or synchronization.
Linked sources and bare empty-source behavior remain applicable.

## Verification

Verified on 2026-07-22 with:

- `@monochromatic-dev/git-policy-cli` `0.0.1`;
- real Git `2.55.0`;
- failing regression commit `8efcf4538799f792976ef0d761c60cd0f248a032`;
- fix commit `230f78959153195cbe01b0497d977dcac84fab71`;
- linked-source fixture commit `0ca54c3512879e16d8237ed26dd57fa41ff3dac6`;
- shared identity refactor commit `3e14beaba75a0d99e01a19cfe53bccce4dd9a3cc`;
- explicit work-tree identity fix commit `4b7d8422f9845d85563d0f7a146d928dfc4b8816`;
- fixed bundle SHA-256
  `f8e5a946e49b6926051e9b60ea06fbd777a4ff3210706b1faadce6af38f4a940`.

### Regression harness

The built-wrapper test
`package/git-policy/cli/src/worktree-copy.unit.test.ts` creates a disposable main worktree,
runs wrapped `status`,
creates a linked worktree through wrapped `worktree add`,
and asserts all of these outcomes:

- `.git/cli-git-worktree-copy` is never created in the main repository;
- main-worktree status succeeds;
- worktree creation still forwards to real Git;
- ignored main-worktree state is not copied;
- no copy summary is emitted.

Before the fix,
the test failed at its first applicability assertion:

```text
AssertionError: expected [..., 'cli-git-worktree-copy', ...]
to not include 'cli-git-worktree-copy'
```

After the fix,
the same built-wrapper test passes.

A separate disposable boundary probe retained a live settlement lock from a linked source while invoking
`git -C <main> status --short` through the fixed bundle.
Main status returned exit `0` in `0.16` seconds,
and the linked holder process and exact `owner.json` record remained live afterward.
This directly verifies that main status no longer waits on linked-source settlement.

### Behaviors that now work cleanly

- Main-worktree `git status --short` forwards without creating worktree-copy state.
- Main-worktree `git worktree add` forwards without copying ignored state or acquiring settlement.
- A linked source worktree still copies ignored state for direct `worktree add` forms.
- A linked source worktree still detects ordinary aliases that create worktrees.
- Bare repositories still synchronize an empty source set.
- Interrupted journals still recover from a later applicable linked-worktree or bare-repository invocation.

### Pre-fix behavior that failed

- Main-worktree `status` created the worktree-copy journal root despite creating no worktree.
- Concurrent main-worktree commands could then hit the settlement timeout and exit `2`.
- Retrying could hide the applicability defect after contention ended.

## Verified workarounds

### Upgrade or rebuild cli-git with the applicability fix

This is the complete correction.
Main-worktree commands no longer enter settlement,
so they do not need a copy-specific flag or real-Git bypass.

Tradeoff:
main-worktree commands that create linked worktrees no longer bootstrap those destinations with ignored main-worktree
state.
Run creation from an applicable linked source when copying is wanted.

### Pre-fix read-only inspection with the wrapper opt-out

On an older artifact,
this avoids worktree-copy lifecycle for one read-only invocation:

```sh
git status --no-worktree-copy --short
git log --no-worktree-copy -6 --oneline --decorate
```

Tradeoff:
this requires callers to remember a flag for behavior that should be automatic,
and it also skips worktree-copy transaction recovery for that invocation.

### Pre-fix forensic real-Git bypass

```sh
/usr/bin/git status --short
/usr/bin/git log -6 --oneline --decorate
```

Tradeoff:
an absolute real-Git path bypasses all cli-git policies,
fixed transforms,
transaction recovery,
and auto-push behavior.
Use it only for deliberate inspection.

## What does not work

- **Explaining only the lock owner.
  **
  A live owner explains the timeout mechanism but does not justify main-worktree applicability.
- **Retrying until contention ends.
  **
  Retry can succeed while leaving the wrong lifecycle boundary intact.
- **Increasing the outer command timeout.
  **
  It does not change cli-git's internal acquisition constants or applicability.
- **Deleting `settlement.lock`.
  **
  Manual deletion races the live owner and can convert contention into an ownership failure.
- **Keeping copy tests rooted only in main repositories.
  **
  Those fixtures encoded the bug as expected behavior.
  Copy scenarios now use linked sources,
  while a separate regression test covers main-worktree bypass.

## Linked-source guard-worktree creation on 2026-09-11

This incident is separate from the main-worktree applicability bug.
The attempted creation was launched from linked worktree
`/var/home/user/worktrees/translation-repair`,
not the main worktree.
Its invocation-specific Git directory was
`/var/home/user/Monochromatic/.git/worktrees/translation-repair`,
while its common directory was `/var/home/user/Monochromatic/.git`.
Copy applicability was therefore expected.

### Observed boundary

The agent requested a detached verification worktree under its private scratch root.
Real Git completed checkout,
but wrapper PID 2089648 retained repository-wide settlement while copying ignored state.
A scoped commit in the source worktree then received cli-git's exact settlement-timeout diagnostic.
The owner record identified that same PID and its process birth identity.
At one observation the process had run for 8 minutes 24 seconds
and reported RSS 1845524 KiB.

The private stage was
`~/temp/agent/.cli-git-worktree-copy-aW97uD`.
Its measured apparent payload size was 33859161062 bytes.
That is not a claim of physical disk consumption:
`package/git-policy/cli/src/worktree-copy/snapshot.ts:47`
requests copy-on-write with fallback:

```ts
// package/git-policy/cli/src/worktree-copy/snapshot.ts
const COPY_MODE = constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE;
```

The deciding lifecycle remains
`package/git-policy/cli/src/worktree-copy/lifecycle.ts:269`:
the exclusive lease covers real Git and the subsequent ignored-state synchronization.
Thus successful checkout output does not mean the wrapper has finished or released settlement.
The linked wrapper bundle used here had SHA-256
`be37f0e5de9cc0622b023846e96fbc66ab5df81c64728c0b78f68351c447f2d2`.

### Recovery performed

- Stopped only the agent-started copier through the process manager.
- Verified the process was absent.
- A later wrapped linked-worktree status recovered the stale active lease without manual active-lock deletion.
- A Node recursive cleanup was stopped after its own resource use and a stop-manager timeout were observed.
  The operating-system PID check subsequently confirmed it was gone;
  the manager's `terminate_timeout` label was not treated as proof of a live process.
- Removed only the identified owned stage with native `rm`,
  idle I/O priority,
  one-filesystem traversal and root preservation.
  That invocation completed in 58 seconds.
- Verified the stage no longer existed.
- A synchronous commit had hit the command tool's outer timeout.
  Its process was absent and HEAD had not advanced,
  so the commit was retried through the managed process tool,
  not blindly repeated synchronously.
  It succeeded as `9f4034d15`.
- Removed only the abandoned `.pending` claim created by that timed-out commit,
  after checking its exact owner PID 2091885 and recorded birth identity and proving the PID absent.
  No live `settlement.lock` was manually deleted.

The root-sentinel checks were run for main,
source and disposable target worktrees:
`find` found no root `HEAD`,
`config`,
`hooks`,
`objects` or `refs` artifacts;
`git check-ignore --verbose` confirmed their ignore rules;
`git clean --dry-run -d -X` named no deletion.
The baked-in allowlist in `allowed-worktree-dirs.ts` was inspected rather than assumed absent.
No unrelated worktree changes were restored or discarded.

A separate logger diagnostic,
`sink verification failed for entry 3`,
appeared during inspection.
It is not attributed to the copy or treated as proof of the settlement cause.

### Verified creation path without ignored-state copying

The disposable target was clean,
so it was removed without force and recreated from the main worktree with an explicit start commit:

```sh
# Run in the actual main worktree; use a newly owned disposable destination.
git worktree add --detach /var/home/user/temp/agent/translation-repair-guard-20260911 9f4034d15
```

The invocation completed,
emitted no ignored-copy summary,
and the target had neither copied `node_modules` nor the owned staging tree.
The main wrapper bundle used for this invocation had SHA-256
`b7020d8041970b77394bf2712cc3c7f1381f73a9c4ac0eac3977fef4f7495b4e`.
These were different wrapper artifacts;
the observations are not a matched timing comparison.
The source applicability gate and the main-worktree creation itself establish the supported path.

Tradeoff:
verification dependencies must be supplied deliberately;
creating the fixture does not copy caches or secret-bearing ignored files.
Use an applicable linked source only when that ignored-state copy is actually wanted.
The reproduction output and cleanup ownership record are retained under `~/temp/agent/`.

Proposed `AGENTS.md` clarification to `WCD`,
not applied:

```text
WCD: Pin commands with -C/--cwd/cd. Verify pwd + Git root before alternate-worktree writes.
Create disposable worktrees from main-worktree context at an explicit commit unless ignored copying is wanted.
```

The expected operation was performed:
the disposable target was recreated through the main-worktree path,
not by disabling copy guards or deleting a live lock.

## Upstream filing artifact

### Upstream filing decision

No external upstream filing applies.
Cli-git and the fixed source are owned by this repository.
`.out-of-scope/` contains no matching exemption,
and searches across open and closed issues and pull requests in
`Aquaticat/Monochromatic` found no duplicate for the timeout or worktree-copy settlement terms.

The six constraints resolve as follows:

1. **Is it really upstream's fault?
   **
   No external upstream exists.
   The defect was this repository's applicability boundary.
2. **Can upstream fix it?
   **
   The owned implementation was fixed by comparing canonical invocation-specific Git and common directories.
3. **Are they supporting this use case?
   **
   Yes.
   Cli-git already classifies main and linked worktrees for other safeguards.
4. **Would the repo welcome our contribution?
   **
   This is the owning repository,
   with no external contribution or AI-assistance policy boundary involved.
5. **Will they likely fix it?
   **
   The fix and regression test are committed locally.
6. **Have we prototyped a minimal compatible fix?
   **
   Yes.
   The built artifact fails before the gate,
   passes after it,
   and retains linked-source and bare-source coverage.

Nothing should be posted to an external upstream.
The repository commits and this troubleshooting record are the filing artifact.
