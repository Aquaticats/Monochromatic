## module-p-map-fork.fuzz

Property-based fuzz campaign for
[`@monochromatic-dev/upstream-fork-p-map`](../p-map/README.md).

Non-runtime sidecar,
 mirroring `css-edit.fuzz`:
the runtime package's `src` stays pure production code
while the workload generators,
 properties,
 and campaign tasks live here.

### Properties

#### Iteration invariants

On every generated workload
 (mixed resolve,
 reject,
 sync-throw,
 and skip mapper
calls,
 value or promise elements,
 sync or async sources,
 and a generated
release-order permutation)
started mapper calls begin in input order,
 every
start observes at most `concurrency` running calls,
 each call settles with
its specified outcome,
 and collected results follow input order minus skips.

#### Streaming invariants

The streaming variant yields mapper results in input order,
 keeps the
resolved-but-unconsumed backlog within `backpressure`,
 and surfaces exactly
one mapper failure through the async iterator.

#### Input totality

Any configuration input yields a run,
 a stream,
 or exactly one
configuration error
 (`InvalidInputError`,
 `MapperRequiredError`,
 `InvalidConcurrencyError`,
 `InvalidBackpressureError`,
 or a plain `TypeError`),
never another failure mode.

#### Upstream differential oracle

The same workloads drive upstream `p-map` 7.1.1
 (the dependency this fork
replaces) and the fork:
 mapper start order,
 per-start overlap,
 settlements,
 result order,
 skip handling,
 source pull and close counts,
 streaming yields,
 backpressure backlog,
 and configuration error messages must all match.

The `p-map` catalog entry floats (`>=7.1.1`), so the oracle moves with the
lockfile while the fork's source is ported by hand.
A parity failure after a lockfile refresh means upstream changed behavior:
read the upstream release notes and diff, then port the change into the fork
(or document the divergence in the fork's README and express it in the parity
test).
The fork was last aligned to `p-map` 7.1.1 for `pMap` and `pMapIterable`.

### Running

```bash
# package/upstream-fork/p-map.fuzz/mise.toml

# Cheap default budget (200 runs per property), as part of the unit suite
mise run //package/upstream-fork/p-map.fuzz:test:unit

# Longer campaign
mise run //package/upstream-fork/p-map.fuzz:fuzz -- --runs 20000

# Coverage-reachability gate (check against the frozen baseline)
mise run //package/upstream-fork/p-map.fuzz:fuzz:coverage

# Refreeze the baseline after intentionally changing reachable runtime functions
mise run //package/upstream-fork/p-map.fuzz:fuzz:coverage -- --write
```

Property files import p-map-fork source through its `/ts` subpath,
 so no
build step is needed.

### Coverage gate

`fuzz:coverage` runs the deterministic `src/coverage-driver.ts` under
`NODE_V8_COVERAGE`
 and gates the covered-function count per runtime source file
against the frozen `coverage-baseline.json`,
 mirroring `jsonc-edit.fuzz`'s
gate.
It measures the runtime package's `src/`,
 not this sidecar:
its
`SOURCE_MARKER` targets `package/upstream-fork/p-map/src`.
The test-only
`test-support.ts` is excluded from the gate
 (unreachable from the public API,
 covered by its own unit test and the mutation run).
The frozen baseline
reaches every function in every gated runtime file,
 so any change that makes
a function unreachable from the public API fails the check.

### Differential notes

Settle order across calls settling in the same microtask batch is a timing
artifact rather than a semantic difference,
 so the trace comparison sorts
nothing:
 workload runs are gate-controlled,
 and every ordering the
implementations produce is compared exactly.
Source pull and close counts
are compared exactly,
 because upstream `p-map`'s "never pull again once the
source reported `done`" rule and its detached close are part of the contract
this fork reproduces.
