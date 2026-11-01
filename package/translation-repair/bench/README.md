# @monochromatic-dev/translation-repair-bench

Benches and calibration:
the window trial,
editor width and calibration probes,
roster and producer benches,
model health and catalogue checks,
the roster card printer,
and the sentinel,
displacement and translate probes.
It is its own package because these runners calibrate the roster and the lanes from outside the pipeline,
and `coverage-census` is the only family package that lists it in `dependencies`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Roster and producer benches:
  run the same slices at every roster width and rank producers by the ballots their work drew.
  Names:
  `runRosterBench`,
  `runBenchRow`,
  `runProducerCalibrate`,
  `runCalibrationRound`,
  `sampleBenchSlices`,
  `writeBenchReport`.
  Declared in `corpus-run/roster-bench-run.ts`,
  `corpus-run/roster-bench-round.ts`,
  `corpus-run/producer-calibrate-run.ts`,
  `corpus-run/producer-calibrate-round.ts`,
  `corpus-run/bench-sample.ts`,
  and `corpus-run/bench-report.ts`.
- Editor calibration and width probe:
  rank models on the editor's own job,
  and compare the production editor width with the whole roster.
  Names:
  `runEditorCalibrate`,
  `runEditorLane`,
  `driveEditorCalibrate`,
  `runEditorWidthProbe`,
  `classifyWidths`,
  `widthControlHolds`.
  Declared in `corpus-run/editor-calibrate-run.ts`,
  `corpus-run/editor-calibrate-lane.ts`,
  `corpus-run/editor-calibrate-drive.ts`,
  `corpus-run/editor-width-probe-run.ts`,
  `corpus-run/editor-width-model.ts`,
  and `corpus-run/editor-width-control.ts`.
- Window trial:
  judge flagged slices and matched controls over one slate,
  with and without the neighbouring original.
  Names:
  `runWindowTrial`,
  `runPick`,
  `reportWindowTrial`,
  `accountTrialLedger`,
  `appendTrialRow`,
  `protocolDigest`,
  `assertWindowReachedJudges`.
  Declared in `corpus-run/window-trial-probe-run.ts`,
  `corpus-run/window-trial-pick.ts`,
  `corpus-run/window-trial-report.ts`,
  `corpus-run/window-trial-ledger.ts`,
  `corpus-run/window-trial-protocol.ts`,
  and `corpus-run/window-trial-witness.ts`.
- Model health and catalogue:
  ask every roster model one trivial question,
  and compare what a provider serves with the compiled catalogue.
  Names:
  `askModelHealth`,
  `reportModelHealth`,
  `compareCatalog`,
  `decodeModelList`,
  `formatCatalogReport`,
  `CATALOG_MODEL_IDS`.
  Declared in `corpus-run/model-health-probe.ts`,
  `corpus-run/model-health-report.ts`,
  and `corpus-run/model-catalog-compare.ts`.
- Roster card:
  print the model card fragment for one model off a provider's live listing.
  Names:
  `renderProviderCard`,
  `fetchListing`,
  `printRosterCard`,
  `cardFieldsFrom`.
  Declared in `corpus-run/roster-card-render.ts`,
  `corpus-run/roster-card-listing.ts`,
  and `corpus-run/roster-card-print.ts`.
- Reader measure:
  hold a candidate picture reader to the readings the pipeline already trusts,
  by the two-clause reader rule of 2026-09-08.
  Names:
  `seatedBandOf`,
  `candidateRatesOf`,
  `verdictOf`,
  `MINIMUM_PRODUCED_READINGS`,
  `readTrustedRecords`,
  `matchRecord`,
  `measureReader`,
  `runReaderMeasure`,
  `readMeasureAsk`,
  `renderMeasureReport`,
  `loadPictures`,
  `filesUnder`.
  Declared in `corpus-run/reader-measure-score.ts`,
  `corpus-run/reader-measure-records.ts`,
  `corpus-run/reader-measure-run.ts`,
  `corpus-run/reader-measure-command.ts`,
  `corpus-run/reader-measure-ask.ts`,
  `corpus-run/reader-measure-report.ts`,
  `corpus-run/reader-measure-pictures.ts`,
  and `corpus-run/reader-measure-files.ts`.
