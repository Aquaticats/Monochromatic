# Depot reuse strategy research

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Active research boundary

The earlier pnpm archive and library compiler-cache probes do not establish a CI reuse strategy.
This campaign exercises cross-language preparation,
installed-tool reuse,
real build/test consumers and input invalidation.
Research is reopened;
no interview or production adoption is pending.

The frozen repository source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Executable harnesses live only on the isolated `research/depot-reuse-20261018` branch.
Main receives documentation and evidence.
No production routing,
subscription,
organization settings,
account-wide usage or invoices are changed or inspected.
Existing automatic job identity supplies Cache authorization;
no broad credential is created or copied into a new secret.

## Repository inventory

### Installed tools, download stores and generated state have different owners

Root `mise.toml:36` retains Bun `1.3.14` for actual Bun-specific consumers:
`package/hall-monitor/daemon` builds a compiled bytecode executable,
and `package/runtime-error/bun` exercises Bun error-output fixtures.
These uses do not make Bun a pnpm package-download cache.
The installed Bun binary and its compilation outputs need their own tool/source identities.
The present prepared image's explicitly installed subset does not include Bun.

Root `mise.toml:1090` owns the Playwright Podman image,
whose frozen `playwright.Dockerfile` pins the browser/system-library base by version and digest.
The Node Playwright package remains in the pnpm catalog.
Therefore restoring `node_modules` alone does not prepare the browser executables or OS libraries.
The Dockerfile additionally downloads Bun through a floating installer;
an immutable resulting image digest would freeze its actual installed bytes,
while merely retaining the Dockerfile's browser base digest would not identify that Bun installation.
This campaign does not execute that installer.
Actual browser/runtime probes belong to the parent campaign.

`package/monorepo-task/core-exec/src/tsc-filter.ts:54` owns TypeScript incremental-state cleanup.
It removes `dist/**/*.tsbuildinfo` and `.cache/typescript/**/*.tsbuildinfo` before checks,
with the stated reason that stale dependency state can produce false negatives.
Restoring that state would conflict with the existing correctness owner;
the present proposal does not override it or claim a TypeScript incremental-cache speedup.
Built JavaScript,
declarations
and native binaries are separately consumed artifacts whose source/compiler/configuration identities matter.

The terminal's existing `package/terminal/app/mise.toml:43` dispatcher chooses host Cargo
only when Cargo,
Zig and the required native libraries are available.
Its container fallback retains Cargo registry and Ghostty Git state in the `terminal-cargo` volume,
while the host fixes `ZIG_GLOBAL_CACHE_DIR` under `target/zig-global-cache` to match its physical source paths.
Generated Ghostty libraries are staged beside the executable by a separate runtime-library owner.
These are active Zig/C/native build stores,
not evidence of a standalone Go workload.

Android preparation is split between the Mise JDK/tool installers and root `prepare:android`.
The latter requests platform `android-37.0`,
build tools `37.0.0`,
NDK `29.0.13846066`,
platform tools and the two Android Rust targets.
The Android native owner also derives NDK paths and heals its own generated native state.
Gradle dependency/plugin downloads,
task-output caching,
SDK/NDK installation,
Rust cross-target artifacts,
APK artifacts and a newly started runtime/device remain different preparation boundaries.
The separate Android cloud build and its JVM/APK evidence belong to the routing agent;
this note does not claim SDK installation from the Node/Java/Rust image alone.

### Current execution checkpoint

Harness `363bd501a0366dc4f74e35ad1b77f1a8ffad95b4` adds repeated cache-off Gradle consumers,
isolated response-fault/input controls and the existing real Detekt host owner.
Cache-off allocations `d3ks91rnnb` and `pq5pzj9mcg` both passed.
Both use the same frozen application,
prepared image,
pnpm content-store preparation and timed `test --rerun jar` command as the warm pair.
The controlled difference disables both remote and local task-output caching.
The research harness changed;
the application did not.

Updated-image Rust allocation `w16c45njf5` ran every selected Nextest case without fail-fast:
1,951 executed,
1,934 passed,
17 failed and five skipped.
Frozen `package/git-policy/cli/src/native/test_support.rs:19` hardcodes `/usr/bin/git`.
The verified Git 2.56 executable on `PATH` therefore did not satisfy that fixture's absolute-path contract.
Several failed cases explicitly observed provider Git 2.55 bytes.
Two real-linter cases require `GIT_POLICY_NATIVE_TEST_LINTER`;
one lock-identity case also failed.
Those failures remain separate evidence boundaries;
the application source is not repaired for this experiment.
Rust harness `c653cef3cdb9bbba2e8d5bbb46d653e09616610f`,
allocation `n2zlbh5gpb`,
provisions the disposable sandbox's exact Git fixture path and the lock-bound published linter prerequisite.
The immutable `monochromatic-lint-v0.1.3` release names source `c954f74798ee47c027ee3d9a31811de6345ca328`.
Its independently verified Linux GNU archive is 7,136,153 bytes,
SHA-256 `9d9021526d5377d268d4c75ed9d42d9df53d9ed76969031edb6a7809863ec665`.
The extracted executable is 16,639,640 bytes,
SHA-256 `6f82c73dadadd11506624a085d38e7e5ab98a26014416bfedd03c1928a85cc69`.
Mise's binstall-only setting prevents fallback to a current-source linter build.
This is a published test prerequisite;
it does not exercise the current linter's Rust-analyzer compilation workload.
The target archive retains its success-only publication guard.
Compiler-cache writes and diagnostic artifacts are retained even when application tests fail.

