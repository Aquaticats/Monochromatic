# Rolldown 1.2.3 inlining Satteri 0.9.5 relocates its native binding lookup

## Symptom

The package-local dependency-bundling prototype builds and type-checks,
but its JavaScript-only output fails during import:

```text
Cannot find native binding. npm has a bug related to optional dependencies ...
```

Satteri's retained cause chain identifies the actual failed lookups:

```text
Cannot find module '@bruits/satteri-linux-x64-gnu'
Cannot find module './satteri_napi.linux-x64-gnu.node'
```

Both use the emitted `assembly-orphan-trim-*.mjs` chunk as their require origin.
The failure precedes unit assertions.
Do not follow the generic npm-reinstallation advice for this measured relocation failure.

## Root cause

The default repository build keeps declared third-party packages external.
`package/config/rolldown/src/package-externals.ts:219`
constructs that list from dependency maps,
then removes names selected for bundling:

```ts
// package/config/rolldown/src/package-externals.ts
const externalNames = Object.keys({
  ...manifest.dependencies,
  ...manifest.peerDependencies,
}).filter(/* configured bundle-inclusion predicate */);
```

The prototype sets `nodeExternal({ alwaysBundle: ['**'] })` through the existing package-level override.
That moves Satteri's loader into an emitted chunk.
The installed `satteri@0.9.5` package's `index.js:6` establishes a file-relative require:

```js
// Installed satteri/index.js
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
```

Its Linux x64 GNU branch at `index.js:287` first tries the adjacent native asset,
then the optional package at `:292`:

```js
// Installed satteri/index.js, separate attempts
return require('./satteri_napi.linux-x64-gnu.node');
const binding = require('@bruits/satteri-linux-x64-gnu');
```

The JavaScript-only output contains neither the adjacent binary nor a resolvable optional package at that new origin.
The existing installed native package is present under Satteri's own dependency directory.
It is not missing from the installation.

## Version evidence

The actual JavaScript package and resolved native package are `0.9.5`.
Direct `createRequire` resolution from both the development and temporary worktrees reaches those same files.
The generated loader nevertheless contains a `0.8.1` version-check literal.
That literal does not identify the installed package version and is not the cause of the observed missing-file failure.
No version-enforcement override was supplied to these containers.

The readonly upstream clone is
`/var/home/user/temp/agent/satteri-runtime-source-20260913`,
commit `4171b78ae86ed815c7e6040715e96e49e7b74969`.
Its package is already `0.10.5`,
so it is architectural corroboration,
not the source of claims about installed `0.9.5` behavior.
`packages/satteri/src/binding.ts:39` forwards to generated `../index.js`;
`packages/satteri/patch-binding.js:3` locates that generated file:

```js
// Upstream packages/satteri/patch-binding.js
const bindingPath = new URL('./index.js', import.meta.url);
```

## Verification

Runtime is Node `26.8.2`,
Linux x64,
in no-network containers capped at 2 GiB RAM,
2 CPUs and 512 PIDs.
The experiment uses the current repository-owned `nodeConfig` override,
not a global configuration or dependency installation change.

Working catalog:

- The JavaScript-plus-native-asset copied artifact loads with no workspace or `node_modules` mount.
- It reproduces the actual 93-entry listing,
  90 eligible entries,
  272-parent population and frozen forty-parent membership.
- Its consumer matches all sixty-three supporting artifacts,
  their semantic roles,
  184 raw corpus documents and ninety archive-origin maps.
- Fetch and provider request counts remain zero.

Refused catalog:

- Original unbundled artifact with dependencies unavailable:
  `ERR_MODULE_NOT_FOUND` for `p-limit`.
- JavaScript-only bundled artifact:
  Satteri's missing native-binding cause chain.
- Copied native-asset artifact with only the `.node` file removed:
  the same native-binding failure.

Retained standalone evidence:

- `/var/home/user/temp/agent/runtime-closure-standalone-YOM2iV`:
  original dependency-absence control.
- `/var/home/user/temp/agent/runtime-closure-standalone-2Oy55F`:
  copied native-asset consumer.
- `/var/home/user/temp/agent/runtime-closure-standalone-ImrE1u`:
  native-asset removal control.

