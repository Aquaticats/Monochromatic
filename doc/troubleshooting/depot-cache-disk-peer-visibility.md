# Archil v0.9.2 fan-out cache-disk consumer completes after 30 seconds

## Symptom

The standalone Depot CI persistent-disk probe
[run `vpp27ld5rx`](https://depot.dev/orgs/l4zncjjbj5/workflows/vpp27ld5rx)
finished successfully,
but its overlapping consumers had different operation intervals:
`reader-a` took 30,261 ms and `reader-b` took 139 ms.
Both checked the seed payload,
wrote only their own directories,
and verified the other writer's manifest and payload hash.
Their consumer intervals overlap.
Their manifest `publishedAt` fields differ by 764 ms,
but those fields are set before atomic manifest writes finish.
They do not measure publication-completion times.
No application error or payload corruption occurred.
The result rejects an assumption of immediately completed peer reads in this harness.

The [retained receipt](../audit/depot-persistent-cache-disk-probe-2026-10-18.evidence.json)
contains source bytes,
precise provider intervals,
consumer timestamps,
independent sandbox IDs,
real mount metadata,
artifacts,
and finite logs.
It does not contain per-filesystem-call timings.

## Root cause boundary

The exact cause of the slower operation is not established.
The observed interval includes seed reads,
own writes,
peer existence checks,
peer reads,
and scheduling.
It does not isolate a negative-lookup TTL,
directory-cache expiry,
or a blocking filesystem call.

The current [Archil consistency contract][consistency] distinguishes strongly consistent server reads
from client-side caches that can delay another client's changes for seconds.
The [client caching guide][client-caching] says file data,
attributes,
and directory listings are cached.
This makes delayed client visibility a relevant hypothesis,
not a diagnosed explanation of this precise 30-second observation.

The pinned `depot/cache-mount` implementation
`38d61fd25163e9ef3a3eff8a61676994d306e549`,
`src/index.ts:50` and `src/index.ts:51`,
removes conflicting mount modes and requests conditional mode:

```typescript
// depot/cache-mount/src/index.ts
const mountArgs = args.filter((arg) => arg !== '--shared' && arg !== '--conditional')
const cliArgs = ['--preserve-env=ARCHIL_MOUNT_TOKEN', ARCHIL_BIN, 'mount', '--conditional', ...mountArgs]
```

Its `src/post.ts:21` and `src/post.ts:26` order unmount before disk-token release:

```typescript
// depot/cache-mount/src/post.ts
await core.group('Unmounting disk', async () => {
  if (debug) core.info(`Unounting disk ${disk} from ${diskPath}`)
  await exec.exec('sudo', [ARCHIL_BIN, 'unmount', diskPath])
})

await core.group('Releasing disk token', async () => {
```

All retained post logs complete the unmount group,
then report token-deletion HTTP 200 and successful release.
There was no independent post-unmount mount-table probe.
The cleanup evidence cannot identify the consumer delay.

The actual Archil binary was preinstalled,
version `v0.9.2-1791935133`,
build `aabe187a`,
SHA256 `3bbf2fa696708c3ce4e8408ae557fc449db2692dee27fdb161281789d59d1bf4`.
Its hash was unchanged after mounting.
The public `archil-data` repository list exposed the JavaScript SDK,
Terraform provider,
and filesystem tests,
not this Linux client's implementation.
The source inspection therefore establishes the Depot invocation and cleanup ordering,
not Archil's cache-invalidation internals.

## Verification

The complete executed harness is retained in the receipt and pushed branch
`trial/depot-cache-disk-20261018T0503`,
commit `1b311a76458c77b5f38cd0029e0573c4a6a92768`.
Its workflow is `.depot/workflows/depot-cache-disk-probe.yml`.
Its inline Node task is `.depot/probe/cache-disk/mise.toml`.

To reproduce,
use that source in a disposable checkout,
replace the unique disk cohort with a fresh experiment name,
commit and push that source,
then dispatch through the current CLI:

```sh
# Disposable checkout containing the committed probe harness
depot ci run --workflow .depot/workflows/depot-cache-disk-probe.yml --org l4zncjjbj5 --repo Aquaticats/Monochromatic
```

The frozen workflow explicitly requests two CPUs/eight GB per job,
a five-minute timeout,
a 1 MiB seed,
and a fan-out limited to two readers.
The seed refuses an already populated fixture,
so rerunning against its original disk is not a valid fresh-disk reproduction.

### Cases that passed

- A completed producer and fresh mounts in separate sandboxes reproduced the seed's source binding and SHA256.
- Overlapping clients wrote disjoint paths and read each other's complete records.
- The payload and every writer's digest matched in both consumers.
- Artifact ZIP SHA256s matched provider upload logs.
- Action post groups completed and disk-token release returned HTTP 200.

### Assertions the evidence does not support

- Peer visibility or consumer completion always occurs within the 250 ms polling interval.
- This is a dependency-cache speed comparison or an estimate of run-to-run spread.
- Concurrent mutation of the same Cargo target,
  Gradle user home,
  lock file,
  or database is safe.
- A directory-cache TTL caused this particular delay.

## Verified consumer handling

The executed consumer uses a 45,000 ms polling deadline,
checks peer paths every 250 ms,
then verifies content hashes.
This completed both consumers in this run.
The tradeoff is waiting time and an explicit eventual failure when the peer does not become readable.
The job timeout provides the outer bound if a filesystem call itself blocks.

The executed `needs: seed` dependency also verified a completed producer followed by fresh mounts.
It is suitable evidence for that preparation boundary.
It does not prove the same mutable directory can be shared by arbitrary tools.

Archil documents `invalidate-cache` and `set-cache-expiry`,
but neither command was exercised by this probe.
They are investigation candidates rather than verified remedies here.

## What does not work

Inferring immediate peer visibility from a successful FUSE mount is insufficient.
The real mounts passed while one consumer still took 30,261 ms.
Inferring a cache-expiry root cause from that duration is also insufficient:
the harness did not time individual filesystem calls or run an invalidation control.

A private collector initially required an `@actions/exec` command-trace line.
Depot's finite logs exposed grouped post output but did not expose that trace.
The corrected validator checks completed ordered groups,
the actual HTTP response,
and release markers;
it does not invent an absent command log or a post-unmount mount-table assertion.

## Upstream filing decision

The `.out-of-scope/` filenames and contents were searched for Depot,
Archil,
and cache-related exemptions;
none applied to this observation.
The `depot/cache-mount` issues and pull requests were searched for `consistency`,
with no matching thread returned.
The Archil public organization listing did not expose the Linux client source or its issue tracker.

- Upstream fault is not established:
  the operation passed and the vendor documents client-cache visibility delays.
- Fixability is not assessed without an isolated cause and the deciding client implementation.
- Concurrent disk mounts are supported by the action's conditional mode and vendor documentation.
- Contribution eligibility is not assessed for an unidentified client-source repository.
- There is no demonstrated defect for which to assess upstream acceptance or rejection.
- No upstream patch was prototyped,
  because the prerequisite of an established upstream fault failed.

Nothing is fileable from this evidence.
The local diagnostic record and precise reproduction are the durable outcome;
no upstream message was sent.

[consistency]: https://docs.archil.com/details/consistency
[client-caching]: https://docs.archil.com/details/caching
