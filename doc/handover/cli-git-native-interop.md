# cli-git native interoperability matrix

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

Evidence for the trailing gate "old-and-new wrapper interop matrix" of
`doc/handover/cli-git-rust-implementation.md`.
Paths starting with `src/`, `bin/` or `native-interop/` are relative to `package/git-policy/cli/`;
every other path is repository-relative.

Terms:
the incumbent is the retired TypeScript wrapper, built from the pinned last revision
`f4fc1fa213ece65672e4314983a75266837892f9`;
native is the release `cli-git-native` (crate version 0.1.2) built from this branch's checkout.

## Decision and end state

The user decided that the matrix runs once, its results are recorded here, and everything that can build or run the
incumbent is then deleted, so that the old wrapper cannot be found or used.
The branch therefore ends in this order:
the provisioning step and the drivers (the evidence), this document (the results),
and then one commit per deletion.

What stays after the deletions:

- `native-interop/old-format-fixtures.mjs` and `native-interop/fixture/`:
  records the incumbent wrote, kept as plain files, which the native wrapper must read.
- `bin/interop-native-container.mjs` and `mise run //package/git-policy/cli:native:interop`,
  reduced to an image with real Git 2.56.0, Node and the release native wrapper.

What is deleted and where it can be read:
the incumbent builder `bin/provision-incumbent.mjs`, the incumbent steps of the image runner, and every driver that
starts the incumbent (`smoke`, `owner-locks`, `phase-recovery`, `hook-dispatch`, `capture-differential`,
`crash-matrix`, `lock-waits`, `mixed-concurrency`, `capture-fixtures`).
All of them exist at commit `5d61d7b442`, the last commit before the deletions (the final native fixes are in it).
Rerunning anything below starts from that commit (see `Rerun commands`).

The deletions, one commit each, in order:

- `b003b8b224`: `bin/provision-incumbent.mjs`, the incumbent steps of `bin/interop-native-container.mjs` and the
  `native:interop:provision` task.
- `c603e53eb9`: the nine drivers that start the incumbent.
- `3a5b620529`: the incumbent and probe helpers of `native-interop/support.mjs`, leaving the native-only ones.
- `788450edac`: the two ignored `interop_probe` library tests and the image step that compiled them.
- `b3f8ec89b1`: the 30 JavaScript files of the retired wrapper's hook dispatcher (`dispatch.mjs`, its `package.json`
  and three hook shims in each of six transaction fixtures), which a fixture would have run if executed.
  The native reader needs only `plan.json` there, and the fixture reader passed without them.

Outside the repository: the runnable incumbent copy under `target/verification/` was removed,
the reduced image was rebuilt (below), and 54 untagged container images that held `/opt/cli-git` were removed.
One untagged image that holds it (`b9fb34691f4f`) is a parent layer of a tagged image and was kept.
Three tagged images from other lanes still hold the incumbent:
`localhost/cli-git-concurrent-e2e:latest`, `localhost/git-policy-native-interop:development` and `:transactions`.
They belong to other lanes' evidence and were not deleted.
Besides git history, no file in the repository can build or run the incumbent;
`rg` for `opt/cli-git`, `INCUMBENT_BIN`, `provision-incumbent` and `interop:provision` over `package/git-policy/cli`
and `doc` finds only the historical records of these documents.

The reduced image (`localhost/git-policy-native-interop:interop`, id `924148b30506`, no `/opt/cli-git`) ran
`old-format-fixtures` after the last deletion: 41 checks passed, 0 failed.
The container gate on the tree after the probe deletion passed (1666 unit tests with 3 ignored, 118 binary-level
tests with 1 ignored, Clippy inside the gate).

## Incumbent provisioning

`bin/provision-incumbent.mjs` rebuilt the incumbent on demand, the way it was built before the retirement:

1. `git worktree add --detach` of the pinned revision under `${HOME}/temp/agent`.
2. `pnpm install --frozen-lockfile --prefer-offline --filter '@monochromatic-dev/git-policy-cli...'`.
3. The pinned revision's own `pack:npm` task (the rolldown bundle `dist/final/node/index.mjs`, then `pnpm pack`).
4. `npm install --ignore-scripts` of that tarball into a staging directory, the layout the end-to-end image once
   carried as `/opt/cli-git`.
5. The worktree is removed with its administrative entry, whether provisioning passed or failed.

The image runner copied the staging directory to `/opt/cli-git`.
The provisioned incumbent for this run:
tarball SHA-256 `425b7c1f2771dafb7f5bf41e9be40c3efb12ce39c04b6ad9884ce1e7e1c03e49`,
`dist/final/node/index.mjs` SHA-256 `60d98632f603ef811aa613803b189fc3aba37e0207e7cefd1494cf91fb05cb3e`,
built with Node 26.11.1, pnpm 12.11.2 and npm 11.20.0 on the host and run under Node 24.21.0 in the image.
No TypeScript entered the repository; the build output sat under the ignored `target/verification/`.

## Environment

Image `localhost/git-policy-native-interop:interop`: the gate's Git 2.56.0 base, Node 24.21.0,
`/opt/cli-git/node_modules/.bin/git` (the incumbent), `/opt/native/bin/git` (the release native wrapper),
run as user 1000 with `--rm --init --network=none --memory=2g --cpus=2`.
Each driver started both wrappers through `PATH` order inside disposable repositories under `/work/scratch`.
The container gate of the final native code (image `e69138b9973c`) passed:
1666 unit tests (5 ignored), 118 binary-level tests (1 ignored), the candidate consumer, and Clippy inside the gate;
`mise run //package/git-policy/cli:native:clippy` is clean too.
Mutation testing of `recovery_files.rs`, `transaction_registry.rs` and `recovery_scan.rs` was not run.
The matrix drivers ran on the final native code (commit `5d61d7b442`)
with the results in the Results per combination section.

## Results per combination

Counts are checks that passed and failed in the final run.
A driver prints one `ok:` or `FAIL:` line per check and keeps going after a failure, so no combination hides another.

Crash recovery across wrappers (`crash-matrix.mjs`, 580 passed, 0 failed):
the writer commits and kills itself at each of `capture-locked`, `preparation-done`, `landing-locked`,
`objects-migrated`, `ref-updated` and `index-installed`;
the reader then recovers through a read-only command in one repository and through a commit of its own in another.
All four writer and reader pairings pass:
incumbent to native, native to incumbent, and the two same-wrapper controls.
After each recovery the dead commit landed whole (from `ref-updated` on) or not at all,
the real index matched `HEAD`, the worktree kept the edit, `git fsck` was clean, and no transaction directory,
shadow repository, `index.lock`, `.keep` or landed-capture record remained.
Both wrappers then landed follow-up commits, and no stale lock directory remained after them.
A lock directory of a dead holder stays after a command that takes no lock (`status`, an `add` that needs no landing)
in both wrappers alike; the next lock-taking command retires it, so the driver allows it only for such commands.

Lock waits with real commits on both sides (`lock-waits.mjs`, 84 passed, 0 failed):
a holder paused at `capture-locked` or `landing-locked` makes a commit, and for `landing-locked` an `add`, of the other
wrapper wait for the whole blocked window and then finish after the release, with both commits on the branch and the
staged path installed.
A dead holder killed at either phase does not block an `add` of the other wrapper.
All four pairings pass.

Incumbent holds, native recovers (`phase-recovery.mjs`, 80 checks, passed) and the lock and identity probes
(`owner-locks.mjs`, 15 checks, passed): the incumbent's capture lock identity equals the native identity byte for byte,
the native acquirer finds a live incumbent lock busy and retires a dead one,
and the incumbent's dispatcher waits for and retires a native hook lock.

Hook dispatch and the hook lock (`hook-dispatch.mjs`, 13 checks, passed):
each wrapper's dispatcher blocks the other's until release, hook runs serialize in order,
the incumbent nested in a native dispatcher and the reverse accept the preparation lease,
and a tampered lease is refused in both directions.

