# cli-git concurrent commits

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Grilling session started 2026-09-25.
Status:
 design in progress;
 nothing implemented.
Note 2026-11-09T01:14:06-05:00:
 superseded on 2026-09-25,
 when the owner accepted the design
 (`doc/decision/cli-git-concurrent-commits.md`,
"Status"),
 and implemented in `cefcee148e`,
 `a50d674d7d` and `f7e501a685` (2026-09-25 to 26):
 preparation in a shadow repository,
 serial landing,
 and replay of a commit that lost a landing race.
 The replay modules sit under
 `package/git-policy/cli/src/native/shadow/`,
 and the landing ones under
 `package/git-policy/cli/src/native/landing/`.

## Goal

Several `git commit` invocations against the same worktree and branch run at the same time,
and all of them land as separate sequential commits.
Native Git rejects the later ones with `index.lock` `EEXIST`;
issue #560 is a real collision.

## Evidence

- `package/git-policy/cli/src/policy-engine/commit-transaction-workspace.ts` opens `<index>.lock` with `wx`,
  so a second transaction fails immediately instead of waiting.
  That file was removed on 2026-10-13 in `642b6d72cb` with the TypeScript wrapper.
  The native wrapper takes no `index.lock` of its own:
  a command that cannot take it ends as native Git ends it,
  and the commit service's landing retries it a bounded number of times
  (`package/git-policy/cli/src/native/lock/index_lock_wait.rs`, module header).
- The lock is held across every policy pass,
  the real `git commit` including its hooks,
  and post-commit index installation (`package/git-policy/cli/SPEC.md`,
   "Transaction protocol").
- Explicit-path commits already build the commit tree in a private index from `HEAD` plus selected paths.
- Policies may read `trackedFiles` for any pathspec and `headOid`,
  and candidate `change`/`revision` are relative to `HEAD`
  (`SPEC.md`,
   "Public authoring declarations"),
  so a verdict can depend on `HEAD` beyond the candidate bytes.
- Local real Git 2.55.0 has `git hook run` and `git merge-tree --write-tree --merge-base`.
- `git help git`,
   `GIT_OPTIONAL_LOCKS`:
   `git status` takes `index.lock` to refresh the index,
  so non-cli-git lock holders are routine.
- Wrapper-added commit latency in the fixture benchmark is roughly 0.3 to 0.45 seconds
  (`package/git-policy/cli/perf/lifecycle-latency-2026-07-16.json`);
  real-repo lock hold time with this repo's plugins is unmeasured.

## Private HEAD prototype, 2026-09-25

Disposable fixture,
real Git 2.55.0 at `/usr/bin/git`,
global and system config disabled,
SSH commit signing configured.

- Native `git commit` with `GIT_INDEX_FILE` private and `--git-dir` pointing at a private per-worktree admin dir
  (registered `git worktree add --detach --no-checkout`,
  or hand-built `HEAD` plus `commondir` file or `GIT_COMMON_DIR`)
  and `--work-tree` at the real worktree
  ran pre-commit,
  prepare-commit-msg,
  commit-msg,
  post-commit,
  the editor,
  template and cleanup,
  and signing.
- Hooks ran with the real worktree top level as cwd,
  saw the private index and private `HEAD`,
  and read real worktree files.
- Shared `HEAD`,
  `refs/heads/main`,
  their reflogs,
  and real `.git/index` bytes stayed unchanged.
- Replay with `git merge-tree --write-tree --merge-base`,
  `git commit-tree`,
  and compare-and-swap `git update-ref <ref> <new> <old>` produced the right tree,
  parent,
  author,
  and committer.
  Non-overlapping hunks in one file merged;
  overlapping hunks exited 1 and still printed a tree ID,
  so callers must check the exit status.

Measured caveats:

- The real index is stale after the ref moves;
  an ordinary commit afterwards committed a revert.
  The landing-time real index update is mandatory.
- Hooks inherit `GIT_DIR` and relative `GIT_WORK_TREE=.`;
  a hook that changes into a subdirectory and runs Git sees the wrong top level.
  Ordinary commits export neither.
- `git commit-tree` ignores `commit.gpgsign`;
  replay must pass `-S` when signing is configured.
- `git update-ref` without `-m` writes an empty reflog message.
- A registered no-checkout worktree protects its private commit from `gc`,
  appears in `git worktree list`,
  and needs `git worktree remove --force`.
  An unregistered admin dir is invisible,
  but `git gc --prune=now` deleted its commits.

Untested:
concurrency,
the wrapper on `PATH`,
hook frameworks,
GPG signing,
user config,
`--amend` and `--verbose` with a private `HEAD`,
sparse checkout,
submodules,
`core.hooksPath`,
`extensions.worktreeConfig`.

