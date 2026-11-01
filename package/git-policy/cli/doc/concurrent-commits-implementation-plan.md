# Concurrent commits implementation plan

## Status and sources

Code map for issue #571,
written 2026-09-25 against branch `feat/cli-git-concurrent-commits` at `d322d083e`
before any of it was implemented.
Every slice is now implemented on that branch.
This document stays as the record of where each change was planned to go;
`SPEC.md` describes the implemented behavior,
and "Final behavior versus this plan" lists every place where the implementation departed from the plan.

- Accepted design:
  [`doc/decision/cli-git-concurrent-commits.md`](../../../../doc/decision/cli-git-concurrent-commits.md).
- Evidence and rejected options:
  [`doc/planning/cli-git-concurrent-commits.md`](../../../../doc/planning/cli-git-concurrent-commits.md).
- Canonical interface:
  [`SPEC.md`](../SPEC.md),
  which now specifies the per-transaction journals,
  private preparation,
  serial landing,
  and every lock this plan introduces.

Line references use `path:line` relative to `package/git-policy/cli/`
and point at the first line of the named declaration at `d322d083e`;
the "Owners today" sections describe that revision,
 not the implemented code.

## Final behavior versus this plan

Each item names the plan's assumption and the implemented behavior;
the `SPEC.md` heading holds the rules.

- Private `HEAD`:
  the plan's options A and B both used a pending ref under `refs/cli-git/pending/`.
  The implementation uses neither:
  each transaction prepares in a shadow repository at `<git-common-dir>/cli-git/shadow/<transaction-id>`
  whose `HEAD` names a private copy of the target branch
  and whose `objects/info/alternates` names the real object store,
  so hooks see the real branch name and upstream.
  New objects stay in the shadow store until landing migrates them into the real store
as a pack kept with a `.keep` file,
  and no `refs/cli-git/` ref is ever created.
  `SPEC.md` "Private preparation" and "Object migration";
  the decision's "Private `HEAD` shape:
   shadow repository with alternates" records the prototype.
- Recovery of a dead owner without a landing record removes its `.keep` files,
  its reservation,
  the shadow repository,
  and the transaction directory,
  instead of deleting a pending ref (`SPEC.md` "Recovery").
- Replay is not a pure three-way merge.
  For a path that this commit and a commit landed since its base both captured from the same worktree,
  the later capture's bytes land ("Capture order").
  Every other shared path first checks subsumption,
  keeping the prepared bytes when they already contain the landed change ("Subsumption"),
  and only then merges three-way from the preparation base with `git merge-tree --write-tree --merge-base`.
  An unsigned replayed commit is rebuilt by rewriting the raw commit object with `git hash-object -t commit -w`,
  because `git commit-tree` transcoded non-UTF-8 messages and dropped headers;
  only a signed commit is rebuilt with `git commit-tree -S` ("Replay").
- Owner locks wait without bound while their owner lives,
  including the worktree-copy settlement lock;
  only an owner record without evidence gets a bounded wait ("Locks").
  The settlement lock is held only by forwarded commands that create or move worktrees
  ("Linked-worktree ignored-state synchronization").
- The hook lock is taken by the dispatcher shim around each hook event,
  not around the whole native preparation,
  so an open message editor never holds it ("Hook lock").
- `inputs` may be static or a function of the validated policy options.
- Reservations are granted oldest invocation first ("Starvation reservation").
- Forwarded index writers wait for the landing lock and pre-wait for foreign `index.lock` holders;
  cli-git never captures Git's stderr to re-forward after `EEXIST` ("Index-writer coordination").
- Missing Git features are detected by exercising each feature,
  not by version:
  replay plumbing is probed after a commit's first lost race,
  and without it the commit fails with `concurrent-commit/head-moved` ("Compatibility and degradation").
- Post-landing runs Git's automatic maintenance before `post-commit`,
  as native `git commit` does,
  because every landing adds one pack to the real store ("Post-landing").
- JSONL additions stay under `schemaVersion: 1`;
  consumers ignore unknown event types.
- `landing.reserveAfterLostRaces` defaults to `1`,
  not the planned `2`;
  `SPEC.md` "Benchmark method" holds the sweeps that set it.

## Findings that shape every slice

### Clean commits never reach the private transaction

`runCommitTransaction` returns without committing when no policy patch changed the candidate
(`src/policy-engine/commit-transaction.ts:356`).
`runCliGit` then forwards the transformed native `git commit`
through `runGitWithWorktreeCopy` (`src/bin.ts:340`).
Native explicit-path commits keep `index.lock` on disk through every commit hook and the editor,
so the common clean commit is the collision path from issue #560.
Parallel preparation therefore has to cover every non-dry-run commit,
not only the autofix path.
That adds private-index work to every commit,
so the lifecycle latency baseline has to be re-measured
(`perf/lifecycle-latency-2026-07-16.json`,
 "Performance gates" in `README.md`).

### Startup recovery blocks every invocation while an owner lives

`recoverCommitTransaction` throws `CommitTransactionRecoveryError`
when the journal owner is alive (`src/policy-engine/commit-transaction-recovery.ts:212`),
and `runCliGit` calls it before every non-short-circuit invocation (`src/bin.ts:193`).
The packed fixture `src/trust/fixture/built-autofix-concurrency-consumer.ts` asserts that `git status` exits `2`
with "is still active" while a commit runs.
Per-transaction recovery must skip live owners instead.

### Linked worktrees already serialize every forwarded Git

For linked worktrees and bare repositories,
`runGitWithWorktreeCopy` holds a common-directory settlement lock across real Git and its hooks
(`src/worktree-copy/lifecycle.ts:181`),
acquired with a bounded wait of 100 attempts 10 ms apart
(`src/worktree-copy/journal-lock.ts:25`,
 `src/worktree-copy/journal-lock.ts:467`).
Nested wrapper calls from hooks pass through only with the inherited `CLI_GIT_WORKTREE_COPY_LEASE`.
Concurrent commits in linked worktrees therefore collide on this lock today,
and hook Git calls made during private preparation carry no lease.
Private preparation must not run inside that settlement lock;
commits cannot register worktrees except through hooks,
which the settlement lock was built to observe.
This interaction needed an explicit decision in slice 2;
the decision's "Implementation-time decisions" settled it:
only forwarded commands that create or move worktrees take the settlement lock.

### Shared `HEAD` is re-read throughout preparation

Each of these reads the live `HEAD` and must read the recorded preparation base instead:

- `initializeCommitIndex` (`src/policy-engine/commit-transaction-index.ts:74`);
- `createPrivateIndexFacts.headOid` (`src/policy-engine/commit-transaction-candidates.ts:302`);
- `listChangedIndexPaths` (`src/policy-engine/commit-transaction-candidates.ts:466`);
- the candidate batch diff (`src/policy-engine/commit-transaction-candidate-batch.ts:245`);
- `prepareTransactionJournal` (`src/policy-engine/commit-transaction-journal.ts:284`);
- `resolveLandedCommitOid` (`src/policy-engine/post-commit-facts.ts:214`),
  which after concurrent landings can name another invocation's commit.

### Recovery trusts only the latest reflog entry

