# cli-git 0.1.6: a stale auto-push is refused, and the merge that would reconcile it refuses dirty files

Note 2026-11-09T16:50:29-05:00:
`package/git-policy/cli/src/native/auto_push.rs` is now `package/git-policy/cli/src/native/push/auto_push.rs`,
moved 2026-11-03 in `d88fd7ab2a`.

Note 2026-11-10T14:52:03-05:00:
the behaviour this record diagnoses was replaced in release 0.2.0,
and the constant it cited is gone.
`UPSTREAM_PUSH_ARGUMENTS = &["push"]` appears in no tracked file now;
`package/git-policy/cli/src/native/push/auto_push.rs` runs the push through `run_push`
(`package/git-policy/cli/src/native/lfs_bounded_push`) and hands a rejection to the reconcile.
The release-by-release narrative this record used to carry lives in the specification now:
the reconcile,
its bounds and the notice it prints are specified in "Reconcile and bounds" of
`package/git-policy/cli/SPEC.md` (line 6122),
and the algorithm itself is in `package/git-policy/cli/src/native/push/auto_push_reconcile.rs`
and the modules beside it.
This record holds one incident and its owned boundary.

## Symptom

On 2026-10-18,
a scoped auto-mode commit lands locally through the native Git wrapper,
then its automatic push fails with Git's `main -> main (non-fast-forward)` rejection.
The wrapper prints:

```text
cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.
```

The wrapper's message prefix read `cli-git:` at the incident and reads `monochromatic-git-policy-cli:` now
(`PUSH_FAILED_NOTE` in `package/git-policy/cli/src/native/push/auto_push.rs:54`),
which is the program rename of 2026-11-04 recorded at the top of this file.

An ordinary merge then refuses to overwrite concurrent uncommitted edits in
`doc/handover/slint-ide-0x.md` and `package/ide/app/README.md`.
Those edits are unrelated to the auto-mode commit and remain owned by concurrent work.
The rejected merge does not establish a conflicting committed history.

## Root cause

The auto-push ran one plain `git push` of the branch and reported whatever Git answered.
That is not enough to publish a branch whose remote has moved,
because a plain push of a branch the remote does not contain is refused as a non-fast-forward,
and nothing in the wrapper merged the two histories:

```rust
// package/git-policy/cli/src/native/push/auto_push.rs:54 (the notice of that path)
pub const PUSH_FAILED_NOTE: &str =
    "monochromatic-git-policy-cli: commit saved locally, but auto-push to origin failed; run `git push` when ready.";
```

The notice sent the person to `git push`,
which is the same command that had just been refused,
so the only remaining remedy was to merge the remote branch into the worktree by hand.
Git refuses a merge that would write a file the worktree has uncommitted changes in,
and it named those two paths,
so the incoming history changed them:
that is why two files the auto-mode commit had not touched blocked the merge.

The wrapper's exit 0 for the commit and its retained push output establish that the commit was saved,
and nothing about publication:
the commit stayed local,
and the original branch was still behind the remote when the recovery ran.

The mechanism that replaced it is specified in
`package/git-policy/cli/SPEC.md` (line 6122,
"Reconcile and bounds"):
a push whose remote refuses it as stale is reconciled by the same push,
by fetching the remote branch into the private ref `refs/cli-git/auto-push/fetch-<pid>`,
computing the merge with `git merge-tree --write-tree`,
recording it with `git commit-tree`,
and pushing the merge with a plain push that is never forced.
That work is done from objects alone,
so it reads neither the index nor the worktree,
which is exactly the step the concurrent edits could block and the old path lacked.

## Verification

The recovery is the controlled log of the environment,
run on 2026-10-18 against the repository whose push had been refused.

- The supervisor creates a detached sparse worktree under private agent scratch and uses the qualified wrapper
  for every Git mutation.
  The original worktree's dirty paths are not stashed,
  restored,
  staged or overwritten by it.
- Upstream `0def389909beb770a1054001c352bd3cdb2b9fb2` already contains the captured local
  `ebefb4de83ca67ef8fbc11f189358338b8c2af92`.
  The isolated worktree fast-forwards and its explicit non-forced push is up to date.
- Ancestry checks confirm both captured histories reach upstream;
  SHA-256 checks confirm both originally blocking dirty files are unchanged byte for byte.
- The original branch is untouched,
  and publication at that checkpoint is established without the supervisor claiming authorship of an already
  published reconciliation.