The Detekt host is the existing `//package/linter/kotlin:lint:detekt` owner,
which invokes Gradle `detektCheck` with the produced plugin JAR and `detekt-cli:2.0.0-alpha.6`.
Controls allocation `r328mp420m` runs the owner and separate documented/undocumented Kotlin fixtures,
retaining native CLI help/version and exact JAR hashes.
This does not substitute for the root `lint:detekt` task's repository-wide fanout.

The original 18 completed job attempts account for 3,428 request-to-terminal seconds,
or USD 1.3712 at the declared eight-vCPU timestamp-price proxy.
The completed cache campaign's job ceiling was 18,
with the same two-simultaneous,
eight-vCPU,
32-GB,
15-minute and USD 5 limits.
Its last dispatch reserved USD 0.36 and forecast USD 1.644 before the reader completed.
This excludes storage,
network,
other agents and any account bill.

The native warm pair uses pushed harness `ce27ef7d9d76615243e969e987ba2efd76195f49`.
Both ran the actual full native CLI/test cohort with failures propagated,
then independently passed the corrected Detekt host fixture controls.
The initial documented constant-returning fixture was rejected by Detekt's existing
`FunctionOnlyReturningConstant` rule;
the corrected fixtures depend on their argument.
The real package Detekt owner and CLI help/version had already passed in `r328mp420m`.
Each warm allocation accepted the documented argument-dependent Kotlin function
and rejected the undocumented counterpart with `Missing KDoc on function 'undocumented'. [RequireKDoc]`.
The loaded `kotlin-linter-0.1.0.jar` remained 6,445 bytes,
SHA-256 `fb64ccda45fe0d9446fb084029246c59063ec9bf1c0c37b47a3fead5d1849707`,
before and after each host invocation.

Corrected-prerequisite seed `n2zlbh5gpb` executed 1,951 cases:
1,949 passed and two failed,
with five additional skipped cases and 1,956 enumerated in total.
The exact remaining cases are
`real_index_lock::tests::releasing_after_a_landing_leaves_a_replaced_lock`
and `real_index_lock::tests::an_install_through_a_replaced_lock_is_refused`.
The first observed the replacement file disappear;
the second observed installation succeed instead of refusing a replacement.
An earlier allocation failed a different replacement-identity case.
The completed matched controls reproduce replacement-lock failures on both providers' ordinary directories
and pass every selected case on their tmpfs fixtures.
The [lock-identity investigation](../troubleshooting/git-policy-replaced-lock-filesystem.md)
retains the source comparison,
actual inode-reuse fields and precise limits of that diagnosis.
These runtime assertions do not establish cache corruption.

The unchanged Rust warm pair each recorded 131 Rust compiler-cache hits,
with zero `cache_misses`,
`compilations` or `cache_writes` at the CLI-build checkpoint.
The corresponding seed populated 131 misses/writes with no hits.
Building and linking the real dev-profile CLI took 52.646 seconds in the seed,
then 41.495 and 39.534 seconds in the warm allocations.
The warm spread was 1.961 seconds.
There is no repeated cache-off Rust baseline,
so this comparison does not establish a cache-off benefit or full-job multiplier.
Final linking and compiler requests that sccache cannot cache still execute.
The deciding sccache `0.18.0` source,
`src/compiler/rust.rs:1179`,
rejects non-rlib/staticlib crate types because it cannot identify all system-linker inputs.
Its `compilations` counter therefore does not count every Rust compiler process.
The retained wrapper journal shows the large native library's test compilation still taking
7.423 seconds in the seed and 7.220 and 7.328 seconds in the warm pair.
Those requests overlap other work and cannot be added to wall-clock totals.
The scanner also has real build-script execution:
`package/forbidden-strings/cli/build.rs:98` declares its baseline/parser inputs
and emits serialized rules under Cargo's `OUT_DIR`.
A compiler cache of the build-script executable and a target archive containing its generated output
cover different work.
The present timing receipts do not isolate that script's execution time.

The warm allocations each ran 1,951 Nextest cases,
with five skipped cases and 1,956 enumerated in total.
Warm `tpqlt2h227` passed 1,948 and failed three;
warm `kfmq92jdm1` passed 1,949 and failed two.
Both observed `real_index_lock::tests::a_replaced_lock_is_never_released`
and `real_index_lock::tests::releasing_after_a_landing_leaves_a_replaced_lock` fail.
Only `tpqlt2h227` additionally failed
`index_lock_evidence::tests::the_change_time_rounds_nanoseconds_down_to_milliseconds`,
whose terminal assertion was `a change time with a millisecond part was observed`.
The raw per-case logs and denominators are retained for a matched provider/filesystem control.
This variability is not evidence that compiler-cache restoration caused the failures.

