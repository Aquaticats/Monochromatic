# Depot frozen Rust coverage research

## Research boundary

The coordinator transferred this area after the completed
[reuse campaign](depot-reuse-strategy-research.md).
The source remains `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Main-worktree changes are documentation and evidence only;
executable harnesses use the retained,
solely owned `research/depot-reuse-20261018` worktree.

## Final execution status

The admitted Rust queue is complete at frozen application source
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
All owned cloud jobs are terminal.
The final conservative compute proxy is USD 4.213,
leaving USD 0.787 under the USD 5 ceiling.
No account invoice was accessed and no real paid model/provider request was made.
The corpus audit intentionally retains an eight-CPU proxy envelope
although its immutable workflow configured four CPUs and 16 GiB;
the original adapter attribution is preserved with the correction.

- The current linter passed 415 cases with one helper skipped.
  Its actual CLI passed help/rule-list checks.
  The separate direct-rustc consumer passed after using Cargo compiler-artifact JSON paths
  and matching full-metadata rlib/rmeta artifacts.
- The ordinary core profile passed 336 cases with 13 skipped.
  WG's 11 privileged cases subsequently passed in `506m7r84kk`,
  with real kernel BPF/cgroup/network execution and default Nextest concurrency.
  Node/Tini controls reproduced and corrected the earlier initless namespace teardown failure.
  Owned cgroups were empty after cleanup and both mounts unmounted with status `0`.
- The official installed Codex host passed its unchanged native consumer.
  Stderr-retaining repeat `m138glct7m` recorded 41 localhost fixture requests,
  40 successful shell payloads,
  40 completed native hooks and zero stderr bytes.
  Reported hook milliseconds are host-resolution observations,
  not isolated overhead or provider-performance comparisons.
- All 35 source-declared fuzz targets completed bounded AddressSanitizer runs
  across the retained original profiles and selected tails.
  Every tracked seed was admitted without source/seed changes.
  The unique crate-qualified generator baselines total 67 passed cases.
  Interrupted whole jobs remain failed attempts;
  completed tail targets do not change those statuses.
- The Git owner noticed all 29 planted faults and the linter owner all six.
  Corrected actual JUnit identities were independently cross-checked against Python's XML parser
  for 30 Git reports and 1,140 testcase elements.
  The original adapter's wrong failure names remain retained as superseded evidence.
  These mutation baselines do not erase the separate full Git suite's two known failures.
- Truepeak's release `--all-features` owner passed 89 cases with two intentional skips
  out of 91 enumerated.
  This is separate from the earlier no-default-features BuildKit library producer.
- Both benchmark sidecars completed their full unchanged workloads twice in `hzfbgz4xq7`.
  Regex processed 3,783,920 lines per repetition.
  The same-image audit verified all 13,860 archive members,
  including 1,464 SHA-256/size-admitted LFS materializations.
  The Truepeak benchmark's three owner unit cases passed.
  Its first synthetic input had no feasible policy.
  Positive-fixture run `1k8ph1wk9h` then completed the unchanged search twice:
  both selected a winning candidate and produced matching 8,704-byte,
  24-row feature files.
  These synthetic peak-bin runs do not measure real audio decoding.

The executable research head is `ae56107178` on `research/depot-reuse-20261018`.
Actual immutable execution SHAs are retained per run.
This lane's main-checkout changes are documentation and evidence only.
No prepared image tag,
account setting or subscription was changed by the final consumers.

### Remaining coverage boundaries

The regex default suite intentionally skips
`pathological_nested_repetition_terminates_without_oom`:
`tests/integration.rs:547` documents its slower two-pattern regression.
Its fixed case was not separately selected;
bounded fuzzing is a distinct runtime boundary.
The linter's `panic_hook_child` and Truepeak's `crash_writer_child`
are exercised by their parent subprocess tests and are not added as new Nextest cases.
Truepeak's ignored `bench_cold_write` requires the `bench:cache` owner
and an explicitly admitted real-disk `CACHE_BENCH_DIR`;
no storage-durability benchmark is claimed.
The Truepeak sidecar's bare `cargo run` owner remains ambiguous between two binaries
without `default-run`;
actual benchmark execution selected the produced benchmark binary explicitly.
Its real-audio collector was not exercised by synthetic-bin fixtures.
The separate full Git-policy suite retains two current-source failures.
Desktop/native and Android consumers remain separately owned.
No historical desktop case count was imported into this denominator.

The prerequisite diagnoses and verified consumer corrections are recorded in
[the WG namespace troubleshooting entry](../troubleshooting/depot-wg-privileged-prerequisites.md)
and [the Git archive input-identity entry](../troubleshooting/git-archive-lfs-input-identity.md).

## Initial budget admission

The cumulative conservative compute proxy begins at USD 1.407,
leaving USD 3.593 beneath the USD 5 ceiling.
At most two owned cloud jobs may run simultaneously.
Each new bounded profile must retain its declared resources,
published-rate reservation,
source/tool hashes and actual provider identifiers before results are compared.
The initial current-source linter profile is admitted at USD 0.36,
bringing the conservative forecast to USD 1.767.
The first run is `j50kf2jlqx`,
from harness `21883c2409b5bb9a7556f869a37cfd43b6045f12`.

## Source catalog checkpoint

An uncapped `rg --files --hidden --no-ignore --glob Cargo.toml package`
returns 23 manifests in the isolated frozen checkout.
The tracked immutable tree contains the same manifest paths and no root `Cargo.toml`.
Those are source inventory facts,
not passing test or build credit.

The first metadata pass stopped when Python 3.14's `tomllib` rejected
`package/ide/app/Cargo.toml:68` with
`Invalid initial character for a key part (at line 68, column 105)`.
That line uses an inline-table trailing comma.
The repository already documents this
[TOML version boundary](../troubleshooting/taplo-toml-1-1-inline-table-trailing-comma.md).
The inventory must use Cargo's actual parser before classifying that source as invalid.
The installed `cargo read-manifest --help` exposes a frozen,
offline manifest-reader invocation and marks it deprecated in favor of metadata.
Any use here is an inventory probe,
not a proposed production tooling replacement.

Cargo 1.98.1 accepts all four manifests rejected by Python 3.14.8's TOML parser.
The complete [source catalog](../audit/depot-rust-coverage-2026-10-18.catalog.json.gz)
retains raw manifests,
owning Mise tasks,
lock identities and tracked source hashes.
Cargo-normalized absolute paths are paired with resolved repository-relative dependencies.
An empty or self-member `[workspace]` still describes a standalone crate;
it does not establish membership in a missing root workspace.

The manifest inventory contains five fuzz sidecars,
two benchmark sidecars and sixteen other packages.
The declared `fuzz_targets/` binaries total 35:
five scanner,
five regex,
four JSONC,
fifteen Git-policy and six linter targets.
Actual `cargo fuzz list` and executed target receipts remain separate runtime gates.
Committed seed files total 46 for scanner,
65 for Git-policy,
10 for linter and three for JSONC.
No tracked regex seed files exist at this source.
The first inventory matcher omitted singular `seed/` directories;
that scope error was corrected before assigning seed counts or execution credit.

### Manifest execution boundaries

The catalog records each manifest,
source closure,
lockfile,
path dependency and actual owning Mise configuration.
Its execution classifications are:

- `package/forbidden-strings/cli`:
  the same frozen source passed its complete 237-case release Nextest owner
  in producer `v7njncr47d` and fresh reader `zq7cc1nd4b`.
  The [reuse evidence](../audit/depot-reuse-strategy-2026-10-18.evidence.json)
  retains its source/tool admission and independent mutation control.
- `package/git-policy/cli`:
  the same frozen source executed 1,951 cases with 1,949 passed and two failed,
  plus five skipped out of 1,956 enumerated in `n2zlbh5gpb`.
  Its Git 2.56 and published-linter prerequisites,
  filesystem controls and later selected 34-case archive replays remain separate boundaries.
  It does not have a passing full-suite result.
- `package/linter/monochromatic-lint`,
  `package/jsonc-edit/core-rust`,
  `package/forbidden-strings/core`,
  `package/wg/cli-quicker-exempt` and `package/codex-plugin/bash-limit`:
  this campaign retains their actual Nextest results.
  Privileged BPF cases and the installed Codex host are separately gated.
- `package/forbidden-strings/cli.fuzz`,
  `package/git-policy/cli.fuzz`,
  `package/linter/monochromatic-lint.fuzz`,
  `package/forbidden-strings/core.fuzz` and `package/jsonc-edit/core-rust.fuzz`:
  their GNU/AddressSanitizer,
  generator,
  seed and planted-control boundaries are the active queue.
  Listing targets does not count as executing them.
- `package/music-player/truepeak-core`:
  the [BuildKit campaign](../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json)
  retains exact-source AMD64 release library production and offline registry/target-cache controls.
  That producer uses Rust/Cargo `1.97.1` with `--no-default-features`,
  whereas the owning `test` task requires release Nextest with `--all-features`.
  No unit-test,
  `service` or `fingerprint` credit transfers from that producer.
- `package/music-player/android-app/rust`:
  [Android producer receipts](depot-complete-coverage-android-junit-execution.json.gz)
  retain JNI/ELF/APK production at the frozen source.
  Its 1,289 JVM cases and
  [headful device consumer](../audit/depot-android-consumer-2026-10-18.evidence.json)
  are consumer boundaries,
  not standalone Rust unit-test counts.
- `package/nested-wayland-session/cli`,
  `package/file-manager/app`,
  `package/file-manager/variant-gtk-sticky`,
  `package/file-manager/variant-qt`,
  `package/ide/app`,
  `package/terminal/app` and `package/music-player/desktop-app`:
  the catalog retains their actual desktop owners and dependency graphs.
  They involve Smithay/Wayland,
  GTK,
  Qt,
  Slint,
  Ghostty or platform audio dependencies respectively.
  Source-specific desktop orchestration belongs to the separate native/routing investigation.
  Historical desktop tests do not enter this campaign's current-source denominator.
- `package/music-player/truepeak-core.bench` and `package/forbidden-strings/core.bench`:
  these are benchmark sidecars with independent build/run owners.
  No benchmark or parent-package test pass transfers merely from their manifest presence.

The [historical feature audit](../audit/depot-test-feature-2026-10-16.md),
[prepared desktop record](../audit/depot-prepared-desktop-2026-10-16.md)
and [expanded execution record](../audit/depot-expanded-execution-2026-10-16.md)
remain historical evidence until their source,
dependency and tool identities match the requested boundary.

## Initial current-source linter profile

The isolated workflow is `.depot/workflows/rust-coverage-linter.yml`,
with an 8 CPU,
32 GB allocation,
15-minute job limit and 12-minute application-step limit.
The reservation uses USD 0.00005 per CPU-second:
`8 * 900 * 0.00005 = USD 0.36`.
This is a timestamp-compute research guard,
not an account bill.

The prepared native image is `research-reuse-20261018:tools-r2-git256`,
bound to `sha256:fb197837831d438592d9455b3a31d808a20ad8bdc2ae76b05d8b36f6656419cd`.
The profile installs no replacement application binary.
It prepares the source linter's locked dependencies,
then executes `cargo nextest run --locked --offline --all-targets --no-fail-fast`.
The owning Mise configuration's compiler fan-out remains two.
Development debug symbols and incremental compilation are disabled,
matching the existing tested container preparation settings;
Nextest retains default test concurrency.

The produced current-source CLI must answer its real help and rule-list commands.
The profile also builds the library artifact,
compiles the existing `src/processors_consumer.rs` against it,
and runs that consumer.
JUnit includes skipped cases and failure output.
Application source,
locks and owner configuration are checked against the frozen tree before and after execution.
Failures remain failures while receipts upload independently.
The source linter's Cargo lock contains 294 packages and pins the rust-analyzer family to `0.0.336`.
That compilation and its own tests are distinct from using the published `0.1.3` prerequisite.

## Standalone and native prerequisite profile

The next isolated workflow is `.depot/workflows/rust-coverage-core.yml`,
with 4 CPU,
16 GB,
a 15-minute job limit and a 12-minute application-step limit.
Its USD 0.18 reservation brings the still-conservative forecast,
including the full initial linter reservation,
to USD 1.947.
The reviewed owner boundaries are release Nextest for `forbidden-regex`,
`jsonc-edit` and `codex-plugin/bash-limit`,
plus ordinary debug unit tests for `wg-quicker-exempt` adapted from Cargo test to Nextest.
Build and test concurrency retain their defaults.
Privileged ignored BPF cases and the installed Codex host case remain explicit separate prerequisites.

The profile also builds the frozen `wg-quicker-exempt` release executable,
checks its actual usage and invalid-zero-mark boundary,
and retains the ELF with source and tool hashes.
That artifact is the missing producer for the routing investigation's
`finds repository sibling release outside secure path` consumer.
Release-file presence by itself is not counted as that consumer passing.

## Completed consumers and retained failures

The [evidence ledger](../audit/depot-rust-coverage-2026-10-18.evidence.json)
binds provider run metadata,
command logs,
source hashes,
JUnit and artifact member hashes.
Both downloaded artifact ZIPs match the provider-reported upload SHA-256 and pass CRC validation.
The [compressed receipts](../audit/depot-rust-coverage-2026-10-18.receipts.json.gz)
retain full command logs and JUnit rather than converting the owning task into a fictional inner test.

Run `j50kf2jlqx` compiled the current linter and passed 415 Nextest cases,
with zero failures,
one skipped case and 416 enumerated cases.
The produced CLI passed actual help and rule-list assertions.
Its SHA-256 is `e698c16275e71a142cd95538c59c0543ab94a6ce3a382727c889e0dbe58da88d`.
The job remains failed because the separate existing library consumer's direct Rust invocation reported:

```text
// existing-processor-consumer-build.log
error: only metadata stub found for `rlib` dependency `monochromatic_lint`
please provide path to the corresponding .rmeta file with full metadata
```

The application tests passed;
the standalone consumer did not run.
The source was unchanged before and after execution.
Rust `1.101.0-nightly`,
commit `5d8fec0669a64607db9ccbf663e7c23125bf4748`,
contains an explicit compiler control passing both the link archive and its matching full metadata.
The next harness-only recovery will test that exact calling convention.
The first run retains an executable hash,
not a downloadable executable handoff.

Run `lwp0vbb2vq` completed successfully from harness `3a7e85d395cb67a5d189224963782c9b45527c71`:

- `wg-quicker-exempt`:
  15 cases run and passed,
  zero failed,
  11 skipped and 26 enumerated.
- `jsonc-edit`:
  81 cases run and passed,
  zero failed or skipped and 81 enumerated.
- `forbidden-regex`:
  235 cases run and passed,
  zero failed,
  one skipped and 236 enumerated.
- `codex-plugin/bash-limit`:
  five cases run and passed,
  zero failed,
  one skipped and six enumerated.

The native release producer also passed its usage and zero-mark controls.
Artifact `01a14e69-58c8-7104-88fd-fea5620fbe2f` retains the 473,368-byte executable,
SHA-256 `02053fc5338835abb4d6a7812a3b71244a2190512724040219cdd68595dee921`,
with its source and tool manifest.
The routing consumer has received this concrete handoff;
its result belongs to that separate investigation.

The completed requested-to-provider-finish proxies add USD 0.0536 and USD 0.0314 respectively.
The cumulative owned compute proxy is USD 1.492,
leaving USD 3.508 beneath the research guard.
This calculation rounds each observed interval upward to whole seconds and includes startup,
checkout and artifact upload;
it is not an invoice or measured CPU utilization.

## Next admitted preparation and recovery

The next profile reserves another USD 0.36 at 8 CPU,
32 GB and 15 minutes,
bringing the conservative forecast to USD 1.852.
It will verify lock-bound `cargo-fuzz 0.13.2`,
prepare the official `codex-cli 0.163.0` release for the separate deterministic host fixture,
repeat the real source-linter tests and existing library consumer with full metadata,
and retain the actual tested CLI bytes.
It will then seal a distinct native tool snapshot,
excluding the source checkout and synthetic secret fixture before capture.
The image is preparation evidence,
not an ordinary OCI container image or proof of general credential exclusion.

The Codex release's GitHub tag is not immutable.
The official compressed asset and signature bundle are therefore bound by their fetched SHA-256 values;
source and signature provenance remain explicit gates before any host-fixture execution.
Version and help inspection do not constitute a model turn.
The installed-host case must use its owned home,
local deterministic response provider and recursion guard.
Actual ASan/GNU fuzz targets,
existing planted controls and seed immutability remain in the owned queue.

## Owned work queue

### Recovered ordinary consumer

Run `zxlt43t7g1`,
from harness `ee873be6f753c611b15cbfd0e925d0048a631dbb`,
finished successfully.
The unchanged `processors_consumer.rs` compiled against Cargo-discovered archive and matching full metadata,
then printed `processor artifact consumer passed`.
The fresh sandbox verified the prepared cargo-fuzz and Codex bytes,
registry manifest digest,
synthetic source/secret exclusion markers,
and unchanged source before and after execution.
This gives `tools-r3-fuzz-codex` ordinary-consumer evidence;
it does not represent a Codex model turn or fuzz execution.

Artifact `01a14e95-7e1f-70e2-a82d-b8ea65ea6fd7` passed its upload SHA-256 and ZIP CRC checks.
The recovery retains the preceding failed runs separately.
Its 197-second requested-to-finish upper bound adds USD 0.0394,
bringing the cumulative compute proxy to USD 1.6564,
with USD 3.3436 remaining.

The next profiles reserve USD 0.36 each:
bounded GNU/AddressSanitizer fuzz execution and the deterministic installed Codex host fixture,
each limited to 8 CPU,
32 GB and 15 minutes.
Their combined conservative forecast is USD 2.3764.
Runtime fuzz processes remain mount-free,
network-disabled containers with 2 CPU,
2 GiB memory and 128 tasks.
Committed corpora remain source inputs;
only owned copies receive generated units and crash artifacts.

### First bounded fuzz and host attempts

`bl31brxwpm` passed the official Rust AddressSanitizer stack-overflow positive control
inside the bounded mount-free runtime.
It then stopped before application compilation:
the harness had merged Mise's `WARN missing:` stderr into `cargo fuzz list` stdout
and rejected the warning as an unexpected target.
That is a research admission failure,
not a failed fuzz target.
Harness `43977f1cca` preserves each stream independently and parses only target stdout.

`c4p5g7lx48` exercised the existing ignored Codex host case with the signature-admitted `0.163.0` executable.
Its local deterministic provider accepted all expected payload responses,
and the host reported 40 completed hooks.
The actual Nextest case nevertheless failed at `tests/native_host.rs:142`,
the final membership-file PID assertion.
No full installed-host pass is claimed.
The next diagnostic consumer retains the unchanged fixture's wire messages and synthetic home;
it does not change application source or substitute an external model.

The two failed jobs add USD 0.05 and USD 0.0182 respectively.
The cumulative proxy is USD 1.7246,
leaving USD 3.2754.
The corrected fuzz retry reserves USD 0.36,
and the host diagnostic reserves USD 0.18,
for a combined forecast of USD 2.2646.
Separate Git-policy fuzz,
linter fuzz and their existing planted controls remain required work.

The tool-preparation run `jf0c9llb1v` finished with the same 415 passing Nextest cases,
1 skipped and 416 enumerated,
and retained a real executable transport artifact.
Its separate consumer recovery stopped at the harness's fixed `debug/deps` directory lookup,
before compilation.
The [compiler metadata trace](../troubleshooting/rustc-full-metadata-artifact-consumer.md)
keeps that failure distinct from the archive-only compiler diagnostic.
Tool preparation and source/synthetic-secret sealing passed;
the resulting native snapshot is `tools-r3-fuzz-codex`,
manifest `sha256:1c2b469a9d7fae8bc364e6392f771c28565647a04c40847f345bc140dfce4e88`.
Its fresh ordinary consumer remains a separate required check.

The completed run adds a conservative USD 0.1148 timestamp proxy,
bringing the cumulative proxy to USD 1.6068.
The next ordinary library consumer reserves USD 0.18 at 4 CPU,
16 GB and 15 minutes,
for a forecast of USD 1.7868.
It will discover the actual archive,
full metadata and dependency directories from Cargo's compiler-artifact JSON,
execute the unchanged public processor consumer,
and verify prepared tool identities plus the image's synthetic exclusion controls.
This focused profile does not rerun the already measured 415-case linter suite.

The exact core `wg-quicker-exempt` ELF was consumed in routing run `39118628603`:
routing reports 44 passing owning invocations,
including the actual default tests.
The one-object signed URL was retained only in an owned mode-0600 file,
then removed after consumption.
Routing verified exact temporary Actions-secret deletion with HTTP 404.
Its per-case assertion and artifact bindings belong to the separate routing evidence.

- Classify every standalone manifest,
  its actual Mise owners,
  source/dependency/toolchain hashes,
  targets and runtime prerequisites.
- Match prior exact-source evidence by its input hashes and command boundary before assigning carryover credit.
- Build and consume the current rust-analyzer-backed `monochromatic-lint` source,
  including its actual Nextest and executable consumers.
  The published `0.1.3` tool in the preceding campaign was a prerequisite,
  not current-source linter coverage.
- Exercise applicable active standalone crate owners through Nextest,
  retaining real failures,
  ignored cases and compilation prerequisites separately.
- Exercise `forbidden-strings/cli.fuzz`,
  `git-policy/cli.fuzz`,
  `linter/monochromatic-lint.fuzz`,
  `forbidden-strings/core.fuzz` and `jsonc-edit/core-rust.fuzz`.
  Inspect the actual owning tasks and target bodies first;
  preserve the GNU target,
  AddressSanitizer,
  planted controls and explicit runtime limits.
- Hash committed seed corpora before and after;
  direct generated seeds,
  mutations and crash artifacts only into owned writable state.
- Reconcile raw command,
  source,
  artifact and cleanup receipts;
  lint and render the resulting documents before closing the campaign.

The separate platform investigation's `depot cargo` command configures local Cargo with sccache.
It does not introduce another remote compiler pool.
Its [wrapper source audit](../troubleshooting/depot-cargo-cache-wrapper.md)
also records authentication precedence and removal of inherited `SCCACHE_*` tuning.
Any use must preserve this campaign's isolated cache namespace and job-scoped credential boundary.

### Retained installed-host diagnostic

Diagnostic `xdv4v475fq` at harness `ecacdf9e31810fa5961f3fe4351141a7dbf9f8c1`
retains the unchanged failing installed-host Nextest case,
40 synchronous hook completions and an empty synthetic membership file.
The transparent protocol tee records no `commandExecution` items.
The retained Codex SQLite log records 30 filesystem sandbox `policy_denied` events before its final flush.
Codex also reports its missing external `bubblewrap` and bundled fallback.
These observations do not yet identify the shell launch failure.

`tests/host_support/server.rs` checks whether each raw provider request contains `host-payload-executed`.
That check can match the preceding function-call arguments,
so it does not establish successful tool output.
The next diagnostic captures the same localhost provider traffic without modifying frozen application or test source.
The host used an unauthenticated local Responses fixture;
no paid provider request was admitted.
Its fresh home also fetched the default public plugin catalog and received an unauthenticated featured-catalog 401.
The artifact retains that background behavior,
while durable raw receipts omit downloaded plugin payloads and retain their individual hashes.

Completed conservative compute proxy is USD 1.7458 of the USD 5 ceiling.
The only active reservations are `jwg2cmvwd1` (small fuzz retry) and `8rh0797pqx` (Git-policy fuzz),
each bounded to 8 CPUs,
32 GB and 15 minutes with a USD 0.36 reservation.
The reservation-inclusive bound is USD 2.4658.
Truepeak all-features,
the remaining fuzz/planted owners,
transparent host capture,
benchmark repetitions and privileged WG prerequisite probes remain the explicitly tracked queue.

### Bounded fuzz checkpoint and sandbox prerequisite

`jwg2cmvwd1` completed all four JSONC and five regex fuzz targets,
plus four scanner targets before the 12-minute step limit terminated `fuzz_scan_format`.
The completed targets report nonzero libFuzzer executed-unit counts,
clean exit status and final statistics.
The ASan stack-overflow positive control failed as expected inside the same resource isolation.
JSONC generator Nextest passed 13 cases;
scanner generator Nextest passed six.
The regex sidecar enumerates zero generator tests and earns no such credit.

`8rh0797pqx` passed 38 generator cases and completed 14 Git-policy fuzz targets.
Its last target,
`wrapper_controls`,
was also interrupted by the 12-minute step limit.
Neither interrupted run reaches the final source admission or complete runtime cleanup.
Their initial frozen-source hashes,
completed per-target container state,
corpus copies,
raw logs and provider-verified ZIP digests remain durable.
The owned sandbox terminates after each job.
The targeted `fuzz-tails` profile replays only these interrupted targets and retains final source admission.

The [Codex sandbox prerequisite diagnosis](../troubleshooting/codex-standalone-bubblewrap-prerequisite.md)
records why hook completions did not prove shell execution.
`mwwmknc37b` captured 41 localhost provider requests;
all 40 returned shell tool outputs have code `101` with a missing Bubblewrap launcher panic.
The signed standalone binary and application sources remain unchanged.
The owning fixture is being retried with distribution Bubblewrap installed only in the disposable job.

The reconciled completed compute proxy is USD 2.3972.
The host prerequisite rerun and combined tail replay reserve USD 0.54 together,
keeping the declared bound at USD 2.9372 of USD 5.
The remaining admitted queue is linter fuzz,
Git-policy and linter planted controls,
truepeak release all-features Nextest,
privileged WG namespaces and the bounded repeated benchmark owners.

### Passing installed Codex boundary

`1h6qf8j86f` passed the unchanged installed-host Nextest case after admitting
`bubblewrap 0.9.0-1ubuntu0.3` in its owned disposable sandbox.
The package verification,
`/usr/bin/bwrap` SHA-256 and actual namespace probe are retained.
The signed Codex `0.163.0` executable is unchanged.

The final captured localhost request contains all 40 actual `function_call_output` items.
Each has code `0` and exactly the expected `host-payload-executed` output.
The native test independently passes its rewritten command membership-file assertion.
All 40 host-reported hook durations are `1 ms`;
this is one fixture sample at the host reporting resolution,
not a comparative overhead estimate.
The structured `consumerSummaries.installedCodex` receipt records this boundary.
It exercises trusted hooks and a synthetic membership file through the real installed host,
not real kernel cgroup placement or paid model execution.

Completed compute proxy is USD 2.418.
The active fuzz tails `w96195cl04` and linter fuzz `v17djp58z3` reserve USD 0.72,
for a reservation-inclusive bound of USD 3.138.

### Admitted remaining native and benchmark prerequisites

The [truepeak owner](../../package/music-player/truepeak-core/mise.toml) is
`cargo nextest run --release --all-features`.
Its `service` graph enables Turso,
Tokio and the `branches` standard-library workaround;
`fingerprint` enables XXH3.
`src/service_tests.rs` supplies fake decoded samples.
`src/cache_tests.rs` uses `:memory:` databases and uniquely named disposable temporary files.
Its crash tests own and kill their writer children after bounded progress,
with four cycles for each batched synchronization setting and 12 row-write control cycles.
The ignored `crash_writer_child` is invoked only by those parents.
The ignored `bench_cold_write` is a distinct disk benchmark needing `CACHE_BENCH_DIR`;
it is not selected by the ordinary all-features test owner.
The admitted profile preserves default Nextest build and test concurrency.
The previous no-default-features BuildKit producer does not exercise this feature graph.

The [WG functional owner](../../package/wg/cli-quicker-exempt/mise.toml) selects 11 ignored tests.
`src/pin_tests.rs` names their requirements explicitly:
root and writable cgroup v2 for protocol/partial-attach and application-watcher cases,
plus a distinct bpffs mount for persisted links,
replacement,
process-owner identity and transition recovery.
`src/bpf.rs` uses `BPF_PROG_TYPE_CGROUP_SOCK_ADDR`,
`BPF_PROG_LOAD` and `BPF_LINK_CREATE`,
with IPv4/IPv6 connect and UDP sendmsg hooks.
`src/pin.rs` requires bpffs magic `0xcafe4a11` at `/sys/fs/bpf`.
The admitted harness first builds the normal debug CLI and test artifacts,
then enters private mount,
network,
cgroup and PID namespaces under root in its owned ephemeral Depot sandbox.
It mounts new cgroup2 and bpffs views,
raises only loopback,
records actual capabilities/mounts and runs the same ignored cases through Nextest.
No user namespace or real account cgroup is used.
Before/after cgroup inventories and exact owned cleanup remain required evidence.
Namespace,
mount or BPF admission failures will remain prerequisite failures.

The [regex benchmark owner](../../package/forbidden-strings/core.bench/mise.toml)
keeps its release LTO and single-codegen-unit build.
`src/main.rs` has eight ten-second throughput loops;
`src/kernels.rs` has 33 four-second bucket loops,
two four-second set loops and three four-second attribution loops.
That totals at least 232 timed seconds per complete repetition,
plus setup,
parity and complete corpus passes.
Whole-pass work can exceed a timer budget.
The real ruleset builder raises both regex memory limits to `1 << 30`;
the bounded container must report any OOM or time termination separately.
The corpus walker reads non-ignored text lines up to 4,096 bytes,
with no overall line-count cap.
The admitted input is a baked exact frozen `git archive`,
its frozen ignore policy and an empty `.git` root marker,
with no local runtime caches or untracked files.
Two unchanged repetitions keep every existing loop intact.
Each receives a native 300-second container lifetime limit,
2 CPUs,
2 GiB,
128 processes,
no network and no host mounts.
Only completed repetitions can establish observed spread.

The [truepeak benchmark owner](../../package/music-player/truepeak-core.bench/mise.toml)
reads tracks JSONL rather than decoded audio.
The collector is a separate executable needing actual audio and `ffmpeg`.
The search executable takes the complete track measurements,
then applies its fixed exhaustive policy grids.
The admitted repetitions use the exact release executable,
unchanged default grids and 24 deterministic synthetic tracks,
with output files confined to each fresh container.
They establish fixture execution and repeated behavior,
not production audio quality or real-library performance.
The crate declares two executables and no `default-run`;
its existing `cargo run --release --` selection is probed separately before explicit-binary execution.
Its existing debug Nextest owner is also included.
Both benchmark sidecars remain separate from the truepeak all-features library tests.

### Planted Git-policy controls and attribution correction

`gm2drnx19m` ran the unchanged Git-policy fuzz sidecar planted-control owner against disposable copies.
The actual Nextest baseline passed all 38 generator cases.
All 29 planted faults produced real failed Rust tests in the corresponding retained JUnit reports.
These 29 expected failures are controls,
not new product failures or extra distinct test cases.

The first adapter incorrectly attributed failure names when a self-closing passing JUnit case preceded a failing case.
Its greedy attribute capture could consume that passing case and pair its name with a later failure body.
The synthetic identity regression was committed at `2d32fbe86f`,
then reproduced exactly that mismatch before the parser correction.
An independent Python `xml.etree.ElementTree` parse of every actual report confirms 38 cases per report,
a clean baseline and real failures in all 29 planted variants.
`consumerSummaries.gitPolicyPlanted` rebuilds exact failure names from that preserved XML.
Original adapter names remain retained but are explicitly superseded.
No application test was rerun during this correction,
and no source or receipt was discarded.

Linter fuzz `v17djp58z3` passed its generator controls and completed `configuration`,
`markdown`,
`merge_values` and `orchestration` before its step limit interrupted the next target.
The exact frozen target list names the remaining targets `rust_explicit_types` and `rust_style`.
A prior commentary mislabeled the latter as TypeScript;
that scope error was corrected from the actual CLI listing and frozen filenames before dispatch.
The target-filter assertion would reject an incomplete selection.
The dedicated replay is `6z1fwznlrx` at `43a51584d0f3ab389490fbe010ec733aafc05cc9`.
It does not repeat already-passed generator tests.

Completed compute proxy is USD 3.0884,
with USD 1.9116 remaining.
The replay reserves USD 0.36.
The tracked queue is corrected-parser linter planted controls,
truepeak all-features Nextest,
WG privileged namespaces,
bounded benchmark repetitions and one final Codex stderr-retention pass.
The last pass retains successful shutdown stderr,
which the original passing fixture deletes with its temporary home.

### Completed generator, all-features and mutation boundaries

All 35 targets now have a completed bounded ASan run with status `0`,
a positive executed-unit count and no recorded container OOM.
The target inventory comprises five scanner targets,
15 Git-policy targets,
six linter targets,
five regex targets and four JSONC targets.
The completed linter tail `6z1fwznlrx` ran `rust_explicit_types` for 460 units
and `rust_style` for 881 units.
The final replay source-admission checks passed.
Earlier step-limit interruptions remain separately retained;
they do not become successful whole-job receipts.
These are smoke bounds of 30 seconds per target with maximum input length 4,096 bytes,
not exhaustive fuzz coverage.
The generator baselines have 67 distinct crate-qualified cases:
scanner six,
Git policy 38,
linter ten and JSONC 13.
The regex sidecar enumerates no ordinary generator tests.
Repeated baselines and tail replays add no distinct-case credit.

Truepeak all-features `221vdm1f21` at `a85e204f078db540c2a292342000ce5423bb8338` passed 89 tests,
with two skips out of 91 enumerated.
The exact skips are `cache::tests::bench_cold_write` and `cache::tests::crash_writer_child`.
Parent-driven crash recovery cases executed;
the writer child is invoked and killed by those parents.
The existing service fixtures use fake samples,
with disposable database paths and no real provider calls.
The command retained `--release --all-features` and ordinary Nextest concurrency.
This is a separate feature graph from the BuildKit `--no-default-features` producer.

Linter planted owner `v8lrx25bl6` at `df00ddbcdfdc9e67759890873c9ca911923a727e` passed its ten-case baseline
and detected every one of its six source-defined faults.
Each faulty copy has real failed Rust tests in its exact retained JUnit report.
The corrected adapter also independently matches Python's standard XML parser
for every identity and status in all 1,140 cases across the 30 retained Git-policy reports.
This local parser cross-check does not rerun application tests.
The original wrong attributions remain preserved and superseded.

A proposed tightening of the existing `AGENTS.md` `QIV` text addresses this evidence gap:
"Before trusting a null or count,
validate scope,
cache,
harness,
generator reach and parsed case identities with a positive control;
list unexercised surfaces."
This is a proposal only;
`AGENTS.md` is unchanged.

### Final native reservations

The first privileged WG attempt `42h9g5sczp` stopped before compilation or namespace creation.
The harness incorrectly required status `0` from `ip --help`;
the actual tool printed its valid usage text and returned `255`.
Upstream `ip/ip.c` sends help through `usage()`,
which calls `exit(-1)`.
The corrected boundary accepts only status `0` or `255` together with the expected usage header,
while functional commands still require success.
The intervening `c9259269e5` workflow had an indentation error
and was rejected by the dispatch parser before receiving a native run ID.
The separate observed push-parse event is `ps_96gh1gkb1z`.
The committed malformed workflow is now a positive control:
PyYAML 6.0.3 rejects it,
then parses all 16 current Rust workflows.
The corrected actual WG retry is `8gts453b5g` at `d9127f3a9f1d92e09820703bf5844e73a86adb1b`,
with a ten-minute job maximum and an eight-minute execution step.
The 11 ignored tests retain their normal Nextest concurrency and existing per-test bound.
This reserves USD 0.24.

The benchmark native lifetime limit was expanded from the initial 300-second plan
to 600 seconds per invocation before any benchmark dispatch.
The source-defined 232-second minimum regex loop remains unchanged,
with two complete repetitions of each executable.
The job maximum is 50 minutes,
with a 48-minute execution step,
for a conservative USD 1.20 reservation.
Frozen Git-object measurement found 13,860 entries totaling 366,655,696 blob bytes.
That exposed an insufficient 256 MiB stdout buffer before execution.
The driver now uses native `git archive --output` and streams its checksum.
A retained 10,240-byte selected-file control verifies native file output and streamed hashing.
The frozen attributes contain no export-ignore or export-subst directives.
The initial plan described the corpus as raw Git-object bytes with LFS pointer text.
That description was too strong:
Git archive calls working-tree conversion before writing regular files.
The executed archive is 602,992,640 bytes,
with SHA-256 `47698c6be07012ab111e7586c1fd23d8b1f610ca5d3f570e53147da9a30ad368`.
A same-image audit now checks every member and any LFS materialization
against its content-addressed frozen source.
Actual benchmark job `hzfbgz4xq7` uses harness `111910f0352b0e65e8f3f0befa61282915b90e2c`.

At this checkpoint completed compute proxy is USD 3.4492,
with USD 1.5508 remaining.
The benchmark and WG reservations give a cumulative bound of USD 4.8892.
They are the only active owned jobs.
The remaining independent queue is the final installed Codex stderr-retention pass;
its admission waits for reconciled capacity and budget.

### Benchmark execution and final isolation checks

Benchmark `hzfbgz4xq7` completed all four native invocations with status `0` and no recorded OOM.
The regex repetitions each reached all throughput,
bucket,
set and line-matches sections over 3,783,920 lines.
Their container intervals were 359.21 and 364.80 seconds.
The same-engine throughput range was 9,162,279 to 10,054,431 lines per second.
These are unchanged-input samples;
no Depot/vendor performance comparison is established.
Generic runtime fields named `finished` and `executedUnits` detect libFuzzer output
and do not describe benchmark completion.
Benchmark completion is established by actual exit status,
full workload output and retained container state.
The Truepeak debug Nextest owner passed three cases.
Its unchanged search executable processed 24 synthetic tracks twice
and reported no feasible candidate in either repetition.
The winning-policy/output-file branch is not credited.
The original bare Cargo run owner separately failed with status `101`
because it declares two binaries without `default-run`;
explicit execution used the produced `truepeak-core-bench` artifact.

WG retry `p81kntfxgs` reached all 11 privileged cases:
five passed and six failed in process-identity teardown checks.
Actual protocol-hook marking/drop cases passed.
The writable private cgroup bind and new bpffs mounts were admitted.
The namespace used Node as PID 1;
source `keeper_process.rs` polls `/proc` after signaling
and only reaps a holder when it is the direct parent.
Unreaped orphan state is therefore an explicit hypothesis,
not yet a product-failure attribution.
The next bounded retry `4hddgrkrk2` adds a real Tini init
and independent zombie/reaping controls before running unchanged tests.
Normal Nextest concurrency is preserved.
The earlier non-lazy cgroup unmount failure remains retained.
A fallback lazy detach is restricted to the private bind view;
namespace and owned VM teardown end remaining process references.

Installed Codex stderr-retention run `m138glct7m` passed its native host test.
Its actual retained stderr and protocol receipts still require final classification.
Completed compute proxy at this checkpoint is USD 4.024,
leaving USD 0.976.
The same-image corpus audit is `8ksdxnqq25`
and the controlled-init WG retry is `4hddgrkrk2`.
They are the only active owned jobs,
with a conservative combined reservation of USD 0.48.

### Closed final reservations

WG run `506m7r84kk` passed all 11 ignored cases after both init controls passed.
The Node control observed orphan state `Z`;
the Tini control observed the orphan absent.
Cgroup inventory after cleanup was empty and both ordinary unmount commands returned `0`.
The original six failed reports remain unchanged.

The corpus audit verified that the executed archive exactly matched its recorded digest,
with 13,860 members and no missing entries.
All 1,464 raw-blob mismatches matched frozen LFS pointer SHA-256 and size.
The earlier pointer-only attribution is superseded.

Positive Truepeak benchmark run `1k8ph1wk9h` used 24 long flat-peak tracks.
The full unchanged grid selected window count `14`,
threshold `60` seconds and margin `0.300` dB.
Decoded budget was `1,440` seconds against target `4,359`.
Both repetitions emitted 24 valid feature rows,
8,704 bytes and SHA-256
`6b353d33aa7b2acda2090abfa6f4d69b71fb15a9ea5096d80bc6d144c7723cbd`.
Their container intervals were 0.1942 and 0.1957 seconds.
The feasible no-classifier branch also produced actual ranked output.
This closes winning/output emission for the synthetic fixture,
while real audio collection and decode performance remain uncredited.

Final completed conservative compute proxy is USD 4.213.
No owned job or reservation remains active.
The final executable adapter syntax check passed all admitted Rust research bins.
The YAML check rejected the committed malformed positive control
and accepted every admitted Rust workflow.

## Independent evidence review

The coordinator's independent read-only evidence review is complete for the job-policy/history,
process-timeout and original MVM/KVM audit bundles dated 2026-10-18,
the combined-product pricing calculation and the root expanded-CI report outside this lane.
The additional service/container and disposable-PR cohorts and container prerequisite source reasoning
are also reviewed.
The published independent note retains original-byte,
CRC/hash/outcome joins,
credential inspection and fresh allowance/source checks.
It identified the accepted original MVM shared-directory qualification correction;
no additional material correction was found in the service/PR cohorts.
The independently reviewed new three-ZIP guest-transformation cohort passes its exact original-byte,
guest-command and cleanup joins without upgrading the original host shared-directory assertion.
No new cloud allocation is admitted for this reviewer.

## Reusable immutable Rust profile handoff

The coordinator requested these exact existing profiles for a repeated Linux DAG.
This is a source handoff,
not another execution or allocation.
Both retain ordinary Cargo build and Nextest test concurrency:
the adapters explicitly delete `CARGO_BUILD_JOBS` and set no test-thread override.

### Scanner producer and admitted consumer

Producer `v7njncr47d` used harness `360abf1382631750c3f26928c2342d0acd13eb6b`
and `.depot/workflows/reuse-scanner-seed.yml`.
Reader `zq7cc1nd4b` used `670d84778d24567e36b01a22c33833310d44d659`
and `.depot/workflows/reuse-scanner-target.yml`.
Both invoke `//package/monorepo-task/core-exec:research:reuse:scanner` through Mise.
The implementation is `package/monorepo-task/core-exec/bin/reuse-scanner-consumer.mjs`,
importing `reuse-context.mjs`.
The real owner is `//package/forbidden-strings/cli:test`,
whose frozen task runs `cargo nextest run --release`.
The adapter forwards `--locked --offline --no-fail-fast`.
The admitted baseline passes 237 cases without skips;
its additional dependency-feature control is separately labeled.

