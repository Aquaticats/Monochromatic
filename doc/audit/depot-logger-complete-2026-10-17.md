# Logger affected CI execution on 2026-10-17

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/vmsync` is now `package/vmsync/cli`.

## Purpose and source identity

Execute the complete reviewed logger change surface at
`ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`,
including its affected consumers and external host boundaries.
The development `.depot/manifests/logger.json` has SHA-256
`f4cbd47f210f5b031d76c5fc1c28fa704b349621f91eefa3f4f3ad575acfedc3`.
It contains 719 package checks and 19 external check specifications.

The trial harness lives on branch `trial/depot-complete-ci-20261016`.
Its source checkout and dependency lockfiles remain historical inputs.
Harness changes add execution, resource bounds and evidence retention;
they do not repair historical source failures.

## Graph verification

A separate traversal verified unique package check identities,
existing prerequisite identities, absence of cycles,
and every declared partition's exact prerequisite closure.
The static wave activates 440 checks and the unit wave activates 230.
Their union contains 564 checks.
The remainder partitions select the other 155 checks without overlap.
Their complete union is exactly the frozen 719 checks.
Every package Oxlint specification retains its unchanged package task and empty argument list.

The retained graph verification is summarized in the companion evidence JSON.
The earlier import audit and source freeze remain in the branch's manifest history.

## Candidate manifest revisions

On 2026-10-17, harness commit `0095ae37c7` corrects the Vmsync external specification
to the authored `//package/cli/vmsync:test:e2e` task with empty arguments,
package scope and the declared `package.cli.vmsync.build` prerequisite.
The source task is retained in `package/cli/vmsync/mise.toml`.
That candidate manifest SHA-256 became
`4f9bcf0a94e09ecda8eae84c8e5ff9d89f15f0bd398df8136f0771011d3931f7`.
The development requests retain their original digest and alternate unit-file route provenance.
Runtime equivalence of that alternate route has not been verified.
A comparison of the parsed manifests proves all 719 package checks
and the other 18 external specifications are unchanged.
The serialized package-check array has SHA-256
`c7c7aca4f7a2d3865bcaf9abc32ae8a7f546d7170be20c90f782402ffa6200b8`.

Harness commit `09941541e8` adds the missing `package.module.dom.build` prerequisite to `root.e2e`
and `package.webapp-productivity.wc.build` to `root.browser`.
Their immutable consumers read the corresponding built artifacts.
The current candidate digest is
`d8cf02e46bafd03ab4f410ccf2a259a29c68c757af1cddf81735258372025af9`.
All 719 package specifications remain unchanged.
Relative to the original development manifest, only those external rows and the Vmsync row differ.

## Current execution requests