Old-format fixtures (`old-format-fixtures.mjs`, 41 passed, 0 failed, native only, see `Old-format fixtures`).

Mixed concurrency (`mixed-concurrency.mjs`, seeds 1 to 3):
three scenarios per seed, with the wrappers assigned to workers in a seeded order.

- `disjoint` (ten workers: eight committing their own file twice, two running `add` and `status`):
  passed for every seed and every wrapper assignment.
- `hooked` (six workers through a shared `pre-commit` hook): hook runs never overlapped, in any seed.
  Replays after a lost landing race run the hook again, so there are more hook runs than commits.
- `killed` (six workers, one killed at a seeded phase): passes once per seed, but fails on repetition in the mixed
  assignment, see `Open findings`.

Totals with seeds 1 to 3 on the final native code:
mixed 189 passed and 0 failed, native only 189 passed and 0 failed, incumbent only 189 passed and 0 failed.
Earlier runs of the same seeds failed in the mixed set (187 passed and 2 failed, then 186 and 3), before the last three
native fixes below; the failures vary from run to run.

Repeated `killed` runs with the victim killed at `landing-locked` (seed 2) before the last three native fixes,
60 repetitions each:
native only 1037 passed and 0 failed;
incumbent only 1037 passed and 0 failed;
mixed 998 passed and 39 failed, with a survivor failing in 19 of the 60 repetitions
(19 incumbent survivor exits and 1 native survivor exit).
A second batch of 40 mixed repetitions with debug output on had a survivor fail in 22, and a native survivor exited 2
in 3 of those.
Seeds 4 to 25 for the `killed` scenario: mixed 374 passed and 0 failed, native only (seeds 4 to 20) 289 and 0,
incumbent only 369 passed and 5 failed (two incumbent exceptions, listed under the incumbent-only findings).

Not run: `capture-differential.mjs` after the retirement (it prints the incumbent's trees for comparison with
native unit tests that already hold those trees), Git 2.55.0, macOS and Windows,
and any run longer than the repetitions recorded per combination.

## Defects in the native wrapper, fixed

All five were found by the `killed` scenario of `mixed-concurrency.mjs`, each fixed with a control written first
(each control was run against the unfixed code and failed with the message the scenario showed).
They are races between recoverers of one dead transaction.

- Retiring a transaction directory failed when another recoverer had already retired it:
  a native survivor exited 2 with `retiring the transaction directory ... No such file or directory`.
  `remove_transaction_directory` in `src/native/transaction_registry.rs` now treats a missing directory as retired.
  Control: `removing_a_directory_a_peer_already_retired_succeeds`.
  The earlier assertion that a missing registry directory cannot be retired was removed because it encoded the defect.
  Commit `fa671c2bd3`.
- Releasing an index lock failed when another recoverer released it between the existence probe and the identity lookup:
  exit 2 with `the landing reservation failed: ... unable to resolve the filesystem identity for .../index.lock`.
  `release_owned_lock` in `src/native/recovery_files.rs` now reports a lock that is gone as absent,
  also when the removal itself finds it gone; a lock that still exists but cannot be identified stays an error.
  Controls: `a_lock_released_by_a_peer_mid_release_is_absent` (two races, through a replaceable identity mechanism)
  and `an_unidentifiable_lock_is_still_an_error`.
  Commits `e2606bca07` and `89f7e54313` (Clippy findings in the controls).
- Startup recovery took the landing lock only for a dead transaction with a landing record.
  A transaction killed at `landing-locked` or `objects-migrated` holds the real `index.lock` and has only an
  `index-lock-<n>.json` record, so it was recovered without the landing lock: a slow recoverer could then release, by
  device and inode, an `index.lock` that a lander had created after another recoverer released the dead one.
  `needs_landing_lock` in `src/native/recovery_scan.rs` now uses `entered_landing` (an index-lock record or a landing
  record), the predicate the landing pass already used and documented as the only recovery that may release such a lock.
  `SPEC.md` section `Recovery` states the same.
  Control: `a_dead_transaction_holding_only_its_index_lock_waits_for_the_landing_lock`
  (recovery waits while a live lander holds the lock).
  Commit `9e101f702b`.
