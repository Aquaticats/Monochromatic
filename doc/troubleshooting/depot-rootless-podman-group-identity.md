# Depot snapshot runner with uidmap 4.13 rejects rootless Podman group identity

## Symptom

The existing `mise run prepare:playwright` task failed in the Depot browser experiment
before any browser test executed.
The [failed run][failed-run] used revision
`229182267dd5d2ff29eac6908e260b08040a617e`
and attempt `3fltxptw1z` on 2026-10-16.
Podman exited 125;
the task wrapper then exited 1.

The deciding diagnostic came from `newuidmap`:

```text
newuidmap: Target process 1678 is owned by a different user:
uid:1001 pw_uid:1001 st_uid:1001, gid:1001 pw_gid:117 st_gid:1001
```

The preceding Podman warning said `/` was not a shared mount.
That warning and the ownership rejection are separate diagnostics;
the successful rootful follow-up does not prove the warning harmless in every workload.

## Root cause and limits

The runner process had UID 1001 and GID 1001,
while its passwd entry named primary GID 117.
The separate [identity probe][control-run],
attempt `kfnj5n805m`,
confirmed those values:

```text
{"uid":1001,"gid":1001,"groups":[4,117,999,1001]}
runner:x:1001:117::/home/runner:/bin/bash
uidmap 1:4.13+dfsg1-4ubuntu3.2
```

`dpkg-query --show podman` produced a package name without a version.
The experiment therefore does not establish the Podman binary version.

The repository task in `mise.toml` invokes rootless `podman build`.
The observed invocation called `/usr/bin/newuidmap` to establish its user namespace.
The Shadow source clone at tag `4.13`,
commit `24b44b686ea2221405cfd806dede046cf1fd9584`,
contains the matching ownership check in `src/newuidmap.c:141`:

```c
/* src/newuidmap.c:141, upstream Shadow tag 4.13 */
if ((getuid() != pw->pw_uid) ||
    (!getdef_bool("GRANT_AUX_GROUP_SUBIDS") && (getgid() != pw->pw_gid)) ||
    (pw->pw_uid != st.st_uid) ||
    (getgid() != st.st_gid)) {
```

The branch at `src/newuidmap.c:145` emits the observed ownership diagnostic and returns failure.
The checked source is the upstream release corresponding to the installed package's base version;
the full Ubuntu patch series was not audited.
The observed values and matching diagnostic support a group-identity rejection.
They do not establish why the sandbox started with a different primary group,
or prove a Shadow or Podman defect.

The current Shadow source at
`e9bc595fd20558cf51aef096db262232090e8fb8`
retains the same decision at `src/newuidmap.c:132`.
No third-party source was modified.

## Verification

The failing catalog is:

- Rootless `mise run prepare:playwright` in the Node/dependency snapshot:
  the image preparation task failed with the identity diagnostic.
