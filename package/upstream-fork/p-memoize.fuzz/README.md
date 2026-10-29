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

The same workloads drive upstream `p-memoize` 8.0.0
 (the dependency this fork
replaces) and the fork:
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
 and the retained synchronous-throw in-flight replay,
 are compared
directly as well.

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
