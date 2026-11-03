# Building and testing the Chromium metrics patch

## Build-input research at 2026-11-03T13:31:17+00:00

The user requested a patch they can attach to the Chromium issue.
The main task committed to preparing and testing it.
This pass researches the real Chromium build and native tests;
it does not substitute a reimplementation of the proposed policy.

The target source revision is `360d943d418b3d6645fea86766f5dcc4a7cc3b2e`.
The main-owned checkout is `${HOME}/temp/agent/chromium-metrics-patch.QLa5st`.
It is sparse;
for example,
`chrome/test/BUILD.gn` and `tools/clang` were absent from its working tree when inspected.
The source graph must be completed in the isolated build environment before GN generation.
This research has not changed that checkout,
installed a tool,
executed an upstream build script on the host,
or built Chromium.

## Recommended build shape

Use Chromium's own pinned toolchain and dependency graph in a disposable build environment.
The [current Linux guide][linux-build] names GN as the generator,
Siso as the main executor,
and `autoninja` as the supported wrapper.
The absence of a host `ninja` executable is therefore not itself a reason to stop.
The [Clang guide][clang] says the supplied prebuilt Clang is the supported compiler.
Use its paired libraries and plugins rather than the unrelated host `clang++`.

For the existing Chrome accessor test file,
build `chrome` and `chrome/test:unit_tests` in the same output directory.
The documented invocation shape is
`autoninja -C out/metrics chrome chrome/test:unit_tests`.
Run the compiled `unit_tests` binary with the `ChromeMetricsServiceAccessorTest.*` filter,
and verify its output includes the actual expected test names.

If the patch instead adds tests to `components/metrics`,
the corresponding executable is `components:components_unittests`.
Build `chrome/test:browser_tests` only if a new native browser test needs it.
The [Chrome metrics source sets][chrome-test-sets]
already contain the accessor and metrics-service browser-test files.
[The components aggregate][components-tests] links `components/metrics:unit_tests`.
Once GN generation succeeds,
use the Linux guide's `gn refs` procedure on the actual changed test file to confirm its executable.
Compiling only a source file with Siso's `source.cc^` syntax can catch compile errors,
but does not satisfy the browser-startup or native-test checks.

A suitable development configuration keeps a component build with assertions
and reduces unrelated symbol output:
`is_debug=true`,
`is_component_build=true`,
`symbol_level=1`,
`blink_symbol_level=0`,
`v8_symbol_level=0`,
`use_siso=true`,
and `use_remoteexec=false`.
The [symbol settings][symbols] and component-build behavior are documented in the Linux guide.
This configuration is for validation;
any timing comparison needs a baseline built with the same arguments.

[The checked Siso configuration][siso-args] requires generated checkout configuration,
including `.sisoenv` and `gclient_args.gni`.
[Remote execution defaults off][remote-args].
The [pinned `autoninja.py`][autoninja-local] explicitly invokes `siso ninja --offline`
when remote execution is disabled.
It converts `-j` to the local-job limit when the requested value does not exceed the detected CPU count.
Google RBE authentication and API keys are unnecessary for this local workflow.

## Pinned Linux build inputs

These are selected required tool inputs from the target revision's [DEPS file][deps-tools].
They are not a hand-maintained replacement for the complete dependency graph.
`gclient` must resolve and record that graph in the disposable build environment.

### Depot tools and bootstrap

Chromium [pins depot_tools][depot-pin] to
`816da11f80777dcf8c10bdaa84df18450dcba12a`.
Use that source revision for `gclient` and `autoninja`.
The [pinned updater][depot-updater] checks `DEPOT_TOOLS_UPDATE=0` and skips automatic updating.
Keep this setting confined to the container workflow.

The depot-tools bootstrap also carries a committed [CIPD client pin and digest][depot-client-digest]:

- Client revision:
  `43615ea9acdcc6c1c213c5c7a30a568fdeb7c3dc`.
- Linux amd64 client SHA-256:
  `7296d878d6ef509e59838769b79b29d7cc76a4a97f0678c5376ac26190924ec7`.
- [The tool manifest][depot-bootstrap] references `cipd_manifest.versions`,
  whose resolved instances are committed to reduce reliance on mutable server-side resolution.
- `vpython3` is pinned to
  `git_revision:1fe128e81d74b6aec15277bc535e57af04bc3209`.

The source was fetched and read;
none of these upstream scripts or binaries was executed by this research pass.

### Generator and executor

The [tool versions][executor-pins] and [package destinations][tool-packages]
are declared in DEPS.
Public CIPD `ResolveVersion` requests succeeded for the selected Linux packages
and returned the instance SHA-256 values recorded here.