`assertTransactionReflog` requires the nonce in the newest `HEAD` reflog entry
(`src/policy-engine/commit-transaction-recovery-validation.ts:368`).
Once several transactions land,
a crashed landing can be followed by other landings,
so recovery must search the target ref's reflog for the nonce.

### Recovery removes only locks it can prove it owns

`assertOwnedLock` (`src/policy-engine/commit-transaction-recovery-validation.ts:323`) plus `rm(lockPath)`
(`src/policy-engine/commit-transaction-recovery.ts:364`)
delete a lock whose device and inode the journal recorded.
The decision's "cli-git never deletes a lock" covers foreign locks;
the plan keeps owned-lock cleanup and states the distinction in `SPEC.md`.

### Shipped repository policies read only through the context

The planning evidence says `repository-policy` calls `readFile` and spawns Git.
Measured with `rg` for `node:`,
 `nano-spawn`,
 and `process.env` under `src/optional/`:
only `bump-dependents-worktree.ts` does,
and neither `forbiddenRootContext` (`src/optional/repository-policy/index.ts:114`)
nor `dependentVersionBump` (`src/optional/repository-policy/dependent-version-bump-policy.ts:247`) imports it;
it is an exported helper.
Both policies can declare `{ external: [] }`.

### Git spawn sites are scattered

Real Git starts from `runTransactionGit` (`src/policy-engine/commit-transaction-git.ts:92`),
`executeRealGit` (`src/worktree-copy/lifecycle.ts:66`),
`runMetadataGit` (`src/git-metadata.ts:52`),
`autoPush` (`src/auto-push.ts:235`),
`post-commit-facts.ts:76`,
`blob-batch.ts:223`,
the manual-push modules,
`rule/commit-index-check.ts:81`,
`rule/commit-sequencer-check.ts:127`,
and `branch-worktree-remote-guess.ts:57`.
Injecting `core.lockfilePid=true` needs one seam that all of them inherit (slice 6).

### Files already over the line budget

`eslint/max-lines` allows 300 lines after skipping blanks and comments
(`package/config/oxlint/src/rule/style.ts:47`).
These raw sizes need an oxlint measurement before any addition,
and every new behavior goes into sibling modules per MXL:
`src/bin.ts` 444,
`src/policy-engine/events.ts` 579,
`src/trust/config-validation.ts` 512,
`src/policy-engine/commit-transaction-candidates.ts` 514,
`src/policy-engine/commit-transaction-journal.ts` 508,
`src/policy-engine/commit-transaction-recovery-validation.ts` 492,
`src/policy-engine/commit-transaction-workspace.ts` 456.

## Cross-cutting conventions for new code

- New modules keep the flat `src/policy-engine/commit-transaction-*.ts` prefix
  or a feature directory like `src/worktree-copy/`.
- Every module gets `@module` TSDoc,
  TSDoc with `@example` on every declaration,
  tagged loggers built with `tagged({ tag: fn.name, l })`,
  destructured object parameters,
  `using`/`await using` cleanup,
  and custom error classes.
- Git always runs with an argument array and no shell.
- Unit tests sit beside the module as `*.unit.test.ts` on `@monochromatic-dev/test-harness-core`,
  run through `mise run //package/git-policy/cli:test:unit`.
- Real-Git disposable fixtures follow `src/worktree-copy-fixture.unit.test.ts`:
  `/usr/bin/git`,
  `mkdtemp`,
  global and system config disabled.
- Packed shadow-bin fixtures live in `src/trust/fixture/built-*-consumer.ts`,
  are called from `src/trust/fixture/built-trust-consumer.ts`,
  and run through `mise run //package/git-policy/cli:test:built:trust`
  (Podman,
   1 GiB,
   1 CPU).
- Deterministic interleaving reuses the readiness-marker technique of
  `src/trust/fixture/built-autofix-concurrency-consumer.ts`:
  Node hook programs write a marker and wait for a release file.
  Shell hooks are not allowed (SCR).
- Every fixture asserts exact ref,
  reflog,
  real index,
  and worktree bytes before and after (`SPEC.md` "Required disposable fixtures").

## Slice 1: per-transaction journals and startup recovery

### Owners today

- `TRANSACTION_DIRECTORY_NAME = 'cli-git-transaction'`
  and `createCommitTransactionWorkspace` (`src/policy-engine/commit-transaction-workspace.ts:41`,
   `:191`):
  one directory per `--git-path`,
  created only after `open(lockPath, 'wx')` on the real index lock (`:260`).
- `PreparedTransactionJournal` schema version 1,
  `prepareTransactionJournal`,
  `recordRefUpdated`,
  `recordIndexInstalled`
  (`src/policy-engine/commit-transaction-journal.ts:57`,
   `:284`,
   `:462`,
   `:497`).
- `recoverCommitTransaction` (`src/policy-engine/commit-transaction-recovery.ts:112`),
  `resolveCommitTransactionDirectory` (`src/policy-engine/commit-transaction-recovery-target.ts:49`),
  `parsePreparedJournal`,
  `assertOwnedLock`,
  `assertTransactionReflog`,
  `assertLandedCommit`,
  `processIsAlive`
  (`src/policy-engine/commit-transaction-recovery-validation.ts:149`,
   `:323`,
   `:368`,
   `:437`,
   `:51`),
  `recoverNormalization` (`src/policy-engine/commit-transaction-normalization-recovery.ts:83`),
  and `installRecoveredIndex` / `removeRecoveryArtifacts`
  (`src/policy-engine/commit-transaction-recovery-files.ts:70`,
   `:148`).
- `resolveProcessBirthIdentity` (`src/policy-engine/commit-transaction-process-identity.ts:114`).
- Call sites:
  `src/bin.ts:164` (management) and `src/bin.ts:193` (wrapper).

### Seam

Keep `recoverCommitTransaction` as the startup entry and its call sites unchanged.
Inside it,
resolve a transaction root instead of one directory,
enumerate every transaction,
skip live owners,
and recover each dead one by its recorded state.
The legacy `cli-git-transaction` single-journal path stays as a read-only recovery branch
until no retained legacy directory can exist,
because users on the current build may hold one after a crash.

### New modules

- `src/policy-engine/commit-transaction-registry.ts`:
  resolves `<git-dir>/cli-git-transactions/`,
  creates `<root>/<transaction-id>/` (`randomUUID`),
  and lists entries with the no-follow checks the current directory uses.
- `src/policy-engine/commit-transaction-journal-states.ts`:
  schema-version-2 state records and their writers.
- `src/policy-engine/commit-transaction-journal-parse.ts`:
  version-2 parsing,
  split out of `commit-transaction-recovery-validation.ts` for the line budget.
- `src/policy-engine/commit-transaction-recovery-scan.ts`:
  enumerate,
  classify owner liveness,
  dispatch per state.
- `src/policy-engine/commit-transaction-recovery-landing.ts`:
  recovery of a crashed landing under the landing lock.
