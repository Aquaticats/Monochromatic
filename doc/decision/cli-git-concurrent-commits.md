# cli-git concurrent commits

## Status

Accepted 2026-09-25 after a design grilling session.
The evidence,
probes,
rejected options,
and per-question rationale live in
[`doc/planning/cli-git-concurrent-commits.md`](../planning/cli-git-concurrent-commits.md).
The execution record is
[`doc/handover/cli-git-concurrent-commits.md`](../handover/cli-git-concurrent-commits.md).
The canonical implementation interface is `package/git-policy/cli/SPEC.md`.

## Decision

Several `git commit` invocations against the same worktree and branch run at the same time,
and each lands as its own sequential commit.
Native Git and today's cli-git reject every overlapping commit after the first with `index.lock` `EEXIST`;
issue #560 was a real collision.

The behavior is on in every repository the wrapper runs in,
with no configuration or environment opt-out:
a misbehaving wrapper is a bug to fix,
not a reason to disable parts of it.

## Parallel preparation

- Each commit prepares without the real index lock.
  Explicit-path commits capture selected worktree bytes at invocation;
  index commits capture the real index at invocation.
- Preparation runs native `git commit` against a private index and a private admin dir `HEAD`
  whose `commondir` names the real common dir,
  with `--work-tree` at the real worktree.
  Git then owns hooks,
  the editor,
  templates,
  message cleanup,
  and signing.