The independent forbidden-strings CLI producer and fresh target reader both passed,
with the CLI's own complete Nextest suite required to pass before its target archive was published.
A distinct exact-source Nextest/Git diagnostic archive retains failing native tests for matched provider controls.
That diagnostic artifact is not a success-only target-cache publication or a passing gate.
Both producer profiles used pushed harness `360abf1382631750c3f26928c2342d0acd13eb6b`.
The scanner used a distinct target disk and compiler-cache prefix;
its release CLI,
entire owned Nextest task and source/dependency/compiler controls passed before target publication.
The diagnostic profile audits actual Nextest archive/run help,
then packages exact-source test executables,
Git 2.56 and the host tools with hashes for the parent to exercise independently.
Nextest's documented `--workspace-remap` relocates runtime workspace paths,
but does not rewrite embedded Rust `env!` strings.
Frozen native Git-config tests contain such a string;
therefore the filtered replacement-lock/metadata cohort is the transport control,
and general relocation of the complete suite remains unproved.

### Independent successful target archive and fresh reader

Producer `v7njncr47d` passed all 237 cases through the scanner's existing release Nextest owner,
with zero skipped cases.
Changing the real warning source ran one case and produced the expected failed assertion.
Enabling the locked dependency feature `gix-index/serde` passed all 237 cases.
Changing compiler flags to `-C debuginfo=1` moved Rust compiler-cache misses from 148 to 275.
The final original-input run again passed all 237 cases before publication.

The compressed target is 128,391,415 bytes,
SHA-256 `c0ebdf4d623deaad706d1aee02e57245bfc29994d935ad1a3780f67df08513f5`.
Its manifest is independently pinned as
`e8f233474e03ca54e18f59fe02dc2020a3fd942623b228e643ae046ee953569c`.
Fresh reader `zq7cc1nd4b`,
from pushed harness `670d84778d24567e36b01a22c33833310d44d659`,
verified a real `fuse.archil` mount,
rejected partial archive bytes,
changed real source and a missing-manifest fixture,
then restored the admitted archive.
The release CLI build reported 116 `Fresh`,
zero `Compiling` and zero `Dirty` records.
The complete owned Nextest task passed all 237 cases with zero skipped cases.

Seed release CLI build took 52.228 seconds;
the admitted reader's build took 0.070 seconds.
Archive extraction took 0.334 seconds,
excluding preceding archive reads,
hash admission and mount setup.
The complete reader Nextest command took 45.200 seconds,
including 22.506 seconds of actual tests.
Seed Nextest took 39.233 seconds,
including 21.449 seconds of tests.
These boundaries establish working target reuse and continued real test execution.
There is one target-reader sample,
so neither a stable restoration ratio nor a whole-job speedup is established.

### Diagnostic archive and matched control extension

Diagnostic run `7sm49f324d` created a 17,046,905-byte Nextest archive,
SHA-256 `d6b7edef092cad3c81a88deb293777665e4f38e5b9caf27221a5225c4965d596`,
plus exact Git and host-tool bundles.
Its initial replay failed before enumeration because the explicit extraction directory did not exist.
The corrected local read-only Podman transport control created that directory,
verified all 1,079 frozen source hashes and both tool hashes,
and passed all 34 selected lock/metadata cases,
with 1,922 cases skipped by the selection.
The [Nextest recovery note](../troubleshooting/depot-native-nextest-completion.md)
retains the deciding source and the fixture identity correction.
Local tmpfs success does not establish the cause of the distinct native filesystem failures.

The coordinator has admitted three additional cloud allocations for the matched portability control,
raising the combined campaign ceiling from 18 to 21.
Only those additional allocations use four CPUs,
16 GB and ten-minute bounds;
the two-simultaneous and cumulative USD 5 timestamp-price guard remain.
Their total conservative reservation is USD 0.36,
giving a USD 1.7312 forecast from the completed cache campaign.
Pushed harness `8a615939f096ba0c4b0f1a28815b759be719368f` launched native run `ptxnhf1ppm`
and GitHub Actions run `39113641724` for the two serial GitHub-provider jobs.
The local isolated transport control passed three unchanged repeats,
each with 34 passed cases and 1,922 filtered skips.
Its rootless image has no `sudo`;
that retained unsupported-mount boundary receives no tmpfs coverage credit.
Each cloud provider completed three unchanged repeats of the exact archived 34-case cohort
in its ordinary owned fixture directory and three in an owned 256-MiB tmpfs.
The providers are native Depot,
GitHub-hosted Ubuntu 24.04 and a four-CPU Depot GitHub runner.
Signed object-read URLs are limited to the existing diagnostic artifact;
no broad credential is installed in either CI service.
Fixture clock/inode fields,
actual mount types and every failed outcome remain evidence boundaries.

Native ordinary-directory repeats passed 32,
32 and 32 cases;
Depot GitHub repeats passed 32,
32 and 31;
standard GitHub-hosted Ubuntu repeats passed 31,
32 and 31.
All nine tmpfs repeats passed all 34 cases.
The only failed identities are the three replacement-lock cases named in the lock-identity investigation.
Every repeat enumerated 1,956 cases and skipped 1,922 through the explicit selection.
These are repeated executions of 34 unique cases,
not complete executions of the original catalog.
The separate ctime case passed in every matched repeat;
its historical failure remains unreproduced.

