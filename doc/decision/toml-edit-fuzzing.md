# TOML edit fuzzing

Decision record for the property-based fuzzing of
`@monochromatic-dev/toml-edit-core` (issue #198).
 This file grows as the work
lands;
 it currently covers the method,
 the unstable seam exports,
 and the bugs
found so far.
 The full implementation plan and status live at
`package/toml-edit/core.fuzz/HANDOVER.fuzzing.md`.

## Method

The suite is property-based,
 built on `fast-check` and the
`@monochromatic-dev/test-harness-core` harness,
 and runs under node (node v26 runs the
`.ts` files directly).
 The same `*.property.unit.test.ts` files serve two layers:
a bounded run in the normal unit suite,
 and a time-budgeted campaign through the
package `fuzz` task (`TOML_EDIT_FUZZ_BUDGET_MS`).
 See
`package/toml-edit/core.fuzz/src/fuzz-budget.ts`.

Every property file imports the built package entry point
(`@monochromatic-dev/toml-edit-core`),
 not sibling source,
 so the suite tests
the shipped artifact across the consumer boundary.
 The `fuzz` task therefore
builds before running.

The oracle stack so far:

- A semantic-equality oracle (`package/toml-edit/core.fuzz/src/equality.ts`)
  over the parser's `getStaticTOMLValue` projection.
- Grammar-complete generators for every value,
   key,
   and document shape
  (`package/toml-edit/core.fuzz/src/arb-*.ts`),
   with an independent string
  escaper (`package/toml-edit/core.fuzz/src/escape.ts`) so generator encoding
  never reuses the emitter under test.
- Structure-aware corruption mutators
  (`package/toml-edit/core.fuzz/src/mutators.ts`).
- A corpus loader (`package/toml-edit/core.fuzz/src/corpus.ts`):
   committed
  fixtures always,
   live repository discovery only in campaign mode,
   with curated
  fixture sets and secret-looking paths excluded.

## Unstable seam exports

Property files exercise internal encoders and emitters through the built artifact
rather than sibling source imports.
 Where a seam is not part of the public API,
it is re-exported from `package/toml-edit/core/src/index.ts` with an
underscore prefix:
 `_encodeKey`,
 `_jsValueToTomlText`,
 `_emitContentNode`,
`_emitStringValue`,
 and `_spliceEmit`.

These exports carry no compatibility promise.
 They exist for observability and
fuzzing,
 their signatures may change without a major version bump,
 and
application code must not depend on them.
 The underscore prefix is the marker.
Each export is documented at its declaration in `index.ts` and in the package
README.

## Bugs found and fixed

- `parseTomlEdit` leaked a raw `RangeError` on pathologically deep `[` or `{`
  nesting (a stack overflow escaping the `ParseError`-only catch).
   Fixed by
  wrapping every non-`ParseError` throw from the parser as `TomlEditError`;
   the
  totality property pins the regression.
- `emitStringValue` escaped only the named control escapes,
   emitting other
  control scalars (NUL,
   U+001F,
   U+007F) raw and producing invalid TOML.
   Fixed by
  escaping every control scalar below U+0020 plus U+007F as `\uXXXX`,
   in both
  single-line and multiline basic strings;
   the emitter property pins the
  regression.
- The from-scratch encoders had the same control-character gap on a different
  code path:
   `encodeStringWithStyle` (used when `tomlSet` writes a fresh string)
  and `encodeKey` (used for any quoted key) escaped only the named escapes,
  emitting NUL,
   U+001F,
   U+007F,
   and bare newlines raw.
   The toml-test encoder
  cases surfaced it.
   Fixed by extracting the exhaustive escaper to a shared
  `basic-escape.ts` and delegating all three call sites (`emit-value-string.ts`,
  `value-encoders.ts`,
   `keys.ts`);
   the emit-seams property pins the regression.
- `parseTomlEdit` accepted a bare carriage return (a lone `CR` not part of
  `CRLF`),
   which TOML forbids everywhere,
   including inside multiline `strings.toml`-eslint-parser inherits the laxity.
   Fixed in a pre-parse scan that rejects
  any `CR` not followed by `LF`;
   the toml-test `invalid/control/*-cr` cases and a
  pinned parser-property example pin the regression.

## Newline policy

`parseTomlEdit` normalizes `CRLF` to `LF` before parsing and holds the normalized
source in the state,
 so the splice,
 comment-range,
 and emission paths only ever
reason about single-byte `LF` newlines.
 The conversion is announced with a
`warn`-level log (suppressible via `MONOCHROMATIC_WARN=false`,
 a mechanism added to
`@monochromatic-dev/logger-core` for machine-protocol consumers whose output
streams must stay clean).
 A bare `CR` is rejected rather than normalized,
 so the
policy stays spec-correct:
 only valid `CRLF` is rewritten.
 The trade-off is that a
`CRLF` document no longer round-trips byte-identically;
 it round-trips as `LF` by
design.

The policy is symmetric on output:
 the canonical builder's `lineBreak` option
(formerly `'\n' | '\r\n'`) was dropped,
 so canonical emission only ever writes
`LF`.
 No caller requested `CRLF` output,
 and keeping an opt-in knob that
re-introduces the line ending the parser refuses to round-trip was the asymmetry
the normalization set out to remove.
 The package is now `LF`-only end to end,
which is what lets `splice.ts` and `comments.ts` reason about a single-byte
newline without any `CR` branch.

## Conformance (phase 6)

Package-local node adapters under `package/toml-edit/core/src/conformance/`
satisfy the upstream `toml-test` runner's decoder (TOML to tagged JSON) and
encoder (tagged JSON to TOML) interfaces,
 exercising the built package across the
consumer boundary:

- The decoder gates acceptance through `parseTomlEdit`,
   validates UTF-8 strictly
  at the byte boundary (a fatal `TextDecoder` rejects the malformed-byte and
  surrogate corpus cases),
   and walks the parse-time AST into a kind-aware tagged
  model (the converter deferred from phase 2):
   integers use the node's exact
  `bigint`,
   offset datetimes normalize to an RFC 3339 instant,
   and local
  datetimes pad in a seconds field the runner's layout requires.
- The encoder rebuilds via `emptyTomlEdit` plus `tomlSet`,
   so the package's
  emission path is what the runner reparses and compares;
   integers use a `bigint`
  wrapper and the three local datetime kinds use tagged wrappers a `Date` cannot
  distinguish.
- The `test:conformance` task acquires the runner through mise's `github:` backend
  at `latest` (a deliberate moving oracle,
   not pinned;
   mise verifies the release
  attestation and SLSA provenance,
   and the task logs the resolved version) and
  runs both adapters for TOML 1.0 and 1.1 under `MONOCHROMATIC_WARN=false`.
   Both versions pass
  every valid,
   encoder,
   and invalid case with no allow-list.

## Bugs found and deferred

The stateful edit-model property surfaced two deeper edit-machinery defects that
need resolver-level changes (making set and delete aware of pending insertions),
so they are tracked as a follow-up in #252 rather than fixed in this scope:

- Repeated path-create set at the same parse-time-absent path emits a duplicate
  key (invalid TOML),
   because the resolver reads only the parse-time AST.
- Delete of an implicit dotted-key parent removes it from delta-aware reads but
  not from the serialized bytes.

The stateful property reparses between operations and stays on single top-level
segments,
 so it does not exercise these edges.

## Phase 7 differential oracle: attempted, then dropped

Phase 7 built a differential parser oracle that decoded fuzzer-generated and
mutator-corrupted documents through both our decoder and the pinned BurntSushi
Go reference decoder (`go:github.com/BurntSushi/toml/cmd/toml-test-decoder`,
v1.6.0),
 then classified the verdict pair:
 agreement,
 we-too-lax (the gold
signal),
 we-too-strict (logged),
 or a value divergence.
 A type-level comparator
normalized spec-equivalent spellings (offset-datetime instants at millisecond
resolution,
 numeric floats including `inf`/`nan`,
 `BigInt` integers,
 datetime
separator).
 The adapters ran in-process for our side and subprocessed the Go
binary for the reference;
 the test lived in `differential.expensive.unit.test.ts`
so the default `test:unit` and CI never spawned the external binary.

The oracle immediately surfaced a real divergence class,
 and on minimization it
was a defect in the reference,
 not in us.
 The BurntSushi v1.6.0 toml-test
decoder's tagged output loses data on an empty key (`""`) interacting with array
structure:

- `[ { "" = 1 }, "z" ]` decodes to `[ { "" = 1 } ]`:
   the `"z"` array element is
  silently dropped.
- `[ [ { "" = 1 } ] ]` decodes to `[ { "" = 1 } ]`:
   a nesting level collapses.
- `"" = [ [ {} ] ]` and `"" = [ [], {} ]` diverge as well.

A single-element `[ { "" = 1 } ]` and an empty key outside an array both agree,
and our parser keeps every element and nesting level,
 so our output is correct by
the TOML grammar (no reading drops `"z"`).
 A strip-and-recheck proof confirmed
empty-key was the sole structural cause:
 renaming the empty keys in two failing
counterexamples and re-decoding made both agree under the comparator.
 The fault
is in the reference decoder's tagged output (possibly its JSON marshaling of an
empty-string key),
 not in our parser.

