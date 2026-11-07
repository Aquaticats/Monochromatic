# Concurrent-commit container suite

End-to-end workload replay for monochromatic-git-policy-cli concurrent commits.
It is the "Container end-to-end verification" part of
[`doc/decision/cli-git-concurrent-commits.md`](../../../../doc/decision/cli-git-concurrent-commits.md)
and slice 8 of
[`doc/concurrent-commits-implementation-plan.md`](../doc/concurrent-commits-implementation-plan.md).
Fresh dummy repositories replay realistic concurrent-agent workloads through the native `git` wrapper (`src/native/`),
and an invariant checker judges every run.

## How to run

```sh
# package/git-policy/cli: run from the repository root
mise run //package/git-policy/cli:test:e2e:concurrent:native
mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1
mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1 --scenario shared-file-overlapping-hunks
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
  `--git` takes comma-separated Git releases, and the image provides its installed Git's release only (the report lines
  name it).
- The native executable is tested with the image's current Git and prints a warning on an older one without refusing
  it (owner decision, 2026-11-04: "Raise minimum, drop the test, merely print a warning on older git and not refuse."),
  so the image no longer builds a second, oldest Git.
  On 2026-10-14 with seed 1, 36 scenarios passed and 2 skipped.

The TypeScript wrapper that this suite first verified is retired;
 its last commit is
`f4fc1fa213ece65672e4314983a75266837892f9`,
 and the sections that describe measurements say so where they cite it.

- `E2E_WRAPPER` (empty or `native`,
   the only wrapper) names the directory whose `git` shadows the real Git
  (`wrapper-selection-fixture.ts`);
  the consumer checks that this `git` is an ELF executable and prints the result first.
- `test:e2e:concurrent:native:image` compiles the release `monochromatic-git-policy-cli` in the gate's base image
  (CachyOS with Rust, `native-test-base.Containerfile`)
  and links it as `/opt/native/bin/git` in a copy of the `cli-git-concurrent-e2e` base image,
  which starts from the same CachyOS image.
  `GIT_POLICY_NATIVE_IMAGE_TAG` names the native image's tag, so concurrent worktrees never share one.
- `test:e2e:concurrent:image` builds the base image alone (CachyOS, Node,
   the Git build,
   `ssh-keygen`).
  It pulls the CachyOS `latest` tag and names the image after that pull and the Containerfile,
   so a new CachyOS image builds a new base and a trial build never replaces the image other runs use.

Regenerate the committed trace with real Git on the host.
It mines this checkout by default;
`--repository <path>` selects another checkout of the same history,
and `--limit` and `--revision` adjust the window.

```sh
# package/git-policy/cli: bypasses the wrapper through /usr/bin/git
mise run //package/git-policy/cli:e2e:concurrent:trace
```

## Commit service

Every commit the scenarios make lands through the resident commit service, the wrapper's own `/opt/native/monochromatic-git-policy-cli`
started with the words `monochromatic-git-policy-cli service run` (`SERVICE_MODE_WORDS` in `service-launch-fixture.ts`).
Each scenario starts its own service (`service-fixture.ts`) with its own runtime directory (`XDG_RUNTIME_DIR`)
and journal under `<scenario root>/service/`,
and stops it when the scenario ends,
so a scenario can freeze, kill and restart its service without touching another scenario.
The container has no service manager,
so `restart` and `restartAfterExit` play the manager:
they remove the stale socket file (the service does not replace one) and start a clean process on the same journal.
The scenario's wrapper environment names the runtime directory.

A commit the service landed returns before its push.
The wrapper writes a `push-<oid>` and a `maintenance-<oid>` record under `<common git dir>/monochromatic-git-policy-cli/work/pending`,
and the service runs them on its own thread and moves them to `done` or `failed`.
`settleServiceWork` therefore waits until no record is pending or claimed
before the observation compares the remote with the local branch,
and the checker control waits before it tampers with the landed commit.
Records that are still pending or claimed after the wait are reported as the `after-commit-work-settles` invariant.

### Instruments

The wrapper runs no external hook program and no program a configuration names
(`doc/planning/cli-git-no-self-invocation-dispatcher.md`),
so hooks are no longer the suite's instrument for pausing, killing or observing a commit.
The instruments that replace them add nothing to the release executables:

- The message editor (`SCRIPTED_EDITOR_PROGRAM` in `hook-program-fixture.ts`, selected with `-e` and `GIT_EDITOR`).
  The wrapper runs real Git in a scratch repository while it prepares a commit that asks for an editor,
  before anything is sent to the service,
  so the editor marks and holds two points of preparation, `editor-open` and `editor-written`,
  and can append a trailer to the message.
  A commit built with an editor is never sent for another tip:
  when the branch moved while the editor was open, the commit fails with the saved-message notice,
  so no scenario races a landing against a held editor.
- A frozen service (`SIGSTOP`, `scenario-queue-fixture.ts`).
  A client prepares before it connects, so an open socket in the wrapper process (`client-connection-fixture.ts`)
  proves that its preparation ended and its commit waits at the service.
  Starting two agents under a freeze pins "both prepared before either landed";
  resuming lets them land in the order they connected.
  A freeze must stay under the client's ten-second start wait.
- The branch moving (`landing-kill-fixture.ts`).
  The harness reads the loose reference file of the branch and freezes the service the moment it changes,
  so the agent is still alive in its exchange with the service when it is killed.
- The work records (`work-record-fixture.ts`).
  The `push-<oid>` record is the durable trace of the after-commit work that the `post-commit` hook once observed:
  one settled record per landed commit stands for "post-commit ran once".
- External `SIGKILL` of the client's process group, or of the service, at a seeded offset or at an observable instant.
- The fault points of the release service (`CLI_GIT_TEST_ONLY_SERVICE_FAULT`, `src/native/service/service_fault.rs`):
  `after-objects`, `after-intent`, `after-ref`, `after-index` and `before-ack`.
  The variable is armed only by the harness, and only on the process a scenario starts for it.

A declared `postCommit` action in `monochromatic-git-policy-cli.config.jsonc` is not an instrument:
that facility was built and deleted on the owner's instruction, and the key is a configuration error
(`src/native/config/config_parse.rs`, `SPEC.md` "External hooks").

### Scenarios whose instrument no longer runs

`baseline-lint-staged` and `lint-staged-stash-concurrent` were written to drive lint-staged's backup-stash and
hide-unstaged sequence from a `pre-commit` config hook.
The wrapper runs no external hook,
so that sequence never runs and the two scenarios now check a partially staged index commit in a linked worktree
(`baseline-lint-staged`) and four concurrent commits beside one such index commit (`lint-staged-stash-concurrent`),
nothing about a stash.
They are kept because those commit shapes are live behavior,
and renaming them would break the seeds that name them;
a stash sequence run directly beside a commit is not covered by this suite.
`gc-prune-during-commits` had the same fault and was ported:
its commits now wait prepared at a frozen service while `git gc --prune=now` runs.

### Retired scenarios

- `reservation-after-lost-races` is retired:
  it asserted the landing reservation and the `race-lost` marker of the transaction route, which the service replaces.
  The property it guarded, that a commit which keeps losing races is not starved, holds by the queue's arrival order and the
  bound on prepared-again answers (`doc/planning/cli-git-resident-service.md`, "Service-side fixes of 0.5.6").
- `sigkill-prepare-commit-msg` is retired:
  between the `pre-commit` and `commit-msg` hooks it marked a point that the editor alone marks now,
  and `sigkill-editor-open` already kills there.
- The five `sigkill-phase-*` scenarios named transaction phases that the service path never reaches
  (the capture lock, the landing lock, the kept pack, the transaction journal).
  Their properties (no lost commit, no stray state, a consistent index, a pushed remote) are held by
  the `sigkill-service-*` scenarios and by `sigkill-queued`.

## Container

`concurrent-commits.Containerfile` builds `cli-git-concurrent-e2e`:

- Node, from the distribution's `nodejs` package (owner decision, 2026-11-04: "We don't need to pin Node too.");
- Git, from the distribution's `git` package at `/usr/bin/git` (owner decision, 2026-11-04: "We don't need to pin git
  because my current system is already using git latest."), whatever release that is; the suite reads it at run time;
- `ssh-keygen`, from `openssh`.

The wrapper's directory comes before `/usr/bin` on each scenario's `PATH`, so the wrapper finds the packaged Git behind
itself.
Pacman checks package signatures (`SigLevel = Required DatabaseOptional`).

The native image adds `/opt/native/bin/git`,
 a link to the release executable,
 on top of this image.

The native executable prints a warning on a Git older than the release it was tested with and runs as usual
(`SPEC.md` "Compatibility and degradation"),
so the image carries one Git, the distribution's current one.
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
  the `sigkill-*` scenarios,
  and the `sigkill-service-*` scenarios;
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
A worker that touches a path another in-flight worker selected waits until that worker's wrapper holds a socket to the service
(`client-connection-fixture.ts`; preparation, which captures the bytes, is over by then),
so every attempt commits exactly the bytes the harness wrote for it.
When the service sends a commit back because a path it names changed on the branch,
the wrapper merges the captured bytes onto the branch with the preparation base as merge base
and never reads the worktree again,
so a later worker's bytes cannot land under an earlier worker's message.
Such an attempt exits `0`, exits `1` with the `concurrent-commit/replay-conflict` finding naming its path,
exits `1` with the `concurrent-commit/already-committed` finding when commits that landed meanwhile already hold
everything it changes,
or exits `1` with the plain "changed N times while this commit was being prepared" message when the service's bounded
retries ran out and nothing landed (`landedOrConflicted`,
`scenario-model-fixture.ts`).
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
  two sequential commits in a repository that configures all four commit hooks.
  The commits land, each warns `external-hooks-unsupported`, the hook program never runs,
  and the service settles one push record per landed commit.
- `baseline-editor-trailer`:
  two sequential commits whose message editor appends a trailer;
  the landed messages carry it and each settles one push record.
- `baseline-ssh-signing`:
  sequential SSH-signed commits.
- `baseline-sigkill-after-landing`:
  `SIGKILL` once the commit landed and its push record exists, timed as well as polling allows,
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
  the first agent edits its hunk and is queued at a frozen service,
  the second edits the same file and is queued behind it,
  and the service resumes.
  `agents-queued-in-order` fails when either agent settled before it connected.
  Far-apart hunks must both land;
  overlapping hunks may fail only with exit `1` and the `concurrent-commit/replay-conflict` `core-finding` event
  that names the path.
  The first lands, and the second, which prepared against the branch before that,
  has the bytes it captured merged onto the branch (`src/native/landing/service_replay.rs`);
  a write the harness makes after the second command started never appears in its commit.
- `shared-file-adjacent-edits`:
  the first agent edits one line and is queued at a frozen service,
  the second edits the line after it, the line before it, or inserts a line right after it (seeded)
  and is queued behind it,
  so its captured bytes hold both edits.
  Both must land and the tip must hold the bytes the second captured.
- `interleaved-index-writers`:
  4 commits and 4 `git add` runs of new files in a seeded interleaving;
  the adds run one after another,
  because two adds that overlap meet each other's `index.lock` and native Git refuses
  the second with exit 128 (the wrapper no longer serializes them:
  a forwarded index writer competes only on Git's own
  `index.lock`,
  `doc/planning/cli-git-resident-service.md`,
  "Step 4d" deletions);
  every add must succeed,
  including while the service holds the index for a landing,
  and its staged entry must survive.
- `hooks-config-concurrent`:
  4 agents commit at once in a repository that configures all four commit hooks;
  none runs, each commit warns, and each settles one push record.
- `ssh-signing-concurrent`:
  4 agents sign their commits at once.
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
- `branch-switch-during-commit`:
  a commit is held in its message editor while the wrapper refuses a switch (tracked files are modified)
  and real Git then switches to another branch.
  The commit must not land on the branch switched to,
  and the checker judges the index and the remote after the held commit finishes.
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
  `git gc --prune=now` while one commit and 3 others are prepared and queued at a frozen service,
  then the service resumes.
  Each commit lands or ends with exit `1` and the plain message that its content was removed by the prune
  (`landed-or-reported-pruned`;
  `SPEC.md` "Private preparation" records the refusal after a second loss);
  a commit that landed without its content fails `fsck-clean` or `landed-bytes`.
  Every seed run so far ended with all four commits refused,
  so the landing branch of the expectation has not fired.
- `sigkill-editor-open` and `sigkill-editor-written`:
  `SIGKILL` to one agent's process group while its editor is open, or after it wrote the final message,
  while a bystander commits, then `git status` and a follow-up commit.
  Nothing was sent to the service, so the victim's commit must not appear.
- `sigkill-queued`:
  the victim's prepared commit and the bystander's wait at a frozen service,
  the victim is killed, and the service resumes.
  The victim's commit must not appear and the bystander's must land.
- `sigkill-after-landing`:
  the service is frozen the moment the branch moves, so the agent is still in its exchange with it,
  and the agent is killed there.
  The commit landed and must appear exactly once.
- `sigkill-offset`:
  the same with the kill at a seeded 20 to 600 ms offset;
  a kill that comes after the agent exited kills nothing.
- `sigkill-service-after-objects`,
  `sigkill-service-after-intent`,
  `sigkill-service-after-ref`,
  `sigkill-service-after-index`
  and `sigkill-service-before-ack`:
  the service ends itself at that point of the first batch while two agents commit;
  the harness restarts it and the clients retry with their request identity.
  Both commits must land exactly once, the index and worktree must agree with the branch,
  and `service-fault-fired` fails when the service never reached the point.
- `sigkill-service-offset`:
  six agents are queued at a frozen service, the service resumes, and the harness kills it
  0 to 5 ms later, inside the batch, then restarts it.
  `service-killed-with-agents-waiting` fails when every agent had its answer before the kill.
- `sigkill-service-during-work`:
  the service is killed once it claimed the landed commit's push record, then restarted, and a follow-up commit is made.
  The dead service's claims stay out of reach until their lease (`CLAIM_LEASE`, 120 s,
  `src/native/landing/service_work.rs`) runs out, because a service that is still running renews its claims
  and a claim's holder is never told by process ID.
  The restarted service looks at the repository again when the lease is over (`work_loop`,
  `src/native/service/service_server.rs`), requeues the claims and runs them.
  The scenario therefore waits the lease plus a margin (`CLAIM_RECOVERY_BUDGET_MS`, counted from the kill)
  for the records to settle,
  and `killed-service-claims-recovered-within-lease` fails when they are not settled by then,
  however late the wait observes it.
  The generic 30 s settle wait of the observation is shorter than the lease and would report the claims as unfinished.
  `service-kill-lease.unit.test.ts` reads the lease from the Rust source and fails when the fixture's value drifts.
  The scenario runs just over the lease, 121 s on 2026-11-04 (seed 1).
  Execution evidence, not remote state:
  `service-killed-holding-the-push-claim` requires the push record of the landed commit to be in `claimed`
  right after the kill (the upkeep record may still be pending, because the service claims its items one after another),
  and `claimed-work-settled-once-by-the-successor` requires each item to be a single file in `done`,
  and for each item that was claimed at the kill a `tries` of one or more.
  The service writes `tries` into an item each time it takes the item back from a holder that did not renew it,
  so the count shows that the dead service's claim was recovered and run by the successor.
  Neither the empty work directories nor a remote that holds the commit show that,
  because the push of a later commit carries its ancestors.
  What no record can show is a second run of the same side effect:
  a holder killed after it pushed and before it settled is run again by design,
  and a push that was made twice leaves the same remote state.
- `sigkill-service-then-idle`:
  the same kill and restart with nobody committing afterwards:
  no wrapper command runs and no wake reaches the restarted service,
  so the claims can be run only by the service's own look at the repositories it was woken for
  (`src/native/landing/service_claim_watch.rs`).
  It answers the objection that the recovery of `sigkill-service-during-work` might depend on the follow-up commit's wake,
  and it fails with the claims left in `claimed` on a service without that look (2026-11-04).
  It runs just over the lease, like the scenario above, and both stay under the 180 s limit of a scenario's workload.
  That limit bounds the workload (`run`) only: the observation that follows it, with its own 30 s settle wait, is outside it,
  so a failing run of these scenarios takes the budget plus that wait, about 180 s (measured 180585 ms on 2026-11-04).
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

Only `baseline-hooks-config`,
`hooks-config-concurrent`,
`baseline-lint-staged` and `lint-staged-stash-concurrent`
register a config hook
(`hooks: 'config'` in `RepositoryOptions`, `repository-model-fixture.ts`),
which `repository-setup-fixture.ts` writes with
`git config hook.e2e-<event>.command "<hook program> <event>"` and
`git config --add hook.e2e-<event>.event <event>`.
The wrapper runs no external hook,
so the hook program is a canary:
the first two scenarios assert that it never ran (`hooks-ignored-fixture.ts`).
The canary has no positive control inside the suite, because no wrapper run can make a hook fill its log.
The lint-staged pair still drives its instrument (a lint-staged sequence) through a hook that no longer runs,
so what they check is not what they were written for ("Scenarios whose instrument no longer runs").
Every other scenario installs no hook and uses the instruments of "Commit service".

Traditional hooks (executable files in `.git/hooks` or a `core.hooksPath` directory)
are unsupported by the wrapper, and any use is undefined behavior,
so the suite installs none and `RepositoryOptions` cannot express them.

Fixture setup commits made with real Git (`seedTracePaths` and `commitAmendBase` in `scenario-setup-fixture.ts`)
pass `-c hook.e2e-<event>.enabled=false` for every event through `hookDisableArguments`,
so the repository's config hooks do not run for them.
The earlier `-c core.hooksPath=/dev/null` was removed:
it silences only traditional hooks and leaves config hooks running
(`doc/troubleshooting/git-hook-disable-switches.md`).

### Phase markers

No scenario arms a phase marker.
The wrapper's `CLI_GIT_TEST_ONLY_PHASE_SIGNAL` (`src/native/transaction/transaction_phase.rs`) names only the phases of an isolated
merge and of a commit headed for the service,
which the binary tests of the crate use;
the phases of the retired landing are gone.
The only instrument this suite arms inside the release executables is the service's fault point
(`CLI_GIT_TEST_ONLY_SERVICE_FAULT`,
see "Commit service").

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
  and that the remote does not contain must have surfaced Git's `non-fast-forward` push rejection and either monochromatic-git-policy-cli's
  `auto-push to origin failed` note or its `monochromatic-git-policy-cli: auto-push: not published:` line,
  on its own stderr or, because the service pushes after the commit returns, on the stderr of a later wrapped command
  in the same repository
  (the observation runs one `git status --short` when the service left a notice, so the notice reaches a stderr)
  (owner decision,
  `doc/decision/cli-git-concurrent-commits.md` "Amending published history",
  narrowed on 2026-10-19 to merges that conflict).
- `no-leftovers`:
  no `refs/cli-git/` ref,
  lock file or lock directory
  (the capture lock included),
  client scratch directory `monochromatic-git-policy-cli-service-<id>`,
  transaction directory under `cli-git-transactions/`,
  legacy `cli-git-transaction` directory,
  shadow repository under `monochromatic-git-policy-cli/shadow/`,
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
- `editor-trailer`,
  `after-commit-work-once`,
  and `signature-valid`:
  in the scenarios that ask for them,
  the landed message carries the trailer the message editor appended,
  the service settled exactly one `push-<oid>` record for every commit whose wrapper exited `0`,
  and `git verify-commit` accepts the commit.
- `after-commit-work-settles`:
  no `push` or `maintenance` record is still pending or claimed when the wait for the service ends.
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
Editor holds and frozen-service queues make the pinned interleavings identical on replay;
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

- A frozen service has to be resumed within the client's ten-second start wait,
  so a scenario that freezes it must reach its resume without long waits in between.
- `sigkill-offset` kills at a seeded time, and a kill that comes after the agent exited kills nothing;
  `sigkill-after-landing` and `sigkill-queued` are the exact kills.
- A claim that a killed service held delays its backup push by up to the lease, 120 s after the kill,
  because the lease is the only proof that a holder died;
  commits themselves never wait for it.
  A shorter lease would need a harness-only lease control, and the release executable has none.
- The container gate (`native:test:container`) runs nextest and clippy, not this suite,
  so these scenarios are run on their own with `test:e2e:concurrent:native --scenario <name>`.
- A kill while the service pushes is pinned only by `sigkill-service-during-work`, which waits for the claim;
  the push to the local remote is too short to hold, so that kill can land after the push finished.
- The lint-staged scenarios run in a linked worktree,
  because the `linked-worktree-only` core policy rejects the hook's `git stash` in a main worktree.
  lint-staged in a main worktree is therefore blocked by monochromatic-git-policy-cli today,
  independent of concurrency.
- The container consumer writes its report with `console`:
  the tagged logger is a workspace package with its own dependencies and is not installed in the container.
- `remote-contains` is checked only after the run,
  not at each wrapper exit.
- The trace has few binary changes (12 of 12,984) and renames (42),
  so windows often contain none;
  the synthetic scenarios do not add binary or rename cases of their own.
