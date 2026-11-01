# Depot Developer pool cache research

## Purpose and status

This note supports the authorized expansion of CI using Depot alongside GitHub.
It separates reusable preparation from build caches,
test-result evidence,
and release artifacts.
It is research for the later interview,
not a production routing change or a subscription change.
The parent investigation owns provider-comparison experiments.
This note also records completed standalone disk and automatic Cache-identity probes.
Other probe designs remain unmeasured until their actual consumer runs complete.

Sources were checked on 2026-10-18.
The repository inspection ended at `bff874861fbae3f4c043714d43b2c4c948567122`;
concurrent work can advance `main` without changing this inspection boundary.
No account-wide usage,
invoice,
subscription,
cache deletion,
or production secret operation was performed.

## The cache products have different boundaries

### Actions archive caches

[Depot's Actions integration][actions-integration] explicitly limits its transparent Actions cache replacement
to Depot GitHub Actions runners.
An unchanged `actions/cache` invocation uses Depot Cache on those runners
and GitHub Cache on GitHub-hosted runners.
Matching keys and YAML therefore do not establish shared entries across the providers.
The same redirection covers setup actions that use the Actions cache API.

Depot scopes these entries to the repository,
but explicitly does not isolate branches.
Its default retention is 14 days with no configured size limit.
Available age policies are 7,
14,
and 30 days;
available size policies range from 25 GB to 500 GB or no limit.
These are documented defaults and options,
not the inspected account's settings.
[Depot Cache][cache-overview] prices storage beyond the included amount at USD $0.20 per GB,
using hourly snapshots averaged over the month.
No unlimited-free-storage conclusion follows from an unlimited retention setting.

[GitHub's current reference][github-cache] documents a default 10-GB repository allowance,
seven-day idle eviction,
and optional paid higher limits.
Its branch access rules differ from Depot's.
It also documents `cache-mode` and token-enforced permissions;
compatibility with Depot's replacement must be exercised rather than assumed.
The existing repository cache policy's 10-GB budget remains a local decision,
not evidence that GitHub offers no paid expansion.

### Remote compiler and task caches

[Depot's sccache integration][sccache] supports use from any CI provider or workstation.
It documents `SCCACHE_WEBDAV_ENDPOINT=https://cache.depot.dev`
and a compatible API token in `SCCACHE_WEBDAV_TOKEN`.
For user tokens with several organizations,
the documented alternative uses `SCCACHE_WEBDAV_USERNAME` for the organization ID
and `SCCACHE_WEBDAV_PASSWORD` for the token.
Rust additionally needs `RUSTC_WRAPPER=sccache`.

This enables a distinct research route:
free GitHub-hosted compute with Depot's remote compiler cache.
It does not redirect `actions/cache` or reuse an Actions archive automatically.
Keep compiler-cache comparisons separate from directory-archive comparisons.
Measure actual hits,
misses,
non-cacheable compilations,
and final consumer behavior.

[Authentication documentation][cache-auth] supports user and organization tokens,
explicitly excludes project tokens,
and gives Depot runners a job-lifetime `DEPOT_CACHE_TOKEN`.
No documented direct sccache OIDC exchange was established in the inspected sources.
An existing project-build authorization is insufficient evidence of remote-cache authorization.
The experiment must not create a broad credential to conceal this limit.

The standalone CI identity experiment in this note establishes an additional automatic route:
its fresh job received `DEPOT_CACHE_TOKEN`
and used that credential successfully for a small HTTP Cache write/read.
No new user token or organization token was provisioned.
The token's lifetime and untrusted-branch permissions were not independently audited.

[Depot's Gradle integration][gradle] also works outside Depot compute.
It requires build caching to be enabled and an `HttpBuildCache` pointing to Depot,
with compatible token credentials.
Depot runners supply an initialization script automatically.
Containerized Gradle or sccache builds still need credentials passed through their boundary;
host environment configuration does not implicitly enter a Docker build.

### Persistent cache disks

The official [`depot/cache-mount` README][cache-mount] says the action works only in standalone Depot CI.
It does not work on Depot's GitHub Actions runners or GitHub-hosted runners.
The namespace is organization-wide,
not repository-scoped.
Public fork pull requests receive an ordinary empty directory instead of a mount.
Disk data persists between jobs;
the action's post step does not delete it.

The inspected action revision is `38d61fd25163e9ef3a3eff8a61676994d306e549`.
`action.yml` requires `path` and `name`,
accepts `debug`,
and runs Node 24.
Names cannot contain whitespace.
Use a name identifying repository,
experiment,
trust level,
and format version.
The README recommends one writer with many readers,
or writes partitioned into separate directories.
Multi-write filesystem support does not prove that concurrent Cargo or Gradle mutations are safe.

In `src/index.ts:49`,
the action selects conditional mount mode:

```typescript
// depot/cache-mount, src/index.ts:50, revision 38d61fd25163e9ef3a3eff8a61676994d306e549
const mountArgs = args.filter((arg) => arg !== '--shared' && arg !== '--conditional')
const cliArgs = ['--preserve-env=ARCHIL_MOUNT_TOKEN', ARCHIL_BIN, 'mount', '--conditional', ...mountArgs]
```

`src/post.ts:21` first invokes `archil unmount`;
`src/post.ts:26` then releases the disk token through the sandbox metadata API.
An unmount failure prevents reaching that later block.
Preserve both post-step outcomes when claiming lifecycle cleanup.
The README documents automatic retention deletion,
defaulting to 14 days,
and manual deletion through Depot CI settings.
No action input for deleting disk data exists in this revision.

`src/index.ts:91` uses an installed `/usr/bin/archil` when present.
Otherwise it downloads and runs the upstream installer.
Pinning the action revision alone therefore does not identify the installed Archil executable.
Record the executable's version,
hash,
and preinstallation status in a probe.

### Prepared filesystems and container builders

[Custom images][custom-images] snapshot the Depot CI filesystem,
including installed packages,
SDKs,
workspace dependencies,
and pre-pulled container images.
They do not preserve process environment variables or running services.
The snapshot action is specific to standalone Depot CI.
An arbitrary Docker image is not interchangeable with a sandbox snapshot.

The [prepared desktop trial](../audit/depot-prepared-desktop-2026-10-16.md)
already verified separately allocated consumers of prepared files and source-bound executables.
Both consumers passed their 117 injectable interaction checks and cleanup assertions.
They still had to start the private user manager,
audio,
and compositor services.
The [snapshot investigation](../troubleshooting/depot-snapshot-mise-cache.md)
records PATH activation,
an unnecessary additional mise cache wait,
and a checkout/local-patch collision.

Keep preparation identity explicit:
base image and package versions,
toolchain identities,
installer policy,
lockfiles,
configuration,
and every included local artifact input.
Use immutable prepared versions and verify a manifest in each consumer.
If preserving dependencies in a pre-cloned workspace,
the documented `clean: false` checkout avoids deleting them;
it also preserves other untracked state.
Verify the actual source head and expected clean-state exceptions.
Publish experiment commits before consuming prepared checkouts that would otherwise receive a local add-file patch.

Depot container builders have another cache:
BuildKit image layers and mutable cache mounts.
That cache is separate from the Actions archive service,
persistent CI disks,
and snapshots.
Measure layer hits separately from compilation-cache hits and local artifact download time.

## Repository boundaries and invalidation inputs

The [CI caching policy](../decision/ci-caching.md)
already records fresh-job GitHub cache wins for selected Rust verification builds.
It also records a pnpm-store archive-cache comparison in the GitHub-hosted
`readonly-semantic-bridge.yml` jobs of run `38672732092` and its repeat.
On `ubuntu-latest`,
restoration plus installation took `7 + 8` seconds versus `8` to `9` seconds without restoration.
On `windows-latest`,
the boundary was `15 + 13` seconds versus `15.6` seconds.
On `macos-15`,
it was `37 + 8` seconds versus `14.7` seconds.
These are GitHub Actions archive restoration plus pnpm installation measurements,
not a general build-cache slowdown or separate measurements of the logger and TOML workflows.
Those measurements do not establish that a different Depot cache backend has the same result.
They make pnpm caching a separate hypothesis rather than an automatic improvement.

- Rust:
  current workflows distinguish OS,
  architecture,
  runner image,
  compiler identity,
  target triple,
  lockfile,
  profile,
  and feature set.
  Preserve those dimensions and relevant linker or `RUSTFLAGS` inputs.
  Keep downloadable crate content separate from mutable target directories.
  The release policy excludes shipped or attested outputs from caches.
- pnpm:
  the installed tree uses isolated workspace installation.
  `pnpm-workspace.yaml` prunes virtual-store orphans on installation
  and records optional-platform coverage for Linux and macOS,
  x64 and ARM64,
  glibc and musl.
  Include lockfile,
  pnpm version,
  workspace settings,
  dependency policies,
  package manifests,
  patches,
  Node identity,
  and native-package platform in install identity.
  A content store and a fully linked `node_modules` snapshot have different portability requirements.
- Linked JavaScript dependencies:
  a restored tree needs working workspace symlinks,
  native optional modules,
  installed build hooks,
  and the expected checkout layout.
  Verify actual imports from fresh consumers at both provider workspace roots.
  Do not infer portability merely because tar extraction succeeds.
- TypeScript and generated bundles:
  root `mise.toml:456` dispatches an existing built Node entry before its source fallback.
  The Oxlint preparation helper also compares source and output modification times.
  Source-bound manifests and changed-input runtime assertions are therefore required
  before treating restored generated output as current.
- Bun:
  it is a pinned task runtime/compiler island in root `mise.toml`,
  not this repository's package manager.
  Compiler or bytecode outputs need Bun identity,
  target,
  flags,
  and source inputs.
  A Bun package-download cache is not a substitute for the pnpm installation graph.
- Gradle and Android:
  Android enables `org.gradle.caching=true`
  and disables configuration caching in its `gradle.properties`.
  The inspected settings files do not configure a Depot remote cache.
  Record wrapper,
  JDK,
  plugin,
  build-script,
  SDK,
  NDK,
  ABI,
  and native-library inputs.
  A prepared SDK can avoid repeated installation;
  it does not prove a built APK or instrumented test is current.
- Browsers:
  `playwright.Dockerfile` binds the browser image to a digest
  and says its Playwright version must match the workspace dependency.
  It separately installs Bun through an installer.
  Preserve the final prepared image identity and actual tool versions,
  not only its browser base tag.
  Browser executables,
  native libraries,
  application bundles,
  fonts,
  and runtime services remain distinct dependencies.

The installed `node_modules/.modules.yaml` was measured,
not assumed:
it contains JSON,
`pnpm@12.11.2`,
`layoutVersion: 5`,
and `nodeLinker: isolated`.
The filename does not establish YAML field syntax,
and the ambient tree does not establish the frozen environment of a new CI job.

## Cargo correctness is a required control

The [Cargo/BuildKit investigation](../troubleshooting/cargo-buildkit-source-timestamps.md)
proved that an older changed-source timestamp could reuse a newer executable for different content.
The command succeeded while the runtime CLI-name assertion failed.
Separate cache namespaces did not prevent it.

The corrected fixture is retained at `ef616a2900269c52c61d24f816520504a3d74dd2`:
`package/music-player/desktop-app/depot-trial/Containerfile`
and `bake.hcl`.
It hashes copied source content,
cleans the local `music-player` and `truepeak-core` release packages on a mismatch,
and records the new hash only after successful compilation.
Registry dependency artifacts survive.
The tested compile command is `cargo build --release --locked --features alsa-fallback --jobs <N>`
inside the production Alpine toolchain;
the executable is checked inside the production runtime.
The archived task is `mise run //package/music-player/desktop-app:trial:github:sizing`.

Its [corrected matrix](../audit/depot-runner-sizing-2026-10-15.md)
passed all 100 runtime assertions,
including repeated changed-source snapshots.
These were repetitions on allocated runners,
not the missing distribution across independently allocated warm jobs.
Do not drop the content check to improve apparent cache performance.

## Proposed fresh-job experiment

### Archive comparison

Use independent GitHub-hosted and Depot-hosted Linux allocations
with the same frozen source,
toolchain,
container digest where used,
commands,
Cargo job count,
cache path inputs,
and consumer assertions.
Record actual CPU,
memory,
OS image,
workspace path,
runtime versions,
runner identity,
and allocation timestamps.
A provider label alone does not establish equal hardware.

Use a unique experiment prefix and fixture-owned directories;
do not reset or delete shared caches to obtain a cold observation.
For each provider,
create independent seed namespaces,
then restore each seed in a newly allocated job.
At least three independent allocations per unchanged condition give an observed spread;
retain individual samples rather than claim population reliability.
Capture a cold miss,
successful seed upload,
and actual fresh-job warm restoration.
An exact-hit output or a lookup-only hit is insufficient without restored files and exercised consumers.

Run changed-input controls:
an observable local source change,
a valid package-manager-generated dependency/lock change,
and a distinct compiler or linker identity.
Keep a separate expected hash for each fixture.
Exercise source state A,
state B,
and A again so an older cached source snapshot cannot silently reuse B's executable.
Keep a known wrong-output control to prove that the runtime assertion can detect stale reuse.
Do not manually edit a lockfile just to obtain a different key.

Dependency archive keys should identify compatible toolchain/platform inputs,
with the lock hash at the end for a controlled prefix fallback.
Local generated-output reuse additionally needs source identity and invalidation.
Log primary key,
matched key,
miss versus prefix versus exact hit,
cache service,
archive size,
restore/save intervals,
build intervals,
consumer intervals,
and all exit statuses.

### Saves when frozen tests fail

The pinned repository action is `actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae`.
Its `action.yml:44` uses `post-if: success()`;
its deprecated `save-always` input says that it does not work as intended.
Use explicit restore and save actions in the experiment.
`src/saveImpl.ts:65` calls the cache toolkit,
and its error handler can report a warning without failing the job.
A green save step therefore does not prove an entry exists.
Require the later fresh-job restore as the consumer verification.

Save only after successful dependency preparation or compilation,
with an explicit non-cancelled condition that still runs after unrelated known test failures.
For example,
the current Windows workflow already separates test compilation,
binary compilation,
cache pruning/saving,
and test execution in `.github/workflows/music-player-windows.yml:260`
(the production workflow moved into the `test-windows` job of `.github/workflows/music-player-release.yml`
on 2026-10-22, so that line number refers to the file before the move).
Preserve test failures and original consumer statuses;
do not turn the complete workflow green through `continue-on-error`.
Keep failed seed uploads and no-restoration consumers in the result ledger.

### Persistent disk and preparation probes

For a bounded disk probe,
mount an isolated disk in a seed job,
verify it is a real mount,
write a small content-addressed fixture and manifest atomically,
then read and hash it in independently allocated consumers.
Use separate per-job write directories for the fan-out control.
Require each consumer's private marker to be absent before writing it.
Record mount identity,
retained data identity,
read/write outcomes,
Archil identity,
and both cleanup operations.
A directory existing after the public-fork skip is not a successful mount.

For prepared images,
separate preparation time from consumer startup,
activation,
validation,
and live service readiness.
Verify unchanged consumers skip installation or compilation only when their manifests allow it.
A changed lock,
toolchain,
base package,
or local artifact input must select or build the corresponding prepared version.
Keep preparation jobs as explicit predecessors of artifact consumers,
with failure propagation and complete test reporting.

### Trust and cost interpretation

Cached executable content can affect later builds and tests.
Exclude credentials,
ephemeral runtime sockets,
and private service state.
Use trusted producers and prove the backend's writer permissions.
Branch names in cache keys prevent accidental collisions;
they do not prevent a malicious writer from choosing the trusted key.
Depot's absent branch isolation must be addressed at the actual authorization boundary
before privileged consumers share caches with untrusted code.

Caches are disposable acceleration inputs.
Artifacts are explicit outputs tied to source,
commands,
producer,
and digest.
Prepared images are versioned environments.
Persistent disks are mutable shared filesystems.
None of these alone is proof that a test executed or that a release artifact is trustworthy.

Report setup,
restore,
compile,
test,
save,
artifact upload/download,
and complete allocation intervals separately.
Charge failed and repeated allocations too.
Preserve all samples and compare provider differences against unchanged-input spread.
For overlapping jobs,
value the user's unfinished waiting interval once;
sum compute usage independently.
Include preparation amortization,
logical cache/archive sizes,
and storage retention in the model.
In-job directory sizes do not measure account-wide billable occupancy.
No invoice or complete monthly cost is inferred here.

## Persistent disk execution ledger

The bounded standalone Depot CI probe was dispatched as
[run `vpp27ld5rx`](https://depot.dev/orgs/l4zncjjbj5/workflows/vpp27ld5rx)
from pushed harness `1b311a76458c77b5f38cd0029e0573c4a6a92768`,
branch `trial/depot-cache-disk-20261018T0503`.
Its workflow hash is `d1b86d0e5b8df5657f7c85762087644fcfd8522213415edf2c7062674c9e7d95`.
The uniquely named disk is `monochromatic-disk-20261018T0503-trusted-v1`.
The workflow has a seed job and two fan-out consumers,
each requesting the documented two-CPU/eight-GB sandbox label
and a five-minute job timeout.
The payload is limited to 1 MiB plus consumer manifests and partitioned writer fixtures.
The workflow refuses the floating Archil installer path when the executable is absent.
All jobs finished successfully.
The [validated receipt](../audit/depot-persistent-cache-disk-probe-2026-10-18.evidence.json)
binds their different sandbox IDs,
source and workflow hashes,
real `fuse.archil` mounts,
absent private-job markers,
and artifact upload/download digests.
Both fan-out consumers read the seed and verified each other's disjoint writer payloads.
Archil was preinstalled as `v0.9.2-1791935133`,
build `aabe187a`,
with executable hash `3bbf2fa696708c3ce4e8408ae557fc449db2692dee27fdb161281789d59d1bf4`.
The hash remained unchanged after mounting;
no installer recovery was needed.

The seed's consumer operation took 126 ms.
The fan-out consumer operations took 30,261 ms and 139 ms.
Their consumer intervals overlap.
Their manifest `publishedAt` fields differ by 764 ms,
but the harness records those fields before each manifest's atomic write completes.
They are not publication-completion timestamps.
The slower consumer's delay has not been isolated causally.
Archil's [consistency documentation][archil-consistency]
explicitly says client caches can delay visibility after another client writes;
this is relevant context,
not proof of the precise mechanism in this run.

All post logs show the unmount group completing before disk-token deletion returned HTTP 200
and the release-success marker appeared.
The implementation awaits unmount before token release.
No separate mount-table probe ran after the action's post step,
so the receipt distinguishes source-ordered cleanup evidence from that stronger assertion.
A private validator initially expected a command-trace line that Depot did not expose;
the final validator checks the retained ordered groups and actual release response instead.

Precise provider attempt intervals total 111.712 seconds.
At the published two-CPU/eight-GB rate,
the timestamp-based gross compute estimate is USD $0.0111712,
before included usage and excluding storage.
The request-to-terminal interval is 106.353 seconds,
which counts the overlapping consumers once.
These values are neither an invoice nor a dependency-cache speed comparison.
The uniquely named fixture disk remains for provider retention;
no shared disk or retention setting was changed.

## Automatic remote Cache authorization ledger

Standalone CI
[run `tgf0n48bmp`](https://depot.dev/orgs/l4zncjjbj5/workflows/tgf0n48bmp)
ran committed and pushed harness `401f074072978217786371baf95d2925a9f752a0`.
The [validated identity receipt](../audit/depot-cache-job-identity-2026-10-18.evidence.json)
binds the independent sandbox,
workflow and task hashes,
provider job/attempt,
and artifact upload/download SHA256.

The job observed `DEPOT_CACHE_TOKEN`,
`SCCACHE_WEBDAV_TOKEN`,
and `SCCACHE_WEBDAV_ENDPOINT` already present.
It recorded only presence booleans,
never their values.
It used only `DEPOT_CACHE_TOKEN`,
with explicit organization-scoped Basic authentication,
to write a unique 144-byte fixture to `https://cache.depot.dev`.
The response was HTTP 201.
The subsequent HTTP 200 read reproduced exactly the same bytes and SHA256.
`DEPOT_TOKEN`,
local CLI credentials,
the Actions runtime token,
and project OIDC exchange were excluded from this request.

The inspected CLI release `v2.102.23`,
`pkg/cmd/cargo/cargo.go:55`,
calls `helpers.ResolveOrgAuth`.
In that release's `pkg/helpers/token.go:13`,
`ResolveOrgAuth` uses `ResolveStaticOrgAuth`,
whose final fallback is `resolveJITToken` at `pkg/helpers/token.go:129`:

```go
// depot/cli/pkg/helpers/token.go, release v2.102.23
func resolveJITToken() string {
  if token := os.Getenv("DEPOT_JIT_TOKEN"); token != "" {
    return token
  }

  if token := os.Getenv("DEPOT_CACHE_TOKEN"); token != "" {
    return token
  }

  return ""
}
```

`ResolveProjectAuth` separately attempts OIDC;
that exchange is not in `ResolveOrgAuth`.
This distinction explains why a successful project-build authorization alone was insufficient evidence,
while the real Cache credential and accepted request now establish the standalone route.
The same default sandbox had Cargo and rustc `1.97.1`,
but `sccache --version` failed to spawn with `ENOENT`.
An actual sccache compile therefore still requires installing or preparing that tool.
This tiny HTTP test is not a full Cargo or Gradle cache-hit measurement.

The allocation interval was 27.761 seconds.
At the published rate,
its gross timestamp-based compute estimate is USD $0.0027761.
The request-to-terminal interval was 43.029 seconds.
The initial local dispatch collector incorrectly assumed the CLI's default output was JSON.
The retained `Run: tgf0n48bmp` line recovered the already accepted run without redispatch.
All final metadata and artifact checks passed.

The proven result is automatic Cache authorization in this standalone CI job.
It does not establish direct OIDC exchange on ordinary GitHub-hosted compute,
the credential's exact lifetime,
fork-job write isolation,
or full compiler/task-cache compatibility.
The compiler-cache follow-up separately establishes cross-job Rust compiler-entry reuse.
Depot GitHub Actions runners have a separately documented automatic Cache credential;
ordinary GitHub compute still needs a valid documented credential route for this remote-cache design.

## Compiler-cache execution ledger

The bounded compiler-cache follow-up was dispatched as
[run `gmrm0ff71h`](https://depot.dev/orgs/l4zncjjbj5/workflows/gmrm0ff71h)
from pushed harness `298860fd9db1b894bba0f4531f2307df6881e0b0`.
Its workflow hash is `6b8056b6b6990e426b4cffa87f9bfb30a595a8e4d5f3b85591a05c3c4a09a00f`.
It permits at most three fresh allocations,
each requesting two CPUs/eight GB and a five-minute timeout.
It builds the existing `truepeak-core` default-feature library with its lockfile,
using fresh isolated Cargo target and package-download paths.
An independent executable calls the compiled library's public gain and meter functions.
Existing Rust tests run through Nextest only when that command is available.

The upstream `sccache` release `v0.18.0` was inspected before dispatch.
The 9,730,540-byte x86_64-musl archive matched both GitHub release metadata
and its published SHA256 file:
`45f1447fbe231e3037bde351ef70677dd212216c8d62ae7ca409fecc4d6acc89`.
The extracted executable hash is
`973cb15f6a986d84ca334bbed3bbe2eb8f1ee8fd81bf9e115b8539a293bf8d59`.
Its actual `--help` reports WebDAV support and JSON statistics.
The checked-out release source is `01c35e69b97f516c2ebfbdb418b5cd6277bde280`.

The probe's unique WebDAV prefix isolates its writes from existing cache entries.
The seed and unchanged consumer use the same source,
compiler,
paths,
and flags;
the third allocation adds a compiler flag as a cache-miss positive control.
The task uses only the supplied Cache credential,
records cache statistics and consumer behavior,
and preserves failures and server shutdown evidence.
All jobs completed successfully.
The [validated compiler-cache receipt](../audit/depot-sccache-fresh-jobs-2026-10-18.evidence.json)
binds the different sandbox IDs,
all source hashes,
compiler executable and version identity,
Cargo identity,
identical paths,
and artifact upload/download digests.
Each target directory and Cargo home was absent at job start.
The reported backend is WebDAV.

The seed recorded no Rust cache hits,
11 Rust misses,
and 11 successful cache writes.
The unchanged fresh allocation recorded 11 Rust hits,
no misses,
and no writes.
The changed-flag allocation recorded no Rust hits,
11 misses,
and 11 successful writes.
All reported read/write error counters were zero.
The task built the real library,
then compiled and executed an independent consumer in every allocation.
Both the public gain and one-sample meter assertions passed.
Server shutdown returned success in every job.

`cargo nextest --version` reported that the subcommand was unavailable.
No unit-test pass is claimed,
and no `cargo test` fallback was used.
The optional service/fingerprint features and shipped applications were not built by this probe.
The changed compiler flag is a positive invalidation control;
changed source,
dependency versions,
and compiler versions remain separate unexercised cases.

Precise allocation intervals were 26.897 seconds for the seed,
23.718 seconds for the unchanged consumer,
and 30.955 seconds for the changed flags.
Their total is 81.570 seconds,
giving a gross timestamp-based compute estimate of USD $0.008157 at the published rate.
The request-to-terminal interval was 72.764 seconds.
There is only one allocation per condition,
so these elapsed intervals do not establish unchanged-input spread or a speed multiplier.
The demonstrated benefit is 11 Rust compiler-cache hits in a fresh allocation with no Rust cache misses.
No target directory,
package archive,
or broad new credential supplied that reuse.

Each allocation reported 20 compiler requests,
eight non-cacheable requests,
and `compile_fails=1`,
while the outer Cargo build and independent consumer returned success.
The exact failed subprocess was not separately traced.
In release source `src/server.rs:1563` and `src/server.rs:1600`,
that counter increments for a failed compile result or a compiler-process error.
No specific probe command or application failure is inferred from the counter.
The receipt preserves it rather than treating cache-hit success as proof that every compiler request succeeded.
The [compiler-request diagnostic](../troubleshooting/sccache-cargo-request-failure-counter.md)
records that boundary and the unassigned subprocess identity.
The library archive hashes match between the seed and unchanged consumer;
the changed flags alter that archive's hash.
All independent consumer executable hashes match.

The local terminal collector initially combined the waiter's stderr progress with stdout JSON.
The retained final JSON was recovered from that combined capture without repeating any cloud job.
This was a collector stream-separation error,
not a failed CI job or a malformed stdout JSON response.

## Independent review of the pnpm archive comparison

The separate [pnpm experiment](depot-pnpm-store-fresh-job-research.md)
now has a completed fresh-job comparison.
Independent review checked its [retained evidence](../audit/depot-pnpm-fresh-jobs-2026-10-18.evidence.json)
against frozen Git source,
original job API receipts,
downloaded ZIP bytes and current GitHub artifact metadata.
All phases bind to source `2a155eec4935e5df225cb7a90af573357c3a94dc`.
The checks covered five source-file hashes,
16 ZIP digests,
67 member hashes,
224 step intervals and conclusions,
and every reported summary calculation.
Read-only artifact API requests at `2026-10-18T05:54:02Z` independently confirmed
the 16 provider digests and their source/run bindings.
Every install-bearing job passed its build and both existing consumer tasks.
The separate branch jobs checked archive access and manifests without installing or exercising application consumers.

For the measured workspace install,
GitHub restore plus install takes 11 to 12 seconds warm versus 9 to 10 seconds with the cache action disabled.
Depot takes 4 to 5 seconds warm versus 3 seconds disabled.
The pnpm subprocess saves time and every warm log records 582 reused packages with zero downloads;
archive restoration consumes that saving at the measured preparation boundary.
Leaving this exact pnpm archive disabled by default is supported by the observed samples.
The shorter Depot warm whole-job intervals also contain faster checkout before restoration,
so they do not isolate a pnpm-cache benefit.
Only two repetitions per comparative phase were collected,
with varying GitHub CPU models and unmatched provider images.
No provider-wide speed multiplier follows.

The review recomputed the timestamp-price proxy for all 13 retained Depot jobs,
including failed and cancelled harness attempts:
1,121 seconds and USD $0.2242 at the recorded label rate.
Allowance consumption,
allocation billing boundaries and storage-inclusive cost remain unverified.
Changed dependency/toolchain archive controls and untrusted-writer authorization remain separate unexercised cases.
The missing experimental GitHub entry establishes neither a deletion nor its disappearance's cause.
No additional cloud jobs or settings changes were needed for this review.

## Verification and retained fixtures

Scoped Markdown lint,
CommonMark rendering,
JSON parsing,
source and artifact digest checks,
and credential-pattern scans passed for the completed evidence.
The private experiment worktree was removed after a clean tracked-state check,
an ignored-file dry run,
and inspection for stray ignored Git-directory entries.
Only its generated forbidden-string cache was ignored;
no stray Git-directory entries existed.
The pushed branch and source bytes in the receipts preserve every harness.
The unique provider fixture disk,
small HTTP Cache entry,
and isolated compiler-cache prefix remain subject to provider retention.
No existing shared cache or account retention setting was changed.

## Sources and limits

Official docs and open source establish supported interfaces and documented boundaries.
Historical linked reports establish only their measured workloads.
The persistent-disk and automatic-authorization ledgers establish their specific exercised boundaries.
The independently reviewed archive comparison establishes only its frozen Linux workspace-install workload.
Snapshot designs
and broader remote compiler-cache designs require their corresponding consumer runs before adoption.
No overall cached speedup,
cross-provider shared archive,
safe untrusted-writer boundary,
or complete storage total was established by this note.

[actions-integration]: https://depot.dev/docs/cache/integrations/github-actions
[cache-overview]: https://depot.dev/docs/cache/overview
[cache-auth]: https://depot.dev/docs/cache/authentication
[sccache]: https://depot.dev/docs/cache/integrations/sccache
[gradle]: https://depot.dev/docs/cache/integrations/gradle
[custom-images]: https://depot.dev/docs/ci/how-to-guides/custom-images
[cache-mount]: https://github.com/depot/cache-mount
[github-cache]: https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching
[archil-consistency]: https://docs.archil.com/details/consistency
