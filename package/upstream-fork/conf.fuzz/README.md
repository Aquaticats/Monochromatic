## module-conf-fork.fuzz

Property-based fuzz campaign for
[`@monochromatic-dev/upstream-fork-conf`](../conf/README.md).

Non-runtime sidecar,
 mirroring `p-limit-fork.fuzz`:
the runtime package's `src` stays pure production code
while the operation-sequence generators,
 properties,
 and campaign tasks live here.

### Properties

#### Store operation invariants

On every generated operation sequence
 (single and multi `set`,
`get`,
`has`,
`delete`,
`appendToArray`,
`reset`,
`clear`,
 and whole-store replacement,
with JSON-safe values,
 dotted and reserved keys,
 and prototype-shaped names)
through the fork:
 the store round-trips through the config file,
`size` equals the store's own key count,
 the reserved `__internal__` key never surfaces,
 defaults survive `clear` and `reset`,
 dot-notation writes read back,
`appendToArray` preserves order,
 and a set with an unwritable value throws
 without changing the store file.

#### Upstream differential oracle

The same generated operation sequences drive the vendored upstream `conf` snapshot
(`upstream-conf/`, commit `83e267178f`)
 (the
implementation this fork replaces) and the fork,
 each against its own disposable
temp directory:
 per-operation results
 (read values,
 probe answers,
 thrown error names
 and message texts),
 final store contents,
 item counts,
 and the config file's parsed `JSON`
 must all be identical.

### Running

```bash
# package/upstream-fork/conf.fuzz/mise.toml

# Cheap default budget (200 runs per property), as part of the unit suite
mise run //package/upstream-fork/conf.fuzz:test:unit

# Longer campaign
mise run //package/upstream-fork/conf.fuzz:fuzz -- --runs 20000

# Coverage-reachability gate (check against the frozen baseline)
mise run //package/upstream-fork/conf.fuzz:fuzz:coverage

# Refreeze the baseline after intentionally changing reachable runtime functions
mise run //package/upstream-fork/conf.fuzz:fuzz:coverage -- --write
```

Property files import conf-fork source through its `/ts` subpath,
 so no
build step is needed.

### Fixtures

Every run constructs its stores in disposable `mkdtemp` temp directories and
removes them afterwards,
 so the campaign never writes outside them.

### Coverage gate

`fuzz:coverage` runs the deterministic `src/coverage-driver.ts` under
`NODE_V8_COVERAGE`
 and gates the covered-function count per runtime source file
against the frozen `coverage-baseline.json`,
 mirroring `p-limit-fork.fuzz`'s
gate.
It measures the runtime package's `src/` reachability,
 not this sidecar:
its `SOURCE_MARKER` targets `package/upstream-fork/conf/src`.
 Test files
(`*.unit.test.ts`)
and the test-only `test-support.ts` are excluded from the gate
 (unreachable
from the public API,
 covered by their own unit tests and the mutation run).
The
driver exercises construction and option validation,
 the item surface and its
error paths,
 store replacement with bookkeeping preservation,
 change
subscriptions,
 schema validation,
 encryption round-trips,
 and migrations;
the frozen baseline records exactly what it reaches,
 so any change that makes
a covered function unreachable from the public API fails the check.

### Differential notes

Call shapes are normalized in the adapters:
 the fork's destructured-object
members
 (`get({ key, defaultValue })`,
`set({ key, value })`,
`set({ values })`,
`appendToArray({ key, value })`,
`reset({ keys })`)
are driven through upstream's positional forms
 (`get(key, defaultValue)`,
`set(key, value)`,
`set(values)`,
`appendToArray(key, value)`,
`reset(...keys)`)
by one shared operation vocabulary,
 per `package/upstream-fork/conf/README.md`,
"Call shapes".

Thrown error names are compared through the settled error-class map:
 the
fork's typed errors
 (`NonArrayValueError`,
`ReservedKeyError`,
 and friends)
carry upstream's message text verbatim
 and extend the same base class
 upstream
throws,
 so the oracle compares message texts and base classes exactly while
normalizing names back to upstream's
 (`TypeError`,
`Error`),
 per
`package/upstream-fork/conf/README.md`,
"Error types".
