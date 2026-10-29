# Concurrent-commit container suite

End-to-end workload replay for cli-git concurrent commits.
It is the "Container end-to-end verification" part of
[`doc/decision/cli-git-concurrent-commits.md`](../../../../doc/decision/cli-git-concurrent-commits.md)
and slice 8 of
[`doc/concurrent-commits-implementation-plan.md`](../doc/concurrent-commits-implementation-plan.md).
Fresh dummy repositories replay realistic concurrent-agent workloads through the native `git` wrapper (`src/native/`),
and an invariant checker judges every run.

## How to run

```sh
# package/git-policy/cli: run from the repository root
mise run //package/git-policy/cli:test:e2e:concurrent:native --git 2.55.0
mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1 --git 2.55.0
mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1 --scenario shared-file-overlapping-hunks --git 2.55.0
```

- `test:e2e:concurrent:native` depends on `test:e2e:concurrent:native:image`,
  which compiles the release executable from the checkout and builds the base image when it does not exist.
  Every run therefore tests the current source.
- Every run prints the selected wrapper and the executable it verified,
  its seed and a replay command,
  then one line per scenario and Git version,
  then a summary.
  The task exits non-zero when any scenario fails or errors.
- `--scenario` takes comma-separated scenario names;
  `--git` takes comma-separated versions, and the image provides `2.55.0` only.
- The native executable requires Git 2.55.0 or newer and runs nothing on an older Git,
  so the image builds that one version.
  On 2026-10-14 with seed 1, 36 scenarios passed and 2 skipped.

The TypeScript wrapper that this suite first verified is retired;
 its last commit is
`f4fc1fa213ece65672e4314983a75266837892f9`,
 and the sections that describe measurements say so where they cite it.

- `E2E_WRAPPER` (empty or `native`,
   the only wrapper) names the directory whose `git` shadows the real Git
  (`wrapper-selection-fixture.ts`);
  the consumer checks that this `git` is an ELF executable and prints the result first.
- `test:e2e:concurrent:native:image` compiles the release `cli-git-native` in `rust:1.97-bookworm`
  and links it as `/opt/native/bin/git` in a copy of the `cli-git-concurrent-e2e` base image.
  `GIT_POLICY_NATIVE_IMAGE_TAG` names the native image's tag, so concurrent worktrees never share one.
- `test:e2e:concurrent:image` builds the base image alone (Node,
   the three Git builds,
   `ssh-keygen`);
  `GIT_POLICY_E2E_BASE_TAG` names a tag other than `latest`,
   so a trial build never replaces the image other runs use.

Regenerate the committed trace with real Git on the host.
It mines this checkout by default;
`--repository <path>` selects another checkout of the same history,
and `--limit` and `--revision` adjust the window.

```sh
# package/git-policy/cli: bypasses the wrapper through /usr/bin/git
mise run //package/git-policy/cli:e2e:concurrent:trace
```

## Container

`concurrent-commits.Containerfile` builds `cli-git-concurrent-e2e`:

- Node 24.11.0,
  the version the suite has always run with;
- Git 2.55.0,
  built from a checksum-pinned kernel.org tarball,
  installed under `/opt/git/2.55.0`;
- no Git on the default `PATH`,
  so each scenario picks its Git by `PATH` order.

The native image adds `/opt/native/bin/git`,
 a link to the release executable,
 on top of this image.

The native executable requires Git 2.55.0 or newer and runs nothing on an older Git
(`SPEC.md` "Compatibility and degradation"),
so the image carries no other version.
The rest of this section records why the image once also carried 2.39.5 and 2.40.0:
the retired TypeScript wrapper degraded on them.
The `replayPlumbing` skips and the `replay-unavailable-*` scenarios below stay in the catalog for that history,
and no supported run exercises the `absent` ones.

For the retired wrapper,
 Git 2.40.0 was the oldest release with every feature it used,
