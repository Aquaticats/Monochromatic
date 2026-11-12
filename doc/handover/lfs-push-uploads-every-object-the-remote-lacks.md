# The LFS upload plan asks the destination, not this clone's memory

## Purpose

This hands over the branch
`fix/git-policy-cli-lfs-push-uploads-every-object-the-remote-lacks`,
which fixes two ways a push published Git LFS pointer files whose objects never
reached the server the push went to.
It is complete and verified;
what remains open is at "Open work".

Everything the branch changed is in `package/git-policy/cli`:

- `src/native/lfs/lfs_destination_confirms_tracking_refs.rs` and its `_tests.rs`
  (new):
  whether this clone's tracking refs are evidence about the destination,
  asked of the destination with `git ls-remote --heads <remote>`.
- `src/native/lfs/lfs_upload_plan.rs` and `lfs_upload_plan_awaiting.rs`:
  the trust condition,
  the destination list for a remote Git writes to more than
  once,
  and `remote.<name>.mirror` read through Git.
- `src/native/lfs/lfs_upload_settings.rs`:
  `git_says_true` and
  `git_says_true_awaiting` replace the list of boolean spellings.
- `src/native/lfs/lfs_upload_plan_tests.rs`,
  `src/native/binary/binary_lfs_server_tests.rs`:
  the controls.
- `SPEC.md`,
  `README.md`:
  what the plan trusts,
  and the price of a push.
- `src/native/lfs/mod.rs`,
  `lfs_object_plan.rs`:
  module entry and the plan's
  contract text.

## The two findings and what fixed them

### A changed remote URL published pointers without their objects

`git remote set-url` leaves `refs/remotes/<remote>/*` describing the server the
last fetch saw,
and the plan read them as a statement about the new one.
The plan now asks the destination for its branch heads
(`git ls-remote --heads <remote>`,
through the same remote name and `insteadOf` rewrites the push resolves)
and keeps a tracking ref as evidence only where the destination reports that
branch at the same commit.
Anything else,
a moved branch,
a branch the destination no longer has,
a
destination that cannot be asked,
a local listing this cannot read,
sends
everything reachable from the pushed refs (`--all`).

### A numeric true `remote.<name>.mirror` skipped the other branches' objects

`reads_as_true` knew `true`,
`yes`,
`on`,
`1` and the empty value.
Git reads `2`,
`-1` and `0x10` as true and the empty value as false,
so a mirror
it performed was planned as an ordinary branch push.
`git_says_true` asks Git (`git config --type=bool --get remote.<name>.mirror`),
only when the settings hold a value for the key,
and a value Git cannot read
refuses the push Git would refuse.

### A third case the audit did not have

A remote with several `remote.<name>.url` values is pushed to at every one of
them by Git,
and the plan uploaded to one destination only.
The destination list is now every push URL,
or every URL when the remote names
no push URL,
with `--all` for each.

## Commits

- `afb0e3e198` fix(git-policy-cli):
  the upload plan asks the destination and
  Git,
  not this clone's memory.
- `5ad03704ee` fix(git-policy-cli):
  read the destination's answer in the shape a
  trimmed query leaves (every query trims its answer's ends,
  which ate the empty
  `%(symref)` field;
  the format now puts that field between two filled ones).
- `6177af8a3b` docs(git-policy-cli):
  the plan's new prose breaks where the
  package linter breaks prose.

## Verification at `6177af8a3b`

- `mise run //package/git-policy/cli:native:clippy`:
  finished,
  no warning.
- `mise run //package/git-policy/cli:native:test:host -- lfs`:
  `104 tests run: 104 passed, 7235 skipped`.
- `mise run //package/git-policy/cli:native:test:binary:host -- lfs`:
  `52 tests run: 52 passed, 1021 skipped`.
- `mise run //package/git-policy/cli:native:test:binary:host -- --no-fail-fast push`:
  `120 tests run: 120 passed, 953 skipped`.
- `mise run //package/git-policy/cli:native:test:host -- --no-fail-fast landing`:
  `1037 tests run: 1037 passed, 6302 skipped`.
- `mise run //package/git-policy/cli:native:test:host -- --no-fail-fast push`:
  `669 tests run: 668 passed, 1 failed`.
  The failure is
  `worktree::worktree_landed_removal::tests::a_push_failure_recorded_after_the_removal_snapshot_reaches_the_person`,
  which `doc/handover/cli-git-rust-implementation.md` ("The gate on
  `d90730337c`") already lists as failing at alpha.12 and assigns to the round
  259 agent;
  every file it compiles is identical to `02a9416b39`.

The evidence a later run needs is under
`/home/user/temp/agent/lfs-plan-uploads`:

- `reproductions/changed-remote-url-skips-lfs-upload.ts` and
  `reproductions/numeric-mirror-value-skips-side-branch-upload.ts`:
  the audit's
  two scripts,
  pointed at this worktree's build;
  both exit `0` after the fix and
  printed `1/1` and `2/2` objects uploaded.
- `probe-family.sh`,
  `family-before.txt`,
  `family-after.txt`,
  `family-final.txt`:
  the family of "the plan believes the remote already has this" cases,
  before and
  after.
- `probe-shallow.sh`:
  a shallow clone whose remote URL moved.
- `probe-residual-cases.ts`:
  the two cases left,
  see "Open work".
- `probe-command-line-mirror.ts`:
  `-c remote.origin.mirror=2`.
- `measure-push-cost-4.ts`,
  `attribute-push-cost.ts`:
  the cost of a push.
- `unit-before.log`,
  `binary-before.log`,
  `unit-6177.log`,
  `binary-6177.log`:
  the failing-first and final test output.
- `lint-package-2.jsonl`:
  the package linter's findings,
  none in a file this
  branch touched.

Builds used:
`wrapper-before` (the branch's start,
`02a9416b39`) and
`wrapper-final` (`6177af8a3b`),
both `mise run //package/git-policy/cli:native:build`.

## Open work

- The plan's evidence is about the destination's git commits,
  not its LFS
  store.
  If the destination's store loses an object while its refs stay,
  a push
  that reuses those commits uploads nothing and publishes the ref:
  measured with
  `probe-residual-cases.ts` (deleting one object from the origin's
  `lfs/objects`,
  then pushing the same commit under a second branch name:
  status
  `0`,
  no `git-lfs` run,
  the object still absent).
  Closing that needs a question
  to the LFS server (a batch request for the objects the walk would skip),
  which
  is a new plan shape,
  not a condition here.
- An unconfirmed destination now sends everything,
  so a clone whose `.git/lfs`
  lacks an old object refuses the push with `git-lfs`'s own words instead of
  publishing pointers (`probe-shallow.sh`).
  `lfs.allowincompletepush` remains the
  documented way through.
- The package linter (`native` tree scope) is red at `02a9416b39` for 25 findings
  in files this branch does not touch (`worktree_copying`,
  `merge`,
  `landing`,
  `binary_merge_*`);
  the branch adds none.
- Nothing under `doc/decision/` was read or written as a decision;
  this branch
  makes no owner decision.
