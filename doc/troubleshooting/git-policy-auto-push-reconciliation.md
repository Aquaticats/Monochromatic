# Divergent auto-push with concurrent worktree edits

## Observed failure

On 2026-10-18,
a scoped auto-mode commit lands locally through the native Git wrapper,
then its automatic push fails with Git's `main -> main (non-fast-forward)` rejection.
The wrapper prints:

```text
cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.
```

An ordinary merge then refuses to overwrite concurrent uncommitted edits in
`doc/handover/slint-ide-0x.md` and `package/ide/app/README.md`.
Those edits are unrelated to the auto-mode commit and remain owned by concurrent work.
The rejected merge does not establish a conflicting committed history.

## Wrapper ownership

`package/git-policy/cli/src/native/auto_push.rs` defines the existing-upstream command as
`UPSTREAM_PUSH_ARGUMENTS = &["push"]`
and the fallback message as `PUSH_FAILED_NOTE`.
Repeating that command cannot itself integrate divergent histories.
The wrapper's successful commit exit and retained failure output do not establish successful publication.

[Issue 1769](https://github.com/Aquaticats/Monochromatic/issues/1769)
records the requested isolated reconciliation,
preservation of concurrent state,
upstream reachability check and bounded handling of further remote advancement.
The user explicitly requested the issue.
Searches across open and closed issues for divergent auto-push,
`non-fast-forward` and dirty auto-push,
plus the auto-push PR search,
return no matching report at filing time.
This is separate from [issue 1768](https://github.com/Aquaticats/Monochromatic/issues/1768),
which concerns launcher protection and executable resolution.
No wrapper implementation fix is claimed at filing time; the fix is described in the Resolution section.

## Recovery evidence

The supervisor creates a detached sparse worktree under private agent scratch,
using the qualified wrapper for every Git mutation.
The original worktree's dirty paths are not stashed,
restored,
staged or overwritten by this recovery.
The first sparse setup still has an uninitialized checkout and its merge refuses missing paths.
Initializing only that disposable worktree at its captured commit produces clean tracked state.

At recovery's next fetch,
upstream `0def389909beb770a1054001c352bd3cdb2b9fb2` already contains the captured local
`ebefb4de83ca67ef8fbc11f189358338b8c2af92`.
The isolated worktree fast-forwards and its explicit non-forced push is up to date.
Ancestry checks confirm both captured histories reach upstream.
SHA-256 checks confirm both originally blocking dirty files remain unchanged.
The original branch is untouched.
Publication at that checkpoint is established;
the supervisor does not claim to have authored the already published reconciliation.

The retained receipt is
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/divergent-auto-push-recovery.json`.
The same private directory contains the exact issue body and wrapper output.
Later commits require their own upstream reachability check.
An isolated publication worktree avoids disturbing concurrent edits,
but does not itself advance the original dirty checkout.

## Resolution

Issue 1769 was fixed in three releases of `monochromatic-git-policy-cli`.

### 0.2.0

A push that the remote refuses as stale (`[rejected]`) is fetched into the private ref
`refs/cli-git/auto-push/fetch-<pid>`,
merged with `git merge-tree --write-tree` and `git commit-tree`,
and published with a plain push of the merge.
Nothing touches the local branch, the index or the worktree while merging,
so the concurrent edits that blocked an ordinary merge cannot block it or be changed by it.
A conflict publishes nothing and names the conflicting paths.

### 0.2.1

Two defects of 0.2.0 were found and fixed.
`git commit-tree` ignores `commit.gpgsign`, so the merge was the one unsigned commit of a signed branch,
which a remote that requires verified signatures rejects.
The merge is now made with `-S` when `commit.gpgsign` is true, and a signer that fails publishes nothing.
The second defect: the local branch stayed behind the remote after a reconcile,
and a failed attempt wrote a record without the last published tip,
so the next commit looked like an amended branch and kept its rejection until someone merged by hand.
The reproducer is a remote `pre-receive` hook that refuses one push between two reconciles.
The real signer was confirmed on 2026-10-19 on merge `08b8bd1547`, which shows `%G?` of `G`.

### 0.2.2

The user's requirement of 2026-10-19 is that auto-push never fails when there is no conflict,
with offline and authentication failures exempt.
The release:

- fast-forwards the branch and its worktree to the published merge when `HEAD` is that branch at the pushed tip
  and no tracked file differs, and otherwise reports why it did not;
  the first production use left the status clean (merge `9a48cb3d13`, signed);
- retries up to ten times instead of three while the remote keeps moving;
- merges an amended or rebased branch like any other, so the replaced commit stays in the remote history;
- derives the push destination from `branch.<name>.pushRemote`, `remote.pushDefault` and `push.default`,
  because `git rev-parse @{push}` refuses to resolve a push remote that differs from the tracked one even though the
  push itself works;
- merges unrelated histories with `--allow-unrelated-histories`.

Still reported and not reconciled: a conflict, a `remote.<name>.push` refspec, `push.default` of `matching` or
`nothing`, the remote `.`, a remote hook refusal and a signer that fails.
The specification is the "Auto-push" section of `package/git-policy/cli/SPEC.md`.