- GN package:
  `gn/gn/linux-amd64`.
  Version:
  `git_revision:ed7fee04cb30b96ed40ef188acfad5417349e399`.
  Instance SHA-256:
  `2bd2c2f10aafea72fa875f8c2d5b87d098031e17f470d4c1b77f2f1ebaf23a75`.
- Siso package:
  `build/siso/linux-amd64`.
  Version:
  `git_revision:c508213533aa5cd425b69cb0f66e53522d77022d`.
  Instance SHA-256:
  `6c12d0e0ea8216d6b7687869c0426b1865327c6101baafa99aa4e986fb147367`.
- Ninja remains a declared dependency:
  `infra/3pp/tools/ninja/linux-amd64`,
  version `version:3@1.12.1.chromium.4`.
  Instance SHA-256:
  `3f1f1cc0f69a1bcfdf67fb6c2bc7419b1dd812bb8d0e79afa9bfa8a3553b5082`.
  The selected workflow uses Siso through `autoninja`.
- Chromium's hermetic CPython package is `infra/3pp/tools/cpython3/linux-amd64`,
  version `version:3@3.11.9.chromium.38`.
  Instance SHA-256:
  `b7b8a0a6d171b8918fda51aee1d6ffeb0c8a14c791fb1d0bc5d7454c8c9d949b`.

### Compiler and Rust

The GCS bucket is `chromium-browser-clang`.
The [Clang entry][compiler-artifacts],
[Rust entry][rust-artifacts],
and [Rust libclang entry][libclang-artifact] specify:

- Clang object:
  `Linux_x64/clang-llvmorg-24-init-14453-ge063be79-1.tar.xz`.
  SHA-256:
  `0b76afc1a29b0583f7723e966b9b65a22b15d8fdcd38e7c567ae32343c68944d`.
  GCS generation:
  `1792086691304470`.
- Rust object:
  `Linux_x64/rust-toolchain-8a9dd7abdf013ec10cfeaaa3685feba18ef1c90c-6-llvmorg-24-init-14453-ge063be79.tar.xz`.
  SHA-256:
  `4b4f64d5e82e845099b6ff63c61df311f8202183f1996ca247e20db6d9cda29c`.
  GCS generation:
  `1793195406905214`.
- Matching Rust libclang object:
  `Linux_x64/rust-libclang-8a9dd7abdf013ec10cfeaaa3685feba18ef1c90c-6-llvmorg-24-init-14453-ge063be79.tar.xz`.
  SHA-256:
  `693fd176a27508efb6cbc06b7db9c64abfc19497664dbaf4a9f4d489b8b4239a`.

Chromium's build configuration enables Rust for a Chromium build.
The repository's unrelated Rust toolchain is not a substitute for this paired prebuilt toolchain.

### Sysroot and WebUI inputs

The [Linux sysroot policy][sysroot-policy] uses a Debian sysroot by default,
independently of the container's host distribution.
[The amd64 input][sysroot-pin] is in bucket `chrome-linux-sysroot`,
object and SHA-256
`f851add76bdae472fb6c94b4f920bc4a132d85ac607951391297f636af7e934e`,
generation `1792192152972543`.
The x64 checkout also declares the i386 sysroot;
do not remove declared inputs solely because the final target is x64.

The [Node entry][node-inputs] includes a Chromium-owned archive
with SHA-256 `bdea62a6857735d62165ec97242d79a693564c72d551890c4d009d2355abf14c`
and the [matching `node_modules` archive][node-modules]
with SHA-256 `5b5e38c58094ee898b1eceb29df7d3a2a2e6b07add6a05b4c554eef0dfb56465`.
Retain these build inputs instead of substituting the host's newer Node installation.

Read-only HTTP `HEAD` probes of the generation-qualified Clang,
Rust,
and amd64 sysroot objects returned `200`.
Their content lengths matched the DEPS values:
`58394648`,
`287835700`,
and `19314716` bytes respectively.
The returned GCS generations matched too.
These checks establish availability and metadata agreement;
the downloader still must verify the declared SHA-256 before executing any retrieved tool.

## Dependency acquisition and provenance

Keep the complete checkout at a stable container path such as `/work/src`.
The [inspected `gclient` parser][gclient-sync] supports `config --name`,
`config --unmanaged`,
`sync --revision src@<commit>`,
`sync --nohooks`,
`sync --no-history`,
and `sync --output-json`.
It also supports [the resolved dependency record][gclient-revinfo]
through `revinfo --actual --output-json`.
These are source-checked invocation shapes,
not a claim that sync has already run.

Configure the public Chromium solution,
sync the exact source revision without hooks,
verify the resulting source and dependency revisions,
and run the required hooks inside the container.
Apply the candidate patch to the container-owned source without changing the main-owned prototype checkout.
Record the source revision,
patch hash,
GN arguments,
resolved dependency manifest,
tool identities,
and image identity beside the resulting test logs.