Decision:
 drop BurntSushi entirely rather than maintain a growing allow-list or
input-exclusion against an oracle that has proven less stable than expected.
Empty keys appear in roughly 60 percent of generated documents,
 so excluding
them would gut the differential's coverage,
 and the bug has several structural
shapes,
 so a precise carve-out is brittle.
 The differential adapters,
 the
reference tool pin,
 and the `test:differential` task were removed.
 The TOML 1.0
and 1.1 conformance suite (phase 6) already exercises our decoder and encoder
against BurntSushi's curated corpus with zero failures and no allow-list,
 which
remains the parser-correctness oracle of record.
 A differential oracle could be
revisited against a different reference implementation if one proves stable.

## Coverage gate (phase 8)

The campaign measures its own reach with a deterministic V8 line-coverage gate,
so a future change that silently stops exercising a parser,
 emitter,
 or editor
branch fails the gate rather than passing green-but-weaker.

A reachability driver
(`package/toml-edit/core.fuzz/src/coverage-driver.ts`) imports the package
implementation from source (the `/ts` subpath),
 not the built artifact,
 and replays
the shared fuzz generators and committed corpus through every public entry point
and every `_` seam at a fixed fast-check seed and run count.
 Run under
`NODE_V8_COVERAGE`,
 that attributes coverage to the `src` files the gate watches.
