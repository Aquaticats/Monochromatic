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

- `7915f885a3` merge(main):
  `0.6.1-alpha.13` into the branch,
  no conflict.
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

## Verification after the merge into `0.6.1-alpha.13`

`main` moved to release `0.6.1-alpha.13` (merge `157d07cd3b`,
twenty-one finished branches),
and the branch was merged into it at `7915f885a3` with no conflict:
Git merged every file on its own,
`src/native/lfs/mod.rs` keeping both the new module and `main`'s
`lfs_command_line_configuration`.

`main` holds neither fix:
`reads_as_true` and `tracking_trusted` are still in its `lfs_upload_plan.rs`
(at its own lines 404 and 457),
so nothing was fixed twice.

Against the build of the merge,
with the disposable fixtures and local bare remotes the audit used:

- both audit reproductions exit `1` when pointed at `wrapper-before`
  (`objectPresentAtChangedRemote: false`,
  `sideObjectPresent: false`)
  and exit `0` with the object present when pointed at the merge's build
  (`1/1` and `2/2` objects uploaded).
- the family probe prints the same table as `family-final.txt`.
- `-c remote.origin.mirror=2` exits `1` before and `0` after.
- `remote.pushDefault` and a second scenario of two URLs
  (the fetch saw only the first) show the same split as the table:
  before,
  a push to two URLs put the object on one of them only
  (`nextObjectAtFirst: false`);
  after,
  both hold it.
- `probe-shallow.sh`:
  before,
  the push that follows a moved URL exits `0`
  and publishes `main` on the second server with the object absent;
  after,
  `git-lfs` stops at `(missing) big.bin`,
  the push exits `2`,
  and no ref is published.
- `probe-residual-cases.ts` still shows the open case below.
- `mise run //package/git-policy/cli:native:clippy`:
  no warning.
- `mise run //package/git-policy/cli:native:test:host -- lfs`:
  `109 tests run: 109 passed, 7329 skipped`,
  naming `lfs_destination_confirms_tracking_refs::tests::*`
  and the three plan controls.
- `mise run //package/git-policy/cli:native:test:binary:host -- lfs`:
  `54 tests run: 54 passed, 1030 skipped` on a second run.
  The first run failed
  `lfs_hang::two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other`
  on `first waited for the other` after a 10 s wait on `git update-index`;
  it passed alone and in that second run,
  and
  `doc/handover/cli-git-rust-implementation.md` already records that control
  failing under contention.
- the package linter is red at the merge for 46 errors in 30 files,
  none on a line this branch wrote:
  the two `SPEC.md` findings sit on text `main` carries unchanged
  (its lines 16620 and 16710),
  and the `lfs_command_line_configuration.rs` finding is `main`'s file.
- what a push asks,
  traced with `strace -f -e trace=execve`
  (`runs/trace-*/trace.log`):
  `for-each-ref --format=%(objectname)%09%(symref)%09%(refname) refs/remotes/origin`
  and `ls-remote --heads origin` are the two the fix adds,
  beside the `remote get-url --push --all`,
  `push --dry-run --porcelain`,
  `rev-list` and `git-lfs push` the wrapper already ran.
- cost on a fixture whose remote already holds 300 LFS objects,
  12 alternating rounds of a push that adds one object:
  before `min 22 / median 91 / max 193 ms`,
  after `min 26 / median 93 / max 100 ms`;
  a push that adds no object `47/48/49` against `53/54/54 ms`;
  the plan that sends the whole history `124/117` medians.
  The two questions alone:
  `for-each-ref` `2/3/4 ms`,
  `ls-remote --heads` `4/5/6 ms` against a local bare remote
  (one round trip against a network one).