because it is the first whose `git merge-tree` accepts `--merge-base`:
`Documentation/RelNotes/2.40.0.adoc` says "`merge-tree` learns a new `--merge-base` option",
and `--write-tree` itself arrived in 2.38.0.
Git 2.39.5,
Debian bookworm's Git,
proved that the retired wrapper degraded without that option
(`SPEC.md` "Compatibility and degradation"):
a commit that loses a landing race fails fast with `concurrent-commit/head-moved` instead of replaying.
Git 2.55.0 is the current release;
it builds with its default Rust support enabled.
Config-based hooks (`hook.<name>.command`) arrived in Git 2.54.0,
so the config-hook scenarios skip on 2.39.5 and 2.40.0.

Each scenario declares how it depends on replay plumbing
(`replayPlumbing` in `scenario-model-fixture.ts`):

- `required`,
  the default,
  skips it on 2.39.5,
  where its commits that lose a landing race would fail fast by design;
- `unused` runs it on every Git that meets its `minimumGit`,
  because none of its commits can lose a landing race:
  the baselines,
  `branch-switch-during-commit`,
  the hook-phase `sigkill-*` scenarios,
  and the `sigkill-phase-*` scenarios;
- `absent` runs it only on 2.39.5,
  to prove the degradation:
  the `replay-unavailable-*` scenarios.

The run uses `podman run --memory=2g --cpus=2 --rm --network=none`.
`e2e/` is mounted read-only at `/fixture/e2e`,
and `package/caught-value/core` is mounted read-only at `/fixture/caught-value`
for `@monochromatic-dev/caught-value-core/ts`,
which the image resolves through a `/node_modules` symlink.
No host repository is mounted writable,
and no credentials enter the container.
Every scenario creates its own repository and local bare remote under `/work/<git-version>/<scenario>/`.

Measured image build times on this host:
255 s for a build that compiles both Git versions
(base images already pulled),
measured while the suite still packed the TypeScript wrapper into the image
(the rebuild after a new pack replaced only the tarball layer;
 that layer no longer exists).

## Workloads

### Commit-shape trace

`commit-shape-trace-generator.ts` runs `/usr/bin/git log --raw --numstat -M` over the newest 4000 non-merge commits
and writes `commit-shape-trace.json`.
The trace keeps shapes only:
files per commit,
path overlap through stable numeric path IDs,
blob sizes,
added and deleted line counts,
add,
modify,
delete,
and rename kinds,
file modes,
and binary flags.
It holds no file contents,
paths,
object IDs,
messages,
or author data.

Replay picks a seeded window of consecutive commits with 1 to 16 plain-file changes
and synthesizes content for each shape
(text sizes capped at 32 KiB,
binary sizes at 8 KiB).
Paths that must pre-exist are committed by real Git before the window.
Text always ends with exactly one LF,
so the default `final-newline` normalization never changes captured bytes.
A worker that touches a path another in-flight worker selected waits until that worker's `pre-commit` hook ran,
so every attempt commits exactly the bytes the harness wrote for it.
A worker that deletes or renames away a path an in-flight worker creates
(or a path created by an in-flight worker before that one)
waits until that worker's wrapper exited instead,
because until that commit lands the path is in neither the deleting commit's preparation base nor its worktree,
and native `git commit -- <path>` fails with "pathspec did not match" the same way.
The workload then stays a valid sequential-per-path history.

### Scenario catalog

Baselines prove the harness sound and must pass on today's build:

- `baseline-explicit-commit`:
  modify,
  add,
  and delete one path each through sequential explicit-path commits.
- `baseline-index-commit`:
  `git add` two paths,
  keep an unrelated unstaged edit,
  `commit --no-only`.
- `baseline-trace-sequential`:
  12 trace commits one at a time.
- `baseline-amend`:
  amend a token-free setup commit.
- `baseline-lint-staged`:
  lint-staged's backup stash and hide-unstaged sequence on a partially staged index commit,
  in a linked worktree.
- `baseline-hooks-config`:
  `pre-commit`,
  `prepare-commit-msg`,
  `commit-msg`,
  and `post-commit` hooks on sequential commits.
- `baseline-ssh-signing`:
  sequential SSH-signed commits.
- `baseline-sigkill-post-commit`:
  `SIGKILL` inside `post-commit`,
  after native Git released every lock,
  then `git status` and a follow-up commit.