- [Package, Pi and WireGuard request `4x77lb56cs`](https://depot.dev/orgs/l4zncjjbj5/workflows/4x77lb56cs)
  uses pushed harness revision `b0c730f8bfb9081345cda89da1f79ff3239e7fb6`.
  It includes static and unit waves,
  the seven remainder partitions,
  configured offline Pi host checks,
  and private WireGuard integration checks.
- [External request `j4m7gw8mwg`](https://depot.dev/orgs/l4zncjjbj5/workflows/j4m7gw8mwg)
  uses pushed harness revision `1c189c7068e50401b0288f8ddbd3d0946999a913`.
  It includes browser, DOM E2E, Electron Wayland,
  generator consistency, full root Oxlint,
  and music desktop artifact/runtime groups.

Both requests were launched from clean worktrees after verifying the remote branch revision.
They therefore avoid Depot's automatic local patch injection.
These are separate request intervals;
their durations must not be described as one measured complete pipeline latency.

## External coverage ownership

The external request selects eight specification identities:
`root.browser`,
`root.e2e`,
both Electron Wayland checks,
`root.generated-consistency`,
`sidecar.generated-type-consistency`,
`root.config-oxlint`,
and `artifact.music-desktop-e2e`.

The browser specification retains its eight declared files and three configured engines.
The DOM E2E specification retains four declared files and two configured engines.
Each generator check receives its own disposable historical worktree.
The desktop group builds the historical desktop artifact and compositor
before invoking the existing private runtime adapter.

The package request's Pi group owns notification delivery,
terminal verification,
configured extensions,
and the expensive offline tmux test.
Its WireGuard group owns route and bypass integration.
The root agent owns the fs-id and semantic-bridge host workflow,
changed-file forbidden-string scanner,
and current-policy Scorecard gate.
The separate VM experiment owns the historical vmsync lifecycle boundary.

The current [host request `39003180992`](https://github.com/Aquaticats/Monochromatic/actions/runs/39003180992)
passed all six jobs across Linux, macOS and Windows.
Independent log verification matched the immutable source revision and both dependency lockfile hashes
before and after each job.
Filesystem checks verified preferred stable identities without colons.
Each semantic job passed its three lifecycle, offset and external-consumer groups.
The authored external-consumer test does not retain its fresh dependency lockfile,
so the versions selected by its allowed dependency ranges remain unverified.
The provider job intervals total 417 seconds and 1530 vCPU-seconds.
Standard public GitHub runner compute is free under the cited allocation policy;
this does not verify storage charges or the account ledger.

## Remaining evidence

Both Depot development requests are terminal with failed status.
The external collection retains every attempt and hashes every downloaded artifact.
The package collection retains its 11 attempts and 11 hashed artifacts.
Independent reconciliation verifies every executed package log hash and byte count.
It finds terminal records for 685 package identities.
The other 34 identities contain three started checks without terminal records and 31 unstarted checks.
That snapshot covers request `4x77lb56cs` only and remains preserved.
The global development union also verifies 46 executor observations from external requests
`j4m7gw8mwg` and `dhxhzzvmc3`, including every task argument, output hash and output byte count.
The browser-properties bundle adds one terminal identity:
the global union contains 686 terminal identities and 33 unresolved identities.
This union is development evidence, not a coherent complete replay or proof of source eligibility.
The missing records belong to the fuzz, mutation and container partitions;
workflow completion does not establish their completion.

Selective prerequisite recovery request `dhxhzzvmc3` uses pushed harness revision
`2676cb365a206434b525bf032a5875cafde827e9`.
It selects 12 package checks plus their declared prerequisite closure,
browser and DOM execution,
and only the sidecar generator check.
The root generator's observed tracked output difference remains a source diagnostic.
The recovery is terminal with failed status and all four artifacts collected and hashed.
Its 12 requested package checks and their 24-check prerequisite closure passed.
Both sidecar generator commands passed without tracked or untracked output changes.
Independent reconciliation verifies every executed check log hash and byte count.

The browser task reached the engines but exceeded the adapter's exact 1,200,000 ms budget.
Its source configuration selected 84 cases with eight workers;
the interrupted group does not prove that every file or engine completed.
The DOM group selected 34 cases and failed during browser or fixture setup,
including `pthread_create` resource errors and a missing built DOM artifact.
These outcomes are distinct from source assertion failures.
The current candidate adds the missing build dependencies and uses one browser worker.
The prepared browser routes partition the frozen files and configured engines into 32 disjoint cells.
Their auxiliary manifest retains the immutable configuration hashes and unchanged primary manifest digest.
Two `2x8` pilot cells precede the remaining 30 cells; the full 32-cell route remains held.
Native Playwright `1.64.0` controls verify that adding JSON preserves the configured dot and HTML reporters.
Actual generated-program controls verify full inventories,
exact partition inventories,
normal source failure exit preservation,
and rejection of an incomplete full inventory before source execution.
Cloud pilot `x0rb16tjzb` started at `2026-10-17T06:47:04Z` from clean pushed harness
`c0eefbea382ef41c4667e90242010757eb5f759f`.
The provider confirms that exact source and head revision.
Independent terminal collection verifies both provider jobs succeeded,
all five selected Playwright cases passed,
and the three logger storage backend properties reported passed.
The independently reconstructed full inventories contain 84 browser cases and 34 DOM cases.
The exact partitions preserve dot, HTML and JSON reporters,
one worker,
two logical CPUs,
the two-CPU/two-GiB container limits and the 512-PID limit.
Both resource samples show no PID rejection or OOM events.
The two `2x8` jobs consumed 443.551 provider-job seconds and 887.102 allocated CPU-seconds,
corresponding to $0.0443551 published compute before plan credits.
The account ledger and physical host RAM were not directly verified.
Remaining-partition request `jps875l9qs` used clean pushed `51d5a2fd69507d9346d7fe6eb6345a4ee9929383`.
All 30 jobs are terminal, with 29 provider successes and one failure.
Independent reconciliation of both requests verifies the disjoint 32-partition union against every full inventory:
all 118 configured cases have outcomes, with 116 passed, one failed and one authored skip.
The failure is WebKit's logger OPFS `verify detects OPFS availability` assertion,
which expected `true` but received `false`.
The WebKit fs-path case explicitly skips after OPFS refuses `getDirectory` with `UnknownError`;
the logger property also explicitly skips its OPFS backend while its other backends pass.
This evidence does not establish why WebKit refused OPFS.
All sampled cgroup memory and PID rejection counters remain zero.
Every prerequisite and external log hash and actual provider attempt URL is verified.
The 32 actual `2x8` jobs consumed 4,681.97 provider-job seconds and 9,363.94 allocated CPU-seconds,
corresponding to $0.468197 published compute before credits.
The authored source Podman command does not forward `CI`;
the retained Playwright configuration therefore reports zero retries despite the outer workflow's `CI=true`.
No adapter silently changes that source behavior.

Provider request IDs and workflow IDs name different objects.
Request `dhxhzzvmc3` contains workflow `fjdhk1bj7r`.
Every recovered artifact's `providerJobUrl` exactly matches its current attempt's `view_url`.
The separate provider identity verification retains that equality control;
the different workflow ID is not a stale environment defect.

An invalid dispatch attempt, `tf9ddwgbw3`, used revision
`ac6b8f5cde945b4a3ce9ea9974744369547bbdba` instead of the intended trial revision.
Its four failed jobs receive no historical source-check credit.
The non-login command environment selected the repository Git shim;
Depot could not detect the repository or ref.
The corrected login-shell dispatch resolved `/usr/bin/git` and printed the intended pushed HEAD.

Continuation request `1wzr86fssk` selects the 34 unresolved checks
and three checks with retained `rsync` ownership infrastructure failures.
Each continuation receives an unchanged leaf task and its prerequisite closure in a separate job.
It started at `2026-10-17T05:41:08Z` from clean pushed revision
`09941541e8ba43d59b03969248f8484a2d2c4269`.
The provider independently reports that revision in its `ref`, `sha` and `head_sha` fields.
All 37 attempts failed before source execution with Podman's diagnostic
`cannot unmount /sys/kernel/debug: device or resource busy`.
All 37 logs independently confirm the same failure and no generic case or remainder execution.
All 37 artifacts are downloaded and hashed; the request adds no source-check credit.
The locally passing user-namespace mapping is rejected at the actual cloud boundary.

Owned-tmpfs-volume pilot `85fv28fjkl` used clean pushed revision
`bc6889aaaa8da7974824380bd18f0447ab6945d5` and is terminal with failed status.
Its cloud preflight passed the private-manifest denial and source-owner positive control,
with verified volume cleanup and no added capabilities or user-namespace mapping.
The unchanged mutation CLI integration then reached 49 shards.
Its `rsync` sender could not read `/baked/node_modules/.pnpm-workspace-state-v1.json`
under the adapter's nonroot UID.
The log proves `Permission denied (13)` but does not record the file's UID, GID or mode;
the next control must measure them before attributing the permission cause.
All 49 shard volumes were removed and verified absent.
This newly exposed adapter prerequisite is separate from mutant assertion failures.
Subsequent request `360d91ht6w` used clean pushed harness
`d098532a5b6946231daf3c4f43e38444359a307e` and passed the unchanged CLI integration and both builds.
Independent collection verifies all three executed logs and unchanged source inputs.
The actual source-image metadata measured UID `0`, GID `0`, mode `0600`, and 33,427 bytes.
Its SHA-256 is `1b0f6d958e4a1269a2c420b3bf55adb9c1030ba8183bd94629a2c7c7cc14084d`.
Six original TAR archives and owned readonly copies preserve identical bytes and unchanged image identity.
All six shard volumes and six metadata-inspection containers were verified removed.
The authored integration assertions passed with 11 killed mutants,
six survivors,
two compile errors and no runtime errors.
This new measurement does not retroactively establish the unrecorded mode in `85fv28fjkl`.
Distinct incidents [#1116][mapping-incident] and [#1117][metadata-incident]
were duplicate-searched,
filed,
and closed with the actual source-consumer proof on the trial branch.
The troubleshooting record preserves every rejected boundary and the measured repair.
Provider execution intervals remain usage proxies until paired with the declared allocation.
No passing complete graph or whole passing latency is established by this interim record.

The executor now separates manifest inventory, requested leaves, prerequisite closure,
live running checks and completed records.
It writes `case-progress` while execution continues,
`case-finished` only after the selected checks and source-input verification finish,
and `case-aborted` when execution is incomplete.
Its `completeGraph` field reflects the manifest.
Five disposable controls passed, including a live consumer of its own running record
and rejection of the previous executor by that same consumer.

The Git host task's immutable description explicitly requires the gate's Git `2.56.0`
for tests that start Git.
The observed host `2.55.0` mismatch is therefore a missing test prerequisite.
Recovery `10zmqch10h` at clean pushed `13c022e6b33007da4c23cdb88f5fdf9eccf3f23c`
builds the exact audited Git `2.56.0` release against the worker ABI with the original build flags.
It verifies 4,966 unchanged upstream inputs,
the installed real ELF,
all three linked-library hashes,
an actual Git commit and `/proc/PID/exe` equal to `/usr/bin/git`.
The installation and original host tasks run only in a private full worker-rootfs overlay
with two CPUs, two GiB, 512 PIDs and no network.
Its ELF SHA-256 is `8c7b36571efa3e7daa5230ec86ab3c2f681d76e09f00eda3261424621cbe45c8`.
The unchanged binary-host suite reports 136 passed, one failed and one ignored;
the unchanged library-host suite reports 1,805 passed, seven failed and four ignored.
Deliberately failing child lock-helper controls are not additional parent-suite failures.
Both build prerequisites pass, every retained check-log hash and provider URL matches,
source inputs remain verified and runtime cleanup is verified.
All sampled OOM and PID rejection counters remain zero.
Two Markdown failures stop at unset `GIT_POLICY_NATIVE_TEST_LINTER`;
they have not exercised the real-linter assertions and are an unmet harness prerequisite.
The earlier description of all eight as source assertion failures is retracted.
The Git version prerequisite remains independently verified.
An authentic linter built from this immutable source, with executable provenance and a real gate,
must precede replay of the two unchanged host checks.
The native accounting owner is comparing the other six actual assertion boundaries with existing incidents.
The actual `2x8` allocation consumed 572.204 provider-job seconds and 1,144.408 allocated CPU-seconds,
corresponding to $0.0572204 published compute before credits.

The Git pilot artifact contains 2,351,425,116 extracted bytes;
2,329,816,576 are the already-audited `native-base-images.tar` cache.
The final upload plan retains image/tool transfer receipts,
source logs, consumer reports, metadata controls and the Git ELF,
while keeping the original audited archive as separate retained evidence.

The current development union independently verifies 306 additional generic log observations
across the ownership pilot, leaf recovery, browser prerequisites and Git host recovery.
It contains terminal observations for 717 of the 719 package identities.
Conf fuzz and TOML mutation have never produced a terminal task record.
Conf mutation has an older harness-failed terminal observation,
but its repaired-prerequisite campaign also remains incomplete.
The latest incomplete set therefore still has three tasks.
The original `685/34` and subsequent `686/33` snapshots remain preserved with their request scopes.
None of these unions is a single coherent replay or proves source eligibility by terminal count alone.

Continuation `1pgpb8qhn6` started at `2026-10-17T06:53:35Z` from clean pushed harness
`51d5a2fd69507d9346d7fe6eb6345a4ee9929383`, independently confirmed by the provider.
Its 35 disjoint selectors contain the 33 globally unresolved identities
and the two remaining ownership-prerequisite repairs.
Their frozen prerequisite closures are verified.
The passed browser-properties bundle and CLI integration are excluded from this development recovery.
This request retains `16x64` provider allocation and original container bounds;
compute must count the allocated 16 CPUs.
All 35 jobs and artifacts are collected, with 26 provider successes and nine failures.
Independent source/task/argument, closure, log-hash and attempt-URL reconciliation verifies 75 terminal logs,
26 selected tasks passed, six selected tasks failed and three selected tasks incomplete.
All retained fuzz runtime records verify evidence copying and container cleanup.
Their actual `16x64` allocation consumed 11,113.703 provider-job seconds
and 177,819.248 allocated CPU-seconds,
corresponding to $8.8909624 published compute before credits.
Six failed jobs contain actual source errors:
missing conf coverage baseline,
JSON-C and logger coverage gates,
an authored async differential timeout,
and matcher and p-map differential counterexamples.
Three tasks have no source conclusion after the 1,440,000 ms adapter budget:
conf fuzz,
TOML mutation and conf mutation.
Their continuation records explicitly say `started-without-terminal-record`.
Both mutation logs continue completing source assertions and shards at cutoff;
the budget does not establish a source failure.
Only those three unfinished tasks are selected in bounded two-hour recovery `lt7qwlz6h1`,
requested at `2026-10-17T07:35:35Z`.
Both provider revisions equal clean explicitly pushed `f69a78a3b31f81cbfe47496bcdc6e2ac75549777`.
The first immediate dispatch returned `not_found`;
an unchanged-ref reprobe succeeded without a source or workflow edit.
No server-side cause is inferred from that observation.
The mutation jobs use `4x16`, matching the authored two concurrent two-CPU containers;
the single fuzz container uses `2x8`.
All three long tasks now have independently verified source terminal records,
with nine exact source/prerequisite log hashes and actual attempt URLs.
Conf fuzz and TOML mutation pass.
TOML reports 922 killed, 255 survived, three timeouts, 1,882 compile errors and zero runtime errors across 462 shards.
Conf mutation exits one after one red baseline in `src/conf.cache.unit.test.ts`;
its summary reports 406 killed, 90 survived, four timeouts, 393 compile errors and one runtime error across 166 shards.
That baseline's narrower cause is unproved.
The development artifacts omitted the native mutation-report JSON files,
so these CLI totals are retained summaries, not an independent per-mutant reconciliation.
The successful fuzz container's evidence copy and cleanup are verified.
Actual allocated usage is 33,447.81 CPU-seconds, or $1.6723905 published compute before credits.
The new development union has terminal observations for all 719 package identities,
with no unfinished selected task.
It retains the superseded 685/34, 686/33 and 717/2 snapshots.
It is not a coherent replay and does not infer eligibility or valid source prerequisites from terminal status alone.
The remaining queue includes fresh native-report retention around each final mutation leaf,
a bounded authentic source CLI consumer control,
filing the remaining distinct source/harness diagnostics,
and root integration of the prepared combined route.

The native-report collector's local controls pass four valid fixtures,
seven rejected fixtures and a stale acceptance after removing its freshness guard.
An immediate-write positive control first exposed filesystem mtime lag behind the wall clock;
freshness now compares before/after presence, identity, timestamps and bytes.
The clock comparison remains a separately reported observation.
The emitted pilot collector parses and inherits its environment without serializing it.
Authentic bounded control `r1mn88ccqt` is running at clean-pushed `99edd42b6e53f4b4ef88b5c39c9a6ae677dfedff`,
with a `2x8` provider, one selected TOML source file, one container and shard size one.
It gives no whole frozen mutation-check credit.
The required package type-check invocation stops before checking types:
Node reports `ERR_MODULE_NOT_FOUND` for `nano-spawn` in this worktree's `tsc-filter.ts` dependency.
All 18 used/control task bodies and emitted container/collector programs parse.
Depot's `ci dispatch` rejects the new no-input workflow as not found or missing `workflow_dispatch`;
its supported `ci run` starts this development control from the verified clean pushed HEAD.
No diagnosis of that registration diagnostic or final common dispatch is claimed.

Control `r1mn88ccqt` is terminal before the mutation CLI is invoked.
Its diagnostic workflow omitted the frozen workspace install,
so the first caught-value build cannot resolve `@monochromatic-dev/config-rolldown`.
The source guard and exact failed prerequisite log remain retained, with zero mutation credit.
Recovery `knsghczl71` runs at clean-pushed `392a3ea04128547d9b452dd3bea7a4df1235acd1`
after adding the same frozen installation already present in the combined workflow.
Its terminal receipt and actual native-report byte/freshness proof remain queued.

The failed async, matcher and p-map fuzz tasks each enumerate two property files.
Their unchanged `execFileSync` loop stops after `src/differential.property.unit.test.ts` fails.
The later async `src/fork.property.unit.test.ts`
and matcher/p-map `src/limits.property.unit.test.ts` are unexecuted/source-blocked in those invocations.
The terminal-task union gives no completed internal campaign or branch credit to these later files.
The final coherent replay is held until the root agent aligns those routes.
The common cohort manifest will be a separate authenticated, byte-verified file on `main`.
Its outer SHA-256 differs from the single-case logger manifest digest.
The combined workflow must bind the same cohort ID,
outer digest and request timestamp before and after execution,
and validate the common manifest's logger source, base, case digest and route harness revision.
The receipt task at `040a4814a63e27e392a68c83bceaaff78441c962` passes actual-program paired controls
with a 1,047,160-byte base64 fixture containing escaped text.
Ten invalid-binding, transport, size and request-clock controls reject their inputs.
Removing the digest guard makes its invalid-digest control pass,
which proves that control detects the guard's absence.
The authenticated live transport proof remains separate from the injected transport fixture.
Common workflow wiring is prepared and independently checked; root integration and the final freeze remain pending.
The candidate complete remainder inventory has 57 disjoint selectors for 155 checks;
its union with the prior static/unit closure covers all 719 identities without changing the package graph hash.
Direct inspection corrected the split to regular 16, Rust four and Git host two:
the Git host tasks belong to the frozen Rust partition.
Both primary and candidate manifests retain `completeGraph: false`.
The `3ac9616096` subject's word `freeze` overstates this candidate inventory's readiness;
[its corrective comment][candidate-correction] records that no final freeze or dispatch occurred.
The current OPFS reproduction is added to existing issue `#1052`;
the separately verified CI propagation boundary is tracked by issue `#1131`.
The generated-check adapter already isolated checks in separate detached worktrees.
The hypothesis that its generated drift necessarily dirties the authoritative checkout is retracted.
Its `changedTrackedBytes` field measures diff bytes, rather than a file count.
The stronger consumer controls and real-source pilot are recorded separately.
The prepared combined workflow remains incomplete until root integration and the remaining controls are verified.
Historical source failures stay terminal diagnostics for this measurement.

## Authentic host linter prerequisite

Recovery `q10910cxfm` uses clean pushed harness `c789a14548fec9df1cee01facb9577d83ddfd42c`,
independently matched to both provider revisions.
The original immutable-source linter build uses `cargo build --offline --locked --bin monochromatic-lint`.
Every retained source input matches its `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e` Git object.
The retained ELF has 110,474,544 bytes and SHA-256
`63fee7f02a04ad493115b8f1494042d602f8541cf7dedd0aa88d3c88368f30de`.
Its real stdin Markdown rewrite produces the expected LFS URL and leaves already-fixed input unchanged.
Both original Markdown host assertions now pass.
The unchanged binary suite reports 137 passed, zero failed and one ignored;
the unchanged library suite reports 1,806 passed, six failed and four ignored.
Those six names exactly match
[the independently searched incident mapping](depot-git-host-failure-mapping-2026-10-17.md).
Their causes remain unproved; the related lock ownership cases remain separate incidents.
The emitted boundaries are filed separately as
[#1135](https://github.com/Aquaticats/Monochromatic/issues/1135),
[#1136](https://github.com/Aquaticats/Monochromatic/issues/1136),
[#1137](https://github.com/Aquaticats/Monochromatic/issues/1137),
[#1138](https://github.com/Aquaticats/Monochromatic/issues/1138),
[#1139](https://github.com/Aquaticats/Monochromatic/issues/1139)
and [#1140](https://github.com/Aquaticats/Monochromatic/issues/1140).
Their remote titles, bodies and triage labels match the rendered reports.
Deliberate child-process failures are not additional parent-suite failures.
All generic log hashes, actual attempt URLs, immutable source inputs and cleanup are verified.
The two-CPU, two-GiB and 512-PID controls remain effective.
The linter sample records 9,663 memory-limit events, zero OOM events and zero PID rejections;
the memory-limit counter is not described as zero.
The actual `2x8` job consumed 687.007 provider-job seconds and 1,374.014 allocated CPU-seconds,
or $0.0687007 published compute before credits.
The retained artifact contains 132,137,361 extracted bytes,
including both authentic ELFs and the full source logs,
without another copy of the audited base-image cache.

## Prepared combined route

The candidate at `18e8c13173f244316ef528467ae0ccbf1fe38c80` expands to 97 jobs:
static and unit closures,
57 remainder selectors,
Pi and WireGuard,
four external adapter groups and 32 browser cells.
Independent controls verify all 719 package identities,
the 155 disjoint remainder identities,
155 full-package Oxlint task arguments,
the 19 external routes,
required common inputs and paired always-run after guards.
The root source-identity helper matches the copied canonical helper bytes.
The browser standalone workflow contains the same cells and must not be dispatched alongside the combined workflow.
The external routing manifest explicitly leaves root-owned GHA and VM integration pending.
Primary and candidate manifests still declare `completeGraph: false`.
There is no final cohort dispatch.

## Generated consumer identity controls

Each authored generator now receives its own disposable checkout and frozen dependency installation.
Before invocation, the adapter verifies its HEAD, index, every tracked blob's bytes and modes,
and workspace dependency targets against the frozen source.
Workspace links and dependency roots may not resolve into the authoritative checkout.
The adapter retains the original command, binary diff, changed bytes and hashes,
actual process termination and cleanup inventory for each invocation.
No independent generator runs on a previous generator's mutated tree,
and generated outputs are not restored to manufacture cleanliness.
The authoritative before/after guard remains strict.
Miniature controls pass for clean consumers and reject consumer drift,
authoritative drift, workspace-link leakage and hidden index flags.
Three guard-removal controls prove the corresponding negative controls detect absent guards.
These fixtures do not establish actual frozen-source generator execution.

Pilot `vt8vzj154t` stops before any original source check:
the harness declares `DEPOT_LOGGER_EXTERNAL_GROUP`,
while the actual adapter reads `DEPOT_LOGGER_EXTERNAL` and emits
`AssertionError [ERR_ASSERTION]: Unknown logger external group`.
This is a pilot wiring defect, not generated source drift.
Commit `673ab92e49885dd61fcf08e168ed9c59bf29c6be` corrects the binding.
Actual registered-program prefix controls accept all six groups,
reject the wrong variable and accept it only after the rejecting guard is removed.
The combined workflow already uses the correct variable.
Corrected pilot `drzwk5q8mc` passes its frozen dependency installs,
then the guard rejects a local workspace link that pnpm deliberately deduplicates into root `node_modules`.
No original generator executes; all three consumers are removed.
The frozen `dedupeDirectDeps: true` setting and pnpm's documented behavior explain this guard-model defect.
Commit `8b64f790134bf0cc4485a9842071760cbc355f08` resolves dependencies through consumer ancestors,
still requiring every workspace target to equal the consumer's own frozen package.
Seven fresh controls verify two valid layouts, five rejected layouts and four guard-removal acceptances;
the old direct-only resolver rejects the valid deduplicated layout.

Actual pilot `ctg9148f4t` verifies all three initial consumers against source tree
`f7835f65413553704f9988f1fcf7f69911a8f2c0` and all 13,715 tracked inputs.
Each consumer records 809 workspace links, including 173 resolved through ancestors.
Every authoritative after snapshot equals its before snapshot,
and all three consumers are verified removed.
Independent reconciliation verifies 14 source/prerequisite log hashes and the actual attempt URL.
The original `//:sync:files` exits zero,
but produces a 2,037-byte patch to `mise.toml` by removing ambient `.bin` PATH entries.
This is an authored-generated-output-drift result.
Applying the retained patch to frozen input in a disposable reconstruction reproduces the changed bytes:
67,669 bytes with SHA-256 `3a5e40b277c792bf7283de38c7b2af3250407506fe5516b36febe23693804b88`.
The exact generated bytes are retained, and the reconstruction fixture is removed.
The original `generate:type-cases` exits zero with no generated drift.
The original `generate:declared-type-cases` is invoked,
then Podman emits `cannot set up namespace using "/usr/bin/newuidmap"` and exits 125.
Its corpus consumer is blocked by the absent rootful adapter, not credited as a source assertion failure.
The actual `4x16` job consumes 290.532 allocated CPU-seconds,
or $0.0145266 published compute before credits.

The existing verified container forwarder is now activated only for the declared-type invocation.
The final combined workflow still selects every authored generator by default.
The development recovery explicitly selects only `sidecar.generated-type-consistency.1`;
its receipt distinguishes the full invocation inventory, requested invocation and prerequisite closure,
and declares that this partial request does not cover the whole external check.
Actual-program controls verify three valid scopes,
five rejected scopes,
an invalid selector accepted only with its guard removed,
and both prerequisite-gating outcomes.
Bounded recovery `62nbb3srhk` is requested at `2026-10-17T09:12:10Z`,
with both provider revisions equal to clean explicitly pushed
`6a17c8fc61cf7613432c9f481416fa9ef7cc7bfe`.
Its original corpus consumer passes with `corpus: kept 439, dropped 61`,
exit zero and no tracked or untracked generated drift.
All 13,715 frozen inputs and the strict authoritative after guard match;
five exact source/boundary log hashes and its actual attempt URL are independently verified.
The two completed development invocations are not repeated.
Its actual `4x16` provider usage is 940.56 allocated CPU-seconds,
or $0.047028 published compute before credits.
The job separately fails because `git worktree remove --force` emits `Permission denied`
while deleting the disposable consumer.
That failed predecessor records no output ownership stat;
ownership is not inferred from the error alone.

The cleanup helper now permits only a frozen invocation label's expected disposable path,
matching source and consumer HEADs,
shared Git repository and exact worktree registration.
It rejects symlinks, authoritative aliases/ancestry and stray root Git-directory entries.
Ignored-file dry runs and root Git-directory/ignore probes precede removal.
It records generated output metadata,
tries ordinary removal,
then handles only a verified orphan left by ordinary Git removal.
Local actual-program controls accept the ordinary fixture,
reject ten invalid fixtures at their intended guards,
and accept an unknown label only when that guard is removed.
Every control fixture is removed.
Cloud control `qsg8tx4bvb` uses a `2x8` provider job and an actual two-CPU/two-GiB/512-PID container
and measures a root `0:0` corpus directory with mode `0755` and file with mode `0644`.
It reproduces ordinary `Permission denied` and shows Git already removed the worktree registration.
The proposed privileged Git retry then reports `is not a working tree`; that hypothesis is rejected.
The corrected fallback requires the removed registration,
measured foreign-owned output,
unchanged authoritative HEAD/status and the same exact nonsymlink disposable path before confined removal.
Actual control `p3jlkgvl99` passes at `53514e17d6ca27fa74efb0bc8d0a731ec7707a84`.
Its image config equals the original declared-corpus pull's `7a0d2c0c...` config.
The fixture and all control containers are verified absent,
and strict source before/after identities match.
Its actual `2x8` usage is 119.006 allocated CPU-seconds, or $0.0059503 before credits.
Neither cleanup control repeats an original generator or gives source-check credit.

## Native reports and final adapter handoff

Authentic source control `knsghczl71` passes at clean-pushed
`392a3ea04128547d9b452dd3bea7a4df1235acd1`.
Both frozen build prerequisites pass.
The original TOML mutation task runs one explicitly selected source file with one container and shard size one,
retaining its authored full-suite context.
Its 37 source test files reach the baseline gate before the mutant loop.
The actual native report independently contains mutant `9cd1056a095eaea1`, status `compileError`, and zero unrun IDs.
This gives zero mutant assertion or whole frozen mutation-check credit.
The fresh native report is 650 bytes, SHA-256
`692e5dd934f799518de76762ee34a0f22558a66a2832188cf5b074dfc37d7825`.
Its ID and bytes are independently read from this `ef014` consumer, not inferred from another source revision.

The exact production snapshot/copy blocks execute with only the diagnostic input-receipt filename changed.
All retained files are hash-bound, and strict authoritative source before/after identities match.
The actual runtime image is `51434727e64ac7daef2c68ce8f962f9082cb3be27c2574999bea4fede500fc81`.
Its 33,427-byte baked metadata is copied byte-identically from root `0600` to source-owner `0600`
and bound readonly; original, copied and archived hashes match.
Actual volume and metadata-container cleanup verify.
The `2x8` provider consumes 360.034 allocated CPU-seconds, or $0.0180017 before credits.
Historical missing reports remain missing.

The verified adapter ref is `392a3ea04128547d9b452dd3bea7a4df1235acd1`.
Parent commits `d30d88df0d` and `ce052bc29c` integrate the generated cleanup task/caller,
native-report filesystem imports and snapshot/finally blocks,
and combined `logger-native-verification/` artifact path.
The exact block hashes and source strings are retained in the companion evidence.
Its strengthened canonical cohort/source/native/desktop helpers must remain intact.
The diagnostic-only pilot task/workflow is not required by the final route.
`completeGraph` remains false until the parent freezes and regenerates the complete common registration.

## Filed terminal boundaries

Distinct source incidents `#1145` to `#1156` record the coverage-baseline/gate failures,
matcher disagreement, separate p-map comparisons, separate async timeout groups and conf red baseline.
The six source host assertions remain `#1135` to `#1140`.
Scoped trial repairs `#1157`, `#1158`, `#1159`, `#1161`, `#1162` and `#1163`
are closed only after their actual consumer/control evidence is posted and remotely verified.
Native report incident `#1160` remains open for final common per-ID artifact reconciliation.
Existing PATH drift `#335` and rootless GID mismatch `#1047` receive additive immutable-source consumer proofs.
Every new title/body/label and posted comment is re-fetched and checked against its retained rendered draft.

The initial `#1162` body reversed the caller/receiver environment names; that direction is retracted.
Immutable harness `18e8c13173` sets `DEPOT_LOGGER_EXTERNAL_GROUP` in the pilot,
while its actual adapter reads `DEPOT_LOGGER_EXTERNAL`.
The corrected body and explicit correction comment are verified against both immutable definitions.
The original failure, correction commit and later consumer proof are unchanged.
A narrow process-rule proposal is retained in
[CI incident source-boundary evidence](../planning/ci-incident-source-boundary-evidence.md).

Related source classifier incident `#1142` shows why a native `killed` classification
alone is not literal assertion credit.
The deciding `test-run.ts` and `mutant-loop.ts` files are byte-identical
between logger `ef014` and its `525d2e` control.
Their source hashes are retained in the companion evidence.
No caught-spawn occurrence is inferred for the logger campaign, and its CLI totals remain source-native classifications.

Fresh controls at `392a3ea041` verify the candidate's 719 package identities,
19 external routes, 57 disjoint remainder selectors and 97 expanded combined jobs.
Both `root.browser` and `root.e2e` are covered by the combined route's 32 disjoint cells;
the standalone browser workflow is an alternative and must not also run.
Actual authored browser execution keeps zero retries because the source Podman boundary omits `CI`.
The configured two-retry CI branch remains unexercised under `#1131`;
its list-only positive counterfactual gives no source-test credit.
The VM route retains original `test:e2e` without arguments and its Node build prerequisite;
both Linux and Windows lifecycle cases remain source-blocked before registration by `#1076`.
The companion evidence's `freezeReadiness` binds the full receipt,
exact generated labels, clean remote-matched harness ref and fresh candidate controls.

The remaining execution belongs to the parent:
regenerate auxiliary digests and stale routing metadata when freezing the complete manifest,
bind every package/external/GHA/VM route to the same authenticated cohort,
then collect source failures, actual internal executions and prerequisite blocks separately.
The development union and these adapter controls establish no single complete passing-pipeline latency.

[mapping-incident]: https://github.com/Aquaticats/Monochromatic/issues/1116
[metadata-incident]: https://github.com/Aquaticats/Monochromatic/issues/1117
[candidate-correction]: https://github.com/Aquaticats/Monochromatic/commit/3ac9616096bc45fe4e02cddfec8a14fb9b6b72d6#commitcomment-205400693
