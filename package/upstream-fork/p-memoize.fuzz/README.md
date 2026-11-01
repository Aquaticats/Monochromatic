## module-p-memoize-fork.fuzz

Property-based fuzz campaign for
[`@monochromatic-dev/upstream-fork-p-memoize`](../p-memoize/README.md).

Non-runtime sidecar,
 mirroring `css-edit.fuzz`:
the runtime package's `src` stays pure production code
while the workload generators,
 properties,
 and campaign tasks live here.

### Properties

#### Memoization invariants

On every generated workload
 (per-key resolve,
 reject,
 and sync-throw behaviors,
 cache and predicate modes,
 synchronous launch batches,
 interleaved `pMemoizeClear` runs)
every call settles with its key's behavior,
 concurrent calls with one key share
one promise,
 invocation ordinals count real runs from zero without gaps,
 and
cache writes carry only fulfilled resolve values.

#### Upstream differential oracle

The same workloads drive the installed upstream `p-memoize`
 (its catalog entry floats at `>=9.0.0`,
 so a newer release is picked up on the next install)
 and the fork:
 call outcomes,
 shared-promise grouping,
 invocation
and failure sequences,
 cache-contract traffic,
 final cache contents,
 and
write-predicate observations must all match.
 Clearing and validation error
messages,
 and in-flight sharing of a synchronous throw,
 are compared
directly as well.
 The workloads reach what the generator covers;
 the fork's own unit tests pin the rest of the 9.0.0 behavior
(re-entrant calls,
 failing cache methods,
 decorator stand-down).

### Running

```bash
# package/upstream-fork/p-memoize.fuzz/mise.toml

# Default campaign as part of the unit suite
mise run //package/upstream-fork/p-memoize.fuzz:test:unit

# Longer property campaign
mise run //package/upstream-fork/p-memoize.fuzz:fuzz -- --runs 5000

# Coverage-reachability gate over the runtime package's src
mise run //package/upstream-fork/p-memoize.fuzz:fuzz:coverage

# Refreeze the coverage baseline after intentional reachability changes
mise run //package/upstream-fork/p-memoize.fuzz:fuzz:coverage -- --write
```
