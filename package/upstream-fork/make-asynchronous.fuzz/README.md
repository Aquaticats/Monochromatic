## module-make-asynchronous-fork.fuzz

Property-based fuzz campaign for
[`@monochromatic-dev/upstream-fork-make-asynchronous`](../make-asynchronous/README.md).

Non-runtime sidecar,
 mirroring `p-limit-fork.fuzz`:
the runtime package's `src` stays pure production code
while the workload generators,
 properties,
 and campaign tasks live here.

### Properties

#### Worker-source fidelity

Built worker bodies carry the serialized function source and the reply
protocol keys (`{id, output}`,
 `reportError`,
 `Symbol.asyncIterator`,
 the
iterator-result guard),
 and the shared templates contain the error reporter
and the Node.js preamble markers.

#### Single-call outcome fidelity

Every generated single-call spec (resolve,
 reject with an `Error`,
 throw a
non-error literal,
 across declared arities) settles through the fork with
its specified outcome.

#### Iteration fidelity

Every generated iterable spec drains through the fork in order,
 with a
trailing failure surfacing the fixture message and values before the
failure preserved.

#### Falsy protocol

Falsy resolved values resolve and falsy thrown values reject through
`getResult`,
 proving the reply protocol keys on `error`,
 not the value.

#### Member shape

`withSignal` stays enumerable,
 writable,
 and configurable on both wrapped
forms,
 matching upstream's plain assignment.

#### Call isolation

Concurrent generated calls resolve in order with their own payloads.

#### Upstream differential oracle

The same generated specs drive upstream `make-asynchronous` 2.1.0
 (the dependency this fork
replaces) and the fork:
 single-call settlements,
 iteration drains,
concurrent-call settlements,
 and the worker-source templates must all match.
Payloads stay structured-cloneable by construction,
 so the comparison
measures restoration fidelity rather than clone-failure handling.

### Running

```bash
# package/upstream-fork/make-asynchronous.fuzz/mise.toml

# Cheap default budget (200 runs per property), as part of the unit suite
mise run //package/upstream-fork/make-asynchronous.fuzz:test:unit

# Longer campaign
mise run //package/upstream-fork/make-asynchronous.fuzz:fuzz -- --runs 20000

# Coverage-reachability gate (check against the frozen baseline)
mise run //package/upstream-fork/make-asynchronous.fuzz:fuzz:coverage

# Refreeze the baseline after intentionally changing reachable runtime functions
mise run //package/upstream-fork/make-asynchronous.fuzz:fuzz:coverage -- --write
```

Property files import make-asynchronous-fork source through its `/ts`
subpath,
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
its `SOURCE_MARKER` targets `package/upstream-fork/make-asynchronous/src`.
The frozen
baseline reaches every function in every gated runtime file,
 so any change
that makes a function unreachable from the public API fails the check.

### Differential notes

Worker replies cross the structured-clone boundary,
 so the trace
comparison stringifies settlements and drains before comparing them
exactly.
Member
descriptors are compared by shape and flags,
 since bound functions are never
reference-equal across implementations.