- `baseline-checker-positive-control`:
  one commit lands,
  then real Git amends it with other bytes,
  stages the replaced blob,
  and plants `refs/heads/stale.lock`.
  It passes only when the checker reports exactly
  `index-no-revert`,
  `landed-bytes`,
  `no-leftovers`,
  `remote-contains`,
  and `worktree-preserved`.

Design scenarios exercise the accepted concurrent-commit design:

- `concurrent-disjoint-paths`:
  4 to 8 agents commit disjoint files with seeded start jitter.
- `concurrent-trace-replay`:
  16 trace commits with 4 in flight.
- `shared-file-non-overlapping-hunks` and `shared-file-overlapping-hunks`:
  the first agent is held in `pre-commit` after capturing,
  the second edits the same file and gets a 1.5 s window to start,
  then the first is released.
  Far-apart hunks must both land;
  overlapping hunks may fail only with exit `1` and a `core-finding` event.
  Under capture order both captures come from one worktree,
  so the second capture's bytes land and both commits succeed.
- `shared-file-adjacent-edits`:
  the first agent edits one line and pauses after preparation
  (phase marker `preparation-done`),
  the second edits the line after it,
  the line before it,
  or inserts a line right after it
  (seeded),
  so its captured bytes hold both edits,
  and pauses the same way;
  then the first is released and lands,
  and the second is released.
  Both must land,
  the second through a replay that keeps its captured bytes
  (capture order,
  since both captures come from one worktree,
  `SPEC.md` "Capture order";
  subsumption reaches the same bytes for a change from elsewhere,
  `SPEC.md` "Subsumption").
- `interleaved-index-writers`:
  4 commits and 4 `git add` runs of new files in a seeded interleaving;
  every add must succeed and its staged entry must survive.
- `hooks-config-concurrent`
  and `ssh-signing-concurrent`:
  4 agents at once with all four hooks or with signing.
- `lint-staged-stash-concurrent`:
  the lint-staged hook while 3 explicit-path agents and 1 partially staged index commit run,
  in a linked worktree.
- `amend-during-commits`:
  an amend races 3 commits;
  it may land only on the parent it saw.
  When it lands,
  the others replay onto the amended history,
  which auto-push merges into the remote branch when the two histories do not conflict
  (see `push-rejection-surfaced` for the conflicting case).
- `reservation-after-lost-races`:
  a victim commit holds in its editor until a first winner lands,
  then loses one race per winner,
  pausing at each `race-lost-<n>` while the next winner lands,
  until its `landing.reserveAfterLostRaces`-th lost race
  (the default of 1,
  so the first winner alone)
  takes the landing reservation,
  where it pauses;
  a commit started then must not land during a 1.5 s window,
  and after release the victim lands first.
  Expectations:
  `victim-lost-races`,
  `reserved-within-bound`
  (exactly one `landing-reserved`,
   at most `landing.reserveAfterLostRaces` + 1 lost races),
  `reservation-blocks-landing`,
  and `reserved-lands-first`.
- `branch-switch-during-commit`:
  `git switch side` while a commit is held in `pre-commit`;
  the switch and the commit must not both succeed.
- `foreign-index-lock-holder`:
  real Git by absolute path with `-c core.lockfilePid=true` runs `commit --all`
  with an editor that holds `index.lock` for 1.5 s while 3 agents commit;
  the PID file proves the owner alive,
  so every agent must wait and succeed.
  It skips Git older than 2.54.0,
  which has no `core.lockfilePid`.
- `foreign-index-lock-unproven`:
  the same holder without `core.lockfilePid`,
  so no evidence proves it alive;
  each agent either lands after the holder finishes
  or fails with exit `2` and only `index-lock-unproven-owner`,
  and the holder's own commit succeeds.
- `gc-prune-during-commits`:
  `git gc --prune=now` while one commit is held in `pre-commit` and 3 others commit.
- `sigkill-pre-commit`,
  `sigkill-prepare-commit-msg`,
  `sigkill-commit-msg`,
  and `sigkill-post-commit`:
  `SIGKILL` to one agent's whole process group inside that hook while a bystander commits,
  then `git status` for recovery and a follow-up commit.
- `sigkill-offset`:
  the same with the kill at a seeded 20 to 600 ms offset instead of a hook.