Each launch records copied file hashes,
mounts,
image identity,
environment and terminal status.
The successful consumer's peak container memory is 1675886592 bytes.
The matched-environment R7 full suite also passes build,
types and all 629 test-entry processes,
with `test:unit exit 0` inspected at line 10374 of its unit log.
All 192 copied R2 files match the R7 artifact by name and byte hash;
the comparator includes an actual one-byte-file-change positive control.
This is not yet a complete runtime-closure or adoption claim:
remaining loader paths and the final target/environment contract still require review.

## Verified remedy at the measured target

The prototype resolves the native package relative to the package's Satteri entry,
then emits its exact bytes as `satteri_napi.linux-x64-gnu.node` beside the generated JavaScript.
Rolldown's current `EmittedAsset` declaration supports a fixed `fileName` and binary `source`;
the build and copied-artifact consumer exercise that path.

The prototype helper lives only in the owned experiment worktree,
`package/module/translation-repair/src/build-native-runtime-assets.ts` (removed before 2026-10-18).
It also emits `runtime-imports.json` for later inspection.

Tradeoffs:

- This prototype targets the measured Linux x64 GNU runtime,
  not every supported Satteri platform.
- Native bytes must be included in the frozen runtime digest.
- Native-loader environment overrides remain separate execution inputs to reject or bind.
- System libraries and the Node binary remain separate from JavaScript dependency closure.

## Separate full-suite harness prerequisites

The native-asset build exposes test-environment failures distinct from missing parser assets:

- Temporary Git fixtures were created under the prototype repository,
  inheriting its unmounted trust configuration.
  Moving container `TMPDIR` to an independently mounted `/tmp` makes the ordinary fixture draws pass.
- A benchmark child deliberately keeps only `PATH` and `HOME`.
  Its Node process loses `LD_LIBRARY_PATH` and exits `127`
  because `libatomic.so.1` is unavailable on default search paths.
  A direct inherited-versus-minimal child probe reproduces `0` versus `127`.
- The missing-visual fixture expects an absent path inside a valid pinned corpus.
  With the corpus itself unavailable it reports `CorpusReadError` kind `other`
  and `resumable-failure`,
  not the expected missing-object stop.

The R7 full-suite harness supplies default-path libatomic,
independent `/tmp` and a readonly corpus at the temporary home's default corpus path.
An intermediate global clone-directory override changed the expected CLI defaults,
so it was removed rather than changing that assertion.
These harness corrections pass the full suite without changing provider behavior or relaxing assertions.

## Dedicated build follow-up

The separate `runtime:seal` task keeps normal build policy intact and writes candidates under package `node_modules`,
not published `dist/final` or `src`.
Its initial emitted-file verifier caught the declaration plugin reporting `.d.mts` files as chunks;
those are now excluded from executable identity.
The target/native/Node manifest is build evidence only.
Pre-import enforcement and location-independent repeated builds remain under verification.

The AST audit reconciles 790 static JavaScript edges and two literal Node dynamic imports
with 794 emitted edges:
the additional edges are declaration-only imports from `index.d.mts` and `roster-bench.d.mts`.
It retains every indirect loader observation for target/environment classification.
`readelf --dynamic` on the resolved native file names `libgcc_s.so.1`,
`libpthread.so.0`,
`libc.so.6` and `ld-linux-x86-64.so.2`.
This enumerates native dependencies;
it does not establish general operating-system compatibility.

## Podman launch controls before application import

This probe checks the host launch contract before the native input runner is implemented.
Podman `5.8.4` uses source commit `5431df23c742e5edea35bef34eed696f4db0106b`.
The independently selected base image ID is
`4251fdebd86463aba655f5069e9890cd3c99cf12e18479840888a74f7f60bab8`.
A disposable derived image adds a benign `NODE_OPTIONS` preload and a separate image entrypoint,
each writing only a marker into an owned output directory.
The derived image and seed container are removed by the probe's cleanup.
No application or model client is imported.

The first probe stops before container execution because it incorrectly expects image inspection's `Id`
to carry a `sha256:` prefix.
The measured field is a bare 64-character lowercase hexadecimal identifier.
Correcting that assertion allows the launch controls to run;
it is not evidence of an image-resolution failure.

### Environment and entrypoint are separate controls

`pkg/specgen/generate/container.go:218-243` clears default and image environment first,
then adds eligible host proxies and explicit environment:

