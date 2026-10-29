## module-pify-fork.fuzz

Property-based fuzz campaign for [`module-pify-fork`](../pify/README.md):
 fork-only invariants against an independent settlement model,
 plus a differential oracle against upstream [`pify`](https://github.com/sindresorhus/pify) 6.1.0.

### Properties

#### `pify.property.unit.test.ts`

Fork-only invariants:

- `forwards the args tuple before the callback and settles like the model`:
 generated wrapped-function specs run under generated flag slots,
 and the fork's settlement must equal the model's
 (upstream `pify`'s documented `multiArgs` / `errorFirst` interpretation with
 the spread-merge presence semantics),
 with the caller's tuple recorded exactly once before the callback.
- `pify wraps any object or function and rejects anything else with one error`:
 `anything()` inputs must either wrap or throw `InvalidInputError` with
 upstream's message text.

#### `differential.property.unit.test.ts`

Same generated spec on both implementations,
 outcomes compared structurally:

- `wrapped-function settlements match upstream pify`
- `module member selection and settlements match upstream pify`
- `sequential wraps over one target match upstream pify first-touch caching`
- `wrap failures match upstream pify messages`

### Running

```bash
# package/upstream-fork/pify.fuzz/mise.toml

# The property files as the normal unit suite (200 runs per property)
mise run //package/upstream-fork/pify.fuzz:test:unit

# Longer campaign (5000 runs per property)
mise run //package/upstream-fork/pify.fuzz:fuzz

# Campaign with an explicit run budget
mise run //package/upstream-fork/pify.fuzz:fuzz -- --runs 20000

# Coverage-reachability gate over the runtime package's src
mise run //package/upstream-fork/pify.fuzz:fuzz:coverage

# Refreeze the coverage baseline after intentional reachability changes
mise run //package/upstream-fork/pify.fuzz:fuzz:coverage -- --write
```

The run budget per property comes from `PIFY_FORK_FUZZ_RUNS`
 (`fuzz-budget.ts`),
 defaulting to `200` so the unit suite stays fast.

### Coverage gate

`coverage-driver.ts` drives every exported code path of
 [`module-pify-fork`](../pify/README.md) once under `NODE_V8_COVERAGE`,
 and `coverage-report.ts` projects that run to a covered-function count per
 runtime source file.
 The committed `coverage-baseline.json` freezes those counts;
 `fuzz:coverage` fails on any per-file regression and
`-- --write` refreezes.
 Counts,
 not percentages:
 a file whose reachable functions shrink fails even when another grows.

### Differential notes

The fork's call shape differs from upstream by design
 (`pify({ input, options })` and `member({ args })` versus
 `pify(input, options)` and `member(...args)`),
 so `pify-adapters.ts` maps one generated call through each shape before
 comparing outcomes.

The one documented deviation is the error class for invalid input:
 the fork throws `InvalidInputError`
 (a `TypeError` subclass)
 where upstream throws a bare `TypeError`.
 Wrap failures therefore compare by `caughtValueText` and TypeError-ness.

Three upstream quirks are compared verbatim rather than normalized away,
 because the fork reproduces them on purpose:

- the shared first-touch member-selection cache
 (a sequential-wraps property pins it),
- `Object.prototype` keys skipping include / exclude evaluation,
- explicitly `undefined` option values overriding defaults,
 including the member-selection crash from `{ exclude: undefined, }`.

Pattern values are rebuilt per implementation
 (`spec-fixtures.ts`),
 so a `RegExp`'s `lastIndex` state can never leak between the two runs and
 produce a false mismatch.