Both workflows use `tools-r2-git256`,
image digest `sha256:fb197837831d438592d9455b3a31d808a20ad8bdc2ae76b05d8b36f6656419cd`.
They first invoke the registry and tool-admission tasks in the same task-util package.
The prepared image manifest is `/opt/depot-reuse-20261018-r1/image-manifest.json`.
The real Archil mount is `/mnt/aquaticats-monochromatic-reuse-20261018-scanner-target-trusted-r1`.
Its producer publishes `scanner-target.json` and `scanner-target.tar.zst` only after real tests and controls pass.
The manifest is also retained as
`.research-reuse/receipts/scanner-success-only-target-manifest.json`.
Reader admission independently pins manifest SHA-256
`e8f233474e03ca54e18f59fe02dc2020a3fd942623b228e643ae046ee953569c`
and archive SHA-256
`c0ebdf4d623deaad706d1aee02e57245bfc29994d935ad1a3780f67df08513f5`.
The archive contains 128391415 compressed bytes.

The manifest binds absolute workspace and target paths,
image digest,
compiler and sccache binary hashes,
`mise.lock`,
release/incremental/compiler-flag profile and all tracked scanner/regex source bytes plus original mtimes.
The reader verifies the real `fuse.archil` mount,
rejects partial archive bytes,
actual changed source and absent-manifest fixtures,
restores source mtimes and then executes the complete owner again.
Its target must initially be absent.
The seed requires unpublished archive/manifest paths and cannot overwrite this existing namespace.
The reader uses read-only remote compiler-cache mode with the workflow-provided cache token.

