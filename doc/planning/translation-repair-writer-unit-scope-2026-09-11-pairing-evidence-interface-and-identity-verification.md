# Writer calibration unit scope: pairing evidence, seat identities and the call bound

The change that returns per-seat pairing outcomes so relations can be replayed,
the identity guard with its guard-removal proof on a disposable worktree,
and the distinction between the preparation call bound and other stages' bounds.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Pairing evidence interface

The production pairing stage returned counts and agreed relations,
but not the final per-seat outcomes needed to replay those relations independently.
`85f331192` adds regressions covering usable replies,
heard but out-of-range replies and missing voices.
`919e517e4` returns the existing `runWindowedRounds` outcomes unchanged on both result paths.
Prompts,
seat selection,
retry policy and the two-voice pair-agreement threshold are unchanged.
Preparation roster quorum remains a distinct condition from pair agreement.

The first green attempt exposed a test assertion mistake:
`toHaveProperty(path, value)` delegates to Chai's non-deep property comparison
in `package/test-harness/core/src/expect-matchers-core.ts`.
The tests now compare `outcome.outcomes` with `toEqual`.
The original red failures were missing-property assertions;
the attempted green failures are not evidence of changed pairing results.
At `06862482e`,
build,
manual type checking,
zero-warning Oxlint and the full unit suite pass.
`writer-pairing-evidence-final-unit-20260911.out` ends with the actual `unit exit 0`.
The successful package formatter takes no appended file arguments;
its attempted file-argument form failed before Oxlint ran.

## Pairing identity verification

Typed seat records alone do not prove independent endorsements.
The new replay operation initially accepted a duplicated identity as two votes.
`e19bb8158` records the behavioral red:
eight invalid-identity/live-dispatch tests fail,
while sparse ordered and single-identity controls pass.
The duplicate live configuration makes two mock model calls before the fix.

The shared `assertPairingSeats` guard now requires a nonempty unique electorate
and an ordered subsequence of recorded seat identities.
One strict-position check rejects unconfigured,
duplicated and reordered outcomes,
including duplicated unheard records.
Empty outcomes and sparse ordered subsets remain valid.
A single configured identity remains unable to meet the separate two-voice endorsement rule.
The live stage checks configuration before calls;
replay checks identities before counting endorsements.

`PairingEvidenceError` is separate from a model's unusable `BlockPairingError`.
Its closed failure kinds and numeric input positions cannot forward model prose.
The safe-message inventory was updated for the numeric position.
Early verification exposed incomplete inventory registration,
not failed identity controls.
After the inventory correction and statement-scope formatting,
`edb40e09d` passes build,
manual type checking,
zero-warning lint and the full suite.
`writer-pairing-identities-final-unit-r2-20260911.out` ends at line 9189 with `unit exit 0`.

A disposable worktree was created from the main worktree at `4df75028b`,
without copying ignored state.
Its exact package source was then overlaid from committed `0deca4036`,
with the changed-file list and every copied blob checked.
No corpus or credential file was copied.
The build/test container has 2 GiB RAM,
2 CPUs,
512 PIDs,
no network,
read-only shared dependencies and explicitly owned writable outputs.
Its worker runs Node 26.8.2;
the mutant test failure traces report Node 26.7.0 under Mise.
These are distinct executable layers,
not a claim that every command changed Node version.

The strict guard-removal proof succeeded:

- Each empty-electorate,
  duplicate-electorate,
  ordered-subsequence,
  live-preflight and replay-preflight removal rebuilt successfully.
- Each failed through the intended named `AssertionError`,
  not a signal or build failure.
- Removing the live preflight exposed two mock calls where zero were required.
- Restored source rebuilt and passed the identity suite.
- The container reported no OOM or PID-limit events;
  recorded memory peak was 718979072 bytes.

Proof log:
`~/temp/agent/pairing-guard-mutations-20260911.out`.
Detailed records were retained in `~/temp/agent/pairing-guard-proof-20260911`.
The owned worktree was removed normally from the main worktree after verifying the restored source,
retaining evidence and removing only owned untracked fixtures.
Its filesystem absence and removal from `git worktree list --porcelain` were verified.
Ignored root sentinels were absent;
ignore rules and targeted dry-run cleanup were checked in both roots.
The fixture's wrapped read-only check refused its untrusted `cli-git` configuration,
so native Git performed those read-only checks without granting persistent trust.
No paid preparation or writer calls have occurred.

## Preparation call-bound distinction

The selected direction must preserve the actual pairing call graph,
not reuse a different stage's bound.
`pairBlocksWithRoster` calls `runWindowedRounds`,
which has the initial gather and `STAGE_RETRY_ROUNDS` retries,
with no separate recovery nudge after that loop.
The writer/judge paths that do have a separate recovery nudge remain unchanged.
The registered preparation plan must derive its bounds from its actual selected parent pool,
configured roster and these specific call sites.
The provider POST bound also includes route fallback and HTTP attempts;
that transport registration is not implemented yet.