```go
// Podman pkg/specgen/generate/container.go, separate excerpts
if s.UnsetEnvAll != nil && *s.UnsetEnvAll {
    defaultEnvs = make(map[string]string)
}
osEnv := envLib.Map(os.Environ())
if envHost {
    defaultEnvs = envLib.Join(defaultEnvs, osEnv)
} else if httpProxy {
    for _, envSpec := range config.ProxyEnv {
        if v, ok := osEnv[envSpec]; ok {
            defaultEnvs[envSpec] = v
        }
    }
}
s.Env = envLib.Join(defaultEnvs, s.Env)
```

`pkg/specgen/generate/oci.go:31-34` independently chooses the image entrypoint only when the caller supplies none:

```go
// Podman pkg/specgen/generate/oci.go
entrypoint := s.Entrypoint
if entrypoint == nil && imageData != nil {
    entrypoint = imageData.Config.Entrypoint
}
```

The tested command therefore combines `--unsetenv-all`,
`--http-proxy=false` and a fixed `--entrypoint`.
None substitutes for the others.

The working child still receives `HOSTNAME`.
`libpod/container_internal_linux.go:537-546` adds it after spec generation when the caller did not provide it:

```go
// Podman libpod/container_internal_linux.go
needEnv := true
for _, checkEnv := range g.Config.Process.Env {
    if strings.SplitN(checkEnv, "=", 2)[0] == "HOSTNAME" {
        needEnv = false
        break
    }
}
if needEnv {
    g.AddProcessEnv("HOSTNAME", hostname)
}
```

A child environment allowlist must account for that explicit platform behavior rather than assuming an empty map.

### Verified launch catalog

`preparation-launch-contract-probe-r3-20260913.out` and
`/home/user/temp/agent/preparation-launch-probe-SyU6K2/report.json` record the controls:

- Fixed entrypoint without environment clearing:
  the image preload executes before the inspection script,
  and the synthetic host proxy is inherited.
- `--unsetenv-all` alone:
  the preload is absent,
  but the synthetic host proxy is still inherited.
- `--unsetenv-all --http-proxy=false` with the fixed entrypoint:
  neither marker executes and neither loader nor proxy canary is present.
- Environment clearing without the entrypoint override:
  the separate image entrypoint marker executes.

The working child sees only `HOME`,
`HOSTNAME`,
`PATH` and `TMPDIR` environment names.
It reports Node `26.8.2`,
GNU libc `2.40`,
only loopback interfaces,
`memory.max=2147483648`,
`memory.swap.max=2147483648`,
`cpu.max=200000 100000` and `pids.max=512`.
Its UID/GID are the caller's `1000`,
and host-side stat confirms the created receipt belongs to that caller.
The mounted default-path libatomic hashes to
`b08060687ffb5768003b0c283cac5bddaa84ea5526d4d7bcb5994af98af5a130` inside the child.
These are observations of this image and launch,
not universal IDs or runtime values to hardcode into production.

The probe requests a read-only root,
read-only Node/library/probe mounts,
a private writable output mount,
no network,
dropped capabilities,
no new privileges,
no healthcheck or image-created volume,
and a bounded temporary filesystem.
Those flags must still be integrated and verified through the actual preparation CLI.

Clearing image defaults requires reintroducing a deliberate environment allowlist.
Disabling host proxy forwarding is appropriate for this no-network operation,
not a policy change for later live acquisition.
Overriding the image entrypoint makes the owning runner responsible for child lifecycle.
Child hash checks happen before application import,
not before Node,
the dynamic linker or its libraries execute;
the trusted host launch establishes their pre-start binding.
No upstream fault or installed-source change is established by these documented flag controls.

### Bind-mount encoding is CSV, not shell quoting

`pkg/specgenutilexternal/mount.go:15-21` admits exactly one CSV record:

```go
// Podman pkg/specgenutilexternal/mount.go
csvReader := csv.NewReader(strings.NewReader(input))
records, err := csvReader.ReadAll()
if err != nil {
    return "", nil, err
}
```

The rejection branch at `pkg/specgenutilexternal/mount.go:20` begins with `if len(records) != 1 {`.

`pkg/specgenutil/volumes.go:322` then separates each option only at its first equals sign:

```go
// Podman pkg/specgenutil/volumes.go
name, value, hasValue := strings.Cut(arg, "=")
```

The native probe mounts an owned file whose filename contains a comma,
colon,
quote,
equals sign and newline.
CSV quoting each whole field and doubling embedded quotes preserves the file identity.
The child reads its exact expected contents and prints `MOUNT_ENCODING_OK`.
`/home/user/temp/agent/preparation-mount-probe-gfyjLA/report.json` retains arguments and results.
This verifies that native `--mount` argument construction,
not a colon-joined `--volume` string or shell escaping,
covers the tested filename.

