# @monochromatic-dev/translation-repair-stage

The panel-stage machinery that every model-backed step of the pipeline runs on:
rounds of calls to a seated roster,
the quorum that settles them,
the tally of what each seat did,
the prompt payload store,
and the stages that pair the sections and blocks of an original with its translation.
It is its own package because the lanes above it (translate,
repair,
consolidate,
consistency) all run their steps through it,
and it needs only text,
provider and shared-shape packages below it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Rounds and quorum:
  gather the voices of a stage in rounds,
  retry short benches,
  and decide when a quorum is reachable.
  Names:
  `gatherStageVoices`,
  `runGatherRound`,
  `runWindowedRounds`,
  `runRecoveryRound`,
  `reachableQuorum`,
  `MIN_STAGE_VOICES`,
  `STAGE_RETRY_ROUNDS`.
  Declared in `stage-quorum.ts`,
  `stage-round.ts`,
  `stage-windowed-rounds.ts`,
  `stage-recovery-round.ts`,
  and `stage-reachable-quorum.ts`.
- Stage calls and settings:
  one attempt at a stage call,
  the settings a stage reads,
  and the fan-out window and bench rotation.
  Names:
  `attemptStageCall`,
  `attemptDecisionCall`,
  `readStageSettings`,
  `StageSettings`,
  `STRAGGLER_GRACE_MS`,
  `askingWindow`,
  `benchRotation`.
  Declared in `stage-call.ts`,
  `stage-decision-call.ts`,
  `stage-settings.ts`,
  `stage-round.ts`,
  and `stage-fanout-window.ts`.
- Seat tally and roster checks:
  count what each seat did across a run,
  give the owner's verdict on a blocklisted model spelling,
  refuse a lane whose required role has nobody in it,
  and size an exact-half quorum.
  Names:
  `createSeatTally`,
  `seatTallyClient`,
  `RUN_SEATS`,
  `seatReportLines`,
  `ROSTER_BLOCKLIST`,
  `blocklistVerdictFor`,
  `assertRostersConfigured`,
  `rosterQuorumSize`.
  Declared in `seat-tally.ts`,
  `roster-blocklist.ts`,
  `roster-configuration.ts`,
  and `roster-quorum-size.ts`.
- Silence and recovery:
  name stages that went unheard,
  and the nudges used to recover from a cut-short or off-shape reply.
  Names:
  `silentStagesOf`,
  `silentStagesOfRecord`,
  `refinementIsResumable`,
  `stageQuorumUnmetFinding`,
  `RECOVERY_NUDGES`,
  `unreadableCauseOf`.
  Declared in `stage-silence.ts` and `recovery-nudge.ts`.
- Prompt payloads and uniqueness:
  store each model prompt durably,
  pick a fence longer than any fence in the text,
  and digest a model with its prompt so that one model is never sampled twice on one completed prompt.
  Names:
  `promptPayloadStore`,
  `PromptPayloadStoreError`,
  `selectFence`,
  `longestFenceRun`,
  `modelPromptDigest`,
  `promptUniqueClient`.
  Declared in `prompt-payload-store.ts`,
  `prompt-fence.ts`,
  and `prompt-uniqueness-client.ts`.
- Section and block pairing:
  the prompts,
  wire formats and stage that pair sections,
  and the agreement and step conversion for block pairs.
  Names:
  `pairSectionsWithRoster`,
  `buildSectionPairingMessages`,
  `readSectionPairing`,
  `buildBlockPairingMessages`,
  `readBlockPairing`,
  `agreePairs`,
  `blockPairingToSteps`.
  Declared in `pair-sections-stage.ts`,
  `pair-sections-wire.ts`,
  `pair-sections-read.ts`,
  `pair-blocks-wire.ts`,
  `pair-agreement.ts`,
  and `pair-blocks-steps.ts`.
- Overlap and memo helpers:
  run a bounded number of jobs at once and return results in item order,
  and reuse the answer of a twin slice within a run.
  Names:
  `mapOverlapped`,
  `OverlapRefusedError`,
  `reuseTwinOrBuy`,
  `TwinMemo`.
  Declared in `overlapped-map.ts` and `twin-memo.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `provider`
- `provider-stream`
- `shared-contract`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `bench`
- `consistency`
- `consolidate`
- `corpus-pass`
- `corpus-source`
- `coverage-census`
- `lane-contest`
- `page-check`
- `probe`
- `reference`
- `rendering-audit`
- `repair`
- `run`
- `slice-cache`
- `slice-reading`
- `translate-floor`
- `translate-lane`
- `translate-stage`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/stage:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/stage:test:unit`,
or build and test together with `mise run //package/translation-repair/stage:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `stage-settings.test-fixture.ts` and `corpus-run/fresh-run-seats.test-fixture.ts`.

Test helpers used across the family,
such as scratch directories and capturing loggers,
come from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/<file>`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