A new image or workspace path therefore requires its own newly admitted producer and namespace.
The existing r2 archive does not qualify under the r3 digest merely because Rust versions agree.
A repeated DAG must retain each repetition's original command receipts before another invocation overwrites them,
and must label fresh versus same-workspace target state explicitly.
The scanner's source admission covers its subject and regex path dependency;
a new driver should independently keep root tool/config inputs frozen as well.
No target-reader repetition spread was measured in the original campaign.

### Truepeak all-features consumer

Run `221vdm1f21` used harness `a85e204f078db540c2a292342000ce5423bb8338`
and `.depot/workflows/rust-truepeak-all.yml`.
Its exact task is `//package/monorepo-task/core-exec:research:rust:truepeak`,
implemented in `package/monorepo-task/core-exec/bin/rust-truepeak-all.mjs`
with `rust-bounded-profile.mjs`.
The adapter executes the release/all-features boundary of `//package/music-player/truepeak-core:test`:
locked fetch followed by locked/offline release/all-features Nextest with complete failure reporting.
Its dedicated JUnit configuration records every skip and output,
with a 90-second per-test bound and ten-second grace.

This workflow uses `tools-r3-fuzz-codex`,
image digest `sha256:1c2b469a9d7fae8bc364e6392f771c28565647a04c40847f345bc140dfce4e88`.
The prepared prefix supplies `mise-data`,
`mise-cache`,
`rustup` and `cargo`.
The adapter admits the frozen Truepeak crate and root `mise.toml`,
`mise.lock` and `.cargo/config.toml` before/after execution.
It uses private `HOME` and `TMPDIR` under `.research-rust-truepeak-all`,
a dedicated target directory,
existing fake samples and disposable Turso databases.
The adapter adds no SQLite/Chromaprint package setup and calls no real paid provider.