The selected DEPS defaults disable Android,
Fuchsia,
and PGO-profile checkout.
Keep the native Linux defaults unless an observed dependency requires a change.
Do not use a moving depot-tools checkout,
`llvm_force_head_revision`,
or `rust_force_head_revision` for this validation.

## Container boundary

The [container-base decision](../../decision/container-base-distro.md)
selects `docker.io/cachyos/cachyos-v4:latest` for the reported v4-capable host,
passed through `ARG CACHYOS_IMAGE`.
Record the resolved image identity for the experiment while retaining the required `latest` selection policy.

The Linux guide contains an [Arch dependency recipe][arch-deps]:
`base-devel`,
`python`,
`perl`,
`gperf`,
`nss`,
`alsa-lib`,
`glib2`,
`gtk3`,
`nspr`,
`freetype2`,
`cairo`,
`dbus`,
`xorg-server-xvfb`,
and `xorg-xdpyinfo`.
This is the documented starting point for the CachyOS base;
actual installation and dependency checks remain part of the container build.
Use the Chromium sysroot for target libraries.
There is no reason to run the Ubuntu-oriented `install-build-deps.sh` on the host.

Use an explicit build context containing only intended source,
patch,
and build-input records.
Give the container its own home and caches.
Use a dedicated non-root user for dependency synchronization,
compilation,
and the browser;
the [pinned updater explicitly exits when `USER` is `root`][depot-user].
Install the OS packages during image setup.
Do not forward the user's home,
SSH agent,
Git credentials,
Google credentials,
repository secret files,
or desktop sockets.
Podman `run` and `build` both default to forwarding proxy environment variables;
their inspected help supports `--http-proxy=false` to disable that forwarding.

Put CPU,
memory,
process-count,
and I/O limits on the container itself.
The existing `systemd-run` research scope does not impose its I/O weight on a Podman container.
Podman `run` exposes the required `--cpus`,
`--memory`,
`--pids-limit`,
`--blkio-weight`,
and `--rm` options.
Its `build` help exposes CPU and memory limits but no direct `--pids-limit` or `--blkio-weight` option.
Run the resource-intensive compiler in an explicitly bounded `podman run` environment,
then retain the built runtime artifacts in an image for the mount-free browser check.
The main task should size the build limits from its current host probe,
not reuse the research scope's `200%` CPU and `2 GiB` memory limits.

## Real browser and native-test verification

Run the actual patched browser with a disposable profile inside the container.
Use a container-owned X display such as the documented Xvfb dependency.
Keep Chrome's `--headless` mode off:
the [startup code][histogram-startup] explicitly skips the relevant initialization in headless mode.
Do not expose the user's display or drive physical keyboard input.

A native startup regression can directly inherit `InProcessBrowserTest`.
At the checked revision,
the [generic setup][harness-setup],
[default command-line helpers][harness-defaults],
and [Chrome test-suite initialization][harness-suite]
do not install either metrics consent override,
enable forced preference lookup,
or append `--force-enable-metrics-reporting`.
The Linux [base fixture invokes real `ContentMain`][harness-content-main].
Use explicit false precondition assertions for forced reporting and forced preference lookup
so external launcher arguments cannot silently change the test's premise.
The specialized `MetricsServiceBrowserTest` and `MetricsServiceBrowserFilesTest` fixtures
install their own metrics overrides and are unsuitable parents for the reporting-ineligible case.

Verify ordinary and sparse histogram counts through the existing trusted WebUI API
and inspect the browser process's actual memory mappings.
Check the normal unbranded path and the deliberately reporting-eligible path separately.
Any force-reporting run must be isolated from public telemetry endpoints.
Native accessor tests and the running browser must exercise the compiled implementation.
An independently copied predicate or a fake browser cannot establish that the patch works.

Capture build failure diagnostics,
native test names and results,
browser startup logs,
selected mapping state,
and histogram counts.
If latency is compared,
build and run an unmodified baseline with the same configuration and establish unchanged-input variation first.
Do not generate synthetic host load.

## Tool-lock boundary

[The repository rule](../../../AGENTS.md)
says tools are installed from `toolchain.lock.json`,
including archive and SHA-256 selection in containers.
The inspected Monochromatic lock had no declared bin for
`gn`,
`ninja`,
`autoninja`,
`siso`,
`clang`,
`clang++`,
or `gclient`.