- `sigkill-phase-preparation-done`,
  `sigkill-phase-landing-locked`,
  `sigkill-phase-objects-migrated`,
  `sigkill-phase-ref-updated`,
  and `sigkill-phase-index-installed`:
  the victim pauses at that landing phase through the test-only phase marker
  (see "Phase markers"),
  the bystander starts and gets 500 ms to reach its landing wait,
  then the victim's process group is killed,
  followed by `git status` for recovery and a follow-up commit.
  At `landing-locked` the victim holds the landing lock and the real `index.lock`,
  so the bystander must recover both from a dead owner.
  `victim-reached-phase` fails when the victim settled before its marker appeared.
- `replay-unavailable-fails-fast`,
  on 2.39.5 only:
  a commit held in its editor while another lands must exit `1`
  with exactly one `concurrent-commit/head-moved` core finding
  and land nothing.
- `replay-unavailable-disjoint-burst`,
  on 2.39.5 only:
  4 to 8 agents commit disjoint files;
  each lands or fails fast exactly that way,
  and at least one lands.

Hooks and editors are Node programs (`hook-program-fixture.ts`),
not shell scripts.
Each hook logs its run,
writes a `<token>.<event>` marker,
and blocks while a `<token>.<event>.hold` file exists,
which is how scenarios pin interleavings and inject kills.

### Hook kinds

Every scenario that needs a hook registers it as a config hook
(`hooks: 'config'` in `RepositoryOptions`, `repository-model-fixture.ts`),
which `repository-setup-fixture.ts` writes with
`git config hook.e2e-<event>.command "<hook program> <event>"` and
`git config --add hook.e2e-<event>.event <event>`.
The command runs the shared hook program of `hook-program-fixture.ts`,
so its logging, markers, holds, trailer and lint-staged mode are the same for every event.
Config hooks need Git 2.54.0,
so every hooked scenario declares `minimumGit: '2.54.0'` and skips on older Git.
The native task runs Git 2.55.0 only, so that skip does not occur there.

Traditional hooks (executable files in `.git/hooks` or a `core.hooksPath` directory)
are unsupported by the wrapper, and any use is undefined behavior,
so the suite installs none and `RepositoryOptions` cannot express them.
Two scenarios that existed only as the traditional-hook twin of a config-hook scenario,
`baseline-hooks-hookdir` and `hooks-hookdir-concurrent`, were removed;
`baseline-hooks-config` and `hooks-config-concurrent` run the same workload and assertions.
No other scenario's meaning depended on a traditional-hook behavior.

Fixture setup commits made with real Git (`seedTracePaths` and `commitAmendBase` in `scenario-setup-fixture.ts`)
pass `-c hook.e2e-<event>.enabled=false` for every event through `hookDisableArguments`,
so the repository's config hooks do not run for them.
The earlier `-c core.hooksPath=/dev/null` was removed:
it silences only traditional hooks and leaves config hooks running
(`doc/troubleshooting/git-hook-disable-switches.md`).

### Phase markers

Landing phases run no hook,
so the wrapper exposes them through a test-only environment variable
(`src/native/transaction_phase.rs`,
`SPEC.md` "Container end-to-end suite"):

```sh
# package/git-policy/cli/e2e: environment of one wrapper invocation
CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill                # SIGKILL itself at the phase
CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill:<directory>    # write <directory>/<phase>.reached first
CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:pause:<directory>   # write the marker, wait for <directory>/<phase>.release
```

The phases,
in transaction order,
are `capture-locked`
(capture lock held,
before the capture sequence number is allocated),
`preparation-done`
(after `prepared.json`),
`landing-locked`
(landing lock and real `index.lock` held,
`index-lock-<n>.json` written),
`objects-migrated`
(kept pack written,
before `landing-<n>.json`),
`ref-updated`
(after `ref-updated.json`),
and `index-installed`
(after the `index-installed` marker,
before conclusion cleanup and worktree completion),
and `race-lost`
(after each lost race and any reservation it earned,
before the replay,
outside both locks),
whose files carry the occurrence:
`race-lost-<n>.reached` and `race-lost-<n>.release`.
The variable name says it is for tests only,
nothing else sets it,
and a malformed value fails the invocation instead of being ignored.
A nested wrapper invocation from a hook inherits the variable,
so scenarios arm it only on commits whose hooks start no nested commit.
The kill scenarios use `pause` and kill the whole process group from the harness,
so the harness records the attempt as deliberately killed.