Each ordinary-directory field control observed inode/device reuse in all 64 unlink/recreate pairs;
each tmpfs control observed none in its 64 pairs.
Positive controls verified that the probe can observe an unchanged inode after replacing an existing file's bytes.
Every tmpfs unmount succeeded.
The actual mounted filesystem types,
raw fields and failure outputs are retained rather than assigning cause from a filesystem label.

All three receipt ZIP digests,
the GitHub mirror's outer ZIP and its original inner ZIP hash were verified.
Across the combined campaign,
21 artifact ZIPs and all 774 members have retained hash/CRC evidence.
The [complete matched receipts](../audit/depot-reuse-lock-portability-2026-10-18.receipts.json.gz)
are stored once as compressed JSON,
with compressed/uncompressed hashes bound in the main evidence.
Both collision-checked temporary object-URL secrets were deleted and their absence verified.
The combined 21 job attempts span 20 workflow runs and produce a USD 1.407 timestamp-price proxy.
The two GitHub windows each begin at their common workflow request and end at their own job completion;
this conservatively overlaps predecessor and queue intervals.
The native windows begin at their respective requests.
These are declared accounting windows rather than measured allocated CPU time or an invoice.

### Repeated Gradle task-output comparison

The unchanged warm command took 17.550 and 15.836 seconds;
the cache-off command took 21.435 and 20.262 seconds.
Within-condition spreads are 1.714 and 1.172 seconds respectively.
Every allocation used eight visible AMD EPYC 9R45 CPUs and a distinct boot ID.
Each executed all six JUnit cases with no failures,
errors or skips,
and generated the plugin JAR.
The warm pair restored compilation outputs;
the off pair made no HTTP cache requests and did not report compilation `FROM-CACHE`.
Actual tests never came from the cache.

The difference of observed means is 4.155 seconds,
or 19.9 percent of the cache-off mean,
for the `test --rerun jar` boundary.
That boundary includes JVM startup,
Gradle dependency/transform access,
compilation or restoration,
actual tests and JAR generation.
It excludes image boot,
checkout,
tool verification and pnpm linking.
The observed mean ratio is 1.249;
two samples per condition do not establish a general multiplier.
Whole-job warm durations were 162 and 177 seconds,
while cache-off jobs were 146 and 153 seconds.
The separate pnpm content-store import/link command varied from 57.243 to 75.891 seconds across these allocations.
Consequently the measured Gradle benefit is not an established whole-job improvement.

A tracked-file inventory at the frozen source contains 13,860 files.
Active `package/` files include 1,894 Rust sources,
294 Kotlin sources,
five Kotlin build scripts,
one C++ source,
4,409 TypeScript sources and 345 `.mjs` files.
There are 23 standalone Rust manifests,
two Gradle settings roots and 181 projects reported by the earlier frozen pnpm install.
No tracked Go source/module,
Python project manifest or CMake project file was found.
This does not exclude native code downloaded by Cargo,
the Android NDK,
Qt/C++ bindings or Zig-backed dependencies.
The inventory retains the complete filename and cache-reference searches,
rather than treating absent language manifests as absent transitive tool use.

The existing preparation owner is root `mise.toml`.
Its `prepare` task runs `prepare:pnpm` first,
then the remaining preparation children concurrently.
`prepare:pnpm` installs packages,
approves Sharp and runs file-enforcer.
`prepare:playwright` builds the existing Podman image.
`prepare:android` provisions platform/build tools,
NDK and Rust targets after checking their installed paths.
This campaign invokes the applicable narrow owners rather than running the umbrella preparation task.
Android emulator and graphics work belong to the parent campaign.

The pnpm layout is explicitly `nodeLinker: isolated` with `packageImportMethod: clone-or-copy`.
Its configured Linux/macOS,
x64/ARM64 and glibc/musl optional artifacts make the installed virtual store broader than one runner's native needs.
Content-store archives,
installed dependency snapshots and workspace symlinks therefore need separate correctness and timing checks.

## Planned command tree and execution manifest

The first campaign permits at most two simultaneous owned jobs,
8 vCPUs and 32 GB per job,
15 minutes per job and initially at most 12 jobs.
At USD 0.00005 per vCPU-second,
that scheduling ceiling corresponds to a USD 4.32 compute timestamp-price proxy.
It is a guardrail,
not a measured bill or storage-inclusive total.
The expanded bound and completed-job arithmetic are recorded in `Expanded campaign bound`.

The preparation job installs only the frozen Mise subset:
Node,
pnpm,
Temurin 21,
Rust,
Cargo Binstall and Nextest.
The lock resolves Rust to `nightly-2026-10-17`,
Temurin to `21.0.12+8.0.LTS` and Nextest to `0.9.148`.
Rust's declared Clippy,
source,
LLVM tools and Android targets are included in the measured cold preparation cost.
Installed files use a unique `/opt` research prefix so their snapshot lifecycle is explicit.
The committed Gradle wrapper resolves `9.8.1`;
the official distribution and wrapper-JAR SHA-256 values are checked before execution.

