# @monochromatic-dev/logger-core.fuzz

Property-based verification campaign for
[`@monochromatic-dev/logger-core`](../core/README.md).

Non-runtime sidecar,
 mirroring `jsonc-edit.fuzz` and `css-edit.fuzz`:
the runtime package's `src` stays pure production code and ships in its tarball,
while the scripted fake sinks,
 the reference model,
 the properties,
 the run-budget tooling,
 and the coverage gate live here and never publish.

The logger differs from the parser campaigns in one structural way:
its bugs are timing and interleaving bugs,
 not input bugs.
So the generators are a scheduler over sink hook settlements,
 not a grammar,
 and every oracle is the reference model in `src/model.ts`.
Decision record:
 `doc/decision/logger-fuzzing.md`.
Plan and grill record:
 `package/logger/core/bulletproofing.plan.md`.

## Layout

- `src/fuzz-budget.ts`:
   the two run layers (bounded in `test:unit`,
   time-budgeted in `fuzz`) keyed on `LOGGER_FUZZ_BUDGET_MS`.
- `src/fake-sink.ts`:
   a `Sink` driven by a per-hook script of outcomes,
   each hook a per-call sequence with a repeating tail,
   with a stable identity so a shrunk counterexample reads as
   `sink 2: verify [resolve*] write [reject, resolve*] flush absent`.
- `src/model.ts`:
   reference model predicting,
   per sink,
   the exact records received and the final availability,
   whether `flush()` settles within its deadline,
   the dropped-count marker record,
   and the breadcrumb count,
   for an operation sequence and the observed hook settlement order.
- `src/boundary-corpus.ts`:
   the committed message corpus behind the sink boundary properties,
   with no fast-check import.
- `src/adversarial-message.ts`:
   record arbitraries interleaving that corpus with binary text and control characters.
- `src/sink-boundary-harness.ts`:
   a throwaway package directory for the file sink,
   readers that reparse what the JSONL sinks persisted,
   and an independent reference for the console sink's output.
- `src/coverage-driver.ts` and the other `src/coverage-*.ts` modules:
   the fixed-input reachability driver over the runtime package's `/ts` source,
   its stand-ins for storage,
   page lifecycle,
   and hostile hosts,
   and the V8 line-coverage projector and baseline gate copied from toml-edit.
- `coverage-baseline.json`:
   covered code lines per runtime source file;
   the gate fails on any per-file decrease.
- `src/browser/`:
   the Playwright browser property layer:
   a page bundle of fast-check,
   the neutral artifact,
   and the localStorage,
   IndexedDB,
   and OPFS properties,
   plus the Playwright test that loads it into the harness page and prints each browser's outcomes.
- `src/*.property.unit.test.ts`:
   the properties;
   every one imports the built runtime artifact through the package name,
   never the runtime package's source.
   `dist` is not committed,
   so a build older than the runtime source runs the reference model against
   code the source no longer has,
   and both layers therefore build the runtime first.
- `src/artifact-freshness.unit.test.ts`:
   the guard for that import,
   failing with the build task to run when the artifact is older than any
   runtime source file it was built from.

## Running

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/logger/core.fuzz:<task>`.
The unit suite runs each property 60 times by default.
Both layers build `//package/logger/core` first,
because the properties compare that build with the reference model:
`fuzz` builds it in its own script,
`test:unit` through its `depends`.
Runs that do not go through this package's `test:unit` task need the build first:
a bare `node src/orchestration.property.unit.test.ts`,
or the root `test:unit`,
which starts each test file directly.
`src/artifact-freshness.unit.test.ts` names that step when it is missing,
because a stale build made a property report a model disagreement
(`no-backend throw agreement`) on 2026-11-10
that no current source produced.
The browser property layer runs locally in podman:
build the bundle with `bundle:browser-properties` first,
then run `mise run test:browser -- package/logger/core.fuzz/src/browser/properties.browser.test.ts`.