- `src/owner-lock/owner-lock.ts` and `src/owner-lock/owner-lock-wait.ts`:
  the rename-published directory lock with process-birth ownership,
  extracted from `attemptAcquire`,
  `retireStaleLock`,
  and `ownedLock` in `src/worktree-copy/journal-lock.ts`,
  parameterized by lock path,
  error factory,
  and wait policy.
  Implemented:
  every owner lock,
  including the worktree-copy settlement lock,
  waits without bound while its owner lives,
  and only an owner record without evidence gets a bounded wait
  (`SPEC.md` "Locks").

### Data structures

Directory layout per worktree Git directory:

```text
<git-dir>/cli-git-transactions/
  landing.lock/            owner-lock directory (slice 3)
  reservation.lock/        owner-lock directory (slice 5)
  <transaction-id>/
    owner.json             pid, birth identity, schemaVersion 2, created time
    preparing.json         invocation facts written before any Git mutation
    prepared.json          shadow repository path, prepared OID, signed flag, intended tree
    landing-<n>.json       one per landing attempt inside the critical section
    ref-updated.json       exact landed OID
    index-installed        empty completion marker
    commit.index, captured.index, post.index, candidate-*.state, patch-*.diff
```

The implemented layout,
with its pre-landing index snapshots and replay artifacts,
is in `SPEC.md` "Transaction directory and journal";
the private admin dir became the shadow repository under `<git-common-dir>/cli-git/shadow/`.

State files keep today's exclusive-create writes (`writePrivateFile`,
 `src/trust/registry-io.ts:374`)
rather than rewriting one journal,
so a crash leaves the newest complete state readable.

`preparing.json` records:
schema version,
transaction ID,
mode (`explicit-path` or `index`),
preparation base (`GitObjectId` or unborn),
symbolic `HEAD` target (ref name or detached),
target ref for compare-and-swap,
conclusion kind (none,
 amend,
 merge,
 cherry-pick,
 revert),
repository root,
real index path,
reflog nonce,
selected paths.

`landing-<n>.json` records:
expected old OID,
new OID (prepared or replayed),
exact pre-landing real index snapshot identity,
post-index artifact identity,
real `index.lock` device and inode,
added-path and selected-worktree records.

### Recovery rules

- Owner alive with matching birth identity:
  skip silently at debug log level.
  This replaces today's `is still active` failure.
- Dead owner without a landing record:
  remove its `.keep` files in the real `objects/pack`,
  release any reservation it owns,
  remove the shadow repository,
  then remove the transaction directory.
  The real index and real refs were never touched.
- Dead owner with a landing record:
  acquire the landing lock,
  then apply today's decision tree
  (not landed,
  landed without index,
  index installed)
  using the landing record's snapshots,
  a reflog search for the nonce on the target ref,
  and owned-lock cleanup.
- Empty or malformed transaction directories keep failing closed with the path named,
  as `README.md` "Commit autofix transaction" describes today.

### Tests and fixtures

- `commit-transaction-registry.unit.test.ts`:
  creation,
  enumeration,
  symlink and non-directory refusal.
- `commit-transaction-journal-parse.unit.test.ts`:
  every state record accepted,
  every missing or mistyped field rejected.
- `owner-lock.unit.test.ts`:
  acquire,
  live contention,
  stale PID reuse retirement,
  ownership change detected at release;
  `src/worktree-copy.unit.test.ts` stays green as the regression proof for the extraction.
- `commit-transaction-recovery-scan.unit.test.ts` on disposable repositories:
  two prepared and one landing transaction with dead owners recover in one startup;
  a live owner is skipped while a dead one beside it recovers;
  a crashed landing followed by another landing is recognized through reflog search;
  a legacy `cli-git-transaction` directory still recovers.
- Packed:
  rewrite `built-autofix-concurrency-consumer.ts` so `git status` succeeds while a commit hook runs.

### Risks and unknowns

- The dead-owner check still depends on `resolveProcessBirthIdentity`,
  which shells out to `ps` on macOS and PowerShell on Windows
  (`src/policy-engine/commit-transaction-process-identity.ts:114`).
  Enumerating many transactions multiplies that cost;
  measure before relying on it at every startup.
- Recovery of a landing record must never race a live lander,
  so it only runs while holding the landing lock.

## Slice 2: private `HEAD` preparation and the hook dispatcher shim

### Owners today

- Mode selection and applicability:
  `runCommitTransaction` (`src/policy-engine/commit-transaction.ts:65`),
  dry-run and `--all` bail-out at `:84`,
  explicit-path without pathspecs at `:109`,
  clean shortcut at `:356`.
- Private index setup:
  `initializeCommitIndex` (`src/policy-engine/commit-transaction-index.ts:74`),
  `materializePathspecFile`,
  `prepareInteractiveSelection`,
  `hasSequencerConclusion`,
  `resolvePrivateCommitArgs`
  (`src/policy-engine/commit-transaction-selection.ts:257`,
   `:195`,
   `:302`,
   `:149`).
- Real commit:
  `concludeCommitTransaction` (`src/policy-engine/commit-transaction-conclusion.ts:71`)
  and `executePreparedCommit` (`src/policy-engine/commit-transaction-finalize.ts:60`),
  which runs Git with `GIT_INDEX_FILE` and `GIT_REFLOG_ACTION` (`:84`)
  and verifies the landed tree (`:122`).
- Commit argv parsing:
  `parseCommitRegion` and `CommitRegion` (`src/parser/commit.ts:212`,
   `:117`);
  global `--git-dir` and `--work-tree` are recognized by `src/parse-global-options.ts:15`.
- Forwarding:
  `src/bin.ts:241` through `:348`.

### Seam

`executePreparedCommit` stops advancing the shared ref.
It runs native `git commit` as shown here;
the implementation points `--git-dir` at the transaction's shadow repository instead of `<tx>/admin`
(see "Private `HEAD` shape"):

```text
git --git-dir=<tx>/admin --work-tree=<worktree root>
    -c core.hooksPath=<tx>/hooks
    -c hook.pre-commit.enabled=false -c hook.prepare-commit-msg.enabled=false
    -c hook.commit-msg.enabled=false -c hook.post-commit.enabled=false
    commit <private commit args>
```

with `GIT_INDEX_FILE=<tx>/commit.index`,
then verifies the prepared tree and records `prepared.json`.
The shortcut at `commit-transaction.ts:356` is removed,
so every commit except dry runs prepares privately,
and `bin.ts` forwards `commit` only for dry runs and short-circuit forms.
The `--all` bail-out at `:84` becomes index-mode preparation:
native `commit -a` updates only the private index copy.

### New modules

- `src/policy-engine/commit-preparation-base.ts`:
  resolves and records the preparation base,
  symbolic `HEAD` target,
  common dir,
  real index path
  (honoring a caller-set `GIT_INDEX_FILE`,
   `GIT_DIR`,
   `GIT_WORK_TREE`,
   and global `--git-dir`/`--work-tree`),
  and replaces every live-`HEAD` read in "Shared `HEAD` is re-read throughout preparation".
- `src/policy-engine/commit-preparation-admin-dir.ts`:
  builds `<tx>/admin` with `HEAD`,
  `commondir` (absolute common dir),
  and copies of per-worktree state native `commit` reads:
  `MERGE_HEAD`,
  `MERGE_MSG`,
  `MERGE_MODE`,
  `SQUASH_MSG`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `sequencer/`,
  `config.worktree` when `extensions.worktreeConfig` is set,
  and `info/sparse-checkout`.
