# @monochromatic-dev/translation-repair-corpus-pass

The corpus pass:
it runs the whole pipeline over every complete corpus pair,
one entry at a time,
from preparation through the lanes,
the lane contest,
consolidation and the page-level consistency pass to a settled artifact.
It is its own package because it drives almost every other family package at once,
and only `probe` lists it in `dependencies`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- The pass and its queue:
  select the pending entries,
  order them,
  run the queue,
  and keep a pass from resuming into the wrong state.
  Names:
  `runCorpusPassOver`,
  `runPassQueue`,
  `selectPendingEntries`,
  `orderPendingEntries`,
  `assertPassResumable`,
  `passLaunchLines`,
  `SOFT_BUDGET_MS`.
  Declared in `corpus-run/corpus-pass-run.ts`,
  `corpus-run/corpus-pass-queue.ts`,
  `corpus-run/corpus-pass-select.ts`,
  `corpus-run/corpus-pass-order.ts`,
  `corpus-run/corpus-pass-guards.ts`,
  `corpus-run/corpus-pass-lines.ts`,
  and `corpus-run/corpus-pass-limits.ts`.
- Entry preparation and settlement:
  prepare one entry,
  run its steps,
  and persist the settled artifact.
  Names:
  `preparePassEntry`,
  `runPassPreparation`,
  `settleEntry`,
  `persistSettledEntry`,
  `printSettledLines`,
  `runPassSettlement`.
  Declared in `corpus-run/pass-prepare.ts`,
  `corpus-run/pass-prepare-reseat.ts`,
  `corpus-run/pass-entry.ts`,
  `corpus-run/pass-entry-persist.ts`,
  and `corpus-run/pass-settlement.ts`.
- Lanes,
  contest and consolidation:
  run both lanes,
  the contest between them and the consolidation of a third rendering over one document.
  Names:
  `runPassLanes`,
  `runPassContest`,
  `runPassConsolidation`,
  `contestDocumentLanes`,
  `consolidateDocument`,
  `lanesHooksFor`.
  Declared in `corpus-run/pass-lanes.ts`,
  `corpus-run/pass-contest.ts`,
  `corpus-run/pass-consolidate.ts`,
  `lane-contest-driver.ts`,
  `consolidate-driver.ts`,
  and `corpus-run/pass-reseat.ts`.
- Page-level consistency in the pass:
  ask the consistency bench for a document,
  decide each group,
  carry the chosen wording and apply it.
  Names:
  `runPassConsistency`,
  `applyConsistency`,
  `askConsistencyBench`,
  `decideAskedGroup`,
  `carryAll`,
  `openConsistencyCache`.
  Declared in `corpus-run/pass-consistency.ts`,
  `corpus-run/pass-consistency-apply.ts`,
  `corpus-run/pass-consistency-ask.ts`,
  `corpus-run/pass-consistency-bench-decide.ts`,
  `corpus-run/pass-consistency-carry.ts`,
  and `corpus-run/pass-consistency-cache.ts`.
- Page assembly:
  compose the settled artifact with the page guard's outcome,
  ask the page-level guards of the would-ship page,
  settle or resume the lexicon of repeated page titles,
  normalize the archive text every stage reads,
  and have the roster attest the archive details a reference states.
  Names:
  `settledPageArtifact`,
  `assertPageGuards`,
  `passPageTitles`,
  `passArchiveText`,
  `attestPassReferences`.
  Declared in `corpus-run/pass-page-assembly.ts`,
  `corpus-run/pass-page-guards.ts`,
  `corpus-run/pass-page-titles.ts`,
  `corpus-run/pass-archive.ts`,
  and `corpus-run/pass-attest-references.ts`.
- Insertions and pictures:
  admit insertions into the pass,
  relabel archive footnotes,
  and read the pictures of an entry.
  Names:
  `admitPassInsertions`,
  `decidePassInsertionAdmission`,
  `relabelArchiveFootnotes`,
  `createPassPictureReader`.
  Declared in `corpus-run/pass-insertion-reseat.ts`,
  `corpus-run/pass-insertion-admission.ts`,
  `corpus-run/pass-footnote-relabel.ts`,
  and `corpus-run/pass-seated-pictures.ts`.

## Runners

`src/build-entries.ts` lists the library index and each runner as a build entry,
and each runner is built to `dist/final/node/<runner>.mjs`.
Every runner has a task of the same name in `mise.toml` that builds first.
Start one with `mise run //package/translation-repair/corpus-pass:<task>`,
passing arguments after `--`.
A built command refuses to run in a process that holds provider keys unless `TRANSLATION_REPAIR_STARTED_BY` names
that command,
and each task sets it to its own name.

- `corpus-pass`:
  runs the pipeline over every complete corpus pair;
  artifacts land in `node_modules/.monochromatic`.
  Append `-- --plan` for a zero-quota setup check,
  `-- --only Id1,Id2` to target named entries (which wants a throwaway `TRANSLATION_REPAIR_RUNS_DIR`),
  or `-- --require-providers synthetic,hyper` to refuse the run unless those providers' keys are set and their
  meters read wet.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `archive`
- `artifact`
- `artifact-pool`
- `assembly`
- `consistency`
- `consolidate`
- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `insertion`
- `lane-contest`
- `markup`
- `page-check`
- `page-pass`
- `provider`
- `provider-stream`
- `reference`
- `rendering-audit`
- `repair`
- `run`
- `shared-contract`
- `slice-cache`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `adjudication`
- `source-scan`
  (the schema cases parse the family's source with its `parseSource` and walk the tree with its helpers,
  through `@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts`)

### Used by

Packages that list it in `dependencies`:

- `probe`

Packages that list it only in `devDependencies`:

- `bench`
- `coverage-census`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/corpus-pass:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/corpus-pass:test:unit`,
or build and test together with `mise run //package/translation-repair/corpus-pass:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/probes-b-built-command.test-fixture.ts`,
`archive-naming.test-fixture.ts`,
and `corpus-run/editor-bench-clone.test-fixture.ts`.

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