### Ambient host mounts remain separate inputs

Explicit bind flags alone do not prove that no ambient mount is added.
`pkg/specgenutil/volumes.go:298-307` parses configured mounts after command-line mounts,
ignoring duplicate destinations rather than discarding the configured inventory:

```go
// Podman pkg/specgenutil/volumes.go, separate excerpts
if err := parseMounts(mountFlag, false); err != nil {
    return nil, err
}
if err := parseMounts(configMounts, true); err != nil {
    return nil, fmt.Errorf("parsing containers.conf mounts: %w", err)
}
```

`vendor/go.podman.io/common/pkg/config/new.go:173-180` lets an explicit `CONTAINERS_CONF`
replace system and account configuration discovery:

```go
// Podman vendor/go.podman.io/common/pkg/config/new.go
if path := os.Getenv(containersConfEnv); path != "" {
    if err := fileutils.Exists(path); err != nil {
        return nil, fmt.Errorf("%s file: %w", containersConfEnv, err)
    }
    return append(configs, path), nil
}
```

The same file at `145-153` still applies `CONTAINERS_CONF_OVERRIDE` last:

```go
// Podman vendor/go.podman.io/common/pkg/config/new.go, excerpt
if path := os.Getenv(containersConfOverrideEnv); path != "" {
    if err := readConfigFromFile(path, config, true); err != nil {
        return nil, fmt.Errorf("reading %s config %q: %w", containersConfOverrideEnv, path, err)
    }
}
```

Subscription mounts have a separate source.
`vendor/go.podman.io/common/pkg/subscriptions/subscriptions.go:193-201` chooses ambient mount files
unless the explicit mount-file override is present:

```go
// Podman vendor/go.podman.io/common/pkg/subscriptions/subscriptions.go
if mountFile == "" {
    mountFiles = append(mountFiles, []string{OverrideMountsFile, DefaultMountsFile}...)
    if rootless {
        mountFiles = append([]string{UserOverrideMountsFile}, mountFiles...)
    }
} else {
    mountFiles = append(mountFiles, mountFile)
}
```

`cmd/podman/root.go:596` exposes `--default-mounts-file`.
The subscriptions implementation at `191-192` explicitly marks this hidden flag as testing-only.
Using it in a specialized runner is therefore a pinned implementation contract,
not a general upstream compatibility promise.

The subscriptions function at `216-220` also permits host FIPS mounts independently of that file:

```go
// Podman vendor/go.podman.io/common/pkg/subscriptions/subscriptions.go
if disableFips || !shouldAddFIPSMounts() {
    return subscriptionMounts
}
```

The probe checks host `/proc/sys/crypto/fips_enabled` is `0` before running.
It does not disable FIPS or establish a contract for FIPS-enabled hosts.
The generated configuration selects an owned empty hooks directory;
host hook discovery is not accepted as an unrecorded application input.

### Verified host-default catalog

The first disposable config control incorrectly uses volume-style syntax in `containers.conf.mounts`.
Podman exits `125`.
Its diagnostic starts with `Error: parsing containers.conf mounts:` and ends with `invalid mount option`.
The affected host file is
`/var/home/user/temp/agent/preparation-defaults-probe-Q4Dcmt/payload`,
encoded as a colon-joined volume string rather than a CSV mount record.
The complete diagnostic remains in `preparation-podman-defaults-probe-20260913.out`.

Changing that fixture to native mount syntax,
`type=bind,src=<owned-file>,target=/ambient-config,ro`,
allows the controls to execute.
This is a fixture syntax error,
not evidence that ambient mounts are absent or that Podman's parser is faulty.

`/var/home/user/temp/agent/preparation-defaults-probe-kR0hyS/report.json` records:

- A disposable `CONTAINERS_CONF_OVERRIDE` injects `/ambient-config` despite the explicit application mount flags.
- A disposable default mount file independently injects `/ambient-subscription`.
- Owned `CONTAINERS_CONF`,
  removal of `CONTAINERS_CONF_OVERRIDE`,
  an owned empty hooks directory and an explicit empty default mount file leave both markers absent.

The fixed mode also requests local Podman operation with `--remote=false`.
All probes use no network,
a read-only root,
2 GiB memory,
2 CPUs and 512 PIDs.
They do not import the preparation application or mutate installed configuration.
The retained scripts are runnable with the measured Node executable:

```sh
# Private reproduction harnesses; each creates fresh owned fixtures.
NODE=/var/home/user/.local/share/mise/installs/node/26.8.2/bin/node
ROOT=/var/home/user/temp/agent
"$NODE" "$ROOT/probe-preparation-mount-encoding-20260913.mts"
"$NODE" "$ROOT/probe-preparation-podman-defaults-20260913.mts"
```

The tradeoff is explicit ownership of these configuration inputs,
including the version-bound hidden flag and the measured non-FIPS profile.
The input runner must retain its generated control files and verify the actual launch;
these disposable controls alone are not the production pre-import gate.

### Native child resource and file-open follow-up

`input-child-context-FN03Xs/output/report.json` records a native Node `26.8.2` context probe,
not execution of the production bootstrap or built file-observation helper.
The probe uses the same pinned image and independently controlled host configuration.
It requests 2147483648 memory bytes and 4294967296 combined memory-plus-swap bytes.
Podman's `docs/source/markdown/options/memory-swap.md` defines the argument as:

```text
A limit value equal to memory plus swap.
```

The observed cgroup values are `memory.max=2147483648`,
`memory.swap.max=2147483648`,
`cpu.max=200000 100000` and `pids.max=512`.
The temporary filesystem reports 67108864 bytes.
`NoNewPrivs` is `1`,
and the inherited,
permitted,
effective,
bounding and ambient capability words are all zero.
Only `lo` is present.
These measurements establish this native invocation,
not that the new argument constructor or child owner has been exercised end to end.

The mount report also exposes a relevant image alias:
the requested `/lib64/libatomic.so.1` appears at `/usr/lib64/libatomic.so.1`.
Generated `/etc/passwd` and `/etc/group` mounts are separate platform inputs,
not immutable image files.
The report contains repeated `/sys/devices/virtual/powercap` masks.
Application role mounts remain unique;
a child checker must not silently resolve stacked application roles by record position.

The native file-open control writes a disposable no-writer FIFO and starts Node with
`O_RDONLY | O_NOFOLLOW`.
The child reaches `ENTER_OPEN` but does not reach the descriptor check before the test's 1000 ms deadline;
Node's synchronous child-process call reports `ETIMEDOUT` and `SIGTERM`.
This is an intentional blocking probe,
not a mutation-test assertion or an application crash.
Adding `O_NONBLOCK` lets `open` return and `fstat` report `regular:false` without a body read.
A regular-file positive control still reads its exact fixture text.
The FIFO is removed after the control.

