# file-enforcer fuzzing

## Status

Accepted.
Plan:
 `~/.claude/plans/add-fuzzing-to-file-enforcer-calm-gem.md`.
Handover:
 `package/file-enforcer/core/HANDOVER.fuzzing.md`.

## Context

`package/file-enforcer/core` parses and transforms structured,
often untrusted input across several surfaces:
 flat JSON (`src/pipeline/json.ts`),
TOML wrappers (`src/pipeline/toml.ts`),
 XML attribute coding and entry
editing (`src/pipeline/xml-coding.ts`,
 `src/pipeline/xml.ts`),
 glob splitting
and mirroring (`src/io/glob-split.ts`,
 `src/io/glob-mirror.ts`),
 `mise registry`
line parsing (`src/package/registry-parse.ts`),
 and JetBrains product
directory names (`src/jetbrains/options-dir.ts`).

Example-based unit tests pin specific fixtures but do not exercise the wide
input space where a parser must never crash uncaught,
 must round-trip,
 and
must preserve a documented invariant.
 The recent regression history is about
exactly these input-handling edges.
 We wanted generative coverage that
surfaces unexpected or undocumented behavior,
 including in the TOML wrappers
even though they delegate to the monorepo-owned
`@monochromatic-dev/toml-edit-core`.

The first run paid for itself:
 the JSON properties found that
`mergeFlatJson`,
 `mergeObjectDefaults`,
 and `omitJsonKey` dropped a
`__proto__` key (bracket assignment routes it through the prototype setter),
and that `mergeObjectDefaults` clobbered an explicit `null` against its
documented "only where absent" contract.
 Both are fixed;
 see the handover.

## Decision

Use `fast-check` property tests,
 the established TypeScript precedent in this
repo (`package/test-harness/core/src/format-error.property.unit.test.ts`).
The test moved on 2026-10-04 to `package/test-harness/core-diagnostic/src/format-error.property.unit.test.ts`.
 The
Rust `cargo-fuzz` setup in `package/forbidden-strings/cli.fuzz` does not apply
to a TypeScript package.

One set of files,
 parameterized by environment,
 not two:

- Each target gets a co-located `*.property.unit.test.ts` that inlines its
  arbitraries and runs through the normal `module-test` harness.
- `src/fuzz-budget.ts` resolves the run plan from
  `FILE_ENFORCER_FUZZ_BUDGET_MS`.
   Unset gives bounded `numRuns` for the
  normal suite;
   a positive millisecond value gives effectively unbounded
  runs capped by `interruptAfterTimeLimit`,
   turning the same files into the
  deep campaign.
- The `fuzz` mise task sets that variable and runs every property file.

Supporting choices:

- Random seed in both modes,
   for the broadest coverage.
   A discovered
  counterexample is reproduced from the seed fast-check prints and pinned as
  a fast-check `examples` entry on the offending property.
- Runtime is node.
   node v26 runs the `.ts` files directly (native type
  stripping),
   the harness runs under it,
   and it exits non-zero on failure.
- Tests import targets from sibling source `.ts` (this package's existing
  test convention),
   which lets them reach internal helpers.
   Helpers needed
  only by tests (for example `parseVersionParts`,
   `compareVersionParts`) are
  exported at file level only and are never added to `src/index.ts`,
   so the
  public API stays unchanged.

## Rejected alternatives

- Example-based tests only.
   They pin known fixtures but miss the adversarial
  and boundary inputs that generative testing reaches;
   the proto and null
  defects above had sat undetected behind passing example tests.
- A separate `package/dev-script/file-enforcer.fuzz` package mirroring the Rust one.
   A
  cross-package consumer can only reach the public API,
   so the internal
  parsers would have to be promoted into `src/index.ts`,
   widening the public
  surface purely for testing.
   Co-located property files avoid that.
- A coverage-guided fuzzer (cargo-fuzz style).
   There is no mature,
   low-cost
  TypeScript equivalent;
   `fast-check` generation plus shrinking is the right
  tool for these pure,
   deterministic targets.
- Separate `*.fuzz.ts` modules feeding a standalone `mise.fuzz.ts` runner.
  Since fuzzing runs through the harness anyway,
   the extra files only
  duplicated definitions;
   the env-parameterized single file set is simpler
  and keeps each target's arbitraries beside its assertions.

## Counterexample handling

Inputs are synthetic values the generators build,
 and every fuzzed target is
a pure function,
 so no real filesystem content or secret can enter a
counterexample.
 There is no raw corpus on disk.
 When a property fails,
fast-check prints `{ seed, path }` and the shrunk counterexample;
 the fix
lands in source and the counterexample is added as a fast-check `examples`
entry on that property so the regression is locked in deterministically.

## Resource isolation and CI

The campaign is a single node process running `fast-check` over pure
functions,
 bounded by `interruptAfterTimeLimit` and small in memory (bounded
generated values plus a shrink path).
 It is not a coverage-guided native
fuzzer,
 so unlike the Rust libFuzzer package it does not risk exhausting the
host and is run directly rather than inside a bounded container.
 If a much
longer or parallel campaign is ever wanted,
 wrap the `fuzz` task in
`podman run --memory=2g --cpus=2` over the monorepo root.

CI integration is deferred.
 The property files already run in the normal
`test:unit` suite at bounded `numRuns`;
 the longer `fuzz` campaign is
on-demand.