- Sentinel,
  displacement and translate probes:
  run named entries through the pipeline,
  read size ratios for moved passages,
  and prototype a translate-first render.
  Names:
  `probeCorpusEntries`,
  `probeDisplacement`,
  `corpusTotals`,
  `probeTranslate`,
  `sparsestPair`.
  Declared in `corpus-run/sentinel-probe-run.ts`,
  `corpus-run/displacement-probe-run.ts`,
  `corpus-run/displacement-probe-totals.ts`,
  `corpus-run/translate-probe-run.ts`,
  and `corpus-run/translate-probe-coverage.ts`.
- Checker sensitivity:
  ask whether the resolution checkers can say no.
  Names:
  `runCheckerSensitivity`,
  `checkOne`,
  `SINGLE_ISSUE_CASES`,
  `checkAllFixedSheet`.
  Declared in `corpus-run/checker-sensitivity-run.ts` and `corpus-run/checker-sensitivity-input.ts`.

## Runners

`src/build-entries.ts` lists the library index and each runner as a build entry,
and each runner is built to `dist/final/node/<runner>.mjs`.
Every runner has a task of the same name in `mise.toml` that builds first.
Start one with `mise run //package/translation-repair/bench:<task>`,
passing arguments after `--`.
A built command refuses to run in a process that holds provider keys unless `TRANSLATION_REPAIR_STARTED_BY` names
that command,
and each task sets it to its own name.

- `sentinel-probe`:
  runs named corpus entries through the pipeline to validate a change.
  Pass ids after `--`.
- `displacement-probe`:
  reads per-slice size ratios against each document's own median to find passages the translation moved across a
  section boundary.
  Spends no quota.
- `editor-calibrate`:
  ranks every model on the editor's own job by driving the whole repair lane,
  every roster model editing and judging each slice,
  and reports the refiner standing off the same spend.
  Pass a slice count after `--` (default 6);
  `TRANSLATION_REPAIR_SLICE_OVERLAP=N` runs N slices at once (default 4).
  `--candidates <seatable ids>` seats those cards as the editors,
  beside the roster's writers and in place of the run's named editors,
  so a card that holds `judge-unmeasured`,
  such as a Sonnet 5.5 effort card,
  can be measured;
  `--candidates-alone` seats only those cards.
  Every other seat stays the whole roster either way,
  so the flag confines the editing and not the spend.
  One editor alone has no rival to be judged against,
  so its standing holds no round,
  and `EDITORS SHIPPED` is the only reading.
  Spends quota.
- `editor-width-probe`:
  runs the same slices at the production editor width and at the whole roster,
  judged by one fixed panel,
  with the narrow arm repeated as a null band and the two winners contested in both seating orders.
  Pass a sample size after `--` (default 18,
  halved into two draws).
  Spends quota.
- `translate-probe`:
  a prototype for the translate-first redesign:
  asks a translator ensemble to render the corpus section with the worst coverage gap,
  which the repair loop cannot express.
- `roster-card`:
  prints the fragment for `package/translation-repair/shared-contract/src/model-cards.ts` for one model off a
  provider's live listing,
  so seating a model is one card and one list entry.
  Usage after `--`:
  `<synthetic|hyper|openrouter|bedrock|neuralwatt|deepseek|xiaomi> <served id>`.
  Read-only;
  the key is never printed.