- The pending commit is protected from `gc` until landing,
  abort,
  or recovery.
  The design session chose a private ref under `refs/cli-git/`;
  the implementation replaced it with a per-transaction shadow repository
  whose own object store holds the pending commit,
  so no `refs/cli-git/` ref exists
  (see "Private `HEAD` shape:
   shadow repository with alternates").
  The shadow repository stays unregistered.
- Hooks run through a dispatcher shim passed with `-c core.hooksPath=<shim>`
  plus `-c hook.<event>.enabled=false` for each event.
  The shim resolves the repository's own `core.hooksPath`,
  respects user-disabled events,
  exports an absolute `GIT_WORK_TREE`,
  restores the caller's `GIT_CONFIG_PARAMETERS`,
  runs `git hook run` for `pre-commit`,
  `prepare-commit-msg`,
  and `commit-msg`,
  and omits `post-commit`.
  Shim executables are not shell scripts.
- A per-repository hook lock serializes preparation hooks and the post-landing `post-commit`
  unless `cli-git.config` sets `hooks: { concurrentCommits: true }`.

## Serial landing

- Landing holds the cli-git landing lock and the real `index.lock`,
  advances the target ref by compare-and-swap with a `-m` reflog message carrying the recovery nonce,
  installs the real index computed against the then-current real index,
  and releases.
- When `HEAD` did not move,
  the prepared commit itself lands,
  keeping its signature.
- When `HEAD` moved,
  a path that the prepared commit and a commit landed since its base both captured from the same worktree
  takes the later capture's bytes
  (see "Capture order" in "Implementation-time decisions").
  Replay resolves every other path that both the prepared commit and the landed history changed
  by a subsumption check first:
  when the landed change for that path
  (its diff from the preparation base to the new `HEAD`)
  applies in reverse to the prepared version,
  the prepared bytes already contain it and land as they are,
  matching what native sequential commits produce in a shared worktree.
  Otherwise replay three-way merges from the preparation base
  (`git merge-tree --write-tree --merge-base=<preparation base>`).
  Owner decision 2026-09-26,
  amending the original pure three-way merge
  after the container trace scenario showed it conflicting on adjacent edits
  whose captured file already held the other commit's landed edit.
  The replayed commit is rebuilt preserving author,
  committer,
  and message
  (see "Conclusions and replay").
- A replay conflict fails without landing,
  as a `core-finding` JSONL event with exit `1`
  naming the conflicting paths and the winning commit.
  cli-git never re-prepares from current worktree bytes.
- After a clean replay,
  cli-git policies re-run only when their recorded reads changed,
  and `pre-commit` re-runs outside the landing lock against the replayed index whenever the replayed tree differs.
  Landing then retries.
- Commits land in preparation completion order.
  After `landing.reserveAfterLostRaces` lost races
  (default 1 since 2026-09-26,
  2 before;
  see "Implementation-time decisions"),
  a commit reserves the next landing slot.
- `--amend`,
  merge,
  cherry-pick,
  and revert conclusions fail when `HEAD` moved.
  Every commit records the symbolic `HEAD` target at invocation
  and fails if a branch switch changed it.
- `post-commit` runs once after landing through `git hook run post-commit` in the real worktree
  with native-equivalent `GIT_INDEX_FILE`,
  `GIT_AUTHOR_*`,
  and `GIT_EDITOR=:`.
- Before `post-commit`,
  landing runs `git maintenance run --auto` as native `git commit` does,
  because every landing adds one pack to the real object store
  and unconsolidated packs slow every later Git process in the repository.
  Added 2026-09-26 (veto open);
  evidence:
  [`doc/troubleshooting/git-plumbing-commit-auto-maintenance.md`](../troubleshooting/git-plumbing-commit-auto-maintenance.md).

## Policy inputs

`PolicyDefinition` gains
`inputs?: 'unrestricted' | { readonly external: readonly PolicyInput[] }`,
default `'unrestricted'`.
An unrestricted policy always re-runs after a replay.
`{ external: [] }` declares context-only reads.
`PolicyInput` kinds are `worktree` (pathspecs),
`executable` (path),
`revision` (rev),
and `env` (name);
cli-git fingerprints them.
cli-git records each policy's lazy context reads
(candidate bytes,
`trackedFiles` pathspecs,
`headOid`)
and re-runs a declared policy only when a recorded read or declared input changed.
Precedent:
 Nx task `inputs`,
broad when absent.

## Locks

- cli-git injects `core.lockfilePid=true` into every forwarded and spawned Git,
  so native Git leaves an owner PID file.
- An `index.lock` owner proven alive
  (open holder matched by device and inode,
  or a PID file naming a live process started no later than the lock's ctime)
  gets an unbounded wait with one line naming the holder.
- A dead or evidence-free owner gets Git-style quadratic backoff with jitter
  up to `indexLock.unprovenOwnerTimeoutMs`
  (default 1000),
  then a diagnostic listing the evidence.
- cli-git never deletes a lock.
- Forwarded index writers
  (`add`,
  `rm`,
  `mv`,
  `restore --staged`,
  `reset`,
  and similar)
  wait under the same rules.
  The design session also allowed re-forwarding after a lock `EEXIST`;
  the implementation does not re-forward
  (see "Implementation-time decisions").
- `git cli-git fix` holds the landing lock across installation and real-index verification.
- Windows holder detection uses the PID file and Restart Manager,
  never a `DELETE`-access probe.

## Auto-push

Pushes are single-flight per branch.
A landed commit joins an in-flight push covering its OID or pushes the branch tip itself,
and exits only once the remote contains its OID.
A push that the remote refuses as stale is reconciled by a merge made from objects, signed when the repository signs
commits, and published with a plain push; since 2026-10-19 the branch and a clean worktree are then fast-forwarded to
it.
The specification is the "Auto-push" section of `package/git-policy/cli/SPEC.md`.

## Recovery and compatibility

- Transaction journals are per transaction;
  recovery handles every prepared,
  reserved,
  or landing transaction,
  and a crashed owner releases its reservation through the owner-liveness check.
- Missing optional Git features degrade per feature;
  missing replay plumbing falls back to today's fail-fast behavior.

## Container end-to-end verification

Concurrent mutation of shared Git state is only verified by running it concurrently against real repositories,
so container end-to-end tests with freshly created dummy repositories replaying realistic workloads
are an inherent part of this design.
Unit and packed shadow-bin fixtures alone cannot show interleavings,
crash recovery,
or lock contention.
The design session omitted this suite until the owner pointed it out;
the agent-behavior gap is tracked separately.

- The suite follows the `test:built:trust` precedent:
  a mise task packs the npm tarball
  and runs a consumer script inside `podman`
  with stated memory and CPU bounds.
- The image provides every Git version under test:
  the declared minimum and the current release.
  Distribution images that ship an older Git do not qualify.
- Each run creates new repositories and a local bare remote inside the container,
  never touching host repositories.
- Workloads come from two sources:
  commit-shape traces mined from this repository's own history
  (files per commit,
  path overlap,
  sizes,
  additions,
  deletions,
  renames,
  binary files)
  with synthesized content,
  and a scenario catalog of concurrent agent behavior:
  shared-file edits with overlapping and non-overlapping hunks,
  interleaved index writers,
  hookdir and config-based hooks,
  lint-staged-style stash hooks,
  `commit-msg` and `post-commit` hooks,
  SSH signing,
  amend attempts,
  branch switches,
  foreign `index.lock` holders,
  `gc --prune=now` during preparation,
  and `SIGKILL` injected at every transaction phase followed by recovery.
- Runs are seeded;
  a failing seed replays deterministically.
- Every run checks invariants:
  each commit that exited `0` appears exactly once with exactly its captured bytes;
  no worktree edit is lost;
  the real index never stages a revert of landed content;
  the remote contains every landed OID;
  no `refs/cli-git/` ref,
  transaction directory,
  or lock remains;
  `git fsck` is clean;
  exit codes match the JSONL events.

## Implementation-time decisions

Made by the agent on 2026-09-25 while the owner was asleep,
under the owner's instruction to work autonomously
and to build both options when two look equally good.
Veto open.
Evidence:
`package/git-policy/cli/doc/concurrent-commits-implementation-plan.md`.

- Every non-dry-run commit uses private preparation,
  including clean commits and `commit -a`.
  Before this change a clean commit returned early and ran native `git commit`,
  which holds `index.lock` through hooks and the editor,
  so it is the #560 collision path.
  The lifecycle latency baseline is re-measured.
- Startup recovery skips transactions whose owner is alive;
  it no longer blocks every invocation while another commit runs.
- Every read of live `HEAD` in the transaction uses the recorded preparation base or the landed OID.
- Recovery searches the target ref's reflog for its nonce instead of checking only the newest entry.
- The worktree-copy settlement lock is held only by forwarded commands that,
  after alias resolution,
  create or move worktrees.
  Before this change it serialized every forwarded command in a linked worktree
  and gave up after about 1 second,
  which would serialize private preparation.
  Worktrees created by Git invoked through an absolute path from a hook lose automatic ignored-state copying,
  matching the documented bypass of everything else.
- The hook lock is taken by the dispatcher shim around each hook event,
  so an open message editor never holds it.
- `inputs` may be static or a function of the validated policy options,
  so option-dependent inputs such as a configured executable are expressible.
  The two shipped repository policies declare `{ external: [] }`.
- Reservations are granted oldest invocation first.
- Subsumption reads "applies in reverse" as strict `git apply --reverse --check`
  or a one-sided extension of the landed hunks,
  added 2026-09-26 (veto open):
  the strict check (and `-C1`) rejects exactly the adjacent-edit case the decision targets,
  because the prepared edit sits in the landed hunk's context,
  and `-C0` also accepts landed deletions the prepared bytes never made.
  Additions on both sides are checked against an empty base.
  Rules and evidence:
  `package/git-policy/cli/SPEC.md` "Subsumption".
- Capture order,
  owner decision 2026-09-26:
  every capture from a worktree takes a short per-worktree capture lock
  and records a monotonically increasing capture sequence number,
  giving a total order of captured disk states.
  For a path that both the prepared commit and a commit landed since its base captured from the same worktree,
  the later capture's bytes land:
  a prepared commit captured later lands its own bytes,
  and one captured earlier keeps the landed bytes for that path.
  This records what native sequential commits would record from the shared disk.
  Paths changed from anywhere else
  (another worktree or clone,
  a native commit that bypassed cli-git)
  still go through subsumption,
  then three-way merge.
  Evidence:
  the container `concurrent-trace-replay` scenario still conflicted under subsumption
  wherever a later capture rewrote lines an earlier in-flight commit had just added.
  Accepted cost:
  a later capture from a stale editor buffer reverts the earlier edit,
  exactly as native Git would.
- Plain index commits record no worktree-captured paths,
  added 2026-09-26 (veto open):
  an index commit without `-a` or `--include` commits bytes staged at an unknown earlier time,
  not a disk state read at its capture,
  so its capture sequence number says nothing about when those bytes were written.
  Ordering them by capture could let a stale staged blob win over an edit a later-landed commit captured from disk,
  reverting landed content.
  Its shared paths keep subsumption,
  then the three-way merge.
  `commit -a` and `--include` record the paths whose private index entry differs from the captured real index,
  because those bytes were read from the worktree at capture.
  Rules:
  `package/git-policy/cli/SPEC.md` "Worktree-captured paths".
- The container checker's `landed-bytes` invariant accepts a later capture's landed bytes under capture order,
  added 2026-09-26 (veto open):
  for an explicit-path attempt,
  a selected path may hold the landed parent's bytes
  when an attempt started later captured exactly those bytes for the same path
  and that attempt's commit lies between `HEAD` before the invocation and the parent.
  This is the case capture order prescribes,
  where an earlier capture keeps the landed bytes of a later capture that landed first.
  The acceptance is that narrow,
  so any other landed parent bytes still fail the invariant.
  Evidence:
  a first seed-3 run failed `concurrent-trace-replay` on Git 2.40.0 with `landed-bytes` for exactly this case.
  Rules:
  `package/git-policy/cli/e2e/README.md` "Invariants".
- The trust registry's recursive-operation lock follows "Locks",
  added 2026-09-26 (veto open):
  it is an owner lock hardened to registry modes and Windows ACLs before publication,
  a live owner gets an unbounded wait,
  and only an unproven owner
  (a PID-only record from an earlier build naming a running PID,
  or a malformed or missing record)
  gets the 1000 ms backoff.
  Its fixed 100 x 10 ms budget failed 16 of 16 concurrent `trust --yes` behind a live owner.
  Built-wrapper tests reach a disposable registry through a test-only `NODE_OPTIONS=--import` preload
  that makes `os.userInfo().homedir` report `HOME`;
  production still ignores `HOME`.
  Rules:
  `package/git-policy/cli/SPEC.md` "Locks".
- Trust provenance readers follow "Locks",
  added 2026-09-26 (veto open):
  config loading,
  `status`,
  and the deleted-config `untrust` lookup read the registry without the recursive-operation lock
  while no provenance journal is published,
  and otherwise take the lock,
  waiting while a live `untrust` settles,
  then recover only journals of dead holders.
  Before,
  they recovered unlocked and failed with "Recursive trust transaction is active in another process"
  whenever a journal named a running PID,
  so a commit overlapping another repository's `untrust` exited `2`.
  Rules:
  `package/git-policy/cli/SPEC.md` "Recursive enrollment and revocation".
- Git feature degradation is detected by exercising each feature,
  added 2026-09-26 (veto open):
  there is no version gate,
  and replay plumbing is probed after a commit's first lost race
  by running `git merge-tree --write-tree --merge-base` on the winning commit
  (exit 129 means absent),
  because the usage text changed its spelling between Git 2.40.0 and 2.55.0.
  Without it the commit fails with `concurrent-commit/head-moved`.
  Before,
  Git 2.39.5 ran ordinary commits,
  and the commit that lost a race exited `2` with `transaction-failed`
  carrying `git merge-tree` usage text.
  The container suite runs Git 2.39.5 with the replay-free scenarios and two degradation scenarios.
  Rules and observations:
  `package/git-policy/cli/SPEC.md` "Compatibility and degradation".
- `landing.reserveAfterLostRaces` defaults to 1,
  changed 2026-09-26 (veto open)
  by the owner's rule:
  most correct first,
  then most performant;
  when the per-commit p95 of the best settings differ by less than the run-to-run band,
  the lower per-commit median wins,
  otherwise the lower p95 wins.
  Every setting lands the same commits with the same bytes
  and bounds a holder's lost races at the setting + 1,
  so performance decided.
  Four rotated-order sweeps of the build that runs automatic maintenance after landing
  gave per-commit p95 of 2630 to 2774 ms for 1
  and 2823 to 3145 ms for 2
  (bands 144 and 322 ms,
  means 232 ms apart),
  and per-commit medians of 2009 to 2062 ms against 2278 to 2544 ms;
  1 led on both in every run,
  so either branch of the rule picks it.
  4 and 8 were slower than both in every run.
  Evidence:
  `package/git-policy/cli/SPEC.md` "Benchmark method".
  Earlier sweeps kept 2,
  measured 2026-09-26 before maintenance ran after landing:
  a single sweep had suggested 1
  (per-commit p95 3850 ms against 4581 ms),
  but four rotated-order sweeps of one build put the p95 difference of 1 and 2 (91 ms)
  far inside either setting's run-to-run band (about 1150 to 1270 ms),
  with 1 ahead in only two runs.
  Setting 1 does lower the per-commit median in every run
  (2586 to 2767 ms against 3024 to 3106 ms),
  which would favor it if median latency rather than tail latency decided the default.
  Evidence:
  `package/git-policy/cli/SPEC.md` "Benchmark method".
- Forwarded index writers coordinate with landings through the cli-git landing lock
  and pre-wait for foreign `index.lock` holders;
  cli-git does not capture Git's stderr to detect a lock failure and re-forward,
  because capturing stderr changes Git's color and progress output.
  A residual race remains only with processes that bypass the wrapper.
- New JSONL event types are additive under `schemaVersion: 1`;
  the SPEC states that consumers ignore unknown event types.
- A failed auto-push keeps today's exit contract:
  the landed commit exits `0` after surfacing the push failure.
- macOS and Windows holder detection spawns `lsof` or a Restart Manager query only while a foreign lock is present.

### Private `HEAD` shape: shadow repository with alternates

A prototype in disposable repositories
(real Git 2.55.0)
compared the candidate shapes with a pre-commit hook that reads the branch name,
the upstream,
`includeIf onbranch:`,
and runs a "reject commits to main" check.

- Chosen:
  a shadow repository per transaction at `<common-dir>/cli-git/shadow/<id>`,
  with its own `HEAD` symbolic to a private copy of the target branch ref,
  a private copy of the branch's upstream ref,
  and `objects/info/alternates` naming the real object store.
  Hooks see the real branch name,
  the upstream,
  and `onbranch:` includes,
  and the "reject main" hook rejects.
  New objects stay in the shadow store,
  so a real `git prune --expire=now` cannot delete a pending commit
  and a prune run inside the shadow cannot delete real objects.
- Required hardening,
  each observed necessary in the prototype:
  the shadow config copies `repositoryformatversion` and `extensions.*` explicitly
  (an include does not apply them),
  sets `core.hooksPath` to the real hooks path before `[include] path=<real config>`
  so a real `core.hooksPath` still wins,
  and disables `gc.auto` and `maintenance.auto`;
  every real Git-dir entry that is not transaction-private
  (such as `info`,
  `rr-cache`,
  `lfs`,
  and `modules`)
  is symlinked into the shadow by default,
  and only `HEAD`,
  refs,
  logs,
  `index`,
  `config`,
  `objects`,
  and the copied per-worktree state stay private;
  `config.worktree` is copied.
- The shadow's ref store holds a snapshot of every real ref at invocation
  with the target branch replaced by the private copy,
  so hooks resolve tags,
  other branches,
  and remote-tracking refs.
  A files-backend snapshot is written as one `packed-refs` file:
  a median 12.2 ms for 20,001 refs against 4166.9 ms for creating loose refs
  (measured 2026-09-25).
  Reftable snapshots use one `update-ref --stdin` transaction.
- Windows creates directory junctions instead of symbolic links for shared directories,
  because symbolic links need a privilege or developer mode there,
  and copies shared files;
  Windows verification runs in CI.
- Landing migrates the shadow's new objects into the real store as a pack kept with a `.keep` file
  until the compare-and-swap succeeds,
  so neither `git prune` nor repacking can drop them in between.
- No `refs/cli-git/` pending ref exists;
  the shadow store protects the pending commit.
- Culled:
  detached private `HEAD` and a per-worktree ref
  (hooks see no branch and no upstream,
  so the "reject main" hook is bypassed);
  a symbolic pending ref
  (same hook bypass;
  kept only as the fallback if the shadow repository hits a blocker);
  `GIT_REFERENCE_BACKEND` with a private ref store
  (Git passes it into submodules and `git -C` from hooks sees private refs,
  and pseudorefs must be kept in two places);
  and a shadow repository using `GIT_OBJECT_DIRECTORY`
  (a prune inside the shadow deleted objects reachable only from real refs).
- Residual risk:
  a Git-dir-derived path that is neither symlinked nor copied points into the shadow.
  The container end-to-end suite covers LFS,
  submodules,
  sparse checkout,
  reftable,
  and SHA-256 repositories.

### Conclusions and replay

- Merge,
  cherry-pick,
  and revert conclusions work in the shadow:
  copying the per-worktree state files in produced commits byte-identical to native.
  Landing reproduces native cleanup in the real worktree's admin dir:
  it removes the files native removed
  (`AUTO_MERGE`,
  `MERGE_HEAD`,
  `MERGE_MODE`,
  `MERGE_MSG`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`),
  copies the private `MERGE_RR` back,
  keeps `ORIG_HEAD`,
  and leaves `sequencer/` as native leaves it.
  Reftable repositories delete pseudorefs with `git update-ref -d` instead of removing files.
- Landing runs `git update-ref` in the owning worktree's context with no `GIT_DIR` override,
  because only then does Git also write the real `HEAD` reflog.
- Replay computes the tree with `git merge-tree --write-tree --merge-base`
  and rebuilds an unsigned commit by rewriting the raw commit object
  (`git hash-object -t commit -w`,
  replacing only `tree` and `parent` lines),
  which preserved `encoding`,
  exact identities and dates,
  and custom headers.
  `git replay` and plain `git commit-tree` both transcoded non-UTF-8 messages and dropped headers.
  A signed commit is rebuilt with `git commit-tree -S`
  under the original identities,
  dates,
  and `i18n.commitEncoding`;
  custom headers on a signed commit are dropped with a warning event,
  because no primitive re-signs a raw object.

### Amending published history

Owner decision 2026-09-26:
when an amend of an already-published commit lands while other commits are in flight,
those commits replay onto the amended history
and auto-push cannot fast-forward the remote,
exactly as native Git behaves.
The container checker exempts commits landed on an amended published history from `remote-contains`
and instead requires the non-fast-forward push failure to be surfaced with exit `0`.
Refusing `--amend` on published commits was rejected
because it would change amend behavior in every repository,
including solo use.

Narrowed 2026-10-19 (user, by question; the original decision above stays as the record):
auto-push never fails for want of a conflict-free merge,
so commits landed on top of an amend of an already-published commit are merged into the remote branch,
and the replaced commit stays in the remote history beside the amended one.
Only a real conflict between the two histories keeps the rejection,
and it is surfaced with the paths that conflict.
The container checker now accepts either the original failure note or the `cli-git: auto-push: not published:` line
for that case.

## Rejected

- A lock queue around today's unchanged transaction.
- Batch syntax creating several commits in one invocation.
- Path-level replay,
  automatic re-preparation on conflict,
  landing in invocation order,
  and hooks inside the landing lock.
- `extensions.worktreeConfig` for the hook work tree:
  it mutates every repository's config and still needs the shim to hold back `post-commit`.
- Opt-out config keys and environment variables.