For this authorized upstream build,
the proposed scope interpretation keeps Chromium's hermetic tools as its checkout-local,
DEPS-owned build inputs inside the disposable environment.
It leaves Monochromatic's installed toolchain and the user's global PATH unchanged.
Record that interpretation explicitly;
the rule does not expressly discuss another project's vendored build inputs,
and Chromium's DEPS is not Monochromatic's `toolchain.lock.json`.
The rule contains no separate instruction to request permission.

If the main task applies the rule literally to these vendor downloads too,
the existing owner is `mpm-shadow`.
[Its documented origins](../../../package/mpm-shadow/cli/README.md)
and [resolver dispatch](../../../package/mpm-shadow/cli/src/toolchain/update_resolve.rs)
currently lack a direct URL,
GCS,
or CIPD resolver.
Plan rows cannot contain resolved `version` or `artifacts` fields.
The corresponding strict implementation route is to extend that owner,
generate the needed lock rows through it,
and consume those rows in the container.
Hand-editing a lock or installing unrelated latest tools globally is not a valid substitute.
This source research does not itself require that adjacent manager implementation or an automatic approval pause.

## Verification status

No Chromium build,
dependency installation,
or native test was run by this research pass.
Current upstream sources,
local task/tool-owner guidance,
Podman help,
and selected artifact metadata were inspected.
All `31` source selections passed browser verification by `2026-11-03T14:29:11.955Z`.
The initial updater-page check emitted `agent-browser`'s
`Wait timed out after 25000ms`;
a fresh navigation exposed the expected source line,
and the targeted recheck passed.
No cause for the initial missing anchor was established.

Scoped `mise run lint:monochromatic-lint` passed after correcting prose line breaks.
GitHub's HTML renderer for commit `2282b557a0`
preserved the complete heading sequence and inline code spans,
with no stray backticks,
unexpected emphasis,
or tables.
Both owned citation browser runs closed successfully;
process and systemd-unit checks found no remaining owned citation jobs.

[linux-build]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/docs/linux/build_instructions.md?plain=1#L194-L218
[symbols]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/docs/linux/build_instructions.md?plain=1#L399-L413
[clang]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/docs/clang.md?plain=1#L1-L7
[chrome-test-sets]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/chrome/browser/metrics/BUILD.gn#L559-L570
[components-tests]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/components/BUILD.gn#L260-L265
[siso-args]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/build/toolchain/siso.gni#L5-L20
[remote-args]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/build/toolchain/rbe.gni#L8-L21
[deps-tools]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L570-L581
[executor-pins]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L284-L288
[tool-packages]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L2783-L2802
[depot-pin]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L2228-L2229
[compiler-artifacts]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L946-L952
[rust-artifacts]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L1102-L1117
[libclang-artifact]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L1140-L1154
[sysroot-policy]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/docs/linux/sysroot.md?plain=1#L1-L14
[sysroot-pin]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L640-L652
[node-inputs]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L833-L850
[node-modules]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/DEPS#L897-L912
[arch-deps]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/docs/linux/build_instructions.md?plain=1#L643-L651
[histogram-startup]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/chrome/app/chrome_main_delegate.cc#L1803-L1825
[depot-client-digest]: https://chromium.googlesource.com/chromium/tools/depot_tools/+/816da11f80777dcf8c10bdaa84df18450dcba12a/cipd_client_version.digests#18
[depot-bootstrap]: https://chromium.googlesource.com/chromium/tools/depot_tools/+/816da11f80777dcf8c10bdaa84df18450dcba12a/cipd_manifest.txt#21
[depot-updater]: https://chromium.googlesource.com/chromium/tools/depot_tools/+/816da11f80777dcf8c10bdaa84df18450dcba12a/update_depot_tools#105
[depot-user]: https://chromium.googlesource.com/chromium/tools/depot_tools/+/816da11f80777dcf8c10bdaa84df18450dcba12a/update_depot_tools#8
[autoninja-local]: https://chromium.googlesource.com/chromium/tools/depot_tools/+/816da11f80777dcf8c10bdaa84df18450dcba12a/autoninja.py#489
[gclient-sync]: https://chromium.googlesource.com/chromium/tools/depot_tools/+/816da11f80777dcf8c10bdaa84df18450dcba12a/gclient.py#4719
[gclient-revinfo]: https://chromium.googlesource.com/chromium/tools/depot_tools/+/816da11f80777dcf8c10bdaa84df18450dcba12a/gclient.py#5106
[harness-setup]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/chrome/test/base/in_process_browser_test.cc#L521-L539
[harness-defaults]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/chrome/test/base/test_launcher_utils.cc#L33-L97
[harness-suite]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/chrome/test/base/chrome_test_suite.cc#L81-L131
[harness-content-main]: https://github.com/chromium/chromium/blob/360d943d418b3d6645fea86766f5dcc4a7cc3b2e/content/public/test/browser_test_base.cc#L626-L629
