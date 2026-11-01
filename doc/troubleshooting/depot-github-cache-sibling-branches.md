# actions/cache v6.1.0 restores sibling-branch markers on Depot GitHub runners

## Symptom

Using the same key and cache version on sibling research branches produced different actual outcomes:
standard public GitHub runners missed the branch A cache from branch B,
while Depot GitHub runners restored its exact marker from branch B.
Both products first passed a separate same-branch restore control.
This matches the respective current documentation.
It is not an application failure or a cache client defect established by this experiment.

The GitHub sibling log emitted
`Cache not found for input keys: monochromatic-cache-scope-79189ea4841acfbebdeb-github`.
The Depot sibling log emitted
`Cache restored from key: monochromatic-cache-scope-79189ea4841acfbebdeb-depot`.
The [receipt](../audit/depot-cache-trust-scope-2026-10-18.evidence.json)
links the original ZIP bytes,
source bodies,
API metadata,
and marker hashes.

## Root cause and source boundary

[GitHub's current cache reference][github-cache] explicitly excludes caches created by sibling branches.
[Depot's current cache integration documentation][depot-cache] explicitly says it does not enforce branch isolation.
The probe observed those respective policies through actual marker consumers.
Neither provider's cache-server implementation was inspected.
The result does not independently establish cross-repository isolation,
fork permissions,
pull-request behavior,
or behavior of native Depot CI and BuildKit caches.

The client was pinned to `actions/cache` release `v6.1.0`,
commit `55cc8345863c7cc4c66a329aec7e433d2d1c52a9`.
Its source was cloned and read without modifying the third-party files.
Source bodies are retained in the linked gzip under `records["actions-cache/..."]`.

`restore/action.yml:33` selects the restore-only entry point:

```yaml
# actions/cache, restore/action.yml
runs:
  using: 'node24'
  main: '../dist/restore-only/index.js'
```

`src/restoreImpl.ts:45` passes the literal paths and keys to the toolkit:

```typescript
// actions/cache, src/restoreImpl.ts
const cacheKey = await cache.restoreCache(
    cachePaths,
    primaryKey,
    restoreKeys,
    { lookupOnly: lookupOnly },
    enableCrossOsArchive
);
```

The pinned bundled toolkit's `dist/restore-only/index.js:43408` computes the cache version from path inputs,
compression,
the Windows distinction where applicable,
and salt.
This client hash does not itself establish the backend's branch-access policy:

```javascript
// actions/cache, dist/restore-only/index.js
function getCacheVersion(paths, compressionMethod, enableCrossOsArchive = false) {
    const components = paths.slice();
    if (compressionMethod) {
        components.push(compressionMethod);
    }
    if (process.platform === 'win32' && !enableCrossOsArchive) {
        components.push('windows-only');
    }
    components.push(versionSalt);
    return external_crypto_namespaceObject.createHash('sha256').update(components.join('|')).digest('hex');
}
```

The source defines `versionSalt = '1.0'` at `dist/restore-only/index.js:43278`.
All six jobs observed `zstd` version `1.5.7`
and independently derived cache version
`d1b6c275328408c4360cae9ac893fcf5f899e752e5f442df4f1389e905cdadb0`.
`src/restoreImpl.ts:54` intentionally leaves `cache-hit` unset on a miss.
The GitHub sibling's empty output therefore agrees with the pinned implementation;
it is not treated as an exact hit or a failed action.

### Cache access mode and branch visibility

GitHub's current reference documents `cache-mode` as cache-operation access granted through scoped tokens:
`read` permits restore,
`write` permits restore and save,
`write-only` permits save,
and `none` permits neither.
When omitted,
trusted triggers such as this experiment's `workflow_dispatch` default to `write`;
low-trust triggers default to `read`.
The workflow omitted this key.
All three public GitHub setup logs explicitly reported `Cache mode: write`.
Depot setup logs did not report a mode.
The shell recorder's null mode is not substituted for the GitHub runner's explicit setup observation.

This comparison therefore measures sibling visibility under the observed defaults.
It does not test any explicit mode override.
The retained released `v6.1.0` restore-only and save-only bundles contain no `ACTIONS_CACHE_MODE` reference.
The current documentation describes token enforcement;
client-side skip behavior for alternate modes was not exercised.

GitHub also documents that other branches can restore a default-branch cache.
Repository-wide reuse through a default-branch seed is consequently a documented possibility.
This experiment seeded a nondefault branch,
not the default branch,
and does not establish that GitHub can never support cache sharing across repository branches.

## Verification

Both research refs identified identical workflow commit
`62de6a858d059b20a43c04f103c45dc8116e4f37`.
The literal cache path was `cache-scope-marker-79189ea4841acfbebdeb`.
Each provider used its own exact nonce key;
the key and path stayed unchanged across that provider's three runs.
No restore prefixes were supplied.
Restore jobs used `lookup-only: false` and the standalone restore action,
which has no cache-saving post step.
Every job verified an initially absent destination and absent application checkout.

Actual standard public GitHub controls were:

- Seed `39141719724`,
  job `117717347078`.
- Same-branch restore `39141763438`,
  job `117717469701`:
  exact-key hit and the expected 54-byte marker.
- Sibling restore `39141821223`,
  job `117717637456`:
  cache miss and marker absent.

Actual Depot GitHub controls were:

- Seed `39141909098`,
  job `117717886287`.
- Same-branch restore `39142023227`,
  job `117718221436`:
  exact-key hit and the expected 53-byte marker.
- Sibling restore `39142130086`,
  job `117718537533`:
  exact-key hit and the same expected 53-byte marker.

The GitHub marker SHA-256 was
`d8d6275ac443ff96981ac25193ded30fd84aac41a2613a005224cfdfc44a5303`.
The Depot marker SHA-256 was
`23ddb91abfb44c16158dbbf775ad6ce87eb8b583ae13b2979572cdf1bb218fd6`.
These hashes measure downloaded file bytes,
not merely the cache action's output flag.

The original dispatch command shape was:

```sh
# Historical owned research workflow invocation, before branch cleanup.
gh workflow run readonly-semantic-bridge.yml \
  --repo Aquaticats/Monochromatic \
  --ref research/depot-cache-scope-a-20261018 \
  --raw-field provider=github \
  --raw-field phase=seed
```

The owned refs were deleted on 2026-10-18 after terminal collection.
The exact workflow remains in source archive record `git/workflow-final.json`.
The invocation is a historical record,
not a currently available live branch.
A fresh cloud replay would require its own bounded admission and owned refs.

The actual inline recorder and cleanup code were also exercised locally in a disposable container:

```sh
# Executed local fake-cache controls, without network or application execution.
podman run --rm --network=none --cpus=2 --memory=2g --pids-limit=128 \
  --tmpfs /work:rw,size=32m --workdir=/work \
  --volume "${HOME}/temp/agent/depot-cache-trust-scope-20261018:/probe:ro,z" \
  docker.io/library/node:24-trixie-slim node /probe/local-preflight.mjs
```

The controls accepted seed recording,
same-branch exact restoration,
sibling misses,
and sibling exact restoration.
They rejected a missing same-branch marker and wrong restored bytes.
All local cleanup controls passed.
These are fake cache-input controls,
not additional cache-service executions or application tests.

## Verified workarounds and operational consequence

No workaround or settings change was required to observe the documented behavior.
The standalone restore action kept misses from silently creating another cache entry.
Its tradeoff is that an explicit save step is needed when a workflow intends to populate a cache.

Depot documents controlling isolation through key formatting.
This experiment held the key unchanged to measure sibling visibility.
It did not test a new branch-prefixed key policy,
fork permissions,
or a production dependency-cache migration.

## What does not work as evidence

- A cache miss without a working same-branch control would not establish branch isolation.
- Different cache versions or path inputs would confound the sibling comparison.
- `cache-hit` alone would not verify the expected downloaded bytes.
- A custom shell's absent runtime-token variable does not establish that a JavaScript action lacked authentication.
  Actual save and restore operations succeeded;
  the receipt labels shell-only environment observations explicitly.
- Branch deletion is not proof of cache-entry deletion.
  No cache purge was performed.
- Comparing these job durations would not establish a performance advantage.
  No repeated timing experiment was performed.

The first local container command returned Podman's
`workdir "/work" does not exist` before executing the probe.
An explicit disposable tmpfs supplied that directory.
The first local recorder then returned Node.js
`TypeError: Cannot read properties of undefined (reading 'trim')`
because optional `zstd` was absent.
Normalizing an absent stdout to an empty string corrected our recorder.
Both local prerequisite failures preceded every cloud allocation;
the corrected commit was identical on both live refs.

Original uploader logs retain Node.js warnings `DEP0040` and `DEP0169`.
The runner also reported that this pinned Node.js 20 uploader was forced to run on Node.js 24.
All jobs and API artifact digest checks passed;
the warnings were not suppressed or attributed to application behavior.

## Cleanup and budget boundary

All six jobs were sequential,
had two-minute job limits,
and finished successfully.
Each original job log confirms both owned temporary paths absent after cleanup.
Direct remote discovery confirmed both owned branches absent;
the local probe branch was deleted and the research worktree returned to its original branch.
Original artifacts and private controller source remain as evidence.

The twelve original ZIPs total `87,571` bytes,
with `66` members and `228,552` uncompressed member bytes.
CRC,
member hashes,
and all six artifact API size/digest joins passed.
No credential-pattern candidate was found.

Depot jobs have `25` seconds of observed job intervals at `USD 0.006/min`,
giving a nominal elapsed list-compute proxy of `USD 0.0025`.
Public standard GitHub compute is nominally zero.
The uniform two-CPU reference rounded-completed proxy is `USD 0.036`,
within the separately admitted `USD 0.10` guard.
No correction job was allocated and no reservation remains.
Actual VM billing intervals,
invoice charges,
and cache-storage charges remain unmeasured.

The benign cache entries were not deleted.
They may remain under existing policy.
Depot documents a default fourteen-day retention with configurable alternatives;
GitHub documents removal after more than seven days without access.
No account or organization settings were queried or changed,
and no expiry time was observed.

## Upstream filing decision

The repository's `.out-of-scope/` inventory and cache-related matches were checked;
none names this Actions/Depot behavior as an exemption.
Tracker searches covered open and closed issues and pull requests for branch scope.
The issue result was capped at thirty and is not an exhaustive absence claim.
Full bodies and comments were read for
[issue 1175][issue-1175],
[issue 1543][issue-1543],
and [pull request 307][pr-307].
They already discuss repository-wide cache requests and branch-scope documentation.
No additional upstream defect was established.

1.  Fault:
    the measured outcomes match each provider's published policy.
2.  Fixability:
    no defective client behavior or required fix was identified.
    Policy changes are a separate product decision.
3.  Supported use:
    both products document cache operations in GitHub Actions;
    the probe exercised ordinary `workflow_dispatch` jobs.
4.  Contribution policy:
    `CONTRIBUTING.md` describes contributions,
    but the pinned `README.md:367` says the repository is currently not taking contributions.
    `README.md:381` still welcomes bug reports.
5.  Maintainer disposition:
    issue 1543's maintainer comment describes branch scoping as intrinsic to the security model
    and points to a separate feedback discussion.
    It does not establish a refusal of every future policy option.
6.  Prototype:
    no upstream fix is appropriate for a documented behavior that this probe confirmed.
    The consumer-side bounded harness and positive controls are retained instead.

Nothing was filed or sent externally.
There is no additive bug report or patch draft to publish.

[github-cache]: https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching
[depot-cache]: https://depot.dev/docs/cache/integrations/github-actions
[issue-1175]: https://github.com/actions/cache/issues/1175
[issue-1543]: https://github.com/actions/cache/issues/1543
[pr-307]: https://github.com/actions/cache/pull/307