The runs behind these are under
`/var/home/user/temp/agent/lfs-plan-uploads-merge`:
`changed-*`,
`mirror-*`,
`family-merged.txt`,
`shallow-*`,
`residual-after.log`,
`cmdline-mirror-*`,
`pushdefault-*`,
`trace-after.log`,
`cost-merged.log`,
`unit-merged.log`,
`binary-merged-2.log`,
`clippy-merged.log`,
`lint-merged.log`,
and `unit-before-mine.log` (the five controls failing against
`02a9416b39`'s three plan files).

## The push the commit service runs finds its programs where the shell does

### The defect

A note of 2026-11-12T09:06-05:00 added a third finding.
After the desktop session was lost on 2026-11-12 and a new one began,
two worktrees' commits landed and their pushes were refused because the commit service could not find `git-lfs`:
the service runs under the user manager and takes the manager's environment,
and that `PATH` (`/usr/local/sbin:/usr/local/bin:/usr/bin`)
held no folder `git-lfs` is installed in,
while the shells that committed had one.
The person committing could neither see that list nor change it from their shell.

Where each program of a push comes from,
read from the code:

- real Git:
  the absolute path the unit passes (`--git`),
  never a search.
- `git-lfs`:
  `PATH` of the environment the push is handed (`lfs_pre_push::find_program`,
  `program_on_path`).
- a credential helper:
  `PATH` of the same environment plus the repository's configuration.
- `ssh` and a proxy command:
  `PATH` and `GIT_SSH`,
  `GIT_SSH_COMMAND`,
  `GIT_PROXY_COMMAND`.
- an askpass program:
  never located,
  because every service push carries the never-prompt pair by the owner's decision
  (`doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`).

That environment was `std::env::vars_os()` of the service:
the manager's.

### The fix

The five values that decide where a push finds its programs,
and how it reaches the remote,
travel from the command that asks for the push to the commit service in its request or wake,
and live only in that service process's memory
(`service_push_programs_held_in_memory`):
`PATH`,
`GIT_EXEC_PATH`,
`GIT_SSH`,
`GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND`.
A commit carries them in its `CommitRequest`;
a merge,
cherry-pick,
revert,
pull's merge,
an applied patch or a pull carries them in the `Work` message that wakes the service for the push record it just wrote.
They are keyed by the push record and never by the worktree or the branch,
so each push runs with the values of the command that asked for it,
and are dropped when that record can no longer run.

Nothing of them is written.
The owner's decision of 2026-11-12T11:58:45-05:00 is "Memory only",
after a review showed that the first version of this fix wrote both command lines into
`<git-dir>/monochromatic-git-policy-cli/push-programs.json`,
that the record was never removed,
that a typed push never refreshed it,
and that a killed client left a staging file holding the whole record.
A first command that meets a record or staging file of that earlier shape removes it
(`earlier_push_programs_record`);
the branch that wrote them was never released.

The one thing a record keeps is a flag:
the push record says that the command which asked for it chose a transport,
and never which.
A service that restarted holds no values,
so a push whose flag is set stops before it runs with the words that name the repository,
the branch and the one command that pushes,
and say that the transport settings of the command that asked are gone with the restart
(`push_without_its_transport`).
A push whose flag is unset searches the service's own `PATH` and its refusal says so
(`lfs_pre_push::ProgramSearch`),
and that refusal's remedy no longer promises that a typed push records a list for the service.

### Checks after this change

- `mise run //package/git-policy/cli:native:clippy` at the final commit (`558eefd875`):
  `Finished ... in 4.29s`, no warning, with `-D warnings`.
- `native:test:host -- --no-fail-fast push_without_its_transport service_push_programs_held_in_memory
  earlier_push_programs_record lfs_pre_push service_work`:
  `148 tests run: 148 passed, 7449 skipped`.
  A wider run of the same modules with `service_protocol`, `service_journal` and `service_handoff`
  (`226 tests run: 224 passed, 2 failed`) leaves two failures in
  `service_journal_restored_at_start`,
  which are `main`'s and not this change's:
  both expect a read to stall on a pipe standing where a plan file would be,
  and `de74ced87d` (in `main`) makes that read skip a name that is not a regular file,
  so the read answers instead of stalling
  (`a_journal_that_gives_no_answer_is_left_behind_at_its_ceiling`,
  `a_plan_a_service_could_not_read_at_its_start_is_read_again_and_tried`).
- `native:test:binary:host -- --no-fail-fast push_programs_held_in_memory`:
  `5 tests run: 5 passed`, each against real Git,
  a real bare remote and the built wrapper:
  neither command line reaches any file under the fixture's Git directory,
  runtime directory,
  state directory or home (with a planted value as the positive control),
  two commits each push with their own command's settings,
  a push a restarted service has to try again stops with the words that name the command and that command sends the
  commit,
  the record of the earlier shape is removed by the first command,
  and the remedy an LFS refusal prints pushes from the shell.
  With three breaks in place at once (the transport stop removed,
  the removal of the earlier record removed,
  and the request's values ignored) all five controls fail,
  and the log is `keep/binary-controls-with-the-three-breaks-failing.log` in the run's scratch folder.

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
- The package linter is red at the branch's start for 25 findings
  and at the merge into `0.6.1-alpha.13` for 46 errors in 30 files,
  in files or on text this branch does not touch
  (`worktree_copying`,
  `merge`,
  `landing`,
  `binary_merge_*`,
  and `main`'s own lines of `SPEC.md`);
  the branch adds none.
- `lfs_hang::two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other`
  failed once of two full binary LFS runs after the merge
  (`first waited for the other` after a 10 s wait on `git update-index`)
  and passed alone and in the second run;
  it is the contention-sensitive control
  `doc/handover/cli-git-rust-implementation.md` already tracks.
- Nothing under `doc/decision/` was read or written as a decision;
  this branch
  makes no owner decision.