The reader (`package/toml-edit/core.fuzz/src/coverage-v8.ts`) projects the raw
V8 block ranges to per-file covered-line counts:
 it paints a per-character bitmap
with the innermost range's count winning (the longest range painted first),
 then
counts a line covered when it holds a non-whitespace character at a covered
offset.
 Node v26's type stripping is position preserving,
 so a V8 range offset
indexes the on-disk `.ts` one-to-one.
 The gate
(`package/toml-edit/core.fuzz/src/coverage-report.ts`) compares per-file
covered-line counts against a committed baseline
(`package/toml-edit/core.fuzz/coverage-baseline.json`) and fails on any per-file
decrease.
 The `fuzz:coverage` task runs it;
 `--write` refreezes the baseline.
 The
operation spread,
 run-and-count harness,
 and edit machinery are split across
`coverage-exercise.ts`,
 `coverage-harness.ts`,
 `coverage-edits.ts`,
 and
`coverage-probes.ts` to stay under the max-lines budget.

Two design points are deliberate deviations worth recording:

- The gate measures `src`,
   not the shipped bundle.
   The property suite imports the
  built artifact by design (the import-boundary deviation above),
   but the bundle
  is minified,
   inlines its dependencies,
   and ships no source map,
   so V8 coverage
  of it cannot summarize per-`src`-file reachability.
   A separate source-importing
  driver is the only way to get the per-file reachability signal the plan asks
  for.
   The driver is a reachability harness,
   not an oracle:
   it asserts nothing,
  because the property suite owns correctness.
- The driver is a dedicated harness rather than the property files themselves.
  The property files use random seeds (a non-reproducible baseline) and import
  the bundle (the wrong surface),
   and the repository has no `development` or
  `source` export condition to remap them.
   A fixed-seed driver against source is
  reproducible and machine-independent,
   so the gate is bit-for-bit stable.

The driver's adequacy was validated empirically,
 not assumed:
 a one-off,
uncommitted `--import` resolve-hook ran the real property suite against source
under coverage,
 and a per-file covered-line diff confirmed the driver reaches a
superset of the suite (zero suite-only lines) after closing six gaps the diff
surfaced (the CRLF and overflow parse paths,
 the existing-node-aware encoders,
the pending-insertion and pending-edit projection arms,
 and the root-delete arm).