## Hook environment probe, 2026-09-25

Disposable fixture,
real Git 2.55.0.
Driver scripts and raw reports lived in the session scratchpad;
the durable write-ups are
[`git-hook-disable-switches.md`](../troubleshooting/git-hook-disable-switches.md)
and
[`git-private-admin-dir-hook-environment.md`](../troubleshooting/git-private-admin-dir-hook-environment.md).
Handover:
 [`doc/handover/cli-git-concurrent-commits.md`](../handover/cli-git-concurrent-commits.md).

- No plain invocation gives hooks an absolute `GIT_WORK_TREE`:
  Git rewrites `--work-tree` and `GIT_WORK_TREE` to `.` for hooks.
- `core.worktree` from `-c`,
  `GIT_CONFIG_COUNT`,
  or the admin dir's `config` is ignored,
  and `-c extensions.worktreeConfig=true` is not honoured.
  Only `extensions.worktreeConfig` in the real common config plus `<admin>/config.worktree` works,
  which is a persistent repository config write.
- `hook.<event>.enabled=false` suppresses config-based hooks but not the hookdir hook,
  contradicting `git help config`.
- `-c core.hooksPath=<dir>` does not disable config-based hooks.
- A dispatcher shim works for hookdir and config-based hooks:
  `-c core.hooksPath=<shim>` plus `-c hook.<event>.enabled=false` for each event,
  where each shim event exports `GIT_WORK_TREE="$(pwd)"` and runs
  `git -c hook.<event>.enabled=true -c core.hooksPath=<original> hook run --ignore-missing <event> -- "$@"`.
  Hooks saw an absolute work tree and correct subdirectory views,
  failures propagated,
  and `post-commit` did not run.
  Every `-c` leaks to hook children through `GIT_CONFIG_PARAMETERS`.
- After landing,
  `git hook run post-commit` in the real worktree runs hookdir and config hooks with no private variables,
  but without the native `GIT_INDEX_FILE`,
  `GIT_AUTHOR_*`,
  and `GIT_EDITOR=:`.
- A `git stash push --keep-index` and `git stash pop` inside a private pre-commit hook
  left real index bytes unchanged
  but transiently reverted the shared worktree's unstaged edit
  and pushed a transient entry onto the shared `refs/stash`.

## Foreign `index.lock` research, 2026-09-25

Git source at commit `0f8e75abebff` plus experiments with real Git 2.55.0.

- Git's only index lock is the `O_EXCL` file;
  there is no advisory lock,
  and the fd is `O_CLOEXEC`.
- Native `git commit -a`,
  `-i`,
  `-o`,
  interactive,
  and partial commits close the lock fd but keep `index.lock` on disk through
  pre-commit,
  prepare-commit-msg,
  the editor,
  and commit-msg
  (`builtin/commit.c`,
   lines 402 to 550 and 1957).
  Measured:
  a 3 second hook meant 3 seconds of lock with no holder.
  So no open holder does not mean abandoned.
- Plain `git commit` of staged changes releases the lock before hooks run.
- `git status` held the lock for a median of about 3.2 ms in a 10,000-file fixture.
- `core.lockfilePid=true`
  (default false,
  present in 2.55.0)
  writes `index~pid.lock` with the owner PID;
  Git's own `EEXIST` message uses `kill(pid, 0)` to call a lock stale,
  and Git never deletes it.
- Git has no index lock timeout;
  its other lock timeouts
  (`core.packedRefsTimeout`,
  `core.configLockTimeout`,
  1000 ms defaults)
  retry with quadratic backoff and jitter.
- After `kill -9`,
  the lock and PID file remained,
  and an orphaned hook still pointed `GIT_INDEX_FILE` at the lock.
- Holder scans must compare device and inode,
  not paths;
  `/proc` visibility stops at other users,
  `hidepid`,
  PID namespaces,
  and network filesystems.
  macOS denial is silent.
  On Windows,
  a `DELETE` probe could delay Git's own rename.
- `git help stash`:
  `pop` without an argument takes `stash@{0}`,
  the latest entry of shared `refs/stash`,
  so hook A's pop after hook B's push restores B's stash.

## Decisions

- Meaning:
   concurrent separate invocations,
  not one invocation producing several commits.
- Scope:
   default on in every repository the wrapper runs in.
- Mechanism:
   parallel preparation,
  serial landing.
  The owner chose this over a lock queue because the engineering capacity exists now.
- Replay:
   three-way content merge from the preparation base onto the new `HEAD`.
- Replay conflict:
   fail,
  nothing lands,
  and the diagnostic names conflicting paths and the winning commit;
  never re-prepare from current worktree bytes.