The actual result is 89 passed and two intentional skips out of 91 enumerated.
Locked fetch took 31.017 seconds;
compile plus Nextest took 75.075 seconds in the single admitted AMD64 sample.
The provider interval was 170 seconds on the 8-CPU/32-GB profile,
with a 15-minute job bound and twelve-minute execution step.
No standalone Truepeak target producer/archive was admitted.
Same-workspace repeated invocations retain the target and overwrite fixed receipt/JUnit paths,
so a new repeated driver must preserve each original result and declare that cache boundary.

The raw AMD64 XML is retained in `depot-rust-coverage-2026-10-18.receipts.json.gz`.
Walk its nested `previous` objects to the extension whose `run.run_id` equals `221vdm1f21`.
In that extension,
`selected[".research-rust-truepeak-all/receipts/nextest/default/truepeak-all.xml"]`
is the original XML string.
The current bundle has eight `previous` levels before that extension.
The ARM64 workload owner received this selector to compare all actual case identities,
not only aggregate counts.

The independent evidence review is published in
[`depot-independent-policy-process-mvm-review-2026-10-18.md`](../audit/depot-independent-policy-process-mvm-review-2026-10-18.md).
The initial queue attributed the three-ZIP guest-transformation work to GitHub run `39132577785`
and harness `3829831039cc43aa717c6c927f835b867e9313b6`.
Attribution correction on 2026-10-18:
those identify the cancelled corrupt-label generator control,
which has no assigned runner or actual step execution.
The completed independent review instead qualifies native `25jpltfpkq`
and GitHub `39134227406` at `1373e624739fbc93502b0c8ca502820c6ff99b45`.
All 198 members and 85 command records join their original archives and source;
all three pools reconstruct exact 51-byte guest input and 57-byte transformed output hashes.
No allocation was performed by the reviewer.