- A transaction that another recoverer removed after this one listed the registry made the landing-lock check fail with
  `listing <transaction> failed: No such file or directory` (exit 2).
  `entered_landing_unless_vanished` in `src/native/recovery_scan.rs` answers that such a transaction needs no lock.
  Control: `a_transaction_a_peer_removed_after_inspection_needs_no_landing_lock`.
  Commit `4d5292bbad`.
- The same removal after the owner check made `recover_dead_transaction` fail on its own listing
  (`code content-unavailable`, seen in a native-only run).
  `recover_dead_transaction_unless_vanished` reports such a transaction as vanished, as the incumbent does for an
  owner record that disappeared; a failure with the directory still present stays a failure.
  Control: `a_dead_transaction_a_peer_finished_after_inspection_is_vanished`.
  Commit `5d61d7b442`.

After these fixes the native-only `killed` scenario (seed 2, victim at `landing-locked`) passed 200 of 200 repetitions
(3417 checks, 0 failed).
Before them the same set failed 2 checks in one of 100 repetitions (the last defect above).

## Open findings

The mixed `killed` scenario still fails after the native fixes, and the cause is on the incumbent's side.
The incumbent's startup recovery keys its landing-lock requirement on a landing record alone
(`needsLandingLock` and `hasLandingRecord` in the pinned `commit-transaction-recovery-scan.ts` and
`commit-transaction-recovery-evidence.ts`), although its own `enteredLanding` predicate and the doc comment of
`recoverDeadLandings` say that an owner killed holding the landing lock and `index.lock` has no landing record yet.
So an incumbent survivor recovers a native worker killed at `landing-locked` without the landing lock, and releases the
recorded `index.lock` by identity.
The merged timelines of the failing runs (the driver prints them with `--verbose`) show the order:

1. A native survivor recovered the dead landing under the lock, released the dead worker's `index.lock`, and started
   landing its own commit, which created a new `index.lock`.
2. An incumbent survivor that had classified the same owner as dead earlier then logged `released owned lock
   .../index.lock` and removed the shadow repository of the same dead transaction, a few milliseconds later.
3. The native survivor's landing failed with `opening the index lock ... No such file or directory`
   (or `protecting .../index.lock failed`), exit status 2, and left its own transaction behind.
4. Every later command of either wrapper in that repository failed with the same error, because the left transaction
   has a landed commit and an index install that needs a lock that no longer exists.
   The repository was whole (`git fsck` clean) but wedged until the state is removed by hand.

Rates on the final native code, seed 2 repeated 100 times:
mixed 1543 checks passed and 174 failed, with a survivor failing in 56 repetitions
(21 native survivor exits, all of the `index.lock` kind above, and 67 incumbent survivor failures);
native only 200 of 200 repetitions clean;
incumbent only 100 of 100 clean in an earlier batch.
The failure rate of the mixed set varies with host load (19 of 60 repetitions in an earlier batch).
Established: the incumbent's predicate (source above), the order of events in the logs, and that homogeneous sets pass.
Not established: whether the incumbent's release matched the new lock by an inode number that the filesystem reused,
or because the release deletes by path after the identity check;
either way the incumbent acts on a lock it does not own.
Not fixed: it is the incumbent's code, and the incumbent has been removed from every `PATH`, so no mixed fleet remains.
The wedge in step 4 can arise natively only when a foreign process removes a lock a native lander owns;
recovery of a landed transaction fails closed when its lock is absent, as the specification requires,
and no change was made.