- Policy re-run after replay:
   record each policy's lazy reads
  (candidate bytes,
  `trackedFiles` pathspecs,
  `headOid`)
  and re-run only policies whose read set changed.
  Corrected default:
  an undeclared policy is assumed to read anything and always re-runs.
  A policy may declare that it reads only through the context,
  or list its external inputs
  (worktree files,
  executables,
  revisions passed to tools),
  which cli-git fingerprints.
  Evidence for the correction:
  every shipped plugin reads outside the API
  (`forbidden-strings` and `markdown-lint` spawn tools with repository cwd;
  `repository-policy` calls `readFile`
  and spawns `git ls-files` and `git cat-file`).
- Hook parallelism declaration:
   `hooks: { concurrentCommits: true }` in `cli-git.config`,
  default false.
- Policy input declaration:
  `inputs?: 'unrestricted' | { readonly external: readonly PolicyInput[] }`,
  default `'unrestricted'`;
  `{ external: [] }` means context-only reads.
  `PolicyInput` kinds:
   `worktree` (pathspecs),
  `executable` (path),
  `revision` (rev),
  `env` (name).
  Precedent:
   Nx task `inputs`,
  broad when absent.
- No opt-out:
  no config key and no environment variable disables concurrent commits.
  Owner rationale:
  a misbehaving wrapper is a bug to fix,
  not a reason to disable parts of it.
- Config keys:
   `indexLock: { unprovenOwnerTimeoutMs: 1000 }`
  and `landing: { reserveAfterLostRaces: 2 }`.
- Forwarded index writers
  (`add`,
   `rm`,
   `mv`,
   `restore --staged`,
   `reset`,
   and similar)
  wait on `index.lock` under the foreign-owner rules before forwarding,
  and re-forward after a lock `EEXIST` only when a disposable fixture proves that command fails before side effects.
  `git status` needs nothing because its optional lock is skipped silently.
  Rationale:
  concurrent agents already collide on `git add` today.
  The earlier claim that concurrent commits raise neighbor collision rates was retracted:
  landing holds `index.lock` far shorter than today's whole-transaction hold,
  and the net effect is unmeasured.
- Hooks and editor:
   run during preparation.
  The editor,
  `prepare-commit-msg`,
  and `commit-msg` run once;
  `pre-commit` re-runs outside the landing lock against the replayed index whenever the replayed tree differs,
  then landing retries.
- Landing order:
   preparation completion order.
- Starvation guard:
   after K lost landing races a commit reserves the next landing slot;
  others keep preparing and revalidating but cannot land until it lands or fails.
  K defaults to 2,
  is tunable,
  and the concurrent-commit benchmark confirms the default.
  Note 2026-11-09T01:14:06-05:00:
  the accepted default is 1,
  changed 2026-09-26 (veto open)
  (`doc/decision/cli-git-concurrent-commits.md`,
   "`landing.reserveAfterLostRaces` defaults to 1"),
  and `SPEC.md` states `1` for the key;
  the native constant agrees
  (`DEFAULT_RESERVE_AFTER_LOST_RACES` in
   `package/git-policy/cli/src/native/config/config_schema.rs`).
  No benchmark recorded a confirmation:
  `SPEC.md` still lists the sweep of `landing.reserveAfterLostRaces`
  among the scenarios to measure.
  The key no longer reserves a slot either;
  the service lands in arrival order,
  and `SPEC.md` says the key sets nothing.
- Waiting:
   unbounded while the cli-git landing owner is alive;
  bounded for an `index.lock` without a known owner.
- `--amend` and merge,
  cherry-pick,
  revert conclusions fail when `HEAD` moved.
- Hook concurrency:
   a per-repository hook lock serializes preparation hooks and post-landing `post-commit` by default;
  `cli-git.config` can declare the repository's hooks safe for parallel preparation.
  Both are retired:
  the wrapper runs no external hook, so no hook lock exists,
  and
  `hooks.concurrentCommits` is only accepted and ignored
  (`package/git-policy/cli/SPEC.md`, section "Retired keys";
  `git monochromatic-git-policy-cli doctor` says so).