## Invariants

`invariant-fixture.ts` checks every run:

- `landed-once`:
  an attempt that exited `0` appears exactly once in local-branch history;
  a failed one never appears unless its `commit-landed` event names the commit;
  a killed one appears at most once.
- `landed-bytes`:
  each selected path holds the bytes captured at invocation,
  a clean three-way merge of them onto the landed parent from a candidate preparation base,
  or,
  for an explicit-path attempt,
  the landed parent's bytes when an attempt started later captured exactly those bytes for the same path
  and its commit lies between `HEAD` before invocation and the parent
  (capture order kept the later capture's landed bytes,
  `SPEC.md` "Capture order").
- `landed-scope` and `landed-branch`:
  a landed commit changes only its selected paths
  and is reachable from the branch `HEAD` named at invocation.
- `worktree-preserved`:
  every path the harness wrote still holds the harness's last bytes.
- `index-no-revert` and `index-matches-head`:
  the real index differs from `HEAD` only where the harness staged on purpose,
  and never stages content a landed commit replaced.
- `remote-contains`:
  the remote branch reaches every commit whose wrapper exited `0` with auto-push applying
  (not amends,
  not the foreign real-Git commit,
  and not commits landed on top of an amend of an already-published commit whose merge conflicted,
  which `push-rejection-surfaced` covers).
- `push-rejection-surfaced`:
  a commit that exited `0` on top of a landed amend of a commit the remote already had
  and that the remote does not contain must have surfaced Git's `non-fast-forward` push rejection and either cli-git's
  `auto-push to origin failed` note or its `cli-git: auto-push: not published:` line
  (owner decision,
  `doc/decision/cli-git-concurrent-commits.md` "Amending published history",
  narrowed on 2026-10-19 to merges that conflict).
- `no-leftovers`:
  no `refs/cli-git/` ref,
  lock file or lock directory
  (the capture lock included),
  transaction directory under `cli-git-transactions/`,
  legacy `cli-git-transaction` directory,
  shadow repository under `cli-git/shadow/`,
  landed-capture record under `cli-git-captures/landed/`,
  or other capture-store entry remains;
  the persistent `cli-git-transactions/` root
  and the capture store's `worktree-id`,
  `sequence`,
  and `landed/` directory are allowed
  (`SPEC.md` "Capture order").
- `fsck-clean`:
  `git fsck --strict --no-dangling` exits `0` with no output.
- `exit-events`:
  wrapper stderr JSONL decodes,
  sequences start at `0`,
  and exit codes agree with the events (`SPEC.md` "Stream and exit contract").
- `commit-msg-hook`,
  `post-commit-once`,
  and `signature-valid`:
  in hooked or signed scenarios,
  the landed message carries the hook trailer,
  `post-commit` ran once per successful attempt,
  and `git verify-commit` accepts the commit.
- Scenario expectations such as `all-commits-succeed`,
  `command-succeeds`,
  `staged-entries-kept`,
  `conflict-reported`,
  `amend-safe`,
  and `switch-invalidates-commit` encode the accepted design's outcome for that scenario.

## Seeds and replay

All workload decisions
(worker counts,
start offsets,
interleavings,
trace windows,
synthesized content,
kill offsets)
come from a generator derived from the run seed and the scenario name plus Git version,
so filtering never shifts another scenario's decisions.
Each result line prints a workload digest over labels,
selections,
and captured bytes.
Two consecutive runs of seed 1 printed identical digests for all 54 non-skipped runs,
and the two seed-1 runs of 2026-09-26 against `09b0eb07b` and `9b5f66c40`
printed identical digests for `concurrent-trace-replay`.
Hook barriers make the pinned interleavings identical on replay;
free-running starts and the `sigkill-offset` kill still depend on process scheduling,
so a replay reproduces the workload exactly but can land a free-running race differently.

## Results of the TypeScript wrapper build, 2026-09-26