The snapshot action is inspected at revision `a082cbabd407afdf324a7b4f41889795636f4b24`.
Its release resolver currently selects snapshot tool `1.2.30`.
The action executes the tool through `sudo`,
authenticates with an automatic snapshot/job token and pushes only a unique research image.
Capture,
upload,
fresh-image startup and installed-tool availability are separate evidence boundaries.
The source checkout is removed before capture.
Only synthetic source/secret markers are used to check exclusions;
explicit environment masks supplement the platform defaults.
Filesystem marker controls do not prove that every possible secret location is excluded.

Kotlin consumers build the existing linter JAR and execute its JUnit tests.
Remote HTTP build caching uses the existing automatic job Cache token,
with local task-output caching disabled and a unique declared namespace in task inputs.
Gradle's task-specific `test --rerun` allows actual test execution alongside reused compilation.
Download/module caches,
configuration state,
compiler outputs and test execution are measured separately.

Rust consumers build the native Git-policy CLI and run existing Nextest library,
binary and public candidate-consumer tests.
The CLI links the repository's scanner,
regex engine and JSONC library.
Registry archives,
target artifacts,
incremental state,
sccache compilation and final linking remain distinct stores and boundaries.
No application source repair may turn a failing frozen baseline into an alleged passing cache result.

External reads cover the frozen source,
official tool releases,
Gradle distributions,
Maven/Gradle plugin repositories,
crates.io and Depot's CI/Cache/Registry endpoints.
Writes use disposable sandboxes,
unique image/disk/compiler-cache prefixes and retained research artifacts.
Gradle task outputs include an experiment-specific input salt.
Gradle's artifact-transform cache keys do not inherit that task salt;
the seed job can populate content-derived transform entries in the organization HTTP backend.
This is a separate trust and namespace boundary,
not a claim that every HTTP key carries the experiment name.
Stop on the declared resource ceiling,
unexpected credential use or an uninspected execution path.
Preserve failed outcomes and include their compute exposure.

## Captured preparation and active consumers

The first preparation run is `xkmm08vpjs`,
dispatched from pushed harness `e65f3ccf26781999717cfd9c17e1bd361f42183a`.
It requests one `depot-ubuntu-24.04-8` sandbox and a 15-minute timeout.
The preparation finished at `2026-10-18T06:32:24Z`.
Its image destination is `l4zncjjbj5.registry.depot.dev/research-reuse-20261018:tools-r1`,
with pushed manifest digest `sha256:9fb974fd32185c8afd5711370869d03e0f28e5b560c928eae324c20151ddeee8`.
Fresh native-CI tag consumers booted the image and matched all prepared tool and frozen-input hashes.
The [preparation evidence](../audit/depot-reuse-strategy-2026-10-18.evidence.json)
binds the frozen source inputs,
tool hashes and all 11 retained artifact members to the provider's uploaded ZIP digest.

Measured command durations are 15.137 seconds for the locked core tools,
1.575 seconds for Nextest,
2.361 seconds for pnpm's 582 downloaded packages,
1.238 seconds to create the installed-dependency archive,
16.880 seconds for Gradle dependency-graph resolution and 0.725 seconds for Cargo fetch.
Gradle's dependency report does not prove all later compile/test JARs were downloaded.
The snapshot helper measured 19.104 seconds for capture plus upload,
with 5,703,139,328 uploaded block bytes.
These boundaries exclude fresh-image startup,
checkout and action startup;
uploaded bytes are not a verified storage charge.
The pnpm store contains 908,109,264 bytes and the installed archive contains 296,779,994 bytes.

The helper selected by release version `1.2.30` reports `snapshot 0.1.0 (119be55)` at runtime.
Both identities are retained rather than silently treating them as matching versions.
The `max-age: 6h` input is not evidence of automatic registry deletion.

Consumer run `6wt5gm6t75`,
from harness `168107a38eadffe157c74be4267c10341c675550`,
failed before sandbox startup when `runs-on.image` used the full registry `@sha256:` reference.
The provider's diagnosis states:
`Dispatch failed: [not_found] image "9fb974fd32185c8afd5711370869d03e0f28e5b560c928eae324c20151ddeee8" not found`.
The documented unique tag form boots successfully.
The deciding server-side image resolver is not public;
this failure does not establish a universal digest-reference prohibition.
This dispatch failure does not measure Gradle,
tool installation or cache restoration.
The helper's official release resolver supplies a pinned Depot S3 archive;
authenticated lookup of public `depot/snapshot` source returned HTTP `404`.
The helper's internal source/build provenance is therefore unverified,
even though the snapshot action's download and invocation code is inspected.

The tag consumer `wh76h0pjpx` exposed a harness error:
the offline network-namespace control dropped to the runner user but retained `sudo`'s home directory.
Mise `2026.10.9` reported permission errors creating `~/.local/state/mise/trusted-configs`.
The corrected control supplies the runner's `HOME`,
`USER` and `LOGNAME` explicitly.
Subsequent consumers pass offline installed-tool availability,
prove that an empty isolated tool home fails offline,
and recover that missing Node installation through the locked online owner.
The failed control is retained separately from application or cache failures.

