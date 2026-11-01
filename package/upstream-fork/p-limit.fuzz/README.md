## module-p-limit-fork.fuzz

Property-based fuzz campaign for
[`@monochromatic-dev/upstream-fork-p-limit`](../p-limit/README.md).

Non-runtime sidecar,
 mirroring `css-edit.fuzz`:
the runtime package's `src` stays pure production code
while the workload generators,
 properties,
 and campaign tasks live here.

### Properties

#### Scheduling invariants

On every generated workload
 (mixed resolve,
 reject,
 and sync-throw calls,
 a
release-order permutation,
 interleaved bound changes and `clearQueue` runs)
started calls begin in FIFO order,
 every start observes
`activeCount <= concurrency`,
 each call settles with its specified outcome,
 and
both counters reach zero.

#### map order

Mapper results equal the sequential reference in input order under any bound.

#### limitFunction fidelity

Every forwarded call resolves with its function's result under any bound.

#### Input totality

Any constructor input yields a limiter or exactly one configuration error
(`InvalidConcurrencyError`,
 `InvalidRejectOnClearError`,
 or a plain `TypeError`),
never another failure mode.

#### Upstream differential oracle

The same workloads drive upstream `p-limit` 7.3.3
 (the dependency this fork
replaces) and the fork:
 start order,
 per-start counters,
 settlements,
`map` results and failures,
 constructor validation messages,
 and the attached
member property descriptors must all match.

### Running

```bash
# package/upstream-fork/p-limit.fuzz/mise.toml

# Cheap default budget (200 runs per property), as part of the unit suite
mise run //package/upstream-fork/p-limit.fuzz:test:unit

# Longer campaign
mise run //package/upstream-fork/p-limit.fuzz:fuzz -- --runs 20000

# Coverage-reachability gate (check against the frozen baseline)
mise run //package/upstream-fork/p-limit.fuzz:fuzz:coverage

# Refreeze the baseline after intentionally changing reachable runtime functions
mise run //package/upstream-fork/p-limit.fuzz:fuzz:coverage -- --write
```

Property files import p-limit-fork source through its `/ts` subpath,
 so no
build step is needed.

### Coverage gate

`fuzz:coverage` runs the deterministic `src/coverage-driver.ts` under
`NODE_V8_COVERAGE`
 and gates the covered-function count per runtime source file
against the frozen `coverage-baseline.json`,
 mirroring `jsonc-edit.fuzz`'s
gate.
It measures the runtime package's `src/` reachability,
 not this sidecar:
its `SOURCE_MARKER` targets `package/upstream-fork/p-limit/src`.
The test-only `test-support.ts` is excluded from the gate
 (unreachable from the
public API,
 covered by its own unit test and the mutation run).
The frozen
baseline reaches every function in every gated runtime file,
 so any change
that makes a function unreachable from the public API fails the check.

### Differential notes

Settle order across calls settling in the same microtask batch is a timing
artifact rather than a semantic difference,
 so the trace comparison sorts
per-call settlements by id and compares everything else exactly.
Member
descriptors are compared by shape and flags,
 since bound functions are never
reference-equal across implementations.
