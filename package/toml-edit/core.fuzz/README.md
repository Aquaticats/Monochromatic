# @monochromatic-dev/toml-edit-core.fuzz

Property-based verification campaign for
[`@monochromatic-dev/toml-edit-core`](../core/README.md).

Non-runtime sidecar,
 mirroring `logger.fuzz`,
 `jsonc-edit.fuzz`,
 and `css-edit.fuzz`:
the runtime package's `src` stays pure production code and ships in its tarball,
while the generators,
 the semantic-equality oracle,
 the corpus loader,
 the properties,
 the run-budget tooling,
 and the coverage gate live here and never publish.
The campaign lived under `package/toml-edit/core/src/fuzz/` until 2026-09-06;
the decision record and the implementation plan below predate the move and name that location in places.
Decision record:
 `doc/decision/toml-edit-fuzzing.md`.
Implementation plan and status:
 `HANDOVER.fuzzing.md` beside this file.

## Layout

- `src/fuzz-budget.ts`:
   the two run layers (bounded in `test:unit`,
   time-budgeted in `fuzz`) keyed on `TOML_EDIT_FUZZ_BUDGET_MS`.
- `src/arb-*.ts`,
   `src/escape.ts`,
   `src/mutators.ts`,
   `src/corpus.ts`:
   grammar-complete generators with an independent string escaper,
   structure-aware corruption mutators,
   and the committed-fixture and live-repository corpus loader.
- `src/equality.ts`:
   the semantic-equality oracle over the parser's static-value projection.
- `src/*.property.unit.test.ts`:
   the properties;
   every one imports the built runtime artifact through the package name,
   never the runtime package's source.
- `src/coverage-driver.ts` and the other `src/coverage-*.ts` modules:
   the fixed-seed reachability driver over the runtime package's `/ts` source
   and the V8 line-coverage projector and baseline gate.
- `coverage-baseline.json`:
   covered code lines per runtime source file;
   the gate fails on any per-file decrease.

## Pre-install workflow scope

`src/ci-scope.ts` is the deployed Node entrypoint for `.github/workflows/toml-edit-fuzz.yml`.
It loads only Node built-ins and local source,
so scope detection does not require pnpm installation or a build.
The workflow's `shell: node {0}` launcher delegates to this entrypoint without Bash.

Push and pull-request events run verification.
Merge-group events validate `SCOPE_BASE_SHA` and `SCOPE_HEAD_SHA` from the event payload,
require the checked-out head to match,
verify base ancestry,
and compare those exact trees.
A missing commit or failed Git operation fails the step without writing a skip decision.
Checkout fetches full history;
the command validates missing objects rather than guessing a fallback base.

The comparison includes deletions and both sides of renames,
uses NUL-delimited paths,
and matches directory boundaries or exact contract files.
Each Git call has a 30-second timeout and a 16-MiB capture limit.
Exceeding either limit fails verification rather than silently truncating the change set.

`src/ci-scope.unit.test.ts` executes the actual workflow launcher against disposable repositories,
including a deliberately fail-open copy that proves the failure oracle detects guard removal.
Tests run on the Linux workflow host and use `/usr/bin/git` with disposable home and Git configuration.
`SCOPE_TEST_NODE` can select an older Node executable for the deployed command,
while the test harness remains on the current project runtime.
The bootstrap runtime floor is Node 22.18.0.

```sh
# From the repository root
mise run //package/toml-edit/core.fuzz:test:scope
mise run //package/toml-edit/core.fuzz:lint:scope
mise run //package/toml-edit/core.fuzz:lint:types
```

## Running

```bash
# Bounded layer, the same files the unit suite runs
mise run //package/toml-edit/core.fuzz:test:unit

# Time-budgeted campaign (rebuilds module-toml-edit first)
mise run //package/toml-edit/core.fuzz:fuzz --budget 60000

# Coverage-reachability gate; --write refreezes the baseline
mise run //package/toml-edit/core.fuzz:fuzz:coverage
```