The covered-line set is saturated:
 it is identical across a second seed and a
2.5 times run count,
 so the committed baseline is a true high-water mark (this
was verified against one random property-suite run,
 which is sufficient given the
margin).
 The first baseline is 5287 covered lines across 39 target files.

The baseline is a V8-coverage artifact,
 and V8's block-coverage output is
node-version-bound.
 This repository deliberately tracks the latest node rather
than pinning,
 so a node release that shifts V8 coverage is handled by refreezing
the baseline with `mise run //package/toml-edit/core.fuzz:fuzz:coverage --write`,
the same maintenance the latest-node policy already implies;
 the gate is not a
reason to pin node.
 Line granularity absorbs minor releases in practice,
 so the
refreeze is expected only at a node major bump.
 The gate catches a regression in
reach,
 not new uncovered code:
 a freshly added `src` file is absent from the
baseline,
 so it does not fail the gate until the baseline is refrozen.

## Sidecar package (2026-09-06)

The campaign moved out of the runtime package into `package/toml-edit/core.fuzz`,
the layout `logger.fuzz`,
 `jsonc-edit.fuzz`,
 and `css-edit.fuzz` use:
the runtime package's `src` holds production code and its tests only,
 `fast-check` leaves its devDependencies,
and the generators,
 oracle,
 corpus loader,
 properties,
 run-budget tooling,
 coverage driver,
 projector,
 gate,
 and baseline live in the sidecar.
Property files keep importing the built artifact by package name;
the coverage driver and the oracle helpers import the runtime source through the `/ts` subpath instead of `../index.ts`,
the coverage report resolves the runtime package root through that subpath instead of walking up from its own file,
and the corpus loader's fixture and repository paths shrank by one directory.
The `fuzz` and `fuzz:coverage` tasks run as
`//package/toml-edit/core.fuzz:fuzz`
and `//package/toml-edit/core.fuzz:fuzz:coverage`;
`toml-edit-fuzz.yml` type-checks and unit-tests the sidecar alongside the runtime package and calls the sidecar tasks.
Every property file and the conformance suite passed in bounded,
 smoke,
 and gate modes after the move.

The baseline was refrozen at the move.
The repository-wide TSDoc reformat (`3d6c20c9f`,
 starless comment bodies) had already left it stale:
the projector counts any line with a non-whitespace character,
 comment lines included,
and the reformat turned every blank ` *` comment line into whitespace,
so each file's code-line total fell below its frozen covered count;
 the local gate reported every file regressed.
The two red `toml-edit-fuzz` runs since that commit never reached the gate:
they failed one step earlier,
 at conformance,
 because CI's newer mise printed a deprecation warning on every `node` shim call
and toml-test treats any stderr as a failed case
(`doc/troubleshooting/toml-edit-fuzz-workflow.md`,
 "Symptom:
 every valid and encoder case fails with a mise warning on stderr").
Every file's coverage percentage after the refreeze
matches or exceeds what the pre-reformat run reported at 100 percent,
so the drop was the whitespace artifact the "Coverage gate (phase 8)" section anticipates,
 not lost reach.

## Reusable fuzz-target checklist

Before calling a fuzz target's suite strong,
 answer all five.
 They are the
distilled diagnosis of why the original wrapper-only suite was green-but-weak,
and they generalize to the next target.

1. Is the tested layer where the logic and the bugs live,
    not a thin wrapper over
    it?
2. Are all public entry points and internal seams covered,
    the seams exposed
    through stable observability exports when needed?
3. Is every oracle stronger than no-crash or returns-a-string (semantic equality,
    round-trip,
    metamorphic,
    conformance,
    or a differential reference)?
4. Do the generators cover the full grammar and its boundary cases,
    with an
    independent encoder so the generator never reuses the code under test?
5. Are real corpus seeds,
    every discovered counterexample,
    and coverage feedback
    wired in,
    so a regression in reach fails rather than passing quietly?

## Deferred

- Mutation testing is available as an opt-in `mise run //package/toml-edit/core:test:mutation` task.
  It uses the repository's container-isolated mutation tester and the package unit suite;
  neither the fuzz sidecar nor the upstream conformance runner participates.
  The default source scan includes conformance adapters,
  whose surviving mutants need separate interpretation.
- A differential parser oracle against a stable reference implementation (not
  BurntSushi v1.6.0) remains possible future work.