### Installed Nextest fail-fast source handoff

The desktop-control owner requested read-only evidence for an environment-only fail-fast override.
The installed `cargo-nextest 0.9.148` reports commit `cd1d6d5467322dbc0d39a80cb671d000ac302798`;
a fresh official tag lookup binds that release to the same commit.
Its clean existing source clone and actual `run --help` agree:
`cargo-nextest/src/dispatch/core/run.rs:169` to `191` declares the fail-fast flags without an environment binding.
The default config at `nextest-runner/default-config.toml:58` sets `fail-fast = true`.
`NEXTEST_FAIL_FAST=false` was not qualified as an override.
The explicit `NEXTEST_PROFILE` parser binding at `cargo-nextest/src/dispatch/common.rs:75`
requires an appropriately configured profile;
this source check does not qualify an unexecuted workaround.
No test build or allocation was performed for this handoff.

## Cache trust-scope follow-on queue

After sealing the local graph preflight,
the reviewer owns a separate bounded GitHub versus Depot GitHub cache scope experiment.
The queue is current action/source and scope-document review,
explicit admission under a separate `USD 0.10` conservative proxy guard,
branch-A seed and same-branch positive restore,
then branch-B restore using the same provider-specific exact key and cache version.
Jobs remain sequential with at most one active reviewer job;
the coordinator is notified at cloud start and end.
No application checkout,
account inspection,
broad credential injection,
cache purge,
subscription/settings mutation or paid provider call is authorized.
Owned branches and temporary files require cleanup;
cache retention/deletion limits must be explicit.
No cache cloud allocation has occurred at this queue boundary.