- Foreign `index.lock`:
  cli-git injects `core.lockfilePid=true` into every forwarded and spawned Git.
  Proven-live owners
  (open holder by device and inode,
  or a PID file naming a live process started no later than the lock's ctime)
  get an unbounded wait with one line naming the holder.
  Dead or evidence-free owners get Git-style quadratic backoff up to 1000 ms,
  configurable in `cli-git.config`,
  then a diagnostic listing the evidence.
  cli-git never deletes a lock.
- Index commits participate with the index captured at invocation.
- Auto-push:
   single-flight per branch.
  A landed commit joins an in-flight push covering its OID or pushes the branch tip itself,
  and exits only once the remote contains its OID.

## Recommendation calibration

Owner,
 2026-09-25:
engineering budget is not a constraint;
recommend the most correct and performant option,
never the cheapest to build.
This applies to this design session only;
the owner declined an `AGENTS.md` rule.

## Settled by the decisions, veto open

- Same worktree and branch only;
  Git refuses one branch checked out in two worktrees.
- Each commit captures selected bytes at invocation,
  inherent in parallel preparation.
- The post-commit real index is computed at landing against the then-current real index,
  never against the invocation-time copy,
  or a landing would erase another commit's staging.
- Transaction journals become per transaction,
  so several prepared transactions coexist;
  recovery handles each.
- Replay onto a moved `HEAD` uses `git merge-tree --write-tree --merge-base`;
  cli-git declares a minimum Git version covering the plumbing it uses.
- Preparation runs native `git commit` against a private index and a private admin dir `HEAD`,
  per the prototype.
- When `HEAD` did not move,
  landing advances the ref to the prepared commit itself,
  keeping its signature.
- Replay signs with `git commit-tree -S` whenever the prepared commit was signed.
- The landing critical section holds the landing lock and real `index.lock`,
  advances the ref by compare-and-swap with a `-m` reflog message carrying the recovery nonce,
  installs the real index,
  then releases,
  so no ordinary commit can observe a moved ref with a stale index.
- Pending prepared commits are protected from `gc` by a private ref under `refs/cli-git/`,
  deleted at landing,
  abort,
  or recovery;
  the private admin dir stays unregistered,
  so `git worktree list` never shows it.
  The ref is briefly visible to `git for-each-ref` and `git log --all`.
  The design session chose that ref;
  the implementation replaced it with the per-transaction shadow object store,
  so no `refs/cli-git/` ref exists
  (`doc/decision/cli-git-concurrent-commits.md`, lines 43 to 46 and 487 to 488;
  `package/git-policy/cli/SPEC.md`, section "Transaction protocol").
- Hooks during preparation run through the dispatcher shim.
  It resolves the repository's own `core.hooksPath`,
  respects user-disabled events,
  omits `post-commit`,
  exports an absolute `GIT_WORK_TREE`,
  and restores the caller's original `GIT_CONFIG_PARAMETERS` before dispatch.
  Shim executables follow the repository's no-shell-script rule.
  `extensions.worktreeConfig` is rejected:
  it mutates every repository's config and still needs the shim for `post-commit`.
  The shim is gone:
  no tracked path matches `*hook_dispatch*`,
  the tracked
  `package/git-policy/cli/src/native/health/external_hooks.rs` is the module that inspects a
  configured `core.hooksPath` today,
  and `package/git-policy/cli/SPEC.md` says the wrapper runs no external hook.
- `post-commit` runs once after landing through `git hook run post-commit` in the real worktree,
  with native-equivalent `GIT_INDEX_FILE`,
  `GIT_AUTHOR_*`,
  and `GIT_EDITOR=:`.
- A replay conflict is a `core-finding` JSONL event with exit `1`,
  matching other expected commit rejections.
- New JSONL events report replays and lost landing races.
- Each commit records the symbolic `HEAD` target at invocation;
  if it changed before landing (branch switch),
  the commit fails like a moved amend base.
  Detached `HEAD` lands by compare-and-swap on `HEAD` itself.
- A crashed reservation holder releases its slot through the same owner-liveness check.
- `git cli-git fix` holds the landing lock across its worktree installation and real-index verification,
  so a concurrent landing cannot falsify its byte-identical real index check.
- Windows holder detection skips the `DELETE`-access probe,
  which can delay Git's own rename,
  and relies on the PID file and Restart Manager.
- Missing optional Git features degrade per feature
  (no `core.lockfilePid` means no PID evidence);
  missing replay plumbing falls back to today's fail-fast behavior.
- Required disposable fixtures and lifecycle benchmarks gain concurrent-commit scenarios.

## Rejected

- Lock queue with the unchanged transaction:
  owner prefers parallel preparation now.
- Batch syntax producing several commits from one invocation.

## Open questions

None.
Awaiting owner confirmation of shared understanding.

## Next action

Owner confirms shared understanding.
Then promote this plan to `doc/decision/`,
update `package/git-policy/cli/SPEC.md`,
and open the implementation issue.
No implementation before confirmation.
Note 2026-11-09T01:14:06-05:00:
 the plan became `doc/decision/cli-git-concurrent-commits.md` on 2026-09-25,
 `package/git-policy/cli/SPEC.md` states the landing order and the replay refusal,
 and the implementation landed in `cefcee148e`,
 `a50d674d7d` and `f7e501a685`.
