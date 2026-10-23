# @monochromatic-dev/translation-repair-run

Run configuration and the reports that read a run back:
the roster and seats,
the runs directory and its lock,
the command-line reader,
meters,
spend and the candidate ledger,
plus the runners that report on them.
It is its own package because `corpus-pass`,
`probe`,
`bench` and `slice-cache` all start from one run configuration,
and the report runners sit with the log,
ledger and meter readers they use.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Run configuration and seats:
  the roster,
  per-call settings and seats a run uses,
  the run client built from them,
  and the floor of writing benches a run needs.
  Names:
  `RUN_ROSTER`,
  `RUN_MODELS`,
  `createRunClient`,
  `resolveRunsDir`,
  `readHeadSha`,
  `judgeSeatsFor`,
  `readJudgeSeats`,
  `WritingBenchUnreachableError`.
  Declared in `corpus-run/run-config.ts`,
  `corpus-run/run-seats.ts`,
  `corpus-run/run-seats-read.ts`,
  and `corpus-run/run-seats-floor.ts`.
- Runs directory and lock:
  lay out the runs directory and allow only one pass at a time per directory.
  Names:
  `prepareRunsLayout`,
  `RUNS_DIR_NAME`,
  `lockRunsDir`,
  `releaseIfOwned`,
  `evictStaleLock`,
  `RunsDirectoryBusyError`.
  Declared in `corpus-run/runs-layout.ts` and `corpus-run/runs-lock.ts`.
- Command line:
  read the flags of a runner against a declared specification.
  Names:
  `readCommandLine`,
  `COMMAND_LINES`,
  `idListFlag`,
  `wholeNumberFlag`,
  `reportingRefusals`,
  `registerRunEndReport`.
  Declared in `corpus-run/command-line.ts`,
  `corpus-run/command-lines.ts`,
  `corpus-run/command-flags.ts`,
  `corpus-run/cli-refusal.ts`,
  and `corpus-run/run-end-report.ts`,
  whose registry prints NeuralWatt's plan-energy difference when a run that called it ends.
- Meters and caps:
  read provider availability back out of logs,
  resolve the hard cap,
  and census the completion cap rule.
  Names:
  `readMeterLog`,
  `dutyCycle`,
  `drySpans`,
  `resolveHardCapMinutes`,
  `HARD_CAP_VAR`,
  `capCensus`.
  Declared in `corpus-run/meter-sample-read.ts`,
  `corpus-run/meter-dry-span.ts`,
  `corpus-run/cap-override.ts`,
  and `corpus-run/cap-census-rule.ts`.
- Spend and ledger:
  tally the spend lines of a log,
  price them,
  hold a spend ceiling,
  and read the candidate ledger a run wrote.
  Names:
  `readSpendLine`,
  `tallySpend`,
  `priceTally`,
  `resolveSpendCeilingUsd`,
  `spendCeilingReached`,
  `summariseLedger`,
  `readLedgerDirectory`.
  Declared in `corpus-run/spend-read.ts`,
  `corpus-run/spend-cost.ts`,
  `corpus-run/spend-ceiling.ts`,
  `corpus-run/ledger-read.ts`,
  and `corpus-run/ledger-directory.ts`.
- Run timing and published trees:
  read where a run's wall-clock went,
  and check a published tree against its artifacts.
  Names:
  `readRunTiming`,
  `measureInFlight`,
  `summariseRounds`,
  `verifyPublishedRun`,
  `judgePublishedEntry`.
  Declared in `corpus-run/run-timing-read.ts`,
  `corpus-run/verify-published-run.ts`,
  and `corpus-run/verify-published-entry.ts`.
- Required providers,
  culled seats and decisions:
  refuse a run whose required providers are not ready,
  refuse culled seats,
  build the client of OpenRouter's decisions router,
  read the corpus pin from the environment,
  and run one constructed comparison past the production judges.
  Names:
  `assertRequiredProvidersReady`,
  `readRequiredProviders`,
  `refusingCulledSeats`,
  `createDecisionsClient`,
  `readCorpusPinSetting`,
  `readRunCorpusPinSetting`,
  `runFidelityTrial`.
  Declared in `corpus-run/required-providers.ts`,
  `culled-seat-guard.ts`,
  `decisions-client.ts`,
  `corpus-run/corpus-pin-override.ts`,
  `corpus-run/run-corpus-pin.ts`,
  and `judge-fidelity.ts`.

## Runners

`src/build-entries.ts` lists the library index and each runner as a build entry,
and each runner is built to `dist/final/node/<runner>.mjs`.
Every runner has a task of the same name in `mise.toml` that builds first.
Start one with `mise run //package/translation-repair/run:<task>`,
passing arguments after `--`.
A built command refuses to run in a process that holds provider keys unless `TRANSLATION_REPAIR_STARTED_BY` names
that command,
and each task sets it to its own name.

- `budget-sample`:
  takes one reading of every provider's meter and leaves it in the log,
  so an outage's end is observed while no pass is running.
  Capture both streams with `> log 2>&1`.
  Spends no generation.
- `meter-report`:
  reads provider availability back out of run logs and reports each provider's duty cycle and longest outage as
  bounded ranges.
  Name one or more log paths after `--`.
  Spends no quota.
- `cap-census`:
  re-reads the completion cap rule over pass-run logs before a launch,
  per seat and provider,
  with how many capped calls ran to the cap and whether they carried content.
  Name log files or directories after `--`.
  Spends no quota.
- `ledger-report`:
  reads the candidate ledger a run wrote:
  who produced what,
  how often it was chosen,
  and what judges said about it.
  Pass `--model <id>` after `--` to read one seat's candidates and its judges' verbatim reasons,
  which prints corpus wording.
  Spends no quota.
- `repeat-census`:
  measures the repeated source wording of the pinned corpus:
  maximal runs and refrain frames under the main,
  loose and tight settings,
  with the body-only groups counted apart.
  Read-only.
  Spends no quota.
- `run-timing-report`:
  reads back where a run's wall-clock went,
  splitting every round into the time it worked and the time it waited on a straggler,
  and counting how many calls were in flight at once.
  Name one or more log paths after `--`.
  Spends no quota.
- `spend-report`:
  reads what a run cost from the spend lines its own log carries,
  priced against a dated Charm Hyper rate table;
  subscription seats are counted but never priced.
  Name one or more log paths after `--`.
  Spends no quota.
- `verify-published`:
  reads a published tree back and checks it against the artifacts that produced it:
  every settled entry has a page,
  and every page carries the wording its artifact says would ship.
  Reads `TRANSLATION_REPAIR_RUNS_DIR`.
  Spends no quota.

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
- `lane-contest`
- `markup`
- `page-check`
- `provider`
- `provider-stream`
- `repair`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `bench`
- `corpus-pass`
- `coverage-census`
- `probe`
- `slice-cache`

Packages that list it only in `devDependencies`:

- `source-scan`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/run:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/run:test:unit`,
or build and test together with `mise run //package/translation-repair/run:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/command-line.test-fixture.ts`,
`corpus-run/cli-refusal-cases.test-fixture.ts`,
and `introduced-defect-scripted-client.test-fixture.ts`.

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