Completion update on 2026-10-18:
the separate [cache-scope receipt](../audit/depot-cache-trust-scope-2026-10-18.evidence.json)
now retains all six terminal jobs and their original ZIPs.
Both same-branch controls passed;
the sibling branch missed on standard public GitHub and restored exact marker bytes on Depot GitHub.
The workflow omitted `cache-mode`;
GitHub setup explicitly reported default `write`.
This does not test default-branch seeding,
explicit mode overrides,
forks,
cross-repository isolation,
or write poisoning.
The nominal Depot elapsed list-compute proxy is `USD 0.0025`,
the uniform rounded-completed proxy is `USD 0.036`,
and no reservation remains.
Owned remote/local probe branches and job temporary paths were removed;
benign cache entries remain subject to unqueried existing retention policy.

The next independent local-only queue is
[the separately sourced allocation follow-on](depot-owned-allocation-follow-on.md).
It will deduplicate post-baseline owned terminal controls against the frozen `223`-row baseline,
without editing the baseline or allocating cloud resources.
Root and coverage owners retain their active watcher ownership;
their final graph and OS rows join only after terminal receipts are available.

## Sealed package-result projection

The coverage lane's requested read-only join is now
[the package-result projection](../audit/depot-rust-package-result-projection-2026-10-18.evidence.json).
It preserves `23` frozen Cargo package roots,
their owning Mise task pointers,
actual recorded commands,
immutable provider run/job/attempt identities,
retained JUnit or original Nextest-summary count bases,
failures,
skips,
and source pointers.
The `61` execution contexts across `15` roots include repeated suites,
bounded fuzz targets,
benchmarks,
and functional consumers;
they are not a summed unit-test denominator.
An empty Rust-lane execution list does not classify a separately owned GUI or Android package as unexecuted.
The derivation records no new test execution or cloud allocation.