Gradle seed `x1psl3qf94` from harness `997db20b3bd5f1cdf3413c645d964c34dac63518` passed.
It restored dependencies from the prepared pnpm content store,
passed the existing semantic plugin build/host/external consumers,
built the Kotlin linter JAR and executed six JUnit tests with no failures or skips.
The recording HTTP proxy observed cache misses and successful uploads through automatic job identity.
This seed alone establishes no speed benefit.
The completed paired result is retained under Repeated Gradle task-output comparison.

Rust seed `ldj6qhgjd5` from the same harness restored the installed-dependency archive,
passed the same JavaScript consumers,
built the real native CLI and compiled Nextest test executables.
The actual test run planned 1,951 tests across three binaries with five skipped.
Default fail-fast executed 234:
230 passed and four failed,
leaving 1,717 unrun.
Those failures compare options against provider Git `2.55`,
while the repository-owned native verification baseline requires Git `2.56.0`.
The required Git revision is `a018953688f1b10bddf91bff8747068f5f4746a4`,
already selected by `package/git-policy/cli/bin/build-git-test-image.mjs`.
The subsequent `tools-r2-git256` image provisioned that prerequisite under a new tag,
retained `tools-r1` unchanged and enabled the actual complete test reruns.
The target archive is published only after tests pass,
so this failed seed did not publish a supposedly reusable passing target.

The registry manifest for `tools-r1` was fetched using the automatic job snapshot identity.
Its configuration media type is `application/vnd.depot.thin.snapshot.config.v2+json`;
its layers use `application/vnd.depot.thin.snapshot.sparse.v1` and `application/vnd.depot.ext4.image.v1`.
This is a native block snapshot,
not verified conventional Docker-compatible filesystem layers.
Fresh jobs contain neither the synthetic source marker nor the synthetic secret file.
Both masked and unmasked synthetic runtime environment controls are absent;
that observation alone cannot attribute their absence to masking.

## Historical 16-job admission

The first expanded authorization permitted at most 16 total owned jobs,
with the same two-job concurrency,
eight-vCPU,
32-GB and 15-minute per-job limits.
Before the expanded launches,
the five completed request-to-terminal windows are 94,
13,
54,
172 and 208 seconds,
totaling 541 seconds.
Charging every second of those windows at all eight requested vCPUs and USD 0.00005 per vCPU-second
gives a USD 0.2164 timestamp-price proxy.
The remaining eleven full 15-minute job ceilings add USD 3.96,
for a combined USD 4.1764 scheduling bound under the USD 5 compute guardrail.
The Gradle request-to-terminal window includes one second before its recorded start;
its start-to-terminal window is 171 seconds.
The failed digest dispatch is conservatively included despite allocating no sandbox.
These request windows can include unbilled orchestration;
the calculation is not account billing evidence.
Snapshot storage,
artifact storage,
cache retention,
registry requests and network charges remain separately unmeasured.

The first paired warm jobs used identical harness `9f59235436` and `reuse-gradle-warm.yml` on separate allocations.
Each verified the same `tools-r1` inputs,
set its Gradle client to read-only for the same remote namespace and executed the existing JUnit consumers.
The client setting does not establish a server-enforced read-only credential.
The image's preserved Mise executable supplies the documented installation-free bootstrap.
That bootstrap differs from the seed's Mise action,
so whole-job timing will not be attributed solely to remote compilation reuse.

Parent work separately owns Android APK/instrumented builds,
headful device consumption,
default-image graphics/KVM and prepared-image portability to GitHub Actions.
This campaign does not launch or delegate emulator work.

## Campaign work queue

The matched-control cleanup retained and removed only its inspected generated rule-cache file,
then verified no ignored state remained.
The root `HEAD`,
`config`,
`hooks`,
`objects` and `refs` ignore patterns match,
but directory listing verifies those entries are absent.
The pushed isolated worktree is retained for the coordinator's subsequent frozen Rust coverage campaign.
That new campaign owns the standalone crate catalog,
current-source linter and five fuzz sidecars,
with the same cumulative USD 5 compute proxy and two simultaneous jobs at most.

The separate Android,
runtime and BuildKit campaigns retain their own provenance and workload boundaries.
Their consumers are not additional test credit for this cache campaign.

The 18-attempt cache campaign and its three-attempt portability extension are complete
within their declared consumer boundaries.
Its native Git-policy aggregate remains failed;
its scanner target producer/reader,
Gradle controls and Detekt fixture consumers have their separate passing evidence.

## Measured fresh Gradle reuse

The unchanged warm runs are `s9w4jkv0hc` and `rrlzfmp2qv`,
both from `9f5923543640df2ee41a00ef7d098e1262a470e7`.
They have independent sandbox boot IDs and the same eight-core AMD EPYC `9R45` model as the seed.
Each restores 24 HTTP objects totaling 1,302,923 bytes.
Every restored object matches the seed's uploaded SHA-256 and length.
Both `compileKotlin` and `compileTestKotlin` report `FROM-CACHE` with identical task keys;
`test` executes rather than restoring its result.
Each run produces a fresh six-case JUnit report with zero failures,
errors or skips.

