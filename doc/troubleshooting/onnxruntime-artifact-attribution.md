# ONNX Runtime 1.30.0 CPU publication lookups leave build attribution unresolved

## Symptom

The full-policy Laya ONNX producer succeeded,
but that does not authenticate an ONNX consumer.
Task #34 is investigating CPU consumer routes without installing or executing their binaries.
The current observations are:

- Exact-subject GitHub attestation lookups for the Python wheel and Linux x64 CPU archive returned HTTP 404.
  The archive metadata query's `gh` diagnostic was `gh: Not Found (HTTP 404)`.
- Selected successful source-pin Linux CI,
  Linux CPU minimal,
  and Web CI runs currently list no artifacts.
  Their API requests returned 200.
- The official CPU archive and Python wheel contain same-length shared libraries with different SHA-256 values.
  This is an identity observation,
  not a runtime failure or evidence of malicious content.

The empty listings do not identify deletion or retention behavior.
The cause of the library-byte difference remains unknown.

## Evidence boundary and source trace

The ORT source pin is `f2c39fe2f838cf35ce7da92824f5a5e3ee6e88a7`.
The read-only clone is `~/temp/agent/onnxruntime-laya-2026-09-27`.
The retained [release metadata](https://api.github.com/repos/microsoft/onnxruntime/releases/tags/v1.30.0)
names `tianleiwu` as release author and uploader of the 10 listed platform archives.
It lists no separately named provenance or SBOM asset.
The local `git cat-file -t v1.30.0` result is `commit`,
so this is not an annotated signed-tag object.
GitHub's [commit endpoint](https://api.github.com/repos/microsoft/onnxruntime/commits/f2c39fe2f838cf35ce7da92824f5a5e3ee6e88a7)
reports `verification.verified: true` and `reason: valid`.
That is GitHub's commit-signature report,
not local signature verification or a binary-build attestation.

### The build-action checksum is not a complete build receipt

The separate read-only action clone is `~/temp/agent/onnxruntime-actions-laya-2026-09-27`,
revision `8bad63a3c05d448311dfa8e5f531171c97471aa1`,
tag `v0.0.12`.
Its `build-docker-image/dist/index.js.map` embeds `../../../src/build-docker-image/index.js`
and `../../../src/common/utils.js`.
The private source ledger binds that map,
the distributed bundle,
and inert extracted copies.
No bundle execution or independently reproduced bundle equivalence is claimed.

In extracted `consumer-source/actions/build-docker-image.js:148` to `200`,
`calculateChecksum` reads Dockerfile bytes,
normalized build arguments,
runner UID,
and context paths/content.
The source includes:

```js
// Embedded src/build-docker-image/index.js:154,160,167,169,196-197, separate excerpts
await hashFileContent(hash, dockerfilePath); // Use parameter
hash.update(normalizedArgs);
const uidString = `BUILD_UID=${uid}`; // Use parameter
hash.update(uidString);
hash.update(fileInfo.relative.replace(/\\/g, '/')); // Normalize path separators
await hashFileContent(hash, fileInfo.absolute);
```

The ordering in extracted `consumer-source/actions/build-docker-image.js:407` to `434`
calculates the checksum before the cache-miss branch can copy dependencies into context:

```js
// Embedded src/build-docker-image/index.js:407-414
const checksum = await calculateChecksum(
  config.hashAlgorithm,
  config.dockerfilePath,
  config.buildArgsInput,
  config.contextPath,
  uid // uid is already separate
);
const imageTag = `${config.hashAlgorithm}-${checksum}`;
```

```js
// Embedded src/build-docker-image/index.js:431,434, separate excerpts
await ensureDepsFile(config.contextPath); // Ensure deps copied before build
await buildImage(config, fullImageNameWithChecksumTag, uid); // <-- Pass UID
```

`ensureDepsFile` at lines 246 to 273 checks for an existing context file before this copy:

```js
// Embedded src/build-docker-image/index.js:252-253,262, separate excerpts
const dstDepsFile = path.join(contextPath, 'scripts', 'deps.txt');
const srcDepsFile = path.join(repoDir, 'cmake', 'deps.txt');
await fs.copyFile(srcDepsFile, dstDepsFile);
```

This source ordering limits what the checksum tag alone proves.
It is not a reproduced cache-correctness defect or an attribution mechanism for released ORT binaries.
The action was not run.
A denied anonymous request to one Azure base-image route also does not prove every source-build route needs it.

### CI defaults do not match the bounded CPU study

`proc_56d2` retained both complete reusable workflows and delegated action sources at private `4e0e7a2`.
The Linux wrapper's input at `.github/workflows/reusable_linux_build.yml:61` to `65`
sets a false upload default:

```yaml
# .github/workflows/reusable_linux_build.yml:61-65
upload_build_output:
  description: 'Whether to upload the build output directory as an artifact (used when tests are skipped)'
  required: false
  type: boolean
  default: false
```

Its upload condition is `if: inputs.upload_build_output == true` at line 208.
The Wasm wrapper's upload condition is:

```yaml
# .github/workflows/linux-wasm-ci-build-and-test-workflow.yml:178-179
- name: Upload WASM artifacts
  if: ${{ inputs.skip_publish != true }}
```

These source conditions do not establish why previously queried runs currently list no artifacts.
The Linux wrapper's permission declaration `attestations: write` at line 85
is not evidence that an attestation was emitted.

The embedded `run-build-script-in-docker/index.js:38` to `55` probes `nvidia-smi`,
independently of the requested execution-provider list.
Its command construction at lines 150 to 164 includes:

```js
// Embedded src/run-build-script-in-docker/index.js:157-160
'--build_shared_lib',
'--parallel',
'--use_vcpkg',
'--use_vcpkg_ms_internal_asset_cache',
```

The Docker argument construction at lines 215 to 255 includes:

```js
// Embedded src/run-build-script-in-docker/index.js:215-216,218-220, separate excerpts
const dockerArgs = ['run', '--rm'];
if (gpuAvailable) dockerArgs.push('--gpus', 'all');
dockerArgs.push('--volume', `${workspaceDir}:/onnxruntime_src`);
dockerArgs.push('--volume', `${runnerTempDir}:/onnxruntime_src/build`);
dockerArgs.push('--volume', `${hostCacheDir}:${containerHomeDir}/.cache`); // Use determined container home
```

That constructed argument list does not supply this study's memory,
CPU,
network,
read-only,
or no-host-mount boundaries.
Do not run this CI wrapper as the bounded CPU experiment.
This is a mismatch between study constraints and upstream CI defaults,
not an upstream defect finding.

The embedded `setup-build-tools/index.js:60` to `108` verifies SHA-512 after an archive download.
However,
`cacheTool` at lines 177 to 199 can return an existing tool-cache entry first:

```js
// Embedded src/setup-build-tools/index.js:179-183
const cachedDir = tc.find(toolName, version, archKey);
if (cachedDir) {
  core.info(`Found cached ${toolName} at: ${cachedDir}`);
  return cachedDir;
}
```

Requested download hashes therefore do not authenticate an unmeasured cache hit.
No cache corruption is inferred.
For vcpkg,
lines 157 to 174 and 310 to 315 select and execute a bootstrap script:

```js
// Embedded src/setup-build-tools/index.js:160-162,172,314, separate excerpts
const bootstrapScriptName = platform === 'win32' ? 'bootstrap-vcpkg.bat' : 'bootstrap-vcpkg.sh';
const bootstrapScriptPath = path.join(extractedPath, bootstrapScriptName);
const bootstrapArgs = ['-disableMetrics'];
await exec.exec(bootstrapScriptPath, bootstrapArgs, options);
await bootstrapVcpkg(extractedActual);
```

The bootstrap and tool-cache dependency command trees remain uninspected and unexecuted.
No action,
installer,
or source build was run.
A standalone source-build experiment would need separately inspected materials and an owned bounded launcher.

## Verification

Private evidence repository:
`~/temp/agent/laya-onnx-source-audit-2026-09-27`.

The frozen acquisition plan and controller are at `0081038`.
`proc_7524` downloaded exactly 11,306,877 bytes from the official release route,
following only GitHub and `release-assets.githubusercontent.com` origins.
The archive SHA-256 matched release metadata:
`a5ed5a3cac51fbb2e90da632ae43d19212faaa20e76484e62bcb7c23ddb3b3fd`.
Evidence is retained at `eacbbe7`.
No signed redirect query was persisted.

The hash-only reader and controller are at `6101efa`;
its input manifest,
raw process result,
and parsed result are at `739ede1`.
The executed command was:

```sh
# Private evidence repository; result files use create-new writes, so do not rerun in place.
mise --no-env --no-hooks run inspect:consumer-cpu-archive
```

`proc_ca7c` passed in frozen inspection image
`c42c66d018ce437903fa8bfd0b511623d9f7e97a3ce17ba6ca8aae8b2c832ecf`.
The controller supplied archive bytes through stdin,
without a host mount.
The reader used Python standard-library gzip,
tar,
and ZIP parsing,
with no disk extraction or candidate-library load.
It inventoried 41 tar members and 30,464,000 decompressed bytes.
The actual cgroup limits were two GiB memory,
zero added swap,
one CPU,
and 16 processes;
both descriptor limits were 64.
The container ceiling was 30 seconds.
These are inspection limits,
not model-inference measurements.

### Passing controls and observations

- Identical synthetic bytes produce equal identities through the shared hashing helper.
- A one-byte change at unchanged length produces unequal identities.
- The archive's outer digest matches the retained release metadata.
- The previously baked wheel's outer digest and shared-library identity match their prior ledgers.

### The equality hypothesis that failed

Both libraries have 28,985,152 bytes,
but their hashes differ:

- Archive member `onnxruntime-linux-x64-1.30.0/lib/libonnxruntime.so.1.30.0`:
  `245a6f8c38127551057a1cd1ffd59f0a186a227ade4f3492dea2494eb565542e`.
- Wheel member `onnxruntime/capi/libonnxruntime.so.1.30.0`:
  `c902c70b3003c0e99fada202f37478c515ae9bba7944c2b2abd0017bae0c82ed`.

The comparison therefore returned `identicalLibraries: false`.
The harness intentionally treats either equality observation as a valid result.
It did not identify the changed fields,
build cause,
ABI compatibility,
or runtime behavior.
The controls are not exhaustive archive-parser fuzzing or independent authorship.

## Verified workarounds

No execution-admitting workaround is established.
Keeping the archive inert permits these identity measurements without assuming its build is qualified.
The tradeoff is that numerical parity,
memory use,
and assessment timing remain unanswered.
SLSA is one possible attribution mechanism,
not the only allowed predicate;
source mapping and build-input review must still be satisfied by whatever evidence is used.

## What does not work

- A matching version or file length does not establish cross-channel byte identity.
- A release checksum or registry signature does not by itself establish all build inputs.
- A source commit's signature does not sign a subsequently produced binary.
- Empty CI artifact listings and route-specific 404/401 responses
  do not exhaust other publication or source-build routes.
- The Node installer must not be treated as a CPU-only default:
  the [qualification plan](../planning/pi-auto-mode-laya-qualification.md#cpu-consumer-publication-routes)
  records its Linux x64 CUDA selection.
  Only helper/proxy imports precede skip-flag handling,
  not provider downloads.

## Upstream filing decision

1.  Upstream fault is not established.
    Missing attribution evidence for this audit and unequal build bytes are not themselves defects.
2.  No corrective target is identified,
    so fixability remains unassessed.
3.  Support for the specific desired attribution contract remains unassessed.
4.  Contribution and AI-report policies were not evaluated for a filing.
5.  Maintainer intent was not evaluated;
    no willingness or refusal is inferred.
6.  No upstream patch was prototyped because the fault and corrective target are unestablished.

There is no issue draft or additive comment to file.
No duplicate-tracker search or upstream mutation was performed for this observation.
Reassess those gates if a concrete fault is reproduced.