## Materialized source archive availability

The benchmark and same-image audit materialized the frozen export inside their ephemeral VM.
The qualified archive is `602,992,640` bytes with SHA-256
`47698c6be07012ab111e7586c1fd23d8b1f610ca5d3f570e53147da9a30ad368`.
Its `13,860` members include `1,464` content-addressed LFS materializations.
The raw receipts retain member hashes;
the full archive was not uploaded as a benchmark artifact.
A hidden and ignored-file-inclusive tar inventory of the retained Rust controller
found no full host archive to hand to another owner.
The receipt ZIPs therefore do not supply a reusable materialized source tar.
The [Git archive input-identity contract](../troubleshooting/git-archive-lfs-input-identity.md)
remains the reconstruction and admission boundary.
## Retained legacy owner identity search

The [legacy owner search receipt](../audit/depot-rust-legacy-owner-search-2026-10-18.evidence.json)
inventories `169` retained Depot JSON,
gzip,
and ZIP evidence files by exact bytes and SHA-256.
It searches `168` of those files;
the static source-assertion catalog is explicitly excluded from runtime evidence.
The bounded local profile used two CPUs,
2 GiB memory,
256 PIDs,
no network,
and a 90-second container lifetime.
It decoded `94` unique retained ZIPs and verified `3,558` member CRCs.
The exact parser passed `72` synthetic local controls covering every selected package/lib/bin alias.
Those controls receive no application execution credit.

