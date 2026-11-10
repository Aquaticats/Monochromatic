## module-matcher-fork.fuzz

Property-based fuzz campaign for
[`@monochromatic-dev/upstream-fork-matcher`](../matcher/README.md).

Non-runtime sidecar,
 mirroring `p-limit-fork.fuzz`:
the runtime package's `src` stays pure production code
while the arbitraries,
 oracles,
 properties,
 and campaign tasks live here.

### Properties

#### Single-pattern core

Every generated input-plus-pattern pair agrees with an independent
backtracking oracle:
 the fork's greedy first-position scan must match the
exhaustive search on every case.

#### Filter consistency

`matcher` keeps exactly the inputs `isMatch` accepts one at a time,
 filtering stays idempotent,
 and kept inputs appear in input order as a
subsequence of the original list with duplicates preserved.

#### Empty patterns

An empty pattern list matches nothing;
 the empty pattern matches only the
empty input.

#### Input totality

Any inputs-plus-patterns value pair yields a verdict or exactly one
validation error
(`InvalidInputsError` or `InvalidPatternsError`),
never another failure mode.

#### Upstream differential oracle

The same generated cases drive upstream `matcher` 7.0.0
 (the dependency
this fork replaces) and the fork:
 `matcher` verdicts,
 `isMatch` verdicts,
and validation error messages must all match.

### Running

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/upstream-fork/matcher.fuzz:<task>`.
The unit suite runs each property 200 times by default.

Property files import matcher-fork source through its `/ts` subpath,
 so no
build step is needed.

### Coverage gate

`fuzz:coverage` runs the deterministic `src/coverage-driver.ts` under
`NODE_V8_COVERAGE`
 and gates the covered-function count per runtime source file
against the frozen `coverage-baseline.json`,
 mirroring `p-limit-fork.fuzz`'s
gate.
It measures the runtime package's `src/` reachability,
 not this sidecar:
its `SOURCE_MARKER` targets `package/upstream-fork/matcher/src`.
The frozen
baseline reaches every function in every runtime file,
 so any change
that makes a function unreachable from the public API fails the check.
