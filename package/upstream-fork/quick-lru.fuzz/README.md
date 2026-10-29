## module-quick-lru-fork.fuzz

Property-based fuzz campaign for
[`@monochromatic-dev/upstream-fork-quick-lru`](../quick-lru/README.md).

Non-runtime sidecar,
 mirroring `p-limit-fork.fuzz`:
the runtime package's `src` stays pure production code
while the workload generators,
 properties,
 and campaign tasks live here.

### Properties

#### Cache invariants

On every generated workload
 (mixed stores and reads,
 per-item lifetime quirks,
 bound changes,
 evictions,
 clears,
 and clock advances)
each step's snapshot keeps the state relations intact:
`entries` matches `entriesAscending`,
 `entriesDescending` is `entriesAscending` reversed,
 every ordering carries the same live item count,
 `size` never exceeds `maxSize`,
 and `toString` agrees with both counters.

#### Input totality

Any constructor input yields a cache or exactly one configuration error
(`InvalidMaxSizeError`,
 `InvalidMaxAgeError`,
 or a plain `TypeError`),
never another failure mode.

#### Upstream differential oracle

The same workloads drive upstream `quick-lru` 7.3.0
 (the dependency this fork
replaces) and the fork under one shared fake clock:
 per-operation results,
 eviction events,
 and full state snapshots
 (including the `__oldCache` test hook) must all match.
Constructor failure texts must match on every junk input,
 and every member
descriptor must match upstream's class-prototype flags.

### Running

```bash
# package/upstream-fork/quick-lru.fuzz/mise.toml

# Cheap default budget (200 runs per property), as part of the unit suite
mise run //package/upstream-fork/quick-lru.fuzz:test:unit

# Longer campaign
mise run //package/upstream-fork/quick-lru.fuzz:fuzz -- --runs 20000

# Coverage-reachability gate (check against the frozen baseline)
mise run //package/upstream-fork/quick-lru.fuzz:fuzz:coverage

# Refreeze the baseline after intentionally changing reachable runtime functions
mise run //package/upstream-fork/quick-lru.fuzz:fuzz:coverage -- --write
```

Property files import quick-lru-fork source through its `/ts` subpath,
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
its `SOURCE_MARKER` targets `package/upstream-fork/quick-lru/src`.
The test-only `test-support.ts` is excluded from the gate
 (unreachable from the
public API,
 covered by its own unit test).
The frozen
baseline reaches every function in every gated runtime file,
 so any change
that makes a function unreachable from the public API fails the check.

### Differential notes

Both caches run under one controlled `Date.now`,
 rewound before each workload,
so expiry comparisons are exact rather than timing-based.
Member functions are
compared by descriptor shape and flags,
 since bound functions are never
reference-equal across implementations.
Stored item records are compared
through their `value` and `expiry` reads,
 so the fork's absent-expiry
encoding compares equal to upstream's present-but-undefined `expiry`.