- `src/policy-engine/commit-preparation-native.ts`:
  assembles the native argv (drops user `--git-dir`/`--work-tree`,
   pathspecs,
   and internal `--only`),
  runs it with inherited stdio,
  classifies failure before or after the prepared commit exists.
- `src/hook-dispatch/hook-shim-writer.ts`:
  writes `<tx>/hooks/` per preparation.
- `src/hook-dispatch/hook-dispatch-program.ts`:
  the dispatcher logic as a plain-JavaScript program text,
  bundled into the single artifact and written to disk at runtime.
- `src/hook-dispatch/hook-dispatch-plan.ts`:
  computes the plan the shim reads:
  repository `core.hooksPath` (absent means `<common>/hooks`),
  events the user disabled through `hook.<event>.enabled`,
  caller `GIT_CONFIG_PARAMETERS` (or absence),
  real Git path,
  absolute worktree root.

### Private `HEAD` shape

The plan offered two shapes,
both built on a pending ref under `refs/cli-git/pending/`.
A prototype rejected both:
hooks saw no real branch name and no upstream,
so a "reject commits to main" hook was bypassed.
The implementation prepares in a shadow repository per transaction
at `<git-common-dir>/cli-git/shadow/<transaction-id>`,
whose `HEAD` names a private copy of the target branch
and whose `objects/info/alternates` names the real object store;
no `refs/cli-git/` ref exists
(`SPEC.md` "Private preparation";
decision "Private `HEAD` shape:
 shadow repository with alternates").
The rejected options follow as planned.

#### Option B: symbolic `HEAD` to the pending ref

`<tx>/admin/HEAD` holds `ref: refs/cli-git/pending/<id>`,
created at the base first (or left unborn).
Native commit advances the pending ref itself.

- Pros:
  the prepared commit is protected from `gc` the moment it exists;
  `--amend` works unchanged;
  hooks that call `git symbolic-ref HEAD` still succeed.
- Cons:
  hooks that read the branch name see `cli-git/pending/<id>`;
  `git branch --show-current` prints nothing.

#### Option A: detached `HEAD` at the base

`<tx>/admin/HEAD` holds the base OID;
cli-git writes the pending ref after commit.

- Pros:
  no private name appears in hook output.
- Cons:
  hooks that refuse a detached `HEAD` fail;
  a crash between commit and ref write leaves an unprotected object
  (still retained until `gc.pruneExpire`).

The plan ranked B over A,
because B closes the `gc` window atomically and keeps symbolic-ref hooks working,
while both shapes break branch-name-aware hooks equally;
the shadow repository keeps branch-name-aware hooks working as well.

### Hook dispatcher shim

The shim is a runtime-generated Node program,
not a shell script,
so the single-artifact rule (`SPEC.md` "Single artifact and shipped optional policies") still holds:
nothing new ships in the tarball.

- `<tx>/hooks/dispatch.mjs` holds the program text;
  `<tx>/hooks/plan.json` holds the dispatch plan,
  encoded with `JSON.stringify` at the final interpolation (SYB).
- `<tx>/hooks/pre-commit`,
  `prepare-commit-msg`,
  and `commit-msg` are executable files whose first line is `#!<process.execPath>`
  and whose body imports `dispatch.mjs` by absolute file URL with the event name.
  `post-commit` is intentionally absent.
- The program restores the caller's `GIT_CONFIG_PARAMETERS`,
  sets `GIT_WORK_TREE` to the absolute worktree root,
  skips user-disabled events,
  and runs
  `git -c hook.<event>.enabled=true -c core.hooksPath=<original> hook run --ignore-missing <event> -- <args>`,
  propagating the exit status.
- It also exports a preparation lease variable
  (named alongside `WORKTREE_COPY_LEASE_ENV`,
   `src/worktree-copy/journal-lock.ts:35`)
  so nested wrapper invocations from hooks skip startup recovery,
  hook-lock acquisition,
  and landing,
  and so their index-lock waits target the private index.

### Tests and fixtures

- `commit-preparation-admin-dir.unit.test.ts` on a disposable repository:
  exact `HEAD`,
  `commondir`,
  and copied state files;
  `git worktree list` output unchanged.
- `hook-dispatch-plan.unit.test.ts`:
  `core.hooksPath` absent,
  relative,
  absolute;
  disabled events;
  caller `GIT_CONFIG_PARAMETERS` present and absent;
  adversarial paths with quotes,
  newlines,
  and spaces (STB).
- `commit-preparation-native.unit.test.ts` on disposable repositories:
  hookdir and config-based hooks each run once;
  a hook that changes into a subdirectory sees the right top level;
  `post-commit` does not run during preparation;
  shared `HEAD`,
  branch,
  reflogs,
  and real index bytes stay unchanged;
  `gc --prune=now` keeps the prepared commit;
  SSH-signed preparation stays signed;
  `--amend`,
  `--allow-empty`,
  merge and cherry-pick conclusions produce native parents and messages.
- Packed:
  the full current fixture list in `SPEC.md` "Required disposable fixtures" re-run through private preparation,
  plus clean commits that previously forwarded.

### Risks and unknowns

- Per-worktree state the admin dir lacks:
  sequencer continuation after a private conclusion needs the native cleanup
  (removal of `CHERRY_PICK_HEAD`,
  advancing `sequencer/todo`)
  replayed into the real Git directory at landing.
  The prototype proved it:
  conclusions prepared in the shadow produced commits byte-identical to native,
  and landing reproduces native cleanup
  (`SPEC.md` "Sequencer conclusion state").
- Hooks that read the branch name see a private name under either option;
  the shadow repository avoids this.
- The shebang with an absolute `process.execPath` breaks when that path contains spaces
  and on Windows,
  where Git for Windows parses shebangs itself.
  Windows needs its own verified form.
- Whether the Node Rolldown flavor bundles a text import (ST6) is unverified;
  the client flavor documents it (`package/config/rolldown/src/index.client.ts:67`).
- The worktree-copy settlement lock must not wrap preparation;
  a hook that runs `git worktree add` would then skip ignored-state synchronization.
  Settled:
  only commands that create or move worktrees take the settlement lock,
  and a worktree created by Git invoked through an absolute path from a hook loses automatic ignored-state copying.
- A `pre-commit` hook that runs `git stash` touches the shared `refs/stash`
  (planning doc "Hook environment probe");
  the hook lock only serializes cli-git's own hooks.

## Slice 3: landing critical section

### Owners today

- Lock-held installation:
  `CommitTransactionWorkspace.installIndex` (`src/policy-engine/commit-transaction-workspace.ts:382`)
  writes through the lock descriptor,
  links `install.index`,
  renames over the real index,
  and applies index timestamps (`src/policy-engine/index-file-timestamps.ts:68`).
- Post-commit index:
  `preparePostIndex` (`src/policy-engine/commit-transaction-index.ts:215`),
  built from the invocation-time copy.
- Landed tree check and phase markers:
  `executePreparedCommit` (`src/policy-engine/commit-transaction-finalize.ts:60`).
- Added-path completion:
  `installAddedWorktreeFiles` (`src/policy-engine/commit-transaction-added-paths.ts`).
- Post-commit policies:
  `runPostCommitLifecycle` (`src/policy-engine/post-commit-lifecycle.ts:99`)
  with injectable `resolveLandedCommitOid`.
- Core findings:
  `createCoreFindingEvent` and `CoreFindingEvent` (`src/policy-engine/events.ts:455`,
   `:188`).

### Seam

`concludeCommitTransaction` calls a new `landPreparedCommit`
where it calls `executePreparedCommit` today.
The real `index.lock` moves from workspace creation (`commit-transaction-workspace.ts:260`)
into the landing critical section,
acquired through the slice 6 wait.
`CommitTransactionResult` gains the landed OID,
and `bin.ts` passes it to `runPostCommitLifecycle` and auto-push instead of re-reading `HEAD`.

### Landing loop

As planned;
the implementation (`SPEC.md` "Landing" and "Post-landing") adds two steps:
before step 5 it migrates the commit's shadow-only objects into the real store as a kept pack,
and after the locks are released it runs Git's automatic maintenance before `post-commit`.
With no pending ref,
step 7 removes the migrated pack's `.keep` instead.

1.  Wait until no live foreign reservation exists (slice 5).
2.  Acquire `landing.lock`,
    then the real `index.lock` (slice 6),
    and write cli-git's own owner PID file in Git's format.
3.  Fail with `branch-switched` when `git symbolic-ref -q HEAD` differs from the recorded target.
4.  Read the target ref.
    Equal to the base:
    the new OID is the prepared commit.
    Different with an amend or conclusion:
    fail with `head-moved`.
    Different otherwise:
    record a lost race,
    release both locks,
    replay outside the locks,
    revalidate,
    and restart at step 1.
5.  Copy the current real index,
    compute the post-index against it,
    and write `landing-<n>.json`.
6.  `git update-ref -m <message with nonce> <target> <new> <old>`.
    A compare-and-swap failure counts as a lost race.
7.  Write `ref-updated.json`,
    install the post-index through the held lock,
    write `index-installed`,
    remove copied conclusion state from the real Git directory,
    delete the pending ref,
    release both locks.
8.  Complete added-path worktree copies,
    run `post-commit` (under the hook lock),
    run cli-git post-commit policies with the landed OID,
    then auto-push (slice 7).

### Post-index computed at landing

- Explicit-path:
  current real index with the committed paths reset to the landed tree
  (today's `git reset --quiet <tree> -- <paths>` on a copy of the current index instead of the invocation copy).
- Index mode:
  for each path the commit changed relative to its base,
  take the landed entry only when the current real index entry still equals the captured one;
  otherwise keep the current entry,
  because someone restaged it after invocation.
- Every copy keeps source timestamps (`copyIndexFile`,
   `src/policy-engine/index-file-timestamps.ts:30`;
   #544).

### Replay

Implemented differently in three ways
(`SPEC.md` "Replay",
"Subsumption",
and "Capture order"):
a shared path captured from the same worktree takes the later capture's bytes,
every other shared path is checked for subsumption before the three-way merge,
and an unsigned replayed commit is rebuilt by rewriting its raw commit object with `git hash-object -t commit -w`,
because `git commit-tree` transcoded non-UTF-8 messages and dropped headers.
The plan as written:

- `git merge-tree --write-tree --name-only -z --merge-base=<base> <current> <prepared>`;
  exit `1` means conflict even though a tree ID prints,
  exit above `1` is an engine failure.
- `git commit-tree <tree> -p <current> -F <raw message file>` with
  `GIT_AUTHOR_NAME`,
  `GIT_AUTHOR_EMAIL`,
  `GIT_AUTHOR_DATE`,
  and the committer triple from the prepared commit,
  plus `-S` (with the invocation's key ID when one was given) when the prepared commit has a `gpgsig` header.
- Conflict:
  a `core-finding` with a new `coreId` (proposed `concurrent-commit`) and code `concurrent-commit/replay-conflict`,
  exit `1`,
  naming the conflicting paths,
  the first commit in `<base>..<current>` touching them,
  and the prepared commit OID so the user can cherry-pick it.
  The object survives in the shadow store until the transaction is removed.
- After a clean replay:
  re-run policies per slice 4
  (until slice 4 lands,
  treat every policy as unrestricted),
  re-run `pre-commit` through the shim against a private index of the replayed tree
  with the admin `HEAD` moved to `<current>`,
  and rebuild the commit with `commit-tree` when a patch or hook changed the tree.
- Missing `merge-tree --merge-base` or `hook run` support:
  fail fast with today's diagnostic shape (decision "Recovery and compatibility").

### New modules

- `src/policy-engine/commit-landing.ts` (loop),
  `commit-landing-cas.ts` (`update-ref` and reflog message),
  `commit-landing-index.ts` (post-index at landing),
  `commit-landing-replay.ts` (`merge-tree` and `commit-tree`),
  `commit-landing-commit-facts.ts` (author,
  committer,
  raw message,
  signature detection),
  `commit-landing-conclusion-state.ts` (real Git directory cleanup),
  `commit-landing-post-commit-hook.ts` (`git hook run post-commit`).
- `src/policy-engine/events-concurrency.ts`:
  replay and lost-race events plus the new core codes,
  re-exported through `events.ts`,
  which is already over budget.

### Data structures

- Reflog message:
  `commit (cli-git <nonce>): <subject>`,
  replacing today's `GIT_REFLOG_ACTION` form (`commit-transaction-workspace.ts:370`).
- New JSONL events (backward-compatible additions under `schemaVersion: 1`,
   as `SPEC.md` now states):
  `commit-replayed` (`fromBase`,
  `onto`,
  `oid`)
  and `landing-race-lost` (`attempt`,
  `winningOid`).
- New core codes:
  `concurrent-commit/replay-conflict`,
  `concurrent-commit/head-moved`,
  `concurrent-commit/branch-switched`.

### Tests and fixtures

Disposable real-Git unit tests driving two in-process preparations with controlled interleaving:

- disjoint explicit-path commits both land in completion order with native parents;
- non-overlapping hunks in one file merge;
- overlapping hunks fail with the conflict event and leave ref,
  index,
  and worktree exact;
- a signed commit landing without replay keeps its exact bytes;
  a replayed signed commit is re-signed (SSH key generated in the fixture);
- another invocation's staged path survives a landing (the stale-index revert caveat);
- amend,
  merge,
  cherry-pick,
  and revert with a moved `HEAD` fail;
- a branch switch between preparation and landing fails;
- detached `HEAD` lands by compare-and-swap on `HEAD`;
- two concurrent initial commits on an unborn branch;
- the target reflog and `HEAD` reflog each contain the nonce entry exactly once;
- `post-commit` runs once with `GIT_INDEX_FILE`,
  `GIT_AUTHOR_*`,
  and `GIT_EDITOR=:`;
- interruption at each marker recovers per slice 1.

### Risks and unknowns

- Whether `git update-ref <branch>` also writes the `HEAD` reflog like a native commit was unverified.
  Resolved:
  it does when run in the owning worktree's context with no `GIT_DIR` override (`SPEC.md` "Landing").
- `commit-tree` may drop an `encoding` header or other extra headers.
  Confirmed:
  it transcoded non-UTF-8 messages and dropped headers,
  so unsigned replays rewrite the raw commit object instead (`SPEC.md` "Replay").
- Replay plus revalidation is a second convergence loop;
  it must reuse `runCommitTransaction`'s pass-limit and cycle rules rather than duplicate them.
- GPG signing during replay can prompt through pinentry;
  replay runs outside both locks for that reason.

## Slice 4: policy read sets and the `inputs` declaration

### Owners today

- Public types:
  `PolicyDefinition`,
  `NamedPolicyDefinition`,
  `PolicyContext` (`src/api/policy-types.ts:275`,
   `:265`,
   `:154`);
  `LazyPolicyGitFacts` (`src/api/context-types.ts:50`);
  `definePolicy` (`src/api/authoring.ts:65`);
  exports in `src/api/index.ts`.
- Runtime shape:
  `RuntimePolicyDefinition` (`src/policy-engine/types.ts:20`).
- Validation:
  hand-written `validatePolicy` (`src/trust/config-validation.ts:221`);
  Valibot is used only for policy options.
- Execution:
  `createPolicyContext` and `runPolicyEngine` (`src/policy-engine/engine.ts:74`,
   `:153`);
  `checkPolicy` and `runPolicyStage` (`src/policy-engine/policy-stage.ts:108`,
   `:162`).
- Private facts:
  `createPrivateIndexFacts` (`src/policy-engine/commit-transaction-candidates.ts:302`),
  `loadTrackedFiles` (`src/policy-engine/commit-transaction-tracked-files.ts:162`).

### Seam

`checkPolicy` hands each policy its own `PolicyContext` whose `git` member is a recording wrapper
around the shared memoized facts,
so memoization (`SPEC.md` "Declaration invariants") is untouched.
`runPolicyStage` returns the per-policy read set with the findings.
The landing loop compares stored read sets and external fingerprints after a replay
and passes the set of policies to re-run.

### New modules and types

- `src/api/policy-input-types.ts`:

  ```ts
  // package/git-policy/cli/src/api/policy-input-types.ts
  export type PolicyInput =
    | Readonly<{ kind: 'worktree'; pathspecs: readonly string[]; }>
    | Readonly<{ kind: 'executable'; path: string; }>
    | Readonly<{ kind: 'revision'; rev: string; }>
    | Readonly<{ kind: 'env'; name: string; }>;

  export type PolicyInputs =
    | 'unrestricted'
    | Readonly<{ external: readonly PolicyInput[]; }>;
  ```

  `PolicyDefinition` gains `readonly inputs?: PolicyInputs`;
  `RuntimePolicyDefinition` carries the resolved value (default `'unrestricted'`).
- `src/trust/policy-inputs-schema.ts`:
  a Valibot schema for `inputs`,
  called from `validatePolicy`,
  with issues rendered like option failures (`SPEC.md` "Configuration validation").
- `src/policy-engine/policy-read-set.ts`:
  the recording wrapper and `PolicyReadSet`:
  candidate list (path,
  mode,
  change,
  blob OID),
  paths whose `bytes()` ran,
  each `trackedFiles` pathspec request with its result entries,
  and whether `headOid()` ran.
  Blob and commit OIDs are the fingerprints,
  so no extra hashing is needed.
- `src/policy-engine/policy-input-fingerprint.ts`:
  `worktree` via `git ls-files --cached --others --exclude-standard -z` plus `git hash-object --stdin-paths`;
  `executable` via `PATH` resolution plus no-follow stat identity (device,
  inode,
  size,
  mtime);
  `revision` via `git rev-parse --verify <rev>^{object}` against the new base;
  `env` via the value or absence.
- `src/policy-engine/policy-rerun-selection.ts`:
  pure decision:
  unrestricted always re-runs;
  declared policies re-run only when a recorded read or fingerprint changed.

### Declarations for shipped policies

- `repository/forbidden-root-context`:
  `{ external: [] }`.
- `repository/dependent-version-bump`:
  `{ external: [] }` (reads only `candidates` and `trackedFiles`).
- `forbidden-strings/forbidden-strings`:
  worktree `forbidden-strings.*.txt` at the repository root,
  env `FORBIDDEN_STRINGS_RULES`,
  executable `forbidden-strings`.
  The executable is a policy option (`src/optional/forbidden-strings/index.ts:39`),
  and an absolute rules path outside the repository has no `PolicyInput` kind.
- `markdown-lint/autofix`:
  stays `'unrestricted'`;
  its command is an option whose default runs a workspace source tree
  (`src/optional/markdown-lint/index.ts:46`).
- Built-ins:
  `final-newline` reads only candidate bytes and can declare `{ external: [] }`;
  the four command policies read Git state outside the context
  (for example `src/policy-engine/branch-worktree-remote-guess.ts:57`)
  and stay `'unrestricted'`.

### Tests and fixtures

- `policy-read-set.unit.test.ts`:
  each lazy method records exactly its read;
  memoized second calls still record per policy;
  a policy reading nothing records an empty set.
- `policy-input-fingerprint.unit.test.ts`:
  each kind changes when its input changes and stays equal otherwise,
  on disposable directories.
- `policy-rerun-selection.unit.test.ts`:
  unrestricted,
  context-only,
  and each external kind.
- `policy-inputs-schema.unit.test.ts`:
  valid shapes,
  unknown kind,
  empty pathspec list,
  wrong types.
- `src/api.unit.test.ts`:
  `inputs` type export and `definePolicy` identity.
- Packed:
  a counting plugin proves context-only policies skip after a disjoint replay,
  unrestricted ones re-run,
  and a declared worktree input changed by the winning commit forces a re-run.

### Risks and unknowns

- The static `inputs` type cannot express option-dependent inputs.
  Owner question at planning time:
  keep the static form and leave such policies unrestricted,
  or allow `inputs` to be computed from validated options.
  Settled:
  `inputs` may be either.
- `bytes()` on a tracked file can be read after the policy returns;
  recording must stop at policy completion (`SPEC.md` "Policy completion").

## Slice 5: starvation reservation, hook lock, and config keys

### Owners today

- Top-level keys and `ValidatedConfig`:
  `ValidatedConfig`,
  `CONFIG_KEYS`,
  and `validateConfig` (`src/trust/config-validation.ts:32`,
   `:72`,
   `:384`).
- Public config types:
  `CliGitConfig` and `CliGitConfigInput` (`src/api/config-types.ts:86`,
   `:114`).
- Runtime threading:
  `resolveRuntimeConfig` (`src/trust/runtime-config.ts`)
  and policy option assembly in `runCliGit` (`src/bin.ts:225`).

### Seam

`validateConfig` delegates the new keys to a sibling validator
and returns a `concurrency` member on `ValidatedConfig`.
`runCliGit` passes it into `runCommitTransactionBoundary` next to `policyOptions`.
Absent or untrusted config uses defaults;
startup recovery,
which runs before config,
always uses defaults.

### New modules

- `src/trust/config-validation-concurrency.ts`:
  `hooks.concurrentCommits` (boolean,
  default `false`),
  `landing.reserveAfterLostRaces` (positive safe integer,
  planned default `2`,
  shipped default `1`),
  `indexLock.unprovenOwnerTimeoutMs` (non-negative safe integer,
  default `1000`),
  unknown nested keys rejected with exit `2`.
- `src/policy-engine/commit-landing-reservation.ts`:
  owner-lock at `<git-dir>/cli-git-transactions/reservation.lock` holding the transaction ID.
  A transaction reaching the threshold waits for the reservation;
  others may prepare and revalidate but wait before landing while a live foreign reservation exists.
  A dead owner's reservation is retired by the owner-lock liveness check.
- `src/hook-dispatch/hook-lock.ts`:
  owner-lock at `<common-dir>/cli-git/hook.lock`,
  held around native preparation,
  around each `pre-commit` re-run,
  and around `post-commit`,
  skipped when `hooks.concurrentCommits` is `true`
  or when a valid preparation lease is inherited.

### Tests and fixtures

- `config-validation-concurrency.unit.test.ts`:
  defaults,
  each valid value,
  zero,
  negative,
  fractional,
  string,
  and unknown nested key.
- `commit-landing-reservation.unit.test.ts`:
  reservation after the configured lost races;
  others cannot land while it is held;
  a killed holder releases it.
- `hook-lock.unit.test.ts`:
  two preparations serialize their hooks by default and overlap with `concurrentCommits: true`
  (observed through hook marker timestamps).
- Packed:
  config type errors surface through `built-policy-config-consumer.ts`.

### Risks and unknowns

- The hook lock covers the whole native preparation commit,
  including the interactive editor between `prepare-commit-msg` and `commit-msg`,
  because Git runs them in one process.
  An open editor blocks other commits' hooks until it closes.
- A reservation queue with more than one waiter needs an order;
  the decision names one slot only.
  Proposed at planning time:
  first to reach the threshold wins.
  Implemented instead:
  oldest invocation first (`SPEC.md` "Starvation reservation").

## Slice 6: foreign `index.lock` classification and index-writer waits

### Owners today

- cli-git's own acquisition fails at once on `EEXIST` (`src/policy-engine/commit-transaction-workspace.ts:260`).
- Forwarding:
  `executeRealGit` (`src/worktree-copy/lifecycle.ts:66`) with inherited stdio.
- Liveness and birth identity:
  `processIsAlive` (`src/policy-engine/commit-transaction-recovery-validation.ts:51`),
  `resolveProcessBirthIdentity` (`src/policy-engine/commit-transaction-process-identity.ts:114`).
- Direct fix:
  `installDirectFix` snapshots the real index before and after
  (`src/policy-engine/direct-fix-install.ts:230`),
  called at `src/policy-engine/direct-fix.ts:176`.

### Seam

- A child-environment helper applied at every Git spawn site listed in "Git spawn sites are scattered",
  or once to `process.env` at the top of `runCliGit`.
  It appends `core.lockfilePid=true` through `GIT_CONFIG_COUNT`,
  `GIT_CONFIG_KEY_<n>`,
  and `GIT_CONFIG_VALUE_<n>`,
  preserving existing entries,
  so the shim's restoration of `GIT_CONFIG_PARAMETERS` does not remove it.
- `acquireRealIndexLock` replaces the `open(lockPath, 'wx')` call and is used by landing,
  recovery,
  and forwarded index writers.
- `runCliGit` waits before forwarding a classified index writer.
- `runDirectFix` holds `landing.lock` across `installDirectFix`.

### New modules

- `src/git-child-environment.ts`:
  the `core.lockfilePid` environment overlay.
- `src/index-lock/index-lock-evidence.ts`:
  lock `lstat` (device,
  inode,
  ctime),
  PID file parse,
  owner start time from the process-identity module,
  and the verdict:
  proven alive,
  dead,
  or evidence-free.
- `src/index-lock/index-lock-holders-linux.ts`:
  `/proc/<pid>/fd/*` stat matched by device and inode,
  recording unreadable processes as partial evidence.
- `src/index-lock/index-lock-holders-darwin.ts` and `src/index-lock/index-lock-holders-win32.ts`:
  platform holder evidence (see risks).
- `src/index-lock/index-lock-wait.ts`:
  unbounded wait with one stderr line naming the holder for proven owners;
  Git-style quadratic backoff with jitter up to `indexLock.unprovenOwnerTimeoutMs` otherwise;
  evidence re-read on every attempt;
  never deletes a foreign lock.
- `src/index-lock/index-writer-commands.ts`:
  classification of forwarded index writers from parsed argv
  (`add`,
  `rm`,
  `mv`,
  `restore --staged`,
  `reset` except `--soft`,
  `stash`,
  `checkout`,
  `switch`,
  `merge`,
  `rebase`,
  `cherry-pick`,
  `revert`,
  `apply --cached`/`--index`,
  `update-index`,
  `read-tree`,
  `am`,
  `pull`,
  `sparse-checkout`),
  each with a flag recording whether a fixture proved "fails before side effects".
- A new engine-failure code for the timeout diagnostic (proposed `index-lock-unproven-owner`),
  added to `EngineFailureCode` (`src/policy-engine/events.ts:82`) and `SPEC.md`.

### Tests and fixtures

- `git-child-environment.unit.test.ts`:
  existing `GIT_CONFIG_COUNT` preserved;
  a real `git add` under the overlay leaves `index~pid.lock` while blocked by a hook.
- `index-lock-evidence.unit.test.ts` on disposable repositories:
  live PID file owner,
  PID file naming an exited process,
  PID file naming a process started after the lock's ctime,
  no PID file,
  and a lock held open by a child process.
- `index-lock-wait.unit.test.ts` with injected clock and jitter:
  backoff schedule,
  timeout diagnostic listing evidence,
  unbounded wait released when the holder exits.
- One disposable fixture per re-forwarded command proving it fails before any side effect on `EEXIST`.
- Packed:
  concurrent `git add` during a commit hook waits and then succeeds;
  a leftover lock from a killed Git produces the diagnostic after the timeout and the lock remains.

### Risks and unknowns

- macOS and Windows holder scans have no Node API:
  `lsof` on macOS,
  Restart Manager on Windows (native calls reachable only through PowerShell P/Invoke).
  Cost and permissions are unmeasured;
  a per-attempt PowerShell start would dominate the wait.
- Re-forwarding after `EEXIST` needs to recognize Git's lock failure,
  but forwarded Git inherits stderr (`src/worktree-copy/lifecycle.ts:88`).
  Capturing stderr would change Git's terminal detection for color and progress.
  Settled:
  cli-git does not re-forward;
  forwarded index writers pre-wait and coordinate through the landing lock ("Index-writer coordination").
- The first release that has `core.lockfilePid` was unrecorded here.
  It is Git 2.54.0 (`SPEC.md` "Compatibility and degradation").
- Linux `/proc` scans over every process are unmeasured on hosts with many processes.

## Slice 7: single-flight auto-push per branch

### Owners today

- `autoPush` (`src/auto-push.ts:235`):
  probes upstream,
  `origin`,
  and symbolic `HEAD`,
  then runs `git push` or `git push --set-upstream origin HEAD` directly against real Git;
  failures keep exit `0`.
- Call site:
  `src/bin.ts:396`,
  after the post-commit gate.

### Seam

`autoPush` gains the landed OID and branch from the landing result
and runs its push inside a per-branch single-flight coordinator.

### New modules

- `src/auto-push-single-flight.ts`:
  owner-lock at `<common-dir>/cli-git/push/<encoded ref>.lock`
  plus `last-pushed.json` (tip OID,
  outcome,
  owner).
  A landed commit whose OID is an ancestor of an in-flight push tip waits for that push's result
  (`git merge-base --is-ancestor`);
  otherwise it takes the lock,
  resolves the current branch tip,
  pushes with today's argument selection,
  and records the tip on success.
  It exits once a successful push covers its OID.
- `src/auto-push-branch-key.ts`:
  filesystem-safe encoding of the ref name.

### Tests and fixtures

- Extend `src/auto-push.unit.test.ts`,
  which already builds local bare remotes:
  several landings produce fewer pushes than commits and the remote contains every OID;
  a joined push failure is reported by every joiner with exit `0`;
  a killed pusher is taken over after the liveness check;
  detached `HEAD` and missing remote skips are unchanged.
- Packed:
  concurrent commits with a local bare remote in `built-post-commit-routing-consumer.ts`.

### Risks and unknowns

- A joined commit's own pre-push hook never runs;
  the joined push ran it once for the tip.
- With a plain `git push`,
  the pushed tip can be newer than the resolved one;
  recording the resolved tip under-reports coverage,
  which only causes an extra push.

## Slice 8: concurrent fixtures and lifecycle benchmarks

### Owners today

- `perf/lifecycle-latency-*.ts` harness,
  `perf/lifecycle-latency-2026-07-16.json` baseline,
  and the `perf:lifecycle-latency` task in `mise.toml`
  (Podman,
  2 GiB,
  2 CPUs,
  six warm-up samples,
  30 recorded samples).
- `SPEC.md` "Benchmark method" and "Required disposable fixtures".

### New modules

- `perf/concurrent-commit-latency-benchmark.ts`,
  `-contracts.ts`,
  `-fixture.ts`,
  `-scenarios.ts`,
  `-summary.ts`,
  mirroring the lifecycle files,
  and a `perf:concurrent-commits` task shaped like `perf:lifecycle-latency`.
- Scenarios:
  concurrency levels 1,
  2,
  4,
  and 8 with disjoint paths;
  same-file non-overlapping replays;
  conflicting pairs;
  a slow hook with `hooks.concurrentCommits` false and true;
  a sweep of `landing.reserveAfterLostRaces` to confirm the default of 2 by tail completion time.
- Metrics:
  per-commit completion time,
  landing lock hold time,
  real `index.lock` hold time,
  lost races per commit,
  and the paired serialized baseline.
- A re-measured lifecycle baseline stored as a new dated `perf/lifecycle-latency-<date>.json`,
  because every commit now prepares privately.
- Packed fixtures:
  `src/trust/fixture/built-concurrent-commit-consumer.ts` plus helpers for each slice's packed cases,
  and the `SPEC.md` fixture list extended with every case named in slices 1 to 7.

### Risks and unknowns

- `test:built:trust` runs with one CPU;
  interleavings must be forced with hook barriers rather than left to scheduling.
- Timing comparisons need the run-to-run band measured first (QNB).

## Order of implementation

1.  Update `SPEC.md` and `README.md` for the accepted design,
    new config keys,
    `inputs`,
    JSONL additions,
    and fixture list (handover next action 2).
2.  Extract `src/owner-lock/` from worktree-copy and add `src/git-child-environment.ts`.
    No behavior change except PID files from spawned Git.
3.  Slice 1 journals and recovery,
    still with today's single-lock transaction,
    so live-owner skipping ships first.
4.  Slice 6 lock acquisition and wait for cli-git's own landing
    (`index-lock-evidence`,
    Linux holders,
    `index-lock-wait`).
    Depends on 2.
5.  Slice 5 config keys and the hook lock.
    Depends on 2.
6.  Slice 2 private preparation and shim.
    Depends on 3,
    4,
    and 5.
7.  Slice 3 landing without replay
    (moved `HEAD` fails fast,
    the decision's fallback),
    then replay with every policy treated as unrestricted.
    Depends on 6.
8.  Slice 4 read sets and `inputs`,
    then shipped-plugin declarations.
    Depends on 7.
9.  Slice 5 reservation.
    Depends on 7.
10. Slice 7 single-flight auto-push.
    Depends on 7.
11. Slice 6 forwarded index-writer waits and macOS/Windows holder evidence.
    Depends on 4.
12. Slice 8 benchmarks,
    lifecycle re-baseline,
    and `reserveAfterLostRaces` confirmation.
    Packed fixtures land with each slice rather than here.

## Existing tests that change

- `src/trust/fixture/built-autofix-concurrency-consumer.ts`:
  inverts from "second invocation refused" to "second invocation proceeds".
- `src/trust/fixture/built-autofix-recovery-consumer.ts`,
  `built-autofix-recovery-adversarial.ts`,
  `built-autofix-recovery-completed.ts`,
  `built-autofix-added-paths-recovery.ts`,
  `built-final-newline-normalization-recovery.ts`,
  `built-racy-index-install.ts`,
  and `built-autofix-filesystem-consumer.ts`:
  they address `.git/cli-git-transaction` and its fixed file names.
- `src/bin.unit.test.ts` and `src/read-only-forwarding.unit.test.ts`:
  they reference the transaction directory and forwarding flow;
  clean commits no longer forward natively.
- `src/policy-engine/post-commit-lifecycle.unit.test.ts`:
  the landed OID is passed in rather than resolved from `HEAD`.
- `src/auto-push.unit.test.ts`:
  new signature and single-flight cases.
- `src/worktree-copy.unit.test.ts` and `src/worktree-copy-fixture.unit.test.ts`:
  regression proof for the owner-lock extraction.
- `src/policy-engine/engine.unit.test.ts`:
  per-policy contexts and returned read sets.
- `src/api.unit.test.ts`:
  `inputs` export.
- `perf/lifecycle-latency-commit-scenarios.ts`:
  scenario expectations change once clean commits prepare privately.

## Questions for the owner

All settled;
the decision's "Implementation-time decisions" records each answer.

- Private `HEAD` shape:
  neither option;
  a shadow repository with alternates.
- Worktree-copy settlement lock versus private preparation in linked worktrees:
  only commands that create or move worktrees take the lock.
- Static `inputs` versus option-derived `inputs`:
  both are accepted.
- Reservation order when several commits cross the threshold:
  oldest invocation first.
- JSONL additions under `schemaVersion: 1` versus a schema bump:
  additive under `schemaVersion: 1`.
- Sequencer conclusions if the admin-dir state-copy prototype fails:
  the prototype succeeded in the shadow repository.