The timed `test --rerun jar` command takes 22.533 seconds for the seed,
17.550 seconds for the first warm consumer and 15.836 seconds for the second.
This includes JVM startup,
dependency access,
transform/compile restoration,
actual test execution and JAR generation.
The seed also uploads outputs;
warm consumers read only.
There is one seed sample;
the subsequently completed repeated cache-off pair is quantified in `Repeated Gradle task-output comparison`.
These results establish real reuse and the observed warm spread,
not a general provider or whole-CI multiplier.
The seed uses a Mise action while the warm jobs use the preserved executable,
so whole-job differences have another setup boundary.

The same image-backed pnpm content store restores all 582 packages with zero downloads,
but reimporting/linking takes 71.447,
75.891 and 65.810 seconds across those fresh jobs.
Pnpm's own completion reports agree with the measured command boundary.
The installed-archive strategy now has nine fresh-job observations,
from 1.434 to 1.631 seconds,
each followed by the real semantic plugin build,
host consumer and external consumer with passing exits.
The five content-store import/link observations span 57.243 to 75.891 seconds.
These are repeated preparation observations,
but the archive cohort includes a later Git/sccache tool image and different subsequent jobs.
They do not establish a fully matched causal ratio between dependency strategies.
All observations preserve the same frozen dependency files and workspace-link check.
No cause for the image-backed content-store import time has been established.

## Required-Git tool image

Preparation `3jg1fmkc69` from `90c122d9a6074d86504c40ea1b45efdb73887f1b` passed.
It extends the original image under the new tag
`l4zncjjbj5.registry.depot.dev/research-reuse-20261018:tools-r2-git256`.
The final pushed manifest is
`sha256:fb197837831d438592d9455b3a31d808a20ad8bdc2ae76b05d8b36f6656419cd`.
The original `tools-r1` remains unchanged.

The repository's existing native verification owner supplies the Git revision and build flags.
The exact upstream Git tree is `96aa7b9f343426cd28cf6c2da737b9f847a2919d` at revision
`a018953688f1b10bddf91bff8747068f5f4746a4`.
Its source build/install takes 57.829 seconds with the existing English/no-Tk choices
and offline Rust support retained.
Git `2.56.0` is installed at `/opt/depot-reuse-20261018-r1/git-2.56.0/bin/git`;
its directory must be added explicitly to `PATH`.
Provider `/usr/bin/git` remains `2.55.0`.
The image also captures checksum-verified sccache `0.18.0` so future consumers need no client download.

The incremental snapshot uploads 263,258,112 block bytes,
measures 24.196 seconds for capture/upload and 25.187 seconds for the helper,
and reuses the parent sparse/base layers.
The helper's capture/upload digest identifies its sparse layer;
it is different from the final pushed manifest digest.
The verified preparation ZIP contains 16 members and matches its upload-log SHA-256.
The source checkout and newly written synthetic source/secret markers were removed before capture.
The first fresh full Rust consumer `w16c45njf5` completed with failed application assertions,
using immutable harness `883fa0eb6363b43a82a5c8c3fb8237cc60c26da6`.
Preparation success alone does not establish a passing full Rust test suite.

The first eight completed request windows total 1,017 seconds,
or USD 0.4068 at all eight requested CPUs and the declared timestamp-price rate.
Eight remaining full job ceilings add USD 2.88,
giving a USD 3.2868 scheduling forecast before the required-Git seed.
Storage and other unmeasured charges remain excluded.

The config blob contains block-map keys,
including `metadata_chunks`,
`layers` and `chunks`.
Absent OCI environment fields or plaintext fixture values in that metadata cannot prove
that referenced filesystem blocks contain no secrets.
The filesystem/runtime fixture controls remain separate,
narrow evidence.

The initial collectors called Depot's AI-labelled diagnosis endpoint for the first five runs.
Public client source does not establish whether analysis is stored or newly generated,
or whether there is any charge.
Further collection uses run metadata,
raw logs and artifacts only.
Research conclusions use the actual tool diagnostics and receipts.

## Cache trust and lifecycle boundaries

The official [Actions-cache integration][actions-cache-doc] is a repository-scoped API replacement on Depot runners.
Its branch namespace is shared unless the workflow's key includes a trust/branch boundary.
It does not route standard GitHub-hosted `actions/cache` requests to Depot.
The separate generic Gradle and sccache endpoints use job-scoped automatic credentials in the measured Depot CI jobs;
that does not establish an equivalent automatic credential exchange on standard GitHub-hosted compute.
No broad user token was added as a GitHub secret.

The official [disk guide][cache-disk-doc] specifies organization-global names,
including cross-repository access,
and concurrent mount/read/write support.
Names partition this experiment's data;
they are not access-control boundaries.
The pinned mount action skips a public-fork pull request,
but the current private-branch settings and organization credential policy were not inspected.
Overlapping writes to a Cargo target directory or Gradle user home would require their own locking semantics;
mount concurrency alone does not make those tools safe against concurrent modification.
This campaign uses one writer,
atomic archive publication and separately admitted readers.