These counts are historical:
they predate the switch from traditional hooks to config hooks,
which removed two scenarios and gave the other hooked scenarios a `minimumGit` of 2.54.0.
The first native run after the switch (seed 1369728287, Git 2.55.0) passed 34 scenarios
and skipped the two `replay-unavailable-*` scenarios, which run on Git 2.39.5 only.

Seeds 1,
2,
and 3,
2026-09-26,
packed from `feat/cli-git-concurrent-commits` at `b2b20ac74`
(after merging the worktree-copy stall fix and automatic maintenance after landing,
waiting for live trust provenance transactions,
failing fast without replay plumbing,
and lowering the `landing.reserveAfterLostRaces` default to 1),
114 scenario runs per seed:
38 scenarios on each of Git 2.39.5,
2.40.0,
and 2.55.0.
Every seed passed with no failure and no harness error.

### Passing

- Git 2.55.0:
  36 passes per seed,
  every scenario except the two `replay-unavailable-*` ones.
- Git 2.40.0:
  33 passes per seed.
  It skips the config-hook scenarios and `foreign-index-lock-holder`,
  which need Git 2.54.0,
  and the `replay-unavailable-*` scenarios.
- Git 2.39.5:
  21 passes per seed:
  the baselines except `baseline-hooks-config`,
  `branch-switch-during-commit`,
  the hook-phase `sigkill-*` and all `sigkill-phase-*` scenarios,
  and both `replay-unavailable-*` scenarios.
  It skips the 17 scenarios that need replay plumbing or Git 2.54.0.

Before replay degradation existed,
seed 1 on Git 2.39.5 failed 12 of 36 runs:
every commit that lost a landing race exited `2` with `transaction-failed`
carrying `git merge-tree` usage text.
No invariant was violated then either:
nothing landed,
and nothing was lost.
With the probe disabled,
both `replay-unavailable-*` scenarios fail again
(`held-fails-fast-with-head-moved` and `landed-or-failed-fast`),
so they detect the regression.

### Fixed on the way

- The replay conflicts `concurrent-trace-replay` reported under subsumption alone
  (a commit's own edit rewrote lines another in-flight commit had just added,
  for example `trace/p1538` on seed 1 with 2.55.0 and `trace/p627` on seed 2 with 2.40.0)
  are decided by capture order:
  the later capture's bytes land
  (`SPEC.md` "Capture order").
- Seed 2 on 2.55.0 failed `t7` with
  `fatal: pathspec 'trace/p5500' did not match any files`,
  because `t7` deleted a path another in-flight commit was adding.
  The harness now waits for the landing of an in-flight commit that creates a path before another commit removes it
  (see "Commit-shape trace").
- A first seed-3 run failed `concurrent-trace-replay` on 2.40.0 with
  `landed-bytes [t9 d9059a229200 trace/p1015]`:
  `t9` kept the landed bytes of a path a later capture had landed first,
  which capture order prescribes and the checker did not accept yet.
  `landed-bytes` now accepts exactly that case
  (see "Invariants").
  The rerun's free-running interleaving did not reach it again.
- The same first seed-3 run failed `sigkill-phase-landing-locked` on 2.55.0:
  the bystander exited `2` with `transaction-failed` `ESRCH: no such process, read`,
  because reading `/proc/<pid>/stat` of the victim while it exited failed with `ESRCH`
  and the owner-liveness check treated only `ENOENT` as exited.
  It now treats `ESRCH` as exited too.

## Open problems

- The shared-file scenarios give the second agent a fixed 1.5 s start window,
  because the accepted design has no hook-visible point between capture and the hook lock.
  A heavily loaded host could release the first agent before the second captured.
- The `race-lost` marker sits between a lost race and its replay,
  but no scenario kills a transaction there yet.
- The lint-staged scenarios run in a linked worktree,
  because the `linked-worktree-only` core policy rejects the hook's `git stash` in a main worktree.
  lint-staged in a main worktree is therefore blocked by cli-git today,
  independent of concurrency.
- The container consumer writes its report with `console`:
  the tagged logger is a workspace package with its own dependencies and is not installed in the container.
- `remote-contains` is checked only after the run,
  not at each wrapper exit.
- The trace has few binary changes (12 of 12,984) and renames (42),
  so windows often contain none;
  the synthetic scenarios do not add binary or rename cases of their own.