The search found no matching Nextest status,
JUnit case,
structured test outcome,
or binary-enumeration candidate for `package/ide/app`,
`package/terminal/app`,
or the three `file-manager` application packages.
The source-qualified alias set includes `ide_app`,
`terminal_app`,
and every declared `monochromatic-file-manager` binary variant.
Actual retained forbidden-strings,
Truepeak,
linter,
and music-player outcomes supply positive corpus controls.
Repeated occurrence counts are not a test denominator.

The legacy test-feature trial selects Truepeak,
forbidden-regex,
its integration binary,
and JSONC.
The expanded native feature matrix selects Truepeak.
The prepared desktop producer emits `music-player` and `monochromatic-nested-wayland-session`;
its interaction checks do not establish these other applications' Rust unit execution.
Across seven specifically identified legacy revisions,
the file-manager and terminal package source/task trees match frozen `e02`,
while the IDE tree differs.
There is still no matching retained case/binary outcome for these owners in the searched corpus.
Resolved dependency equality is not inferred from Git tree equality.
No additional authored cases receive credit,
and no new allocation ran.

Four mutation-fixture members ending in `.json` contain the authored `mutation-private-input` marker,
rather than JSON.
The scanner retains those parse issues,
their original hashes,
and their raw-text search.
None contains a literal selected owner alias.
The receipt makes no global absence claim about other private filesystem material.

The [separate retained-input version join](../audit/depot-rust-legacy-input-version-joins-2026-10-18.evidence.json)
binds all `169` inventoried input hashes to exact retained Git commits and blob IDs.
It preserves reproducible source versions after concurrent evidence updates
without changing the immutable outcome-search receipt or repeating its workload scan.
## Locked Mise affected-selector version join

The [actual locked-version metadata probe](../audit/depot-mise-locked-affected-2026-10-18.evidence.json)
closes the version gap between installed Mise `2026.10.0` and CI's locked `2026.10.9`.
The actual `2026.10.9` binary rejects frozen `e02` workspace metadata with the same Node dependency cycle
before producing affected JSON,
including with equal explicit committed revisions.
A changed-producer acyclic fixture first selected its development consumer,
an unrelated project was excluded,
and equal fixture revisions returned a successful empty selection.
The offline `2` CPU and `2` GiB probe executed no owning task,
changed no dependency edge,
and preserves all `419` source metadata hashes.
The [existing troubleshooting entry](../troubleshooting/mise-affected-workspace-cycles.md)
contains the exact arguments and deciding locked-version source excerpts.
This is selector evidence only and adds no Rust test,
benchmark,
cache,
or cloud-allocation credit.