Mutable cache bytes are admitted with source,
tool,
profile and archive hashes retained outside the same cache.
A manifest copied solely from that writable disk would not authenticate those bytes against another writer.
Scanner reader `zq7cc1nd4b` rejected changed real source,
a partial archive and a missing manifest before restoring the admitted complete archive.
Its actual owned test consumer passed all 237 cases with no skips.
Release artifacts require their own producer/run provenance and consumer verification;
the diagnostic archive deliberately retains failing cases and cannot satisfy a success-only publication gate.

The documented disk default retention is 14 days under the organization's configured policy.
The action's post-job unmount and credential release do not delete a disk.
Snapshot `max-age: 6h` decides reuse freshness,
not a six-hour image deletion guarantee.
The research leaves named images and disks retained for its readers;
actual storage retention,
reclamation and any resulting charge remain unmeasured.
The block upload sizes and compressed target sizes are recorded as separate quantities.
They are not an account-level storage invoice.

Installed tools and dependency archives are filesystem preparation,
not a captured running service.
A fresh JVM,
cache service client,
systemd user manager or device still needs a real startup/readiness probe.
The parent's separate native-runtime findings are recorded in
[the snapshot/runtime troubleshooting note](../troubleshooting/depot-snapshot-mise-cache.md).

The separate Android investigation found an uncontrolled build-clock input in locked `turso_core 0.8.2`.
Its original build script already accepts `SOURCE_DATE_EPOCH`;
the [source-bound timestamp controls](../troubleshooting/turso-android-native-build-timestamps.md)
show matching generated constants for repeated valid seeds,
a changed constant for a changed seed and clock fallback for malformed input.
The seed's actual numeric value and validity belong in native-output and cache provenance.
Those are build-script controls,
not a complete normalized Android rebuild or proof of whole-APK byte reproducibility.

[actions-cache-doc]: https://depot.dev/docs/cache/integrations/github-actions
[cache-disk-doc]: https://depot.dev/docs/ci/how-to-guides/cache-disks

## Primary sources

- [Depot custom images](https://depot.dev/docs/ci/how-to-guides/custom-images)
- [Snapshot action source](https://github.com/depot/snapshot-action)
- [Depot Gradle integration](https://depot.dev/docs/cache/integrations/gradle)
- [Gradle 9.8.1 cache model](https://docs.gradle.org/9.8.1/userguide/build_cache.html)
- [Gradle task-specific reruns](https://docs.gradle.org/9.8.1/userguide/command_line_interface.html#sec:rerun_tasks)
- [Nextest 0.9.148 release](https://github.com/nextest-rs/nextest/releases/tag/cargo-nextest-0.9.148)
## Frozen Kotlin authored-test identity join

The [Kotlin owner projection](../audit/depot-kotlin-gradle-owner-join-2026-10-18.evidence.json)
joins the unchanged warm/off cache samples to the six authored JVM unit methods.
The entire `package/linter/kotlin` Git tree is `fa0167ad3af17896374a22b40ede76e9e9f51250`
in frozen `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`
and both executed harness revisions.
The source file is `src/test/kotlin/dev/monochromatic/detekt/RequireKDocTest.kt` within that package.
Its SHA-256 is `85aabcd1a45a34c82ed6725b3a6eb72d4e60c0c3f383998056246191f3d6806a`.

The authoritative JUnit classname is `dev.monochromatic.detekt.RequireKDocTest`.
Each repetition passed these authored methods with no failures or skips:

- `reportsUndocumentedTopLevelDeclarations()`
- `reportsMembersLocalsAndNested()`
- `reportsSecondaryConstructorAndEnumEntries()`
- `acceptsFullyDocumentedCode()`
- `skipsExcludedKinds()`
- `allowOverrideExemptsOverridingMembers()`

Warm runs `s9w4jkv0hc` and `rrlzfmp2qv` use harness `9f5923543640df2ee41a00ef7d098e1262a470e7`.
Off runs `d3ks91rnnb` and `pq5pzj9mcg` use harness `363bd501a0366dc4f74e35ad1b77f1a8ffad95b4`.
The projection retains exact workflow/job/attempt identities,
original XML hashes,
original ZIP hashes,
and source pointers into `gradleReuse.repeatedOffVersusWarm.summaries` in the canonical reuse receipt.
The independent join verified four original ZIP CRCs and `102` original member hashes.

The frozen owner `//package/linter/kotlin:test` executes `./gradlew test --no-daemon --console=plain`.
The cache experiment executes that same authored Gradle `test` task through `mise exec java`,
adding forced `--rerun`,
`jar`,
and the recorded cache/diagnostic/resource arguments.
It does not literally invoke the Mise owner wrapper.
The actual timings `17.550128155` and `15.83624124` seconds warm,
versus `21.43464331` and `20.26238305` seconds off,
include the recorded JVM/Gradle command boundary and fresh six-case JUnit execution.
They are not whole-job or general provider speed claims.
Modified controls in `r328mp420m` and the emitted-JAR Detekt host integration remain separate evidence.
Repetitions do not multiply the distinct authored-case count.