- Merely selecting the correct Playwright image version did not resolve that failure.
  The version mismatch is independently tracked by [issue #1045][image-issue].

The working catalog is:

- Rootful image preparation in [run `5f1hbcxtws`][rootful-run],
  attempt `0h9nf3dbv2`:
  succeeded and published `monochromatic-trial-browser:20261016-v1`.
- The consumer browser job used the prepared container and executed 69 cases.
  Its WebKit assertion failure is separate from container setup.
- The DOM consumer in [run `zlhbx5k5rq`][control-run],
  attempt `106fpb437j`:
  passed 34 existing browser cases.

The [execution audit](../audit/depot-expanded-execution-2026-10-16.md)
and its evidence file retain job durations,
resource samples,
test cases,
and the deliberately failing browser control.

## Verified workaround

The isolated trial changed both the image probe/build and container-run boundary
from `podman` to `sudo --non-interactive podman`.
The committed patch is `c7906236c3c2c744dd79f616952a84fadd24b81c`
on branch `trial/depot-expanded-20261016`.
The [matching patch](depot-rootless-podman-group-identity.patch)
preserves the exact task changes.

```text
# Isolated trial command boundary, not a workstation account change
podman build ...
  becomes sudo --non-interactive podman build ...
podman run ...
  becomes sudo --non-interactive podman run ...
```

Both preparation and consumption must use the same container storage owner.
The workaround uses root-owned container storage and can create root-owned files in the mounted checkout.
It was verified only in disposable Depot sandboxes bounded at 4 CPUs and 16 GiB RAM.
It is not a production rollout or a recommendation to change workstation account ownership.
The repository issue is [#1047][local-issue].

## What does not work

The unmodified rootless task fails in the measured sandbox identity.
Correcting the browser image version alone does not correct process identity.
Treating a later browser assertion failure as evidence that container preparation still failed
would conflate distinct boundaries.

Changing the runner account,
using an alternate login group,
and changing `GRANT_AUX_GROUP_SUBIDS` were not tested.
No claim is made about those alternatives.

## Upstream filing decision

No applicable exemption was found among the repository's `.out-of-scope/` entries.
Searches covered Shadow issues for `newuidmap` plus `different user`,
Shadow pull requests for `newuidmap` plus `group`,
and Podman issues for `pw_gid`.
The Podman results concerned different container networking or lifecycle boundaries;
they do not establish an exact duplicate or an upstream cause for this sandbox identity.

1.  Upstream fault:
    not established.
    The observed process and passwd primary groups differ,
    and the ownership guard rejects that mismatch.
2.  Fixability:
    the execution environment or consumer invocation can change;
    removing the ownership check is not justified by this evidence.
3.  Supported use:
    `man/newuidmap.1.xml:109` documents caller ownership validation.
    This investigation does not establish support for the mismatched sandbox identity.
4.  Contribution policy:
    Shadow's `doc/contributions/introduction.md` points to `doc/contributions/ai.txt`.
    That file prohibits contributions created with nontrivial AI assistance.
    No Shadow filing is prepared.
5.  Likely acceptance:
    no request to weaken this ownership guard is supported by the measured failure.
6.  Prototype:
    no upstream code prototype was made because upstream fault is unestablished
    and the contribution policy excludes this contribution.
    The consumer-side invocation was tested in the isolated trial.

There is nothing to file upstream from this evidence.
The actionable repository integration gap is preserved in [issue #1047][local-issue].

[failed-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/t3jdjq2nd3
[rootful-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/5f1hbcxtws
[control-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/zlhbx5k5rq
[image-issue]: https://github.com/Aquaticats/Monochromatic/issues/1045
[local-issue]: https://github.com/Aquaticats/Monochromatic/issues/1047

## Expanded graph recurrence on 2026-10-18

Native run `gb01kjx2qk`,
using harness `234ccc92dabb81e998e0a0bb5e838713924dd83f`,
reproduced the same identity diagnostic in both repetitions.
The retained original browser preparation log reports
`uid:1001 pw_uid:1001 st_uid:1001, gid:1001 pw_gid:117 st_gid:1001`
and Podman exit 125.
The graph does not record the installed uidmap package version.
Each repetition completed its shared build and Rust branches,
but image preparation and its dependent browser invocations failed before browser assertions.
Those failures give no browser-case credit and cannot support a complete-graph speed comparison.

Isolated correction `72af18d5b1` applies the verified rootful boundary through the existing Podman wrapper.
Only `GRAPH_PROVIDER=native-depot` selects `sudo --non-interactive`;
the original absolute Podman executable and arguments are retained.
Image probes,
builds and container runs therefore use the same storage owner.
The wrapper still adds 2 GiB memory,
2 CPUs and 256 PIDs to container runs.
Application inputs,
the command plan and the graph driver remain unchanged.
The corrected bootstrap SHA-256 is
`d6adb8579ea9888147d0b3f5eba90f730630a243b03db07d4753feb1b8f0f85a`.
The [independent local wrapper receipt](../audit/depot-expanded-graph-podman-preflight-2026-10-18.evidence.json)
executes that exact bootstrap in a disposable Node container bounded at 2 CPUs,
2 GiB,
256 PIDs,
and no network.
Its 24 provider/command controls preserve native-only noninteractive sudo,
the original absolute Podman target despite a competing `PATH` entry,
and the run resource flags.
Image probes,
builds,
runs,
inspection,
cleanup commands,
and exit-status propagation share the expected boundary.
Removing fake sudo produces the explicit `spawnSync sudo ENOENT` failure.
The fixture is removed.
These controls use fake sudo and Podman executables;
they establish generated dispatch and arguments,
not actual rootful Podman operation or application case execution.
Actual corrected native consumers remain required for passing browser evidence.

## Mutation work tmpfs ownership on 2026-10-17

This is a separate boundary from the `newuidmap` rejection.
In logger development request `4x77lb56cs`,
the rootful mutation adapter mapped the source owner's numeric UID and GID with `--user`.
The source task then invoked `rsync --archive` against a root-owned `/work` tmpfs.
The retained mutation logs report:

```text
rsync: [generator] chgrp "/work/." failed: Operation not permitted (1)
rsync error: some files/attrs were not transferred (see previous errors) (code 23)
```

The source copy failed before mutant assertions.
The affected checks include the mutation CLI integration,
`conf-fork` mutation and `make-asynchronous-fork` mutation tasks.
The later matcher mutation task reached the same error before its partition budget expired.

A mount-free local control used Podman `5.8.7`,
Alpine `3.22`,
2 CPUs,
2 GiB RAM,
64 PIDs,
no network,
a read-only root filesystem and `--cap-drop=ALL`.
With container user `1000:1000`,
`/work` reported owner `0:0` and `chgrp` failed with the measured permission error.
Adding `uid=1000,gid=1000` to `--tmpfs` did not work:
Podman exited 125 with `unknown mount option "uid=1000"`.
Harness commit `d8902dcf16` preserves that rejected development attempt;
it is superseded by `153d7bc98c`.

The positive control used rootful mappings
`--uidmap 0:1000:1 --gidmap 0:1000:1 --user 0:0`.
The process and tmpfs both reported container identity `0:0`,
and the ownership check passed with the same capability and resource restrictions.
The upstream Podman `v5.8.7` source file
`docs/source/markdown/options/uidmap.container.md`
documents direct host-to-container mappings for rootful callers.
This evidence supports the mapping mechanism;
it does not yet establish a passing Depot mutation consumer.

The corrected harness derives mapping IDs from the disposable runner process,
asserts tmpfs ownership in its existing private fixture,
and verifies that the host result file belongs to the source owner.
The local result did not predict the actual cloud result.
Request `1wzr86fssk` used Podman `5.8.4` and rejected that mapping before container execution:

```text
Error: cannot unmount /sys/kernel/debug: device or resource busy
```

All 37 retained job logs contain that diagnostic;
none reached the generic source executor.
This is a separate visible boundary from the original `rsync` ownership error.
The emitting diagnostic is Podman's output;
no narrower OCI-runtime attribution or upstream cause is established.

The replacement uses a uniquely named tmpfs volume with
`--opt type=tmpfs --opt device=tmpfs --opt nocopy`
and `--opt o=rw,exec,size=...,mode=1777,uid=...,gid=...`.
Podman's versioned `v5.8.4` volume-create documentation supports that mount-option form.
It keeps the source-owner nonroot UID,
original resource and capability restrictions,
and readonly source mounts without a user-namespace mapping.
The original `/work` tmpfs size is retained.
Only the owned named volume replaces that mount;
the adapter removes its unique volume and verifies absence after the child exits.

Bounded local positive and negative controls passed.
The initial local volume test also required the same SELinux relabeling on both controls;
that local label denial is distinct from UID ownership and from the cloud mapping rejection.
Actual generated-boundary controls use the source's existing `label=disable` setting.
Cloud pilot `85fv28fjkl` passed the private-manifest denial and source-owner positive control,
with verified cleanup and no added capabilities.

## Baked pnpm metadata under the nonroot mutation adapter

The same pilot then reached the unchanged mutation CLI integration and exposed another boundary:

```text
rsync: [sender] send_files failed to open "/baked/node_modules/.pnpm-workspace-state-v1.json": Permission denied (13)
rsync error: some files/attrs were not transferred (see previous errors) (code 23)
```

The retained log proves that the trial adapter's source-owner UID cannot read the generated image metadata.
It does not print the file's UID,
GID or mode.
Root-owned mode `0600` is a hypothesis from the image and package-manager setup,
not a measured cloud fact.
The next extraction control must capture those properties before attributing the permission cause.
The failure happens before mutant assertions and is an adapter prerequisite gap.
The consumer made 49 failed shard attempts,
with every owned volume verified removed.
The proposed repair extracts only that generated file from a stopped disposable image container,
retains original image identity,
bytes and mode evidence,
and binds an owned byte-identical copy readonly at its original path.
Request `360d91ht6w`,
using pushed trial harness `d098532a5b6946231daf3c4f43e38444359a307e`,
passed the unchanged mutation CLI integration and both build prerequisites.
The newly built actual source-image file measured UID `0`,
GID `0`,
mode `0600`,
and 33,427 bytes.
Its SHA-256 is `1b0f6d958e4a1269a2c420b3bf55adb9c1030ba8183bd94629a2c7c7cc14084d`.
This proves the new run's file properties,
not an unrecorded stat in the failed predecessor.

The adapter streams `podman cp CONTAINER:PATH -` from a stopped disposable container.
The original TAR retains the file metadata;
Podman `v5.8.4`'s `cmd/podman/containers/cp.go` sends that archive directly to stdout.
Normal host-directory copying instead assigns the host caller's UID/GID,
so that route would not prove original ownership.
The adapter extracts only the declared regular file,
retains the original archive,
and writes an owned mode-`0600` byte-identical copy.
Only that copy is bound readonly at the original path.
The original image identity and authored source bytes remain unchanged.

All six original source-image archive/copy pairs were independently byte- and hash-verified.
The source consumer completed its assertions with six shards and no runtime errors.
All six work volumes and six inspection containers were verified removed.
The separate cloud mapping and metadata incidents are retained in
[issue #1116][mapping-incident] and [issue #1117][metadata-incident],
closed with the actual source-consumer controls on the trial branch.
The final coherent pipeline replay remains separate work.

[mapping-incident]: https://github.com/Aquaticats/Monochromatic/issues/1116
[metadata-incident]: https://github.com/Aquaticats/Monochromatic/issues/1117

## Playwright CI environment on 2026-10-17

This is separate from image matching,
user mapping and OPFS capability assertions.
The unchanged root `run_playwright` helper at source
`ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`
does not pass `CI` in its `podman run` arguments.
The matched Playwright `1.64.0` image declares no `CI` environment value.
Both development requests `x0rb16tjzb` and `jps875l9qs`
set `CI=true` in the outer job,
but their raw container-side JSON reports show zero retries for every project.

Podman `v5.8.4` source commit `5431df23c742e5edea35bef34eed696f4db0106b`,
`docs/source/markdown/podman-run.1.md.in`'s environment section,
documents image values and explicit host-variable forwarding separately.
Its `docs/source/markdown/options/env.md` documents explicit variable forwarding.
The observed missing variable matches that contract;
no Podman defect is established.

A disposable control copied the exact frozen `playwright.browser.config.ts`
and `playwright.e2e.config.ts` without edits,
then ran the real locked Playwright `1.64.0` CLI in list-only mode.
With `CI` unset,
every project has zero retries and browser server reuse is enabled.
With `CI=true`,
every project has two retries and browser server reuse is disabled.
The unchanged source configuration therefore has a measured positive counterfactual.
The fixture itself gives no source-test credit.

The audit adapter preserves the authored environment behavior.
It does not silently enable retries or change the server policy in the frozen-source replay.
The repository runner needs an explicit CI propagation repair with actual container consumer controls.
The current full-suite OPFS assertion remains tracked independently by issue `#1052`.
All 118 configured development cases have retained outcomes:
116 passed,
one OPFS availability assertion failed,
and one authored WebKit OPFS case skipped.
No narrower cause for WebKit's OPFS refusal is established.
The complete per-case evidence and resource counters are in
[`depot-logger-complete-2026-10-17.evidence.json`](../audit/depot-logger-complete-2026-10-17.evidence.json).

## Generated workspace dependency resolution

Pilot `drzwk5q8mc` completed frozen installs for all three disposable generator consumers,
then the identity guard rejected a missing local `config-typescript` link.
No original generator ran,
and all three consumers were removed.
The guard had assumed every declared workspace dependency appears directly under its owner's `node_modules`.
That assumption contradicts this frozen source's configured `dedupeDirectDeps: true`.
The root package directly declares `@monochromatic-dev/config-typescript`.
The [current pnpm setting documentation][pnpm-direct-dedupe] states that root-linked dependencies
are not also linked into subproject `node_modules` directories when direct deduplication is enabled.
This is a trial guard defect,
not a failed source dependency installation.

The corrected guard searches ancestor `node_modules` directories only within the disposable consumer.
The first actual workspace target must still equal that consumer's frozen package directory.
All dependency directory and symlink checks still reject resolution into authoritative source.
Source HEAD,
index flags,
every tracked blob and executable mode remain verified before the original task.
No source guard is waived.
At `8b64f790134bf0cc4485a9842071760cbc355f08`,
seven disposable controls pass:
ordinary and deduplicated valid layouts,
consumer drift,
authoritative drift,
ordinary and deduplicated link leaks,
and hidden index flags.
Four guard-removal controls accept the corresponding invalid fixtures,
and the original direct-only resolver rejects the valid deduplicated layout.
All seven consumers are removed.
An initial control-driver mutation retained a newly introduced reporting variable;
its rejection was not counted as the expected missing-link control.
That failed fixture was retained,
inspected and removed after ignored-file and root Git-directory probes.
The fresh corrected control report is `depot-generated-controls-09PD1b/results.json` in the accounting scratch.
Actual frozen-source generator execution remains a separate cloud consumer control.

## Authentic linter for Git host checks

The authored Git host tasks require the gate's Git `2.56.0`.
Their real Markdown cases also require `GIT_POLICY_NATIVE_TEST_LINTER` to name an authentic linter.
Supplying the Git ELF alone leaves those cases at the explicit unset-variable diagnostic,
before the real-linter assertions.
The private worker-rootfs adapter now builds the linter from immutable source
`ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`,
using the original snapshot build arguments and recorded compiler environment.
Every retained source input matches its Git object;
the actual ELF,
tool versions and linked-library hashes are retained.
A real stdin Markdown rewrite and fixed-input idempotence gate precede both unchanged original host tasks.

Actual consumer `q10910cxfm` passes both original Markdown assertions.
Its binary suite reports 137 passed,
zero failed and one ignored;
the library suite reports 1,806 passed,
six failed and four ignored.
Those six separately mapped assertion boundaries remain source diagnostics under the retained rootfs environment.
No common filesystem cause or universal platform failure is established.
Source inputs,
exact source logs,
actual provider attempt attribution and cleanup are verified.
The two-CPU,
two-GiB and 512-PID bounds remain effective.
The linter sample records 9,663 memory-limit events with zero OOM events and zero PID rejections.
The final common cohort replay remains separate work.

## Disposable generator output cleanup

The rootful declared-type consumer `62nbb3srhk` completes its original corpus task:
439 draws are kept,
61 are deliberately dropped,
and the tracked generated corpus is unchanged.
The separate disposable-worktree cleanup fails with Git's `Permission denied` diagnostic.
The failed predecessor retains no ownership stat;
the error alone does not establish its UID,
GID or mode.

Bounded control `qsg8tx4bvb` uses the same actual Node image config
`7a0d2c0c30811c393f278a255613b2a78cac778438bc06464e4a8b6979c3d4a9`
and the source task's readonly repository plus writable output mount geometry.
It measures a root `0:0` directory with mode `0755` and file with mode `0644`.
Ordinary Git removal reproduces `Permission denied` and still removes the worktree registration.
A proposed privileged Git retry therefore fails with `is not a working tree`.
That remedy is rejected.

The verified cleanup validates the frozen invocation's exact disposable path,
HEADs,
shared Git repository and original worktree registration before removal.
It records ignored-file dry runs,
root Git-directory/ignore probes and output metadata.
After ordinary removal fails,
it permits privileged deletion only if registration is gone,
foreign ownership was measured,
the exact nonsymlink orphan path is unchanged,
and authoritative HEAD/status still match.
It never changes ownership or source bytes.
Ten rejected fixtures,
an ordinary positive fixture and a guard-removal acceptance verify the scope.
Actual control `p3jlkgvl99` passes the rootful fallback,
proves the fixture and containers absent,
and retains identical strict source before/after identities.
Both controls give zero source-check credit and do not repeat the completed corpus consumer.
The final common cohort replay remains separate work.

## Native report freshness and artifact retention

Development mutation tasks can finish while their native JSON remains outside the artifact directory.
Request `lt7qwlz6h1` retains actual task terminals and CLI totals,
but its default root TOML report and package-dist conf report were not copied.
Those missing historical payloads cannot be reconstructed from totals.

The first collector control also rejected a report created after its before-snapshot:
the selected start clock was `2026-10-17T09:45:57.552Z`,
while the immediately written file reported mtime `1792230357550.949` milliseconds.
That observation disproves strict wall-clock ordering as a sufficient collector predicate here;
it does not diagnose a Node or filesystem fault.
The rejected guard in trial `57cd4d8d4b`,
`package/logger/core.fuzz/mise.toml:211`,
required the following comparison:

```javascript
// package/logger/core.fuzz/mise.toml, trial 57cd4d8d4b
stat.mtimeMs >= Date.parse(startedAt)
```

The corrected trial task snapshots presence,
timestamps,
inode and SHA-256 before its exclusive worker invocation.
It copies every actual regular report immediately after termination,
binds exact bytes/hash and records freshness from changed identity or bytes.
The clock comparison is retained separately.
Passing source tasks require a fresh report;
stale candidates stay explicitly uncredited.
Package-dist JSON and nested native `target/verification` archives are preserved separately.
The tradeoff is extra retained before/after metadata and explicit payload validation during reconciliation.

Four valid,
seven rejected and one freshness-guard-removal controls pass.
Actual `ef014` consumer `knsghczl71` verifies the same collector blocks with its authored TOML CLI,
a fresh 650-byte native report and actual readonly metadata/volume cleanup.
The independently read mutant `9cd1056a095eaea1` is `compileError`;
it receives zero assertion or whole mutation-check credit.
The report SHA-256 is `692e5dd934f799518de76762ee34a0f22558a66a2832188cf5b074dfc37d7825`.
The deciding mutation-runner modules also match the source in classifier issue `#1142`,
so a native `killed` classification is not inferred to mean a literal assertion executed.

Issue `#1160` owns native-report omission and final common per-ID reconciliation.
No upstream Node/filesystem defect is established,
so no third-party filing is proposed.
The verified repair belongs to the collector's own predicate and artifact boundary.

[pnpm-direct-dedupe]: https://pnpm.io/settings/other#dedupedirectdeps