The retained receipt is
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/divergent-auto-push-recovery.json`,
beside the exact issue body and wrapper output.

The runnable harness for the current behaviour is the crate's reconcile suite,
`package/git-policy/cli/src/native/push/auto_push_reconcile_tests.rs`,
which drives a real repository through the wrapper's own push:

- `a_divergent_branch_is_published_as_a_merge_and_the_worktree_is_not_touched` (that file,
  line 256)
  covers the working pattern:
  the path the remote changed is dirty in the checkout and an untracked file sits beside it,
  the push still publishes the merge,
  and `worktree_snapshot` shows the worktree unchanged.
- `a_conflicting_merge_is_kept_unpublished_with_the_conflicting_paths` (line 345) covers the failing pattern:
  a real overlap publishes nothing and names the paths.
- `a_remote_that_never_settles_is_reported_after_the_bound` (line 475) covers the bound,
  so a remote that keeps moving ends in a report rather than a wait.
- `a_remote_that_moves_again_is_merged_again` (line 427) and `a_busy_remote_is_merged_into_until_it_settles`
  (line 455) use a remote `pre-receive` hook that refuses one push between two reconciles,
  which is the reproducer of the 0.2.1 retry defect.

The release boundary is verified by tag:
`monochromatic-git-policy-cli-v0.1.6` was committed 2026-10-18T06:28:24-04:00,
`monochromatic-git-policy-cli-v0.2.0` at 2026-10-18T21:38:39-04:00 with the subject
"release 0.2.0,
auto-push reconciles a moved remote branch".
The installed wrapper link identified release 0.1.6 in the same investigation
(`git-policy-wrapper-launcher-path.md`,
"Controlled verification").

## Verified workarounds

Publish from an isolated worktree,
then let the original checkout catch up on its own terms.
This is the procedure the recovery ran:

- add a detached worktree at the captured commit under private scratch
  (`git worktree add --detach <path> <commit>`),
  and initialize it before merging in it,
  because a worktree whose checkout is not initialized refuses the missing paths;
- fetch,
  then fast-forward that worktree and push with an explicit non-forced push;
- check ancestry and the blocking files' hashes before and after,
  and leave the original worktree's dirty paths alone;
- repeat the upstream reachability check before every later commit,
  because each one needs its own.

Tradeoffs:
the isolated worktree publishes the captured commit and cannot advance the original dirty checkout,
so the person still merges or rebases that checkout before working there;
and the workaround carries no reconciliation of its own,
so a remote that moves again between the fetch and the push needs the whole sequence again.

Since 0.2.0 none of this is needed for a stale push:
the wrapper reconciles the branch itself (see Verification).

## What does not work

### Repeating `git push`

Rejected because the same command meets the same non-fast-forward refusal;
a push of the existing upstream publishes a branch and never merges one.

### Merging upstream into the dirty worktree

Rejected because Git refuses a merge that would overwrite uncommitted changes,
and the two files the incoming history changed belonged to concurrent work,
not to the commit being published.
Clearing the way by stashing or restoring those edits would discard work the wrapper never owned,
and the recovery keeps them byte for byte instead.

### Merging inside a sparse worktree whose checkout is uninitialized

Rejected because the first sparse setup still refuses missing paths at its merge.
Initializing only that disposable worktree at its captured commit is what made it work.

## Upstream filing artifact

Nothing to add,
and nothing to file.
Both Git behaviours involved are Git's documented ones:
a non-fast-forward push is refused,
and a merge that would overwrite uncommitted changes is refused.
The defect and its fix are ours,
and the fix shipped in releases 0.2.0 to 0.2.2,
now specified in `package/git-policy/cli/SPEC.md`,
"Reconcile and bounds".

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

### Upstream filing decision

1. **Is it really upstream's fault?**
   No.
   The refusal of a non-fast-forward push and the refusal of a merge over uncommitted changes are documented
   Git behavior,
   not wording and not an architectural restriction.
   The gap is that the wrapper published nothing and pointed at the command that had just failed.
2. **Can upstream fix it?**
   Not applicable.
   There is no upstream defect,
   and the reconciliation Git already supports through `fetch`,
   `merge-tree` and `commit-tree` is what our fix
   now uses.
3. **Are they supporting this use case?**
   Yes,
   and the use case is ordinary:
   merging a moved upstream branch is what `git fetch` and `git merge` exist for,
   and computing a merge from objects without a worktree is what `git merge-tree --write-tree` is for.
4. **Would the repo welcome our contribution?**
   Not applicable.
   No filing is made and no contribution is offered.
5. **Will they likely fix it?**
   Not applicable.
   Nothing is broken upstream to fix,
   and this record asserts no leaning either way.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Not applicable upstream,
   and the equivalent step was done in our own code:
   the reconcile described in the Root cause section is the minimal fix that publishes a stale branch without
   touching the worktree,
   and the crate's reconcile suite named in the Verification section is the harness that drives it.

The default policy is not to file,
and this record files nothing.