Linux man-pages `6.19`,
[`open(2)`](https://man7.org/linux/man-pages/man2/open.2.html) and
[`fifo(7)`](https://man7.org/linux/man-pages/man7/fifo.7.html),
describe the deciding behavior:

```text
Opening the read or write end of a FIFO blocks until the other end
is also opened (by another process or thread).
```

```text
A process can open a FIFO in nonblocking mode.  In this case,
opening for read-only succeeds even if no one has opened on the
write side yet
```

The consumer correction in
`package/module/translation-repair/src/corpus-run/producer-input-file.ts:129-142` (removed before 2026-10-18)
adds an initial `lstat` refusal for known nonregular inputs,
then uses the descriptor flags and a fresh descriptor check:

```ts
// package/module/translation-repair/src/corpus-run/producer-input-file.ts (removed before 2026-10-18)
constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK
```

The initial pathname observation does not replace `fstat` or the pathname/descriptor identity comparison.
`O_NONBLOCK` addresses a FIFO introduced between metadata inspection and open.
It does not promise that regular-file or device I/O never waits;
`open(2)` explicitly notes that the flag currently has no effect on those I/O operations.
Built-helper race tests and actual bootstrap import-marker controls remain required.
No Node,
kernel or Podman patch or upstream filing is proposed for this consumer-side flag choice.

### Stop diagnostics and terminal state are independent

The native host lifecycle fixture `input-native-host-lifecycle-uj1yIB`
reaches SIGINT during an active application.
Podman exits `0` from the stop command but writes:

```text
StopSignal SIGTERM failed to stop container producer-input-f3cc837a-5009-4904-aaa8-8ef219be09b1 in 5 seconds, resorting to SIGKILL
```

The frozen bootstrap `9860eb668e5e446df6dfae51f53bf8fb1d9e6041bfabbe58be88b6fa27852180`
refuses that nonempty native stderr,
then skips `inspect-stopped`.
Its terminal record therefore says `unconfirmed`,
and `cleanup-complete.json` is absent.
The CLI still preserves SIGINT status `130`.
The preceding normal-output and output-rejection fixtures passed;
the lifecycle suite stops at this cancellation case and is not a passing suite.

Podman `v5.8.4`,
commit `5431df23c742e5edea35bef34eed696f4db0106b`,
emits the warning at `libpod/oci_conmon_common.go:402`,
then independently sends SIGKILL and waits for stopping at lines `409` to `423`:

```go
// libpod/oci_conmon_common.go, excerpt after the warning
stopped, err := killCtr(uint(unix.SIGKILL))
if err != nil {
    return fmt.Errorf("sending SIGKILL to container %s: %w", ctr.ID(), err)
}
if stopped {
    return nil
}
```

The warning is not a claim that the container remains running.
The consumer's defect is using command refusal to suppress independent state observation.
No cause for the fixture's failure to exit on SIGTERM is inferred from this warning alone.
The child CLI does not install the host's `producerInputSignals` listeners.

The local change retains strict command diagnostics,
settles the stop attempt before a fresh exact-ID inspection,
and synchronizes both outcomes in `stop-observation.json`.
Only validated nonrunning state can permit removal.
A failed or running inspection still withholds removal;
stop refusal and interruption cannot become successful execution.
Rebuilt frozen bootstrap `7903446442925e7d91a0be0d04053f1d02a63c9bd4c588f9ca1fa2057ebb59b8`
passes `input-native-host-lifecycle-xeXdpS`.
SIGINT and SIGTERM fixtures explicitly ignore child SIGTERM;
the native warning remains recorded,
fresh exited state permits removal,
and cleanup plus checked absence coexist with statuses `130` and `143`.
Late SIGINT also retains complete artifact files while returning `130`.
No warning filtering or Podman patch is proposed.

### Node close observation must survive an earlier error

Node `v26.8.2`,
commit `f2f2c2f246c36bd74f082cb43ecfe830657d81c9`,
implements `events.once` at `lib/events.js:1001`.
Its error listener removes the requested event listener before rejecting:

```js
// lib/events.js:1008-1013
const errorListener = (err) => {
  emitter.removeListener(name, resolver);
  if (signal != null) {
    eventTargetAgnosticRemoveListener(signal, 'abort', abortListener);
  }
  reject(err);
};
```

For EventEmitters,
`lib/events.js:1028-1031` installs this listener for every requested event except `error`.
Thus `once(child, 'close')` does not guarantee observing close after a native spawn error.

`input-native-host-lifecycle-g790sq` exercises actual `ENOENT` spawning,
with a trusted fixture delaying delivery of its close event.
The frozen bootstrap writes `image.exit.json` before that close is delivered.
The ordinary ordering assertion fails with `true !== false`.
The fixture's nonexistent executable path stays absent from the CLI's names-only diagnostic.
This is a consumer ordering defect,
not a Node bug.

The local observer now records only fixed error categories and resolves on actual close.
The command owner then synchronizes stdout,
stderr and exit evidence before throwing a native-error refusal.
The same delayed-close test passes against rebuilt frozen bootstrap
`7903446442925e7d91a0be0d04053f1d02a63c9bd4c588f9ca1fa2057ebb59b8`:
the exit record is absent immediately before close is delivered,
then native-error refusal exits `6`.
A separate timed native-client fixture verifies deadline escalation to SIGKILL without creating a container.
The event delay is test instrumentation;
no claim is made that every real spawn failure exhibits that delay.

### OCI-hook isolation has a positive control

`input-native-hook-isolation-gSqIgO` registers an owned `precreate` hook through
`CONTAINERS_CONF_OVERRIDE`.
A direct bounded Podman invocation executes it and writes its marker.
After that marker is removed,
the specialized frozen CLI reconstructs a fixture artifact with the same ambient override,
without executing the hook.
Both invocations exit `0`;
both containers are removed and independently checked absent.
This is hook/configuration isolation with a fixture application,
not corpus reconstruction or a claim that the host cannot be modified.

The pinned vendored hook matcher at `pkg/hooks/1.0.0/when.go:29-37`
counts an enabled `always` condition as matching.
The owned fixture uses no pattern-dependent selection:

```json
{
  "version": "1.0.0",
  "when": { "always": true },
  "stages": ["precreate"]
}
```

This excerpt omits the separately recorded hook executable and argv,
not required fields from the actual fixture.
Podman's `docs/source/markdown/podman.1.md:75` describes the extension contract:

> precreate hooks receive the proposed runtime configuration on their standard input.

The fixture passes that JSON through unchanged and only writes its private marker.
The full schema was read from `common/v0.67.1` in
`podman-container-tools/container-libs/common/pkg/hooks/docs/oci-hooks.5.md`,
matching the common version in Podman's `go.mod:69`.
The initial `containers/common` and `podman/common` URLs returned `404`;
the `go.podman.io/common?go-get=1` repository mapping identified the owning repository.

### Container removal also removes its CID file

The first retention-control harness incorrectly read `container.id` after native removal.
It fails with `ENOENT` in the absence-diagnostics fixture,
not a production cleanup assertion.
The corrected harness reads the already synchronized terminal record's container ID after removal.
Before removal it continues to use the fixture-owned creation ID.

Podman `libpod/runtime_ctr.go:1020-1025` explicitly removes its annotated CID file:

```go
// libpod/runtime_ctr.go
if cidFile, ok := c.config.Spec.Annotations[define.InspectAnnotationCIDFile]; ok {
    if err := os.Remove(cidFile); err != nil && !errors.Is(err, os.ErrNotExist) {
        reportErrorf("cleaning up CID file: %w", err)
    }
}
```

The corrected retention suite completes in `input-native-retention-controls-r2-20260913.out`.
Its controls retain containers after unsuccessful independent inspection,
stop/terminal-record collisions,
removal-command collision and ambiguous creation.
Successful removal followed by refused absence evidence still withholds `cleanup-complete.json`.
All remaining fixture containers are independently inspected and removed by the verification owner.
No production CID-file workaround or upstream change is needed.

### CLI fixture uses the existing asynchronous process utility

The CLI fixture's native callback adapter passes behavior tests,
but the full linter reports `promise/prefer-await-to-callbacks`.
The replacement uses the package's already declared `nano-spawn` dependency,
not a new process framework or a suppression.
Builds,
types and `devtest-RNODLX` pass with actual exit-zero,
exit-six,
`ENOENT`,
`SIGTERM` and parent-environment-canary controls.
The native signal and spawn failures also assert `isCanceled: false`,
so deadline cancellation cannot impersonate those cases.

Nano-spawn `v2.1.0`,
commit `cc231e2c7b1e434a96f25f907ca2cb2f7c596e90`,
merges environment values in `source/options.js:16`:

```js
// source/options.js
const env = envOption ? {...process.env, ...envOption} : undefined;
```

The fixture therefore enumerates parent environment names,
assigns Node's omitted-entry value to them,
then adds only its explicit fixture variables.
It never copies parent values into the child environment.
Node `v26.8.2`,
`lib/child_process.js:748-754`,
omits those entries when serializing native environment pairs:

```js
// lib/child_process.js
for (const key of envKeys) {
  const value = env[key];
  if (value !== undefined) {
    validateArgumentNullCheck(key, `options.env['${key}']`);
    validateArgumentNullCheck(value, `options.env['${key}']`);
    ArrayPrototypePush(envPairs, `${key}=${value}`);
  }
}
```

The library's `source/result.js:76-78` also removes one final newline from captured streams.
Fixture result documentation now states that normalization;
these strings are not byte-preserving artifact evidence.
Application file identities remain verified separately by the production runner.
The installed `options.js` and `result.js` were compared with the release source.
No dependency installation,
lockfile edit,
upstream patch or filing is part of this change.

### Frozen verification drivers use JavaScript extensions

The first frozen native-verification driver used a `.mts` filename under package `node_modules`.
Node `v26.8.2` refused it before any case executed:

```text
ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING
```

At `lib/internal/modules/typescript.js:152-155`,
the inspected Node source rejects that placement:

```js
// lib/internal/modules/typescript.js
function stripTypeScriptModuleTypes(source, filename, sourceURL) {
  assert(typeof source === 'string');
  if (isUnderNodeModules(filename)) {
    throw new ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING(filename);
  }
```

The drivers contain JavaScript syntax,
so newly frozen `.mjs` copies run without a type-stripping step.
The failed frozen directory is not rewritten.
`.frozen-input-native-checks-EjQapl` completes all current child,
host,
retention and hook cases.
This is a fixture packaging correction,
not a production bootstrap or Node patch.

### Pre-start phase fixtures distinguish image inspection

The first added pre-start fixture treats every `inspect` argument before terminal inspection as created-container inspection.
Podman's image-inspection command also contains that token.
The fixture therefore tries to read `inspect-created.stdout` before it exists and receives `ENOENT`.
This is a fixture-stage selection failure,
not evidence that the bootstrap bypasses a startup gate.

The corrected fixture also requires the exact `inspect-created.command.json` record.
`input-final-startup-signals-wUUrF1` then passes positive startup,
created-metadata mismatch,
pre-start bootstrap drift and interruption during create,
remove and absence verification.
A positive application marker is observed only after permitted startup.
Failed pre-start cases have no start command or application marker.
The fixture's modified bootstrap is its own copy,
not the frozen production artifact or a host installation.

### Final current-artifact verification

The final frozen bootstrap is 69196 bytes with SHA-256
`b83a852e6a1dc203a5df1185a257a34db9c20a4ac661dc649359bb876afd46f7`.
Its current application manifest remains
`b0495c956322e7b32c424f78f5d641c7377ba5acc535741f40aa10a669c0bcdb`.
Separately read launch `e65934692b540d8c0e6b7f5f3af17774af5520d1a1494b44acd1ce15f0c9070f`
produces the unchanged pinned-corpus unqualified input serialization,
with a one-byte comparator control and independent output/absence verification.
Current native qualification reruns the child,
host,
retention,
hook,
file-race,
collision and signal-phase controls against those bytes.
The committed CLI suite also exercises unexpected-fault exit `5` without a private error excerpt.

The complete default unit suite records 631 expected and observed test-entry processes on Node `v26.8.2`.
After scoped terminal-newline reconciliation,
fresh builds,
types,
unit tests and lint still pass,
and generated bootstrap/application-manifest bytes match the same frozen identities.
This is not independent parser correctness,
hostile-host immunity or exhaustive portable native CI coverage.
Ownership and qualification limits are explicit in
`package/module/translation-repair/producer-input-verification.md` (removed before 2026-10-18).

Final private proof `input-runner-proof-kIIDf7` contains 9429 hash-verified copied files.
Only 75 independently identified stopped task containers are removed afterward;
every ID is checked absent.
The unrelated container,
all worktrees,
proof roots,
frozen artifacts,
corpus data and owner lockfile are preserved.
No installed dependency patch or external filing is made.

## What does not work

- JavaScript bundling alone does not carry this native resource.
- Build and TypeScript success do not prove importability.
- Checking `NODE_OPTIONS` only after Node starts does not prevent the measured image preload from executing.
- `--unsetenv-all` alone does not suppress subsequent host-proxy forwarding.
- Environment clearing does not replace the image entrypoint.
- Explicit bind flags do not remove nonconflicting `containers.conf` mounts or subscription mounts.
- `CONTAINERS_CONF` alone does not defeat a subsequently applied `CONTAINERS_CONF_OVERRIDE`.
- Colon-joined volume syntax is not the syntax of `containers.conf.mounts`.
- A generated loader's version literal does not prove a dependency-version change.
- Keeping libraries available only through inherited environment variables does not cover minimal-environment children.
- A missing corpus and a missing path inside a valid corpus are different inputs.
- The historical pipeline-digest comment calling dependency bundling a repo-wide-only configuration change
  does not describe the current per-package override.

## Upstream filing decision

1.  Upstream fault:
    not established.
    The application prototype moved a file-relative loader without its asset.
    The Podman controls exercise configuration precedence and native mount grammars,
    not an established upstream defect.
2.  Fixability:
    the measured consumer-side asset emission succeeds.
3.  Supported use:
    Rolldown supports emitted binary assets;
    this report does not claim Satteri promises resource-free JavaScript bundling.
    Podman's hidden default-mount-file override is explicitly marked testing-only;
    the measured invocation does not create a general compatibility promise.
4.  Contribution policy:
    not reached because no upstream patch or filing is proposed.
5.  Maintainer intent:
    not inferred from the generic npm diagnostic or these application probes.
6.  Prototype:
    the asset-carrying artifact and removal control were exercised without installed dependency patches.

Nothing to file upstream for the observed relocation failure.
No issue,
comment,
package reinstall or installed-source edit occurred.