Incumbent-only findings (the incumbent is retired and not fixed):
concurrent recovery of one dead transaction made an incumbent survivor exit 1 with
`ENOENT: ... rename .../cli-git-transactions/<id> -> ....retired` (incumbent only, seed 12; mixed, seed 2) and with
`ENOENT ... scandir .../cli-git-transactions/<id>` (incumbent only, seed 18);
an incumbent hook dispatcher exited 1 with `ESRCH: no such process, read` inside its process birth identity reader
(mixed `hooked`, seed 3), which left that worker's commit unlanded.
The native wrapper reads `/proc` records through a path that maps `ESRCH` to an exited process
(`src/native/process_identity.rs`).

## Old-format fixtures

`native-interop/capture-fixtures.mjs` (deleted with the incumbent) wrote the incumbent's state into eight repositories:
`phase-<phase>` for the six phases (the incumbent killed itself there), `completed` (two finished commits, which
leave the capture store and landed-capture records) and `hook-lock-dead` (the incumbent killed while its `pre-commit`
hook ran, which leaves a dead hook lock).
They are kept under `native-interop/fixture/<name>/` as plain files:
every `.git` directory is stored as `dot-git`, because a repository cannot track a path named `.git`;
symbolic links of the incumbent's shadow repositories are listed in `links.tsv` of the fixture and recreated on restore;
directories that hold nothing are recreated on restore.
Each fixture restores to the absolute path it was captured at, since its records name absolute paths.
The only executable file left in the fixtures is `hook-lock-dead/hook.sh`, the three-line test hook the capture
ran (it appends `start` and `end` to a log); it holds no wrapper code.
The commit hook's final-newline fixer changed seven files of the captured set (six driver marker files and
`objects/info/packs` of Git itself); no record written by the incumbent differs from its capture.

`old-format-fixtures.mjs` restores each fixture in the image and runs only the native wrapper:

- `phase-capture-locked`, `phase-preparation-done`:
  `git status` recovers the transaction, the commit did not land, the real index equals `HEAD`, `git fsck` is clean,
  and a native commit lands.
- `phase-landing-locked`, `phase-objects-migrated`, `phase-ref-updated`, `phase-index-installed`:
  the records name device and inode numbers of files the incumbent created, and a restored copy has other numbers,
  so full recovery cannot be asserted statically.
  The native wrapper must read every record and either recover or refuse with its identity-changed message,
  never with a message about an unreadable record, and the repository must stay whole.
  The live pairings of `crash-matrix.mjs` above prove full recovery of these four phases.
  Observed: `landing-locked` and `objects-migrated` recover `git status` and leave the incumbent's `index.lock`
  alone because its identity differs; `ref-updated` and `index-installed` refuse with
  `Commit transaction file link identity changed`.
- `completed`: a native commit lands over the incumbent's capture store and advances the sequence.
- `hook-lock-dead`: a native commit retires the dead hook lock, runs the hook once and lands.

## Rerun commands

The incumbent exists only in history, so a rerun starts from the last commit that still builds it:

```sh
# Throwaway worktree of the last commit that holds the provisioning step and every driver.
git worktree add --detach "${HOME}/temp/agent/interop-rerun" 5d61d7b442
# Build the incumbent from the pinned revision, then run a driver (about 10 minutes for the first image build).
cd -- "${HOME}/temp/agent/interop-rerun/package/git-policy/cli"
GIT_POLICY_NATIVE_IMAGE_TAG=interop node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:native:interop -- crash-matrix
# Other drivers: lock-waits, mixed-concurrency [--wrappers=native|incumbent] [--only=killed] [--verbose] [seeds],
# owner-locks, phase-recovery, hook-dispatch, old-format-fixtures.
```

The fixture reader needs no incumbent and runs from any later commit that keeps
`native-interop/old-format-fixtures.mjs`:

```sh
GIT_POLICY_NATIVE_IMAGE_TAG=interop node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:native:interop -- old-format-fixtures
```

## Trailing gates

The matrix and the old-format fixtures are done.
The long fuzz campaigns, non-core mutation, the `pathBytes` field, the Windows checks and the release-artifact
performance gates are separate items of `doc/handover/cli-git-rust-implementation.md`.