- `reader-measure`:
  measures a candidate picture reader through OpenRouter (zero data retention on every request)
  against the picture readings the pipeline already stored,
  and says whether both clauses of the reader rule are met and which hold to lift.
  It builds the OpenRouter client itself,
  because the router keeps pictures off that route while a card carries the `openrouter-pictures-unmeasured` hold
  (no card carries it since `gemma-4-26b-a4b-it` was measured on 2026-10-21 (UTC),
  and the account of that measurement closes this entry).
  Usage after `--`:
  `--trusted <runs dir>` (repeatable),
  `--pictures <people dir>`,
  and optionally `--model <seat id>` (default `gemma-4-26b-a4b-it`)
  and `--sample <count>` (default 9 matched pictures).
  Needs `TRANSLATION_REPAIR_OPENROUTER_API_KEY`.
  Spends the OpenRouter key:
  one ask per matched picture.
  Where the trusted records live:
  a `--trusted` directory holds runs directories,
  and a runs directory holds a record when its `slice-cache/<entry>/` holds `picture.<key>.json` files.
  The recent `corpus-pass-*` runs directories hold few of them,
  all for one entry in the case of 2026-10-21 (UTC),
  and the older single-entry runs directories hold most:
  107 of them held 450 files and 192 distinct trusted reading sets across 10 entries.
  A run that matches too few pictures needs `--trusted` directories of other entries,
  and a larger `--sample` cannot help once every picture has been tried.
  The report prints the `ENTRIES:` line (the matched pictures counted by entry),
  the `SEATED:` band and the `CANDIDATE:` line (the candidate's reading count and both shares beside the band's floor
  and ceiling),
  so a verdict is checked from the printout and a narrow sample is visible.
  The measurement of 2026-10-21 (UTC):
  three runs of `gemma-4-26b-a4b-it` through OpenRouter with zero data retention at `--sample 9`.
  The first (08:44 UTC) and second (08:48 UTC) named the recent runs directories
  (the second adding the full-corpus pass of 2026-08-19),
  matched 3 of 291 pictures and were not measured (ledger B800).
  The third (09:02 UTC) named 107 older runs directories,
  matched 9 pictures from two entries (Mio 7,
  Toka_ls 2),
  and met both clauses (seated floor 1.00 and ceiling 0.00 over 43 readings),
  so the hold was lifted.
  It spent 15 calls (3,
  3 and 9).
- `model-health`:
  asks every roster model one trivial structured question and reports what came back,
  so a schema mismatch can be told apart from a dead model.
- `editor-standing-read`:
  reads the editor and refiner standings off already-settled artifacts,
  reported per pipeline digest and never pooled across them.
  Observational:
  it ranks only models that held a seat.
  Pass run directories after `--`.
  Spends no quota.
- `model-catalog`:
  compares what the provider serves against the compiled catalogue,
  so a model that vanished stops costing a silent lost voice and a model holding no run role becomes visible.
- `checker-sensitivity`:
  asks whether the resolution checkers can say no:
  a genuine fix,
  an untouched text,
  and a fix that damaged its slice;
  cat-themed fixtures only.
- `producer-calibrate`:
  runs the same corpus slices with every roster model writing and every one judging,
  then ranks producers by the share of disinterested ballots their work drew.
  Wants a throwaway `TRANSLATION_REPAIR_RUNS_DIR`;
  pass a slice count,
  then `--candidates` with seatable ids and `--candidates-alone`,
  after `--`.
  Spends quota.
- `roster-bench`:
  runs the same corpus slices through the translate lane at every producer-roster width,
  recording decline,
  tie,
  self-vote and cost rates.
  Wants a throwaway `TRANSLATION_REPAIR_RUNS_DIR`;
  pass a slice count after `--`.
  Spends quota.
- `window-trial-probe`:
  judges flagged slices and matched controls three times over one slate (narrow,
  narrow again,
  then with the neighbouring original) and reports whether the window moved the replacement rate further than two
  narrow runs move it.
  Resumes from its ledger;
  wants a throwaway `TRANSLATION_REPAIR_RUNS_DIR`.
  Spends quota.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `artifact`
- `artifact-pool`
- `assembly`
- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `insertion`
- `markup`
- `page-pass`
- `probe`
- `provider`
- `provider-stream`
- `repair`
- `run`
- `shared-contract`
- `slice-cache`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `corpus-pass`

### Used by

Packages that list it in `dependencies`:

- `coverage-census`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/bench:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/bench:test:unit`,
or build and test together with `mise run //package/translation-repair/bench:buildAndTest`.
Pass test file paths to run only those files.

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
