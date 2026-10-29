# Auto-mode frontier archive, 2026-10-18

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/task-util` is now `package/monorepo-task/core-exec`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/logger` is now `package/logger/core`;
> `@monochromatic-dev/module-logger` is now `@monochromatic-dev/logger-core`.

This record moved from `doc/handover/pi-auto-mode-current-frontier.md` on 2026-10-18.
The [current frontier](pi-auto-mode-current-frontier.md) owns the active queue and current verification status.
The historical next actions in this archive are not new authorization or a current queue.
The original body is retained without changing its evidence, decisions, or failed experiments.

## Purpose

Resume the axiom-based migration without reopening settled choices or replaying completed phases.
This current summary complements the
[historical evidence ledger](pi-auto-mode-axiom-evaluation.md).
Updated 2026-10-18.

## Reopened MiMo preparation failure on 2026-10-18

### Current status and user instructions

The MiMo lint handoff is **not ready**.
The user reports at least five premature readiness claims and explicitly requires continuing until the work is done.
Treat passing narrow catalogs as evidence for their exercised boundaries only.
Inspect actual Pi, Node, Linux, dependencies, and the configured consumer wherever the evidence points.
The user clarified that their concern is platform behavior exposing defects in our code,
not suspected defects in Pi, Node, or Linux themselves.
The user authorizes shared logger repairs and requires a version bump to trigger auto-publish if it changes.
The logger version is bumped to `0.4.2` after its build, full units, types, lint and guard-mutation checks pass.
The user authorizes invoking and monitoring MiMo directly through Pi RPC.
After readiness verification, run that agent and monitor the lint work rather than returning a manual handoff.
The direct RPC connectivity pilot passed on 2026-10-18:
configured `llmgateway-devpass/mimo-v2.6-pro` replied `READY`,
dispatched no tools, and exited normally inside a 2-GiB, 2-CPU disposable container.
Evidence: `${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/mimo-rpc-pilot.log`
and fixture `pi-mimo-rpc-preflight-WNIl6a`.
The controller resolves the repository's Mise environment without printing credentials,
and launches the maintained command from the repository root.
Calling the package `native:cli` task instead changes cwd to the package,
whose absent `AGENTS.md` makes this policy bootstrap fail before RPC starts.
Initial container probes also corrected a missing canonical home mount and disabled automatic tool installation.

The logger's final source build, all units and type check pass.
Its complete Oxlint sweep reports zero warnings and zero errors across 63 files.
Logs in the current scratch are `logger-release-build.log`, `logger-release-units.log`,
`logger-release-types.log` and `logger-release-lint-final.log`.
The unchanged 100,000-record burst retained 48,304 heap bytes in the preceding capacity verification.
The same-millisecond, separate-sink regression first fails with malformed JSONL at position 349544;
a serializer returning `undefined` separately produces invalid JSONL.
Both committed regressions pass after unique, exclusively created files and serialization validation.
The final suite also covers admission during an issued append and recovery after a real filesystem error.
Private source-copy guard mutations pass:
completion deduplication, missing JSON validation, filename identity and issued-batch admission each rebuild,
then produce the expected regression failure when weakened.
Production source and emitted entry hashes remain unchanged.
The receipt is `logger-guard-mutations-receipt.json` in the current scratch.
Root `TCV` stays unchanged; missed-boundary guidance belongs in package-local docs.
There is no adopted 13k line target.

The active output is `native-release-mmaT8k`, rebuilt through `dist/native` on 2026-10-18
after a fresh JavaScript build against logger `0.4.2`.
The emitted policy bundle is `native-policy-api-CtuEfa81.mjs`.
The logger publishing commit `3befa69c14` is on `origin/main`;
automatic npm release run `39104844467` has started.
The new complete configured-host replays pass with explicit artifact receipts:
faithful fixture `pi-mimo-failure-rpc-7vxnGu` and fragmented fixture `pi-mimo-failure-rpc-LD83ST`.
Their logs are `faithful-logger042-mmaT8k.log` and `fragmented-logger042-mmaT8k.log` in the current scratch.
Both complete all 52 calls with 62 live Luna requests, zero selectors, exit 0 and no termination signal.
Both actual payload audits pass their positive controls and find no excluded structured fields.
The final native verification matrix passes as `native-logger042-matrix.log`.
The configured prohibition, Stop, cancelled-confirmation and abort controls all withhold both members.
Their logs are `prohibition-logger042-mmaT8k.log` in the older scratch
and `configured-stop-logger042.log` in the current scratch.
The additional `final-lifecycle` task passes host lifecycle, then fails session-action mode `tree`:
`SourceCollectionError: Run message data has an unsupported prototype` while retaining a real navigation event.
The private `session-prototype-probe` task identifies the rejected field and runtime kind before any repair.
This additional failure prevents starting MiMo's lint edits yet.

The automatic logger release run `39104844467` fails publishing with
`ERR_PNPM_FAILED_TO_PUBLISH` and HTTP 404, matching the recorded `0.4.1` incident.
The established targeted workflow is dispatched as run `39105453035`,
but its pack step fails because `npm:module:pack` no longer exists in the generated root Mise config.
`file-enforcer.config.ts` owns `mise.toml` from `mise.no-env.toml`.
Restore the established pack/publish tasks in that source, regenerate only the managed root config,
then repeat the targeted logger workflow and verify a registry consumer.
The source tasks are restored and the original generator regenerates the root output without running concurrent owners.
Generated-output commit `d6d1d67b06` reaches `origin/main`.
Targeted run `39106517976` succeeds at that revision.
The fresh, disposable registry consumer installs logger `0.4.2`, verifies its public exports and repository,
and persists five ordinary, tagged, large, reentrant and independently owned-file records as valid JSONL.
Evidence is `logger042-registry-consumer.log` and `logger042-registry-receipt.json` in the current scratch.
The automatic workflow's HTTP 404 remains a separate path;
the targeted trusted-publisher recovery completes publication.

The stderr inventory is retained as `final-stderr-receipt.json` with the original raw files.
It includes deliberate private `nativeRunReserve` stack probes, a Synthetic catalog route rejected by the offline hook,
and tagged `SourceCollectionError: Collection cancelled` after deliberately withholding a group.
Those are distinct from bare shutdown errors; the first broad error-word audit falsely treated probe stacks as failures.
The direct connectivity pilot also reveals Pi's settings reader needs a write lock even to read project settings.
This project's current `.pi/settings.json` contains only `_comment`, with no packages or extensions.
The prepared direct MiMo controller now shadows the project `.pi` directory with a private writable copy,
preserving the original files and allowing the actual settings reader to lock them.
The persistent controller and atomic command sender are `mimo-rpc-session` and `mimo-rpc-command`
in the current scratch; they are prepared but have not started lint editing.
The direct diagnosis session is now live as `pi-mimo-rpc-session-jMWeUL`,
with controller tool session `18970` and log `mimo-rpc-session-diagnosis.log`.
It is performing read-only analysis of the signal boundary, not lint editing.
Its first reads expose missing `rg` on the container's initial PATH.
The controller stops a broad filesystem-discovery group,
supplies the measured executable path,
then sends a new prompt because Stop settles the whole Pi turn and a steering message alone does not resume it.
The next controller generation mounts `rg` in `/fixture-bin`.
Stop this diagnosis host before retiring its pinned `mmaT8k` release;
the eventual lint worker must start against the repaired native artifact.

The private signal probe identifies `field=signal`, `[object AbortSignal]`, constructor `AbortSignal`.
The installed Pi `AgentSession.navigateTree` emits that top-level control field in `session_before_tree`.
The probe changes only the error message and can complete some action modes despite retained capture-error diagnostics,
so action success alone is not a sufficient regression assertion.
Keep ordinary message/tool copying strict while repairing the constructor-owned native event boundary.
The committed RED regression is `43accdb953` and task `test:native:event-controls`.
`native-event-controls-red.log` shows the original callbacks succeed,
then the retained-source read throws `SessionInputCaptureError` with the unsupported-prototype cause.

Live MiMo diagnosis later reaches selector `c0b612fd-08a5-415c-ae5a-97ead39b4ddd` after 36 tool results.
The diagnostic is `Original policy review and evidence closure failed`;
stderr identifies `Native run prompt was replaced after this source snapshot`.
A queued RPC steering message was sent during the live turn.
Keep this freshness incident separate from the `AbortSignal` event issue;
verify its deciding source and subsequent-turn refresh before calling it a product defect or an expected transient.
The controller authorizes only the displayed two scoped source-search commands through RPC Allow.
Evidence is `mimo-live-closure-failure.json` and the raw live host stderr.

Reviewed cleanup removes eight inactive releases after an empty, freshly measured native consumer audit.
The active `mmaT8k` and verified rollback `mHAoem` remain.
The post-cleanup package measures 22,933,936 regular-file bytes and 20,180,992 unique allocated inode bytes,
across 2,912 regular files; shared dependency symlink targets are excluded.
The cleanup receipt is `reviewed-native-cleanup-receipt.json` in current scratch;
the footprint JSON remains in the older lint-approval scratch.
The corrected process-consumer audit detects both live `/fixture-node` hosts by Pi title and fixture inode.
Cleanup must wait for every consumer to terminate.

The preceding `native-release-XGbh0H` affected verification matrix passes.
Its faithful configured-host replay completes all 52 original calls,
including both reported failures,
with no selectors and host exit 0.
All 62 actual Luna requests pass the structured-payload audit.
The driver initially rejects successful inspection output containing the diagnostic phrase;
the corrected saved-event audit detects both original real failures and excludes that successful read.
Its evidence is `pi-mimo-failure-rpc-WHbepQ/terminal-audit.json` in private agent scratch.
The fragmented-stream replay `pi-mimo-failure-rpc-2nQOZg` still exits 134 after 56 live assessments.
Current full Oxlint inventory is 13,834 warnings and 83 errors across 818 files;
the new retirement fixture's six errors are fixed.
Independent current fragmented probes profile retained allocations and replace only private log append completion.
Their fixtures are `pi-mimo-failure-rpc-3umw1N` and `pi-mimo-failure-rpc-ekAFp9` respectively.
Both remain bounded to two CPUs and 2 GiB and preserve every original callback and log record.
Both private fragmented controls terminate with exit 134.
The append control changes the policy bundle,
but allocation frames point to the separately loaded logger TypeScript source;
its application receipt does not prove it changed the active append path.
Do not use that control to rule out the logger.
The collected heap `pi-mimo-failure-rpc-bUKHKw/retainers.heapsnapshot` contains a large logger pending-write set.
Its private analysis completes under the same 2 GiB bound.
An isolated logger burst delivers 100,000 records while retaining 93,858,312 to 93,859,232 bytes
across three unchanged runs.
A private promise-deduplication change retains 37,120 to 37,152 bytes across three runs,
with all 100,000 sink invocations preserved.
That ownership repair is implemented with asynchronous file batching and shared synchronous-sink completions.
Actual-host verification remains required after the logger's publishing bump and consumer rebuild.
No compressed receipt representation is justified by the current retainer evidence.
The maintained built-artifact burst regression fails at 93,863,816 retained bytes before the repair.
The file regression also reproduces corrupted JSONL when a record's `toJSON` calls the same sink reentrantly.
Source repairs now share pending observations,
reuse synchronous sink completions,
and batch captured complete file records under a 64 KiB aggregation bound.
The logger's package-local explanation is `package/module/logger/doc/write-completion.md`.
The source directory test now uses built artifacts and captures actual child stderr through clean shutdown.
The required publishing bump is `0.4.2`.
The remaining independently verifiable work is:

- Verify the logger `0.4.2` publishing commit reaches the remote.
- Rebuild auto-mode JavaScript, then its owned native output against the repaired logger.
- Run both complete configured-host replay variants with corrected terminal validators.
- Audit actual Luna payloads and exercise prohibition, Stop, cancellation and cleanup boundaries.
- Render the package and canonical documents; update the architecture and handoff measurements to the final artifact.
- Complete the positive-control consumer audit, cleanup dry run,
   reviewed inactive-artifact removal and footprint measurement.
- Invoke MiMo through Pi RPC, monitor the lint repair, and validate its resulting changes.

Earlier outputs still exit 134 from Node heap exhaustion;
the private retirement-only replay instead reaches its 600-second driver limit after 58 live assessments.
The immediate work is diagnosing the remaining fragmented-stream retention,
then complete consumer verification,
artifact cleanup and measurement,
durable package documentation,
and the corrected MiMo handoff.
Private experiments are not production repairs.

### Incident and completed repairs

The user supplied later MiMo calls that return `Complete group preparation or assessment failed`.
The readiness claim applies only to the earlier replay corpus and does not establish readiness for these calls.
The actual session is `2026-10-17T15-47-34-073Z_01a14a8c-0a38-760f-aca3-aa17ebd2a725`.
Reported calls are `call_cec448f1353443bd88da096e`
and `call_8cacc98bebf7433a9945f1d3`.
They belong to one parallel group.
Earlier Python heredocs in the same session succeed,
including `call_fe5cc4c2d73542b097c79140`.
The incident used `native-release-qWSxFz`;
the active output changed to `native-release-6z61x7` on 2026-10-18.
Private evidence and the active reproduction work live in
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018`.
The original log retains a `RangeError: Invalid string length` in run prompt owner `charge()`
while capturing retained context inputs.
Automatic review then presents a manual selector at `2026-10-17T16:14:03.781Z`.
The selector returns at `2026-10-17T17:18:03.286Z`,
records the verdict,
and calls the native Stop path.
The dispatcher replaces that result with the generic preparation failure after seeing the aborted signal.
Configured-host replay `pi-mimo-failure-rpc-xWj1Z0` reproduces the generic failure after Stop;
its assertion rejects the exact reported message.
Restoring only persisted history does not reproduce the original capture chain:
it instead loses usable request evidence and falls back to manual approval.
The active full replay therefore recreates every original MiMo proposal from a fresh session,
under a two-CPU and 2 GiB container bound,
with the original repository read only.

Next actions:

- Measure capture growth through the original proposal sequence and repair the prompt capture failure.
- Add a genuine native Stop regression and preserve its withheld reason through cancellation.
- Rebuild and replay the reported group and denial controls before renewing the lint handoff.

Source fixes are committed in `2f90217589` and `daf6b8e179`.
Build `native-release-cXp8co` passes the actual aggregate string-limit regression,
the Stop/cancel regressions,
the complete decision fallback catalog,
run history,
prompt capacity,
run-message freshness,
native phases,
and single-call catalogs.
Type checking passes with a disposable writable package cache;
the first read-only container attempt fails only because its cache cannot be unlinked.
The full character-fragmented MiMo replay still exhausts the 2 GiB container,
so readiness remains open.
The unchanged repeated-JSON capture fixture retains 688,995,568 to 689,029,488 heap bytes across three runs.
Commit `77620b6c23` shares equal stored text and hashes inside the existing pool,
while retaining every source occurrence and publication separately.
Its regression includes a distinct later input and keeps per-occurrence byte accounting.
Next: measure this change against that baseline,
complete the original configured-host replay,
check actual outgoing payloads and prohibitions,
and finish the durable troubleshooting record.

Shared-text fixture runs retain 3,609,280 to 3,610,472 heap bytes with unchanged receipt accounting.
The source-sharing build is `native-release-iQw5fd`.
Its full unit suite passes.
The first full replay of this build reaches the private recorder's 4 MiB output cap;
this is not another recorded host heap failure.
The recorder now discards transient `message_update` events while retaining actual terminal messages,
tool results,
selectors,
provider requests,
and stderr.
The actual host still consumes every original streamed delta.
The revised full replay remains required before readiness is renewed.
The native-root guard mutation is running in a private module loader;
production source and output remain untouched.

The native-root guard mutation is killed by the new regression;
the production owner hash remains `71858a7f9c81f979050396fba9f629f437d5642a34b5446032511dc308e757a8`.
The fragmented replay still exhausts the heap on `native-release-iQw5fd` after 26 live assessments.
The faithful assistant-text replay instead hits the old private bridge's 30-request limit;
that is a separate harness failure.
The bridge now permits at most 128 requests for the complete original proposal corpus.
Real configured-host prohibition control `pi-hyper-rpc-allow-SSsGVq` withholds both process and Bash members,
with zero selectors and three live Luna responses.

`test:native:event-sharing` reproduces callback-phase payload duplication through the actual native runner.
The old build fails its storage assertion while retaining all phase values correctly.
The new event snapshot stores distinct copied values once per emission,
with ordered phase records referencing their original value ordinal.
This changes storage only:
no callback phase,
mutation,
output,
or source occurrence is discarded.
The existing native receipt fixture expands this lossless representation before checking all original assertions.
The source is committed and rebuilt as `native-release-6z61x7` on 2026-10-18.

The rebuilt output is `native-release-6z61x7`.
The event-sharing regression retains all 60 phases from 30 callbacks in 36,476 bytes,
against 30,984 bytes for a single callback.
The complete native event receipt catalog,
capacity,
fallback,
history,
prompt capacity,
message freshness,
full units,
and type check pass.
Both complete host replays still terminate with Node exit 134:
fragmented fixture `pi-mimo-failure-rpc-1RoD9a` after 28 live assessments,
and faithful fixture `pi-mimo-failure-rpc-abwWtu` after 34.
No readiness claim is renewed.
The next private probe measures original pool retention by source kind and native method,
unique content characters,
metadata characters,
and heap usage.
Ranked hypotheses are retained source snapshots,
other retained context versions,
and pending diagnostic writes;
the source counters can falsify the first hypothesis before another storage change.

Private pool profiling reaches 103,200 retained occurrences,
including 102,228 tool-loadout query observations,
while unique content totals 61,899,464 characters and the sampled heap reaches 1,396,560,032 bytes.
The source is `pool-profile-summary.json` in the incident scratch;
the profiled fixture is `pi-mimo-failure-rpc-uaHgtz` and still exits 134.
This distinguishes source text volume from the overhead of retained publications and other owners.
A private module-loader probe now measures whether input-only publication views unnecessarily retain
the complete message-projection owner.
Input-only views cannot match provider payloads;
the probe shares their false matcher without changing distinct occurrences or native aliases.
A separate faithful replay removes only private diagnostic module rewrites,
to check whether the recorder's stack tracing contributes to the heap failure.

The clean replay also exits 134,
so removing private diagnostic rewrites is insufficient.
The private input-view probe lowers the existing 600-input fixture's retained heap to 2,753,896 bytes;
this is not a complete-session repair and is not applied to production.
An inspector sampling profile at 20,000 original receipts retains its explicit 2,097,168-byte
positive control and attributes the largest sampled allocation group to queued file-log writes.
The sampled profile and its paths are private incident artifacts.
A separate configured-host control changes only the bundled logger's append completion to synchronous I/O,
preserving all records and native callbacks.
Fixture `pi-mimo-failure-rpc-YW5iY2` confirms that the early loader hook applies and still exits 134.
The earlier late-hook attempt has no application receipt and is not causal evidence.
Production logger source remains unchanged.

### Latest allocation evidence and immediate next action

`allocation-summary.json` in the incident scratch summarizes fixture `pi-mimo-failure-rpc-e8GtWu`
at 100,000 original receipts.
Its retained 2,097,168-byte positive control is present.
The largest sampled allocation is 476,526,736 bytes attributed to `bindOriginalRunMessage`
in `native/contract/integration/policy-source/run-messages.mjs`.
`createMessagePublication` accounts for another 153,547,688 sampled bytes.
Total sampled live allocation is 1,150,318,896 bytes.
The earlier 20,000-receipt logger profile was a startup snapshot, not the complete-session cause.

Each original source currently receives its own frozen view plus closures for identity and freshness checks.
A private class experiment measures 120,000 genuine source bindings across twelve governing snapshots.
It reduces a measured 48,741,184-byte retained-heap result to 20,186,064 bytes.
This is preliminary evidence, not an accepted implementation:
the class changes the plain-record prototype and constructor surface.
The scratch harness also emits a read-only default-log-directory error;
set `MONOCHROMATIC_LOG_DIRECTORY` to its disposable writable home before trusting repeated measurements.

The old bundled policy lacks that environment support;
the corrected probe mounts its discovered log directory writable and preserves every diagnostic.
Three unchanged runs retain 48,742,376 to 48,742,952 bytes.
The plain-record shared-method probe retains 30,762,600 to 30,763,960 bytes across three runs.
The committed red test `c6ece26944` fails at 48,724,128 bytes against its 36 MiB budget.
Source repair `23f24480de` preserves plain records and checks receiver membership through a per-snapshot weak set.
Both JavaScript policy and native ownership output are rebuilt as `native-release-2vxGY8`.
Its source-binding regression passes at 30,747,064 retained bytes,
including forged receivers, cross-snapshot methods, substituted sources, and retired parents.
Complete faithful and fragmented configured-host replays are running against that output.
The isolated memory reduction does not establish complete-session repair.

Both `native-release-2vxGY8` replays subsequently exit 134:
fragmented fixture `pi-mimo-failure-rpc-XYDlO0` reaches 34 live assessments;
faithful fixture `pi-mimo-failure-rpc-7osKnn` reaches 42.
The original target corpus contains 31 assistant proposal groups and 52 tool calls.
The complete native lifecycle/fallback/freshness/execution catalogs, units, and type check pass.
Final formatted source-binding and event-sharing regressions pass;
both receiver and native-root private mutations fail their intended assertions without changing production hashes.

Immediate investigation now follows completed group ownership:
`native-batch/prepared-group.mjs` closes by setting only `state.closed`.
Its retained context callbacks still reference the full review and its request-bound input views.
A private retirement counterfactual will replace only closed-group review references with the terminal outcome,
while the runtime still retains the actual latest full review for diagnostics.
This is not applied to production and needs live consumer proof plus lifecycle/freshness regression coverage.
The user was told that the fragmented replay still fails; readiness remains withdrawn.

The committed native review-retirement regression `eeef4291d1` retains 96,805,456 bytes on the old output.
Its private retirement probe retains 8,792,696 bytes while preserving the actual latest full diagnostic,
terminal outcomes, source checks, and rejection of execution after closure.
Maintained repair `8ad0d4725b` releases completed review references and rejects closed-group execution checks.
The rebuilt output is `native-release-qEBKwW`.
Fresh faithful and fragmented host replays plus the complete affected verification matrix are running.
The earlier private full retirement counterfactual continues separately on its original artifact.

An independent private probe measures the pool's immutable whole-array copy on every appended receipt.
It compares genuine manager append CPU and memory against publication of cached frozen snapshots on read.
It must preserve prior frozen snapshots, stable repeated reads, every occurrence, and distinct publications.
No pool append implementation change is applied yet.

The append implementation is now committed as `80803f87db` and rebuilt in `native-release-CV2Wga`.
The committed red CPU guard fails at 9,989,359 microseconds;
the maintained implementation passes at 483,453 microseconds.
Three original 50,000-input probes measure 9,125,882 to 9,450,071 microseconds;
three private cached-snapshot probes measure 441,879 to 543,405.
The snapshot probe has higher final retained heap, about 95.6 MB versus 90.8 MB;
its established benefit is append CPU work.

Input-only publication storage is also repaired in `1edb75d52c`.
Its baseline retains 3,607,008 to 3,608,432 bytes across three unchanged capacity runs.
The committed red guard fails at 3,607,384 bytes;
the private shared-function probe passes at 2,753,832 bytes.
Actual message-payload, system/tool contribution, original occurrence, and native-root checks remain required.

The combined output is `native-release-XGbh0H`.
Final controllers are `final-faithful-replay.ts` and `final-fragmented-replay.ts`;
their logs are `faithful-retention-final.log` and `fragmented-retention-final.log`.
The complete affected matrix is `retention-final-matrix.log`.
These finite RPC replays retain the 2 GiB/two-CPU bounds,
add Podman's 1,200-second native container bound,
and set the parent driver to 1,260 seconds so client termination is distinguishable from the container result.
The earlier private fixture `pi-mimo-failure-rpc-9dEcRp` ends with SIGKILL after about 601 seconds,
matching its driver's 600-second bound;
captured stderr has no heap-abort diagnostic.
A subsequent Podman inventory contains no container mounted to that completed fixture.
This is separate from the exit-134 failures.

Final receiver, native-root, completed-review, and input-only-owner mutation controls are running privately.
The live process/Bash prohibition control is running on the combined output.
No full-session success has been established yet.
Final payload audits, full-package lint inventory, rendered docs, reviewed artifact cleanup,
footprint/source inventory, and remote commit verification remain in the completion queue.
An auto-push lock-wait message occurred during commit landing;
a subsequent remote check contains the replayed append-test commit `7747823556`.
The latest final commits still need a concluding remote check.

Next, measure shared methods within each governing snapshot,
using a private `WeakSet` to authenticate the original frozen plain binding records.
Preserve every source occurrence, request identity, original ordering, and parent freshness check.
Exercise wrong-member, forged-receiver, cross-snapshot, substituted-source, and retired-parent cases.
Commit a genuine failing memory regression before applying the production repair.
Use the existing private `binding-capacity.mjs`,
`prepare-binding-probe.ts`,
and bounded binding runners as experiment scaffolding, not production source.

Next actions:

- Repair source-binding allocation with measured baseline spread and a genuine red regression.
  Rebuild the maintained native contract after the source change.
- Repeat full native units,
  both configured-host replays,
  outgoing-payload audits,
  source-root mutation,
  and the live prohibition control against the final output.
- Resolve only new lint findings,
  record the remaining full-package debt for MiMo,
  update package/troubleshooting/handoff docs,
  render Markdown,
  and review inactive generated releases before final footprint measurement.
- If shared logger source changes, verify it and bump its package version for auto-publish.
- Consolidate final handover and package-local ownership lessons;
  verify the final scoped commits reached the remote before reporting readiness.

Replay harness cautions:

- Concurrent controllers overwrite `latest-fixture.txt`; pass explicit fixture paths to auditors.
- The live bridge permits 128 requests;
  `prepare-replay.ts` can restore an obsolete 30-request bridge if regenerated carelessly.
- Parent recorders omit transient `message_update` events to avoid their own output cap;
  the actual host consumes every streamed delta and terminal results remain recorded.
- The bounded native owner rebuild copies existing `dist/final/node` policy output.
  TypeScript policy or logger changes require a fresh JavaScript build before that rebuild.

## Historical MiMo lint handoff on 2026-10-17

The current handoff is [MiMo lint handoff](pi-auto-mode-mimo-lint-prompt.md).
The prepared group now owns automatic and manual review and execution membership.
Root and child groups share actual native prepare/run/finalize phases.
The duplicate manual group,
original-source approval router,
child dispatcher,
and unused whole-call native API are removed.
The remaining source compiler authenticates questions,
source attribution,
model responses,
and asynchronous freshness rather than owning another execution path.

Full units and type checking pass after migrating API catalogs to actual native slot phases.
The phase-only API build `native-release-mHAoem` passes all 22 real lifecycle modes,
all 24 fallback modes,
and native execution catalogs.
The final build `native-release-qWSxFz` includes the final type/documentation changes.
Configured replay `pi-hyper-rpc-allow-nTpk0q` completes all 16 original calls,
with zero selectors and 17 actual live-Luna requests.
Positive-controlled inspection of the actual requests excludes raw structured commands,
thinking,
and signatures.
Prohibition control `pi-hyper-rpc-allow-gSMsqH` withholds both process/Bash members,
with zero selectors and three Luna responses.
The process-local mutation removes both independent denial barriers;
the actual native test executes three harmless fixture commands and fails with `3 !== 0`.
Both production artifact hashes remain unchanged.

The package measures about 22.9 MB after reviewed cleanup,
including the 8,211,396-byte active output and a verified rollback.
Shared external dependency symlink targets are excluded.
The emitted runtime graph contains no missing static targets and records five computed import sites.
Its 794 conservatively reachable files total 6,099,319 bytes;
the complete output contains 1,030 regular files and 8,211,396 bytes.
The source parser counts 813 tracked files,
65,692 nonblank code lines,
29,136 runtime statements,
and 28,596 code lines in filename-classified verification.
The user did not adopt a 13k line target.

Cleanup dry-run and root stray-Git-entry checks precede removal of 43 inactive generated releases.
The active output and `native-release-mHAoem` remain;
no live native process references an old release at cleanup.
An uncapped package-consumer scan has no hits outside this package.
The final 622 unchanged logs preserve 265,115,894 original bytes as 10,014,676 compressed bytes.
Independent combined-archive verification checks all 13,248 files and 7,329,290,087 decompressed bytes,
and rejects changed same-length content in valid gzip.
The first combined audit correctly rejects the old single-batch membership count;
the corrected auditor checks both original inventories and every manifest entry.

The existing logger now accepts an explicit directory or `MONOCHROMATIC_LOG_DIRECTORY`.
Package tasks use private task diagnostics outside the package.
The logger build,
full unit suite,
and type check pass.
Directory tests exercise real filesystem records and independent default-logger child processes.
Ancestor-discovery fixtures explicitly clear the new environment setting;
concurrency verification now waits for both verifiers to start before releasing either.
Its original elapsed-time assertion failed twice at about 94 and 97 ms against a 50 ms threshold.
Root `TCV` remains unchanged;
the provider-payload lesson stays in package docs.

Full maintained Oxlint reports 83 errors and 13,686 warnings across 813 files and 486 rules,
using one thread in the disposable 2 GiB,
two-CPU runner.
This supersedes the intermediate 30-error,
13,163-warning snapshot and is an explicit increase in lint debt after phase extraction and real fixtures.
MiMo's next task is the complete lint queue,
using the per-file error index and preserving the verified behavior.
Automatic release eviction and diagnostic retention remain unimplemented;
current build paths are immutable for running consumers.
The original second incident's discarded underlying transport cause remains unknown.
Desktop notification termination is separate.

## Active work on 2026-10-15

### Reopened MiMo lint handoff on 2026-10-17

The readiness claim covered the saved Bash/read interaction only and was too broad for the MiMo handoff.
Session `2026-10-17T04-53-14-520Z_01a14834-fc97-70de-9615-03cc0636384b` reports separate failures:

- [x] Resolve and verify the mixed `process`/`bash` group containing `chatcmpl-tool-b60c3965f752386f`.
  Whole-operation approval scores `0.48`, denial `0.34`, unknown `0.18`.
  The model answered; the existing cutoff correctly required confirmation.
  Its captured input includes raw `process.start.command`, outside the prior Bash-only projection.
- [x] Resolve and verify the Bash-only group containing `chatcmpl-tool-a7a4468548c5838a`.
  Clause assessment succeeds; the whole-operation request throws `AssessmentTransportError`:
  `Assessment HTTP request or response failed; no provider answer was admitted`.
  The discarded underlying exception prevents identifying its cause from this log alone.
- [x] Replay both groups with preceding turns through the configured installed consumer,
  audit transmitted inputs, and run prohibited-operation controls before renewing readiness.

The original session and eight runtime logs are copied privately to
`${HOME}/temp/agent/pi-mimo-lint-approval-20261017`.
`mise run target-evidence` there isolates both captured groups and their model evidence.
The `notify-send` SIGTERM warning concerns notification delivery and does not establish either review failure's cause.
The next action at that point was replaying captured requests to distinguish ambiguity from transport failure.

The unchanged mixed-group request reproduces `approve: 0.48`, `deny: 0.34`, `unknown: 0.18`
in three live trials.
Replacing only proposed `process.start.command` copies with the incumbent parser's structure
returns `approve: 0.69`, `deny: 0.20`, `unknown: 0.11` in three live trials.
Commit `ac6b8f5cde` applies that projection to proposed and historical process starts.
The built projection regression was committed red and then passes,
including original input retention and non-start process actions.
Three unchanged replays of the Bash-only final request succeed at `approve: 0.88`.
Its historical discarded transport cause remains unknown.

Commit `065b2683d2` classifies actual fetch TypeError failures without borrowed text or causes
and connects the existing original judgment retry owner.
The native regression first fails with two dispatches instead of three;
the fix completes an approved group without confirmation after the injected failure,
and executes no member when the retried answer is denial.
The expanded fallback matrix and type check pass before architecture consolidation.
They require a final rerun against the consolidated release.

### Package size and architecture work authorized on 2026-10-17

The user also requests a size investigation, implementation of a manageable architecture,
and package-local justification of every major part.
The size measurement uses `du --bytes --max-depth=2 package/pi-plugin/auto-mode`:
7,354,649,993 bytes total, including 7,089,712,671 bytes of package-local diagnostic logs,
258,159,664 bytes of build output, 2,265,351 bytes in `src`, and 2,025,723 bytes in `native`.
These are apparent filesystem sizes, separate from tracked-source complexity.

- [ ] Inventory tracked production, test, research, generated, and obsolete code and actual runtime reachability.
- [ ] Reduce repeated diagnostics and retained build duplication without losing decision reconstruction
  or invalidating active sessions.
- [ ] Consolidate maintained modules behind explicit interfaces, removing superseded paths after proving consumers.
- [ ] Add package-local purpose, invariants, alternatives, callers, and verification for every major retained part.
- [ ] Verify permission failures, installed interactions, lint, types, tests, and measured before/after size.

The root `TCV` rule stays unchanged as explicitly requested.
Concurrent unrelated edits remain outside this work.

### Structural redesign required after size clarification

The user rejects both the measured approximately 400 MB directory and a hypothetical 100 MB package.
The user also expected substantially less implementation than the 13,000 lint-warning count suggests.
The user explicitly clarifies that 13,000 is not a code-size target.
Do not turn that comparison into an arbitrary line budget or declare storage archival sufficient.
The earlier consolidation and component documents are a baseline,
not completion of this clarified maintainability request.

The validated parser-based inventory now records 68,040 code lines and 30,301 runtime AST statements
across the captured 830-file scope.
It removes actual comments and type-only declarations,
but remaining annotations and string data still count as source lines.
The static runtime graph contains 20,153 lines across 284 files;
verification contains 30,073 lines across 300 filename-classified files.
Generation prevents treating the remaining 17,568 lines as dead code.

All three independent design reviews select the same substantive seam:
review a complete group between Pi's real `prepareToolCall` and `executePreparedToolCall` phases,
then reuse `finalizeExecutedToolCall`.
One private group record replaces separate automatic/manual execution ownership and evidence-handle round trips.
The [package-local design](../../package/pi-plugin/auto-mode/doc/compact-review-design.md)
records retained semantics,
specific deletion targets,
and the migration's verification order.
The original source-attribution and serialized-primary-request contribution requirements remain in scope.
Clause compilation and whole-operation fallback remain distinct.

An intermediate configured-host replay reaches all 16 original calls and issues no confirmation,
but an intermediate `process.output` returns `Complete group preparation or assessment failed`.
The two reported final Bash calls succeed.
Do not claim the full interaction passes until the intermediate failure is diagnosed and resolved.
Fixture `pi-hyper-rpc-allow-jZsxRg` retains the events and runtime logs.

### Native phase verification and remaining freshness failure

The maintained `test:native:phases` consumer suite passes its 12 lifecycle modes against `native-release-GSPIDj`.
It exercises actual Pi preparation,
Bash execution,
awaited update subscribers,
extension callbacks,
group withholding,
cancellation,
custom notifications,
user steering,
and policy-file changes in disposable sessions.
The notification-context followup also reaches a later primary request and a fresh reviewed tool group.
The notification-turn followup augmentation still needs its focused rerun.
The copied-artifact denial mutant executes three members and fails the maintained test as required;
`native-phase-mutant.json` retains the production hash and unchanged-artifact check.

The intermediate `process.output` failure is reproduced by queuing native custom data during assessment.
`run-prompt-custody-v3/lifecycle.mjs` previously replaced the run-composition object
when the constructed-input pool gained any entry.
This does not prove instruction text changed.
Pending custom data now remains captured until actual context admission without replacing that run object.
Queued user instructions and changed policy files still withhold every member without another approval dialog.

Full configured replay `pi-hyper-rpc-allow-GRsLAm` confirms that `process.output` now succeeds.
It then stops before assessment of the next Bash pair with `Observed request is no longer current`.
The retained cause is `SourceCollectionError`:
`Only original unregistered manager or native event input occurrences may extend this source snapshot`.
The rejected appended origin is not yet identified.
This is a separate failure from the replaced-run-object check;
do not claim the full interaction or lint handoff is ready.
Next action: identify the rejected original input at `manager-source-growth.mjs:88`
using the configured disposable replay,
then reproduce it in the maintained native consumer fixture.

That next action is complete.
The new `notification-primary` consumer case reproduces the exact rejection against `native-release-GSPIDj`.
Instrumentation identifies an original `native-custom-construction` with role `custom`,
permission `false`,
matching publication and source occurrences,
and no registration or run replacement.
Commit `10931aaacf` admits authenticated custom-data origins through the existing source-growth checks
and shares their classifier with lifecycle reads.
All 18 maintained lifecycle modes pass against `native-release-J4fNNL`,
including custom data delivered through context,
steering,
and followup during primary requests.
User steering and policy changes during primary requests or assessments execute no proposed member.
Types pass.
The full configured live-Luna replay is running;
its receipt is required before calling either reported interaction fixed.
The mocked configured replay `pi-hyper-rpc-allow-tN2zEc` exits `137` without an established cause
and does not count as passing verification.

The configured live replay now passes against `native-release-J4fNNL`.
Fixture `pi-hyper-rpc-allow-henGUH` completes all 16 original tool calls,
including `process.output` and both reported groups,
without confirmation.
All 17 actual Luna requests pass the payload audit with no raw structured command fields,
thinking,
or signatures;
the audit's authored positive controls detect excluded fields and preserve literal text.
Prohibition fixture `pi-hyper-rpc-allow-vDIezK` blocks both process/Bash members without confirmation
and receives three actual Luna responses.
The package documents and troubleshooting/frontier render through CommonMark,
and all local documentation links resolve across the 18 rendered files.
These proofs complete the reported permission-flow repair at this release;
the structural redesign,
final lint handoff,
and release/storage reduction remain open.

The emitted runtime graph now follows generated imports,
absolute file URLs,
loader replacements,
and type erasure.
Its `native-release-J4fNNL` baseline has 1,499 regular files totaling 12,743,268 bytes.
The conservative graph reaches 781 files totaling 5,958,591 bytes.
It does not prove all remaining files are removable:
five computed imports and runtime assets require explicit accounting.
The source and compiled qualified SDK trees are both currently copied into each release.
Next build change: compile from the verified checkout directly,
retain emitted runtime/assets and qualification metadata,
and compare every qualified SDK runtime file against this baseline before consumer verification.

The child dispatcher cannot be replaced with a thin phase adapter alone.
`program-runner-execution-callbacks.ts` invokes the original child slot around the whole native call;
its invocation lifetime currently encloses both preparation and execution.
The private-group consolidation must retain that lifetime through finalization when removing its token protocol.

The direct-checkout compilation change is implemented in commit `df529bd748`.
Bounded build `native-release-FcmP01` uses two CPUs and 2 GiB with disposable home/log directories.
Its regular-file total is 8,225,739 bytes across 1,031 files.
The comparison against `native-release-J4fNNL` finds all 507 qualified SDK runtime files and assets
byte-identical (5,234,528 bytes).
The 18 lifecycle cases,
disposable successive-release/publication rollback test,
and package type check pass.
This release-size improvement does not complete the requested source architecture reduction.

Commit `799f43d1d8` shares root and child execution/cleanup instead of maintaining separate bodies.
Its bounded verification passes the 18 native lifecycle cases,
actual progress callbacks,
single-call profiles,
the 24-case fallback matrix,
and execution/child-lifetime catalogs.
The former `execute-child.mjs` is removed;
its provenance entry retains the original path/hash and records the current maintained path and move date.

Commit `167a6e7dfa` consolidates native membership and execution state across automatic and manual review.
`prepared-group.mjs` owns disposition and scoped confirmation.
`prepared-group-context.ts` owns the shared root/child member and execution maps.
The source compiler borrows those exact records and original prepared serialization.
The separate `manual-group.mjs` owner and automatic/manual execution lookup branches are removed.
This internal context is never accepted from provider responses or tool arguments.
Build `native-release-Q1COLq` and types pass.
The first native consumer run on `native-release-hvOFxD` fails before all three effects:
the shared execution check calls a public source operation from inside another public source operation.
The actual exception is `JudgmentConstructionError: Judgment operations cannot reenter through callbacks`.
Commit `7b9faaa6e7` binds the existing unwrapped dependency check for native entry;
it retains the public freshness checks at the outer dispatcher boundary.
Constructor and source-preparation fixtures now create genuine native group records.
Source preparation failure retains those prepared records without publishing a source-policy selection.
On `native-release-WjFpol`,
the 18 native lifecycle cases,
progress callbacks,
single-call program profiles,
24 fallback cases,
execution/child-lifetime catalogs,
constructor/preparation tests,
and package type check pass in bounded disposable consumers.
The followup restricts manual entry to the actual scoped Allow response,
without retaining eligibility from a failed automatic review.
Next action: replace the child's deferred dispatcher with the same actual native phases,
preserving its original enclosing invocation lifetime,
then repeat configured-host and payload/prohibition verification before renewing readiness.

Commit `7a36bd67ef` replaces the child's deferred dispatcher body with the root phase dispatcher.
Factory-owned native slots reserve their existing return record before actual preparation starts.
The shared dispatcher captures every completion,
including withheld members,
before forwarding completion events.
Terminal capture does not reread parent eligibility;
new execution and script-value consumption retain their currentness checks.
The existing whole-call helper remains temporarily for its old consumers and tests.
The first build fails because the ordinary qualified SDK index does not export the new generated phase factory.
The next build shows that relative imports are rebased against the original Session location.
The factory import now has an explicit generated-loop override in `prepare-session.ts`.
TypeScript also rejects an unknown logger argument and an optional child signal;
the fixes use the existing caught-value formatter and original parent-signal fallback.
The corrected type check passes.
The corrected build `native-release-5QqLSO` passes the existing bounded native catalogs.
The direct dispatcher import subsequently builds as `native-release-1DiyGD`.
The maintained phase suite now also runs genuine Codemode child contrasts:
normal completion,
a before-hook block on the second parallel member,
a hidden argument property added by an actual extension callback,
and cancellation inside child preparation.
All 22 modes pass with the original caller,
actual SDK phases,
and captured cleanup stderr.
The hidden-property case proves that the peer is withheld,
the invalid slot records `threw` without conversion,
and already prepared peers retain one native error conversion.
The positive control executes all three children.
The first new fixture run fails before construction because a suggested workspace wait package is not installed;
the fixture now uses the same Node timer import as the existing native phase consumer.
The full bounded unit run reaches the 22 passing modes and publication rollback case,
but two old runner test files still call the removed whole-call dispatcher callback.
Those assertions are retired after their replacement passes through the actual SDK;
the remaining queue,
host callback,
member association,
and failure tests stay in place.
Full unit verification must rerun after that test migration.

The migrated full bounded unit suite and package type check pass on `native-release-1DiyGD`.
That includes the 22 real native lifecycle modes and disposable publication rollback.
The next consolidation removes a remaining second confirmation owner:
`decision-core.mjs` now returns compiled policy and source currentness,
and `prepared-group.mjs` handles both uncertain outcomes and assessment failures through one scoped selector.
Established denials remain terminal;
stale established results throw `EstablishedPolicyDecisionError` instead of requesting another generic Allow.
Confirmation checks source dependencies directly when entering from an existing compiler operation,
and retains the complete compiled currentness check outside that wrapper.
Unknown headless outcomes stay withheld without opening a selector.
The old `finishWithApproval` router is removed.
The new build and confirmation matrix are pending;
the removed router's original-source TUI adapter still needs a usage audit before retirement.

On `native-release-Elj1GJ`,
all 24 fallback cases and all 22 native lifecycle cases pass with unified confirmation.
The original-source TUI adapter has no remaining code consumers and is removed after a full reference scan.
Its provenance retains the old path and names the prepared group as its current owner.
The configured MiMo replay `pi-hyper-rpc-allow-ZArrWh` completes all 16 original tool calls,
including both reported groups and intermediate process output,
with zero selectors and 17 actual Luna requests.
Positive-controlled payload inspection finds no raw structured command fields,
thinking,
or signatures.
The explicit prohibition replay `pi-hyper-rpc-allow-Pgbj60` withholds both process/Bash members,
opens zero selectors,
and receives three actual Luna responses.
An earlier attempt `pi-hyper-rpc-allow-sWaoxo` exits before any assessment:
the private diagnostic preload still searches the removed handler-local `retainAutomaticFailure`.
Updating that instrumentation to `prepared-group.mjs`'s `retainFailure` produces the passing replay;
the failed fixture remains untouched.

The new footprint measurement is 777,859,462 regular-file bytes,
including 506,254,358 bytes across 43 retained releases and 265,115,894 diagnostic bytes.
The active release is 8,223,549 regular-file bytes.
The complete emitted import graph now handles rewritten `.ts` provider paths,
finds no missing static targets,
and explicitly records all five computed import sites.
Unreached files are still not declared dead from that graph alone.
Ignored cleanup dry-run lists only the package cache,
dist,
and node_modules trees;
root stray Git directory entries do not exist although ignore rules cover their names.
Active Node command inspection finds Dsh,
Codex launchers,
the CPU monitor,
and the search-fetch MCP server;
the maintained Pi replay runs separately in its disposable fixture.

A provider-payload audit finds a separate missed shell boundary:
known process result metadata and `ad-process:notification` details retain raw `command` strings.
The new regression was committed red;
`review-process-result.ts` now projects the installed finite metadata shapes with the existing shell parser.
All 19 visible-context cases and the package type check pass after rebuilding.
Unrelated tool/custom metadata,
complete output text,
quoted literal data,
and original stored metadata remain intact.
The final installed payload audit remains required.

The process-result fix's configured replay `pi-hyper-rpc-allow-QaT10o`
transmits 17 assessment requests with no raw structured `command` fields.
The same audit detects nine fields in the older `jZsxRg` capture,
so its null result has a positive control.
Both reported groups execute without confirmation;
the separate intermediate `process.output` freshness failure still prevents a complete replay pass.
The error proves a different run-composition view,
not changed instruction text.
Current investigation distinguishes genuine run replacement from metadata-only publication changes.

The production main-group dispatcher now uses actual native preparation and execution phases.
Its three-member traces match immutable baseline `native-release-1voz8z`
across nine cases including cancellation and validation/hook failures.
`test:native:phases` covers real disposable SDK sessions and original native Bash definitions;
all eight initial cases pass,
including awaited updates,
serialized and parallel execution,
complete-group blocking,
and cancellation.
The existing 24-case fallback matrix also passes against `native-release-MDBPSM`.
The superseded executable/member/lifecycle/completion protocols and their exclusive tests are removed:
22 files and 3,093 physical lines.
Their four task entries are removed.
The rebuilt deletion candidate and its type check complete successfully.

Next actions:

- Finish the new notification-during-review cases and identify the exact run-view change.
- Demonstrate the native denial guard fails under a copied-build mutation.
- Consolidate the child dispatcher and manual/automatic execution state into the same private group record.
- Remove internal judgment/evidence publication protocols after their guarantees move to native interface tests.
- Emit only runtime contents in clean releases;
  measure implementation and build contents again.
- Complete package justification updates,
  troubleshooting evidence,
  rendered Markdown checks,
  configured-host replay,
  final lint/types/tests,
  and the corrected MiMo handoff.

The existing inventory labels roughly 66,000 lines as production source,
including comments and types;
its filename-based categories are not executable-line or runtime-reachability proof.
The measured `dist/final/node` tree is 376,443 apparent bytes.
Most of the remaining directory total is retained native releases and diagnostics.
Measure the clean artifact,
runtime dependency graph,
and actual maintained code separately before making another size claim.

- [ ] Measure executable source and statement counts across runtime,
  build transformations,
  tests,
  research,
  and unreachable code with a validated parser-based inventory.
- [ ] Apply the module deletion test to the request,
  evidence,
  lifetime,
  and prepared-group machinery;
  replace avoidable bookkeeping with deep modules whose interfaces express required behavior.
- [ ] Consolidate implementation and tests through those interfaces,
  preserving permission scope,
  original execution and cancellation,
  freshness,
  scoped confirmation,
  bypass,
  transport accounting,
  and configured-host behavior.
- [ ] Reduce clean build and runtime contents rather than relying on moving old artifacts.
- [ ] Rewrite each package-local justification around the final implementation,
  include its measured size and deletion rationale,
  and verify the redesigned installed consumer and complete diagnostic scope.

The retained `buildVisibleContext` cases were incorrectly removed with mixed legacy tests.
Commit `e611393f02` restores all 13 maintained cases in sibling test modules,
directly exercising the built projection through Pi's actual session projector.
Its task and type check pass;
two logger sink-verification timeout warnings during the first run remain visible in the captured stderr.
The native fallback matrix and all other targeted final tasks pass.
The first long replay after process-ID correlation reaches all 16 calls with no confirmation
and 17 successful Luna requests,
but its final `cat` fails because the proposed parallel setup races output-directory creation.
The next replay seeds that directory as a fixture precondition and must still run.

Completed source consolidation removes the obsolete extension pipeline,
exclusive tests,
heuristic catalogs,
unused model and signal types,
and the pass-through command-analysis wrapper.
The canonical native interface now supplies generated runtime and declaration bridges,
and the public entry reuses its native exports.
Package-local architecture documents justify host integration,
instruction collection,
operation preparation,
decision review and transport,
confirmation,
build lifecycle,
and diagnostics.
The previous README is retained as an explicitly historical package-local snapshot.

Commit `9f025a0b64` reuses byte-identical files only in a new verified release before publication.
It does not rewrite existing release paths.
The disposable publication regression fails on the expected inode equality before the fix,
then passes with distinct changed CLI files and a usable previous command.
The latest build reuses 1,476 files totaling 11,714,237 bytes.
Current installed release is `native-release-NteRIg`.
Its build reuses 1,479 identical files totaling 11,813,141 bytes.

The private scratch task `archive-historical-logs` completes in a two-CPU,
2 GiB container.
It archives 12,626 unchanged log files predating this boot,
totaling 7,064,174,193 original bytes,
into 194,896,137 compressed bytes outside the package.
The independent `archive-audit` rereads every gzip and verifies the source-path manifest's SHA-256 and length.
Its positive control rejects changed content of the same length in a valid gzip.
The reference scan checks all 1,670 tracked Markdown files and finds no archived filename references.
No original is removed before decompressed content verification and manifest append.
Root stray Git-directory entries are absent;
the scoped ignored-file dry run lists only `.cache`, `dist`, and `node_modules`.
No whole-directory cleanup is performed.

Remaining verification and documentation work:

- Measure package and archive sizes separately after the completed independent archive audit.
- Run types,
  complete maintained lint,
  bounded package units,
  native interfaces,
  projection,
  budget,
  preparation,
  confirmation,
  and fallback regressions after consolidation.
- Complete `replay-bash` through Pi's normal prompt entry.
  The rejected prefix-resume harness bypasses the required primary prompt observer and fails before assessment.
  Its failure is not a product regression.
  The replacement replays every original tool group from the normal prompt through each reported group,
  preserving preceding turns and configured resources.
  Its first run passes all 11 calls through the mixed process/Bash group,
  with 12 live Luna requests and no confirmation (`pi-hyper-rpc-allow-ZCKjZH`).
  The final replay through the later Bash group runs against `native-release-NteRIg`.
- Audit complete transmitted payloads and run live prohibited-operation controls.
- Prove the newly committed failure-category tests reject a copied classifier with its guard removed.
  Run all package units with disposable state and bounded resources.
  The first container cannot use the unavailable `cpuset` controller;
  the runner now uses `taskset` within the unchanged two-CPU and 2 GiB limits.
  The second launch exposes mise's basename-sensitive shim dispatch;
  its executable mount is corrected to `/fixture-bin/mise`.
- Update troubleshooting,
  package verification lessons,
  architecture evidence,
  and the MiMo handoff;
  render all changed Markdown before the final handoff.

### Completed advisory timing and operation projection on 2026-10-17

Session `2026-10-16T03-32-01-642Z_01a142c4-45e9-75fa-b579-ea10d43323ae`
reached `Original policy review and evidence closure failed` for tool calls
`chatcmpl-tool-a23317ad323e080b` and `chatcmpl-tool-9328d924e6057489`.
Review `01906150-53fc-4313-94ad-0c7ae40d26df` retains a nested `JudgmentDeadlineError`.
The first assessment request appears `9772` milliseconds after review start;
its response arrives `1050` milliseconds later.
The whole-operation request is recorded at timestamp `1792121942107`,
and its failure at `1792121942542`.
The preparation-inclusive budget begins before the review-start record.
The final request has no retained model answer.
This is a separate deadline incident, not evidence of a low-confidence final answer.
The exact log is retained privately under `${HOME}/temp/agent/pi-bash-bones-20261016/session.log.jsonl`.

The user additionally requires no model thinking in Luna inputs and parser-derived Bash structure only.
The reported installed projection retained thinking text and raw Bash strings;
the confirmation dialog also displays the unprojected parent response and its thinking signature.
The existing parser-first decision was therefore not fully implemented at the current wire boundary.

- [x] Remove thinking and signatures from review context and human confirmation summaries.
- [x] Parse Bash locally and send only structural operation data to Luna, preserving permission-relevant details.
- [x] Remove the hard fifteen-second cutoff, now explicitly a performance guideline.
- [x] Replay the complete reported interaction through the installed host.
- [x] Verify the installed consumer, prohibited-operation controls, and complete transmitted payloads.

Commits `77001d41e6`, `79b62c75a5`, `29b3361ffd`, `af5693baa7`, `baecc20430`, and `c718ae955a`
implement advisory timing, thinking exclusion, and the incumbent Bash parser's structural projection.
Built shell-projection, startup-relevance, budget, transport, decision-wire, dependency-facade,
and decision-fallback checks pass on `native-release-3tBgOk`.
The first full replay fixture `pi-hyper-rpc-allow-56qZR4` completed eleven tool calls
and fifteen successful Luna requests before the native process exited with status `137`.
No confirmation appeared before termination.
This incomplete replay does not establish success for the reported later group.
The retained stderr has no native fatal diagnostic; the termination cause remains unestablished.
The retry used a final rebuild and a resource-bounded sequential replay with retained termination evidence.
The bounded replay `pi-hyper-rpc-allow-oNjLac` exits normally and executes all 21 calls,
including both reported identifiers, without confirmation, with 25 successful Luna requests.
Its assertion that every command succeeds is invalid: the original transcript includes a failed search.
The replay also lacks `rg`, producing exit status `127` instead of the original search's exit status `1`.
The harness now mounts the actual `rg` executable and checks expected tool-error identities.
The corrected replay `pi-hyper-rpc-allow-Caf02F` completes all 21 original tool calls across 12 groups,
including both reported identifiers, without confirmation.
All 25 Luna requests succeed.
The original failed search remains an exit-status `1` result, with no missing-executable error.
The replay runs sequentially in a disposable two-CPU, 2 GiB container and retains configured resources.
The saved-payload audit checks all requests, including per-question operation copies,
prepared arguments, and selected historical messages.
It finds no assistant thinking, provider signatures, or raw Bash command fields in those model-input surfaces.
The installed prohibition fixture `pi-hyper-rpc-prohibited-nNA56F` blocks both original reads
without confirmation and completes three successful Luna requests.
Built context, shell projection, startup relevance, transport, decision-wire, budget,
constructor, preparation, and live/retained dependency checks pass.
The 22-case fallback matrix, shared shell-analyzer tests, both package type checks,
and focused production lint also pass.
Installed release is `native-release-b8o8BL`.
Verification artifacts are retained privately under `${HOME}/temp/agent/pi-bash-bones-20261016`.
This closes the reported failure and input-boundary corrections;
the first replay's exit-status `137` cause remains unestablished.
The user chose to keep `TCV` unchanged and document request-capture lessons in
`package/pi-plugin/auto-mode/native/review-input-verification.md`.
The MiMo lint handoff is refreshed for the verified release and current input and timing contracts.
The remaining handoff task is measuring and fixing the complete maintained lint queue while preserving that baseline.

### Completed permission scope, startup relevance, and resource measurement on 2026-10-16

The earlier installed release failed ordinary inspection across a real conversation.
The earlier passing initial-command replay did not establish that broader claim.
Session `2026-10-16T02-17-34-349Z_01a14280-1b8c-7186-9ebb-fc9c2e546e0c`
reached manual approval for the group inspecting task headings with `grep` and reading this document with `sed`.
Review `51396c5c-31bb-476b-bac9-ded1c23150be` admitted `RGP` and `CM3` as prohibitions at `0.64` and `0.70`.
Its whole-operation answer was `deny: 0.57`, `approve: 0.41`, `unknown: 0.02`;
the existing `0.60` cutoff correctly routed that uncertain answer to manual review.
This establishes a semantic-review failure, not a timeout.
The user explicitly excluded the concurrent sessionStorage warning from investigation.
Three identical live replays of the saved whole-operation request return the same `0.57` denial distribution.
The full request also contains Pi's instruction to use `read` instead of `cat` or `sed`.
That convention is relevant to the proposed command but is not itself an access restriction.
The user explicitly selected permission-boundary review:
tool-use conventions remain agent guidance while actual restrictions and missing consent remain enforced.
The accepted scope is recorded in `doc/decision/pi-auto-mode-semantic-admission.md`.
The user then corrected the source boundary:
include visible conversation, proposed operation, and readable project policy files;
exclude Pi's hidden instructions, tool metadata, and skill catalog.
The shared permission-scope question change is committed.
Collector selection now excludes hidden Pi inputs before clause indexing.
The user additionally required initialization-time Luna relevance selection and no complete `AGENTS.md` payload.
The existing `native/contract/lifecycle/rule-relevance-cache` supplies original-source lifetime,
incremental rule hashing, persistence, and concurrency checks.
The initial inspection found that its requests included `fullPolicy`, production bootstrap did not configure it,
and its returned view deliberately admitted no exclusions.
Commits `d9e9771bd8`, `c35670685a`, and `22532b415e` connect that owner to startup and reload,
filter original clause handles, and transmit selected source excerpts instead of complete policy documents.
The cache continues to publish estimates without permission;
the decision consumer applies the user's selected relevance cutoff.
The earlier adopted direction refreshes only changed or missing rules after startup cache lookup.
The user subsequently required uncertain rules to be culled too.
The current selection retains only tagged rules scoring at least `0.60` for potential permission relevance.
The action-approval cutoff remains `0.60` independently.
The first candidate build completed, but type checking found missing declarations and an optional-array type mismatch.
Those were corrected; type checking and focused lint subsequently passed.
Built regressions verify the `0.60` relevance boundary, unchanged-cache reuse, changed-rule refresh,
hidden-input exclusion, and full-policy-copy omission from transcript material.
The startup-failure matrix verifies ordinary chat, scoped Allow and Stop, and headless withholding.
The installed live replay in `pi-hyper-rpc-allow-PiFtXg` executed all nine original tool calls across six groups,
including the reported `grep` and `sed` group, without a confirmation dialog.
It made thirteen successful Luna calls, including startup classification of 211 tagged rules;
25 rules remained selected.
An earlier concurrent-verification run hit an internal review failure after automatic approval of its third group.
That failed fixture did not retain the terminal exception before shutdown;
do not attribute its cause to the unrelated sessionStorage warning or to model uncertainty.
The replay harness now retains stderr and events on failure.
Commit `747bca5544` also moves excerpt indexing to resource loading so source captures reuse the same original spans.
Additional readable policy files now use the incumbent file observer for identity and freshness;
callback-only content does not qualify as a readable file.
Final startup and readable-context regressions pass on installed `native-release-8uGKtw`.
The relevance regression fails with culling removed in a disposable build and passes when restored.
The live prohibition control `pi-hyper-rpc-prohibited-aar6Xn` blocks both reads without confirmation.
All 22 decision-fallback cases pass after the final source selection changes.
The approved `VUB` update now requires the reported interaction's preceding turns,
retaining configured extensions, skills, and tool catalogs.
Type checking and focused lint pass.
The package README and Luna troubleshooting entry now describe the implemented input boundary.

- [x] Restrict review sources to visible conversation, operation, and readable policy files.
- [x] Wire startup relevance refresh into the installed host; remove complete-document classifier payloads.
- [x] Feed selected original rules into per-operation questions and shared context, preserving source checks.
- [x] Reproduce the full reported conversation with live Luna and verify explicit prohibitions and consent controls.
- [x] Measure local CPU, memory, writes, and model usage in repeated guarded and bypass replays.

The exact decision log and extracted operation assessment are retained under
`${HOME}/temp/agent/pi-manual-fallback-20261016`.
The installed correction is complete for the reported interaction and named regression controls.
All six resource replays execute the original nine calls across six groups without confirmation.
Guarded fixtures are `pi-hyper-rpc-allow-zqgI2z`, `pi-hyper-rpc-allow-6Eb1t7`,
and `pi-hyper-rpc-allow-SMu1kz`.
Resource replays run sequentially inside a two-CPU, 2 GiB Podman container with disposable state.
Both arms retain cold startup classification; bypass removes operation assessment only.
Primary-model replies are the original captured tool-call sequence; Luna remains live.
Three bypass runs establish the observed baseline spread before three guarded runs.

- Bypass: CPU `82.16` to `83.96` seconds, median `82.81`;
  elapsed `79.68` to `81.95` seconds;
  peak RSS `1774.59` to `1799.59` MiB, median `1790.12`;
  writes `7127040` to `7315456` bytes.
- Guarded: CPU `120.90` to `123.56` seconds, median `122.18`;
  elapsed `120.30` to `121.49` seconds;
  peak RSS `1930.80` to `1974.50` MiB, median `1935.27`;
  writes `55607296` to `55840768` bytes.
- Cold startup: two Luna calls, 211 questions, `78404` input tokens, and `271786` request bytes in every run.
- Guarded totals: thirteen calls, 352 questions, `400547` to `400561` input tokens,
  and `1900253` to `1900289` request bytes.
  The increased request count is the positive control that bypass actually skips operation assessment.

These are whole-Pi-process measurements, excluding child processes and the host-side provider bridge.
Elapsed time includes the harness's ten-second startup wait.
Luna inference runs remotely; primary-model generation was scripted and is excluded.
The replay still reads the shared repository, whose Git metadata and handover text changed during the run series.
Consequently these are observed repeated workloads, not a byte-identical causal estimate of approval-only overhead.
The CPU ranges are separated, but the exact difference must not be presented as isolated production overhead.
Memory approaches the fixture's imposed limit; the cold replay is not a steady-state desktop-memory measurement.
No power or battery measurement was made.
The private `resource-matrix` and `resource-summary` tasks retain per-run receipts and `resource-summary.json`.
The initial harness failed before Pi started because nested Bubblewrap lacked a resolver-file source
and could not mount `/proc`.
Explicitly binding `/etc/resolv.conf`, using Podman's documented `unmask=/proc/*`,
and mounting the resolved fixture path made the first replay pass.
The original real-session memory sample and this bounded cold replay measure different workloads.
The final type check, focused lint, startup selection, readable-context, omission control,
decision-fallback matrix, live prohibited-read control, and full conversation replays pass.
CommonMark rendering was inspected for the updated README and troubleshooting entry.
New Pi processes use the installed release; already-running processes retain their loaded code.
The earlier readiness claim is retracted beyond its tested commands.

The user also requested local resource drain.
Read-only `/proc` samples of live Pi PID `2545748` at uptime `193399.71` and `193421.52` seconds
recorded `5022` and `5473` CPU ticks at `100` ticks per second:
`4.51` CPU seconds over `21.81` elapsed seconds, or `20.7%` of one core.
The later `ps` RSS was `639128` KiB, approximately `624` MiB;
the earlier RSS was `690656` KiB.
OS write counters increased from `61661184` to `62574592` bytes.
These measurements cover the whole Pi process, exclude child processes,
and do not isolate approval overhead or measure power consumption.
The process exited before further sampling.

The earlier disposable two-command replay `pi-hyper-rpc-allow-JPMcvY`
reported `771295` input tokens, zero cached tokens, `518` questions,
and `3958323` request bytes across four Luna calls.
Individual calls took `3418.46`, `2670.05`, `2032.04`, and `683.05` milliseconds;
these overlap and must not be summed as elapsed latency.
Its log directory occupied `28163758` apparent bytes including directory metadata.
These are one-run observations, not stable averages or isolated approval CPU measurements.

### Completed false prohibition and log corruption correction on 2026-10-16

The MiMo session `2026-10-16T01-17-32-548Z` rejected a group containing repository inspection commands.
Its decision evidence identifies `DCB` as the blocking clause at prohibition scores `0.62` and `0.63`.
The compiler admitted the first source at priority `2`;
the router promoted that estimate to an established prohibition and skipped whole-operation review.
This is a separate failure from the completed false-prerequisite correction.
The user confirmed that uncertain clause denials must reach scoped Allow/Stop confirmation
when both automatic reviews fail,
and requested fixes for every issue shown.
The user reiterated that accommodating Luna's interpretation failures is part of the product requirement.

- [x] Reproduce and correct clause-denial routing, retaining whole-operation denials and original ownership checks.
- [x] Measure the actual DCB clause interpretation against inspection and restricted-operation controls;
      correct the assessment path rather than treating the false classification as an acceptable vendor quirk.
- [x] Reproduce and fix concurrent JSONL record corruption;
      the incident log contains two malformed physical lines with interleaved record content.
- [x] Verify the installed host with the reported commands and record the final evidence.
- [x] Publish logger `0.4.1` through the automatic release workflows and verify the registry artifact,
      as requested by the user.

Incident log: `node_modules/.monochromatic/2026-10-16T01-17-31.942Z.log.jsonl`.
Private inspection task: `${HOME}/temp/agent/pi-host-isolation-20261015`,
`mise run inspect-prohibition-incident`.
It recovered `1046` valid decision/assessment records and counted malformed lines without printing their content.
The authored false-prohibition regression failed with zero executions instead of two before the routing fix.
The corrected native matrix passes final approve/deny,
unknown,
transport failure,
Stop,
and headless cases after a first-pass prohibition.
The file-sink regression also failed before serialization and passes after the fix.
Logger's complete unit suite and type check pass;
package lint reports zero warnings and errors across `58` files.
The user requested the version bump for automatic publication.
The unchanged live DCB replay repeated prohibition scores `0.62` and `0.63` across three runs.
Reducing the batch to the DCB questions left those answers unchanged.
Local choice rubrics changed the choices to `none` but did not reach admission.
Full local interpretation instructions distinguishing rule examples from proposed invocations
produced six `none` answers above `0.60` in the next repeated replay.
The candidate retains complete shared context and now also supplies local instructions and rubrics.
Broader semantic controls still exposed uncertain first-pass answers for printed command data,
reading under a modification restriction,
and an inapplicable approval requirement.
Those controls have not passed the predeclared first-pass cutoff;
do not describe the prompt change as universal semantic qualification.
The installed recovery matrix now verifies reconsideration after an admitted first-pass denial,
including uncertainty and failure in the whole-operation decision.
The live installed replay also passes;
this establishes recovery and the reported operation boundary,
not universal first-pass semantic accuracy across the broader controls.

Logger release commit `f6c406c1c2` bumps `0.4.0` to `0.4.1`.
The initial npm run `38835764830` failed startup because the Actions allowlist excluded Changesets.
The user added `changesets/*` at the organization;
the repository API confirms it and SHA pinning remains required.
Restarted npm run `38836163322` built and packed the package,
then failed publishing with `ERR_PNPM_FAILED_TO_PUBLISH`, HTTP `404 Not Found`.
Its npm trusted-publisher configuration still needs inspection.
Resolved after the release requests:
`npm-publish.yml` run `38838431661` published logger `0.4.1`,
with registry provenance and a fresh consumer confirming complete ordered large JSONL records.
The full result is recorded in [the logger release plan](../planning/module-logger-release.md).

The subsequent readiness check found `bin/pi` still loading `native-release-Jg5gMF`.
The installed first-pass prohibition regression failed at its exact-clause diagnostic assertion;
the current source and installed decision router differ.
The live installed replay did execute both reported commands without a dialog in fixture
`pi-hyper-rpc-allow-kdJq9p`,
with four successful Luna requests.
That passing sample does not exercise recovery from an admitted false prohibition.
The candidate's corresponding matrix and repeated live command replays passed before publication work.
The full package build then staged and activated `native-release-ikWviS`.
All 22 installed `test:native:decision-fallback` cases pass,
including false-prohibition approval and denial,
uncertainty,
transport failure,
Allow/Stop,
headless withholding,
source changes,
and cancellation.
The live installed replay `pi-hyper-rpc-allow-JPMcvY` executes both reported Bash commands,
makes four successful Luna requests,
and opens no dialog.
Its persisted log check parses 142509 physical JSONL lines with zero malformed lines,
including 1043 correlated diagnostic records,
two complete assessment requests and responses,
and 518 question-evidence records.
The live prohibition control `pi-hyper-rpc-prohibited-adEoOO` blocks both reads,
makes two successful Luna requests,
and opens no dialog.
New Pi processes load the corrected runtime;
already-running processes retain their loaded modules.
Receipts:
`functional-installed-20261016.log`,
`functional-installed-prohibition-regression-20261016.log`,
`functional-native-build-20261016.log`,
`functional-installed-matrix-20261016.log`,
`functional-installed-live-final-20261016.log`,
`functional-installed-log-integrity-20261016.log`,
and `functional-installed-prohibit-final-20261016.log`
under `${HOME}/temp/agent/pi-host-isolation-20261015`.

The private registry's `401 Unauthorized` occurred after repository transfer:
GitHub reports repository ID `710240121`, owner `Aquaticats`, owner ID `343218767`.
The user approved updating the corresponding trust identity.
Commits `39fc0da926` and `15e8a365d1` update the generator and regenerated config;
only subject, owner ID, and workflow reference changed.
Private publish run `38836247574` still returned `401`.
The configured deployment's adoption of the new identity is unverified.
The existing registry runbook records withheld host SSH and Coolify API access;
honor that boundary when checking deployment.
The npm publication and installed-host checks are now complete.
The private registry deployment remains a separate unresolved boundary;
ask for its remaining deployment action only if the authorized bridges cannot perform it.

### Decision-model fallback correction on 2026-10-16

The user reported that the installed release blocked repository inspection commands with
`An independent authorization prerequisite remains unsatisfied or unresolved; session confirmation cannot satisfy it`.
The affected commands list `doc/handover/` and read the repository's working-directory identity.
The session beginning `2026-10-16T00-30-17-591Z` retains both rejected calls.
The earlier readiness claim was withdrawn for this boundary.
The previous live direct-Bash fixture used a synthetic working-directory policy;
it did not establish correct decisions under the real repository's complete policy.

The user clarified that fallback means the decision model must reconsider the complete operation.
Only when automatic decision paths fail should the host open a dialog.
The agent's intervening interpretation, sending prerequisites directly to manual approval, was rejected.
Use the already selected Luna decision route; do not introduce another vendor or weaken categorical thresholds.

- [x] Replay the reported commands with actual repository policy and record the original evidence-retention limit.
- [x] Add decision-model fallback before dialog selection when rule assessment cannot resolve the operation.
- [x] Verify permit, deny, unavailable-model, uncertain-model, cancellation, and complete-context boundaries.
- [x] Retain correlated clause/source references, model distributions, admission and fallback decisions in local logs;
      reconstruct a rejected review from those logs in a built-consumer regression.
- [x] Activate the verified runtime and update decisions, troubleshooting, and the MiMo handoff.

The user identified the absent per-clause distributions as insufficient logging on 2026-10-16.
Logging repair is part of this task, not a reason to leave future incidents unattributable.
The user accepted tightening the existing `LOG` rule to require reconstructable decision evidence.
Commit `9d31943c2b` also routes recoverable initial clause-assessment failures through the decision model,
while retaining the original currentness checks and deadline.

The maintained regression `test:native:decision-fallback` first failed with one assessment instead of two.
Initial candidate `native-release-duzo2x` passed that regression:
an authored false prerequisite is reconsidered by a distinct whole-operation model question,
and its approve result executes both direct Bash calls without an approval host.
The new question retains original claim identity,
complete prepared operations,
selected conversation and policy,
and the existing judgment's transport budget.

That initial candidate also passed the live replay of the user's exact `ls` and `git rev-parse` commands
with the actual repository working directory and complete policy.
It made four successful Luna HTTP calls,
including the whole-operation fallback,
executed both commands,
and opened no dialog.
Receipt: `${HOME}/temp/agent/pi-host-isolation-20261015/repo-decision-candidate.log`.
It was not installed; the final release incorporates the subsequent failure-path and diagnostic corrections.

The first unchanged installed replay reached a dialog instead of reproducing the original prerequisite block.
It estimated `PX2` as a prerequisite at `0.39`, below admission,
even though that rule's destructive/external condition does not fit the proposed inspection.
The original session logs do not retain its per-clause distributions,
so this does not establish which clause caused the historical false positive.
The maintained regression supplies the false-prerequisite estimate explicitly to verify routing deterministically.

The current built matrix passes automatic approve and deny,
unknown and lower-scoring answers,
initial and final transport failure,
malformed replies,
manual Allow and Stop,
dismissal,
headless withholding,
source changes,
cancellation,
and established prohibition.
Receipt: `decision-matrix-2.log` under the same scratch parent.
Its final pure-review assertion needed correction because incumbent-channel diagnostics retain
`requires-approval` while unsupported channels retain `unresolved` after scoped Allow.
The deadline extension then exposed a separate distinction:
assessment-budget expiry must allow unchanged operations to reach manual review,
while source changes must not.
Commit `d84594b527` makes the source-observer check handle that distinction.
Final deadline and fresh-confirmation cases pass in `decision-matrix-4.log`.

Local assessment logs now retain complete selected inputs, replies, and malformed responses.
Decision logs retain source bindings, answer distributions, admission, fallback errors, and final outcomes.
`operationId` joins model exchanges to the `reviewId` used throughout the original judgment.
The new regression reconstructs its false prerequisite from these logger records.
The first focused lint pass found four errors;
callback shape, parameter documentation, and the fixture's asynchronous abort type were corrected.
Final candidate checks now pass:

- `decision-matrix-4.log`: every fallback case, original-deadline expiry, explicit fresh confirmation,
  and the pure review-outcome assertions.
- `decision-approval-matrix-2.log`: original approval, native contexts, reload, fresh confirmation,
  uncertainty, bypass, and selected conversation regressions.
- `decision-manual-matrix-2.log`: unavailable observation and failure of both assessment paths,
  including Allow, Stop, cancellation, and no UI.
- `decision-fallback-types-7.log` and `decision-fallback-build-js-final.log`: successful type checking and bundle build.
- `decision-fallback-lint-2.log`: 77 changed files, 3771 warnings, zero errors.
  Warnings remain in the separate MiMo cleanup queue; this is not complete-package lint clearance.
- Built `test:decision-wire`, `test:semantic-operating-policy`, and `test:assessment-bundles` pass.

Final live candidate `native-release-g2QlU8` repeats the exact reported commands under actual repository policy
with all configured metadata and opens no dialog.
Fixture `pi-hyper-rpc-allow-5yMBCp` made four successful Luna HTTP calls and executed both commands.
`repo-decision-candidate-final.log` retains the consumer result.
`decision-log-file-proof.log` reconstructs its decision from the actual JSONL files:
1043 correlated records, 518 question answers, 515 clause admissions,
both complete assessment requests and replies, and the final approval without confirmation.
Both `PX2` occurrences remain below admission in this new observation;
this still does not identify the original incident's missing clause evidence.

The installed runtime is now `native-release-Jg5gMF`.
`build:native` staged and activated the release after validating the CLI.
Installed `bin/pi` passes the same repository-policy replay in fixture `pi-hyper-rpc-allow-HTVK1s`:
four successful Luna calls, both original commands executed, and no dialog.
Its actual JSONL files retain the same complete diagnostic counts and final approval;
`decision-log-installed-proof.log` verifies the persisted records.
The installed explicit-prohibition control `pi-hyper-rpc-prohibited-a1TH2T` makes one successful Luna call,
blocks both proposed reads, and opens no dialog.
Receipts: `repo-decision-installed-final.log` and `repo-decision-installed-prohibit.log`.

Installed fallback, budget, instruction identity/lifetime/batch, public API, and publication tests pass.
The broader instruction task exposed stale tests for the 256-clause and 64-heading ceilings removed in
`6bd25e4227`.
Their replacement checks complete membership across those former boundaries and retains empty-subject rejection.
`decision-fallback-installed-ownership-2.log` passes;
`decision-fallback-types-8.log` passes;
`decision-instruction-test-lint.log` reports zero errors and warnings.
The other production files retain the zero-error focused check already recorded.

The approved `LOG` rule and generated `CLAUDE.md` are updated.
Decisions, README files, troubleshooting, and the MiMo handoff describe model-before-dialog behavior.
Markdown renderer checks pass.
New Pi processes load the installed release;
already running processes retain their loaded modules.
This correction is complete; the separate lint-cleanup handoff remains the next named work item.

### MiMo first-group correction and direct Bash review

The real MiMo session at 23:29:17 UTC reached manual fallback on its first group of two Bash calls.
The displayed error was `Original policy review and evidence closure failed`.
The session records the user's Stop response and two aborted tool results.
The prior installed controls covered authored codemode responses and did not establish this boundary.
That handoff-ready conclusion was withdrawn until the configured-catalog failure was reproduced and corrected.

The configured replay now reproduces the exact aggregate with a settled catalog of 40 MCP tools.
The nested cause is `DependencyDeadlineError: Original dependency assessment deadline expired`.
Its stack reaches `bindRunMessageSources`, which revalidates the complete parent once per retained input.
The original incident logged 53,108 resource snapshot attempts during input capture.
The old configured harness disabled MCP and omitted home-discovered Depot skills.
The corrected fixture retains those inputs; metadata is replayed locally and commands receive scripted Stop.
Receipt: `${HOME}/temp/agent/pi-host-isolation-20261015/mimo-mcp-forty-settled.log`.
The diagnostic fixture is `pi-hyper-rpc-stop-RhgubS` under the same scratch parent.
The original session did not retain its nested exception, so its exact cause remains inferred from this reproduction.
The separate 140-tool cached-catalog probe exhausted its 512 MiB child heap before review;
the 20-tool immediate-start probe observed a catalog change during confirmation.
Neither is substituted for the reproduced aggregate.

- [x] Recover a nested failure reproducing the exact two-call shape in a disposable host.
- [x] Correct the demonstrated cause and verify the real configured consumer boundary.
- [x] Retain regression evidence and update the MiMo handoff readiness.

The candidate fix `5a8760d15f` passes the unchanged 40-tool replay and reduces binding parent checks
from 257 to 2 for 256 retained inputs.
The stale-parent and changed-during-construction controls pass;
omitting the final parent check makes the adversarial test fail.
The full cached catalog (66 BrowserOS tools and 74 Linear tools) exposed a separate request-preparation ceiling:
`Original request tool schemas exceed the 10000-property profile`.
This happens before the provider request and is not the original review-deadline exception.
Commit `ac68881314` removes that ceiling while preserving descriptor freshness and structural rejection.
The maintained test retains 12,000 properties and rejects changed properties, accessors, and unexpected prototypes.
The intermediate failure receipt is `${HOME}/temp/agent/pi-host-isolation-20261015/mimo-full-candidate.log`.

The user explicitly requires direct Bash calls to receive automatic review as well.
The complete-operation review boundary now includes direct Bash groups,
retaining uncertainty fallback and governing prohibitions.

Commit `3827af2da9` replaces the repeated retained-origin search with a source-identity index.
The full cached catalog generated 83,404 message inputs and 83,399 retained origins in the diagnostic fixture.
The old search took 113190.85 ms in that run;
the new linear-read regression and full 140-tool replay pass.
The full-catalog receipt is `mimo-full-index-after.log`, fixture `pi-hyper-rpc-stop-fsbmtV`.

Commit `427ae71387` adds direct Bash to automatic semantic review;
`d4467e1fc3` publishes its scope matcher through the native generated adapter.
Controlled built-consumer cases pass for allowed direct commands,
prohibited commands,
unknown model answers without an approval host,
and direct Bash mixed with a supported codemode read.
Each case sends an assessment containing both complete operations.
Receipt: `direct-bash-after-3.log` under `${HOME}/temp/agent/pi-host-isolation-20261015/`.

Live Luna has also received the direct Bash group through the configured host with all 140 cached MCP tools.
Fixture `pi-hyper-rpc-allow-DWhbjc` received HTTP 200 for 94 questions.
The permission clause's selected probability was 0.75;
other clauses remained uncertain, so the original confirmation dialog appeared.
The scripted Stop executed neither command.
This proves live model dispatch and uncertainty fallback, not automatic approval of the complete live context.
The live candidate prohibition fixture `pi-hyper-rpc-prohibited-GaSZbp` received HTTP 200,
opened no dialog,
and blocked both commands.
Candidate fixture `pi-hyper-rpc-allow-kBHlyH` passed scripted scoped confirmation and executed both commands.

The corrected runtime is installed as `native-release-tmMxY3`.
The installed `bin/pi` consumer repeats the complete configured-host fixture,
including 17 configured packages,
27 home-discovered skills,
and all 140 cached MCP tools.
Fixture `pi-hyper-rpc-allow-LuIIFZ` sent 94 questions to live Luna,
received HTTP 200,
and executed both native Bash commands after one scripted Allow.
This is automatic review with uncertainty fallback, not a claim that the model automatically approved the group.
The installed prohibition fixture `pi-hyper-rpc-prohibited-QwVjXt` also received HTTP 200 for 94 questions,
blocked both commands,
and opened no confirmation dialog.
New Pi processes load the new release; existing processes retain their loaded release.

Final built policy and scope units pass,
as do package type checking and installed direct Bash,
message freshness,
schema capacity,
source-freshness work,
bypass,
and existing native program permission/prohibition/unspecified controls.
Removing the final parent check still makes the adversarial freshness regression fail.
The focused lint measured 635 warnings and one error across 14 changed files;
the sole error was a rest-parameter test wrapper.
After replacing it with the native callback's explicit arguments,
that file reports 95 warnings and zero errors.
The unchanged files had no errors in the first pass.
These are focused checks and do not claim the package is lint-clean.
Receipts live under `${HOME}/temp/agent/pi-host-isolation-20261015/`:
`direct-final-semantic.log`, `direct-final-types.log`, `direct-installed-regressions.log`,
`direct-installed-additional.log`, `mimo-omission-final.log`, `direct-final-lint.log`,
`direct-test-final-lint.log`, `direct-installed-allow.log`, and `direct-installed-prohibit.log`.
The warning queue is ready for the [MiMo cleanup prompt](pi-auto-mode-mimo-lint-prompt.md).

The user accepted the VUB clarification requiring configured extensions, skills, and tool catalogs in consumer checks.
`AGENTS.md` is updated and `CLAUDE.md` regenerated through file-enforcer.
The revised rule has 197 normalized characters and 26 words.
Pandoc CommonMark rendering and inspection pass for the rule sources,
package READMEs,
troubleshooting record,
and MiMo handoff documents.

### Host-isolation replacement complete

The host-isolation release was installed as `native-release-Yhc7sX` and verified through the original `pi` command.
It is superseded by `native-release-tmMxY3` for new processes on 2026-10-16.
New Pi processes load it; existing processes retain their loaded release.
All functional work in this section is complete.
The remaining warning cleanup belongs to the requested [MiMo handoff](pi-auto-mode-mimo-lint-prompt.md).

Final package type checking passes.
Focused Oxlint on 40 changed files reports zero errors and 2709 warnings;
its nonzero exit is the warning gate, not a clean-lint result or whole-package measurement.
The candidate live controls pass scoped Allow, prohibition, and an independent external-office prerequisite.
Installed-command controls pass with all 17 configured packages, followed by live Luna Allow verification.
That live installed check uses three successful HTTP requests for 516 questions and executes one native read.
Primary responses and dialog selection are authored fixtures; no physical human input is claimed.
Receipts are `live-final.log`, `installed-configured.log`, and `installed-live.log`
under `${HOME}/temp/agent/pi-host-isolation-20261015/`.

The user corrected the fallback on 2026-10-15:
unavailable observation or assessment evidence must fall back to manual approval,
not withhold the operation outright.
Implementation and failure-isolation tests must exercise Allow, Stop, and cancellation.
The implemented fallback owns manual approval independently of the unavailable observation,
while retaining exact prepared-group identity and native cancellation.
Primary prompt, event, and transport capture now retain failures without rejecting native chat.

The disposable release `native-release-Jx9I6F` verifies manual fallback after missing observation
and failed assessment: Allow executes one native child; Stop, cancellation, and headless sessions execute none.
The maintained fixture is `src/native-observation-isolation.test.mjs`.
The subsequent disposable release `native-release-S8Ip9X` keeps prompt and event capture passive.
Its navigation, cancellation, reload, fork, and resume controls pass.
The retry fixture initially set overrides before native resource loading replaced them;
setting and asserting overrides on the constructed session verifies both agent and provider retries.
The native prompt callback fixture also passes capture failures before and after callbacks,
exactly-once invocation, and preservation of original thrown values.
Package type checking passes.
An intermediate focused lint check on 36 changed files reports 2247 warnings and zero errors;
warnings still fail the lint command and remain assigned to the requested MiMo handoff.
The combined session, passive-prompt, lifecycle, runtime-hook, policy, and manual-fallback matrix passes
on disposable release `native-release-JWvYLT`.
The approval regression then caught a second-dialog bug after established authorization became stale.
Commit `e9864891ee` prevents an established decision or its approval from becoming evidence-free fallback.
Release `native-release-LNNqh3` passes the corrected original-approval, native-context,
fresh-confirmation, and uncertain-review controls.
The out-of-profile bypass fixture requires the new manual fallback after disabling bypass;
its scripted Stop response is now explicit instead of accidentally approving the fallback.
Final original-approval, bypass, and selected-context controls pass in `approval-final.log`.
The configured-host candidate checks subsequently pass with all 17 packages in `configured-final.log`.
Live verification and commits were interrupted by the shared worker's 256-task limit:
Mise reports failed thread creation and Git reports failed forks and threaded `lstat`.
An unrelated portfolio Chromium preview consumes most slots.
The user instead authorized increasing worker capacity on 2026-10-15.
The hook requires the exact task limit, so raising systemd alone blocks every shell command.
The kernel value was restored through the file-edit tool to recover shell access.
The user then specified 2048 tasks.
Commit `561ac46f20` updates the maintained hook; its Rust tests and Clippy pass.
The installed hook, persistent slice definition, runtime parent, and existing `codex` child now agree on 2048 tasks.
The 4 GiB memory cap, zero swap, and two-CPU quota remain unchanged; the preview stays running.
Ordinary shell commands through the updated hook succeed.
The exact manual-group identity/lifetime fixture also passes in `manual-group.log`.
The original installed manual-group test passes in `installed-manual-group.log`.
Removing only the parent-identity guard in a process-local disposable probe makes the same test fail
with `changed-parent must reject its stale Allow response`; production keeps that guard.
Use the candidate's canonical path and preload its own HTTP dispatcher in the configured fixture.
Scratch multi-task invocations without separators passed later names as arguments;
only explicit per-task output counts as execution evidence.
The combined scratch task now sequences independent `mise run` calls.

After the replacement handoff, the user reported `Root lease has been retired` and `Connection error.`.
The real session beginning at 22:17:29 UTC records connection failures on `llmgateway-devpass/mimo-v2.6-pro`,
followed by a lease error on `opencode-go/mimo-v2.6-pro`.
The user confirms using Esc, editing/retrying, `/tree`, and `/reload` between attempts.
These are separate incidents until reproduced at their respective consumer boundaries.

Both failures now reproduce independently with no tool calls.
Provider switching fails the observer's fixed startup-endpoint comparison;
tree navigation retires the root captured by the next ordinary prompt.
Retry, reload, and abort-after-completion controls pass.
The user then identified the repeated failures as an architectural problem.
The user accepted replacement of the integration rather than targeted endpoint/navigation repairs.
Implementation follows the
[host-isolation decision](../decision/pi-auto-mode-host-isolation.md)
and its [consumer matrix](../planning/pi-auto-mode-host-isolation.md).
The [incident record](../troubleshooting/pi-auto-mode-session-interference.md)
contains the exact emitters, reproductions, and limitations.

- [x] Reproduce and fix the connection failure after selecting another provider.
- [x] Reproduce and fix retired-root failure after native navigation and retry.
- [x] Resolve the host-isolation design choice and update the accepted integration scope.
- [x] Move primary observation failures to tool admission and use operation-scoped native currentness.
- [x] Verify original host operations, install the corrected artifact, and document the evidence.

Scratch reproductions live under `${HOME}/temp/agent/pi-reported-errors-20261015/`.
The prior successful controls did not cover this complete UI-operation sequence.
Warning cleanup remains assigned to the MiMo handoff after these functional regressions are resolved.

The implementation now removes judgment creation from `message_end` and starts it at complete-group dispatch.
Native branch readers supply operation-local currentness; constructor ownership remains separate.
Primary observation retains failures for tool admission and preserves native transport retries and callbacks.
The startup endpoint is no longer an admission condition.
Build and consumer checks first ran in a disposable package before the verified release was activated.
The expanded native fixture exposed general event capture rejecting a native `AbortSignal`.
That failure is now retained only for automatic evidence; native callbacks and subsequent chat complete.
The source owner can remain unavailable after such a failure, so affected tools require manual review.
The tests do not claim that automatic evidence recovers after every unsupported event.
The completed verification retains this automatic-evidence limitation explicitly.

### Replacement verification before the new report

The user subsequently requested a whole-system re-review and consideration of rebuilding.
The review is complete.
The user accepted decision-core replacement and requested avoiding overengineering.
The [accepted decision](../decision/pi-auto-mode-decision-core.md) fixes implementation scope.

- [x] Select current context before semantic candidates and exclude retained archives from model input.
- [x] Replace decision composition and re-resolve scoped confirmation without erasing independent prerequisites.
- [x] Separate explicit bypass from semantic profile admission while retaining native execution checks.
- [x] Verify prepared-operation identity, freshness, cancellation, deadlines, and configured-host behavior.
- [x] Verify activation preserves a usable artifact after build failure and record remaining feature scope.
- [x] Remove superseded paths, run relevant lint and tests, and update consumer documentation.

Functional replacement and consumer verification are complete.
Final package type checking passes.
Focused Oxlint covers all 40 changed TypeScript and JavaScript files and reports zero errors and 2149 warnings.
The lint command still exits nonzero for warnings;
this is not a clean-lint or whole-package warning-count claim.
Per the user's instruction, this agent stops warning cleanup once Pi is functional.
The remaining queue belongs to the
[MiMo v2.6 Pro lint prompt](pi-auto-mode-mimo-lint-prompt.md):
measure the full maintained scope, then fix and verify its remaining warnings.
The installed command points to the verified replacement release;
already-running Pi processes retain their previously loaded release until restarted.

Release activation now stages an immutable native directory and its matching policy bundle,
loads the generated CLI, then publishes the stable command link.
`test:native:build-activation` passes generation and CLI-loading failures in a disposable installation,
with successful directory migration and subsequent link replacement as positive controls.
The configured-host fixture passes Allow, unrelated-prerequisite, and prohibition controls
with all 17 configured packages and copied workspace instructions.
Its current assessment contains 516 questions in three requests of 200, 200, and 116.
The authored-response controls are followed by live Luna receipts in
`pi-core-live-configured-corrected.log` and `pi-core-live-restrictions.log`
under `${HOME}/temp/agent/`.
Live Allow executes one original native read;
an actual prohibition and an external-office approval prerequisite each withhold execution without a dialog.
All use the original installed CLI, disposable homes, all configured packages,
copied workspace instructions, and authored primary responses.
They do not claim human approval or physical keyboard delivery.
The direct-import regression found missing public exports for shared context and native bypass;
both package entries now retain those identities and the direct-import checks pass.

The first selected-context live run falsely admitted `PX2` as an independent prerequisite for a local read.
Both unchanged replays reproduced it.
Repeating the complete operation beside each clause removed that admitted prerequisite
in both diagnostic observations, without changing policy or the `0.60` cutoff.
Commit `67a2d7cfe3` implements that representation and removes a redundant source read.
The subsequent live host checks pass;
the permission case sends 516 questions in three calls and its slowest response took 5760.916 milliseconds.
The [Luna troubleshooting record](../troubleshooting/llmgateway-luna-admission.md)
retains the original failure, controls, limitations, and exact receipts.

The broader regression run passes type checking, import boundaries, budget and cancellation,
dependency freshness, original source identity, input sealing, prepared membership,
approval routing, uncertainty handling, semantic transport, bypass, wire conversion, and bundle accounting.
Its sole failure was the redundant source read's changed callback sequence;
the corrected preparation, failure, and boundary consumers pass in `pi-core-final-checks.log`.
That final run also passes release activation, selected context, transport, and semantic-policy consumers.
Removing CLI preflight only in a disposable fixture makes the activation regression fail with
`The deliberately invalid native release was activated`.
The production guard stays present.

Remaining feature scope is explicit:
guarded execution retains the accepted single-call and producer/successor program profiles;
explicit bypass also runs ordinary native programs outside those semantic profiles.
Legacy `/guard`, `propose_trust`, and approval reuse remain legacy-only features.
No full-session memory benchmark or universal semantic accuracy claim follows from this replacement.

The replacement is `native/contract/integration/action-policy/decision-core.mjs`.
It replaces `evaluate.mjs`, `index.mjs`, and `reviewer.mjs` on 2026-10-15;
historical references to those files describe the reviewed implementation.
The original judgment still owns preparation-inclusive accounting, evidence, closure, and execution.
Scoped confirmation re-resolves the captured program;
missing, unsupported, or unresolved independent prerequisites do not open the generic confirmation dialog.
The rebuilt native approval matrix passed those controls and the valid-confirmation, prohibition,
retired-evidence, and cancellation controls before core consolidation.
The consolidated core subsequently passed the native permission, prohibition, and no-policy consumers.

`test:native:decision-context` passed with six assessments and eighteen inert child executions.
It exercised the five-message floor, the complete latest-user span,
complete long selected content, follow-up selection, and reload.
Its positive control retained the old prefix in the main-agent request;
the assessment wire omitted that prefix, hidden messages, and the raw provider request.
Receipt provenance remains local and separately validated.
The projection uses the same `buildVisibleContext` implementation as legacy mode,
with the running host's entry projector supplied at its boundary.

The rebuilt `test:native:bypass` passed for both inspected and ordinary three-statement programs.
Native execution completed three and two inert child calls respectively, without assessment.
Turning bypass off restored guarded admission;
reload, audit entries, status, and the fixed virtual-input guard passed in both cases.
These fixture results use authored model replies and presentation, not live provider or human approvals.
The [whole-system review](../planning/pi-auto-mode-whole-system-review.md)
recommends replacing the decision core and correcting bypass routing,
while retaining and requalifying useful native ownership controls.
It records the context-selection defect,
a built approval-router witness that generic Allow overrides an unrelated unsatisfied prerequisite,
and a built-host bypass regression against an upstream Codemode positive control.
The earlier `generic` control sent Deny and did not prove prerequisite preservation after Allow.
The accepted replacement is implemented; the review itself made no production edits.

The user corrected the assessment scope:
never send the full transcript at every decision point,
and inspect the legacy auto-mode implementation before redesigning context selection.
The incumbent contract is in `src/visible-context.ts` and `src/constants.ts`:
retain the larger of the latest five visible messages and the complete span from the latest user message.
With no user message, retain the latest five visible messages.
Keep each selected message complete;
omit persisted system messages, hidden custom messages, and provider-only metadata.
Use Pi's compaction-aware `buildContextEntries()` projection.
Project-context files, current action/input, and active trust directives are separate judge inputs.
Commit `4c8fe3a909` records the window decision;
`cad376be5b` records the compaction-aware projection.
The rebuilt legacy `context.unit.test.ts` passes those window and omission controls on 2026-10-15.
The reviewed native path had indexed captured source receipts and emitted every source's complete content
through `native/contract/integration/action-policy/wire.mjs`,
then repeats selected text through `wire-clause.mjs`.
Local retention for provenance does not require sending every retained byte to every assessment.
The context-sharing experiments remain scratch-only and are not the selected repair.
The replacement restores the incumbent decision-context selection boundary while retaining local provenance.
Do not describe byte sharing or a larger context window as resolution of this scope defect.
The user chose to keep the migration lesson in auto-mode documentation only;
do not add a general `AGENTS.md` migration rule.

The latest user decision raises the preparation-inclusive assessment deadline to 15 seconds.
The three sequential bundles of at most three parallel calls remain unchanged.
Native ownership, conversion, and HTTP lifetime guards now use the accepted duration.
The five-second results in this evidence ledger describe the superseded deadline.
Rebuild, type checking, transport, wire, bundle, deadline, preparation, lifecycle,
and dependency regressions passed after the deadline change in `c607c5f564`.
The unchanged live replay returned HTTP 400 for its third request after about 5.3 seconds:
`context_length_exceeded`, with `param: input` and `type: invalid_request_error`.
The original error text was:
`Your input exceeds the context window of this model. Please adjust your input and try again.`
The response did not report a token count or the effective limit used by the serving route.
OpenAI's current Luna model page documents 1050000 tokens;
serialized byte counts cannot establish whether that token limit was exceeded.

The user rejected treating normal workspace size as grounds to abandon automatic assessment.
The process-local assessment probe measured 615 instruction candidates,
617 total questions,
and 7420722 bytes in the internal selection request.
This is not a live provider rejection or a model token measurement.
The one-question-per-candidate, one-request implementation is the active design defect.
The unverified manual-capacity fallback in `8fe7e33410` is withdrawn.
The user now permits three sequential bundles with at most three parallel calls per bundle.
The user removed only the average USD 0.001 per judgment cost limit.
At that point the five-second preparation-inclusive hard deadline remained.
Complete attributed request partitioning and shared bundle accounting are implemented.
The configured-workspace CLI now assesses 617 questions in requests of 200, 200, 200, and 17,
using two sequential bundles.
Allow executes the prepared read;
Deny, Stop, dismissal, abort, generic unsatisfied prerequisites, prohibition, and bare-policy controls withhold it.
All checks use authored provider responses in a disposable network-isolated home.
No live-provider latency claim follows from these controls.
The built bundle tests verify parallel starts, exact membership through 1800 questions,
cancellation, malformed responses, stale capabilities, overlap rejection, and shared retry accounting.
The subsequent live-provider replay failed at the original five-second deadline.
The first bundle started three requests of 2566713, 2690737, and 4538472 bytes;
none returned response headers before cancellation and the fourth request was not sent.
The complete shared state is 2152170 bytes in each request.
This single observation establishes failure under the retained deadline,
not a causal latency comparison or a provider capacity rejection.
Normal live use is therefore not yet qualified.
The deciding receipt is `${HOME}/temp/agent/pi-live-bundled-assessment.log`;
the replay verifies exact equality with the configured fixture's provider request bodies.
An initial investigation considered sharing repeated context bytes while retaining source identities.
The user subsequently corrected that direction to restore legacy decision-context selection.
Do not drop instructions or silently relax the deadline.
Do not present size-triggered manual review as the completed repair.

Gateway controls on 2026-10-15 returned HTTP 200 for every small-input request.
Repeated identical one-question requests took 1692, 390, and 350 milliseconds;
repeated identical 200-question requests took 766, 1150, and 735 milliseconds.
Their encoded bodies were 276 and 38973 bytes respectively.
The receipt is `${HOME}/temp/agent/pi-gateway-small-controls.log`.
These establish a working route and measured variation,
not that gateway overhead is absent for the captured workspace input.
No direct-provider comparison has been performed.
The resolved repository environment has the Gateway credential but no direct OpenAI API key.
Do not recommend topping up direct service from the full-context timeout alone.

The user explicitly accepted removal of arbitrary capture limits on 2026-10-15.
Commits through `c65c7c9a3e` remove fixture-derived byte and occurrence limits from
prompt options,
tool metadata,
event/session/resource receipts,
run histories,
governing source lists,
and clause indexes.
The data copier now uses an iterative work stack with explicit ancestor-cycle detection.
Message aliases remain rejected;
supported shared tool metadata remains copied by occurrence.
Ownership,
freshness,
descriptor checks,
and cancellation are retained.
The generic prompt error now includes retained deciding error messages.

The original regression was red at `49b0dd1b08` and passes after the removal.
The installed native CLI fixture completes initial and follow-up prompts with bypass off,
all configured packages,
the copied repository instructions,
default tools,
and user/project skills.
It uses local provider replies in a network-isolated disposable home,
with no process-local capacity overrides.
The fixture is `capture-removal-repro.ts` under
`${HOME}/temp/agent/pi-tool-call-scope-20261014`.
Type checking and the updated capture, resource, event, history, governing-index,
tool-publication, file-observer and native-bypass regressions pass.
The configured native CLI's existing Allow, Deny, Stop, cancellation, abort,
generic prerequisite, prohibition and bare-policy controls pass with their original small policy.
The expanded full-workspace tool check originally found a separate downstream limit:
`instruction-batches.ts:144` rejects more than 256 instruction candidates before assessment.
Its exact error is `Clause relation requires one through 256 original instruction candidates`.
That limit and arbitrary aggregate claim/wire/response ceilings are now removed.
The provider's per-request question limit is handled by complete attributed partitioning.
The scratch `bundled-tool-controls` task verifies the repaired configured-workspace consumer.
Do not claim live-provider or physical-terminal verification from these offline checks.

The deciding pre-repair measurements were:
83938-byte callback options and a 113905-byte tool-loadout bundle against 65536-byte limits;
2850095-byte base snapshot against one MiB;
5282773-byte follow-up run accounting against four MiB;
and shared receipt bytes exceeding four MiB with only 123 receipts.
These counted repeated serialized observations, not measured heap or model tokens.
The smaller original smoke test had missed the configured workspace and follow-up interaction.
See [the accepted capture repair](../planning/pi-native-capture-capacity-repair.md).

The latest user instruction is to stop this agent's cleanup once Pi is functional,
then draft a prompt for MiMo v2.6 Pro to continue lint fixes.
The Shift+Tab regression is repaired through `775efbc86c`,
with formatting in `5e40bca7c4`.
The built native bootstrap now registers `shift+tab`.
The built native bypass consumer passes enabled execution of a producer and parallel children,
zero policy assessments in bypass,
disabled-mode prohibition,
reload restoration,
visible status,
audit entries,
and the fixed virtual-input guard in both modes.
Native and maintained builds plus package type checking pass.
The installed command resolves to this rebuilt native host;
an already-running Pi process needs a restart to acquire the changed runtime.
Physical keyboard delivery in that running terminal has not been exercised.
The lint queue remains paused for this agent.
Use [the MiMo handoff prompt](pi-auto-mode-mimo-lint-prompt.md) for the next cleanup session.
The warning counts in the historical entries are measurements of their named commits,
not a recount after the bypass repair.

### Historical regression and cleanup evidence

User-reported regression takes priority over the warning cleanup:
the native migration lost the Shift+Tab bypass toggle.
`mise run probe-native-bypass` in `${HOME}/temp/agent/pi-tool-call-scope-20261014`
reproduces missing registration in the built native bootstrap.
A control extension registers the same key successfully.
Installed thinking cycling remains on Ctrl+E;
the legacy auto-mode extension is absent from installed settings.
The legacy entry point owns the toggle,
while native resource construction only captures its API and does not register a replacement.
The prior blanket statement that Pi is functional is retracted for interactive feature parity.
The provider and approval checks remain valid within their tested surfaces.
At the initial diagnosis no bypass restoration had been applied;
the active-work section records its subsequent repair.
See the Shift+Tab section in `doc/troubleshooting/pi-sdk-staging.md` for reproduction and source evidence.
The native replacement must preserve explicit bypass behavior,
the fixed virtual-input guard,
session restoration,
visible status,
and audit entries before this regression can be closed.
The user also proposed delegating cleanup to a cheaper model;
Luna with the existing verification gates was recommended,
but no delegation was launched.

The dispatcher member-lifecycle boundary is complete:
`prepared-lifecycle.ts` and `prepared-lifecycle-types.ts` now own original member callback sequencing.
The dispatcher retains shared group state and passes original queue operations through borrowed callbacks.
Built consumers cover ordering,
opaque callback failures,
exact native identities,
once-only preparation release,
and synchronous disposal failure ownership.
These changes are committed through `9d120a7020`.
Focused formatting reports zero diagnostics in the six new files.
Native build,
package type checking,
all built consumers,
and original dispatcher replay pass.
Six domain comparisons preserve public outcomes and native execution counts,
including a permission-versus-prohibition positive control.
All 40 process-local omissions or substitutions produce assertion failures;
the built lifecycle and dispatcher hashes remain unchanged.
Receipts are `typed-prepared-lifecycle-comparison.json` and `typed-prepared-lifecycle-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The original dispatcher is saved as
`${HOME}/temp/agent/pi-tool-call-scope-20261014/prepared-dispatch-before-lifecycle-types.mjs`.
The complete lint recount covers 826 files with zero errors and 10151 warnings.
All six lifecycle files have zero diagnostics.
The remaining dispatcher glue has 88 warnings,
compared with 85 before extraction;
the additional borrowed callbacks still reside in the untyped original module.
The next boundary is that remaining dispatcher ownership and queue setup.
`program-profile-reviewed.mjs` remains the following profile boundary with 74 warnings.
The complete lint output is
`${HOME}/temp/agent/pi-maintained-typed-prepared-lifecycle-final-lint-20261015.log`.
The saved original dispatcher has SHA-256
`e7a2feb4f3093675fb0b2ea3e4be5570d66720dc260106340903ffbc82a83ce1`.

The next extraction delegates original parallel successor inspection to `profile-successors.ts`.
Its borrowed call inspector and private pattern ownership remain unchanged.
Native build, package type checking, all built consumers, and exact original replay pass.
Six domain comparisons preserve public outcomes and native execution counts,
including a permission-versus-prohibition positive control.
Public syntax consumers retain the direct and binding forms and both parallel combinators.
All 33 process-local omissions or substitutions produce assertion failures;
the built helper and profile-inspector hashes remain unchanged.
Consumers isolate overlapping syntax guards and retain borrowed callback identities,
repeated binding and member reads,
partial member inspection before later failures,
and original sparse synthetic mapping.
The helper leaves declaration envelopes unfrozen for their original outer owner.
The complete recount covers 820 files with zero errors and 10148 warnings;
all six successor files have zero diagnostics.
The remaining profile inspector has 61 warnings;
the dispatcher has 85 and is the next native boundary.
Receipts are `typed-profile-successors-comparison.json` and `typed-profile-successors-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The complete lint output is
`${HOME}/temp/agent/pi-maintained-typed-profile-successors-final-lint-20261015.log`.
The saved original profile inspector has SHA-256
`dab8f248c9ba906586e967e1e55ca90911170a05faa054aedfc99ff623c907d4`.
The functional scope correction and live/human checks remain complete.
Continue the remaining maintained native quality queue.

Original pattern decoding now delegates to `pattern-decoder.ts` and `pattern-decoder-types.ts`.
The helper borrows the original private pattern owner and diagnostic constructor.
Original ownership and materialization stay in `result-pattern.mjs`.
Native build, package type checking, all built consumers, original replay, and six domain comparisons pass.
Public syntax consumers also pass.
The final complete recount covers 814 files with zero errors and 10194 warnings;
all six new decoder files have zero diagnostics.
Omission controls found an overlapping binding guard in the initial fixtures:
removing the defined-binding check still rejected named identifiers through the name check.
Added nameless synthetic identifiers isolate the defined-binding guard.
Positive collection-bound and child-binding tests now assert successful completion explicitly.
Final package type checking, all consumers, and original replay pass after these additions.
All 42 process-local omissions or substitutions produce assertion failures;
the built decoder and original pattern-owner artifact hashes remain unchanged.
Consumers preserve child-before-parent ownership publication,
literal and key getter ordering,
partial publication before later child rejection,
and original sparse synthetic array mapping.
Arrays and objects both exercise inclusive nesting bounds and overflow rejection.
The helper retains owner return identity and original diagnostic constructor identity.
Receipts are `typed-pattern-decoder-comparison.json` and `typed-pattern-decoder-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The complete lint output is
`${HOME}/temp/agent/pi-maintained-typed-pattern-decoder-final-lint-20261015.log`.
The saved original pattern module has SHA-256
`94e2d51ff0945655754fd36fbc636b7fdc1aaf75bfc6650d4a6a837a192b7cbd`.
The remaining pattern module has 40 warnings;
`program-profile-inspect.mjs` has 101 and is the next native boundary.

The dispatcher now delegates original `beforeToolCall` member preparation to
`prepared-member.ts` and `prepared-member-types.ts`.
Native build,
package type checking,
and all three built consumer suites pass.
Original replay and six domain comparisons pass.
All 46 process-local omissions or substitutions produce assertion failures,
with both built artifact hashes unchanged.
The readiness-publication and gate-resolution omissions select the first complete-group consumer,
which independently completes its gate after observing noncompletion;
other omissions execute their entire selected consumer suite.
The first complete recount found `no-restricted-syntax(no-variable-function-expression)`
in the added callable-result fixture.
That fixture now uses a function declaration;
final package type checking,
all built consumers,
and original replay pass.
The final complete recount has zero diagnostics in the member helper,
its types,
and its four consumer files.
The remaining dispatcher has 85 warnings;
`result-pattern.mjs` has 103 and is the next native boundary.
Receipts are `typed-prepared-member-comparison.json` and `typed-prepared-member-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The complete lint output is
`${HOME}/temp/agent/pi-maintained-typed-prepared-member-final-lint-20261015.log`.
The saved original dispatcher has SHA-256
`a702ad6088db1055149724cf4a26ac59127d6c1d8183215a5d4bb4fc8e196cc0`.
Consumers cover original hook entry,
input sealing,
declaration-ordered manifests,
shared assessment admission,
and retained preparation failures.
They preserve first-decision ownership when assessment settles after another decision,
original gate rejection handling,
opaque preparation reasons,
callable preparation results,
and changing assessment getters.
The native callback directly forwards the helper's asynchronous result.
Currentness and preparation-release callbacks retain their synchronous return semantics.
Entry publication remains outside retained-failure handling.
The call-inspection evidence recorded here is complete.

Original call inspection now delegates to `profile-call.ts` and `profile-call-types.ts`.
The original pattern decoder and diagnostic constructor remain borrowed from `result-pattern.mjs`.
Native build,
package type checking,
and both built consumers pass.
Original replay and six domain comparisons also pass.
Public syntax consumers accept the original direct and binding forms,
reject 26 unsupported forms,
and retain both parallel combinators.
The computed-member omission exposed overlapping rejection guards in the first consumer fixture:
`tools["tool"]` was still rejected by the property-kind check.
The added `tools[tool]` fixture isolates computed-member rejection.
All 26 process-local omissions or substitutions produce assertion failures;
the built profile inspector and call helper retain their original hashes.
Final package type checking,
built consumers,
and original replay pass after adding the isolated computed-member fixture.
The complete recount has zero diagnostics in the call helper,
its types,
and its three consumer files.
The remaining profile inspector has 101 warnings;
`prepared-dispatch.mjs` has 121 and is the next native boundary.
Receipts are `typed-profile-call-comparison.json` and `typed-profile-call-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The complete lint output is
`${HOME}/temp/agent/pi-maintained-typed-profile-call-final-lint-20261015.log`.
The saved original profile inspector has SHA-256
`a637372dd68a7fcf2cac28661021aface213396afe31e5ad0ad556f7a087da12`.
Replay retains the original call body and original `requireProfile` predicate,
borrowing the original error constructor and decoder through the extracted helper boundary.
Consumers preserve decoder argument identity,
property read ordering,
escaped-identifier source text,
and original fractional end-position admission for synthetic nodes.
That synthetic case does not establish that the parser emits fractional source positions.

The tool-call scope correction,
live Hyper/Gemma assessment,
Luna question execution,
and genuine human Allow verification are complete.
Do not repeat the completed live or human checks merely to resume the maintained-code cleanup.

The maintained package remains unfinished.
The latest complete lint report covers 820 files and 486 rules,
with zero errors and 10148 warnings.
The strict warning gate still fails.
Clean extracted pieces do not establish package completeness.
The preceding 10902-warning report was audited by category:
8421 unsafe-value diagnostics,
1409 formatting diagnostics,
674 strict-boolean diagnostics,
and 398 other diagnostics.
Its warning distribution and the subsequent recounts are recorded in this handover's evidence chronology.

Original failure ownership,
collector lifetime,
collector currentness,
dispatch,
construction,
and native arrival are qualified through built consumers,
original-code replay,
domain comparisons,
and deliberate omission controls.
Reached-program runtime is also qualified.
Continue the remaining native dispatcher and profile contracts.
The collector implementation and new consumers have zero diagnostics in the full report.
The latest report has zero runtime,
extracted runner-binding,
extracted runner-event,
extracted runner-execution,
or extracted runner-facade diagnostics,
and retains 22 warnings in `program-runner.mjs`.
Next prioritize the remaining high-count native boundaries using the complete diagnostic report.
No current user-input blocker exists.

Prepared input sealing now delegates to `prepared-input-seal.ts`.
The original dispatcher still owns the contract error constructor.
The native build and package type check pass after correcting a fixture's directly exported binding pattern
(`TS9019`).
Both built sealing consumer suites and their exact original-code replay pass.
Six domain comparisons preserve public outcomes and native execution counts,
including a permission-versus-prohibition positive control.
The saved original dispatcher has SHA-256
`be4d2f66a767fc95669eb4369f9b957668b9a93e22a50ead971bdcde51f11f8c`.
Final consumer verification and the package type check pass,
including the 10,000-container deep-tree fixture.
All 18 process-local omissions or substitutions produce assertion failures;
the built dispatcher and sealing helper retain their original hashes.
The complete lint recount has zero diagnostics in the sealing helper and its three consumer files.
The first recount found a destructuring style warning in the added deep-tree test;
the final recount follows its correction.
Receipts are `typed-input-seal-comparison.json` and `typed-input-seal-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The complete lint output is
`${HOME}/temp/agent/pi-maintained-typed-input-seal-final-lint-20261015.log`.
The remaining dispatcher has 178 warnings and is the next native boundary to extract.
The next extraction now delegates original call admission,
per-call cloning,
judgment capability checks,
and sequential scheduling to `prepared-schedule.ts` and `prepared-schedule-types.ts`.
The native build,
package type check,
built consumers,
and exact original-code replay pass.
Six domain comparisons preserve public outcomes and native execution counts,
including the permission-versus-prohibition positive control.
All 21 process-local omissions or substitutions produce assertion failures,
with the built dispatcher and scheduler hashes unchanged.
The complete lint recount has zero diagnostics in the scheduler,
its types,
and its three consumer files.
The first recount found a multiline TSDoc warning;
the final recount follows its correction.
Final native rebuild,
package type check,
built consumers,
and original replay also pass with the added sparse-array fixture.
That fixture preserves original scheduling of an array hole when no tool lookup observes it;
it does not establish complete dispatch for a sparse group.
The remaining dispatcher has 151 warnings.
The complete lint output is
`${HOME}/temp/agent/pi-maintained-typed-prepared-schedule-final-lint-20261015.log`.
Receipts are `typed-prepared-schedule-comparison.json` and `typed-prepared-schedule-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The saved original dispatcher has SHA-256
`44cce9180cdcd4b0e17b611ae1cfa5d20d96f2fed2f46f7f3f5cb29ab77b168f`.
Consumers preserve first-match tool lookup,
cloning before mode admission,
context validation before serial-policy reads,
and explicit sequential mode bypassing those policy reads.
The pre-extraction dispatcher is saved as
`${HOME}/temp/agent/pi-tool-call-scope-20261014/prepared-dispatch-before-schedule-types.mjs`.
The original array check counts own keys;
it can accept a missing index offset by an extra enumerable property.
The extraction preserves that behavior and explicitly tests it.
It also preserves parent freezing before child validation and original proxy failure identity.

The next dispatcher extraction delegates final native outcomes and retained lifecycle failures to
`prepared-completion.ts` and `prepared-completion-types.ts`.
Original error constructors remain in the dispatcher.
The native build,
package type check,
both built consumer suites,
and exact original-code replay pass.
Six domain comparisons preserve public outcomes and native execution counts,
including the permission-versus-prohibition positive control.
All 18 process-local omissions or substitutions produce assertion failures;
the built dispatcher and completion helper retain their original hashes.
The complete lint recount has zero diagnostics in the completion helper,
its types,
and its three consumer files.
The remaining dispatcher has 137 warnings.
Consumers preserve original failure precedence,
duplicate and nullish aggregate causes,
native outcome identity,
getter ordering,
first-snapshot identity,
and frozen completion envelopes.
Receipts are `typed-prepared-completion-comparison.json` and `typed-prepared-completion-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The complete lint output is
`${HOME}/temp/agent/pi-maintained-typed-prepared-completion-final-lint-20261015.log`.
The saved original dispatcher has SHA-256
`3810c52f074c0e5e71af7fd90742c797162ade5536e62330ecd1697c96c2c59d`.
The original dispatcher is saved as
`${HOME}/temp/agent/pi-tool-call-scope-20261014/prepared-dispatch-before-completion-types.mjs`.

The next extraction delegates original tool snapshots and native execution wrappers to
`prepared-executable.ts` and `prepared-executable-types.ts`.
The dispatcher retains its synchronous `turns[ordinal].promise` lookup through a borrowed `waitForTurn` callback.
Original context ownership and error constructors remain unchanged.
The native build,
package type check,
both built consumers,
and original wrapper replay pass.
Six complete domain comparisons preserve outcomes and native execution counts,
including the permission-versus-prohibition positive control.
All 17 process-local omissions or substitutions produce assertion failures;
the built wrapper and dispatcher hashes remain unchanged.
The retained-callback control first failed with a raw getter error;
the final test explicitly asserts successful completion,
so reintroducing that read now produces an assertion failure.
Final consumer execution,
type checking,
and original replay pass after that test correction.
The full maintained recount has zero diagnostics in the wrapper,
its types,
and its three consumer files.
The dispatcher now has 121 warnings;
`program-profile-inspect.mjs` has 127 and is the next native boundary to extract.
Wrapper consumers retain original metadata getter order,
native argument identity,
turn failure ownership,
synchronous currentness semantics,
and shared guard-failure admission.
Diagnostic callback names are captured before wrapper publication,
so retained execution never rereads a replaced wrapper method.
Receipts are `typed-prepared-executable-comparison.json` and `typed-prepared-executable-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The full lint output is
`${HOME}/temp/agent/pi-maintained-typed-prepared-executable-final-lint-20261015.log`.
The saved original dispatcher has SHA-256
`70fd8fc54b6af128c5c29bf7dd4f7f63ff2dc194cbccd7fd0936645aa7b1de69`.
Wrapper replay adapts only the original turn lookup expression to the borrowed synchronous callback;
complete domain replay uses the untouched saved dispatcher.
The pre-extraction dispatcher is
`${HOME}/temp/agent/pi-tool-call-scope-20261014/prepared-dispatch-before-executable-types.mjs`.

The dispatch extraction is implemented in `group-dispatch.ts` and `group-dispatch-types.ts`.
Its production scope passes focused lint with zero diagnostics.
The original collector delegates dispatch while retaining its existing arrival code.
Independent consumer tests cover original callback order,
work publication before callbacks,
complete member association,
opaque failure propagation,
closed transport,
and retained unassociated array outcomes.
The native build,
package type check,
and built consumer suites pass.
The same consumers pass against the exact saved original dispatch body.
Six domain comparisons retain identical public results and native execution counts,
with a permission-versus-prohibition positive control.
All 32 process-local dispatch omissions or substitutions are detected;
the collector and dispatch artifacts retain their original hashes.
The first omission harness stopped on a selector matching both currentness calls.
After anchoring the selector to its exact line,
a phase-publication omission exposed an unbounded observation wait in a test.
The corrected test checks for missing observation through the package's existing Node scheduling checkpoint.
The final complete omission run passes with assertion failures for every deliberate change.
The complete lint report has zero diagnostics in the new dispatch implementation and consumers.
The package-wide warning gate remains failing.
Receipts are `typed-group-dispatch-comparison.json` and `typed-group-dispatch-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
Build,
type check,
consumer replay,
and omission output are retained in
`${HOME}/temp/agent/pi-group-dispatch-bounded-final-20261015.log`.
The unchanged original collector is saved as
`${HOME}/temp/agent/pi-tool-call-scope-20261014/program-collector-before-dispatch-types.mjs`,
with SHA-256 `740fd23a569b058f0d46e7c7b12ff9ffcfed85bd6f519f5d5de5408d7fa13912`.

Collector construction and member arrival are now extracted to `group-collector.ts` and `group-arrival.ts`.
The original `program-collector.mjs` path delegates to those typed owners.
The native listener retains its original identity for installation,
manual replay,
and detachment.
Completion input now consumes the shared original primary-failure slot,
including the collector's explicit pre-failure `undefined` value.
Independent consumers cover capability and frozen-view checks,
out-of-order arrivals,
argument serialization,
replay and reentry,
combined cancellation,
late native outcomes,
and cancellation during final currentness or listener installation.
The native build,
package type checking,
and composed collector,
dispatch,
and completion consumers pass.
The composed collector consumers also pass with the exact saved original collector.
Six domain comparisons retain identical public results and native execution counts,
with a permission-versus-prohibition positive control.
The first omission run detected its earlier changes but did not detect removal of arrival's pending-rejection callback:
the composed failure owner had already rejected the same transport ticket.
Independent arrival consumers now separate retention from rejection and exercise phase,
closure,
primary-failure,
and native-listener guards independently.
The expanded built consumer suites and package type check pass.
The same suites pass against the exact saved original collector and original arrival function body.
All 51 process-local omissions or substitutions are detected through assertion failures,
including the separately observed pending-rejection callback.
The collector bridge,
collector implementation,
and arrival implementation retain their starting artifact hashes.
The complete maintained lint report has zero errors and 10760 warnings across 740 files and 486 rules;
all 12 changed collector files have zero diagnostics.
The package-wide warning gate remains failing.
Receipts are `typed-group-collector-comparison.json` and `typed-group-collector-omissions.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The final verification sequence is retained in
`${HOME}/temp/agent/pi-group-collector-independent-final-20261015.log`;
the full report is
`${HOME}/temp/agent/pi-maintained-typed-group-collector-final-lint-20261015.log`.
The preceding collector body is saved as
`${HOME}/temp/agent/pi-tool-call-scope-20261014/program-collector-before-arrival-types.mjs`,
with SHA-256 `eada90c16fdb852469c4079f7fea850afbf54c52e55eec199d4f997a3fb9d419`.

Reached-program runtime is now implemented in `program-runtime.ts`,
with separate original lifetime operations in `program-runtime-lifetime.ts`.
The original `.mjs` path delegates to these typed owners.
Independent runtime consumers and lifetime fixtures pass the native build,
package type check,
and original-code replay.
Focused runtime and fixture lint passes with zero diagnostics after documenting fixture ownership.
Six domain comparisons pass with identical public outcomes and native execution counts,
including the permission-versus-prohibition positive control.
The first omission pass exposed an undetected missing collector-parent currentness callback.
Added consumers now change parent currentness after reaching
and pass original native cancellation through the source tool.
The expanded consumers pass against both implementations.
All 44 deliberate runtime omissions or substitutions produce assertion failures,
and the runtime bridge and implementation artifacts retain their starting hashes.
The complete report has zero errors and 10621 warnings across 752 files and 486 rules;
all 13 runtime implementation and consumer files have zero diagnostics.
The package-wide warning gate remains failing.
The original body is retained as
`${HOME}/temp/agent/pi-tool-call-scope-20261014/program-runtime-before-types.mjs`,
with SHA-256 `2d2b88d31a0d6dd0febf4a42624b2b561b133ef2836758ac990950fa513b459f`.
Diagnostic probes measure a literal base error category for subclasses instead of their derived constructor name.
Missing program,
reached-group,
or current-group views and missing source slots now produce explicit `Error` null-check diagnostics;
the original implementation produced implicit `TypeError` diagnostics.
A sparse guard-history array now throws the explicit null-check error rather than `undefined`.
These malformed-publication differences do not establish exact failure-category equivalence.
Ordinary value-conversion failures retain their original identity,
and only original native result guards trigger the conversion-specific program-guard callback.
Receipts are `typed-program-runtime-comparison.json`,
`typed-program-runtime-omissions.json`,
and `typed-program-runtime-diagnostics.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
Build,
type checking,
original replay,
and complete omission output are retained in
`${HOME}/temp/agent/pi-program-runtime-expanded-20261015.log`.
That sequence stopped afterward on a diagnostic fixture that accidentally replaced the original callback receiver;
the corrected diagnostic probe passes in
`${HOME}/temp/agent/pi-program-runtime-diagnostic-contrasts-20261015.log`.
The final type check,
consumers,
and full lint report are in
`${HOME}/temp/agent/pi-maintained-typed-program-runtime-final-cleaner-lint-20261015.log`.

Runner group binding is now extracted to `program-runner-binding.ts` and its contract file.
The original runner class delegates its private binding method to that helper.
The original `bindProgramCalls` tool-array declaration now admits absent name-lookup results,
matching its existing runtime rejection boundary without introducing an earlier runner check.
Native option spread remains generic and preserves primitive,
symbol,
and getter behavior;
member cancellation lookup follows the option getters.
Independent consumer tests pass after the native build and package type check,
as do the existing program-owner consumers affected by the wider tool-lookup input type.
The same binding consumers pass against the exact saved original method.
Six domain comparisons retain identical public outcomes and native execution counts,
with a permission-versus-prohibition positive control.
All 23 deliberate binding omissions or substitutions produce assertion failures,
and both runner and binding artifacts retain their starting hashes.
The complete lint report has zero errors and 10568 warnings across 758 files and 486 rules;
all seven extracted binding and consumer files have zero diagnostics.
The original runner class and its remaining execution methods are not yet qualified as clean.
The complete original runner is saved as
`${HOME}/temp/agent/pi-tool-call-scope-20261014/program-runner-before-binding-types.mjs`,
with SHA-256 `fc2e5c4e145641e023a4d841a42f6e83628c7ecba1d8ce7f9fd87a2f7f3ce121`.
Unknown native call identifiers and removed reached-member slots now produce an explicit `Error` null-check diagnostic
instead of the original implicit `TypeError` from reading the missing member's signal.
Both implementations first evaluate original option getters and stop before native execution;
the diagnostic contrast is measured separately from valid-call equivalence.
Receipts are `typed-runner-binding-comparison.json`,
`typed-runner-binding-omissions.json`,
and `typed-runner-binding-diagnostics.json`
under `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The final build,
type check,
built consumers,
original replay,
omission controls,
and diagnostic contrasts pass in
`${HOME}/temp/agent/pi-runner-binding-final-verification-20261015.log`.
The full report is
`${HOME}/temp/agent/pi-maintained-typed-runner-binding-final-lint-20261015.log`.
Continue the runner's native event callbacks and reached-group execution lifecycle.

Runner event callbacks are now extracted to `program-runner-events.ts` and its contract file.
The original record map remains allocated before queue selection.
Recording precedes start and end event publication;
the original member ticket resolves only after the end event finishes.
The native build,
package type check,
and independent event consumers pass.
The same consumers pass against the exact saved original callback bodies.
Six domain comparisons retain identical public results and native execution counts,
including the permission-versus-prohibition positive control.
All 23 process-local event omissions or substitutions cause assertion failures;
the runner and event implementation artifacts retain their original hashes.
Unknown call identifiers and removed member tickets produce explicit `Error` null-check diagnostics
instead of the original implicit `TypeError` from reading a missing resolver.
Both implementations first complete recording and host end-event publication.
Receipts are `typed-runner-events-comparison.json`,
`typed-runner-events-omissions.json`,
and `typed-runner-events-diagnostics.json`.
The original runner is saved as `program-runner-before-events-types.mjs`;
the original callbacks are saved as `program-runner-events-original-methods.txt`.
These artifacts are retained under
`${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The complete maintained lint recount has zero errors and 10511 warnings across 764 files and 486 rules.
All six event implementation and consumer files have zero diagnostics.
The strict package-wide warning gate remains failing.
Final consumer tests and package type checking pass in
`${HOME}/temp/agent/pi-runner-events-final-consumers-20261015.log`.
The complete report is
`${HOME}/temp/agent/pi-maintained-typed-runner-events-final-lint-20261015.log`.
Continue reached-group execution and native queue ownership.

Reached-group execution is now extracted to `program-runner-execution.ts`,
with separate callback and contract files.
The original runner delegates execution while retaining its declared-program facade.
Queue selection,
parent currentness after acquisition,
native cancellation,
complete dispatch,
and scope disposal now pass independent built consumer tests and original-code replay.
The native build,
package type check,
and existing program-owner consumers pass.
Six domain comparisons retain identical public outcomes and native execution counts,
with the permission-versus-prohibition positive control.
The first type check exposed a pre-admission signal contract that required already validated `AbortSignal` values.
The input now retains opaque native signal values until the existing member-preparation owner validates them.
No earlier runner cancellation guard is introduced for absent native member signals.
Original judgment,
slot,
and ordinal-map capture precedes host callbacks;
independent consumers replace these fields during context lookup and still reach the original captured owner.
Malformed call identifiers or removed native slots now produce explicit null-check `Error` diagnostics
before judgment invocation.
The original method passed an undefined slot to the judgment.
The diagnostic fixture intentionally does not validate slots,
so its successful original callback return does not establish real judgment admission for malformed input.
This change in malformed-input rejection ownership is recorded separately from valid-call equivalence.
Comparison receipts are `typed-runner-execution-comparison.json` and `typed-runner-execution-diagnostics.json`.
All 33 deliberate execution omissions or substitutions cause assertion failures.
The runner,
execution owner,
and callback implementation retain their starting artifact hashes.
The complete maintained lint report has zero errors and 10433 warnings across 772 files and 486 rules.
All nine execution implementation,
shared contract,
and consumer files have zero diagnostics.
The strict package-wide warning gate remains failing.
The omission receipt is `typed-runner-execution-omissions.json`.
The original runner is saved as `program-runner-before-execution-types.mjs`,
and its original method is saved as `program-runner-execution-original-method.txt`.
Final build,
type checking,
and consumer results are in
`${HOME}/temp/agent/pi-runner-execution-final-verification-20261015.log`.
The complete lint report is
`${HOME}/temp/agent/pi-maintained-typed-runner-execution-final-lint-20261015.log`.
Continue the runner's declared-program facade,
including reached-group settlement,
retained failures,
and aggregate draining.
The facade extraction and independent consumers are now committed.
The native build,
package type checking,
and both independent consumer suites pass.
The same consumers pass against the exact original facade method.
Six domain comparisons retain identical public outcomes and execution counts,
with the permission-versus-prohibition positive control.
All 31 deliberate facade omissions or substitutions cause assertion failures;
the runner and facade artifacts retain their starting hashes.
The complete maintained lint recount covers 778 files and 486 rules,
with zero errors and 10383 warnings.
All six facade implementation and consumer files have zero diagnostics.
The strict package-wide warning gate remains failing.
The full report is
`${HOME}/temp/agent/pi-maintained-typed-runner-facade-final-lint-20261015.log`.
The remaining runner class adapter retains 22 warnings;
the prepared dispatcher retains 185 warnings and is the next extraction boundary.
`prepared-input-seal.ts` now contains the first typed dispatcher helper.
Its delegation,
independent consumers,
original replay,
omission controls,
and lint verification are pending.
The unchanged dispatcher is saved as `prepared-dispatch-before-input-seal-types.mjs`.
The original facade runner is retained as `program-runner-before-facade-types.mjs`,
and its original method is retained as `program-runner-facade-original-method.txt`.
The typed facade adds an explicit null check for missing script-value slots.
The diagnostic probe measures rejection before judgment invocation;
the original facade forwarded undefined to the independently controlled judgment.
That fixture does not validate slots and does not establish real judgment admission for malformed input.
Receipts are `typed-runner-facade-comparison.json`,
`typed-runner-facade-omissions.json`,
and `typed-runner-facade-diagnostics.json`.
Build,
type check,
and consumer results are retained in
`${HOME}/temp/agent/pi-runner-facade-second-verification-20261015.log`.
Original replay is retained in
`${HOME}/temp/agent/pi-runner-facade-original-replay-20261015.log`.
The original method and runner are retained under
`${HOME}/temp/agent/pi-tool-call-scope-20261014/`.

## Tool-call scope correction

The user clarified that automatic mode is meant to check model-initiated tool calls.
The authorization subject is the proposed tool operation and its arguments,
evaluated against applicable instructions and authorization.
The model's outbound API request supplies context;
its entire provider protocol is not a separate operation requiring automatic-mode approval.

The agent's status update described complete API-request field derivation as remaining delivery work.
That promoted an implementation mechanism into a product requirement.
Do not continue extending provider-field coverage as a delivery objective.
In particular,
Hyper's `thinking` control is not a model-initiated tool call.

The current implementation still contains the broader gate.
`native/contract/integration/native-batch/request-derivation-envelope.ts` rejects unknown request fields,
and `src/request-material-obligations.ts` can retain an independent whole-request instruction-use obligation.
The exact isolated Hyper capture failed the built envelope check with
`Request field thinking has no supported native instruction-use derivation`;
the same captured object without that field passed.
`${HOME}/temp/agent/pi-hyper-capture-20261014/envelope-before.json` retains this contrast.
No production code was changed during that probe.
This demonstrates the existing coupling,
not a requirement to add Hyper protocol validation.

The next implementation decision must narrow that coupling around tool-call policy evaluation.
Preserve applicable instructions,
the exact proposed operation,
authorization,
and cancellation checks.
Reassess the earlier 84-question admission count against this corrected boundary before treating it as a delivery gate.
The prior status estimate and provider-coverage next step are superseded by this correction.
No further `AGENTS.md` edit is adopted;
the user's existing decision to leave that file unchanged remains in force.

After the user instructed continuation,
commit `3ec320b8fa` removed whole-request admission from the native evaluator and compiler.
Commit `09bf9f9080` removed provider-field derivation from the original observer path
and replaced the old derivation-gate scenarios with tool-policy invariance checks.
The semantic policy factory no longer configures a whole-request question.
The original request remains contextual evidence;
it no longer receives an independent approval obligation.

The JavaScript and owned native builds,
package type checking,
and changed TypeScript lint scope pass.
The native metadata suite passes permission,
prohibition,
and unspecified policy with both Hyper-style `thinking` and an unrecognized provider metadata object.
The original approval-context suite passes Allow,
Deny,
Stop,
cancellation,
and Allow after reload with authored prerequisite facts and fixture UI responses.
Policy replacement,
cancellation,
and argument replacement after approval also retain their existing guards.

The actual configured Hyper CLI passes isolated permission,
prohibition,
and unspecified-policy controls with all 17 configured packages.
The permission case executes the original `read` tool and returns the disposable file contents;
the other cases execute no child tool.
Each assessment contains 82 clause questions and no whole-request question.
Primary responses and semantic answers are authored local inputs,
so these results establish host integration rather than live model decision quality.
All cases make zero provider requests.
Receipts are under `${HOME}/temp/agent/pi-hyper-tool-call-permission-SAmMCJ/`,
`pi-hyper-tool-call-prohibition-xgH6VW/`,
and `pi-hyper-tool-call-none-4W2K6i/` in the same scratch directory.
The permission controller originally miscounted nested completion events;
the prohibition controller accidentally treated its imported verifier as a CLI.
Both host runs had already finished successfully.
Their preserved outputs were verified without replaying the hosts.

The requested [Astra limitation entry](../limitation/astra.md) is committed as `930e3e854f`
and checked in rendered output.
Production prerequisite ownership,
live semantic behavior,
and the previously measured native lint backlog remain separate unfinished work.

### Fresh current-session confirmation implementation

The configured Hyper CLI reproduced the missing prerequisite-owner path with an otherwise permitted read
and an explicit requirement for a fresh Allow from the current session user.
It executed no child tool and reported unresolved instruction or operation facts.
`${HOME}/temp/agent/pi-hyper-tool-call-prerequisite-2Y4UOq/verification.json` retains that control.

Commit `58a3265939` adds the `fresh-session-approval` clause meaning and its original native channel facts.
The meaning describes a clause expressly requiring a new response from this session's user
for the exact prepared group.
Generic authorization,
other approvers,
existing or reusable grants,
and additional conditions remain outside this particular meaning.
The existing `0.60` admission cutoff applies to the expanded vocabulary.
Historical response distributions retain their old vocabulary and cannot be replayed as current responses.

The native evaluator now captures the actual pending confirmation channel before decision closure or tool entry.
Its code-owned facts start unsatisfied;
absent or headless presentation is unsupported.
Original capability identities and judgment dependencies remain checked during review and execution.
Only an admitted fresh-confirmation clause consumes these facts.
Generic prerequisites retain their independent owner path.
Permission and prohibition still resolve separately,
and the existing native binder consumes the actual Allow,
Deny,
or Stop response.
Quoted fresh-confirmation targets retain unresolved authorization facts through their existing incorporation path.

Unit and native fixture additions are committed in `67fa816d5a`.
The first lint pass found callable identity reads represented as unbound methods,
missing documentation,
and test assertion typing and formatting errors.
The implementation now retains callable identities as non-invocable unknown values;
tests preserve the actual dependency failure values.
The second pass had only one test formatting warning,
which was corrected before starting the current build,
type check,
and final changed-file lint.
These checks and the new native approval scenarios remain pending at this checkpoint.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/rpc-host.ts`
is prepared to exercise the real configured Hyper CLI's RPC approval dialog with authored semantic answers,
without injected authorization facts or external provider requests.
Run it only after the updated native artifact is built.

### Confirmation rejection stops the run

The user requested identical Stop semantics for denial and cancellation whenever the confirmation dialog is shown.
Commit `b711afc237` reduces the dialog to `Allow` and `Stop`.
Every non-Allow result,
including a legacy `Deny` value or dismissed selector,
records `user-deny` with reason `user stopped`,
then invokes the original context's abort capability.
The obsolete `reflectExplanationOnDeny` option and its callers are removed.
The separate headless fallback still records the unavailable approval channel.

The JavaScript and owned native builds,
package type check,
changed TypeScript lint,
and focused built confirmation tests pass.
The lint task reported zero warnings and errors and exited zero;
its logger subsequently emitted `sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify`.
The abort omission control makes the rejection assertions fail while retaining the original artifact unchanged.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/abort-verification.json` records that check.

The native fresh-confirmation suite passes Allow,
legacy denial,
Stop,
cancellation,
unspecified policy,
and prohibition without an injected authorization owner.
The actual configured Hyper RPC host passes the same response routes,
external cancellation,
and generic-prerequisite rejection with all 17 configured packages.
Allow reads the disposable file once;
all other cases execute no child tool.
The fixture expects `Allow` and `Stop` and exercises the original production requester.
`${HOME}/temp/agent/pi-fresh-session-hyper-rpc-original-source-20261014.log` names every preserved receipt.

These host cases author clause meanings for the original policy source and `none` for other candidates.
They establish confirmation plumbing,
not live semantic interpretation or genuine human permission.
The first RPC fixture also assigned the fresh-confirmation meaning to a same-tag occurrence
inside an extension-produced composite prompt.
That case remains blocked by unresolved source authority;
its original outputs are preserved under `pi-hyper-rpc-allow-aIIepr` and `pi-hyper-rpc-allow-rKcQFn`.
Restricting fixture answers to the intended source does not repair this separate interpretation boundary.

The next check sends the unchanged current captured assessment to Luna,
using the expanded vocabulary and the adopted `0.60` operating rule.
Its artifacts live under `${HOME}/temp/agent/pi-luna-current-confirmation-20261014/`.
Do not replay those calls or replace their original responses.

### Manual confirmation for uncertainty

The user selected confirmation for uncertain automatic assessments on 2026-10-14.
Commit `e39c3da741` records that decision;
`7d04a67ea8` implements the native routing,
with tests in `88c8b19010` and formatting in `c2b5f9ed89`.
Unresolved and unspecified assessments reach the existing original-group dialog when a UI is available.
An established prohibition at the strongest known applicable tier still blocks,
even if another interpretation remains unresolved.
Known stronger applicable permission may supersede a lower-priority prohibition.
Original source and operation freshness checks remain mandatory.

The unchanged captured assessment produced identical answer data in three live Luna calls.
The adopted consumer admitted 22 of 84 answers each time,
including original permission at `0.60`,
fresh confirmation at `0.99`,
and the duplicate confirmation at `1`.
These are admission measurements,
not accuracy estimates.
The first local verifier incorrectly passed response metadata into the strict choice consumer.
Its already-completed response was preserved and rescored with the actual choice/probability fields;
only the remaining two planned calls were made.

The native manual-review suite passes its priority cases,
Allow,
denial,
Stop,
cancellation,
prohibition alongside uncertainty,
and missing UI.
The package type check and changed TypeScript lint pass.
The original authorization fixture suite also passes.
The approval-context suite passed its initial cases but its reload case exhausted the original judgment deadline.
The standalone unchanged reload control passes with one selection and three child calls.
The failed run remains preserved;
no cause for its deadline exhaustion has been established.
`${HOME}/temp/agent/pi-manual-review-reload-control-20261014.log` records the successful control.

Exact replay of the older Luna response in another configured session was rejected:
captured event timestamps and durations changed the assessment input.
No response was admitted in that failed replay.
A fresh-request bridge then exercised the configured native Hyper host with live Luna responses,
using scripted primary responses and scripted UI responses in a disposable network-isolated child.
The parent sends only the actual captured assessment to Luna;
the child receives no real credentials.
Each case makes one Luna request.
Allow executes the original read once;
Stop and dismissal execute no child tool.
Receipts are in `pi-hyper-rpc-allow-hRnQ6s`,
`pi-hyper-rpc-stop-UJfsga`,
and `pi-hyper-rpc-cancelled-9LsqI6` under `${HOME}/temp/agent/`.

The real Hyper primary and live Luna assessment now pass together through the installed native CLI.
The generated program reads `proof.txt` with the original `read` tool after a scripted Allow.
All 17 configured packages are loaded,
the read executes once,
and the child exits zero.
`pi-hyper-rpc-allow-Gt6QZx/verification.json` records two Hyper requests and one Luna request.
The initial receipt counted only Luna;
its provider total was corrected from retained responses without another host run.
The first attempt stopped before any provider request because the saved Hyper OAuth access token had expired.
The installed provider's `refreshHyperToken` and Pi's locked credential owner refreshed it normally.
The disposable child still receives only fake credentials;
the parent holds current credentials in memory for the exact bridged requests.

Both new process-local omission controls pass:
removing established-prohibition protection fails its named outcome assertion,
and removing uncertainty routing fails the native confirmation assertions.
`pi-tool-call-scope-20261014/manual-review-prohibition-verification.json`
and `manual-review-uncertainty-verification.json` retain unchanged built-artifact hashes.
The package README now distinguishes installed native behavior from the retained legacy extension.
Native documentation uses the adopted `0.60` cutoff and current confirmation semantics.
Rendered output has been inspected.

The user supplied Allow to the pending original native dialog in
`pi-hyper-rpc-human-bFNRvU` under `${HOME}/temp/agent/`.
Its exact operation was `return await tools.read({path: "proof.txt"});`.
The controller forwarded that response to the unchanged pending request identifier.
The original tool executed once and returned the expected disposable file contents;
the host exited zero.
`verification.json` records one genuine human response,
two actual Hyper requests,
one actual Luna request,
and no authored primary responses,
semantic answers,
or authorization facts.
This completes the live production-host human-confirmation boundary.
It grants no permission for another operation or session.

The full package lint now reports 14,550 warnings and 454 errors across 425 files.
`${HOME}/temp/agent/pi-native-full-lint-manual-review-20261014.log` retains the complete diagnostics.
This is a changed file set,
not a controlled comparison with the older 389-file count.
Continue the maintained native quality work;
do not reinterpret this passing live operation as full package completion.

## Current Pi execution model

### Owned runtime source correction

The October 14 native formatting pass exposed exact-text dependencies in the maintained build.
`replaceOnce` in `native/contract/integration/native-batch/build-rebase.ts`
locates a literal substring and rejects absent or repeated matches.
Formatting owned source therefore broke the runner,
dispatcher,
and judgment transformations.
The user challenged this design.
The agent acknowledged carrying the brittle prototype mechanism into the maintained package.

The first repair replaced the runner and dispatcher transformations with parsed source selection.
A subsequent consumer search established that the owned runner,
dispatcher,
and judgment inputs had only the native build as their consumer.
The source-patching requirement was unnecessary for those modules.
The integrated implementations are now maintained directly in
`native/contract/collector/program-rule-batch-sdk-copy-v2/input/`.
The build emits export bridges for them instead of rewriting their bodies.
The judgment directly imports its generated result converter,
whose separate installed-Pi adaptation remains unchanged.
The obsolete judgment transformations and the intermediate runner/dispatcher syntax transformers are removed.
Historical provenance hashes remain the original import record rather than being rewritten as current hashes.

`${HOME}/temp/agent/pi-tool-call-scope-20261014/owned-program-before/`
retains the successfully built implementations used for this integration.
`owned-program-promotion.json` records source and generated hashes.
Only import locations and the dispatcher's original shared error declarations changed during promotion.
The direct-integration build and type checks passed.
The first runtime suite stopped before executing its test because the formatting commit had inserted
`readonly #state` into `policy-source/system-slot.mjs`.
Commit `bbb1aa2f64` removes that TypeScript modifier from JavaScript;
the rebuilt domain,
manual review,
approval context,
single call,
approved dependency,
event receipt,
host lifecycle,
and actual CLI process suites all pass.
The syntax task separately exposed its own invalid assumption that `node --check` strips TypeScript.
The task now explicitly strips types before checking TypeScript modules,
while JavaScript modules reach Node unchanged.
That syntax task passes 227 maintained modules.
The native lint report after direct integration contains 533 errors;
the maintained-package quality queue remains open.
Commit `53c651c0c0` separates the shared governing-source error to remove the source-history import cycle.
Its owned build and native domain suite pass.
Commits `d9c7183140` and `d014715404` name native runner callbacks,
extract original recorder ownership,
and scope exclusive queue release with `using`.
The parallel branch retains synchronous entry without acquiring a queue turn.
The rebuilt native domain suite passes;
the focused lint report contains no errors in either runner module or the error/history modules.
Commit `99886bdca1` retains governing binding failure and source selection in one lifecycle record.
Commits `2e025dc5cc` and `d6c4d39496` share the original judgment operation wrappers
and name the remaining judgment callbacks.
Their owned build and complete direct-runtime regression sequence pass.
The focused lint check found no errors in the runner or source-binding modules;
the constructor retained file-length,
mutable-binding,
cleanup,
and function-declaration findings.
Commit `72a7e2ca36` adds original recorder ownership and queue-release tests.
Normal and exceptional queue release both pass.
The process-local omitted-release control fails with `queue-release-missing`,
and the built helper hash remains unchanged.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/program-queue-omission.json` retains that receipt.
Commits `25c6a6819f` and `813b9c1b35` separate original program ownership and instruction-claim ownership
from the judgment constructor.
Their owned build and focused lint checks are running;
their consumer regression verification remains queued before any completion claim.
The user requested a status update during this cleanup.
The response distinguished the completed live Hyper/Gemma,
Luna,
and genuine Allow check from the unfinished lint and module-splitting work.
Continue the maintained-package quality queue without reopening the completed live approval.
Commit `4e7545abda` consolidates one-time original instruction-handle publication;
its changed source is not part of the currently running pre-cleanup split regression artifact.
The pre-cleanup split artifact subsequently passed the full direct-runtime regression sequence.
Commit `a4bfcb943e` scopes judgment activity cleanup with `using`
and separates guarded result consumption.
The focused check of the constructor and its new owner modules now has zero lint errors.
Commit `a66c5d5c96` applies named callbacks,
owned state records,
and scoped cleanup to both prepared dispatchers,
preserving their separate root and child execution behavior.
The dispatcher cleanup build passed;
the full native structured lint report contains 312 errors.
Its domain checks passed,
but manual uncertainty Allow exhausted the original preparation deadline.
The same built artifact passed that exact Allow case in isolation,
with one selector call,
three executions,
and no provider calls.
The cause remains unestablished;
the interrupted sequence does not qualify the remaining consumer suites.
Commits `17cd623ee4`,
`4dddea7b46`,
and `d3d0de3168` separate reached-group failure and completion ownership,
retain delivery state explicitly,
and represent idle group work with a settled promise.
Their owned build and queue tests pass.
The domain sequence stopped at `JudgmentDeadlineError: Judgment preparation and assessment deadline expired`.
An unrelated bounded CPU stress task was active during this run;
its overlap does not establish the cause.
The complete regression sequence remains pending after that task finishes.
Commits `443d6bc709` and `d4e52c5394` separate original resource capture,
reload,
binding,
and governing-policy ownership and scope their cleanup.
Their owned build passes.
The focused structured lint report contains no errors across these resource modules and the group-runtime modules.
It still exits with failure because warnings remain;
`package/config/oxlint/src/config-base.ts:31` sets `denyWarnings: true`.
Zero error-severity diagnostics therefore do not mean the lint task passes.
Commit `fe77f1d9d6` retains base-composition and governing-source lifetime state explicitly.
The adjacent policy reviewer now scopes review activity and retains compilation results in its review record.
The source-lifetime and native host-hook changes subsequently built successfully.
The syntax task passed all 241 maintained native modules,
and package type checking passed.
The full lint attempt failed before producing diagnostics:
`tsgolint` emitted `failed to create new OS thread (have 2 already; errno=11)`.
The post-stress domain and manual-uncertainty suites passed;
ordinary native-context Allow then exhausted the original judgment deadline.
That exact case passed three unchanged isolated repeats,
with total process durations from 7205.50 to 7954.08 milliseconds.
These durations include setup and shutdown;
they are not judgment-only timings or evidence of a permanent timing remedy.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/native-allow-repeat.json` retains each result and CPU counters.
The subsequent approval-context,
single-call,
approved-dependency,
event-receipt,
host-lifecycle,
CLI-process,
queue,
resource-reload,
resource-copy,
and context-receipt suites all passed on that unchanged artifact.
The complete log is `${HOME}/temp/agent/pi-native-resource-runtime-remaining-20261014.log`.
Later source changes name remaining profile callbacks,
separate program inspection,
replace profile regex scans,
and retain the original reload and assessment await boundaries.
Those later edits remain pending build and consumer verification.
Their owned build subsequently passed;
the expanded resource/runtime regression sequence is running again on that artifact.
The full lint retry with `GOMAXPROCS=2` completed across 438 files,
reporting 203 errors and 12801 warnings.
All error-severity findings are in native source.
The later request-hook callback correction and explicit input-key comparators still need their focused lint check.
Direct source-consumer comparisons passed 456 identifier cases and 152 whitespace cases
against the replaced regex behavior.
The current full report is `${HOME}/temp/agent/pi-maintained-resource-lint-retry-20261014.log`;
the current running regression log is `${HOME}/temp/agent/pi-native-profile-cleanup-regressions-20261014.log`.
The user requested another status update during cleanup;
the answer distinguished the completed live installation from unfinished lint and intermittent deadline verification.
The profile-cleanup regression run subsequently stopped at ordinary native-context Allow:
`JudgmentDeadlineError: Judgment preparation and assessment deadline expired`
led to the original selector assertion `0 !== 1`.
Its preceding domain and manual-uncertainty suites passed.
This recurrence remains unresolved;
the earlier isolated passes do not establish a timing remedy.
Commit `fe24956975` separates the run-prompt owner into emission,
rendering,
retained-input,
context,
publication,
and lifecycle modules while retaining its original scope and failure maps.
Its owned build passes.
Focused structured lint reports zero errors and 1082 warnings across the eight changed modules;
the warning gate still fails.
Consumer verification is running in `${HOME}/temp/agent/pi-native-run-owner-regressions-20261014.log`.
That run subsequently passed all domain cases and manual uncertainty Allow,
Deny,
Stop,
and cancellation.
Its prohibited case stopped at `JudgmentDeadlineError`,
with the outcome assertion reporting `undefined` instead of a policy outcome.
Commit `c3cb4b1f97` separates loaded-input field selection,
history,
manager frames,
events,
tools,
Agent inputs,
and user inputs while retaining the original pool budget.
The first loaded-input build passed;
focused lint exposed detached method references after extraction.
Those callbacks now use named function values;
the updated build and focused checks remain pending.
Named function expressions still produced `typescript(unbound-method)` findings.
Commit `13dc500508` declares the loaded callbacks before constructing their returned facade.
Its build passes;
the loaded-pool lint subset now has zero errors and 51 warnings.
The other new loaded modules still need the updated full subset check.
The user requested status twice during this work;
the replies distinguished verified live installation from unfinished quality work and the timing diagnosis.
Unchanged ordinary Allow repetitions on the loaded-input build failed twice and passed once.
Their complete process durations were 10089.97,
8502.33,
and 12054.84 milliseconds,
including setup and shutdown.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/binding-set-before.json` retains the results and CPU counters.
Both separate CPU-profile runs passed,
for ordinary Allow and manual-uncertainty prohibition.
Their profiles identify repeated source-freshness validation as a sampled runtime cost;
these separate modes are not a before/after timing comparison.
An instrumented Allow fixture failed at `DependencyDeadlineError`
after 23740 opens of its original governing file.
The exact captured counts are in
`${HOME}/temp/agent/pi-tool-call-scope-20261014/loaded-allow-fs.json`.
The new `test:native:source-freshness-work` task exercises the built Allow consumer
and requires fewer than 10000 governing-file opens for that fixed fixture.
Its required-red run precedes the proposed complete-binding-set validation change.
Required-red commit `971f1bd` completed the native Allow fixture,
then failed its work-bound assertion with 45637 governing-file opens.
Commit `5b29e43e8d` validates local source-binding identities between fresh common-parent checks.
Individual public source assertions still perform their original parent freshness check.
The updated owned build passes.
The same native Allow fixture then completed with 15653 opens,
but the new work-bound regression still fails its 10000-open limit.
This is a measured reduction in repeated reads,
not completion of the performance fix or a full regression pass.
The logs are `${HOME}/temp/agent/pi-native-source-work-red-20261014.log`
and `${HOME}/temp/agent/pi-native-source-work-green-20261014.log`;
the latter filename names the intended stage,
not its actual failed assertion result.
Unchanged repetitions of this changed artifact all passed.
Their complete process durations were 8224.54,
6064.67,
and 6179.07 milliseconds;
`${HOME}/temp/agent/pi-tool-call-scope-20261014/binding-set-after.json` retains the results.
These include setup and shutdown and are not judgment-only latency measurements.
The next change introduces `readCurrentSourceBinding()` on the original judgment.
It returns original request and source identities only after `assertDependenciesCurrent()` succeeds.
The governing-source owner uses that operation instead of independently repeating observer and collector checks.
Commit `2ea0d1c836` builds successfully.
The unchanged work-bound regression still fails at 14185 opens after the Allow scenario completes.
`${HOME}/temp/agent/pi-native-current-source-work-20261014.log` retains that result.
A profile of this artifact is collecting evidence for the next caller to address;
the 10000-open bound remains unchanged.
The next built Allow probes still failed that bound:
`48e2532d46` put nested rule reads inside the original synchronous binding segment and measured 13233 opens;
`45c180d7d7` collected related policy selections in that segment and measured 12169 opens;
removing the repeated request observation before `active()` measured 11224 opens.
Each fixture completed its Allow scenario before failing the work assertion.
The current probe reads a prepared member's input and program inspection together after one dependency check.
The complete rejection and lifecycle suites remain pending.
Commit `b3f980b0f8` also fixes an extraction error:
the whole-request claim branch must register `retained.requestMaterialBinding`,
not the removed standalone variable.
The built `test:native:request-claim` consumer passes original request identity,
repeated selector reuse,
foreign-handle rejection,
and dependency retirement.
A process-local omission restored the removed variable reference and failed with
`ReferenceError: requestMaterialBinding is not defined`;
the built module hash was unchanged.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/request-claim-omission.json` records that control.
The prepared-member probe measured 10812 policy opens;
using the original judgment's validated identities during approval measured 10634.
Selecting the matching source before validating its binding did not change this fixture's 10634-open count.
That fixture therefore supplies no performance evidence for source lookup paths
that encounter earlier unrelated sources.
Removing the redundant dependency check before complete source validation measured 10236 opens.
The native Allow scenario passes,
but the unchanged 10000-open regression remains red.
The broader domain,
manual review,
approval,
single-call,
changed-dependency,
event,
host,
and CLI regression sequence is running against this frozen built artifact.
The user requested status during this work;
the response reported the verified live flow,
the remaining work-bound failure,
the last full lint counts,
and the absence of a reliable completion estimate.
The first broader regression sequence passed on `ac35a7c2ac`:
domain,
manual review,
approval context,
single-call,
changed-dependency,
event-receipt,
host-lifecycle,
and CLI-process checks.
Batch candidate selection consolidation then measured 10208 opens;
removing duplicate decision-wrapper validation before original approval measured 10086.
Commit `f79ac0ea45` reads execution origins through the already-required native entry check.
Its built work-bound regression passes at 9046 opens without changing the 10000-open limit.
The baseline required-red count remains 45637.
`${HOME}/temp/agent/pi-native-entry-origin-work-20261014.log` records the passing consumer.
The final expanded regression sequence is running in
`${HOME}/temp/agent/pi-native-source-boundaries-final-20261014.log`.
It adds program queue,
resource reload,
resource input,
context receipts,
and the whole-request claim consumer to the prior sequence.
On the `f79ac0ea45` build,
the repeated original runtime sequence passed again.
The scratch runner then stopped before the queue task because its working directory was outside the monorepo;
this was a task-launch error rather than an application assertion.
With the scratch task's `dir` pinned to the repository,
all remaining queue,
resource reload,
resource input,
context receipt,
and whole-request claim checks passed.
The complete split records are
`${HOME}/temp/agent/pi-native-source-boundaries-final-20261014.log`
and `${HOME}/temp/agent/pi-native-source-boundaries-remaining-20261014.log`.
Lint cleanup resumed with `45269cca6f` and `0313276bd7`:
tool diagnostic and declaration budgets,
guideline publication,
forced-message retirement,
and governing selection now retain mutable values in their original owned records.
Their updated build and source-work/resource consumers pass;
the work count remains 9046.
The full lint recount is running with `GOMAXPROCS=2` and the package's existing worker and heap limits,
without concurrent native runtime tests.
Its log is `${HOME}/temp/agent/pi-maintained-source-boundaries-lint-20261014.log`.
The recount completed across 455 files with 175 errors and 13188 warnings.
Errors decreased from the previous 203;
warnings increased as module extraction exposed additional untyped boundaries.
The rule configuration still rejects warnings.
Current structured diagnostics and error allocation are retained in
`${HOME}/temp/agent/pi-tool-call-scope-20261014/current-maintained-diagnostics.json`
and `current-native-errors.json` in that directory.
The new work counter now uses Node's actual `openSync(path, flags, mode)` callback signature.
The request preparation hook now retains its function value directly instead of detaching an object method.
The initial function-value edits replaced the prior findings with
`no-restricted-syntax(no-variable-function-expression)` findings.
Commit `0fab7d8f10` constructs the request callback through a named factory
and assigns the filesystem callback directly to its native hook.
Commit `b9a6a57750` retains the approval lifecycle fields in one owned record.
The focused callback,
counter,
and approval check has zero errors and 300 warnings;
it remains a failing warning gate.
The updated owned build and source-work/manual-review/approval/dependency consumers pass in
`${HOME}/temp/agent/pi-native-callback-approval-consumer-20261014.log`.
Native syntax passes all 259 maintained modules;
the package type check also passes.
Commit `81ecaf586b` separates dependency eligibility and synchronous binding checks into
`judgment-dependencies.mjs`,
with preparation and finalization state still supplied by the original constructor.
Its rebuilt manual-review,
approval-context,
and changed-dependency consumers pass.
The same source-work fixture passes at 9046 policy-file opens.
`${HOME}/temp/agent/pi-native-dependency-owner-consumer-20261014.log` retains those results.
Commit `33c1c35cd8` groups runtime publication state and judgment accounting in their respective owners
and scopes budget cleanup with synchronous disposal.
Focused lint reports zero errors and 736 warnings across the constructor,
dependency owner,
runtime owner,
and judgment budget.
The warning gate remains failing.
The rebuilt source-work,
domain,
host-lifecycle,
and actual CLI-process checks pass in
`${HOME}/temp/agent/pi-native-lifecycle-records-consumer-20261014.log`.
The host-lifecycle fixture includes runtime replacement,
cancelled replacement,
persisted resume,
and incomplete-disposal retry.
Commit `53defdddad` next scopes native wrapper reentrancy and execution cleanup
and gives owned wrapper constructors named input fields.
Native execute callbacks retain their positional SDK signature through a fixed factory.
Commit `79622d6e43` derives parent cancellation options without mutable temporary state.
The rebuilt wrapper consumers pass domain outcomes,
single-call execution,
post-approval source/cancellation/argument changes,
and normal and exceptional program queue release in
`${HOME}/temp/agent/pi-native-wrapper-lifetimes-consumer-20261014.log`.
The first wrapper lint found one remaining mutable options binding;
the subsequent cancellation helper addresses that finding.
The full maintained-package recount reports 131 errors and 13218 warnings across 456 files and 486 rules.
`${HOME}/temp/agent/pi-maintained-lifecycle-cleanup-lint-20261014.log` contains the structured diagnostics.
The constructor,
dependency owner,
runtime owner,
budget,
and wrapper files now have no error-severity findings in that full scan.
The warning gate remains open;
these results do not establish package completion.
Commit `9c28c21d7d` replaces synchronous fixture directory and file setup with awaited filesystem operations.
It also records network requests by their URL and keeps provider-call accounting in one fixture-owned record.
Focused lint reports zero errors and 64 warnings for `environment.mjs`.
The rebuilt domain,
context-receipt,
resource-input,
and approval-context checks pass in
`${HOME}/temp/agent/pi-native-async-environment-consumer-20261014.log`.
Commit `a96c4d1da1` scopes the CLI fixture's reader and process cleanup
and the historical-run fixture's session cleanup with disposal.
The historical-run fixture now retains its counters and original source in one record.
The package also exposes that existing fixture as `test:native:run-history`;
its rebuilt consumer and CLI cleanup checks pass.
Native syntax also passes all 260 modules at that artifact.
`${HOME}/temp/agent/pi-native-fixture-cleanup-consumer-20261014.log` retains those results.
The package type check passes after the lifecycle and wrapper interface changes.
Commit `2c67f036e7` adds explicit budget input,
retained-state,
and response types.
The budget's type-safety findings disappear in focused lint;
its total warning count changes from 55 to 41 because the first annotations introduced comment-format findings.
Commits `1e14b97233` and `35c0febc93` then separate stable error identities,
name the unchanged budget limits,
and correct the remaining statement and comment shapes.
The next focused report has two warnings:
sequential retry awaiting and an explicit return required in the classifier-error branch.
Commit `a8b5cf2811` addresses that return and documents the narrow sequential-retry exception in
the troubleshooting entry's "Sequential native judgment retries" section.
Commit `c9807abb0c` adds a built consumer for retry accounting,
overlap refusal,
deadline and cancellation retirement,
late response rejection,
classifier failures,
and stable error names and causes.
Its built check passes.
The process-local omitted overlap guard fails the expected rejection assertion;
the original built artifact hash is unchanged in
`${HOME}/temp/agent/pi-tool-call-scope-20261014/budget-overlap-omission.json`.
The final JavaScript subset reports zero errors and 139 warnings,
mostly in the new consumer.
The consumer's imported calls remain unchecked despite annotations inside the budget module.
The inherited TypeScript configuration sets `allowJs: false` and excludes these JavaScript consumers.
Commit `4f27290d9b` moves the implementation into TypeScript;
the package type check passes,
but the still-JavaScript consumer retains its unchecked-call warnings.
The migration also exposes TypeScript documentation requirements,
which are addressed in the following ownership and dispatch split.
Commits `a1f2204eed` and `0fc4da484c` document original ownership
and separate sequential dispatch from clock lifetime.
Commit `a4d9d43970` moves the consumer to typed tests using the existing module-test harness
and built native imports.
The rebuilt typed consumer tasks pass.
Their first typed lint exposes copied-artifact import classification and callback-return errors;
the unsafe-value warnings are absent.
The tests now follow the existing native fixture pattern:
load the copied module through its artifact URL,
check the required exports,
and invoke an independently typed consumer interface.
They do not add copied output to the TypeScript source project.
The final formatting pass reports zero errors and 27 remaining warnings;
the following edits separate callback statements and complete the fixture assertion documentation.
Commit `feb3b62cec` separates the remaining compound export assertion operators.
Package type checking passes;
focused lint passes all eight budget implementation and consumer files with zero warnings or errors.
`${HOME}/temp/agent/pi-native-budget-operator-lint-20261015.log` retains the complete report.
The rebuilt typed consumers pass in
`${HOME}/temp/agent/pi-native-budget-checked-consumer-20261015.log`.
Removing the dispatch overlap guard in memory makes the typed consumer fail its expected rejection assertion.
The unchanged artifact hashes and result are recorded in
`${HOME}/temp/agent/pi-tool-call-scope-20261014/budget-overlap-typescript-omission.json`.
Original source-work,
manual-review,
approval-context,
and changed-dependency consumers pass on the rebuilt typed budget in
`${HOME}/temp/agent/pi-native-budget-approval-consumer-20261015.log`.
The source-work check records 9045 policy-file opens within its unchanged 10000 limit;
this count alone does not establish a performance change.
Commit `a21d26fd12` separates host fixture construction and observations from lifecycle assertions
and scopes runtime cleanup and failed-disconnection restoration.
Its rebuilt host-lifecycle consumer passes replacement,
cancellation,
fork,
persisted resume,
and incomplete-disposal retry in
`${HOME}/temp/agent/pi-native-host-fixture-consumer-20261015.log`.
The subsequent full lint recount covers 464 files and 486 rules,
with 107 errors and 13167 warnings.
`${HOME}/temp/agent/pi-maintained-typed-budget-lint-20261015.log` retains the complete report.
This remains a failing package gate.
Commits `0f55a81f89`,
`fbbeedd83c`,
and `7924d5b74a` scope original producer construction and invocation claims,
retain CLI manager observations in a fixture record,
and name collector and manual-review helper inputs.
Focused lint reports zero errors and 356 warnings across that six-file subset.
The rebuilt request-claim,
domain,
CLI-manager,
host-lifecycle,
and manual-review consumers pass in
`${HOME}/temp/agent/pi-native-producer-manager-consumer-20261015.log`.
Commit `897d64e1b3` retains authorization observations in their fixture record;
commit `7095845a18` separates approval setup and scenario state from its original assertions.
The package now exposes `test:native:authorization-inputs` for the maintained authorization fixture.
The final approval and authorization subset has zero errors and 163 warnings.
Its rebuilt authorization,
manual-review,
approval-context,
and source-work consumers pass in
`${HOME}/temp/agent/pi-native-approval-fixtures-consumer-20261015.log`.
The work counter remains 9045 policy-file opens in that run.
The source-association split then separates envelope shape,
message origins,
original prompt contributions,
and one-time serialization closure into sibling modules.
The original callback and closure share one private observation record.
Focused lint has zero errors and 439 warnings across those modules.
Rebuilt serializer-receipt,
event-receipt,
approval-context,
and source-work checks pass in
`${HOME}/temp/agent/pi-native-source-association-consumer-20261015.log`.
The source-work count remains 9045.
The request observer now separates its original stream and receipt publication from lifetime validation.
The caller retains the same private response and receipt maps,
and original observation generation remains shared through its owner.
Focused lint has zero errors and 315 warnings across the observer modules.
Its rebuilt source-work,
domain,
changed-dependency,
and host-lifecycle consumers pass in
`${HOME}/temp/agent/pi-native-observer-split-consumer-20261015.log`.
Native syntax passes all 274 modules at that artifact.
Root lifecycle construction,
transition publication,
and diagnostic isolation are separated by commit `4d2b8c4748`.
Their focused lint has zero errors and 607 warnings.
The rebuilt manager-constructor,
CLI-manager,
host-lifecycle,
and changed-dependency consumers pass in
`${HOME}/temp/agent/pi-native-root-lifecycle-consumer-20261015.log`.
The policy compiler now delegates output packaging and remaining-premise filtering to `compilation-result.mjs`.
The compiler retains the original evidence and input validation before that final filtering call.
The full maintained lint recount reports 44 errors and 13464 warnings across 479 files and 486 rules in
`${HOME}/temp/agent/pi-maintained-observation-splits-lint-20261015.log`.
Errors decreased from 107;
warnings increased as extracted untyped module boundaries became visible.
The compiler retained one file-size error after its first extraction.
Its rebuilt domain,
semantic-operating-policy,
and manual-review consumers pass in
`${HOME}/temp/agent/pi-native-policy-compilation-consumer-20261015.log`.
The following extraction places original scheduler and inspected-program coverage in `operation-coverage.mjs`.
Focused lint now reports zero errors and 234 warnings across the compiler and its output and coverage helpers.
The rebuilt domain,
semantic-operating-policy,
and manual-review consumers also pass on that final extraction in
`${HOME}/temp/agent/pi-native-operation-coverage-consumer-20261015.log`.
The remaining full lint gate is still open.
Commit `41a33c04cc` separates adoption,
request,
heading,
and clause wire projections while retaining original binding checks and batch membership.
Its focused lint has zero errors and 376 warnings.
Commit `487e5ad3b0` separates resource,
run,
and generated prompt source bindings from authority assembly.
Its focused lint has zero errors and 384 warnings.
The rebuilt domain,
semantic-operating-policy,
manual-review,
source-work,
resource-input,
context-receipt,
and retained-history consumers pass in
`${HOME}/temp/agent/pi-native-wire-authority-consumer-20261015.log`.
Commit `42a7606059` separates loader publication state from collector snapshot ownership.
The original private policy-input construction key remains in `system-slot.mjs`;
the loader retains its publication record and each collector retains its own snapshot map.
Focused lint has zero errors and 550 warnings.
The same rebuilt policy and source consumers pass in
`${HOME}/temp/agent/pi-native-system-slot-split-consumer-20261015.log`.
The fixed ASCII rule-code check agrees with the original pattern in 196623 comparisons,
including every UTF-16 code unit in each tag position and line-ending controls.
Commit `b6dfaf8981` separates message descriptor signatures from publication ownership.
The original alias map remains private to the publication entry module;
signature and ownership helpers receive the same original data and map.
Focused lint has zero errors and 606 warnings across the helpers and their publication callers.
Rebuilt serializer-receipt,
event-receipt,
approval-context,
and source-work consumers pass in
`${HOME}/temp/agent/pi-native-handler-publication-split-consumer-20261015.log`.
Commit `10a606eb32` separates definition validation and answer settlement from question identity ownership.
Original claim,
batch,
and evidence maps remain private to the selection owner.
Focused lint has zero errors and 463 warnings.
Rebuilt domain,
semantic-operating-policy,
and manual-review consumers pass in
`${HOME}/temp/agent/pi-native-question-selection-split-consumer-20261015.log`.
Commit `b48b22a7d6` adds named inputs around the native result converter.
Generated staging preserves the original SDK functions byte-for-byte and appends the adapter;
the adversarial source-literal test imports and executes that generated module.
Package type checking and rebuilt program-value,
native-return,
program-queue,
and domain consumers pass in
`${HOME}/temp/agent/pi-native-conversion-adapter-types-20261015.log` and
`${HOME}/temp/agent/pi-native-conversion-adapter-consumer-20261015.log`.
The first focused converter report has zero errors and 25 warnings;
the fixture expectation now retains original text directly instead of parsing it back to an unchecked value.
Commit `18bc909876` separates event-receipt assertions under the original shared native fixture.
The first concurrent lint,
build,
and commit attempt encountered resource-allocation failures before assertions;
the separate diagnostics and recovery are recorded in `doc/troubleshooting/pi-sdk-staging.md`.
The sequential rebuild passes the original event suite and revised generated converter test in
`${HOME}/temp/agent/pi-native-event-receipt-split-retry-consumer-20261015.log`.
The package type check also passes after the fixture expectation change.
The event lint retry reports zero errors and 250 warnings across its source set and the converter test.
Commit `b42541c5bf` separates domain fixture construction,
transport controls,
authored assessment answers,
and source assertions.
Its first focused lint reports zero errors and 602 warnings.
The initial rebuilt run failed before assertions with the already-documented generated-manager import cycle.
Commit `92a305735c` enters through the maintained policy module before generated manager assertions.
The rebuilt domain,
unconfigured-policy,
semantic-operating-policy,
single-call,
changed-dependency,
next-decision-receipt,
serializer-receipt,
and context-receipt consumers then pass in
`${HOME}/temp/agent/pi-native-domain-fixture-entry-consumer-20261015.log`.
The failed original log remains in `pi-native-domain-fixture-split-consumer-20261015.log`.
The full maintained recount covers 513 files and 486 rules:
16 errors and 13771 warnings remain in
`${HOME}/temp/agent/pi-maintained-domain-splits-lint-20261015.log`.
Remaining errors are in the relevance store,
its cache owner,
the composed-distribution protocol,
and the descriptor-bound file observer.
The relevance store now awaits directory validation and file publication during cache operations;
its constructor still returns the cache owner synchronously.
Commit `2d804201df` introduces the asynchronous cache filesystem operations;
`8b97abd75a` joins owner-local same-key publications before reading their completed entry.
Commit `32d3e551d4` adds built store consumers using disposable directories.
Those consumers pass private modes,
original-value retention,
concurrent publication,
symlink rejection,
malformed and mismatched persisted data,
digest filename boundaries,
and invalid probability rejection in
`${HOME}/temp/agent/pi-native-relevance-store-consumer-20261015.log`.
Package type verification passed.
The focused lint pass found an inferred absent promise in initialization
and a nullable test contract.
Initialization now declares its pending operation;
the consumer treats read values as unknown and compares them with the original complete entry.
The built refresh consumers in `99ecff732d` pass reuse across owners,
changed-rule membership,
recursive refresh joining,
source replacement during classification,
and source replacement or cancellation during cache hits and publication.
`${HOME}/temp/agent/pi-native-relevance-refresh-consumer-20261015.log`
retains those results and the initial fixture declaration failure.
Commit `fb56538d9e` adds the required explicit fixture return type;
package type verification then passed in
`${HOME}/temp/agent/pi-native-file-observer-baseline-20261015.log`.

The built file-observer baseline in the same receipt passes completed synchronous observations,
short reads,
descriptor cleanup,
ordered original read and close failures,
invalid counts,
byte bounds,
and selected-path rejection.
Commit `0df033dd92` replaces its `finally` with a lexical descriptor owner.
The exact synchronous filesystem calls retain scoped exceptions documented in
[the file-observation contract](../../package/pi-plugin/auto-mode/native/file-observation.md).
The rule has no per-call configuration allow-list;
changing the observer alone to return asynchronously would change the synchronous policy-selection
and dependency-finalization consumer contracts.
The package rules and their severities remain unchanged.
The rebuilt file observer,
relevance store and refresh,
source-freshness work,
approved-dependency integration,
and package type checks all pass in
`${HOME}/temp/agent/pi-native-cache-observer-consumer-20261015.log`.
Process-local omission controls remove descriptor closure or post-await cache freshness checks
from the copied modules while loading.
The committed consumer tests fail in both cases,
and artifact hashes remain unchanged.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/cache-observer-omissions.json`
retains those results.
Focused lint now has zero errors and 484 warnings across the changed source and fixtures.
The complete package recount covers 517 files and 486 rules:
zero errors and 13812 warnings in
`${HOME}/temp/agent/pi-maintained-cache-observer-lint-20261015.log`.
Remaining warnings still prevent the strict lint task from passing.
Untyped native module boundaries and statement formatting remain the largest unfinished groups.
Commit `bf9eee5abc` moves the selected-file observer from `file-observer.mjs` to `file-observer.ts`,
with explicit completed-snapshot and bounded-reader contracts in `file-observation-types.ts`.
Its direct consumers and provenance mapping now select the typed source.
Commit `a6e2b905a7` names the selected-path revalidation boundary while retaining its original checks and order.
The typed port passes the rebuilt observer,
store,
refresh,
source-freshness,
approved-dependency,
and package type checks in
`${HOME}/temp/agent/pi-native-typed-observer-consumer-20261015.log`.
That scratch runner then reached its lint command outside the repository
and reproduced the documented missing-monorepo-root diagnostic.
The separate lint task now pins its repository directory;
its first pass found only a path-condition line-layout warning.
That layout is corrected;
final focused lint passes with zero errors and warnings across the typed observer and new cache/observer fixtures in
`${HOME}/temp/agent/pi-native-typed-observer-complete-lint-20261015.log`.
The next typed boundary is structural policy indexing.
Its original copied JavaScript module is retained under
`${HOME}/temp/agent/pi-tool-call-scope-20261014/policy-index-before-types.mjs`
for direct comparison with the typed artifact before qualification.
Commit `f8db05b2ca` moves `policy-index.mjs` to `policy-index.ts`
and adds explicit structural-index contracts.
Commit `c1bd9b7bad` passes original coordinate values to the span checker
and uses a bounded ASCII tag scan instead of spreading source text.
The formatted index and its contracts pass focused lint with zero errors and warnings in
`${HOME}/temp/agent/pi-native-typed-index-final-format-20261015.log`.
The rebuilt relevance-cache,
domain,
semantic operating-policy,
manual-review,
and package type checks pass in
`${HOME}/temp/agent/pi-native-typed-index-consumer-20261015.log`.
The built old/new comparison matches complete index output or original diagnostic fields for
12 real-parser documents and six invalid parser trees.
It includes headings,
unclassified text,
quoted and fenced examples,
astral characters,
literal tag boundaries,
duplicate tags,
and invalid source positions.
A changed-source control produces different rule handles and index output.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-policy-index-comparison.json`
retains both artifact hashes and the results.
The complete lint recount after the typed observer and index ports covers 519 files and 486 rules:
zero errors and 13665 warnings in
`${HOME}/temp/agent/pi-maintained-typed-observer-index-lint-20261015.log`.
The strict package gate remains failing on warnings;
continue typing the remaining native boundaries and resolving their formatting diagnostics.
Commits `25b8a629f7`,
`1ee94a2029`,
and `18725748e3` type the relevance definition,
persisted-entry admission,
and asynchronous store owner.
The original `store.mjs` is now `store.ts` as of 2026-10-15;
`definition.mjs` is now `definition.ts` on the same date.
Cache misses now use the explicit `CACHE_ENTRY_MISSING` domain sentinel,
and the refresh caller narrows that sentinel before reading an entry.
The persisted schema and fixed classifier wording remain unchanged.
Parsed non-object values now receive the cache's invalid-entry diagnostic.
If an existing file disappears between exclusive-publication rejection and reuse,
publication now rejects with a cache error instead of returning an absent entry.
Commit `3f4cb12020` covers parsed primitive and array values,
stable miss identity,
that removal boundary,
and publication recovery after the rejection.
The initial focused formatter reported ten warnings and one missing return-type error;
`6b0be39a70` documents helper exports,
names fixed bounds,
and adds the disposer return type.
The rebuilt store,
refresh,
and domain consumers pass in
`${HOME}/temp/agent/pi-native-typed-store-consumer-20261015.log`.
That run's package type check found `TS9010` and `TS9013` on the fixed question export;
`e88603cef2` adds its explicit contract.
Type checking then passes in
`${HOME}/temp/agent/pi-native-typed-store-types-20261015.log`.
The typed definition,
store,
filesystem admission,
contracts,
and store consumer tests pass focused lint with zero warnings and errors in
`${HOME}/temp/agent/pi-native-typed-store-final-format-20261015.log`.
The final native artifact is rebuilt in
`${HOME}/temp/agent/pi-native-typed-store-final-build-20261015.log`.
Removing the new publication guard through a process-local load hook makes the committed removal test fail;
the artifact hash stays unchanged.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-store-publication-omission.json`
retains that result.
The complete maintained lint recount covers 521 files and 486 rules:
zero errors and 13583 warnings in
`${HOME}/temp/agent/pi-maintained-typed-store-lint-20261015.log`.
Commits `1e54dbf7d6`,
`fbebc54443`,
`65ccb967e6`,
and `6fa1a6dce3` type the relevance refresh owner,
materialized rule keys,
untrusted answer admission,
and source-bound publication work.
The original `cache.mjs` is now `cache.ts` as of 2026-10-15.
The completion handle is still registered before injected callbacks can recursively refresh.
The final source and cancellation checks still precede synchronous owner publication without a yield.
Commit `1f81821698` captures the original caller signal once and excludes later caller-option changes.
Focused formatting of the typed refresh sources passes with zero warnings and errors in
`${HOME}/temp/agent/pi-native-typed-refresh-format-20261015.log`.
Commit `aba3229861` extends built consumers with complete malformed-answer membership,
no partial publication,
retained failed-job identity,
ordered original work and reporting failures,
single signal capture,
and source replacement or cancellation during classification and final view preparation.
The rebuilt store,
refresh,
domain,
semantic-policy,
manual-review,
source-freshness,
and package type sequence passes in
`${HOME}/temp/agent/pi-native-typed-refresh-consumer-20261015.log`.
The expanded fixture and consumer tests also pass focused lint with zero warnings and errors in
`${HOME}/temp/agent/pi-native-typed-refresh-fixture-format-20261015.log`.
Process-local omissions of the final source check,
final cancellation check,
or complete answer admission each make the committed consumers fail.
The original built refresh hash is unchanged;
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-refresh-omissions.json`
retains those results.
The complete maintained lint recount covers 524 files and 486 rules:
zero errors and 13465 warnings in
`${HOME}/temp/agent/pi-maintained-typed-refresh-lint-20261015.log`.
Commit `dcd7a9c176` types fixed-question answer admission and its error category.
The original `selection-values.mjs` and `selection-error.mjs` are now
`selection-values.ts` and `selection-error.ts` as of 2026-10-15.
The reader retains scalar bounds,
original choice-object identity,
distribution freezing without normalization,
and descriptor-based serving-model attribution.
Commit `6e65184e92` adds built consumers for valid and rejected answers;
`3b774e9be8` corrects their callback signatures to the harness's asynchronous contract.
The fixture now describes its consumer contract independently of maintained source imports,
because the artifact-import lint rule also rejects source imports used only for types.
The rebuilt answer,
request-claim,
domain,
semantic-policy,
manual-review,
and package type sequence passes in
`${HOME}/temp/agent/pi-native-typed-answers-consumer-20261015.log`.
The original and built typed readers match values,
diagnostic fields,
freezing,
and retained choice identity across 59 cases.
A changed-choice control changes success to rejection;
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-answer-comparison.json`
records the artifact hashes and cases.
Process-local omissions of the serving-model byte bound,
complete distribution check,
and scalar upper bound each make the committed consumers fail.
The built artifact remains unchanged;
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-answer-omissions.json`
records those controls.
Final focused formatting reports no diagnostics in the typed readers or answer tests;
the independent fixture correction also passes with zero warnings and errors in
`${HOME}/temp/agent/pi-native-typed-answers-fixture-format-20261015.log`.
The full maintained lint recount covers 528 files and 486 rules:
zero errors and 13357 warnings in
`${HOME}/temp/agent/pi-maintained-typed-answers-lint-20261015.log`.
Warnings still fail the strict package gate.
Commit `52d3470048` types fixed question-definition catalog admission;
`selection-definitions.mjs` is now `selection-definitions.ts` as of 2026-10-15.
Commits `d5adb29fd4` and `e77dbd1a86` retain scalar-specific rubric keys
and freeze each admitted predicate through its original kind.
The original byte digest,
catalog limits,
diagnostics,
definition order,
scalar rubric order,
and independent per-definition identity indexes remain explicit contracts.
Commit `59a3c7d809` adds built consumer coverage for those contracts and rejected catalog shapes.
The typed catalog and contracts pass focused lint with zero warnings and errors in
`${HOME}/temp/agent/pi-native-typed-definitions-format-20261015.log`;
the new catalog consumer tests also pass with zero diagnostics in
`${HOME}/temp/agent/pi-native-typed-definitions-tests-format-20261015.log`.
The rebuilt catalog,
answer-value,
request-claim,
domain,
and package type sequence passes in
`${HOME}/temp/agent/pi-native-typed-definitions-consumer-20261015.log`.
The original and built typed catalog readers match across 33 cases,
including definition values and order,
criteria order and freezing,
independent identity indexes,
original diagnostics,
and parse-error cause categories.
A changed digest changes success to rejection;
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-definition-comparison.json`
retains the comparison and artifact hashes.
Omitting the pinned-byte rejection,
definition-count rejection,
or choice-option-count rejection makes the committed consumer tests fail.
The built artifact remains unchanged;
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-definition-omissions.json`
retains those controls.
The full maintained lint recount covers 530 files and 486 rules:
zero errors and 13328 warnings in
`${HOME}/temp/agent/pi-maintained-typed-definitions-lint-20261015.log`.
The warning gate remains open.
Before continuing native type cleanup,
revisit the documented direct generated-manager and generated-SDK bootstrap cycle,
which earlier fixtures bypassed by entering through the maintained policy module.
The current built manager reproduces the same initialization failure on Node 26.11.1:
`reproduce-direct-manager-import` in `${HOME}/temp/agent/pi-tool-call-scope-20261014`
exits with `ReferenceError: Cannot access 'SessionManager' before initialization`
at `native-host/host.ts:160`,
called by `native-host/bootstrap.ts:236`.
`${HOME}/temp/agent/pi-native-direct-manager-import-before-20261015.log`
retains the exact diagnostic.
The standalone probe asserts callable manager exports after import,
so it currently fails before reaching those assertions.
Commit `bdee84f697` adds committed fresh-process regressions for both generated entry orders.
Both fail with the same manager initialization diagnostic in
`${HOME}/temp/agent/pi-native-direct-import-regression-before-20261015.log`.
Commit `63be621a7d` separates the native policy entry from extension initialization.
The existing bundler emits both entries together so shared policy identities remain common,
and the native generator selects the package's `./native-policy` export.
This removes the extension SDK import from the intended native policy dependency graph;
the built import regressions and complete native host lifecycle checks now pass.
Host construction and its original method capture remain unchanged.
The checks also compare every native policy export with the extension's export by identity,
and exercise manager constructors,
CLI RPC state and clean shutdown,
serializer controls,
domain contrasts,
and authored approval decisions without provider requests.
`${HOME}/temp/agent/pi-native-policy-entry-consumers-20261015.log` retains the completed consumer results.
Its final type check failed with `TS9037` because the config default export lacked an explicit type.
Commit `6ddeaef3eb` adds that type and removes both policy-preload fixture workarounds.
The rebuilt direct-import identity,
publication,
domain,
and package type checks pass without either preload in
`${HOME}/temp/agent/pi-native-policy-unordered-consumers-20261015.log`.
The original standalone manager probe now passes in
`${HOME}/temp/agent/pi-native-direct-manager-import-after-20261015.log`.
The native policy entry,
generator,
declarations,
regression,
and build config have zero errors and one `tsdoc(multiline-blocks)` warning in
`${HOME}/temp/agent/pi-native-policy-entry-final-lint-20261015.log`.
The added config comment needs multiline formatting;
the earlier zero-warning statement in `12a4a8a00c` was incorrect.
Commit `6701c3937d` expands that comment.
The repeated focused lint now passes with zero warnings and errors in
`${HOME}/temp/agent/pi-native-policy-entry-clean-lint-20261015.log`.
The full maintained recount before that comment correction covers 532 files and 486 rules:
zero errors and 13325 warnings in
`${HOME}/temp/agent/pi-maintained-policy-entry-lint-20261015.log`.
The direct generated import cycle is resolved;
continue the remaining native type cleanup at question settlement and selection.
Commit `2012634ade` ports question settlement to TypeScript and separates complete answer decoding
from original claim publication.
Commit `64d3775711` adds built-consumer checks for original answer identity,
complete membership,
malformed responses,
UTF-8 bounds,
model attribution,
and predecode/postdecode lifecycle failure.
The first focused lint exposed explicit nullish unions in the original absent-valued fields.
Commit `e4b1d58d52` changes missing raw diagnostics and private initial answer/record fields to optional properties.
This is an explicit record-shape change:
`raw` is absent when no bounded raw string is retained,
and initial private claims and batches omit their absent `answer` and `record` fields.
Original thrown failure values still occupy the `failure` field.
The public inspection methods continue to expose their existing answer and record projections.
The direct `discard` owner also omits absent raw text.
The rebuilt settlement consumers,
request claims,
domain contrasts,
authored approvals,
and package type check pass in
`${HOME}/temp/agent/pi-native-typed-settlement-consumer-20261015.log`.
The old and new settlement implementations agree across 37 authored cases on returned answer values,
original errors,
freeze state,
claim-to-answer identity,
and terminal transitions,
with the documented absent-property change checked separately.
A changed-estimate positive control proves the comparison observes answer differences.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-settlement-comparison.json`
retains that comparison and artifact hashes.
The added extra-answer and object-backed-array cases pass focused formatting without diagnostics.
The final settlement consumer and type check pass in
`${HOME}/temp/agent/pi-native-typed-settlement-final-controls-20261015.log`.
Removing the final budget checkpoint,
current-binding check,
answer membership rejection,
model-attribution rejection,
or raw byte bound makes the committed tests fail.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-settlement-omissions.json`
records those process-local controls and unchanged artifact hashes.
The full maintained lint recount covers 536 files and 486 rules:
zero errors and 13258 warnings in
`${HOME}/temp/agent/pi-maintained-typed-settlement-lint-20261015.log`.
All settlement implementation and consumer files have zero diagnostics in that report.
The remaining warning gate is still open;
the next boundary is the original question-selection owner in `selection.mjs`.
Commits `0a7ae510e6`,
`1c0bca211a`,
`8c7922a5b8`,
and `1a66229fc5` port that owner to typed ownership,
declaration,
issuance,
inspection,
and evidence modules while preserving the original entry path.
The first focused check found an overlapping owner callback type that hid complete batch fields;
`a42e76330f` replaces the inherited callback type explicitly.
The rebuilt public selection consumers pass in
`${HOME}/temp/agent/pi-native-selection-owner-tests-20261015.log`;
the package type check passes in
`${HOME}/temp/agent/pi-native-selection-owner-types-20261015.log`.
The committed tests cover canonical claim and evidence identity,
terminal failure retention,
foreign handles,
ordinary dense selection arrays,
original resolver identity,
reentry recovery,
declaration and batch caps,
the exact four MiB UTF-8 request boundary,
and the distinction between retained diagnostics and current evidence.
Formatting and descriptor-admission corrections are complete through `6b47e03aa8`.
The focused lint reports zero diagnostics across all 12 selection source and consumer files in
`${HOME}/temp/agent/pi-native-selection-owner-final-lint-20261015.log`.
The saved original owner passes the same committed consumer tests through a process-local entry replacement in
`${HOME}/temp/agent/pi-native-selection-owner-baseline-tests-20261015.log`.
The old and new owners agree across 19 authored lifecycle scenarios on values,
errors,
checkpoint and resolver counts,
freeze state,
and retained identities.
Both use the same already-qualified catalog and settlement helpers;
the comparison isolates the selection-owner port.
A changed-estimate positive control proves that comparison observes answer differences.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-selection-owner-comparison.json`
retains the comparison and artifact hashes.
The rebuilt selection and settlement consumers,
request claims,
domain contrasts,
authored manual approvals,
and package type check pass in
`${HOME}/temp/agent/pi-native-typed-selection-consumers-20261015.log`.
Removing reentry,
binding currentness,
canonical binding-record identity,
final issuance lifetime,
declaration count,
batch count,
wire bytes,
ordinary-array admission,
descriptor membership,
canonical claim evidence,
or canonical batch evidence guards makes the committed tests fail.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-selection-omissions.json`
records all 11 process-local omissions and unchanged artifact hashes.
The maintained lint recount covers 547 files and 486 rules:
zero errors and 13007 warnings in
`${HOME}/temp/agent/pi-maintained-typed-selection-lint-20261015.log`.
All selection source and consumer files have zero diagnostics in that full report.
The warning gate remains open;
continue typing the original judgment consumers and their retained owner boundaries.
The next port is original governing-rule binding and source-set ownership.
Commits `feefb247f9`,
`f22e014e5d`,
`ab39c3a8a6`,
and `8bef59cdd6` introduce typed source projections,
structural indexing,
candidate capture,
heading retention,
and per-source/set lifetime operations behind the unchanged original entry paths.
The first focused binding check has zero errors and four formatting warnings.
The expanded set check identified callback type loss after `Array.isArray` narrowing;
the occurrence callback now explicitly retains the original source type.
The built consumer checks cover exact source intervals and digests,
native JSON representation,
permanent invalidation,
duplicate source-code ambiguity,
and original source/clause limits.
Their rebuild and broader consumer verification pass in
`${HOME}/temp/agent/pi-native-governing-bindings-consumers-20261015.log`.
An added constructor-category regression in `8cd0103039` then exposed a porting error:
the new error class's literal name replaced the original derived-constructor category.
The committed consumer fails with `AssertionError [ERR_ASSERTION]`,
actual `GoverningRuleBindingError` versus expected `TypeError`,
in `${HOME}/temp/agent/pi-native-governing-error-category-before-20261015.log`.
The saved original binding and set implementations pass the same complete consumer tests in
`${HOME}/temp/agent/pi-native-governing-baseline-consumers-20261015.log`.
The attempt to retain constructor-derived naming in `55ebbbcda2`
then triggered `unicorn(custom-error-definition)`.
The installed Oxlint 1.88.0 source explicitly requires a literal class name for minification stability.
The owner adopts a stable `GoverningRuleBindingError` operation category as an explicit unpublished design change;
derived construction no longer supplies the category automatically.
The current consumer asserts that deliberate difference while preserving original message and cause identity.
No lint rule or suppression changed.
The [source-audited troubleshooting entry](../troubleshooting/oxlint-custom-error-name.md)
records the deciding source,
failing and passing forms,
and rejected preservation attempt.
All 14 governing-source implementation and consumer files now have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-native-governing-bindings-final-lint-20261015.log`.
The final rebuild,
constructor-contract consumer verification,
and package type check pass in
`${HOME}/temp/agent/pi-native-governing-bindings-final-consumers-20261015.log`.
The saved baseline and typed implementation agree across 15 binding cases and 11 source-set cases on values,
errors,
reader/parser call counts,
freeze state,
and retained identities.
A changed-source positive control demonstrates the comparison observes changed output.
The deliberate derived-error category change is checked separately.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-governing-comparison.json`
retains the comparison and source/artifact hashes.
Removing source identity,
permanent source invalidation,
digest agreement,
per-source clause count,
final construction currentness,
source-list identity,
permanent set retirement,
unique source occurrence,
source-list count,
combined clause count,
or duplicate-code ambiguity guards makes the committed consumers fail.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-governing-omissions.json`
records all 11 process-local controls and unchanged built artifact hashes.
The complete maintained lint report covers 559 files and 486 rules:
zero errors and 12834 warnings in
`${HOME}/temp/agent/pi-maintained-typed-governing-lint-20261015.log`.
The structured report independently confirms zero diagnostics in all 14 migrated source and consumer files:
`${HOME}/temp/agent/pi-maintained-typed-governing-json-lint-20261015.log`.
Native source-input location serialization used by policy request construction is the next typed boundary.
Commits `71200e9fc7`,
`97de715341`,
and `6dad42e12c` preserve ordinary-record admission,
exact native slot shapes,
and original property-read ordering behind the existing entry path.
The error category follows the documented stable-name design decision.
The first focused check rejected a switch and a positional function declaration;
the owner now retains the original ordered branch checks and an inline sorting callback.
`ed4316f871` adds built consumers for every native slot,
missing and hidden properties,
accessor rejection,
safe ordinal boundaries,
fresh copies,
and opaque companion strings.
The focused report has zero diagnostics across all nine source and consumer files in
`${HOME}/temp/agent/pi-native-input-location-format-third-20261015.log`.
The rebuilt consumers,
request claims,
domain contrasts,
authored manual approval paths,
and package type check pass in
`${HOME}/temp/agent/pi-native-input-location-consumers-20261015.log`.
The saved original implementation also passes the consumer suite,
with only the derived-error category assertion adapted to its explicit historical behavior,
in `${HOME}/temp/agent/pi-native-input-location-baseline-20261015.log`.
The baseline and typed implementation agree across 360 authored cases on copied values,
errors,
descriptor events,
property-read order,
and freeze state.
The changed-value positive control distinguishes a different original named-text slot value.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-input-location-comparison.json`
retains the comparison and artifact hashes.
`77854a77ff` adds null-source and disappearing-proxy-descriptor controls.
Their final rebuild,
consumer tests,
and package type check pass in
`${HOME}/temp/agent/pi-native-input-location-final-consumers-20261015.log`.
The expanded baseline suite passes in
`${HOME}/temp/agent/pi-native-input-location-baseline-expanded-20261015.log`.
All 15 process-local omissions of descriptor,
record,
key,
slot-shape,
supported-field,
safe-integer,
and nonnegative-ordinal guards fail the committed consumers.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-input-location-omissions.json`
records those controls and unchanged artifact hashes.
The maintained lint recount covers 567 files and 486 rules:
zero errors and 12751 warnings in
`${HOME}/temp/agent/pi-maintained-typed-input-location-lint-20261015.log`.
All nine input-position source and consumer files have zero diagnostics in that full report.
`e813dc328c` moves original judgment input admission behind typed JSON and construction-error owners.
`cd9ff435c0` and `6e2d274a47` add consumer coverage for preserved values,
undefined and lossy numeric values,
array holes,
native cyclic and bigint errors,
callback ordering,
and original cause identity.
The checker retains its limited role after structural admission;
it does not become a general validator for callable or symbol values.
All four source and consumer files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-native-judgment-input-verified-lint-20261015.log`.
The owned rebuild,
new consumers,
request claims,
domain contrasts,
and type check pass in
`${HOME}/temp/agent/pi-native-judgment-input-consumers-20261015.log`.
The original implementation passes the same consumer suite with its historical derived-error name assertion in
`${HOME}/temp/agent/pi-native-judgment-input-baseline-20261015.log`.
The saved original and typed implementations agree across 54 cases on values,
errors,
and callback events;
a negative-zero positive control distinguishes rejected and admitted numeric input.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-judgment-input-comparison.json`
retains that evidence and artifact hashes.
Removing missing-value,
nonfinite-number,
or negative-zero checks fails the committed tests;
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-judgment-input-omissions.json`
records the controls and unchanged artifact hashes.
The complete maintained lint recount covers 570 files and 486 rules:
zero errors and 12726 warnings in
`${HOME}/temp/agent/pi-maintained-typed-judgment-input-lint-20261015.log`.
All four judgment-input source and consumer files have zero diagnostics in that full report.
Instruction-claim construction now routes through the typed owner in `4eba8033e8`.
`deabf144fd` separates structural adoption selection from later member authentication in its types;
authority-consuming types retain their authenticated-object defaults.
`be7a0a8e34` and `dde93c7164` retain original source selection,
canonical clause/request/member handles,
and dependency resolution.
`e8c613b9cf` preserves original bootstrap property reads once in their incumbent order.
The constructor-read control also verifies that construction does not invoke lazy capabilities.
Built source,
member,
request,
and batch consumers pass after the owned rebuild;
package type checking passes in
`${HOME}/temp/agent/pi-instruction-consumers-second-20261015.log`.
The existing whole-request consumer and native permission,
prohibition,
and unspecified-policy cases passed in the preceding broad consumer run.
The source and bridge format pass covers 13 files with zero diagnostics in
`${HOME}/temp/agent/pi-instruction-format-fourth-20261015.log`.
All five independent consumer files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-instruction-tests-final-lint-20261015.log`.
The saved original remains
`${HOME}/temp/agent/pi-tool-call-scope-20261014/judgment-instructions-before-types.mjs`,
with SHA-256 `202e8affd3aedada221270f4c8d5b4ac9f91844dec57cb32fa53107910db4559`.
All three committed instruction consumer suites also pass against the saved original implementation in
`${HOME}/temp/agent/pi-instruction-baseline-20261015.log`.
A process-local request-identity guard omission makes the built consumer fail its expected rejection assertion in
`${HOME}/temp/agent/pi-instruction-positive-control-20261015.log`;
the test reaches the changed instruction resolver.
The saved original and built typed owners match across 412 observed calls in six scenarios,
including object identity,
own keys,
frozen state,
errors,
and callback order.
Replacing the request occurrence distinguishes the trace as a positive control.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-instruction-comparison.json`
records that comparison.
All 25 process-local guard controls fail committed consumers;
`${HOME}/temp/agent/pi-tool-call-scope-20261014/typed-instruction-omissions.json`
records the omitted guards and unchanged built-artifact hashes.
The failed-publication control causes the recovery consumer to reject the prematurely retained handle;
the other controls fail assertion expectations.
These controls cover observable admission,
identity,
retirement,
publication,
segment,
and batch limits;
they do not establish reachability of every redundant internal projection-identity check.
Whole-request,
domain,
manual-review,
structural adoption,
and adoption-consumption tests pass in
`${HOME}/temp/agent/pi-instruction-broad-20261015.log`.
The complete maintained lint recount covers 585 files and 486 rules:
zero errors and 12269 warnings in
`${HOME}/temp/agent/pi-maintained-typed-instruction-lint-20261015.log`.
All 18 implementation,
bridge,
and consumer files touched by this port have zero diagnostics in that full report.
The instruction-owner port is qualified.
Original program ownership in `judgment-programs.mjs` (547 warnings in that report)
now routes through typed owner modules in `d14153d57f`.
`0dce710cd3` retains declaration and native return contracts;
`bc985d13f4`,
`db749fa089`,
and `362f24ef07` separate custody,
opening,
reached groups,
native calls,
value access,
and retained inspection.
The original source is saved as
`${HOME}/temp/agent/pi-tool-call-scope-20261014/judgment-programs-before-types.mjs`,
with SHA-256 `3744591589d89ba1aa3586b10cea037374310a9f9b177d4b24a225f3bbe67b83`.
The operation contracts compile in
`${HOME}/temp/agent/pi-program-owner-types-second-20261015.log`.
`722dae4094` prevents native array admission from widening already declared elements to `any`.
The native README records the scoped nullish-return alias for the original `preparedProgramFor` API;
no lint configuration was loosened.
Private uninitialized record slots now begin absent;
public retained inspection still includes its original undefined-valued fields.
The owned rebuild,
native-return consumers,
single-call consumers,
domain contrasts,
program queue,
and package type check pass in
`${HOME}/temp/agent/pi-program-owner-consumers-first-20261015.log`.
All 17 implementation and bridge files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-program-owner-format-third-20261015.log`.
The independent program owner,
native group,
result,
and publication-boundary suites pass in
`${HOME}/temp/agent/pi-program-owner-direct-third-20261015.log`;
their package type check passes in
`${HOME}/temp/agent/pi-program-owner-test-types-third-20261015.log`.
Those suites also pass against the saved original in
`${HOME}/temp/agent/pi-program-owner-baseline-20261015.log`.
The original native converter turns an empty content array into supported empty text;
the first result fixture incorrectly expected rejection.
The corrected rejection supplies non-string structured output from a tool declaring an output schema,
and a separate positive test confirms empty text can populate successor arguments.
After adding opening preflight cases and formatting,
all seven fixture and consumer files have zero lint diagnostics in
`${HOME}/temp/agent/pi-program-owner-tests-format-fourth-20261015.log`.
The final consumer and type checks pass in
`${HOME}/temp/agent/pi-program-owner-direct-final-20261015.log`
and `${HOME}/temp/agent/pi-program-owner-types-final-20261015.log`.
The final suites also pass against the saved original in
`${HOME}/temp/agent/pi-program-owner-baseline-final-20261015.log`.
Eight original-versus-typed traces match across 250 observed calls,
including values,
errors,
identities,
frozen keys,
and callback events.
The dependency-retirement positive control changes the trace.
All 28 process-local guard omissions fail their intended assertions;
the 17 built artifacts retain their original SHA-256 hashes.
Receipts are `typed-program-comparison.json` and `typed-program-omissions.json`
in `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The complete maintained lint recount covers 608 files and 486 rules:
zero errors and 11716 warnings in
`${HOME}/temp/agent/pi-maintained-typed-program-lint-20261015.log`.
All 24 program implementation,
bridge,
fixture,
and consumer files have zero diagnostics in that report.
Program ownership is qualified for the tested original consumer boundaries.
This does not establish unreachable trusted-internal misuse branches.
Continue `child-constructor.mjs` (471 warnings in the instruction-port report),
preserving existing behavior and consumer boundaries.
Its original member and execution custody now delegates to typed execution ownership in
`fc923515e3` and `ccee1949bf`.
The saved original is
`${HOME}/temp/agent/pi-tool-call-scope-20261014/child-constructor-before-execution-types.mjs`.
The typed modules preserve shared member records,
once-only execution reservation,
immediate-parent entry checks,
and cancellation-independent terminal inspection.
Execution origin types now describe the complete original identity record.
The five implementation files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-execution-format-third-20261015.log`.
Independent built consumers now cover original identity and check order,
foreign handles,
successful closure,
dependency and cancellation failures,
active children after parent return,
and retained terminal state.
Their first type check exposed fixture callable narrowing,
the harness's required asynchronous callbacks,
and stale assertion narrowing across a mutating call;
the fixture corrections are committed.
The rebuilt domain,
single-call,
program-queue,
and program-owner consumers pass in
`${HOME}/temp/agent/pi-execution-consumers-second-20261015.log`.
The direct owner and lifetime suites pass in
`${HOME}/temp/agent/pi-execution-direct-first-20261015.log`;
the final type check passes in
`${HOME}/temp/agent/pi-execution-types-final-20261015.log`.
All three independent consumer files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-execution-tests-format-second-20261015.log`.
Six complete domain scenarios produce identical public outcomes and native execution counts
with the saved original constructor and the typed extraction.
Permission versus prohibition is the positive control;
this comparison does not establish per-call trace equivalence.
The original constructor's SHA-256 is
`a848a34bf40013014aeca7eb9eed41f6379a6fc5449847efad47e0137e1d414b`.
All 14 process-local execution guard omissions fail their assertions,
and all six built artifacts retain their SHA-256 hashes.
The receipts are `typed-execution-constructor-comparison.json`
and `typed-execution-omissions.json`
in `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The full maintained lint recount covers 615 files and 486 rules:
zero errors and 11625 warnings in
`${HOME}/temp/agent/pi-maintained-typed-execution-lint-20261015.log`.
The constructor's remaining JavaScript paths are not yet qualified for lint completion.
Then continue the remaining constructor preparation,
assessment,
finalization,
and inspection paths.
`fd6b78e388` and `19d2e8beda` extract original response membership and prepared-member JSON validation
into typed snapshot modules.
The response still validates native arguments before rereading and serializing original content;
prepared validation still authenticates every identity before serializing any prepared input.
The original one-MiB bounds,
64-member response limit,
native serializer behavior,
and original failure messages remain the intended contract.
Independent consumers in `ebddcfd5c0` cover complete frozen identities,
serializer callback order,
invalid membership,
exact byte boundaries,
original primitive prepared inputs,
and serializer failures.
The owned build,
type check,
four built snapshot suites,
domain consumers,
and execution suites pass in
`${HOME}/temp/agent/pi-membership-consumers-first-20261015.log`.
All four snapshot suites also pass against the exact response and prepared validation blocks
taken from the saved original constructor in
`${HOME}/temp/agent/pi-membership-original-blocks-20261015.log`.
Those process-local comparison wrappers expose the original blocks without rewriting their statements.
They do not substitute the rest of the constructor.
The complete original-versus-current constructor comparison separately matches all six domain scenarios,
including native execution counts and the permission-versus-prohibition positive control.
All 17 process-local snapshot guard omissions fail their assertions;
all seven built artifacts retain their original hashes.
The receipts are `typed-membership-constructor-comparison.json`
and `typed-membership-omissions.json`
in `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.
The three implementation modules and five consumer files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-judgment-membership-format-third-20261015.log`
and `${HOME}/temp/agent/pi-membership-tests-format-third-20261015.log`.
The final mutable-group contrast and type check pass in
`${HOME}/temp/agent/pi-membership-final-20261015.log`.
The full maintained lint recount covers 623 files and 486 rules:
zero errors and 11561 warnings in
`${HOME}/temp/agent/pi-maintained-typed-membership-lint-20261015.log`.
All eight snapshot implementation and consumer files have zero diagnostics in that full report.
`child-constructor.mjs` now has 322 warnings.
Original snapshot extraction is qualified at the tested consumer boundaries.
Continue the constructor's assessment,
evidence closure,
preparation publication,
and retained inspection paths.
`512fcf8523` and `3919aeb0ec` move original assessment,
evidence capture,
and finalization into typed lifecycle owners.
The shared operation exclusion,
issued and attempted batch memberships,
original transport budget,
failure aggregation,
and retained closure record remain coupled to one judgment.
`0fdd07f189` declares exact disposal and closure-result contracts.
The saved pre-extraction constructor is
`${HOME}/temp/agent/pi-tool-call-scope-20261014/child-constructor-before-lifecycle-types.mjs`.
All five lifecycle implementation files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-judgment-lifecycle-format-third-20261015.log`.
Oxlint's `eslint(preserve-caught-error)` autofix initially added a `cause` property to an original aggregate failure.
`89b85a975e` restores the original paired-error shape through `createAssessmentRetentionError`,
keeping both failures in their original order without a new cause property.
The [source-audited troubleshooting record](../troubleshooting/oxlint-aggregate-error-cause.md)
contains the installed-version probe,
the observed own-key change,
and the verified factory alternative.
That record was rendered and inspected with Pandoc.
Owned rebuild,
type check,
domain,
single-call,
execution,
and program-owner consumers pass in
`${HOME}/temp/agent/pi-judgment-lifecycle-consumers-first-20261015.log`.
`2fdb1ed4d5`,
`84fc88caa3`,
and `5e6d67724b` add independent lifecycle fixtures and direct evidence,
assessment,
and finalization suites using the actual built selection and budget.
The first direct run caught an incorrect test expectation of 10000 milliseconds;
the original budget's `durationMs` is 5000 milliseconds,
and the built attempt supplied exactly 5000.
The assertion now matches that original fixed budget.
Type checking passes with the new fixtures.
The complete saved constructor and the current artifact match all six domain scenarios,
including native execution counts and the permission-versus-prohibition positive control.
`typed-lifecycle-constructor-comparison.json` retains the saved original's SHA-256 and both result sets.
The direct evidence and assessment suites pass against current typed modules and exact saved original function bodies.
The original-body wrapper supplies instrumented selection lookup,
dependency-handle access,
and exposed state;
it leaves extracted function statements unchanged.
Both variants caught the same finalization-test recording mistake:
its ordering assertion included the preceding issuance events.
Clear the event record after issuance before asserting closure order.
All five direct consumer files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-lifecycle-tests-lint-final-20261015.log`.
All three direct suites now pass against both the current artifact and the saved original function bodies,
as recorded in `${HOME}/temp/agent/pi-lifecycle-tests-third-20261015.log`
and `${HOME}/temp/agent/pi-lifecycle-original-blocks-final-20261015.log`.
The final type check also passes.
All 29 process-local guard omissions trigger test failures;
all six built artifacts retain their original hashes in `typed-lifecycle-omissions.json`.
The first omission verifier incorrectly required the string `AssertionError` for foreign batch acceptance;
the rejection helper correctly failed with `Expected original budget operation to reject` instead.
The corrected verifier accepts that exact failure for that control and completes the full set.
The full maintained recount covers 633 files and 486 rules:
zero errors and 11466 warnings in
`${HOME}/temp/agent/pi-maintained-typed-lifecycle-lint-20261015.log`.
All ten lifecycle implementation and consumer files have zero diagnostics in the full report.
`child-constructor.mjs` retains 227 warnings.
Original lifecycle extraction is qualified at these tested boundaries.
Continue preparation publication,
retained inspection,
and the remaining source/dependency owners.
`9ace41903a` and `33dda5e1d9` introduce typed preparation,
shared retained preparation state,
and final root-member publication.
The original complete input checks,
source seal,
dependency finalization,
selection creation,
late publication,
and failure retention remain in their original order.
The saved constructor is `child-constructor-before-preparation-types.mjs`
in `${HOME}/temp/agent/pi-tool-call-scope-20261014/`,
with SHA-256 `b093cc1bd1602b9766fd5065d75e3cd50676fc7e0ea32f25773cdca180f9c556`.
Private root records now omit unused `execution`,
`parent`,
and `program` fields until publication;
their reads remain `undefined`,
matching the existing typed member-owner representation.
This is an intentional private own-key change,
not a claim that every internal object is byte-for-byte identical.
All three preparation implementation files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-judgment-preparation-format-third-20261015.log`.
The owned rebuild,
domain,
single-call,
execution,
and program-owner suites pass in
`${HOME}/temp/agent/pi-judgment-preparation-consumers-first-20261015.log`.
`829fce2c72`,
`dbb98f4348`,
and `776f3486be` add independent built preparation consumers;
`032bdefd82` supplies their final callback contracts.
All three direct suites and type checking pass before final formatting.
All five consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-preparation-tests-format-second-20261015.log`.
The full saved-versus-current constructor comparison matches all six domain scenarios,
including native execution counts and the permission-versus-prohibition positive control,
in `typed-preparation-constructor-comparison.json`.
Final direct suites and type checking pass in
`${HOME}/temp/agent/pi-preparation-tests-final-20261015.log`
and `${HOME}/temp/agent/pi-preparation-types-final-20261015.log`.
The same suites pass against the exact saved original preparation and selection-reader function bodies in
`${HOME}/temp/agent/pi-preparation-original-blocks-20261015.log`.
That wrapper instruments source publication and lookup while leaving the extracted statements unchanged;
private optional own-key differences remain outside its equality claim.
All 24 process-local omissions trigger test failures;
all five built artifacts retain their original hashes in `typed-preparation-omissions.json`.
The initial verifier rejected an expected original missing-selection error because it required `AssertionError`;
the corrected verifier accepts that exact diagnostic for omitted selection publication,
rechecks the saved starting hashes,
and resumes the remaining controls without repeating completed runs.
The full maintained recount covers 641 files and 486 rules:
zero errors and 11401 warnings in
`${HOME}/temp/agent/pi-maintained-typed-preparation-lint-20261015.log`.
All eight preparation implementation and consumer files have zero diagnostics in the full report.
`child-constructor.mjs` retains 162 warnings.
Original preparation extraction is qualified at the tested boundaries.

`c2bcb12ea0` and `18451f1560` introduce typed current-member reads and retained judgment inspection.
Prepared-member access retains ownership,
dependency,
and immediate-parent ordering;
original-file access retains readiness before ownership and its original prepared input path.
Retained inspection keeps preparation,
closure,
accounting,
root execution,
prepared programs,
root programs,
and reached groups separate from live eligibility.
The saved constructor is `child-constructor-before-inspection-types.mjs`
in the same scratch directory,
with SHA-256 `bb100fcead79be4ad62a52867952e723d239686bd91babbbb285f5e7722e98c4`.
Both inspection implementation files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-judgment-inspection-format-second-20261015.log`.
`f88983e3a2` records the final member-access formatting.
The owned rebuild,
type check,
domain,
execution,
lifecycle,
and preparation suites pass in
`${HOME}/temp/agent/pi-judgment-inspection-consumers-first-20261015.log`.
The complete saved-constructor comparison matches six domain scenarios and native execution counts,
including the permission/prohibition positive control,
in `typed-inspection-constructor-comparison.json`.
`f98f916b0f` and `bd71ac8abf` add independent current-member and retained-snapshot consumers.
Both suites pass against the built owner and the exact saved original function statements in
`${HOME}/temp/agent/pi-inspection-tests-first-20261015.log`
and `${HOME}/temp/agent/pi-inspection-original-blocks-20261015.log`.
The original wrapper only binds the shared state and supplied capabilities;
it does not rewrite the extracted functions.
The initial type check required an explicit fixture return contract under isolated declarations;
`81a4bfd1af` adds that independent contract.
`3ecafa1806` records the final callback contracts and formatting repairs.
All four direct consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-inspection-tests-format-second-20261015.log`.
The final direct suites and type check pass in
`${HOME}/temp/agent/pi-inspection-tests-final-20261015.log`
and `${HOME}/temp/agent/pi-inspection-types-second-20261015.log`.
All 29 process-local omissions trigger test failures;
all three built artifacts retain their starting hashes in `typed-inspection-omissions.json`.
The first omission attempt caught an ambiguous indentation substring in the mutation hook;
anchoring that fragment at the line boundary selects the intended freeze operation.
The resumed verifier rechecks starting hashes and retains the completed controls.
The full maintained recount covers 647 files and 486 rules:
zero errors and 11361 warnings in
`${HOME}/temp/agent/pi-maintained-typed-inspection-lint-20261015.log`.
Original current-member and retained-snapshot extraction is qualified at these tested boundaries.
Continue the remaining source and dependency owners.

`efa1a4a8dc` and `37cd18ca8f` introduce the typed original dependency owner
and separate synchronous binding-segment lifetime.
The original request lookup remains a lazy callback to `readMainAgentRequest`;
the first dependency failure remains permanent,
and retained release remains separate from the completed decision clock.
`5345686a3e` and `0fb1733f3b` preserve paired operation/postcheck failures as ordered aggregate members
without adding an own `cause` property.
This uses the existing verified factory workaround in
[`oxlint-aggregate-error-cause.md`](../troubleshooting/oxlint-aggregate-error-cause.md).
The saved files are `child-constructor-before-dependency-types.mjs`
and `judgment-dependencies-before-types.mjs` in the same scratch directory,
with SHA-256 values
`d83cb829b3cecb380f153679cf3a4f6ddbc5ac538030c14cdf66c512f3f0baa5`
and `6828c1d2eef2b7a811d2b025369dc76d4c491d2415fa21435a17a7eba4c0b0fc`.
All three implementation files have zero focused lint diagnostics in
`${HOME}/temp/agent/pi-judgment-dependency-lint-final-20261015.log`.
The owned rebuild,
type check,
domain,
execution,
lifecycle,
preparation,
and inspection suites pass in
`${HOME}/temp/agent/pi-judgment-dependency-consumers-first-20261015.log`.
`60c1711e4c`,
`16c18151b8`,
and `bc615cba7f` add independent built dependency consumers.
Their live,
retained,
and binding suites pass before formatting,
including actual budget expiry/cancellation and original primitive or undefined throws.
Initial type checking also passes.
`ce6dd23fac` records final consumer callback boundaries and formatting.
All five consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-dependency-tests-format-second-20261015.log`.
The final direct suites and type check pass in
`${HOME}/temp/agent/pi-dependency-tests-final-20261015.log`
and `${HOME}/temp/agent/pi-dependency-types-final-20261015.log`.
The same suites pass against the saved original dependency function statements in
`${HOME}/temp/agent/pi-dependency-original-blocks-final-20261015.log`.
The wrapper supplies the original request reader through the independent fixture callback
and leaves the dependency function bodies unchanged.
The six saved-constructor scenarios match public domain outcomes and native execution counts,
including the permission/prohibition positive control,
in `typed-dependency-constructor-comparison.json`.
All 32 process-local omissions trigger assertion failures;
all four built artifacts retain their starting hashes in `typed-dependency-omissions.json`.
The full maintained recount covers 654 files and 486 rules:
zero errors and 11275 warnings in
`${HOME}/temp/agent/pi-maintained-typed-dependency-lint-20261015.log`.
Dependency eligibility and original binding-segment extraction are qualified at these tested boundaries.
Continue original source binding and the remaining constructor composition.

`d9c4ea53d8` and `af01ad50c2` introduce typed original source handoff and collection ownership.
The native preparation callback still precedes request authentication;
legacy observers retain their existing binding callback or receipt-source fallback.
The collection adapter still requires the original cancellation signal and source identity
before invoking freshness callbacks.
Preparation publication and current binding reads retain the same mutable collection lifetime.
The saved constructor is `child-constructor-before-source-types.mjs` in the same scratch directory,
with SHA-256 `0303111624e942b4d71ff419b459dda1a9ec4f7c3276317e122d7fb530c3c2b4`.
`3f722d3466` records the final source-owner formatting.
Both implementation files have zero focused diagnostics in
`${HOME}/temp/agent/pi-judgment-source-lint-final-20261015.log`.
The owned rebuild,
type check,
domain,
single-call,
dependency,
preparation,
and inspection suites pass in
`${HOME}/temp/agent/pi-judgment-source-consumers-first-20261015.log`.
The six saved-constructor scenarios match public domain outcomes and native execution counts,
including the permission/prohibition positive control,
in `typed-source-constructor-comparison.json`.
`741544d19d` and `8b1996d2f7` add independent built source consumers.
Direct suites and type checking pass in
`${HOME}/temp/agent/pi-source-tests-first-20261015.log`
and `${HOME}/temp/agent/pi-source-types-first-20261015.log`.
The same suites pass against the saved original handoff,
collector,
and current-binding statements in
`${HOME}/temp/agent/pi-source-original-blocks-20261015.log`.
These consumers retain native/legacy callback receiver and response identity,
receipt fallback,
request authentication order,
current collection and signal identity,
callback failures,
and publication during freshness or dependency observations.
`f49e36b019` records final consumer formatting.
All four consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-source-tests-format-final-20261015.log`.
Final direct suites and type checking pass in
`${HOME}/temp/agent/pi-source-final-20261015.log`.
All 24 process-local safeguard changes trigger test failures;
all three built artifacts retain their starting hashes in `typed-source-omissions.json`.
The initial control verifier required an assertion error even when an added cancellation check
correctly caused the original construction error instead.
The resumed verifier accepts that exact diagnostic for added cancellation rejection,
and the exact injected diagnostic for an added post-freshness identity check,
while rechecking starting hashes and retaining completed controls.
The full maintained recount covers 660 files and 486 rules:
zero errors and 11255 warnings in
`${HOME}/temp/agent/pi-maintained-typed-source-lint-20261015.log`.
All six source implementation and consumer files have zero diagnostics in the full report.
`child-constructor.mjs` retains 103 warnings.
Original source handoff and collection extraction are qualified at these tested boundaries.
Continue the remaining constructor composition and maintained-package quality queue.

Constructor composition now has a saved baseline in
`child-constructor-before-composition-types.mjs`,
with SHA-256 `43b0254ae670f11c8188567758ae565e2e7d9c121dc71193f02b3a731ec8d8d4`.
`a3ecae3ca8` preserves caller-owned source types through source handoff and preparation
without changing the source statements executed at runtime.
The composition audit found that the original member map includes root records with claim bindings
and reached-child records without claim bindings.
At the audit boundary,
the instruction type described only the root shape.
A built consumer checks original missing-child binding results,
root projection failure,
and recovery before the type contract was reconciled.
The source/preparation rebuild,
type check,
and direct consumer sequence pass in
`${HOME}/temp/agent/pi-source-generic-contract-first-20261015.log`.
`1434f3905c` records the mixed-membership consumer.
`4b05ea17ea` makes root claim bindings optional on shared instruction records
and limits preparation to the root-record publication capability it uses.
Type checking then identified the same root projection assumption in unit adoption (`TS18048`).
`8b6fde2bc2` shares original projection access between member clauses and unit adoption,
retaining the original missing-record and missing-binding error messages before publication.
The rebuilt source,
preparation,
and instruction consumers and type checking pass in
`${HOME}/temp/agent/pi-member-binding-contract-second-20261015.log`.
The expanded consumers also pass against the saved original instruction implementation in
`${HOME}/temp/agent/pi-member-binding-original-20261015.log`.
All eleven changed contract and consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-composition-contract-format-first-20261015.log`.
`6090a6ee8d` records final projection-helper formatting.
Final post-format rebuild,
type checking,
and source,
preparation,
and instruction consumers pass in
`${HOME}/temp/agent/pi-composition-contract-final-20261015.log`.
`3bf53fc710` moves original public assembly into typed capability contracts
while retaining constructor ownership and the final budget checkpoint.
The public surface has 55 methods.
`734054d14e` records assembly formatting.
`cf8571fe7d` and `0578a7cdd2` add independent built consumers
for every guarded and direct method,
frozen property order,
lazy retained selection,
argument cardinality,
callback and receiver identity,
and failure propagation.
The initial direct consumers pass;
fixture declaration checking reports `TS9010`,
`TS9013`,
and `TS9016` under isolated declarations.
`3378daf2f3` supplies explicit independent fixture contracts.
`e1cfc446d9`,
`7a6fa5d69b`,
and `701b7c260e` finish independently observable call fixtures and formatting.
Final rebuild,
direct consumers,
and type checking pass in
`${HOME}/temp/agent/pi-judgment-api-tests-final-20261015.log`.
The same consumers pass against original saved guard and public-assembly statements in
`${HOME}/temp/agent/pi-judgment-api-original-20261015.log`.
Six original and extracted constructor scenarios match public domain outcomes and native execution counts,
with a permission/prohibition positive control,
in `typed-api-constructor-comparison.json`.
All 64 process-local call-boundary changes cause failures;
all four built artifact hashes remain unchanged in `typed-api-omissions.json`.
The full maintained recount covers 669 files and 486 rules:
zero errors and 11243 warnings in
`${HOME}/temp/agent/pi-maintained-typed-api-lint-20261015.log`.
All eight public API implementation and consumer files have zero diagnostics;
the remaining constructor composition has 91 warnings.
Public API extraction is qualified at these tested boundaries.
Constructor composition remains pending beyond public assembly.

`1030e8cde9` preserves the actual dependency-projection result in the public API type,
types original request handoff,
and declares the existing dependency facade's composition boundary.
The facade implementation remains JavaScript;
its declaration is not evidence that the facade body has been migrated or qualified again.
The request reader still authenticates the producer before looking up the observer callback,
retaining both method receivers and exact receipt identity.
Its original saved source is `request-before-types.mjs`,
with SHA-256 `abcf9f32007082c1e32d492c6bad8302f688ac0ab797059744bf8ba5ee4e02b7`.
Independent built request consumers cover successful handoff,
producer failure before observer access,
observer failure,
and recovery.
`2b6019404d` records the initial request consumers;
`8eec1bf371` records final boundary and consumer formatting.
The boundary rebuild,
type checking,
request and API consumers,
domain scenarios,
and single-call checks pass in
`${HOME}/temp/agent/pi-composition-boundaries-first-20261015.log`.
Original request replay passes in `${HOME}/temp/agent/pi-request-original-20261015.log`.
A process-local stream replacement escaped the initial test's deep equality assertion:
both stream objects had no properties.
`a542c28ae7` requires exact stream and response identities.
All eight process-local handoff changes now cause assertion failures,
with all three built hashes unchanged in `typed-request-omissions.json`.
The tightened consumers still pass against the original request body in
`${HOME}/temp/agent/pi-request-original-identity-20261015.log`.

`2deac389d9` aligns shared constructor capabilities with their actual native operations.
Original weak membership reads accept foreign values without restricting publication to those values;
the read capability preserves primitive queries without adding a new rejection boundary.
Execution member lookup and instruction resolution retain the original map reads.
Lifecycle dependency handles remain absent until original preparation publishes them.
Retained dependency state accepts both successful and failed closure records.
Type checking passes in
`${HOME}/temp/agent/pi-composition-membership-types-first-20261015.log`.
Focused lint rejected the method-signature capability under `typescript(method-signature-style)`.
After `2053573a7c` changed it to the required function-property form,
TypeScript reported `TS2322` for the wider input assignment.
The declaration shortcut is rejected.
`9e94e6e9b7` narrows queries at the original object-keyed membership boundary instead,
returning native absence for primitive inputs and retaining original record identity for object/function keys.
This is type narrowing for existing native membership outcomes,
not a new security safeguard.
`3263cc5c58` adds an independent built consumer comparing those query results with native WeakMap reads.
The instruction lifetime suite also checks readiness before primitive rejection and subsequent owned-member recovery.
`cf0afe7889` records original readiness ordering and native membership query consumers.
The rebuild,
type checking,
membership and request consumers,
execution,
instructions,
lifecycle,
dependencies,
and public API suites pass in
`${HOME}/temp/agent/pi-composition-membership-consumers-first-20261015.log`.
All ten focused contract and consumer files have zero diagnostics in
`${HOME}/temp/agent/pi-composition-membership-format-final-20261015.log`.
The expanded instruction consumers pass against the saved original implementation in
`${HOME}/temp/agent/pi-membership-instruction-original-20261015.log`.
The full maintained count has not been refreshed since the API extraction report.
Continue typed constructor ownership and the maintained-package quality queue.

`bec0e3f129` moves the remaining original constructor into typed composition.
The saved baseline is `child-constructor-before-owner-types.mjs`.
The first type check reports `TS2322` and `TS2379`:
the composition forwards explicitly absent optional values,
and the preparation map capability exposed its full map return type despite ignoring that return.
`275292b52f` narrows publication to its actual write capability and records the optional-value correction.
Focused lint rejects explicit nullish unions,
the constructor length,
and void-only contracts for callbacks that return dependency projections.
The current correction preserves optional indexed reads and actual callback results,
and separates initial budget/source/membership observations into a typed foundation.
`c180dbeeb1` records the initial foundation split;
`62204d236b` separates completed-owner publication from constructor wiring.
Optional values now use their actual optional ownership projections;
the rejected `Partial` shortcut is absent.
`3fa070b59b` records final implementation formatting.
The rebuilt constructor type check passes in
`${HOME}/temp/agent/pi-constructor-types-third-20261015.log`.
The seven final focused implementation files have zero diagnostics in
`${HOME}/temp/agent/pi-constructor-format-third-20261015.log`.
Six original and typed constructor scenarios match public domain outcomes and native execution counts,
with a permission/prohibition positive control,
in `typed-owner-constructor-comparison.json`.
The baseline SHA-256 is `d07545d2124a477a6dc2b2d59f9c43bf2e1fd4e9368081e5c0a75474d241b984`.
`349d73e4a5` adds independent constructor consumers for original callback order,
captured bootstrap capabilities,
start time,
cancellation,
late expiry,
failure identity,
and recovery.
`91772da929` independently checks every public method's original owner at publication.
`42a9e6816a` and `bafbb59f58` finish consumer declarations and formatting.
The direct constructor and publication consumers pass,
and the same consumers pass against saved original constructor and publication statements in
`${HOME}/temp/agent/pi-constructor-original-20261015.log`
and `${HOME}/temp/agent/pi-publication-original-20261015.log`.
All four new consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-constructor-tests-format-final-20261015.log`.
All 59 process-local construction and publication changes trigger consumer failures;
all five built artifacts retain their starting hashes in `typed-constructor-omissions.json`.
The final rebuild,
type check,
constructor,
publication,
program,
execution,
instruction,
lifecycle,
dependency,
preparation,
inspection,
source,
and public API consumers pass in
`${HOME}/temp/agent/pi-constructor-consumers-final-20261015.log`.
The full maintained recount covers 685 files and 486 rules:
zero errors and 11145 warnings in
`${HOME}/temp/agent/pi-maintained-typed-constructor-lint-20261015.log`.
All eighteen changed implementation,
contract,
bridge,
and direct consumer files have zero diagnostics in that full report.
Typed original constructor composition is qualified at these tested boundaries.
Continue the maintained-package quality queue with the dependency facade body;
its composition declaration remains separate from implementation qualification.

Dependency facade migration is now in progress.
`0f97d68407` moves the body behind its stable native bridge into typed owners for
original records,
declared profiles,
and file inspection.
The commit was prepared as `5be83a89e1` and replayed after a concurrent landing.
The original source is retained in `dependencies-before-owner-types.mjs`.
The first type check reports `TS2322` for widened mechanical-projection literals;
`7fedacfcaa` retains their declared literal values and documents original lifecycle boundaries.
The first consumer build then reports `TS9017` for inferred exported fixture arrays;
those independent event contracts now have explicit readonly string-array types.
`3ade565745` adds original callback ordering,
foreign-handle rejection,
failure recovery,
and live versus retained lifetime consumers.
File consumers use disposable roots to check exact original bytes,
changed-file rejection,
controlled-reader identity,
and accepted/rejected declared profile boundaries.
Primitive handle narrowing preserves the original object-keyed membership outcome;
it is not a new authorization safeguard.
`30096381d2` completes fixture formatting.
The rebuilt consumer suite and package type check pass in
`${HOME}/temp/agent/pi-dependency-facade-final-20261015.log`.
All four consumer files also pass against the exact saved original facade body.
Six original/current domain comparisons preserve public outcomes and native execution counts;
the permission/prohibition positive control distinguishes execution from denial.
The saved original body has SHA-256
`519ef9a07af42d9749f1235d0c593589b10df5449feffc68b17d5ecd2b4ec339`.
The focused twelve-file formatting pass has zero warnings and zero errors.
File consumers now also compare the complete preparation and live revalidation event order,
so removing an individual file checkpoint cannot hide behind successful observation.
`b7429f19ad` adds those assertions;
`490490d960` places the pre-observation clock at its original position.
The rebuilt strengthened suite and exact original-body replay both pass.
All 43 process-local changes are detected,
including every preparation,
revalidation,
and file checkpoint,
original callback identities,
freshness comparison,
projection boundaries,
and configuration guards.
`typed-facade-omissions.json` retains unchanged hashes for all five built implementation/bridge artifacts.
The scratch verifier initially required `AssertionError` for every failure;
the retained-clock control correctly throws the fixture's original callback failure,
and the profile-capture control correctly throws `ObservedFileMissingError` for the replacement path.
The verifier now names those exact expected failures;
completed prefix controls and their original logs were retained.
Constructor,
judgment dependency,
preparation,
and approved-dependency consumer suites pass in
`${HOME}/temp/agent/pi-dependency-facade-consumers-20261015.log`.
The full plain-text lint recount reports 696 files,
486 rules,
zero errors,
and 11045 warnings in
`${HOME}/temp/agent/pi-maintained-typed-facade-lint-20261015.log`.
The structured reader correctly rejected that non-JSON output.
The subsequent structured recount confirms the same totals and zero diagnostics across all fourteen facade files in
`${HOME}/temp/agent/pi-maintained-typed-facade-structured-lint-20261015.log`.
The original dependency facade is qualified at these tested boundaries.
Continue the native program execution queue with original recorder scope and serial queue ownership,
then its reached-program runner composition.

`113d71975c` moves original recorder and queue ownership behind `program-scope.ts`,
with unchanged native entry paths and private token/public label publication.
`ba72ea82cf` formats that implementation;
the three-file scope has zero lint diagnostics.
`95994b387b` adds independent recorder and queue consumers.
The first fixture lint identifies a self-referential constructor inference error;
an explicit native constructor type preserves the proxy's checked receiver without unsafe inferred values.
The rebuilt artifact,
package type check,
existing native queue controls,
and new recorder/queue consumers pass in
`${HOME}/temp/agent/pi-program-scope-first-20261015.log`.
The tests retain immediate queue-tail publication,
normal and exceptional release,
idempotent disposal,
and the original unresolved successor after a predecessor rejection.
`6d81bcc321` fixes the fixture's constructor inference;
`634b29b15e` finishes its failure-case formatting.
All three fixture/consumer files now have zero diagnostics.
Original-body replay and final package type checking pass.
Twelve deliberate process-local scope and queue changes fail the intended consumers,
with both built artifact hashes unchanged in `typed-scope-omissions.json`.
Six original/current domain comparisons retain identical public outcomes and native execution counts,
including a permission/prohibition positive control.
The saved original scope body has SHA-256
`17dba680086086e2cff55295ecdca30e134bc6afb3ef7311f271ad800230b4f3`.
Original recorder and queue ownership is qualified at these tested boundaries.
The last full-package warning count remains the facade report;
no extrapolated package total is claimed for the subsequent scope migration.

GitHub rejected the automatic push of `634b29b15e` with `GH009`.
The commit remained local;
one ordinary retry succeeded,
and direct remote/local ref reads both confirmed that commit.
The [incident record](../troubleshooting/github-lfs-integrity-rejection.md)
keeps the unknown server cause separate from the verified recovery.
The user's subsequent status request was answered with the completed live host boundary,
qualified facade and scope work,
and the still-open maintained quality queue.
No additional human input is needed for this queue.

`fe97ed7482` moves original group completion retention into typed implementation and record contracts.
`ef8f58d922` adds independent native outcome,
conditional delivery,
and cleanup-failure consumers.
`6973251c83` separates readonly completion counting from mutable record ownership.
The first consumer type check rejects an empty-array assertion that narrows later event inspection to `never`;
the fixture now checks the empty count and preserves explicit string-array typing.
Opaque rejection tests use independently issued native tickets,
keeping rejection identity without treating it as an authored application error.
`571e76379d` retains opaque rejection tickets and adds strict outcome identity assertions.
The final five-file implementation/test formatting pass reports zero diagnostics.
The owned native rebuild,
package type check,
both consumer suites,
and exact original-body replay pass.
All nineteen process-local completion changes fail the expected consumer assertions,
with both artifact hashes unchanged in `typed-group-completion-omissions.json`.
The controls remove listener cleanup,
allow premature settled publication,
alter delivery states,
omit ticket settlement,
or replace retained identities.
Six original/current domain comparisons preserve public results and native execution counts,
with the permission/prohibition positive control still distinguishing execution from denial.
The saved original helper has SHA-256
`8b74e711ae1f065768149c111463b57b4d5396dec0abd977fe19fe5dcda66612`.
The original completion owner is qualified at these tested boundaries.
The full-package recount covers 707 files and 486 rules,
with zero errors and 10958 warnings in
`${HOME}/temp/agent/pi-maintained-typed-group-completions-lint-20261015.log`.
All thirteen changed scope/completion implementation,
declaration,
bridge,
fixture,
and direct consumer files have zero diagnostics in that report.
Continue original group-failure occurrence ownership,
then the remaining collector,
runtime,
and reached-program runner composition.

The next original failure-owner body is now behind `group-failures.ts`,
with dedicated occurrence/state contracts and the original native bridge.
The original helper is retained in `program-collector-failures-before-types.mjs`.
Occurrence append,
primary failure publication,
pending rejection,
and cancellation retain their original order.
Abort caching remains after callback completion,
so callback reentry and failure must be checked before qualifying the port.
The internal token now has stable diagnostic category `RecordedGroupFailureError`;
the original `RecordedFailure` inherited `Error` as its `name`.
This is an intentional unpublished diagnostic change under the existing literal error-category convention,
not an equivalence claim for the token's name or constructor identity.
The underlying occurrence's original error identity must remain unchanged.
Read-only inspection of
`${HOME}/temp/agent/oxc-oxlint188-20261015/crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs`
confirms that the rule requires the class naming convention and a literal matching name.
The [existing error-category investigation](../troubleshooting/oxlint-custom-error-name.md)
records that convention and lack of a preserving per-rule configuration.
`2439fc3802` introduces the typed owner;
`f5b768de69` formats its source.
The five-file focused report has zero diagnostics,
and the owned native rebuild and package type check pass in
`${HOME}/temp/agent/pi-group-failures-build-first-20261015.log`
and `pi-group-failures-types-first-20261015.log`.
`cc9b52248a`,
`3143a1cb05`,
and `7cbe2818ca` add independent built failure consumers.
The initial direct suites and type check pass.
The fixture now declares pending rejection at the native factory boundary,
preserving receiver observations without a positional declaration or variable function expression.
The three consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-group-failure-tests-format-third-20261015.log`.
`875e78efe6` records final callback formatting.
The rebuilt native artifact,
package type check,
and both direct consumer suites pass in
`${HOME}/temp/agent/pi-group-failures-final-20261015.log`.
Both suites also pass against the saved original failure helper in
`${HOME}/temp/agent/pi-group-failures-original-20261015.log`.
The original helper has SHA-256
`2a539ec0ddb66c0d393eddc1aebbf29771b4be345f70e206e04e531507c6556b`.
All twenty process-local changes fail their consumer checks;
all three built artifact hashes remain unchanged in `typed-group-failure-omissions.json`.
Controls cover occurrence freezing and identity,
history and primary publication,
closed/settled guards,
rejection and cancellation order,
cached signals,
and already-recorded boundary propagation.
An added rejection for uncancelled signals also fails the original consumer contract.
Six original/current domain scenarios preserve public outcomes and native execution counts,
with a permission/prohibition positive control,
in `typed-group-failure-comparison.json`.
`typed-group-failure-category.json` directly measures the intentional change from
`Error`/`RecordedFailure` to `RecordedGroupFailureError`/`RecordedGroupFailureError`
for name/constructor,
while verifying the unchanged message and underlying caught-value identity.
The failure owner is qualified at these tested boundaries.
The full maintained recount covers 714 files and 486 rules,
with zero errors and 10920 warnings in
`${HOME}/temp/agent/pi-maintained-typed-group-failures-lint-20261015.log`.
All eight changed failure implementation,
declaration,
bridge,
fixture,
and consumer files have zero diagnostics in that report.
Continue original collector composition,
then reached-program runtime and runner composition.

`ed4178bf49` extracts original collector retirement,
draining,
and retained inspection into `group-lifetime.ts` and its independent state contracts.
Arrival and dispatch still reside in the original collector body.
The saved collector baseline is `program-collector-before-lifetime-types.mjs`;
`group-lifetime-original-functions.txt` retains its exact lifetime function statements for direct replay.
The exported `DeclaredGroupCollectionError` now declares its literal diagnostic category in a typed class.
Its base constructor still requires the original message;
subclasses now retain that category instead of deriving it from their constructor name.
This is an unpublished diagnostic design change under the existing literal error-category convention.
`64b99f24ac` adds independently declared built consumers for retirement,
callback failures,
work observation,
and retained snapshot publication.
The first focused source pass found only `eslint(no-useless-constructor)`.
Assigning the original literal name in the constructor retains explicit message-only construction
without a redundant superclass-only constructor.
`b10fb56679` retains explicit diagnostic initialization;
`4e03644092` records final fixture formatting.
All six changed source and consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-group-lifetime-format-second-20261015.log`.
The owned rebuild,
package type check,
and both direct consumer suites pass in
`${HOME}/temp/agent/pi-group-lifetime-final-20261015.log`.
Both direct suites pass against the exact original lifetime statements in
`${HOME}/temp/agent/pi-group-lifetime-original-20261015.log`.
Six original/current domain scenarios preserve public outcomes and native execution counts,
including the permission/prohibition positive control,
in `typed-group-lifetime-comparison.json`.
The saved original collector has SHA-256
`f1ea72c6c995e3d9186be79a8d658d5dd15649cfe3631d8c0e736257cbf2636d`.
All twenty-nine process-local changes fail the consumer assertions;
all three built artifact hashes remain unchanged in `typed-group-lifetime-omissions.json`.
The first control run stopped because its snapshot replacement selector also matched a nested member snapshot.
After narrowing that selector,
the complete control sequence passed with a fresh artifact-hash comparison.
`typed-group-collection-category.json` directly verifies unchanged base diagnostic names and messages,
and the intentionally stable subclass category.
The original lifetime extraction is qualified at these tested boundaries.
The full maintained recount covers 720 files and 486 rules,
with zero errors and 10902 warnings in
`${HOME}/temp/agent/pi-maintained-typed-group-lifetime-lint-20261015.log`.
All six changed lifetime implementation and consumer files have zero diagnostics in that report.
Continue original collector currentness,
dispatch,
and arrival composition before the reached-program runtime and runner.

The currentness extraction now has `group-current.ts` and separate original input contracts.
It preserves the original first-failure,
closure,
cancellation,
parent-reader,
and reached-view checks in their original order.
The original collector baseline is `program-collector-before-current-types.mjs`;
its exact extracted statements are `group-current-original-functions.txt`.
`1ccc58cb0c` introduces the original currentness owner;
`1d1935e4c6` re-reads failure and closure through a shared helper after callbacks can publish state.
The helper preserves original checks while preventing TypeScript from retaining pre-callback narrowing.
`c6aaab1bbc`,
`b1a75a47dd`,
and `07336598f6` add and format independently declared currentness consumers.
All five changed implementation and consumer files have zero focused diagnostics in
`${HOME}/temp/agent/pi-group-current-lint-final-20261015.log`.
The owned rebuild,
package type check,
and both currentness suites pass in
`${HOME}/temp/agent/pi-group-current-final-20261015.log`.
Both direct suites pass against the exact original currentness statements in
`${HOME}/temp/agent/pi-group-current-original-20261015.log`.
Six original/current domain scenarios match public outcomes and native execution counts,
with the permission/prohibition positive control,
in `typed-group-current-comparison.json`.
The saved collector baseline has SHA-256
`acce762fc8e61341662b53454db84c446ce74907503796df0d3d0cada3c74707`.
All twenty-four process-local currentness changes fail the consumer checks;
both built artifacts retain their starting hashes in `typed-group-current-omissions.json`.
The verifier initially required an assertion diagnostic for an added unconditional cancellation check,
although that check correctly caused the unexpected original cancellation token to escape.
The completed verifier requires that exact token cause for this control
and assertion diagnostics for the other controls.
The original currentness owner is qualified at these tested boundaries.
The full maintained recount covers 725 files and 486 rules,
with zero errors and 10889 warnings in
`${HOME}/temp/agent/pi-maintained-typed-group-current-lint-20261015.log`.
All five changed currentness implementation and consumer files have zero diagnostics in that report.
The remaining collector body has 129 warnings;
the connected runtime has 133 and reached-program runner has 266.
The status response on 2026-10-15 confirms that the live model and approval verification are already complete,
failure and lifetime owners are qualified,
and the maintained-package quality queue remains unfinished without a current user-input blocker.
The follow-up concern about the warning count prompted a complete structured-report breakdown.
`warning-distribution-20261015.json` reconciles all 10902 warnings:
8421 unsafe-value type diagnostics,
905 multiple-statements-per-line diagnostics,
504 other formatting diagnostics,
674 strict-boolean diagnostics,
and 398 other findings.
Path classification places 10077 in native implementation files,
443 in test files,
and 382 in fixture files.
The largest individual counts are the policy-input collector (284),
reached-program runner (266),
and root lifecycle transitions (231).
Formatting alone cannot resolve the measured unsafe-value majority.
Continue the active collector extraction,
then prioritize the connected dispatch/runtime/runner contracts and remaining high-count native boundaries.
Status reports must distinguish clean extracted pieces from the still-failing package-wide warning gate.

Auto-pushing `6973251c83` was separately rejected with `GH006` and no detailed requirement.
The original local signature verified,
the current protection settings were read,
and one unchanged ordinary retry succeeded.
The incident record preserves this as a distinct unknown-cause remote rejection.

The preparation commit initially printed prepared OID `5d90f18fab`,
then the wrapper replayed it as `776f3486be` after a concurrent landing.
Its `cli-git` push-wait notice was initially misread as a timeout.
The deciding source emits that notice when waiting begins;
the completed command had no terminal push-failure diagnostic.
Direct `git ls-remote` and ancestry checks verified that both preparation commits were already on remote `main`.
No push retry,
lock removal,
or tool change was needed.
The [source-traced correction](../troubleshooting/cli-git-push-wait-notice.md)
was rendered and inspected.
The handover,
troubleshooting entry,
and native README were rendered and inspected with Pandoc.
The earlier formatting-only commit is `2728f357f`.
Its judgment-input formatting was temporarily restored to recover a passing build before this direct integration.

The native JSON lint report before direct integration contains 499 errors.
The earlier human-readable report did not print every diagnostic counted in its summary;
use structured diagnostics when allocating cleanup work.
Most current errors concern arrow functions,
mutable bindings at function or module scope,
cleanup using `finally`,
and file length.
Do not replace the removed owned-source patch layer with another formatting-sensitive mechanism.

On 2026-10-14,
the user corrected the requested Pi execution model to `hyper/gemma-4-26b-a4b-it`.
Use that provider and model for subsequent Pi work and configured-host verification.
The decision model remains Luna with the separately adopted `0.60` selected-probability cutoff.
Historical Qwen verification records retain their original model identities.

The installed `@charmland/pi-hyper-provider` version is `0.4.2`.
The existing native model store contains the exact requested Hyper model.
Pi's saved default was `opencode-go/mimo-v2.6-pro`;
Qwen had been selected explicitly by the earlier verification commands.
Pi's native settings owner has now saved `hyper` and `gemma-4-26b-a4b-it`
as the global defaults,
with every unrelated parsed setting preserved.
`${HOME}/temp/agent/pi-hyper-model-20261014/selection.json` records the change.
The isolated configured-host RPC check selected that exact provider and model without CLI overrides,
using all 17 configured packages and the existing Hyper catalogue.
The process exited zero and made zero provider requests inside its network namespace.
The check verifies selection rather than a live model response.
Its initial empty-stderr assertion was inappropriate for this configured fixture:
the deliberately blocked Synthetic startup discovery,
missing other-provider fixture catalogues,
and ordinary extension lifecycle logs produced retained diagnostics.
The original RPC result and complete stderr were inspected without replaying the completed host.
`${HOME}/temp/agent/pi-hyper-model-20261014/state-verification.json`
records the successful selection and failed empty-stderr assertion separately.

## Adopted Luna migration

The user subsequently lowered the selected-option probability cutoff from `0.80` to `0.60` on 2026-10-14.
The maintained cutoff and unit/native boundary fixtures are updated and verified.
Both maintained and native builds,
package type checking,
and the changed TypeScript lint scope pass.
Built admission tests accept supported meanings at `0.60` and abstain at `0.599999`.
Native HTTP fixtures pass permission,
prohibition,
unspecified,
lower-score,
unknown,
and unsupported controls with zero provider requests.
Removing the cutoff process-locally makes the boundary assertions fail;
the built artifact remains unchanged.
The isolated Qwen CLI also passes with clean shutdown and zero provider requests.
Replaying the original baseline through the built consumer admits 29 of 84 answers,
compared with four at `0.80`.
There are 28 admitted clause meanings and one admitted request-content meaning;
the other 55 answers remain unadmitted.
`${HOME}/temp/agent/pi-luna-score-semantics-20261014/measurement-six.json`
and `cutoff-verification.json` retain the recount and guard control.
The `0.80` results in this section remain historical measurements.

The user selected `gpt-6-luna-decisions`,
excluded Perplexity,
and accepted scores of `0.80` or higher to reduce manual request judgment.
The agent's earlier `0.99` heuristic was Jev-specific and uncalibrated;
it must not be imposed as an unchanged Luna requirement.
The [updated decision](../decision/pi-auto-mode-semantic-admission.md#current-luna-decision)
preserves unique-leading supported meanings and independent source and authorization guards.
Only four of the retained 84 Luna answers reach `0.80`,
including the primary permission answer at `0.83`.
The migration therefore includes the serving adapter,
fixed model binding,
cutoff tests,
and bounded question-format experiments retaining all input.
The explicitly authorized Luna format comparison is complete:
three baseline requests admit four answers each at `0.80`,
and three requests with existing rubrics repeated beside their questions admit six each.
Every request returns all 84 answers inside five seconds,
with identical answer data within each format's repeated runs.
The primary permission estimate remains `0.83`.
The [comparison record](../planning/pi-auto-mode-gateway-decisions-current-contract.md)
retains the overlap in latency ranges and the mixed admission changes.
The maintained implementation now selects Luna and `0.80`
and contains the lossless Decisions adapter.
The package and owned native builds pass.
Package type checking passes;
the 16-file changed TypeScript lint scope reports zero warnings and zero errors,
with subsequent fixture-only changes checked separately.
The initial formatter diagnostics and fixture declaration-generation failure were corrected,
not ignored.
Built wire,
transport,
and admission tests pass.
Additional wire tests cover nullable request shapes,
the 255-choice boundary,
predicate bounds,
independent confidence,
and usage counts;
their final lint,
type,
and built-consumer runs also pass.
Native authored HTTP cases verify permission at `0.80`,
prohibition,
unspecified policy,
lower scores,
unknown meanings,
and unsupported meanings.
Every native case makes one local HTTP call and zero provider requests.
The native lifecycle and actual isolated Qwen CLI also pass with clean shutdown.

Local replay of all six retained provider responses preserves all 504 answer distributions
and their independent confidence values exactly.
Re-encoding the retained baseline request preserves its complete parsed JSON value.
Process-local omission of the model,
answer identity,
duplicate-label,
normalization,
or question-count guard makes the corresponding behavior assertion fail.
The first identity omission survived because mixed-kind reordering failed another guard;
an otherwise valid answer with a changed name now isolates original name attribution.
The final controls leave the built artifact unchanged.
Receipts live in `${HOME}/temp/agent/pi-luna-guard-controls-20261014/`
and the comparison directory's `built-adapter-replay.json`.
The explicit-rubric variant remains an experiment;
its mixed results do not establish greater accuracy.
The production adapter currently uses the verified lossless baseline encoding.
The installed host's new offline capture retains 73 source occurrences,
84 questions,
and 584,007 bytes.
Compared with the original provider-trial input,
only event timestamps,
recorded durations,
and their validated source hashes differ.
The first local capture used the smaller authored domain fixture,
which was not comparable to the installed extension set.
Its capture path also needed binding before the fixture clears its environment.
The installed capture uses the existing network-isolated disposable host
with all 17 configured packages and 37 read-only workspace package mounts.
`${HOME}/temp/agent/pi-luna-guard-controls-20261014/installed-equivalence.json`
retains the complete comparison result.
No new provider request was needed for this check.
The [question-batch experiment](../planning/pi-auto-mode-gateway-decisions-current-contract.md#question-batch-compensation-experiment)
is complete after explicit authorization resolved the automatic review rejection.
The user also authorized further Luna calls needed for this task without a total call cap.
Retain finite experiments and the original production judgment bounds.
All nine calls succeeded;
every complete repetition returned the same 84 estimates as the original baseline,
with four admitted at `0.80`.
The shared judgments took 1,993.272,
1,433.419,
and 1,182.940 ms.
Each used 246,783 reported input tokens rather than the baseline's 126,129.
Batching is not adopted because this experiment supplies no semantic improvement.

The subsequent readable-selection experiment also completed under the user's standing authorization.
It retains the complete baseline input and question bindings,
adding each selected passage verbatim beside its escaped JSON representation.
All three calls returned 84 valid answers within five seconds,
with identical answer data across repetitions.
Eight supported answers reach `0.80`;
the primary permission estimate is `0.82`.
This changes estimates without establishing accuracy or a complete admitted interpretation.
The candidate remains experimental.
Further representation-only trials on this same input are deferred;
continue the source-boundary investigation and native quality work.

The whole-request interpretation has a separate missing consumer,
independent of the low-scoring clause answers.
The retained baseline's `head_83` selects `potential-instruction` at `0.96`;
the readable-selection variant selects it at `0.97`.
A fresh built native contrast changes only the authored whole-request relation:
with every clause admitted,
`none` executes three prepared children,
whereas `potential-instruction` executes zero and retains `request-material-instruction-use`.
Both runs have 33 original sources,
five accounted partial source contributions,
one local HTTP assessment,
and zero provider requests.
The log is `${HOME}/temp/agent/pi-luna-request-use-control-20261014.log`.
This is an implementation boundary,
not evidence that Luna should report absence when the request contains instructions.
Continue original request-to-source governing-use accounting through the existing serializer and source owners.
Partial contribution associations alone must not be promoted to complete governing-use evidence.

The first producer extension now observes actual native system-message emission and converted tool arrays.
The existing payload owner retains the original caller transcript and the actual normalized transcript
passed to native parameter construction.
System associations require their original emission record,
and retain that record beside each partial contribution.
This corrected an initial comparison against the wrong transcript boundary:
Pi's normalizer creates a new array when collapsing system messages.
The rebuilt native SDK control passes with five accounted contributions and three executed children.
Built emission tests and copied-payload/transcript controls pass without provider requests.
The copied-system control now retains an unresolved system contribution and the unaffected accounted user contribution,
executing no child.
Process-local omission of the system-emission guard makes the copied system incorrectly accounted
and fails the native assertion;
`${HOME}/temp/agent/pi-serialization-controls-20261014/verification.json`
records the unchanged artifact hash.
Scoped TypeScript lint and package types pass.
The actual isolated Qwen CLI process also passes with zero provider requests and clean shutdown.
The next original-owner link is from serialized tools to their host input receipts.
Commit `4187ef7e13` extends the existing manager input publication through the incumbent message alias index,
using the declaration metadata already retained by the original append call.
The new typed owner propagates tool object identity only at observed native copy sites;
equal replacements and cross-publication reuse do not receive membership.
The original source still has no permission.
Built native-manager tests now pass for original append/context copies,
equal-content separate owners,
cross-owner reuse,
array/tool replacement,
schema mutation,
and accessor rejection without invocation.
The process-local array identity omission fails the corresponding assertion while leaving the built artifact unchanged.
`${HOME}/temp/agent/pi-tool-publication-controls-20261014/verification.json` retains that check.
The normal SDK fixture and all serializer receipt controls also pass with zero provider requests.
The test enters through the maintained policy consumer:
direct generated-manager and generated-SDK imports exposed an existing owned bootstrap cycle,
documented in `doc/troubleshooting/pi-sdk-staging.md`.
Removing that coupling remains native-quality work.
The serialized tool consumer now passes the normal SDK fixture with seven accounted partial contributions,
including both original tool declarations,
and three executed fixture children.
Copied-tool,
copied-tool-array,
and changed-description controls leave the affected contributions unresolved and execute no children.
All existing system/payload/transcript substitution controls still pass,
with zero provider requests.
The built membership consumer also passes each original-source/publication/input-occurrence contrast.
The typed native tool association module passes scoped lint,
package types,
the rebuilt SDK control,
all serializer substitution controls,
and the actual isolated Qwen CLI with clean shutdown and zero provider requests.
The original map callback retains each converted value before payload callbacks;
removing that owned snapshot makes the changed-description control fail.
`${HOME}/temp/agent/pi-tool-publication-controls-20261014/emission-verification.json`
retains the unchanged artifact hash.
The next boundary is complete system-message derivation and original request governing-use consumption.
The consolidated built contribution regression also passes the base,
run,
message,
publication,
and tool membership cases through `test:source-contribution`.
The current tool records remain partial contributions;
they do not remove `request-material-instruction-use`.
Complete system and tool governing-use accounting is still required before removing the independent obligation.

The original manager publication now validates its complete selected system input:
content,
sections,
tool declarations,
and tool removals.
System inputs without tool declarations receive the same original-owner checks.
The existing alias index carries these publications through observed native copies;
no second source inventory or authority registration is added.
Built manager tests pass for unchanged copies,
equal replacement messages and sections,
changed prose and section text,
replaced removal arrays,
and accessor rejection without invocation.
The package type check and four-file scoped lint pass with zero lint diagnostics.
The rebuilt native SDK and all serializer substitution controls pass with zero provider requests.
Process-local omission of either the full-field integrity check or field-identity check
makes its corresponding assertion fail.
`${HOME}/temp/agent/pi-system-publication-controls-20261014/verification.json`
records both failures and the unchanged built-artifact hash.
The serialized system derivation consumer remains the next step;
these original input publications alone do not establish complete request coverage.
The isolated Qwen CLI also passes with zero provider requests and clean shutdown.
The retained-input consumer now handles system and tool publications through the same original membership checks.
Its built contribution regressions,
package types,
and six-file scoped lint pass.
`src/source-contribution-tool.ts` moved to `src/source-contribution-retained-input.ts` on 2026-10-14.
The first derivation integration probe found the original system input publication,
but rejected its emitted system message because the new identity check used the wrong normalization stage.
Pi normalizes once in `stream` and again inside `convertMessages`;
the emitted message comes from the latter's transformed transcript.
The producer receipt now retains that exact inner normalized and transformed transcript,
and the consumer checks both original identities.
Native revalidation passes with eight accounted contributions,
including the complete captured system input,
and three executed fixture children.
The built emission test now verifies both actual normalization stages.
All serializer substitution controls and the package type check pass.
The request-level consumer is now implemented and under verification.
It uses the observer's existing JSON parse,
original producer cardinalities,
complete system replay inputs,
whole user-message publications,
and original converted tool declarations.
The supported envelope permits the original model and stream fields,
known numeric controls,
`store: false`,
and the native usage-reporting shape.
Unknown protocol fields,
additional message fields,
unsupported message roles,
missing original sources,
and changed cardinalities retain an unresolved request-use obligation.
Successful field coverage can discharge only `request-material-instruction-use`;
source-domain,
semantic admission,
authority,
delegation,
and operation premises remain independent.
The native request-derivation suite now passes permission,
prohibition,
and unspecified-policy cases while the request interpretation remains `potential-instruction`.
Permission executes three fixture children;
the other outcomes execute none.
Unsupported protocol and message fields,
added or removed messages,
removed tools,
copied system inputs,
unsupported roles,
and copied serialized system or tool values all retain unresolved request use and execute no children.
Every case makes zero provider requests.
The built source-contribution and request-obligation regressions pass,
including preservation of independent premises;
package type checking also passes.
The process-local unsupported-protocol-field guard omission fails the native unresolved-outcome assertion:
without that guard,
the fixture incorrectly resolves the request.
The verification record at `${HOME}/temp/agent/pi-request-derivation-controls-20261014/verification.json`
retains the unchanged built-artifact hash.
The initial verifier expected a later assertion;
it was corrected to recognize the actual earlier outcome assertion.
The final native rebuild and isolated Qwen CLI also pass,
with zero provider requests and clean shutdown.

The user subsequently asked about Jev 1.14 and whether `0.80` gates the correct quantity.
The [Jev release check](../planning/pi-auto-mode-jev-1-14-current-contract.md)
establishes native release availability but no qualified replacement through the current gateway.
The [Luna score trace](../planning/pi-auto-mode-luna-score-semantics.md)
confirms that the built consumer gates selected-option probability for each interpretation.
The baseline has 83 clause questions and one content question;
three clause answers and the content answer pass.
Applying `0.80` to the separate confidence field selects the same four baseline answers.
Neither score is established as whole-decision correctness,
and the suitability of requiring a unique meaning for every candidate remains a separate design question.

## Original native RPC approval verification

The disposable harness in `${HOME}/temp/agent/pi-native-rpc-human-20261014/`
now constructs the original native SDK session,
selects its installed `qwen-token-plan-individual/qwen3.8-flash` model record,
and runs Pi's original RPC mode.
Its complete proposed program is `return await tools.fixture_value({path:"seed"});`.
The tool returns fixed inert text.
All stream responses,
semantic answers,
and independent prerequisite facts are authored fixture inputs.
No provider request is allowed.
The direct SDK host facade supports only its original disposable session;
this is not the production CLI host's authorization owner.

The initial Allow control passed with one assessment,
one original verdict entry,
one tool execution,
zero external provider requests,
exit zero,
and empty child stderr.
The controller preserves the complete pending native dialog and response transcript.
Deny also passed with no tool execution and empty stderr.
Stop retained its original `user-deny` entry,
executed no tool,
and exited zero,
but failed the controller's unconditional empty-stderr assertion.
The source trace identifies deliberate request cancellation during assessment,
which the existing approval-domain test already expects as a handler failure.
This is not evidence of a disposal failure.
The corrected controller captured every stderr byte with its execution or disposal phase,
without filtering output.
Stop's 10,467 diagnostic bytes belong to execution;
disposal output is empty.
Stop,
cancellation,
Allow after 6,007 ms,
and an unrelated response identifier followed by Deny now pass.
The unrelated identifier leaves the original native request pending.
Every completed control uses one assessment,
zero provider requests,
and exit zero.
Only Allow executes the inert tool.
The user supplied a genuine Allow response.
The original pending controller had disappeared without a completion report,
so recovery recreated the disposable native dialog.
The complete displayed operation and choices matched exactly;
only the pending request identifier changed.
The user's Allow response was forwarded to that recreated request.
The fixture completed with one execution,
one assessment,
one original `user-approve` verdict,
zero provider requests,
exit zero,
and empty execution and disposal stderr.
`human-recreated-dialog.json` and `human-verified.json` retain the comparison and result.
This verifies the genuine human response through the native dialog boundary;
the production authorization-owner integration remains separate work.

## Typed native session construction

The manager registry port and its verification are complete.
`root-lifecycle-retirement-entry/registry.mjs` moved to `registry.ts` on 2026-10-14,
with its input and canonical epoch types in `registry-types.ts`.
A complete tracked-source search for `createNativeManagerRegistry` found only the maintained SDK factory
and its static bridge as consumers.
The factory consumes only `lifecycle`.
The unused `bind` snapshot adapter was removed:
its historical profile checked a planned session ID,
persistent file path,
leaf,
and a branch of at most two entries under qualification clocks.
Those were qualification-fixture restrictions,
not requirements of the maintained native lifecycle.
Canonical manager state,
constructor ownership checks,
registry-local epochs,
and the incumbent lifecycle remain.
The `session-root-dependencies.mjs` bridge and its declaration were removed on the same date;
the factory imports both the typed registry and typed native reader directly.
Registry branch controls pass against the rebuilt native artifact:
one manager shares reservations,
distinct managers and registries remain independent,
released reservations reuse canonical state,
and foreign or revoked ownership fails before canonical accounting changes.
The returned interface remains frozen without freezing borrowed managers.
The package type check and scoped lint pass with zero lint warnings and errors.
The owned rebuild,
native manager constructors,
host lifecycle,
and actual isolated Qwen CLI pass with zero provider requests and clean CLI shutdown.
Process-local omission of `owner.assertOwned(manager)` makes both foreign and revoked ownership assertions fail.
`${HOME}/temp/agent/pi-registry-controls-20261014/verification.json`
records the unchanged built-artifact hash and expected failures.
Commits `6d5a8642d1`,
`6260d5d19a`,
`2ed70ed78a`,
and `988d7cb453` retain the port and its branch controls.
The full native lint backlog and original request governing-use consumer remain separate work.

The next maintained dependency is the native session-manager identity reader.
`root-lifecycle-retirement-entry/root-reader.mjs` moved to `root-reader.ts` on 2026-10-14;
the original provenance remains recorded.
The SDK constructor now imports that typed interface directly.
The reader retains the original callbacks and exact returned occurrence,
and rejects a manager mismatch before invoking the occurrence reader.
The incumbent lifecycle still owns its original error constructors.
The owned native build and package type check pass.
The five changed TypeScript files pass scoped lint with zero warnings and errors.
Built-artifact checks pass for exact arguments,
original callback capture,
live manager changes,
missing callbacks,
and unchanged thrown values.
Native manager constructors,
host lifecycle,
and the actual isolated Qwen CLI process pass with zero provider requests.
Process-local removal of either callback validation or the manager-identity guard
makes the corresponding behavior assertion fail.
`${HOME}/temp/agent/pi-root-reader-controls-20261014/verification.json`
retains those controls and the unchanged built-artifact hash.
The first formatter run identified a test-local root `let` and incomplete ambient constructor declarations;
both were corrected without suppressions.

The next owned dependency is now ported from `runtime-factory-scopes/scopes.mjs`
to `scopes.ts` and `construction.ts`.
The maintained SDK caller now imports its real typed interface directly.
The original callback and manager captures,
exact prepared-result identity,
factory-wide exclusion,
and retryable failed cleanup remain explicit requirements.
The unused legacy result-preparer default is removed:
the maintained caller already supplies `prepareSdkResult`,
and the typed interface now requires that callback.
The historical provenance records the old path and October 14 move date.
Built-artifact checks for missing capabilities,
input replacement,
publication,
failed cleanup,
and reentrancy pass.
The final ten-file lint scope reports zero warnings and zero errors.
Package type checking passes with `TSC_SINGLE_THREADED=1` and `GOMAXPROCS=2`.
Native manager constructors,
host lifecycle,
and the actual isolated Qwen CLI process pass after the owned rebuild.
They report zero external provider requests;
the CLI reports clean shutdown and zero genuine human approvals.
Process-local omission of reentrancy exclusion or cleanup blocking fails its corresponding assertion.
Omitting original callback capture invokes the forbidden replacement preparer and fails the test.
The initial omission auditor incorrectly required all failures to be `AssertionError`;
the corrected readback verifies each actual failure boundary.
`${HOME}/temp/agent/pi-factory-scope-controls-20261014/verification.json`
records those controls and the emitted scope file's exact match with maintained source afterward.
The first type-check attempt emitted Go `newosproc` and is invalid despite its wrapper returning exit zero.
That masked compiler failure was reproduced and repaired separately in `dev-script/task-util`.
The built wrapper now preserves failed process status unless the existing known-diagnostic filter
fully accounts for the failure.
Original failing-before cases include a panic,
silent failure,
unfamiliar diagnostic,
and a panic beside suppressed diagnostics.
The repaired real-process fixture and existing output-filter tests pass.
The [wrapper incident record](../troubleshooting/typescript.md#task-tsc-002-converted-compiler-process-failures-into-successful-checks)
retains the initial success-control newline mistake and its source-backed correction.
Final wrapper lint reports zero warnings and zero errors across four changed files;
its package type check and all declared process cases pass.
Pi's real `lint:types` task also passes through the repaired wrapper.
The broader native quality queue remains open.
The genuine human response is now verified in the native RPC fixture described in
the original native RPC approval verification section.

The next approval preparation uses the existing native RPC dialog boundary.
The emitted Pi 1.2.0 `rpc-mode.js` creates the original extension selector request
and resolves only its pending response identifier.
A direct SDK fixture can retain authored prerequisite premises and local model responses
while allowing a genuine human to answer the actual pending native request.
This would verify that human response boundary only.
The production CLI still lacks an independent authorization-requirement owner;
unknown requirements must remain unresolved and must not be converted into UI requests.
The later genuine Allow result completes this human response verification.
Production authorization-owner integration remains separate work.

The maintained constructor now lives in `native-batch/session-root.ts`,
with typed native readers,
SDK result fields,
registration,
and publication.
Its static bridge keeps the incumbent lifecycle and factory-scope owners.
The old `session-root.mjs` consumer imports were updated together;
provenance retains the historical path and records the move date.
The original scoped measurement was 56 warnings and two errors in that module.
The seven typed implementation and bridge files now pass scoped lint with zero diagnostics,
and package type checking passes with the new tests included.

`test:native:session-result` exercises the built descriptor-copy helper.
Its original artifact passes;
a process-local omission of only the own-data-descriptor guard makes the accessor cases fail.
The omission marker and failure are retained in
`${HOME}/temp/agent/pi-session-result-guard-omission-20261014.log`.
The real artifact is byte-for-byte unchanged by that control.
Native manager construction and host lifecycle tests pass,
including foreign managers,
retirement,
failed disposal,
and cleanup retry.
The actual Qwen CLI process fixture also passes with clean shutdown and no external requests.

The first approval-context run hit `JudgmentDeadlineError` while other verification was running.
An unchanged isolated repeat passed Allow,
Deny,
Stop,
cancellation,
and Allow after resource reload.
`${HOME}/temp/agent/pi-session-root-approval-isolated-20261014.log`
retains every completed mode with zero provider requests and zero genuine human approvals.
The repeat establishes recovery,
not the deadline failure's cause or a permanent timing fix.
Full native lint completion and genuine approval-boundary verification remain outstanding.

The subsequent full package lint measured 14,368 warnings and 458 errors across 389 files.
Its log is `${HOME}/temp/agent/pi-native-full-lint-after-session-root-20261014.log`.
This broader file set differs from the earlier 368-file baseline;
the totals are not a controlled before/after comparison.
The verified construction port's scoped files remain clean.
Remaining findings include historical build inputs and maintained runtime owners;
do not silently exclude the former or rewrite their provenance to obtain a passing count.
The next directly used construction dependency is `lifecycle/runtime-factory-scopes/scopes.mjs`,
which this full run reports with seven errors.

The [decision-model trial priorities](../planning/pi-auto-mode-decision-model-trial-order.md)
retain the user's requested candidate recommendation and separate trial order from adoption.

## Newly enabled Gateway decision models

The user enabled `microsoft-decision-1` and `gpt-6-luna-decisions` on the existing Gateway key.
The [frozen compatibility plan](../planning/pi-auto-mode-gateway-decisions-current-contract.md)
uses the documented `/v1/decisions` route and an exactly invertible wire conversion.
Both supplied resources passed synthetic access checks with exact returned model names.
Microsoft returned the expected label at 0.998989 in 1,805.284 ms;
Luna returned it at 0.99 in 991.616 ms.
Automatic approval review blocked the complete private assessment requests before execution,
then the user explicitly authorized both prepared complete-input probes.
Each runs once with the original frozen input and a five-second deadline,
reusing the successful synthetic control.
The corrected Jev row encoder independently passes envelope and exact-reconstruction checks;
its 117,254-byte complete candidate has a separate one-call capacity plan.
No provider replacement or admission-policy change is adopted.

Both explicitly authorized complete-input probes are now terminal.
Microsoft returned HTTP 422 in 2,221.452 ms,
with `invalid TypeSafe request` and an estimated 145,150 tokens against a named 64,000-token limit.
No returned Microsoft model identity establishes which layer owns that limit.
Luna returned all 84 answers in 2,087.943 ms with the exact requested model name and no refusals.
None met the unchanged 0.99 unique-maximum admission rule;
the primary permission answer was 0.83.
The corrected Jev request independently returned HTTP 400 `max_tokens_exceeded` in 997.329 ms.
These phases are complete without retries.
The remaining authorized work includes maintained native quality checks and genuine approval-boundary verification;
provider capacity success alone does not supply semantic admission.

## SubQ authorization and compatibility check

The user authorized sending anything needed to `subq.ai`
using `AUTO_MODE_SUBQ_API_KEY`.
The models endpoint returned HTTP 200 with `subq-snap-preview`,
but the credential-loader correction invalidates the original claim of authenticated key verification.
The [current contract check](../planning/pi-auto-mode-subq-current-contract.md)
records the documented 16-question limit and a frozen 16/17-question synthetic boundary probe.
No production provider or assessment-budget change is adopted.
The pending Jev row-encoding correction remains tracked independently.

The user also authorized sending anything needed to `liquid.ai`
and added `AUTO_MODE_LIQUID_API_KEY`.
The [Liquid contract check](../planning/pi-auto-mode-liquid-current-contract.md)
records its documented hosted `d1` route and the initial public serving control.
Both provider keys are present in the ignored local configuration.
The first controls mistakenly sent SOPS ciphertext:
Liquid returned 401 and SubQ returned 403.
Those are not evidence against the actual keys or models.
Mise now resolves both decrypted keys in memory.
Fresh controls succeeded for both providers:
Liquid returned one valid answer in 591.486 ms;
SubQ returned 16 valid answers in 845.179 ms.
SubQ then rejected 17 questions with HTTP 422 `validation_error`:
`questions: must NOT have more than 16 properties`.
That measured cap cannot carry 84 questions within three calls.
Liquid's complete-input probe preserved every original state field and question,
changing only its model name.
The 504,929-byte request returned HTTP 422 in 1,098.969 ms,
naming a 65,536-token or 250,000-text-byte limit for `head_0`.
The exactly reconstructable shared-content candidate was 140,601 bytes.
Its single attempt ended with client `TimeoutError` after 5,005.097 ms without a response.
Liquid therefore has no completed full-assessment result within the production deadline.
No retry or production adoption occurred.

## Drex 2.0 release check

The user requested the best next action after reporting Drex 2.0.
The bounded [compatibility audit](../audit/tech-pi-auto-mode-drex-2-release-vet-2026-10-13.md)
retains the completed authorized provider evidence.
Both the constant-state control and complete-policy-once request returned `drex-v2.0`.
The complete native request failed first on structured question instructions,
then,
after lossless JSON-text conversion,
with `too many questions (244); limit is 16`.
The original policy,
sources,
question membership,
and three-call assessment ceiling remain unchanged.
Keep the incumbent route and continue native integration;
this is a failed replacement compatibility gate,
not a semantic quality comparison.
The [troubleshooting record](../troubleshooting/drex-model-route-label.md)
separates the owned schema mismatch from the unexplained hosted question limit.

### Reopened contract check on 2026-10-14

The user reiterated that Drex looks more promising.
The [current contract recheck](../planning/pi-auto-mode-drex-current-contract.md)
finds changed published context limits and an explicit no-fallback statement for Drex 2.0.
The observed 16-question rejection remains unexplained despite the published 512-question allowance.
The old context and fallback description is historical;
the current published limits are 131,072 state tokens and 262,144 state-plus-longest-question tokens.
No model replacement or new semantic qualification is adopted.

A prepared private fixture at `${HOME}/temp/agent/drex-question-boundary-20261014/probe.ts`
uses only the public synthetic state `The card is green.`
and repeated text-only `Is the card green?` questions.
Its `mise run check` passed with zero provider calls:
the 16-question request is 2,246 bytes and the 17-question request is 2,383 bytes.
The frozen request hashes are
`9f57a20c7564b0ac0868a834f2ed56d4d809d78e4e530dd4388e5857b29247b5`
and `6b46f9af796187cae0e937b7df5f38c01bbea0a844e229c433d832f845194711`.
The live schedule pins `drex-v2.0`,
caps each request at five seconds,
performs no retries,
and refuses to overwrite a started experiment.
The user explicitly approved these two synthetic probes on 2026-10-14.
The unchanged `mise run probe` schedule then started.
No policy or session content enters either request.
Both approved probes completed with HTTP 200 and `model: drex-v2.0`.
All original question IDs and typed finite Noul answers were present:
16 answers for the control and 17 for the boundary request.
The result disproves a universal 16-question cap on this route at this time.
It does not establish acceptance of the complete native assessment or semantic qualification.
The [result record](../planning/pi-auto-mode-drex-current-contract.md#approved-synthetic-boundary-results)
retains timings,
token usage,
and probability ranges without treating them as calibration.
The user then explicitly requested more calls to establish actual Drex limits.
The completed extended phase made 23 sequential synthetic requests with no retries:
12 returned HTTP 200,
10 returned HTTP 422,
and the final combined input returned HTTP 529.
Every attempt and response is retained in `${HOME}/temp/agent/drex-request-limits-20261014`.
The measured boundaries are 512 questions,
4,194,304 counted body bytes,
the server's named 131,072-token state boundary,
and an effective backend row limit of 139,264 despite the published 262,144 allowance.
The 244-question case with 65,536 repeated-text units succeeded in 25,155.119 ms;
the near-maximum state with 244 questions returned `overloaded` after 33,331.147 ms.
The frozen stop condition ended the schedule without retry.
These measurements reopen Drex's question-count compatibility but do not qualify the complete native assessment,
establish reliable five-second latency,
or adopt a new production route.

The latest approved installed Qwen/Jev verification is separately incomplete:
`${HOME}/temp/agent/pi-native-installed-live-NB6KVz/host-stderr.txt`
retains `AssessmentTransportError: Assessment request returned HTTP 400`.
The transport discards upstream error detail,
so the cause is not established and must not be inferred from historical token-limit failures.
The installed command emitted the supported literal single-call program,
but no successful governed file read was observed.
Package types and scoped lint of the five changed semantic interfaces pass;
`${HOME}/temp/agent/pi-single-call-clean-scope-lint-20261014.log`
reports zero warnings and zero errors.

### Jev workaround assessment on 2026-10-14

The user asked which Jev problems were encountered and whether they can be worked around.
The [current assessment](../planning/pi-auto-mode-jev-current-contract.md)
separates reproduced token-limit failures,
earlier semantic binding errors and repeat variability,
owned host defects,
the latest unclassified HTTP 400,
and historical gateway-source limitations.
One shared-guidance representation returned all 205 estimates within its original budget,
but no universal capacity remedy or current categorical semantic qualification follows.
The current `0.99` operating rule and independent source/authority guards remain unchanged.
The refreshed vendor documentation also warns about Choice option order;
the present clause label list puts permission first,
so provider order-sensitivity remains an explicit untested boundary.
LLM Gateway now announces that System One will be deprecated in favor of Decisions,
without a retirement date in the inspected page.
That is maintenance evidence,
not an explanation for the observed HTTP 400 or a tested capacity workaround.
No new provider request or product change follows from this deliberation request.

The user subsequently said to keep working.
The transport now preserves the recognized `max_tokens_exceeded` code while excluding upstream free text.
Its regression failed before the fix and passes after rebuilding;
the transport suite,
package types,
and scoped changed-file lint pass.
The approved installed rerun at `${HOME}/temp/agent/pi-native-installed-live-pygBoi`
reached Jev with Qwen's supported literal read and returned
`Assessment request returned HTTP 400; provider code max_tokens_exceeded`.
The governed read did not execute successfully.
The next step is local measurement of the complete native assessment and lossless encoding opportunities;
the adopted provider,
policy,
source membership,
and admission cutoff remain unchanged.

Local replay captured 73 source occurrences and 84 questions in 504,946 bytes.
Exact reconstruction checks preserved the complete state and every question in experimental encodings.
The shared-content candidate measured 165,373 bytes,
but its one frozen Jev request still returned `max_tokens_exceeded` in 1,269.832 ms.
It is not installed or a verified workaround.
The complete original request was then prepared for Drex with text-encoded question instructions,
round-tripped exactly,
and measured 541,947 bytes.
The user explicitly approved this request after automatic approval review rejected recorded historical consent.
The approved five-second call ended with `TimeoutError` after 5,005.129 ms and no usable response.
No retry or production route change occurred.
The user then renewed standing authorization to send anything needed to Drex and Jev and use credits freely.
The next diagnostic observes complete-request completion beyond the production deadline;
late results cannot authorize an action.

The longer complete-input observation returned `drex-v2.0` with 84 answers,
then failed local distribution validation at 48,528.415 ms.
It reported 2,418,163 input tokens.
The four-decimal probabilities explain a separate precision mismatch with the consumer's `0.000001` sum tolerance;
no validator was loosened.
None of the selected scores reaches `0.99`.
The primary explicit permission clause received `none` at `0.3586`.

The completed clause-isolation study preserves the complete state and selects only the primary clause.
Unchanged baselines give identical permission probability `0.7748`
at 2,254.570 ms and 1,342.972 ms.
Inlining the same definition and rubrics gives permission `0.3473` at 1,338.757 ms.
It is not a verified remedy.
A further full-batch repeat returned HTTP 529 `overloaded` after 26,134.675 ms;
its frozen stop condition prevented the planned two-question control.
All provider jobs have finished.
The subsequent fixed representation study completed five HTTP 200 responses.
Its unchanged primary-clause baselines returned permission `0.6906`;
removing pre-assessment guard-status fields returned `0.7468`,
and parsing the program JSON strings returned `0.6533`.
A deliberately incomplete minimal control reached `0.8576`.
None reaches `0.99`.
The unchanged baseline previously returned `0.7748`,
so cross-study variation exceeds the observed status-removal gain.
These are not demonstrated remedies.
No production semantics or provider choice has changed.
Provider experiments are terminal;
the remaining authorized mechanical queue includes native runtime lint/type completion
and verification of the genuine human approval boundary.
Successful installed governed execution remains blocked by the provider capacity and semantic evidence gates.

The independent maintained CLI fixture check reproduced replacement of its fetch blocker by Pi 1.2.0 setup.
The typed `native-host/process-network-guard.ts` now imports the dispatcher before overriding fetch.
The required-red regression fails on the original blocker identity and passes after rebuilding.
Initial setup,
reconfiguration,
data-URL rejection,
and use outside the fixture are covered.
The real CLI process check retains the expected Qwen model and clean shutdown.
Both changed TypeScript files pass scoped lint with zero warnings and zero errors;
package types pass.
The final owned rebuild,
network-guard regressions,
and real CLI process check pass.
The rendered troubleshooting record retains the failing and passing evidence.

The native semantic-policy forwarding is now TypeScript and consumes the maintained consumer's original types.
`action-policy/assessment-model.ts` supplies the unchanged fixed model to both the wire and scope selector.
Absent configuration stays at bootstrap selection;
the typed copier receives present configuration only.
The measured bridge scope changed from 11 warnings to zero warnings and zero errors,
and package types pass.
The rebuilt operating-policy and domain suites pass with authored responses and no provider requests.
The unconfigured-policy SDK control reaches an unresolved review with all four consumers unconfigured,
zero assessments,
and zero tool executions.
A process-local omission of the bootstrap absence guard fails at the expected policy-copier diagnostic;
`${HOME}/temp/agent/pi-native-policy-absence-omission-second-20261014.log`
retains the result and the harness verifies unchanged maintained artifact bytes.
The first omission log lacked the final diagnostic after retaining 65,934 bytes;
it was not accepted as a verified omission result.
The second capture records the original exception before Node prints the bundled source context.
No classifier definition,
operating cutoff,
or instruction-source membership changed.

The latest Jev capacity diagnostic confirms that the 125,822-byte shared state fails
even with one original question:
HTTP 400 `max_tokens_exceeded` after 731.176 ms.
Its literal-label serving control succeeded first.
Splitting the same shared-state input is therefore insufficient.
The placement experiment moved original question instructions out of shared state
and back into their respective question entries with exact reconstruction checks.
The 105,339-byte state still failed:
the complete batch returned HTTP 400 `max_tokens_exceeded` in 1,049.183 ms,
and its single-question control returned the same code in 518.147 ms.
Both calls are terminal with no model answers.
The offline structure comparison reconstructs every source and question exactly.
Inlining single-use references reduces state to 95,749 bytes;
combining that with shared ordered field lists reduces it to 88,015 bytes.
The next frozen capacity schedule sends the complete inlined request first,
then the combined representation only after recognized `max_tokens_exceeded`.
Each call has a five-second deadline;
there are no retries or production admissions.

The inlined request returned `max_tokens_exceeded` in 1,233.578 ms.
The combined row prototype instead failed gateway `invalid_union_discriminator` validation:
it wrapped required question-envelope fields in rows.
That owned encoding defect invalidates the combined candidate as a capacity measurement.
The next local correction preserves the envelopes and adds a failing/passing envelope preflight.

## Restart checkpoint after installing automatic Codex shell limits

The user restarted the session and resumed work on 2026-10-13.
The first default sandboxed `cat /proc/self/cgroup` returned
`0::/user.slice/user-1000.slice/user@1000.service/piwork3285273.slice/codex`.
The parent controls returned `memory.max=4294967296`,
`memory.swap.max=0`,
`cpu.max=200000 100000`,
and `pids.max=256`.
Automatic attachment is now verified in the ordinary sandbox.
The agent supplies ordinary commands;
the installed native hook supplies the attachment prefix.
Historical manual worker wrappers in this document are superseded for Codex Bash calls.
The Pi host remains outside the worker slice.

Native RPC reopened session `01a0dc52-0955-77f6-ae77-68a6e15bb12b`
with `qwen-token-plan-individual/qwen3.8-flash`.
`get_state` returned the expected session identity,
no streaming or compaction,
and no pending messages.
Closing stdin completed shutdown with exit zero.
No model prompt was sent.

The next domain-resolution acceptance stopped during preparation:
Node reported `ERR_MODULE_NOT_FOUND` for the retired `yuku-parser@0.17.0` path.
The installed parser is now 0.19.0 and Pi is 1.2.0.
Pi's installed coding-agent package no longer has the expected unbundled JavaScript files.
The current native integration therefore needs adaptation before its prior results describe the current host.
The read-only upstream source checkout is Pi tag `v1.2.0`,
commit `06a466935536f8c9bcacb2d5efc0fc021be1bdff`.

Private `contract/integration/current-sdk` now emits separate Agent,
AI,
and coding-agent JavaScript using the existing TypeScript 7.0.2 compiler.
Staging passed without changing installed sources or installing dependencies.
Commits `2699da6e`,
`2edd3014`,
and `52cced2f` retain the staging source and fixture connection.
The first native patch check reached the changed `emitBeforeAgentStart` call:
Pi 1.2.0 takes a run trigger and prepares user or custom runs in `_prepareRun`.
Private commits `dfdbb616` and `1e7824bb` adapt that native boundary.
`mise run native:prepare` now passes against Pi 1.2.0 and copies 13 instrumented modules.
The staging loader uses native package export resolution,
including wildcard subpaths,
and the source build retains generated model catalogs from the exact installed release.
The domain acceptance then found that its historical Markdown parser owner,
`package/cli/markdown-lint`,
was removed by the repository's Rust linter migration.
Private `f411f9da` declares the previously qualified `satteri@0.10.5` directly in the fixture workspace.
Its installation uses `pnpm install --ignore-scripts`.
Private `ba4ecadd` records the generated parser lockfile.
The current acceptance reaches the native policy reviewer with zero provider calls.
Pi 1.2.0's `reachesTools` callback initially produced an unsupported tool-metadata gap.
Required-red `2fe40907` and correction `27d713a7` establish its executable-field handling;
the metadata owner controls pass.
That intermediate permission case reported only `request-source-domain`
and failed its required closure assertion with 26 original sources.
Required-red `dab689bd` then demonstrated that an original resource input,
already retained before a suppressing callback,
was absent from governing-source collection.
Private `9ebc722c` and `0cbee730` connect those original receipts and their native source locations.
The suppressed-input control passes.

Private `5d69e8cb` adds structural domain accounting through the original native publications.
Unaccounted native event types remain named gaps;
the current supported callback path is `before_agent_start`.
Source membership,
source indexing,
resource receipts,
and resource/base/run links are checked independently of semantic answers.
The current authored native contrasts pass with 33 original sources:
permission approves and executes all three child calls,
prohibition denies with zero child calls,
and no applicable instruction stays unspecified with zero child calls.
No external provider call or production semantic policy adoption follows.
The missing-evidence controls now pass:
removing source membership,
request-bound views,
source indexing,
resource receipts,
or native event inspection leaves a gap.
An actual unaccounted `context` callback keeps execution unresolved.
Potential request instructions retain their independent interpretation obligation after structural domain accounting.

Required-red `cab88eb3` found that a prior run's mapped prompt source lost its original occurrence
when a later run omitted that field.
Private `c0a3910b` and syntax correction `22e1e4f4` retain original prompt sources
through the existing run owner's history,
scoped to the same native root.
The native history control passes,
and the permission,
prohibition,
unspecified,
unobserved-callback,
and potential-request-instruction contrasts still pass.
Private `b8a8d705` also charges resource receipt records to the shared input count and byte budget.
Historical comparison controls read retained immutable evidence;
they do not reopen live eligibility after the provider request advances.
The targeted guard omission check also passed:
removing only the unobserved-native-event guard in a disposable worktree made the real `context` callback
case fail its required-domain-gap assertion.
Restoring the guard made both domain-gap cases pass.
The worktree was inspected and removed without deleting shared fixture targets.
Original approval-request routing and maintained integration are next.

The approval trace found a separate concrete gap:
`compileBoundClauseProgram` omitted independent authorization satisfaction and channel facts.
Required-red private `99bd9818` reached the canonical native handler but opened no selector.
Private `5d266c33` connects a fixed bootstrap authorization-owner factory,
binding each original evidence record to its judgment,
clause,
source,
and request.
The owner supplies independent prerequisite facts;
it supplies no clause meaning or final decision.
Private `f22d8db0` keeps standalone evaluation from accepting that factory as a per-call override.
Redundant full-source scans initially exhausted the unchanged five-second judgment deadline.
Private `76eeac39` reuses captured source bindings while retaining checks around asynchronous evidence capture
and before release.

Private `d66e8250` adds the rejection and native approval contrasts.
`mise run test:authorization-inputs` and `mise run test:approval-domain` now pass.
The canonical native Allow path calls the maintained `askUser`,
writes through Pi's actual extension API into its original session manager,
and executes all three prepared children.
Deny and Stop write native verdicts and execute no child.
Missing evidence,
unknown satisfaction,
unsupported channels,
a bare prerequisite without permission,
and an independent prohibition execute no child and open no selector.
Evidence retired during selection prevents execution even after an Allow entry is written.
A satisfied independent requirement permits execution only with separately established permission.
These controls use authored independent facts and a fixture selector,
with zero external provider requests and zero genuine human approvals.
They establish canonical routing,
not generic prose authorization,
production semantic qualification,
or installed human confirmation.
Affected consumer regressions,
a targeted authorization-binding omission,
and maintained integration are next.

The prerequisite evidence collector is now maintained in
`package/pi-plugin/auto-mode/src/authorization-evidence.ts`
and its sibling contract,
capture,
record,
and binding modules.
Private `dfb0082f` replaces the native collector copy with maintained exports.
Missing facts have an explicit `MISSING_AUTHORIZATION_EVIDENCE` result;
the compiler preserves the unresolved prerequisite.
The first package build,
type check,
and unit suite passed.
Lint corrections and final native verification remain in progress.

Native preparation then found missing external dependency paths.
The fixture now owns the same `yuku-parser@0.19.0` release directly,
recorded in private `f0627729` and generated lockfile `f18a93f9`.
The current `pi --version` returns 1.1.0,
matching the repository lock;
the previous staged build and dependency symlinks targeted 1.2.0.
Private `402e47ac` emits separate version-specific staging trees.
The current read-only source tag is `v1.1.0`,
commit `abe508e1b89912adde45528136c3221eb69acdd7`.
Earlier 1.2.0 test results retain that version boundary.
Current Pi 1.1.0 acceptance now passes with the maintained authorization collector.
Authorization-input rejection controls,
domain resolution,
unobserved-callback and potential-request gaps,
and retained run history all pass.
The native approval cases cover Allow,
Deny,
Stop,
missing or unsupported facts,
unknown satisfaction,
satisfied requirements,
bare prerequisites,
independent prohibition,
and evidence retirement during selection.
The first unknown-satisfaction run reached `JudgmentDeadlineError` while package lint was also running.
Three unchanged repetitions after stopping that lint process passed.
This records the observed difference without attributing its cause.
The authorization binding guard also failed its omission control and passed after restoration.
No provider call or genuine human approval was used in these fixtures.

Maintained delivery is now the active work item.
The runtime still imports private integration modules,
and the CLI uses `createAgentSessionFromServices` through a runtime factory.
Its session managers originate before the SDK session constructor.
The maintained host must preserve that original constructor custody across resume,
new session,
and fork operations;
an SDK-only replacement does not establish CLI coverage.
The measured runtime graph contains 100 owned modules plus 11 earlier generated owned inputs.
The build graph and dynamic provider imports still require explicit accounting.
The full package lint was stopped after 528 seconds without output;
the changed authorization files pass their scoped lint.
No full-package lint success or installed-host completion is claimed.

The maintained native graph is now committed in `package/pi-plugin/auto-mode/native`.
Commit `1019171c30` retains 138 modules and assets with original source hashes;
15 entries are explicitly recorded as earlier generated inputs.
The build introduced by `9e25198b39` emits the exact Pi 1.1.0 upstream revision,
then applies maintained transformations inside the package's `dist/native` directory.
The build and package type check pass.
Its first relocated native permission,
prohibition,
and unspecified cases pass.
That fixture location also exposed an inherited repository `AGENTS.md` input;
`2202547b51` moves disposable fixture workspaces outside the repository.

The original CLI manager boundary is now explicit:
`createFromManager` accepts only managers returned by the same constructor owner,
including `continueRecent` and `forkFrom`.
Required-red `5416067c04` reached the native failure
`Original manager already has a context-copy owner` when constructing a replacement session.
Fix `30dbd25246` releases the old observer only after completed native disposal.
The original manager then supports a replacement session,
while foreign managers and the retired lease remain rejected.
Additional committed controls retain observation custody after incomplete disposal
and demonstrate that repeated completed disposal entered old cleanup.
The completed-disposal guard passes the native handoff,
incomplete-cleanup,
and repeated-disposal controls.
The normal SDK stream now passes the same permission case with original serializer contributions:
five contribution uses are accounted,
all prepared children execute,
and the fixture makes no external provider request.
Required-red `90cd721a44` proved that the earlier direct-stream identity check omitted those contributions.
Commits `ed7c9688c5` and `a603ce5343` bind the payload at its native serializer occurrence,
retaining the original serializer input before Pi creates a normalized message array.
The package type check also passes.
The candidate CLI entry is now `package/pi-plugin/auto-mode/dist/native/cli.mjs`.
Maintained transformations route Pi's original manager selection,
service creation,
session creation,
runtime replacement,
and disposal through the same constructor owner.
Native resource reload now forwards the original project-trust callback through policy capture.
The CLI graph required retaining the native SDK's runtime exports
and rebasing its original `chalk` dependency after relocation.
Commits `36dbdb4526` and `b4028644ff` correct those owned staging failures.

`test:native:host-lifecycle` passes native reload,
cancelled replacement,
new session,
in-memory fork,
persisted resume,
and disposal retry.
The verified case constructed four sessions and invoked the original trust callback four times,
with no external provider request.
Required-red `fde85bf48d` demonstrated that a new session could be constructed after incomplete disposal.
Fix `dc56e44191` withholds further construction until the original cleanup succeeds.
The extended native lifecycle case passes,
including retry and repeated completed disposal.
The normal SDK permission case still passes with five accounted contributions and all prepared children executed.
Copied-payload and copied-transcript controls now stay unresolved and execute no prepared child.
Removing only the native payload-occurrence guard in a disposable worktree made the copied-payload test fail:
it incorrectly reported the copied contribution as accounted.
Restoring the guard and rebuilding made both rejection cases pass.

The candidate CLI also passes a separate-process RPC check using disposable state and synthetic credentials.
It returns `qwen-token-plan-individual/qwen3.8-flash`,
has no active stream or pending messages,
and exits with no stderr output.
The check disables external fetch and sends no model prompt.
The built-in Codemode extension initially copied away the original owned definition.
Required-red `0a5f3080ca` and fix `1adc116d99` preserve that definition through native registration;
the host lifecycle and normal SDK execution cases pass afterward.

The package type-check scope needed a correction.
The inherited configuration excluded `native/`,
so its earlier success did not validate the native build helpers.
Commit `d99d0db3a5` adds those TypeScript files to the configured source set.
A disposable `native/type-scope-probe.ts` produced the required TypeScript `TS2322` failure,
and the same check passed after removing the intentional error.
The build-info file now lists the original native helpers.
The owned JavaScript graph still needs type and lint work.
The measured scoped lint run over the new host files and initial helpers reported 975 warnings and 97 errors.
The build helpers are being refactored to asynchronous file and process operations,
with tagged diagnostics and explicit types;
full lint success remains unestablished.
The subsequent complete native-source lint at `95bf856283` measured
23,389 warnings and 943 errors across 154 files.
This includes the relocated JavaScript runtime and earlier generated owned inputs.
The result is a remaining implementation requirement,
not a passed package check.
The resolver and dependency paths are now being moved into type-checked source;
input snapshots and policy checks retain their existing requirements.
The typed resolver and asynchronous dependency paths now pass their scoped lint check
with zero diagnostics and the package type check.
Their original `loader.mjs` entry is retained as a re-export of `loader.ts`.
Native manager constructor ownership is also ported to TypeScript,
with the actual Pi 1.1.0 constructor signatures and original-result membership preserved.
That port passes its scoped lint and package type check.
The reference search found that the earlier `runtime-root-lifecycle/lifecycle.mjs` implementation
had no constructor consumer;
its only imported value was an ownership error.
The unused earlier transition implementation was removed,
and its constructor error hierarchy remains in `runtime-root-lifecycle/errors.ts`.
The active transition implementation remains `root-lifecycle-retirement-entry/lifecycle.mjs`.
The package build now sequences maintained exports before native preparation;
the inherited parallel build fanout did not encode that dependency.
The complete sequenced build and native SDK execution check pass.
The SDK case accounts for five original contributions and executes all three prepared children,
with zero provider requests.
The existing CLI manager and host lifecycle assertions also pass after the typed constructor port.
The host lifecycle run emitted
`logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify`.
Its emitting process and delayed operation are not established;
the assertion result does not establish clean diagnostics or a connection to earlier logger incidents.
The separate-process CLI check passed with empty child stderr.
Additional constructor controls now cover persisted history,
explicit session IDs,
working-directory overrides,
recent-session fallback,
and cross-directory fork inputs;
`test:native:manager-constructors` passes all five native entry points
and constructs actual sessions from the recent,
forked,
and externally populated in-memory managers.
The delayed-file-sink positive control detects the logger diagnostic and identifies its pending `mkdir`.
One unchanged native lifecycle run with that tracer passed without the diagnostic.
This does not establish the earlier timeout's cause or a fix;
the result is retained in the
[logger record](../troubleshooting/pi-sdk-staging.md#native-lifecycle-logger-timeout-on-2026-10-14).
The refactored build helpers now pass scoped lint with zero warnings and zero errors across six files.
The package type check,
complete native build using its default qualified source checkout,
and separate-process CLI fixture also pass.
The original approval suite now runs through the maintained `test:native:approval` task.
Its ten cases pass with original native verdict entries,
zero external provider requests,
and fixture selections only.
Approval context capture is the next native host boundary:
Pi invalidates its extension runner on reload,
so each original review must capture the current native capabilities.
Committed required-red `0b98ee3203` rejects the new fixed bootstrap source before the implementation.
The maintained source is now connected.
Native Allow,
Deny,
Stop,
cancellation during selection,
and approval after reload pass in `test:native:approval-context`.
These cases retain Pi's actual extension context and original verdict API;
only the displayed selection response is a fixture.
Cancellation executes no child,
and the reloaded Allow case executes the complete prepared group.
The direct-SDK reload fixture now uses the same original policy reload route as native host services.
The [source trace](../troubleshooting/pi-sdk-staging.md#pi-110-native-approval-contexts-change-on-resource-reload)
retains the initialization and fixture corrections.
The candidate CLI has no adopted semantic admission policy.
Complete lint,
installed activation,
and genuine human approval remain outstanding.

The first maintained-source formatter pass applied existing fixes to 141 modules,
excluding historical generated snapshots.
It changed 122 files in `e6acf4ff53`.
The repository's arrow autofix emitted invalid declarations for unparenthesized parameters,
and `typescript/prefer-readonly` inserted TypeScript syntax into a JavaScript class.
Correction `26aaecf2dd` writes the intended named declarations and preserves the JavaScript field.
`lint:native:syntax` now passes all 157 native modules using Node's actual parser.
The owned rebuild and permission,
prohibition,
and unspecified-policy cases pass after correction.
The formatter's 573-error result included parse failures and is not a complete remaining lint count.
The first approval-context run expired at the existing five-second judgment deadline.
An unchanged isolated run then passed Allow,
Deny,
Stop,
cancellation,
and Allow after reload.
This records recurrence behavior without establishing the deadline failure's cause.
Copied-payload and copied-transcript controls still rejected execution,
and the separate CLI process passed with empty child stderr.
Commit `ff0a34feaf` names 407 callbacks across 74 maintained modules.
The preparatory parser comparison preserved all comments and normalized executable syntax,
apart from converting the arrow declarations to named functions.
It rejected lexical-binding hazards and avoided introducing names that shadow existing identifiers.
The Node parser and owned rebuild pass on that change.
Native SDK execution,
all current approval-context cases,
and host lifecycle replacement and cleanup also pass.
At `ff0a34feaf`,
the complete native lint run reported 14,790 warnings and 538 errors across 157 modules.
The original generated input snapshots remain included in that check.
No lint rule or admission condition was changed.
Commit `fbcd866f88` connects asynchronous package manifest capture to native preparation.
Source reads,
generated writes,
real-path checks,
and Markdown parser qualification now await their filesystem operations.
The captured import selector remains synchronous and pure during source rewriting.
The owned build and permission,
prohibition,
and unspecified fixtures pass;
they execute three,
zero,
and zero child calls respectively.
Node parses all 158 native modules.
The four changed modules still report 592 warnings and eight lint errors;
none is a synchronous filesystem finding.
The typed manifest helper separately passes its type and scoped lint checks.
Commits `dd0158bd40` and `3cd3d11f22` split resource,
agent,
and program preparation into sibling modules and replace mutable source stages with named constants.
Static imports and native host relative imports now use the existing parser's original literal locations.
Comments,
unrelated strings,
and dynamic expressions retain their original bytes.
The rewriter visits parsed structure with a work stack,
orders replacements by original tokens,
and joins disjoint source fragments without rebuilding an accumulating string.
Built-artifact tests cover static,
side-effect,
multiline,
re-export,
and nested dynamic imports;
they also cover escaped source names and adversarial replacement values.
The initial static test import reported TypeScript `TS6307`
because generated `.ts` was outside the composite source set.
The final test loads the built module by URL and checks its expected callable consumer contract.
No source import,
type-check suppression,
or additional generated-source include was introduced.
The owned build,
import boundary tests,
package type check,
and canonical permission,
prohibition,
and unspecified fixtures pass after connection.
Scoped lint for the import test and typed rewrite/mapping helpers reports zero warnings and zero errors.
Native SDK stream,
all approval-context cases,
and host-lifecycle regressions also pass against this artifact.
Removing the parser diagnostic guard in a disposable copied module makes the committed invalid-source test fail;
the original copied module passes,
and the production artifact remains unchanged.
Node parses all 163 modules after the split.
The seven connected JavaScript preparation files report 565 warnings and zero errors.
Warnings still fail the shared `denyWarnings` lint policy;
this is not a passing package lint result.
The staged-import TypeScript port in `3ebdfb4d19` and `8dede28e63` retains its original file-identity checks.
Its scoped lint,
package types,
owned rebuild,
and canonical decision fixtures pass.
The native Markdown adapter is now typed in `native/contract/native-index/native-parser.ts`
with a separate consumer contract.
It retains the measured `satteri@0.10.5` profile and original UTF-16 coordinate control,
and verifies that CommonJS and ESM resolve the same measured parser entry.
The old unused repository-root input was removed from both maintained callers.
Manifest identity and its hash now describe the same captured bytes.
The rebuilt native decision fixtures,
package type check,
installed parser test,
and disposable dependency variants pass.
The parser implementation and fixture files report zero lint diagnostics.
The first nested-array assertion was corrected after reading the Chai-backed matcher implementation;
it now checks original tree fields and retained source positions directly.
The fixture loader's missing explicit URL input type was corrected after TypeScript `TS9011`.
Disposable copies also verify each parser guard's sensitivity:
manifest identity,
fixed source epoch,
ESM entry identity,
MDX rejection,
root shape,
and source coordinates.
Every omission fails its intended consumer assertion,
while the original copied module passes;
neither the production artifact nor installed dependencies were changed by those controls.
Commits `0da2d65bcd`,
`7f3630185d`,
and `68b047682a` type native host source preparation and its file boundary.
The finite upstream inputs are captured before source transformation;
validated generated modules are written independently after every original call-site check succeeds.
The original constructor,
reload,
Codemode,
and paired disposal edits are preserved.
Both typed preparation files report zero lint diagnostics.
Package types,
the owned native build,
and host lifecycle replacement and cleanup pass.
The separate CLI process passes with empty shutdown stderr,
and every approval-context case passes against this artifact,
including cancellation and approval after reload.
Remaining preparation generators and the JavaScript runtime still need lint and type completion.
The tool-description generator is now typed in `ccccf6820c` and `b3956dae21`.
Its scoped lint,
package types,
owned rebuild,
and native decision contrasts pass.
A direct comparison against its prior generator confirms identical generated Session bytes.
The rebuild inspection also found retired `.mjs` files beside their typed replacements.
Commit `ba238bce51` replaces the copied contract tree before rebuilding it.
The disposable rebuild retains the original emitted Pi inputs,
removes an intentionally stale nested module,
and passes the actual CLI process check with clean shutdown.
The first disposable attempt omitted the root-resolved `nano-spawn` dependency and failed with Node's
`ERR_MODULE_NOT_FOUND` before the build ran.
The corrected fixture reproduces that original dependency resolution without changing installed packages.
Native Agent,
Session-state,
and prompt-builder generators are now typed with named source stages.
The assignment and literal-rule guards inspect the qualified parser's executable nodes;
comments,
strings,
and unrelated calls cannot change their counts.
The import rewriter shares their iterative structural walk.
Scoped lint,
package types,
owned rebuild,
and built-artifact import,
assignment,
and literal-rule tests pass.
Direct comparisons against the prior generators retain identical generated Agent,
Session,
and prompt-builder bytes.
Disposable omission controls remove only the corresponding parse or membership guard.
Every omitted guard fails its intended consumer assertion,
while both unchanged copied modules pass.
The maintained source and production artifact remain unchanged by those omission controls.
The native SDK stream,
all approval-context cases,
and host lifecycle replacement and cleanup pass against the rebuilt artifact.
Typed message conversion,
provider serialization,
extension loading,
and resource receipt generators also pass scoped lint,
package types,
and the owned rebuild.
Comparisons against their prior implementations retain identical generated module bytes,
with the same module base URL supplied to the resource generator comparison.
Copied-payload and copied-transcript controls remain unresolved and execute no prepared child;
the separate CLI process still returns the expected Qwen state with empty shutdown stderr.
The typed Agent and resource preparation phases share an explicit captured-input contract.
Both phases read their complete finite source set before transformation and generated writes.
Their scoped lint,
package type check,
owned rebuild,
and canonical native decision fixtures pass.
The Agent input observer and complete-group boundary retain identical generated bytes to their prior implementations.
The original import,
literal-rule,
and direct instance-member transformations now share ordered source-fragment assembly.
The private-member consumer tests preserve comments,
strings,
computed accesses,
other receivers,
identifier prefixes,
and original escaped identifier spelling.
Scoped lint,
package types,
owned build,
and all three built transformation suites pass.
Disposable parser and reserved-private-name omission controls fail their intended assertions;
the original copied module passes.
The typed Session lifetime and disposal-completion generators pass scoped lint,
package types,
and the owned build.
Comparisons retain identical generated native Session bytes after both transformations.
Native host lifecycle,
original CLI manager custody,
and every approval-context case pass after the typed lifetime and disposal port.
The typed native manager custody generator passes scoped lint,
package types,
and the owned build.
Its generated module remains byte-identical to the prior implementation.
Every original manager constructor,
CLI manager custody,
and host lifecycle fixture passes.
The maintained reference search found only `root-source.mjs`
(now `root-source.ts`,
moved on 2026-10-14) importing its `generate` export;
the copied standalone `withBranchFailure`,
`mutate`,
and `replaceOne` fixture exports had no maintained callers and were removed.
The production root-occurrence,
method-membership,
instance-freeze,
and original transition-site checks remain in the generator.
Original private qualification artifacts remain unchanged.
The typed root composition also passes scoped lint,
package types,
and the owned build;
both manager and Session generated modules remain byte-identical.
The first following lifecycle invocation stopped before its fixture ran:
Node reported `EROFS` while creating the designated disposable home directory.
The rerun grants access to that same disposable fixture location and passes native host lifecycle,
every original manager constructor,
and the SDK stream decision fixture.
The generated-byte comparison separately encountered `spawnSync git EPERM`
despite a populated successful Git result;
capturing the prior source through a direct Git invocation completed the comparison.
These command-boundary failures do not establish a native runtime failure.
The typed program preparation modules pass scoped lint,
package types,
and the owned build.
Original dispatcher,
executor,
runner,
judgment,
definition,
and result-converter generated bytes remain identical.
The generated Session differs only in the private runner field spelling,
now retaining the original `_nestedToolCalls` identifier after adding its private marker.
The parser-based member transformation preserves unrelated source text.
The SDK stream,
all original approval contexts,
and host lifecycle pass after this change.
Dedicated result-converter tests retain original helper order and adversarial literal contents;
they reject missing functions,
changed declaration spelling,
invalid syntax,
and duplicate module declarations.
The copied unchanged converter passes;
omitting each parse,
membership,
or spelling guard fails its intended consumer assertion.
The omission fixtures do not modify maintained source or the production artifact.
Typed original judgment preparation also passes scoped lint,
package types,
and the owned build with identical generated module bytes.
The canonical permission,
prohibition,
and unspecified fixtures pass after the port.
The shared member transformer now supports explicit private destination names for native prompt fields.
It validates each destination with the consuming JavaScript grammar before assembling edits.
Built tests preserve source escapes,
comments,
strings,
computed accesses,
and unrelated receivers while rejecting reserved names and injected declaration syntax.
Scoped lint,
package types,
the owned build,
and those member tests pass.
Prompt preparation is split into typed input,
owner,
base,
run,
and handler phases so each original source boundary remains independently inspectable.
These phases pass scoped lint,
package types,
and the owned build.
The generated handler module is byte-identical.
The generated Session differs only in one original comment:
the parser-based member rewrite preserves its `_extensionRunner` spelling,
where the prior regular expression changed it inside the comment.
The SDK stream,
all native approval-context cases,
and the actual Qwen CLI process pass;
CLI shutdown stderr remains empty.
Disposable destination syntax and declaration-shape omission controls fail their intended assertions;
the unchanged copied member transformer passes.
Typed program preparation captures all source inputs before publishing generated modules.
It passes scoped lint and package types,
and its owned rebuild preserves all eight generated program module hashes.
The main native preparation entry,
its build caller,
and the extracted Session preparation phase pass scoped lint,
package types,
and the owned build.
The native preparation chain now uses maintained TypeScript modules.
Full package unit verification passes.
The syntax sweep parses all 179 native modules,
and host lifecycle and the separate Qwen CLI process pass with clean shutdown.
The full package lint invocation did not return diagnostics and was terminated deliberately;
it is incomplete verification.
The wrapper buffers diagnostics until its child exits.
Its two TypeScript workers held 38 and 102 threads during a process snapshot.
A concurrent documentation commit was saved,
but automatic push reported `failed to create new OS thread`
and `fatal error: newosproc`.
The enclosing worker slice retains its 256-task limit and reports `pids.events: max 60`;
those cumulative events do not establish the exact failing thread's allocation path.
After the lint process stopped,
the slice reported 23 tasks.
The separate `git push origin main` completed successfully,
publishing the saved documentation commit.
Continue lint verification through an explicit complete source manifest in sequential groups,
preserving the rules and worker limits.
That manifest check now covers all 319 source files with unchanged final source hashes:
14,477 warnings and 495 errors remain.
The retained results combine completed file groups with individual checks for the final groups.
This is complete failing coverage,
not package lint success.
`source-contribution-test-fixture.ts` completed after Oxlint reported 396.2 seconds.
A bounded CPU profile identified repeated `expressionResultSites` and `reachableValueSources`
traversal in the repository-owned readonly-analysis plugin.
The isolated completion removes the need to repair that plugin before continuing this migration.
No linter source or rule was changed.
A brief duplicate continuation was stopped;
overlapping timings are not used for comparison.
The [worker record](../troubleshooting/codex-shell-worker-limits.md#task-pressure-during-pi-package-lint)
retains the separation between incomplete runs,
the completed source manifest,
and the earlier push failure.

Resource input copying now uses typed `values.ts`,
`input-shape.ts`,
and the shared collector error constructor.
Its built consumer tests cover detached snapshots,
opaque dependency identity,
own data descriptors,
adversarial text,
and the original input limits.
Scoped lint,
package types,
the owned build,
and those consumer assertions pass.
The subsequent native permission case twice exhausted its unchanged 512 MiB Node heap.
Disposable artifact comparisons isolate the new per-snapshot logging:
the prior implementation passes,
the typed copy with only its new log emissions omitted passes,
and the complete typed copy aborts with `heap out of memory`.
Retaining logger imports while counting calls measured 169,893 snapshots and two option copies
in the passing permission case.
Commits `838f5aa849` and `c1d4fe7ecd` replace per-snapshot emissions with bounded microtask attempt counts.
Package types,
the owned rebuild,
and native permission,
prohibition,
and unspecified-policy cases now pass at the original heap limit.
The rebuilt input tests and native host lifecycle also pass.
Disposable omissions of record,
descriptor,
field,
text,
array,
aggregate-budget,
option-key,
and input-kind guards each fail their intended consumer assertion;
the unchanged copied implementation passes.
The production artifact is not modified by those omission controls.
The built resource consumer and real logger sink also retain all 303 attempted snapshots
in two diagnostic records across a microtask boundary,
including both unsupported-kind attempts.
No heap limit,
source bound,
freshness check,
or semantic admission policy changed.

Contribution contracts now express absent fields through optional properties;
their fixtures omit absent values and retain explicit malformed-JavaScript rejection controls.
Commits `07ce8e0a95`,
`757ef72c92`,
`104e7e7f44`,
and `af00dfcf6f` retain that work and its statement-layout corrections.
The rebuilt declaration artifact and package type check pass.
The fixture formatter's first fix child remained active at 829 seconds elapsed,
then completed and entered the wrapper's separate verification pass.
The later ten-second profile contains 9,190 of 9,234 samples inside `expressionResultSites`.
That verification child was stopped deliberately;
the completed formatter mutations remain in `8cea077432`.
This incomplete run establishes no lint success.
Commit `a07329f154` separates original source references,
run construction,
shared fixture types,
and final request associations into explicit construction stages.
The full formatter and verification pass over those construction stages and native guard files
reports zero warnings and zero errors across six files,
with a final lint duration of 3.0 seconds.
The final plain lint covers all ten changed contribution and guard source files
with zero warnings and zero errors.
The rebuilt declarations,
package types,
and all contribution consumer suites pass.
Original profile references,
absent fields,
malformed nullish associations,
copied publications,
retained history,
and exact caught values retain their assertions.
The normal native SDK stream accounts for five original contributions and executes three prepared children.
Copied-payload and copied-transcript controls remain unresolved with zero executions.
No linter source,
rule,
or worker limit changed.

The independent native result-binding guard is also ported to `script-value-guard.ts`.
Its original caller imports and source provenance now point to that maintained file.
The typed declaration retains `AggregateError`'s original copied list and property descriptor;
an emitted field initializer would have changed descriptor ownership and is not used.
Built consumer tests are committed for caught primitives,
object identity,
copied-list freezing,
call diagnostic quoting,
existing guard identity,
and derived guard names.
The base error category is explicitly named;
derived error classes declare their own category rather than inheriting constructor-name inference.
The reference search found no existing native subclass consumer.
That port passes package types,
scoped lint,
the owned native build,
and its built consumer tests.
Disposable identity-propagation and error-list-freezing omissions each fail the intended consumer assertion;
the unchanged copied artifact passes.
The guard tests retain ordinary caught primitives and objects without changing their values,
and confirm the native non-enumerable aggregate descriptor.
The native syntax sweep parses all 182 maintained modules.
The complete package unit task also passes after the contribution fixture and native guard changes.
Its full output is retained at `${HOME}/temp/agent/pi-auto-mode-unit-after-contribution-20261014.log`.
No provider request or production semantic policy adoption is involved.

The next maintained boundary is the original native return consumer.
Commits `62a6ad4efe`,
`1eeda119fb`,
and `ac4a768272` port its custody contracts and distinguish retained completion from live script-value access.
Native staging erases TypeScript annotations before generating its JavaScript consumer;
import replacement selects the parsed original named binding and still verifies its filesystem identity.
The owned native build passes.
The initial generated-consumer tests and package type check pass after removing copied TypeScript imports
from the test compilation graph.
Those assertions cover conversion-once behavior,
original return identity,
parent retirement,
copied outcomes,
prepared-member completion,
and synchronous or asynchronous native failures.
The rejection and staging controls subsequently passed,
and plain lint reported zero warnings and zero errors across all 13 changed typed files.
The normal SDK stream and native permission,
prohibition,
and unspecified-policy contrasts also passed.
Disposable omissions demonstrated the original return guards and parsed-import guards independently.

Required-red commit `f14fc78e8d` then exposed a scheduling regression introduced by this port:
the extra asynchronous completion helper allowed a later queued result mutation to precede conversion.
Its generated-consumer assertion received `later queued mutation` instead of `original text`.
Commit `b80cb3cd8a` moves synchronous return retention directly after the original native await.
The changed source passes scoped lint,
package types,
and the owned native build.
The rebuilt generated-consumer scheduling assertion and both return suites now pass.
The repeated disposable return omissions also pass their sensitivity checks,
with original controls passing and every targeted omission failing its intended assertion.
Their final artifact directory is `${HOME}/temp/agent/pi-native-return-omissions-eiEqn9`.
The parsed staging controls and their independent parse,
uniqueness,
URL-identity,
and file-identity omissions pass;
the retained artifact directory is `${HOME}/temp/agent/pi-native-staged-omissions-IxFVXJ`.
Production artifacts were not mutated for those omission checks.
The final native SDK stream accounts for five original contributions and executes all three prepared children.
Native approval-context Allow,
Deny,
Stop,
cancellation,
and post-reload Allow checks pass with fixture presentation and zero provider requests.
The syntax task parses all 185 maintained native modules.
The complete package unit task passes after the native return changes;
its log is `${HOME}/temp/agent/pi-auto-mode-unit-after-native-return-20261014.log`.
This checkpoint is not full-package lint success or production activation.

The native manager and external-dependency error modules are now TypeScript as well.
Commits `e330b1306b` and `14297e8e81` retain their original constructor signatures,
update every native import,
and record the moved source paths without replacing historical hashes.
Both modules pass scoped lint,
package types,
and the owned native build.
A direct comparison against the pre-port modules exercised all 18 constructors in 24 cases.
Original outer messages,
instance property descriptors,
constructor arity,
and cancellation causes match.
The observation lifecycle aggregate now has the descriptive message
`Original file observation and closure failures`;
its copied error list and borrowed failure values retain their native identity and mutability behavior.
The comparison artifact is `${HOME}/temp/agent/pi-native-error-contracts-9EOJCg`.
Actual native manager constructor and host lifecycle tests pass after this port,
with zero provider requests.
Maintained native runtime lint/type work and production delivery remain in progress.

The next production-path gap is original context callback receipt accounting.
Required-red commit `485efc02e9` adds actual `context` and `context_with_system` callbacks
to the normal SDK serializer fixture.
The rebuilt fixture failed with `AssertionError [ERR_ASSERTION]` at the required source-domain closure assertion:
both native event types remained unaccounted.
The callbacks include in-place input mutation and unselected fields in complete return values.
Implementation `301ab621de` adds a synchronous traversal observer;
the follow-on integration routes its captures into the existing native manager input pool.
The original awaits and native error reporting remain at their existing sites.
Completion must be linked to the original published run before these event types can be accounted.
Native integration now passes after the source-location correction in `625ade8649`.
The first wired run stopped at `SourceInputLocationError: Source input location names an unsupported native field`;
the correction admits the new `contextHandler` representation through the existing location copier.
It does not confer authority on those source records.
The observer is now a frozen factory interface with private closure state,
and `06262e6364` checks the combined handler bound before allocating positions.
It retains the existing 64-source and 256-handler traversal limits.

`test:native:context-receipts` passes its observer unit cases and actual SDK contrasts.
Permission executes three prepared children;
prohibition,
no applicable instruction,
and potential request instructions execute none.
The ordinary callback fixture retains eight context receipts among 41 original sources,
and preserves the original result before both later queued mutations.
The throwing callback fixture retains its mutated input,
records no invented return value,
and has seven context receipts among 40 original sources.
Missing completion or a completion associated with another run leaves a source-domain gap.
The actual unobserved `turn_start` callback also leaves that gap and executes no child.
All these cases use authored semantic answers and zero provider requests.
The final contrast log is `${HOME}/temp/agent/pi-native-context-receipts-final-20261014.log`.

The package type check,
owned native build,
and scoped lint of nine changed TypeScript files pass.
The disposable omission campaign at `${HOME}/temp/agent/pi-native-context-omissions-gf45Nz`
passes its original baseline and fails the intended assertions for all eleven omissions:
stage order,
callback identity,
traversal completion,
emission completion,
extension bound,
handler bound,
data-path inspection,
original capture-error propagation,
input capture,
complete result capture,
and settled input capture.
Every retained failure log was inspected;
the production artifact was not modified.
Native syntax checks pass for 188 maintained modules.
The original Pi approval context passes Allow,
Deny,
Stop,
cancellation,
and approval after reload with fixture presentation.
The actual host lifecycle passes reload,
cancelled replacement,
in-memory fork,
persisted resume,
and disposal retry.
Those logs are `${HOME}/temp/agent/pi-native-context-syntax-20261014.log`,
`${HOME}/temp/agent/pi-native-context-approval-20261014.log`,
and `${HOME}/temp/agent/pi-native-context-host-20261014.log`.
The complete package unit task also passes after the final context diagnostics were built;
its log is `${HOME}/temp/agent/pi-auto-mode-unit-after-context-20261014.log`.
The final observer lint reports zero warnings and errors;
package types and the owned native build pass after its formatting correction.
The rendered native documentation and current context-accounting paragraphs were checked.
This work adopts no production semantic policy,
raises no budget,
and does not activate the installed plugin.
The unobserved-event regression now uses a native `turn_start` callback;
historical `--unobserved-context` commands retain their original meaning and are superseded by `--unobserved-event`.

The production bootstrap now binds the maintained direct HTTP assessment sender.
It retains the incumbent Jev route,
original encoded body,
preparation-inclusive deadline,
client-attempt accounting,
and inherited byte limits from the private transport.
Credentials are read lazily from `AUTO_MODE_LLMGATEWAY_API_KEY`.
The sender performs one HTTP request per original attempt and does not adopt semantic admission.
The 28 isolated transport cases and four actual loopback HTTP cases pass.
The loopback cases cover successful POSTs,
redirect refusal,
cancellation after response headers,
and a stalled response reaching its original deadline.
Canonical native permission,
prohibition,
and unspecified-policy contrasts pass through the maintained HTTP sender.
Each makes one authored HTTP response;
only permission executes its three prepared child calls.
Package types and scoped lint of six changed TypeScript files pass.
The disposable source-build campaign at `${HOME}/temp/agent/pi-assessment-omissions-7fuLwj`
passes its original baseline and fails all ten intended omission assertions.
It covers request and response limits,
attempt bounds,
credential withholding in both directions,
HTTP status,
original clock and cancellation,
combined cancellation propagation,
and invalid UTF-8 rejection.
Each omitted variant exits one;
every failure log was inspected and the production artifact was not modified.
The native CLI process verifies Qwen selection and clean shutdown.
Host lifecycle checks pass after the transport binding.
All 188 maintained native modules pass syntax checks.
The complete package unit task passes,
including actual loopback HTTP;
its log is `${HOME}/temp/agent/pi-auto-mode-unit-after-transport-20261014.log`.
The native README,
current transport checkpoint,
and troubleshooting addition were checked in rendered output.
No external provider request was made.
The package JavaScript fanout task stopped at `spawnSync mise EPERM` while listing tasks;
the explicit `build:js:node` task succeeds.
This failure has not been equated with the historical `ENOBUFS` incident.
The [staging troubleshooting record](../troubleshooting/pi-sdk-staging.md#node-26111-package-build-fanout-reports-spawnsync-mise-eperm)
retains the observed reporting path and verified explicit-task workaround.
On 2026-10-14,
the user answered the production acceptance question:
"You do it and adopt it yourself."
The adopted operating rule requires a supported selected label to be the unique highest-scoring choice
at `0.99` or greater.
Unknown,
unsupported,
tied,
or lower-scoring answers remain unresolved.
This is an operating cutoff and carries no measured accuracy claim.
The [admission decision](../decision/pi-auto-mode-semantic-admission.md)
records its fixed model,
program profile,
per-definition meanings,
and independent authority and coverage obligations.
The maintained factory `createNativeSemanticOperatingPolicies` now supplies all categorical policies
to the original native bootstrap.
Its built-package boundary cases pass for every definition and supported meaning.
The native SDK controls pass through the maintained HTTP sender:
permission at `0.99` executes its three child calls,
prohibition denies,
no instruction stays unspecified,
and lower-scoring,
unknown,
or unsupported clause answers remain unresolved with no child execution.
Every case retains its original source accounting and makes zero provider requests.
Package types and scoped lint of all six changed TypeScript implementation and test files pass.
The full package unit task passes against the rebuilt adopted policy;
its log is `${HOME}/temp/agent/pi-auto-mode-unit-after-semantic-policy-20261014.log`.
The disposable source-build campaign at `${HOME}/temp/agent/pi-semantic-operating-omissions-4cV6Sz`
passes its original baseline and fails all seven intended guard mutations.
These cover the cutoff,
unknown and unsupported meanings,
normalization,
score bounds,
selected vocabulary,
and inclusion of the exact `0.99` boundary.
The vocabulary control retains a valid distribution to isolate its selected-label check.
Every failing log was inspected;
the production artifact was not changed.
Original host construction and disposal checks pass.
The separate actual Qwen CLI process passes RPC state and empty-stderr shutdown checks,
with zero provider requests and no genuine human approval in these fixtures.
All 196 maintained native modules pass parser checking.
The native README,
current handover,
and admission decision were checked in rendered output.
The admission decision's initial sentence incorrectly described unresolved evidence as an automatic approval request.
It is corrected against the original approval router and guard-outcome implementation:
unknown or unspecified policy blocks execution;
only otherwise established permission with an eligible authorization requirement reaches native approval.
No runtime fallback was changed.
The native construction and disposal owner now lives in `host.ts`,
renamed from `host.mjs` on 2026-10-14.
Its original static generated dependencies,
SDK result fields,
retirement tokens,
and cleanup-retry state remain intact.
The host's membership registry now retains the already-created original resource record beside its loader key;
no replacement policy owner or source registry is introduced.
Package types,
the owned native build,
original host lifecycle checks,
and scoped lint of all six changed TypeScript interfaces pass.
Native approval,
native-context Allow,
Deny,
Stop,
cancellation,
reload,
and separate-process Qwen CLI checks pass after the host port.
An overlapping verification run first stopped at a native `SIGABRT`
and a separate Go `newosproc` diagnostic from full lint.
The [staging troubleshooting record](../troubleshooting/pi-sdk-staging.md#overlapping-native-verification-and-oxlint-1870-stop-before-completing-checks)
retains their separate boundaries and the successful sequential checks.
The unchanged full lint sweep now completes on its own and reports 13,432 warnings and 454 errors
across 368 files.
Its log is `${HOME}/temp/agent/pi-auto-mode-full-lint-sequential-20261014.log`.
Package-wide lint completion remains unfinished despite the clean changed interfaces.
All 198 maintained native modules pass parser checking.
The current host configuration still loads the legacy auto-mode package through the user's global Pi settings;
installation preparation must replace that legacy activation and route the ordinary `pi` command
to the maintained host without duplicating guards.
This historical activation state was superseded on 2026-10-14:
the maintained launcher is installed at `${HOME}/.local/bin/pi`,
and `file-enforcer.config.ts` now places the package's `bin` directory before dependency-provided Pi shims.
The installer removed only the legacy auto-mode package declaration,
verified every other setting against its original value,
and retained a private backup at `${HOME}/temp/agent/pi-native-install-4AsZGN/settings.json`.
The global provider and model defaults remain unchanged.
Disposable installer controls cover successful activation,
repeat installation,
exact preservation of unrelated settings,
and rejection of a foreign existing command without changing its settings.
Both configured decisions pass with all 22 installed extensions plus the explicit receipt-control callback.
The first operation executes three children;
the second assesses the later input and remains unresolved without more child execution.
Actual provider-backed governed execution and genuine native approval remain unfinished.
The installed command reached Qwen in a read-only package sandbox with disposable state,
but its ordinary single-call read was blocked before assessment.
The existing program decoder requires an awaited producer followed by a returned parallel successor group.
The diagnostic repeat's `try` block also failed that profile;
local tagged diagnostics now expose the retained `ResultProgramProfileError`.
The [adopted single-call profile](../planning/pi-auto-mode-governing-domain-closure.md#adopted-live-host-single-call-program-profile)
adds only a returned literal tool call or an immutable binding followed by its return,
preserving the existing producer/successor profile and every budget.
The user adopted this extension on 2026-10-14.
Implement its two stated forms,
preserving existing guards,
budgets,
and the 0.99 admission rule,
then repeat the installed governed read.
Live logs are `${HOME}/temp/agent/pi-native-installed-live-20261014.log`
and `${HOME}/temp/agent/pi-native-installed-live-diagnostic-20261014.log`.
The adopted single-call implementation is committed in `00f8f1fe67`,
`9e99064bbc`,
`8997d1568d`,
and `66e0f358ce`.
The decoder and compiled-source reconstruction preserve both single-call forms,
and the fixed semantic scope includes only those forms and the existing producer/successor profile.
The unchanged native group owner receives one reached group for a single call.
Model-visible loadout guidance now states the supported forms and preserves native nested-tool declarations.
Both maintained and owned native builds pass.
The first native consumer check exposed an omitted export in the maintained bridge,
then in its generated bridge.
Commits `41ab9e575b` and `fb1d7f4361` connect that export without changing profile matching.
The rebuilt `test:native:single-call` passes:
both forms return the original native result with one assessment and one child call;
prohibition executes no child;
26 unsupported syntax cases reject;
copied declarations reject;
both existing parallel combinators retain their profile.
Its log is `${HOME}/temp/agent/pi-single-call-generated-bridge-test-20261014.log`.
Package types,
scoped lint,
guard omission,
and the installed provider-backed read remain to be verified.
Automatic approval review rejected the repeat installed read before launch,
citing insufficient retained authorization for the external Qwen payload.
Recheck the original explicit Qwen testing instruction and the disposable fixture inputs before retrying;
do not bypass the rejection through another launch path.
The fixture source check confirmed the selected Qwen route,
synthetic read task,
synthetic policy and proof file,
read-only package mounts,
and absent real-session mounts.
Automatic approval review still rejected the same command because it did not accept retained authorization evidence.
The user then explicitly approved this prepared verification and its synthetic policy,
configured tool descriptions,
synthetic result,
Qwen route,
and Jev reviewer through LLMGateway on 2026-10-14.
Run the same bounded fixture;
this does not authorize a new semantic qualification campaign.
The approved installed run now emits a supported single-call program,
but exposed `TypeError: effectOwnerFor is not a function` before assessment.
The host omits this optional implementation-fact callback;
the fixture had hidden that path with a callback returning `undefined`.
The local omitted-property regression reproduced the same failure.
Commit `b3581b0be0` uses the existing absent-owner branch without supplying facts or permission.
Both single-call results and prohibition pass after rebuilding with the callback genuinely absent.
The [callback incident](../troubleshooting/pi-sdk-staging.md#pi-120-installed-policy-capture-calls-an-absent-optional-effect-owner)
retains the separate diagnosis and logs.
Repeat the same approved installed verification after this wiring correction.
The current command resolves through generated workspace dependency shims,
and the root `mise.toml` path list is owned by `file-enforcer.config.ts`.
That command-resolution description is historical;
`mise exec -- which pi` now selects `package/pi-plugin/auto-mode/bin/pi`.
The next consumer check must exercise the configured extension set in disposable state.
The generated `readActiveSessionDomainEvents` includes every registered handler kind except `before_agent_start`;
`inspectNativeSourceDomain` currently accounts only original `context` and `context_with_system` receipts.
Other registered kinds remain explicit gaps even if their callbacks were not invoked in that run.
The no-extension CLI startup fixture does not establish that the installed extension configuration can decide an action.
Do not silently disable the user's other extensions or treat their unresolved inputs as absent.
The concurrent dependency refresh in `755685da03` moved installed Pi from `1.1.0` to `1.2.0`.
The first installed-extension probe stopped before resource loading:
Node emitted `ERR_MODULE_NOT_FOUND` for `partial-json` because the existing emitted graph's
dependency links still targeted the removed `1.1.0` package directories.
The native source input is being updated to upstream tag `v1.2.0`,
commit `06a466935536f8c9bcacb2d5efc0fc021be1bdff`.
Emission now replaces each owned generated package tree before compiling,
so obsolete dependency links and source files cannot survive a release change.
The original migration qualification must pass on that graph before resuming the extension check.
The full `1.2.0` build,
package types,
original host lifecycle,
actual Qwen CLI,
semantic operating-policy,
context-receipt,
and native approval-context checks now pass.
Scoped lint of the changed release and emission files reports no diagnostics.
Read-only resolution found 18 configured packages and 22 enabled extension entries after legacy auto-mode exclusion.
Automatic approval review rejected an ordinary subprocess test of those extensions because their side effects
were not contained by a disposable home and rejecting `fetch` alone.
The replacement fixture uses Bubblewrap `0.12.0` with isolated user,
network,
PID,
IPC,
UTS,
and cgroup namespaces;
read-only code and dependency mounts;
no host credentials or Git metadata;
and a writable disposable tmpfs home.
Its preflight proves host code writes fail and only loopback is visible.
Automatic review accepted that isolated execution.
The isolated fixture loads all 22 extension entries and observes their registered event kinds.
Synthetic's public model-catalog lookup is blocked and retained as an expected exact URL;
no provider inference is dispatched.
The consumer test fails at its original source-domain assertion:
all 22 registered event kinds other than the separately captured `before_agent_start` lack complete receipts.
The actual native review retains its unresolved source-domain issue.
Complete capture at the original event invocation and settlement boundaries is the next implementation step;
the existing guard remains in force.
The maintained native event observer and source generator are now connected to the generated host.
The generator selects original parsed `ExtensionRunner` methods:
`emit`,
`emitBoundary`,
`emitMessageEnd`,
`emitToolResult`,
`emitToolCall`,
and `emitUserBash`.
It retains the native awaits and catches while adding synchronous input,
complete result,
settled mutation,
and traversal-completion observations.
The observer distinguishes native early returns from missing callbacks,
tracks overlapping emissions independently,
and publishes original observation identities with pending and failed state.
Commit `a0c220d54f` connects its capture callback to the existing manager input pool,
selects the generated runner through the existing loader,
and binds original observation freshness into the source snapshot.
The first generated build rejected an ambiguous final-return marker;
anchoring it to the original line resolved that build failure.
The first native-domain regression then rejected requests after empty event dispatch.
Empty dispatch now preserves the observation identity because it invokes no callback and adds no source.
The native permission,
prohibition,
and unspecified-policy cases pass again.
The configured-extension trace first exhausted the original 64-source allowance in `emitMessageEnd`.
Capturing each callback phase as a separate source caused that failure.
The manager now copies each phase immediately and groups one emission into bounded source chunks.
It retains every input,
complete result,
and settled input mutation.
Each chunk remains within 65,536 bytes;
the original source count and total byte bounds remain unchanged.
Pending emission bytes are charged before final publication.

`test:native:event-receipts` passes against the generated original runner.
It exercises all six instrumented dispatch methods,
90 callback phase observations in one source,
queued result mutation,
native cancellation and blocking,
native error identity,
overlapping emissions,
empty dispatch,
and nullish capture failures.
The new TypeScript modules initially had three lint errors and 238 warnings.
Immutable transformation steps and direct parsed-node traversal remove the errors;
the formatter left ten warnings,
which were then corrected.
The scoped six-file lint reports zero warnings and zero errors.
Package type checking passes with `GOMAXPROCS=2` after an uncapped TypeScript worker reported
`failed to create new OS thread (have 156 already; errno=11)`.
This is a completed bounded invocation,
not evidence of a permanent thread-allocation fix.

Native preparation events append original source receipts after initial judgment construction.
The existing source handoff now admits authenticated manager and event growth during preparation,
without changing its policy,
resource,
runner,
request,
or judgment identity.
The original `prepare` operation seals the final collection before question selection.
The ordinary `readSourceSnapshot` method preserves the judgment facade's data-property requirement.
The assessment representation now retains the `eventHandler` input location.
The configured fixture reaches one successful authored assessment,
then stops because another event changes the frozen snapshot before nested program execution.
Its original judgment and inference budget remain closed;
the freshness guard has not been bypassed.
The exact transition is revision 28,
`tool_call`,
pending zero,
to revision 29,
`tool_execution_update`,
pending one.
Pi's original Codemode executor calls its `publish` callback before `ctx.createDeclaredProgram`.
Its `publish` function returns `onUpdate`'s result,
but that call does not await the result.
The generated agent-loop update callback returns the native asynchronous event emission.
The native callback is therefore still pending when the enclosing program checks its original judgment.
The next implementation boundary must distinguish incomplete asynchronous event delivery
from completed callback contributions that change the selected instruction sources.
It must retain native callback identity,
complete received results,
and the same original closed judgment budget.
No configured-extension consumer success or installed migration completion is claimed.
The maintained single-callback regression in commit `e8294dd52e` reproduces the same failure
without loading configured extensions:
`test:native:progress-events` records one successful authored assessment,
one unchanged callback,
and zero child executions.
A disposable in-memory change that awaits the initial Codemode progress publication still fails:
the event observation reaches revision two with pending zero,
but differs from the original revision-zero source snapshot.
Waiting alone is therefore not the complete remedy.
The decision-scoped versus continuously expanding receipt timing is recorded in
`doc/planning/pi-auto-mode-governing-domain-closure.md`.
The user adopted decision-scoped membership on 2026-10-14:
new inputs govern the next decision.
Retain every later input while binding the approved operation to its assessed sources and original budget.
Changes to existing policy or resource dependencies,
prepared arguments,
cancellation,
and execution or result identity must still invalidate execution.
The implementation now distinguishes authenticated appended receipts from changed selected dependencies.
The initial approved-source implementation in `05f6276bff` exposed a second snapshot in the dependency facade.
The retained diagnostic confirmed that this independently captured snapshot still rejected the progress receipt.
Commit `1149f50605` makes the dependency facade retain the same assessed collection.
The unchanged progress consumer now passes with three child calls,
seven progress callbacks,
and one authored assessment;
the package type check passes.
The following-decision receipt control in `80f877e998` initially reached only one assessment.
The original diagnostic was `RuleAssessmentWireError: Primary source differs from mandatory policy`.
Retained prior-run inputs preceded the selected policy in the source collection.
Commit `696400dba7` preserves the primary-policy position while retaining those inputs.
The consumer now completes the first operation,
retains both the later callback input and result,
and blocks the next operation after its own assessment.
Each decision has one assessment;
the completed operation's judgment is not reopened.
The approved dependency controls pass:
rewriting the selected policy or cancelling after the first child prevents the remaining child calls,
and mutation of the assessed argument object throws while the unchanged approved operation completes.
The first argument-control attempt inspected a copied event payload rather than the assessed input;
`c365ce2b7a` corrects the control to use the original judgment's prepared member.
All 22 configured extensions now reach three child executions,
but the 64-receipt limit prevents cleanup from completing.
A disposable in-memory 128-receipt experiment passes the complete configured consumer,
measuring 81 retained receipts and 347,913 serialized bytes at the last observed check.
All byte bounds and the original assessment budget remain unchanged in that experiment.
The initial diagnostic hook accidentally matched other packages' built entries;
its failed extension import is not evidence about capacity.
The corrected hook matches only this package's artifact and its named native source modules.
The prepared capacity patch increases combined and session receipt counts to 128,
preserving the resource-local 64-receipt cap and every byte bound.
Automatic approval review rejected its commit because this resource and policy-boundary change requires
explicit user authorization.
The user explicitly approved the prepared increase to 128 receipts on 2026-10-14.
Commit `b45783436f` records the approved change.
Both maintained exports and the owned native runtime rebuild successfully.
The original isolated configured-extension runner passes without the diagnostic hook:
22 extensions load,
three child calls execute,
and one authored assessment completes with zero provider requests.
The log is `${HOME}/temp/agent/pi-approved-capacity-configured-20261014.log`.
The 128/129 native receipt boundary,
combined count and byte unit controls,
next-decision receipts,
selected-policy changes,
cancellation,
and immutable assessed arguments pass against this build.
Package types and scoped receipt-accounting lint pass with zero lint diagnostics.
Continue maintained host activation and remaining package quality checks.
The complete maintained unit task passes against the approved capacity build.
The real CLI process check then exposed an owned module initialization cycle:
`run-emitter.mjs` eagerly read `ExtensionRunner.prototype` before the class initialized.
Commit `7c210e871b` captures those methods in the defining runner module and imports live bindings.
The rebuilt real CLI returns the requested Qwen model and idle RPC state,
then exits with empty stderr and zero provider requests.
Next-decision receipts,
package types,
and scoped generator lint pass after that correction.
The [initialization incident](../troubleshooting/pi-sdk-staging.md#pi-120-native-cli-fails-during-owned-runner-initialization)
records the failing and passing process checks.
The original assessment budget,
resource-local 64-receipt cap,
and byte bounds remain unchanged.
The malformed traversal controls pass,
including omitted positions and forged observation identity.
Removing either of those guards in a disposable in-memory module causes its control to fail.
After the fixture split,
native build,
event receipts,
adopted-policy contrasts,
and context receipt controls pass.
The scoped fixture lint has zero errors and 988 warnings;
the lint task still fails because of those warnings.
The new generator and approved-change fixture initially report 99 warnings and one error.
The fixture now has explicit TypeScript owner interfaces;
the scoped generator and fixture lint reports zero warnings and zero errors,
and package type checking passes.
The existing built artifact still uses the 64-receipt limit.
Against that artifact,
all six semantic operating-policy contrasts,
Allow,
Deny,
Stop,
cancellation,
approval after reload,
and native host lifecycle checks pass after the timing change.
A disposable in-memory omission of the approved-source validator makes the policy-change control fail:
all three children execute instead of stopping after the first.
Its application marker and the unchanged built 64-receipt cap are asserted by the driver.
No product artifact file is changed by that omission control.
The user explicitly authorized continued attachment to the existing Codex worker cgroup on 2026-10-14
after automatic approval review rejected the wrapper's attachment step.
No resource limit changed.
The six native adopted-policy contrasts and the native context receipt controls pass after these changes.
The expanded event receipt controls also pass:
chunk rollover retains every phase,
oversized and excessive sources still reject,
result accessors are not invoked,
and completion failures retain their original thrown identity.
The first approval-context sequence passed Allow and Deny,
then Stop failed with `DependencyDeadlineError: Original dependency assessment deadline expired`.
An unchanged-code repeat passed Allow,
Deny,
Stop,
cancellation,
and reload,
followed by the native host lifecycle check.
The deadline failure remains recorded;
the repeat establishes recovery,
not a cause or permanent timing fix.
The semantic and approval runs also emitted the previously recorded logger sink verification timeout.
Passing assertions do not establish clean stderr or connect that warning to the deadline failure.
Its unit,
omission,
native consumer,
and package-quality checks remain unfinished.
The existing source-count and byte bounds have not been raised.
The fixture and logs are `${HOME}/temp/agent/pi-extension-isolation-20261014.test.ts`
and `${HOME}/temp/agent/pi-configured-extensions-isolated-20261014.log`.
Independent maintained bootstrap quality work also continues.
Its scoped lint reports `no-restricted-syntax/no-sync` for `statSync`
and `no-restricted-syntax/no-regex` for trailing-slash removal,
alongside untyped native-boundary warnings.
The policy designation now reads file metadata asynchronously;
the host captures outer resource options before awaiting it,
and the generated native services await the resulting loader.
Trailing-slash handling now uses string operations with the same single-slash behavior.
The package type check,
owned native build,
actual host lifecycle,
CLI RPC startup and clean shutdown,
and 188-module syntax check pass after this bootstrap change.
The actual CLI selects `qwen-token-plan-individual/qwen3.8-flash` and makes no provider request.
Maintained native interface typing and full lint completion remain unfinished.
The bootstrap now has explicit TypeScript contracts for its original SDK session and resource inputs.
Lifecycle and policy ownership records remain opaque at this forwarding boundary;
their existing native consumers retain authentication.
Generated callbacks retain their original identities through a callable-export bridge,
and a declaration companion describes the existing maintained-package re-export module.
The host generator and lifecycle fixture select `bootstrap.ts`.
The initial dynamic import of the generated session introduced a startup regression:
the native lifecycle task exited `13` with Node's `Detected unsettled top-level await`
at `bootstrap-dependencies.ts`.
Restoring the original static generated-session dependency through `session-disposal.mjs`
made that same lifecycle task pass.
The declaration companion retains an opaque function type and does not change callback identity.
The native host error constructor now identifies itself as `NativeCliHostError`.
Final package types,
the owned rebuild,
and scoped lint pass.
The actual lifecycle check passes all original construction,
reload,
replacement cancellation,
fork,
resume,
and incomplete-disposal retry cases.
The separate-process CLI also passes with the selected Qwen model,
zero external provider requests,
and empty shutdown stderr.
All 193 maintained native modules pass the final parser sweep.
The native README and bootstrap checkpoint were checked in rendered output.
The remaining host ownership modules were measured separately:
`host.mjs`,
`resources.mjs`,
and `resource-reload.mjs` report 111 warnings and one error.
The error is the positional `reloadHostResource` interface;
most warnings concern their untyped ownership and generated dependency boundaries.
Their maintained interfaces remain the next independent package-quality work.
The resource registration and reload boundary now lives in `resource-reload.ts`.
It retains the same WeakMap,
original record freezing,
and policy-versus-standalone routing,
with a destructured reload input at both generated native call sites.
Direct SDK fixtures can still register only their policy reload;
CLI ownership capabilities remain optional in the shared record contract.
New built-artifact controls cover original receiver,
options,
result,
registration identity,
duplicate rejection,
and unchanged thrown values.
The first format check exposed a missing documentation terminator in this edit;
it is corrected.
Formatting,
package types,
the owned native rebuild,
and all eleven built-consumer cases pass.
The source-built disposable campaign at `${HOME}/temp/agent/pi-resource-reload-omissions-RwMiiF`
passes its original baseline and fails each intended assertion after omitting duplicate-loader rejection,
original-loader rejection,
outer-record freezing,
or policy reload routing.
Every failing log was inspected;
the production artifact was not changed.
Native host lifecycle,
Allow,
Deny,
Stop,
cancellation,
Allow after resource reload,
and the separate Qwen CLI process also pass.
These use fixture responses with zero provider requests and zero genuine human approvals.
All seven changed TypeScript integration and test files pass scoped lint with no diagnostics.
All 194 maintained native modules pass syntax checking.
The native README and resource-reload checkpoint were checked in rendered output.
Native resource construction now lives in `resources.ts`,
renamed from `resources.mjs` on 2026-10-14.
Its static dependency bridge retains original generated constructors and observation readers.
The full CLI resource record has explicit policy-input,
cleanup,
and API-publication interfaces;
reload-only registrations still use the narrower shared contract.
Construction package types,
the owned native rebuild,
and original host lifecycle verification pass after formatting commit `fe068dd927`.
The remaining host session/disposal owner is still untyped;
adopted semantic policy binding now takes precedence over that independent quality work.

The first approved native dispatch failed because the inherited test loader derived dependency paths
from the codemode worker's disposable home.
Private `f3d40d82` retains the measured repository dependency root in the staging manifest;
the repeated native execution passed.
The strict TypeScript check for the source staging helper also passes.
Domain closure,
maintained delivery,
and installed approval verification remain unfinished.

The user resumed this handover on 2026-10-13,
then prioritized automatic Codex Bash resource limits before further Pi implementation.
The native Rust implementation is now in `package/codex-plugin/bash-limit`.
The user approved Rust tests through nextest with default build and test concurrency,
and the corresponding clarification in `AGENTS.md` and the TypeScript testing skill.

The installed binary is `$HOME/.local/libexec/codex-bash-limit`.
Trusted user hooks in `$HOME/.codex/hooks.json` initialize the worker child at session start
and prepend the cgroup attachment before each Bash command.
The persistent user unit is `$HOME/.config/systemd/user/piwork3285273.slice`.
The parent limits remain 4 GiB memory,
zero swap,
200% CPU,
and 256 tasks.
The user authorized raising only the CPU quota if needed;
that further increase has not been made.

The native fixture suite passes through nextest,
Clippy passes with warnings denied,
and two native Codex host runs reported 2 to 4 ms hook duration across 40 commands each.
The guard test failed as required when its attachment-failure exit was removed in a disposable worktree.
The [implementation record](../decision/codex-shell-worker-limits.md)
and [troubleshooting record](../troubleshooting/codex-shell-worker-limits.md)
contain the installation and native-host evidence.

The daemon accepted the live hook reload.
Escalated commands now enter the worker cgroup automatically without a manual launcher.
This already-running turn retained its old sandbox snapshot and blocked default sandboxed commands
with status 70 before their payloads ran.
The updated thread policy adds the child cgroup directory while preserving the other workspace-write restrictions.
The API applies that policy to subsequent turns.
The user offered to restart the session,
and the agent requested restart after this checkpoint was saved.

After restart,
first run a default sandboxed `cat /proc/self/cgroup`.
Expect a path ending in `/piwork3285273.slice/codex`.
Read the parent's memory,
swap,
CPU,
and task controls before any build or test.
If attachment is not active,
diagnose the exact hook or sandbox failure before doing unrestricted worker work.
Once verified,
the manual systemd prefix is unnecessary for Codex Bash calls.
Keep the Pi host outside the worker slice.

Then resume the finite Pi delivery queue in this handover.
The user supplied `pi --session 01a0dc52-0955-77f6-ae77-68a6e15bb12b`
and reported that OpenAI models currently do not work in Pi.
Use provider `qwen-token-plan-individual` and model `qwen3.8-flash` for Pi testing.
No Pi implementation or provider inference call was performed while completing the limiter priority.
No subagents or Advisor calls were introduced.

## Historical CPU-replacement pause

The user requested a safe pause for a CPU change on 2026-10-13.
That pause ended with the explicit resumption recorded in the restart checkpoint.
The retained evidence describes the state at the original pause.

At pause intake,
the process manager reported no running jobs.
The current handler-result receipt work is a pausing point:
its required native red,
implementation,
owner controls,
exact failure propagation,
canonical assessment,
history/reset behavior,
shared budget boundary,
and affected message/option-history regressions are recorded.
No provider inference call,
dependency installation,
or migrated-host activation was performed.

### Current code and evidence

- Maintained source-contribution accounting builds and passes package units and type checks.
  Latest type/unit verification is `proc_5306`.
  Native export identity,
  serializer scenarios,
  and action-policy results are recorded in
  "Maintained original source-contribution accounting".
  The publication test's reviewed formatter-produced declaration splitting was saved in `42baa8e662`.
- Private handler-result implementation is `7bde6d84`.
  The required red is `3d06c614` / `proc_0b0c`.
  Native receipt green is `proc_a410`;
  owner controls are `proc_2f20`;
  native exact-failure and canonical review are `proc_0389`;
  lifecycle is `proc_65b4`.
  Shared-budget controls passed in `proc_85bf` before its obsolete message-count assertion failed.
  Fixture correction `eaaea58b` passed message/history regressions in `proc_f0c6`.
- Source-contribution fixture lint is unresolved.
  Both formatter and ordinary no-fix lint were stopped without diagnostics.
  Do not infer a cause,
  claim zero lint errors,
  relax rules,
  or repeat the same unbounded verification loop.
- `request-source-domain` remains unconditional.
  Complete decision-specific domain closure,
  remaining maintained capture/compiler/host integration,
  installation,
  and genuine Allow / Deny / Stop and cancellation are unfinished.
  The historical 219-case input parent predates raw handler-result capture.

### Resume checks and next work

The canonical private progress file is
`/var/home/user/temp/agent/auto-mode-consumer-contract.mDLkyNoP/contract/lifecycle/runtime-continuation/current-progress.json`.
It retains the active `deliveryCompletion` queue and this pause.
The maintained package is `package/pi-plugin/auto-mode`.
Private SDK generators,
not installed or generated historical SDK files,
remain the source of the native changes.

After explicit resumption,
probe the new CPU and re-establish the authorized worker slice before any build or test.
Every shell and process launch still uses the restricted launcher:
shared slice `piwork3285273.slice`,
4 GiB memory,
zero swap,
CPU quota 200%,
and 256 tasks.
Runtime-only slice settings may disappear on reboot.
Do not raise policy,
source,
or preparation budgets to match the new hardware.
The existing cgroup probe needs `EXPECTED_MEMORY_MAX=4294967296`,
not its historical 2-GiB default.
Keep the Pi host outside the worker slice.

Check repository status and current process state before editing.
Preserve concurrent work.
Inspect recorded terminal logs rather than rerunning completed receipt campaigns.
Address the unresolved fixture-lint behavior with a bounded diagnostic,
not another blind formatter run.
Continue owner-backed domain resolution and maintained host delivery,
not another helper-only port by default.
Retain the domain hold until its actual evidence exists.

The user explicitly prohibited introducing the `openai` npm package.
The Decisions API intake in
[batched assessment](../planning/pi-auto-mode-batched-assessment.md#decisions-http-contract-intake-on-2026-10-13)
is documentation only.
Use direct HTTP if that transport is implemented;
no new model,
semantic qualification,
or paid inference campaign was adopted.
No subagents or further Advisor calls.
Do not change the settled `AGENTS.md` / `CLAUDE.md` decision about `PXQ`.

Earlier auto-push and manual push failures are recorded in
[Git credentials](../troubleshooting/git-credentials.md#native-push-wrapper-loses-the-inherited-environment).
Local commits were retained.
Do not infer remote publication solely from a successful local commit.

## Historical resumption after the previous break

The user explicitly resumed implementation on 2026-10-12.
The requested break was completed;
its stop instruction is now historical.
The no-subagent restriction remains unchanged.
The initial resumption kept the resource limits;
the later user authorization raised only the worker memory budget as recorded in
"Resource-budget correction".
The resumption probe found the expected slice limits,
no active managed jobs,
and no additional OOM kills beyond the recorded count of one.
The maintained kernel files have no intervening package commits since the pause.
Maintained built-artifact tests and full-package lint have since passed.
That resumption continued the named native governing-domain and runtime integrations;
the CPU-replacement pause now supersedes it.

At pause intake,
the process manager reported no running jobs,
and the restricted slice reported `pids.current=0`.
No build,
formatter,
test,
or agent is intentionally left running.
The remaining work is not blocked on generic delivery authorization.

The recent execution was not reliable enough.
Agent-authored command and fixture mistakes,
a formatter termination,
and excessive supporting work delayed integration.
The latest passing build and type check do not change the unfinished delivery status.
Do not turn the break into another investigation queue.

## Current delivery direction

The user rejected describing delivery or installation as withheld,
and rejected indefinite depth-first work.
Those descriptions were the agent's framing,
not a current user authorization barrier.
Historical statements about separate cutover authorization do not override this correction.
Missing implementation and verification are remaining work.

The completion target is the working migration,
not another private source-capture checkpoint.
Use this finite delivery queue:

- [x] Integrate the current constructor-description change into the native regression parent.
  Full native `proc_7042` passed all 82 outputs and 157 integrated input cases.
  Native parity `proc_9cc4`,
  lifecycle and description consumers `proc_a37a`,
  and action/source parents `proc_50d1` passed.
  Reuse the established required reds rather than opening a duplicate omission campaign.
- [x] Complete the code-owned decision consumer.
  Demonstrate justified approval,
  denial,
  eligible original approval requests,
  unresolved premises,
  and unspecified policy through the canonical native handler without injected final decisions.
  Replace unconditional missing-premise placeholders with decision-specific evidence from existing owners.
  The maintained domain and approval fixtures demonstrate these cases with authored semantic relations
  and independent prerequisite facts.
  Production semantic admission and broader unobserved native callbacks remain explicit unfinished boundaries.
- [ ] Integrate the implementation into the maintained package and necessary host path.
  The product must not depend on private scratch files.
  Run package checks against that integration.
  The maintained implementation,
  native build,
  installation,
  package type checks,
  and scoped behavioral checks are complete.
  Full package lint remains open with the October 14 measured diagnostics.
- [x] Build,
  install,
  and exercise the actual plugin through its host,
  including native approval and cancellation.
  The configured production CLI passes with the selected Hyper model,
  live Luna decisions,
  and the user's genuine Allow for the exact original disposable read.
  Separate scripted Stop and dismissal controls abort without child execution.

Further source work must fix a named failure of these delivery criteria;
standalone inventory expansion is not the default next step.
This direction does not turn absent coverage,
authority,
facts,
or semantic admission into evidence.
Preserve accepted governing-domain A and the models-estimate/code-decides boundary.
Ask only when a concrete user-only input or unsettled acceptance choice is reached.

Historical checkpoint sections retain their evidence and original scope.
The current delivery queue supersedes their stale next-step ordering.

The user chose to leave `AGENTS.md` unchanged for now,
not to adopt the proposed `PXQ` replacement.
[Issue #677](https://github.com/Aquaticat/Monochromatic/issues/677) records the observed failure,
exact corrections,
existing and proposed rules,
technical evidence,
and open design questions.
It is a separate workflow investigation,
not a gate on migration delivery.

## Maintained-package integration in progress

The shared AST adapter repair is complete:
issue #593 is closed after rebuilt consumer checks,
generated lockfile refresh `e801aa1e5f`,
and the 18-document check `proc_b14d`.
Documentation is committed in `7135c69304`.

### Landed code and verification

The deterministic directive resolver,
authorization constraints,
and outcome calculation now live in `package/pi-plugin/auto-mode/src/`:

- `afa2268e1d` adds `policy-data.ts`,
  `instruction-rule.ts`,
  `instruction-resolution.ts`,
  `authorization-requirement.ts`,
  `authorization-requirements.ts`,
  `guard-outcome.ts`,
  and their `public-api.ts` exports.
- `2b05330473` adds `policy-test-fixture.ts`,
  `policy-resolution.unit.test.ts`,
  `policy-data.unit.test.ts`,
  and `policy-conformance.unit.test.ts`.
  Tests import the maintained built package.
  The conformance test carries the existing bounded complete-world oracle,
  not a new provider trial.
- `c34e4acfb5` fixes the computed Symbol-key syntax reported by TypeScript `TS1109`,
  removes redundant field-count branches,
  and separates the identity-byte limit from the fact-count limit.
- `dd9cd4570f` adds explicit path argument decoding to the package's existing Oxlint tasks,
  using the existing mise helpers and unchanged lint rules.
  `proc_e2d8` exercised that route on the exact 11 selected files and reported real lint findings.
  Those findings were addressed without suppressions in the subsequent kernel commits.

The rebuilt artifact and package type check passed `proc_4615` with exit zero.
Despite that process's name,
its command contained only `build` and `lint:types`:
it did not run unit tests.
After resumption,
`proc_a39e` rebuilt the artifact and passed all three new kernel test files,
including the retained 14,400-program and 112,896-completion oracle.
`proc_41fe` rebuilt the final kernel and passed package type checking.
The full unit task passed in `proc_f4ce`,
including the conformance loop with the bundled logger flushed before and after it.
Its previous startup-drop diagnostic was absent;
intentional negative-fixture warnings and errors remained.
The following full-package lint was killed by the old memory limit.
After the memory-budget correction,
`proc_0f9a` passed all 77 linted files with 486 rules,
zero warnings,
and zero errors.
The pure resolver does not establish source authority,
coverage,
human approval,
or execution permission.
Its explicit internal premises flag is not a native admission boundary.
No migrated host/plugin was installed.

### Recent failures and resource state

`proc_3f44` never ran Oxlint.
The agent appended paths to an inline Node task without an argument-consumption contract,
producing `ERR_INVALID_TYPESCRIPT_SYNTAX`.
The initial package build then succeeded in `proc_55f3`,
but its type check found the agent-authored `TS1109` fixture error.
That syntax error is fixed,
and the later build/type run passed.

The no-argument package formatter `proc_0d52` reached the configuration build,
then terminated with `SIGTERM` without an Oxlint report or source changes.
The shared restricted slice recorded `oom 1` and `oom_kill 1`.
The initial calendar-window query excluded the event in the host's local time zone.
The later relative-window query identified `tsc` PID 4039089 in that exact scope,
with `constraint=CONSTRAINT_MEMCG` and this session's slice named as `oom_memcg`.
The same query identified the later `proc_f4ce` and `proc_f064` failures as cgroup OOM kills of `tsc`.
These are not the earlier systemd transport failures,
which happened before workloads started.

### Resource-budget correction

After seeing memory-shortage notifications,
the user explicitly authorized system or configuration changes to prevent interference.
The host probe reported 40,066,496 KiB available memory,
while the worker slice was capped at 2 GiB.
The cap was therefore raised to 4 GiB for this session,
using `systemctl --user --runtime set-property piwork3285273.slice MemoryMax=4G`.
Every subsequent command wrapper also uses `MemoryMax=4G`.
This changes the worker budget,
not any policy-judgment deadline or source bound.

`proc_0f9a` verified the real command and its child in the same 4-GiB scope,
then completed the previously failing full-package lint.
The slice's measured peak was 3,622,907,904 bytes,
which exceeds the old cap and fits the new one.
System-wide OOM protection remains enabled;
notifications were not muted.
Run memory-heavy checks serially within the shared budget.
The package Oxlint tasks now retain the tested single-thread,
512-MiB Node old-space,
and 768-MiB Go soft-limit settings;
`proc_3c0c` verified that persisted task route without caller-supplied memory settings.
It passed the same 77 files and 486 rules with zero warnings/errors,
and the OOM-kill counter remained at three.

All shell commands and process startups now use these aggregate limits:
4 GiB memory,
CPU quota 200%,
256 tasks,
and no swap.
Use the verified `systemd-run` wrapper with `piwork3285273.slice`;
never retry unrestricted.
Recheck the slice's runtime limits and child inheritance if the environment changes or restarts.
The Pi host itself was not moved into that slice.
No subagents or further Advisor calls.
The deferred `PXQ` proposal remains unadopted.

### Working locations and command wrapper

The maintained repository is `${HOME}/Monochromatic`.
The private implementation and progress record are under
`${HOME}/temp/agent/auto-mode-consumer-contract.mDLkyNoP`.
Its canonical progress file is `contract/lifecycle/runtime-continuation/current-progress.json`.
Earlier process logs are in `/tmp/pi-processes-rZNk8G/`.
Resumed process logs are in `/tmp/pi-processes-bBy8W5/`
(the current log directory since resumption on 2026-10-12).
These logs are temporary,
so the results and limitations in this handover are the durable summary.

Every shell command and process startup in this session uses this launcher,
including Git and diagnostic commands:

```sh
# Read-only repository status through the verified restricted launcher.
exec /usr/bin/systemd-run \
  --user --scope --quiet --collect --expand-environment=no \
  --slice=piwork3285273.slice \
  --property=MemoryMax=4G --property=MemorySwapMax=0 \
  --property=CPUQuota=200% --property=TasksMax=256 \
  /usr/bin/bash --noprofile --norc -c \
  'cd -- "$HOME/Monochromatic" && git status --short'
```

The slice also needs its verified aggregate runtime limits;
per-command limits alone do not bound concurrent jobs together.
Do not assume that runtime slice configuration survives a restart.
The empty `direct` child cgroup was never used as an alternate launcher.

### Resume from these concrete gaps

The maintained kernel build,
type check,
full units,
and full-package lint have passed.
The persisted resource-task route also passed;
continue the native decision-consumer work after the document closeout.
Do not rerun the full private native parent merely because a new session starts.

Then complete the existing native decision consumer,
rather than creating another standalone inventory:

- In private `contract/integration/action-policy/premises.mjs`,
  `inspectPolicyPremises` still unconditionally adds `request-source-domain`.
  `compiler.mjs` therefore keeps the native program incomplete.
  Replace the placeholder only with decision-specific evidence from existing owners.
- Private commit `436d83c2` now carries bootstrap `approvalHost` through the runtime,
  handler,
  reviewer,
  and evaluator into `approvalContext`.
  The incumbent binder uses the existing `judgment.sourceSnapshot`,
  not a new observer API or a post-closure recapture.
  Approval failures latch;
  restoring source or UI values cannot revive eligibility.
  The action-policy parent passed `proc_7367`,
  and the actual owned-session bootstrap/rollback regression passed `proc_df80`.
  Those checks establish routing and lifecycle mechanisms,
  not a canonical semantic-policy approval branch or genuine human approval.
- Production runtime and host integration still depend on the private generated graph.
  The prior static scan reached 115 modules,
  but was not a complete packaging inventory.
  The maintained pure kernel removes none of the remaining host-constructor integration obligations.

Preserve accepted all-host-input domain A and receipt at host processing boundaries.
Already-received inputs remain accounted for after hiding,
transformation,
rejection,
or non-transmission.
Unprocessed edits undone before receipt are excluded.
Unregistered potentially governing inputs remain unresolved;
roles and filenames do not establish authority.
Unknown premises do not become permission or automatic approval requests.
No operative instruction remains `unspecified`.
Prerequisite fulfillment grants no permission and overrides no prohibition.

Keep the original budgets,
source limits,
failure retirement,
native cancellation,
and Allow/Deny/Stop behavior.
No new hash locks,
request re-encoding,
parallel source registries,
provider trials,
or speculative source-capture campaigns.
Complete maintained integration,
then build,
install,
and exercise the actual host/plugin.

For resumption,
inspect the maintained files and the named native consumer gaps,
not every historical checkpoint in this document.
Report actual executed checks and the remaining delivery boundary.
Ask only for a concrete user-only input or genuinely unsettled acceptance choice.

## Original rendered prompt-input contributions

The serializer consumer now accounts for original host,
append,
and context base inputs through the existing `runInputCarriers`.
It also accounts for already-retained run replacements through their original run publication.
Neither route invents a source inventory,
reparses the request,
or re-encodes it.

Each nonempty rendered input retains its own original source reference.
Independent equal append or context inputs do not borrow the host's authority.
Removed inputs receive no representation association,
but non-transmission does not revoke a separately governing instruction.
Replaced or reordered inputs retain their replacement occurrence rather than claiming the displaced base identity.
Empty run inputs and native-default rendering remain separate cases.

The canonical consumer requires the original source view's request receipt,
the original terminal request,
and the original run carrier or replacement publication.
Associations remain partial contributions to a containing field,
not byte ranges or complete composite coverage.
They add no priority,
permission,
or delegation.

Missing host association failed `proc_c437`;
missing replacement association failed `proc_09d9`.
The extended renderer matrix passed `proc_7111`,
and source-owner controls passed `proc_85bb`.
The original owners still bound context entries to 64 and append inputs to 32.
The actual-serializer bound control retained 97 separate partial contributions,
including repeated explicit republications of one original source.
Independent sources are not merged.

A proxy that changed its sections between validation and serialization failed `proc_50ca`.
Association now excludes proxy envelopes and arrays through Node's native proxy predicate.
The fixed profile also requires the native normalized message-only context.
Accessor and proxy exclusions do not replace original identity checks.

Full native `proc_efb9`,
action `proc_4e12`,
and ten-document render `proc_bc5d` passed after replacement support.
The already-retained forced prompt now has a separate original publication capability
created by the existing run owner.
The actual native forced-prompt projection publishes its new message through that owner.
The serializer associates only that original message with the original forced input;
equal text in a copied message or displaced base input supplies no link or authority.
The initial forced-source control failed `proc_84e0` and passed `proc_6a2e`.
Full native `proc_f939`,
action `proc_bf15`,
and consumer identity `proc_3056` passed copied,
equal-text,
empty,
and permanently retired forced-projection controls.
Omission `c316f17` failed `proc_305b` by associating an independent equal-byte copy.
The identity guard is restored.
Full native `proc_38d6`,
action `proc_3eaf`,
and ten-document render `proc_f55b` passed.

Already-retained custom sections now use original `mappedPromptInputs` and the full sections publication.
A nonempty `addendum` or `project_context` override displaces its base or replacement input contribution;
an empty override preserves the native base rendering.
This affects representation only,
not separately established governing authority.
Section names and equal bytes grant no authority.

Missing-association red `proc_ff08` preceded green `proc_48c0`.
Expanded matrix `proc_49c5` passed 18 modes,
including replacement shadowing,
empty overrides,
forced precedence,
copied sections,
malformed Unicode,
native-default rendering,
and invalid section names.
Source-owner and inherited map-bound controls passed `proc_73fc`.
Omission `proc_30bf` incorrectly credited a displaced context input;
omission `proc_cb23` incorrectly accounted for a source outside the original run's mapped inputs.
Both guards are restored.
Full native `proc_744b`,
full action `proc_09ee`,
and ten-document render `proc_fed3` passed.

The direct native-generated default preamble,
tool-discovery guidance,
and documentation guidance now retain their original run and render association.
They use existing generated-input occurrences,
not a second inventory or authority inferred from native placement.
Custom-host rendering does not construct these sources;
truthy section overrides and forced prompts prevent their contribution association.

Missing native-fragment association failed `proc_b831` before initial green `proc_c667`.
Expanded visibility matrix `proc_4165` passed 24 modes;
original source/render controls passed `proc_814f`.
Displacement omission `c79930b` failed `proc_1ffa` by associating overridden documentation guidance.
Render-binding omission `1dae346` failed `proc_d254` by accounting for a source from a foreign render.
Both guards are restored.
Full native `proc_fd1a`,
action `proc_5a42`,
and ten-document render `proc_d198` passed for these direct generated fragments.
Project-context introduction and skill-usage guidance now use the same original run/render linkage.
The skill formatter constructs leading blank lines that native section construction trims;
the source remains unchanged and its contribution follows that known native transformation.
No skill-file identity or authority is inferred.
Disabled skills,
absence of a selected reader,
custom-section overrides,
forced prompts,
and independent payload copies retain their distinct behavior.

Missing context-introduction association failed `proc_0f4b`;
initial context/skill consumer `proc_0500` and the 32-mode matrix `proc_edd5` passed.
The existing base-bound fixture still has 97 base-input contributions;
the separately retained native introduction makes 98 total contributions without increasing any input limit.
Full native `proc_4f7c`,
action `proc_2217`,
and ten-document render `proc_bd76` passed for context/skill guidance.
Default-rule sources now retain the fixed native builder's observed `nativeRuleInsertion` result.
The recorder measures the original local rules-array length around the untouched `addRule` call.
It preserves every constructed default occurrence,
including rules suppressed by normalization or deduplication.
Only an inserted occurrence can receive a contribution association,
and a subsequent section override or forced prompt still prevents it.
Equal text emitted by an independent guideline supplies no identity for the suppressed default.

Missing insertion association failed `proc_ed24`;
initial consumer `proc_bfe3` and the 38-mode rendering matrix `proc_a236` passed.
The `proc_a236` command forwarded later task names as script arguments,
so it did not verify builder parity or render-owner controls.
Corrected separate invocations passed those checks in `proc_4b3c`.
Consumer controls passed `proc_52c2` and `proc_0b8d`.
Producer omission `proc_3a5c` fabricated a suppressed occurrence's contribution;
consumer omission `proc_1fd3` incorrectly accounted for a `not-inserted` fact.
Both guards are restored.
Full native `proc_a618`,
action `proc_412e`,
and eleven-document render `proc_d1a5` passed.
The new fact concerns only local native insertion,
not instruction authority,
complete governing coverage,
or permission.
Already-retained tool snippets now use the original mapped-input sources.
Only selected,
visible,
nonempty snippets in native-default rendering receive a contribution association.
Custom prompts,
tools-section replacements,
and forced output do not borrow snippet identities.
Equal inactive snippets remain distinct and unassociated.

Native declaration hiding comes from the tool's `prepareLoadout` hook;
the SDK overwrites `hiddenTools` during loadout preparation.
The controls exercise that actual hook and inspect the resulting original options.
A hidden read tool still permits native indirect skill guidance,
without exposing its declaration or naming the reader.

Missing snippet association failed `proc_9121`;
initial `proc_04cd`,
expanded visibility `proc_4f4e`,
and consumer-owner `proc_43a1` checks passed.
Hidden-declaration omission `proc_882a` and selected-tool omission `proc_d76d`
each wrongly associated a non-rendered snippet.
Both guards are restored.
Full native `proc_18cf`,
action `proc_7c12`,
and eleven-document render `proc_3594` passed.
Already-retained skill descriptions now use their original source,
run,
and sections publication.
Their contribution follows the native XML escaping rather than rewriting the retained description.
No skill-file identity,
instruction authority,
or delegation is inferred from the description's companion metadata.

Disabled entries,
no selected reader,
empty descriptions,
skills-section overrides,
forced output,
and independent payload copies receive no description contribution.
Hidden readers still support the native indirect route.
Equal enabled descriptions retain separate original occurrences.

Missing description association failed `proc_f07f`;
initial `proc_cd76`,
55-mode visibility/escaping `proc_1cc5`,
and consumer-owner `proc_848f` checks passed.
The XML control includes closing-tag delimiters,
ampersands,
quotes,
backslashes,
and newlines.
Disabled-entry omission `proc_0567` and reader-availability omission `proc_0dff`
each wrongly associated non-rendered descriptions.
Both guards are restored.
Full native `proc_8600`,
action `proc_4807`,
and eleven-document render `proc_9301` passed.

Original prompt and tool guideline sources now collect insertion evidence within their native render.
The existing source records are constructed as private drafts from the frozen render input,
updated directly by the fixed native loop observations,
then sealed and published as those same records.
No parallel source inventory or request parser is introduced.

The observer uses the existing active-render window and its existing cleanup path.
Late calls cannot change or poison a published run.
A caught observation failure still retires the original render,
including when its thrown value is `false`.
Unobserved legacy sources receive no insertion fact.

Missing guideline association failed `proc_99ae`;
initial native consumer `proc_f2dc` passed.
Original-source/lifetime and builder parity controls passed `proc_8829`;
the 69-mode visibility matrix passed `proc_b1b1`;
consumer-owner checks passed `proc_c84f`.
Equal entries within one array and equal prompt/tool entries remain independent sources.
Only the actual inserted occurrence contributes,
and later rules-section or forced overrides still prevent association.
Failure-ledger omission `proc_acd7` allowed a caught observation error to disappear;
render-window omission `proc_3d7a` poisoned the old scope on a late call.
Producer omission `proc_95dc` fabricated a suppressed contribution;
consumer omission `proc_5308` incorrectly accounted for a `not-inserted` fact.
Every guard is restored.
Full native `proc_f733`,
action `proc_8b6b`,
and eleven-document render `proc_1cf2` passed.

## Original handler-message contributions

Original `before_agent_start` message inputs now retain their source identity through actual native construction,
manager copies,
session projection,
custom-to-user conversion,
and user-content serialization.
The existing handler sources and active run ancestry are reused.
Text-block arrays stay original JSON sources;
there is no joined Markdown,
additional request parser,
or second source inventory.

Only actual original-capability transfers establish these links.
Equal independent messages and context-edit replacements cannot borrow an original handler source.
Payload custody owns new text arrays and blocks before the OpenAI client's awaited authentication,
without freezing callback-owned objects.
Historical receipts discard transient payload nodes and do not depend on later mutations of those aliases.

Required missing-association control `proc_51d1` failed;
canonical handler accounting passed `proc_e424`.
Original emitting-run/ancestry consumer controls passed `proc_9621`.
Installed/private conversion parity passed `proc_1bd1` with 15 SDK cases and 60 provider cases;
existing serializer regressions passed `proc_6131`.
Private preparation now copies 11 native modules and leaves installed files unchanged.

Nested equal replacement failed `proc_2adc`;
descriptor-restoration revival failed `proc_9061`.
Both defects are fixed:
existing aliases retain content and part identities,
and observed currentness failure retires eligibility before structural rejection can short-circuit it.
The restored owner controls and 24-mode native handler matrix passed `proc_7137`.
These include empty/nullish content,
structured text,
equal independent inputs,
context edits,
payload copies,
borrowed input mutation,
malformed Unicode,
and pre/post-materialization payload mutations.

This is partial source lineage only.
System-like labels and user transport roles supply no authority or permission.
Independent boundary drafts,
`sendCustomMessage` deliveries,
and general persisted-session provenance remain outside this retained-handler profile.
All guards are restored.
Expanded 26-mode handler verification passed `proc_2471`,
including an actual later review using original run ancestry and isolation of new prompts from persisted equal text.
Nested part-identity omission `proc_cecc` republished an equal replacement;
context-edit omission `proc_dd37` credited equal replacement text;
nested payload-custody omission `proc_6256` exposed mutation during awaited authentication;
emitting-run membership omission `proc_6486` accounted a copied run.
Each targeted control failed.
Restored full native `proc_41bf` and action `proc_de99` passed.
Eleven-document rendering passed `proc_c7e4` before this final evidence update.

The unassociated ordinary request domain and other unconsumed producer relationships remain explicit.

## Maintained policy computation in the native prototype

The native prototype now imports the maintained package's actual built exports for directive resolution,
authorization requirements,
outcome calculation,
and shared input-budget arithmetic.
The canonical private implementations were replaced with import adapters,
not copied into another runtime implementation.
The existing SDK stager resolves the package-declared entry before fixtures change their home directory.
No source registry,
request parse,
or completeness flag was added.

The maintained input-budget implementation is
`package/pi-plugin/auto-mode/src/input-receipt-budget.ts`.
Build/types passed `proc_eeaf`;
full package units and changed-file lint passed `proc_fc5d` with zero lint diagnostics.
`proc_512e` verified exact maintained function identities and passed all 32 canonical compiler scenarios.
`proc_b709` verified the actual native combined 64/65 receipt boundary through the maintained helper.
The built-artifact guard omission `proc_1bd6` failed the required exception assertion in a disposable copy.
Its first attempt was a minified-whitespace fixture mismatch,
not a guard-sensitivity result.

Clause-to-fact lowering is also being moved into maintained TypeScript,
with the shared priority validator reused by both the resolver and lowering.
`proc_ca39` passed its build/types after correcting TypeScript `TS9010` with an explicit exported type.
The full unit task `proc_8ac8` passed its authored meaning,
prerequisite,
shape,
and ownership cases.
Final build/types passed `proc_c0ac`;
full units and changed-file lint passed `proc_186b`.
Full package lint `proc_be6e` covered 81 files and 486 rules with zero diagnostics.
`proc_173a` verified the maintained lowering export and catalog agreement,
then passed all 32 canonical compiler scenarios.
Action/source parents passed `proc_527c` and `proc_33db`,
including incumbent fixture approval choices and irreversible retirement.

These native scenarios still retain the independent governing-domain obligation.
This is maintained computation used by the private native path,
not completed domain closure,
a maintained host launch path,
or installed activation.

## Original callback-result receipt

The accepted processing-boundary domain includes the complete original extension result received by the host,
not just fields selected for native processing.
Private `3d06c614` adds the separate required acceptance task `test:handler-result-domain`.
In `proc_0b0c`,
the native-consumed message positive control passed;
the untransmitted result-field assertion failed.
The native run owner had retained processed messages and options,
not that complete received result.

Private `7bde6d84` captures defined `before_agent_start` results before native field selection.
The observer binds to the original Session input pool before handler dispatch,
so manager reset cannot redirect the observation into a replacement pool.
Original one-argument frames own input-only publications;
borrowed result objects receive no publication or freeze.
The metadata copier retains supported non-enumerable data and charges shared references per occurrence.
The existing source,
copier,
Session,
and resource bounds remain unchanged.

`proc_a410` passed both native receipt assertions.
`proc_2f20` passed owner controls for identity,
mutability,
scalar and empty results,
metadata copying,
getter/proxy refusal,
the shared 64-receipt limit,
source bytes,
exact unexpected failures,
and irreversible retirement.
`proc_0389` passed exact native false,
undefined,
and proxy observation failures,
with zero proxy traps,
then passed original-source canonical review with two local requests and one assessment.
`proc_65b4` passed settlement,
later borrowed-output edits,
history,
root reset,
old-pool observer binding,
and profile-refusal parity.
Native result processing continued without invoking the unread getter.

The budget portion of `proc_85bf` passed the combined resource/Session 64/65 boundary
both with and without a returned result.
Its message-history test then exposed the old fixture count,
`23 !== 18`.
Private `eaaea58b` explicitly checks each complete original result receipt,
its metadata,
identity,
order,
and settlement retention instead of dropping those sources.
`proc_f0c6` passed the message-history and handler-option-lifetime tasks.
The historical 219-case input parent was not rerun and predates this receipt addition.
This receipt correction does not close `request-source-domain`.

## Maintained original source-contribution accounting

Main `2ea68db692` ports `consumeSourceAssociations` and `PolicyInputBindingError`.
Type and helper siblings separate original run publications from message origins.
Private `701f2585` changes the existing consumer and input-error owner to maintained exports.
No second source registry,
request parse,
or completeness assertion is introduced.
Native owners still authenticate the source graph and revalidate its lifetime.

The consumer retains base,
replacement,
forced,
mapped,
generated,
skill,
guideline,
handler,
expanded-prompt,
and historical message contribution paths.
Historical origins do not enter active ancestry.
Input-only Session origins cannot impersonate run material through absent run identifiers.
Only owned result containers are frozen.

Initial build `proc_b641` succeeded,
but its type check reported `TS18048` for the optional request receipt.
An explicit missing-receipt guard now supplies the required narrowing.
Built-artifact identity,
publication,
and copied-origin controls are written.
Main `13f4df97f7` passed build/types `proc_1d88`
and built-artifact package units `proc_6abf`.
Exact maintained consumer and error identity passed in `proc_b724`.
Its serializer parent failed the enumerable-getter case with no completed review.
The focused unchanged-runtime case passed `proc_0754`;
the complete serializer parent then passed `proc_55aa` with 16 scenarios,
empty child stderr,
and zero providers.
No cause or fix is inferred from that recovery.
The complete action-policy parent passed `proc_108b`.

Full lint remains unfinished.
Formatter runs were stopped without diagnostics,
including isolated fixture runs.
Removing unnecessary recursive ancestor types passed build/types `proc_12c4`,
but did not establish a formatter fix.
Concrete fixture fields and fixed record shapes likewise have no verified lint result.
The ordinary no-fix lint `proc_e7e5` also produced no diagnostics
before being stopped at 426 seconds.
The fixpoint loop alone does not explain the delay.
Current fixture shapes passed types and package units in `proc_5306`.
Stop blind lint reruns;
the unresolved lint item remains explicit while independent migration work proceeds.
No lint rule or resource bound was relaxed.
This increment does not implement decision-domain closure.

## Maintained bounded unit incorporation

Main `f82323d386` ports `unit-adoption.ts` and the structural pair selector,
with binding,
meaning,
and type siblings.
The native compiler now imports those maintained consumers through the existing stager.
Original batch authentication and dependency revalidation remain with native owners.
The accepted same-source two-unit profile and 64-source opportunity bound remain unchanged.

Scope still requires the original member,
exact adopter/target projections,
original authority/source membership,
and matching original request scope.
Primitive adopter meaning cannot be silently combined with adoption-only control meaning.
Target permission,
prohibition,
prerequisite,
or no-op meaning remains independent of incorporation.
Independent implementation-fact gaps stay unresolved.

`proc_b660` passed initial build/types.
Built-artifact controls exercise meaning combinations,
structural selection,
original identity,
authority/request scope,
prior lowering,
and malformed or unadmitted inputs.
`proc_376a` reported documentation,
limit naming,
and nullable-number condition diagnostics.
`a56b345955` preserves the same limit with an explicit constant and checks prior lowering explicitly.
`proc_2cf2` passed formatting/lint with zero diagnostics.
Final implementation `af11516512` passed build/types `proc_c342`.
`cd4e211d21` adds original reader and lost-state controls.
`proc_a53e` passed package units and type checks;
`proc_53ad` passed 12-file lint with zero diagnostics.
`proc_54c8` passed exact maintained-export identity and all 36 native incorporation scenarios,
with empty child stderr and zero provider requests or protected executions.
`proc_c2f8` passed the complete action-policy regression task.
The native scenarios preserve source/member identity,
registered priority,
independent target meaning,
implementation-fact gaps,
cancellation,
and preparation-inclusive expiry.
No domain closure,
production semantic qualification,
installed host,
or genuine approval result is claimed.

## Maintained source-local heading consumer

Main `876b63c06c` adds `package/pi-plugin/auto-mode/src/heading-context.ts` and its reference types.
Private `bbd6a767` removes the duplicated heading-consumption body in favor of the maintained export.
The native compiler still authenticates original batch membership and candidate issuance,
and revalidates dependencies after code policy callbacks.
No heading estimate lowers a directive,
assigns authority,
or closes the host domain.

`proc_7cbf` passed initial build/types;
`proc_99c3` passed formatting/lint on the changed files with zero diagnostics.
Built-artifact controls cover equal independent groups,
original references and callback receiver,
admission/abstention,
invalid selection and question shapes,
and exact falsy callback failures.
A disabled empty selection remains supported;
missing policy or candidate handles reject selected malformed input.
Those malformed shapes are outside native owner-issued records.
Final Main `8f6bf8e93b` passed build/types `proc_3577` and package units plus four-file lint `proc_a2d4`.
The lint result had zero diagnostics.
`proc_4580` passed exact maintained-export identity and all ten native heading scenarios,
with empty child stderr and zero provider requests or protected executions.
`proc_506b` passed the complete action-policy regression task.
Original equal-text groups remain distinct;
heading `none` discharges only its own source-local content obligation,
not the independent request-domain hold.

A read-only Git diff invocation emitted `@monochromatic-dev/module-logger`'s diagnostic:
`logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify`.
It exceeded its command deadline before the planned commit/build calls started.
The existing [sink-verification investigation](../troubleshooting/cli-git-logger-sink-verify-timeout.md)
does not establish a cause for this occurrence.
The background diff reader `proc_b03c` completed and confirmed formatting-only changes.
No logger setting,
Git policy,
or resource limit was changed;
the diagnostic does not establish a cause for the delay.

## Maintained bootstrap semantic admission

Main `ad46df49bf` ports configured-policy copying and original program-profile matching into
`package/pi-plugin/auto-mode/src/semantic-admission-policy.ts` and `semantic-admission-scope.ts`.
Private `dc349792` delegates through the incumbent maintained-artifact link.
The native adapter still fixes definition IDs and the expected model from the actual wire contract.
A caller's extra model field is not read or used to override that contract.
No default acceptance,
production qualification,
source authority,
or complete-domain policy is added.

Initial build/types passed `proc_cf5e`.
Formatter `proc_648b` reported the nullable-return type,
an unbound fixture method,
and style diagnostics.
The configured-policy copier now requires present input;
absence stays with native bootstrap and still means no configured consumer.
No lint rule was suppressed or relaxed.
Final Main `0f79ccf033` passed build/types `proc_329e` and package units plus six-file lint `proc_8f09`.
The lint result had zero diagnostics.
Private `0ca19921` verifies that native bootstrap preserves absent policies and ignores caller model overrides.
`proc_4711` passed exact maintained-export identity and all 32 native compiler scenarios,
with empty child stderr and zero provider requests.
`proc_784c` passed the complete action-policy regression task.
These checks do not qualify model accuracy,
close the governing domain,
or install the migrated host.

## Maintained whole-request content consumer

`package/pi-plugin/auto-mode/src/request-material.ts` and `request-material-obligations.ts`
now own the whole-request content consumer and its independent unresolved-use obligations.
Private `43db5919` replaces those implementations with exact maintained-export links.
`SemanticAdmissionPolicyError` is shared with the native semantic-policy consumer.
Original batch authentication,
request-source ownership,
and post-callback dependency checks remain with the incumbent native owners.
No authority,
directive,
complete domain,
or production semantic qualification is inferred from the content estimate.

Initial build/types passed `proc_664d`.
Built-artifact tests cover equal-byte foreign requests,
model attribution,
Boolean acceptance,
original callbacks and failure values,
borrowed mutability,
and independent potential-instruction use without a domain hold.
Final source `bf55373ca8` passed build/types `proc_6cd4` and package units plus eight-file lint `proc_fd4c`.
The lint result had zero diagnostics.
`proc_e93d` passed exact maintained-export identity and all nine native request scenarios,
with empty child stderr and no provider requests or protected executions.
`proc_865d` passed the complete action-policy regression task.
`proc_cf47` passed the native permission/prohibition/none domain contrasts:
request-content `none` still retains `request-source-domain`,
while `potential-instruction` independently retains `request-material-instruction-use` as well.
The domain consumer,
remaining capture/compiler/host integration,
installation,
and genuine migrated approval remain unfinished.

## Maintained native hook lifetime

`f04d34e79d` introduces `package/pi-plugin/auto-mode/src/native-runtime-hooks.ts`.
Private `990e177c` removes duplicate hook installation,
rollback,
and detachment bookkeeping from `contract/integration/native-batch/runtime.mjs`.
The original native request,
judgment,
collector,
and tool owners remain unchanged.
Native bootstrap still checks original identity,
idle state,
and an unoccupied executor before owner construction.
The helper is not an authority or completeness certificate.

Removal remains reverse-order and retryable for failed removers.
Independent installation,
cleanup,
and restoration failures retain their original values.
Incomplete cleanup leaves the executor occupied.
Detachment still requires actual Session disposal followed by Agent idle,
and leaves the invalidated original-request observer on the stream.
Only the helper's returned method container is frozen.

`proc_e830` passed the initial build and type check.
The unit portion of `proc_7d13` passed;
its lint portion failed with 50 warnings and five errors.
`08cce3b59a` adds native slot failure controls and addresses the diagnosed lint findings.
`393983938a` / `proc_f09e` passed the final build and type check;
`proc_4407` passed package units and six-file lint with zero diagnostics.
The hook controls cover installation,
partial slot writes,
rollback,
independent cleanup failures,
release retry,
and exact falsy/proxy failure values.

The first native cutover `proc_f31e` exposed an outdated source-offset assertion.
Pre-cutover comparison `proc_9b32` reproduced the same failure.
Private `0a69355c` now explicitly checks the original handler-option sources before constructor metadata.
`proc_6fa5` passed the maintained native cutover and the pre-cutover comparison,
including original source/judgment ownership,
installation rollback,
retryable native cleanup,
and native start-error preservation.
`proc_a1b3` passed all 32 compiler scenarios with empty child stderr and the original-source variants.
Disposable omission `proc_49ef` reached the intended missing-exception assertion:
`An undisposed native session must retain policy ownership`.
The original maintained artifact was not modified.
`proc_9bdc` passed the native review-failure controls.
`proc_0354` passed full-package lint on 89 files with 486 rules and zero diagnostics.
`proc_5fe0` passed 20-document lint/render;
the rendered hook sections were read back.
The hook helper does not close the governing domain,
move the remaining capture/host graph,
or activate the installed plugin.

## Decision-domain consumer frontier

`2b0d054c` / `proc_382f` isolates the remaining decision-domain obligation through the native provider serializer.
Authored permission,
prohibition,
and no-instruction cases each bind 26 original sources and interpret 26 clauses.
Their five represented request contributions are accounted for;
those contribution records still describe partial origin coverage.
The only remaining diagnostic in these cases is `request-source-domain`.
Every case remains unresolved with no protected execution or provider request.
Authored semantic answers establish neither model accuracy nor original-domain completeness.

`proc_6f9b` runs the separate resolution acceptance.
Its permission case fails the required domain-closure assertion;
the later prohibition and no-instruction acceptance cases were not reached.
This does not justify deleting the hold:
original-owner domain membership,
applicable scope,
and valid delegation still need their code-owned consumer.
No closed-host-profile policy or new completeness interface has been adopted.

A preparatory control exposed an independent obligation hidden by the universal hold.
`373eab30` / `proc_8594` showed that an admitted `potential-instruction` request estimate
left no specific use obligation when other issues were absent.
`c5375b8b` now retains `request-material-instruction-use` independently.
Potential material may repeat indexed text;
the new diagnostic creates neither another directive nor uniform authority for the request.
Pure request,
semantic-policy,
and compiler tests passed `proc_b09d`.
`proc_01d8` passed the native whole-request scenarios with empty child stderr
and the decision-domain contrasts.
The potential-instruction contrast retains both `request-source-domain` and `request-material-instruction-use`;
the request-content `none` contrasts retain only `request-source-domain`.
No protected operation or provider request occurred.

## Original before-agent-start option inputs

The decision-domain work found an unconsumed original input already present in the run ledger.
`c567fb4f` / `proc_e511` changes a section in one callback,
then passes it to another callback that restores the section before rendering.
The native ledger retains the second callback's original options,
but the governing source set omitted them.
The retained-final-state control passed;
the restored-state assertion failed.

`900864cb` projects `handlerObservations[].before.options` through the existing run owner.
Sources use `native-run-handler-options-input`,
location `handlerOptions`,
and view `original-handler-options-view`.
Original run membership and observation ordinal authenticate the projection;
JSON shape does not.
The projection is `native-content-json`,
charged to the run ledger and existing source bounds.
Native callback behavior and request serialization are unchanged.
No source receives authority,
registration,
or payload eligibility.
`proc_dfce` passed source bounds,
retained/restored state,
refresh ancestry,
and canonical assessment with the original value absent from the native request.

The first projection exposed only active run ancestry.
`8cac8e60` / `proc_73be` and `proc_5a69` demonstrated loss after settlement and a subsequent prompt.
`fd78cf05` now retains original references through the existing run history,
scoped by the native root captured when the run begins.
Input-only publications belong to owned observation frames,
not borrowed options.
The original handler references precede manager references in the combined base view,
so manager-only append handoff keeps its existing prefix proof.
Session receipt usage now includes those handler sources before the shared resource/Session check.

`proc_6bc3` passed settlement,
next-prompt identity,
root-reset exclusion,
and actual shared 64/65 receipt boundaries.
Manager and resource counters alone were below the limit in the refusal control.
Historical inputs do not become active ancestry.
`8f7161f8` adds live-root and old-alias checks;
`proc_dfd4` passed their owner controls and the manager-handoff lifecycle cases.
Unpublished original callback inputs retain a source-publication failure rather than disappearing.
`946a4cae` / `proc_2f77` passed the affected message-history regression and final option consumers.
`85aae781` / `proc_9251` passed all 219 input-parent cases with empty child stderr and no provider requests.
`a54546f6` / `proc_c28e` separately passed historical-input canonical assessment:
the next request assesses the same original source,
without promoting the prior prompt into current ancestry.
That follow-up is now in the parent but was not part of its 219-case run.
Affected action/source parents passed `proc_b6d4` and `proc_9144`,
including source identity,
manager-only handoff,
approval ownership,
and irreversible retirement.

The run-owner helper task initially failed with Node `EEXIST` on a historical output directory.
`b672127d` uses unique disposable output directories and preserves historical artifacts.
`proc_61c6` passed the helper controls before the subsequent history/root changes.
This work does not close the host domain or provide maintained host installation.

## Native metadata-query receipts

`94f39f5e` / `proc_666b` reached a separate required retention failure.
The control changes a tool namespace inside a callback,
reads it through the actual native `getNamespace` capability,
then restores the definition before the callback returns.
When the callback publishes the value as a description,
retention succeeds.
When only the native query processes it,
the original instruction is absent from governing sources.
This is a processed live edit,
not an edit undone before processing.

The callback input and output snapshots did not close this return-value path.
`3705ded2` added query-result receipts;
`proc_cab7` passed the published-output and query-only contrast.
A failure-path test then demonstrated that query arguments also need an entry receipt:
`f98aa363` / `proc_6d89` lost an extra instruction argument when the native namespace getter threw.
`f87569f7` now captures inputs before lookup and outputs after successful lookup.
Both phases use the existing Session pool and shared limits without deduplication.

The observed capabilities are `getExposure`,
`getNamespace`,
and `getPromptGuidelines`.
Sources use kind `native-tool-loadout-query-input`,
origin `native-tool-loadout-query`,
location `toolLoadoutQuery`,
and view `original-tool-loadout-query-view`.
Companion metadata distinguishes `queryPhase: input` and `queryPhase: output`.
Publication belongs to the actual observation frame,
not the borrowed result,
argument,
or query function.
Receipt creates no authority,
registration,
payload capability,
or permission.

`78cf7af5` / `proc_99eb` passed owner and native controls.
They preserve query receiver behavior,
arity,
returned array identity,
one/one namespace getter counts,
and native getter failure reporting.
Unexpected false and undefined failures remain exact at both observation phases.
A callback catching the failure cannot restore eligibility or permit later native consumption.
Independent callback/reporting failures remain represented rather than replacing the observation failure.
Expected profile refusal preserves the raw native result and invokes no proxy reflection.
Original arguments remain retained when native lookup fails.
Owner tests retain source-byte,
serialized-record,
shared occurrence,
and aggregate expanded-container bounds.

`proc_df5e` passed canonical query-only review and the lifecycle controls:
pre-request receipt,
pre-binding growth,
post-binding growth,
oversize output after a retained input,
reset,
and disposal.
The canonical query value is absent from the native request but present in original source bindings,
assessment policies,
and indexed questions.
It remains unresolved with two local exchanges,
one authored assessment,
and no protected execution or provider request.
The query controls now belong to the input parent;
its historical 198-case pass predates these additions and is not evidence for them.
Affected compiler/source regressions passed `proc_a0e6`:
32 compiler cases with empty child stderr,
plus the original-source variants and construction/cleanup controls.
No provider calls or genuine human approvals occurred.

This repairs a named processing boundary,
not decision-specific domain closure,
maintained host delivery,
or installed activation.
The initial query staging failure `proc_3495` was an unmapped owned import;
`b8077a0e` added only its exact URL to the incumbent import map.
The first native parity attempt `proc_e6cf` expected exposure `default`;
native exposure was `direct`,
and the fixture was corrected without changing native behavior.

## Maintained query observation lifetime

`56b562852f` adds `src/native-query-observation.ts` and built-artifact tests to the maintained auto-mode package.
`proc_b197` passed build and types.
The private adapter now delegates its failure lifetime to the package export;
original argument-frame construction and returned-value identity remain local.
The existing stager and export-identity test include this function.
Final source is `e64644ae85`.
Build/types passed `proc_95dd`;
full package units and four-file lint passed `proc_02e6` with zero lint diagnostics.
The native import cutover passed `proc_c3b0`,
including exact export identity,
owner and native failure controls,
query-only review,
and lifecycle retirement.
`71cd85fb` / `proc_921c` removed the continuation check in a disposable built copy.
The intact control passed;
the changed artifact reached the required missing-exception assertion after successful recapture.
The maintained artifact was not modified.

No lint protection was disabled.
An owned optional failure field replaced the root-level mutable binding;
the test fixture uses native `Generator.throw` to produce foreign JavaScript exceptions.
This moves failure-lifecycle code out of scratch,
not source ownership,
governing-domain closure,
or host launch.

## Original loadout callback inputs

`41a49c45` / `proc_c6a4` established a separate input-side loss.
One callback changed a borrowed declaration;
the next callback actually received that instruction and restored the declaration before final assignment.
The retained-state control passed,
but the restored variant lost the original instruction.
This differs from a direct edit undone before host processing.

`778c4e0d` captures the actual `declared`,
`callable`,
and `registered` argument views before each callback.
It reuses declaration metadata selection and the Session input pool.
All views share one copier budget,
including repeated schema references;
query capabilities remain borrowed and uninvoked.
The source kind and origin are `native-tool-loadout-input`,
location `toolLoadoutInput`,
and bound view `original-tool-loadout-input-view`.
Companion metadata records `prepareLoadout` and phase `input`.
No borrowed argument,
array,
tool,
or query function gains a publication or authority.
Unread declaration fields retain source-specific metadata obligations.

`proc_84cd` passed the retained-state and restored-state native controls.
The owner task in `proc_2d6c` passed original-frame identity,
shared 64-input accounting,
aggregate copying at 4,096 containers,
byte refusal,
getter/proxy refusal,
and unread metadata obligations.
Its canonical fixture initially assumed that codemode leaves the description suffix unchanged;
`proc_2125` then exposed incorrect fixture cleanup ordering.
The fixture now preserves native description composition and disposes/awaits idle before detaching.
`proc_54a3` passed canonical review and input-specific native parity.
The original input appears in exact source bindings,
assessment policies,
and indexed questions while absent from the native request.
The outcome remains unresolved,
with two local request exchanges and one authored assessment.
Getter counts match one/one;
unexpected false and undefined failures stop before callback entry,
and reset does not rebind original publications.
`proc_aca8` passed the shared declaration-metadata regression.
Before the maintained-kernel import change,
`proc_9120` passed all 198 integrated input cases and the original-source variants.
Action/source parents passed `proc_5331` and `proc_faca`.

Argument snapshots do not establish the values later returned by query capabilities,
valid delegation,
or complete governing-domain closure.
The maintained runtime and installed host migration remain unfinished.

## Original loadout callback outputs

`1e60894f` / `proc_eb28` established the original-output loss.
Its native single-output control retained the callback instruction.
A later callback could replace that description before `Agent.state.tools` received the combined result,
leaving the earlier instruction absent from governing sources.

`31d9f721` observes each original `prepareLoadout` output before native aggregation.
The Session input pool owns a distinct observation frame for each received output.
The source kind and origin are `native-tool-loadout-output`,
location `toolLoadoutOutput`,
method `prepareLoadout`,
and phase `output`.
The bound view is `original-tool-loadout-output-view`.
Output objects remain mutable and gain neither publication aliases nor authority.
Undefined output supplies no prose;
explicit empty outputs remain distinct receipts.
The existing count,
serialized-record,
source-byte,
and copier bounds still apply.

`proc_cb31` passed single-output and overwritten-output native cases.
`3e18bdf0` / `proc_1c6c` passed owner/profile controls and the canonical pre-request,
pre-binding,
post-binding,
oversize,
reset,
and disposal paths.
These used zero provider requests and protected executions.
`bf115d3b` / `proc_3720` preserved native getter counts:
descriptions one/one,
hidden declarations one/one,
and unused metadata zero/zero.
Native callback and output-getter falsy errors retain their original extension-error reports.
Unexpected observer failures leave the native catch as their exact original value,
without a replacement report or later tool assignment.
The disposable unwrapping omission `proc_38d4` failed its required exception assertion.
The maintained guard was never removed.

`proc_34a6` passed all 192 integrated input cases with empty child stderr.
`proc_e13b` passed original-source variants and prompt/input owner tasks.
Those results precede the subsequent callback-input changes.
This output-specific repair does not establish callback-input coverage,
complete governing-domain closure,
or maintained host installation.

## Original extension registration inputs

The native extension API can receive a tool definition,
then replace it under the same name before any loadout or request exposes the original text.
`3cee6795` / `proc_f002` reached the required retention failure after verifying the native registration map.
The earlier fixture failures queried the filtered session registry;
[the staging diagnosis](../troubleshooting/pi-sdk-staging.md) records that correction.

The private implementation reuses the existing bounded input store and publication aliases.
The native resource loader owns its registration receipts in a private field.
`registerTool` observation occurs on the actual argument frame before native validation and map updates.
The source kind is `native-resource-tool-input`,
origin `native-extension-register-tool`,
location `extensionTool`,
and method `registerTool`.
Original resource occurrences and publication references remain distinct from Session occurrences,
registration success,
authority,
and payload eligibility.

The staged SDK now copies 13 native modules,
including the resource and extension loaders,
without editing installed sources or historical generated artifacts.
Resource records retain the one-MiB resource bound;
Session records retain their four-MiB bound.
Native counter readers also enforce the shared 64-input/four-MiB limit across both owners.
The combined resource observation remains bounded to one MiB.
None of these counters establishes governing-domain completeness.

`proc_6749` passed owner limits,
inline excluded/allowed definitions,
and actual file-loaded extension replacement.
`proc_ceb6` passed rejected-registration retention and irreversible collector retirement.
Getter counts matched installed behavior:
description zero/zero and parameters three/three.
Native and observer falsy failures stayed exact.
`proc_bcf4` passed canonical retention and entry/binding/oversize/reset/disposal controls,
with zero protected executions or provider requests.
New resource receipts cannot enter the manager-only post-emission handoff.

Action/source parents passed `proc_797c`.
Before aggregate-counter changes,
`proc_8d4b` passed original-source variants and all 181 integrated input cases with empty child stderr.
The counter controls passed `proc_9aa1`;
`proc_9143` accepted 64 combined native receipts and rejected 65 while each store remained below its own cap.
The disposable guard omission `proc_df1c` failed with
`Missing expected exception (InputReceiptBudgetError)`.
The maintained guard was not removed.
`proc_d80a` passed the final aggregate-counter integration:
original-source variants and all 182 integrated input cases,
with empty child stderr and zero provider requests.

This repairs original extension-input loss.
It does not close `request-source-domain`,
qualify model accuracy,
or complete maintained installation and genuine approval verification.

## Original Agent tool-setter receipts

The native `Agent.state.tools` setter receives declaration data even when the caller later clears or restores it.
`c1ff7330` / `proc_f8a4` reproduced the missing receipt after a positive native-state read.
`8bedf07a` observes the actual setter argument frame through the existing Agent observer and manager input pool.
It reuses the metadata copier and publishes no role,
authority,
registration,
or payload capability.
The source keeps kind `native-tool-description-input`,
origin `native-agent-input`,
location `agentInput`,
and method `state.tools`.

`proc_9458` passed native array-copy parity,
borrowed-setter routing,
shared input-count enforcement,
exact falsy observer failure retirement,
and canonical retention after tool-set restoration.
The hidden received text reached original source views and indexed questions without entering the model request.
The lifecycle portion of `proc_1009` passed before-request,
post-emission,
post-binding,
oversize,
reset,
and disposal controls.
Tool receipts do not acquire the manager-only post-emission handoff.

The source-order diagnostic `proc_b4a8` showed distinct constructor-time and prompt-time tool-setter receipts.
Both are retained,
not deduplicated because their data can match.
Updated original-source variants passed `proc_b187`;
message-setter and constructor parity passed `proc_bf23`.
The shared input parent then hit its actual 64-occurrence limit in an accumulated enqueue-failure fixture
(`proc_cc4b`).
Independent failure cases now use fresh fixture roots;
no source limit or native behavior is loosened.
Queue parity passed in `proc_7a73` before its prompt-owner assertion exposed extra native follow-up loadouts.
Those separate received loadouts are now accounted for explicitly.
`proc_bade` passed prompt/run/handler ownership,
original judgment and bootstrap cleanup,
publication controls,
and all 169 integrated input cases.
The input parent reported empty child stderr and zero provider requests.

These receipts close a named input loss,
not complete governing-domain coverage or maintained host installation.

## Constructor declaration metadata

The decision consumer cannot account for an original hidden schema instruction
when constructor capture retains only names and descriptions.
Private `119bbd34` / `proc_c8f0` reproduced that specific missing field.
`35b34cf8` extends the existing original-frame observation,
manager input pool,
and `native-content-json` source view with supported declaration metadata.
No new source registry or request parser is introduced.

Owner and native declared/hidden metadata checks passed `proc_bcaa`.
Each native case made two local request exchanges and one authored semantic assessment,
with zero protected executions or provider requests.
The declared schema instruction appeared in the native request;
the hidden instruction did not,
but both reached the original source view and indexed assessment questions.
These are source-delivery mechanisms,
not model accuracy or governing-domain closure.

The initial probe inspected input schemas only.
The native consumer exposed the read tool's non-enumerable output-schema `~kind` property
through the recorded `SessionInputCaptureError` in `proc_80c7` and the property diagnostic in `proc_fa3d`.
The metadata copier now preserves string-keyed non-enumerable data in an owned JSON view.
It does not invoke getters or freeze borrowed schemas.
Shared metadata is charged for every serialized occurrence under the unchanged aggregate bounds;
message copying still rejects aliases and non-enumerable data.
Symbols,
accessors,
and unsupported declaration slots cannot certify complete capture.
The selector records unread top-level slots as explicit observation gaps;
unsupported nested shapes latch capture failure.

`proc_f50e` caught reversed property order in the first owned-copy implementation.
The copier now reserves owned properties in original order before visiting their values.
The unchanged name/description byte-order assertion,
constructor controls,
and native getter parity passed `proc_7057`.
The integrated input parent then reached an obsolete description-only assertion in
`agent-state-native.test.mjs` (`proc_6961`).
That assertion now checks the retained description and added metadata without dropping callback-order checks.
The corrected input parent passed all 160 integrated cases in `proc_fc94`,
with empty child stderr and zero provider requests.
Its subsequent original-judgment task failed in the test's new metadata assertion:
the fixture uses a read definition with `parameters.properties.path`,
not a codemode definition with `parameters.properties.code`.
That authored assertion is corrected;
`proc_ec25` passed the original-judgment variants,
including discovered,
empty,
and replaced prompt inputs,
with original ownership and bootstrap cleanup controls retained.

The code-owned obligation consumer in `ab498c0` retains unread metadata as
`tool-metadata-observation-gap`.
Required red `proc_527b` failed before the consumer existed;
the action-policy parent passed `proc_2389`.
Canonical `proc_9cc0` passed declared,
hidden,
and unread-accessor metadata cases.
The accessor gap survives admitted `none` estimates for captured clauses and the whole request.
Restoring the declaration cannot repair its original received-input gap.
These are conditional metadata obligations,
not removal of the remaining `request-source-domain` placeholder.

The remaining delivery target is decision-specific domain closure and maintained host/runtime integration.
Constructor metadata alone does not account for earlier configuration processing,
extension registration,
other processing boundaries,
or cross-root applicability.
The original source limits,
manager-only post-emission growth,
and irreversible failure retirement remain in force.

## Original Session constructor tool descriptions

This historical description-only checkpoint completed before the constructor metadata extension.
The completed manager checkpoint predates this runtime change.
Native `prepareLoadout` can replace declared descriptions,
and hidden tools need not appear in a model request.
The original definition text remains a separate potentially governing input.

Declared-description control `7414bed9` / `proc_2d95` passed.
Hidden `proc_fc39` and replaced `proc_acbd` completed native request and review,
then failed the original-description retention assertion.
The first setup attempt `proc_8d89` instead expected an unaugmented declaration;
the native Codemode call/output hint is preserved,
not removed.

Private `67a2a405` observes the actual Session constructor frame after its original manager assignment,
before reading `customTools` or running loadout callbacks.
It selects ordinary `name` and `description` data fields under one aggregate copier budget.
The existing manager source owner retains an input-only publication on that frame,
not on the borrowed config,
list,
or definition objects.
The distinct `native-tool-description-input` kind and `customTools` location add no message role,
ordinal,
authority,
or payload capability.
Absent input and an explicit empty list remain different.
The original occurrence,
content,
and serialized-record bounds are unchanged.

Owner controls and declared/hidden/replaced native consumers passed
`d1e67271` / `proc_1456`.
They require the constructor-specific view,
original source/publication references,
unchanged request bytes,
and indexed assessment questions.
At this historical checkpoint,
schema,
executable,
and other opaque fields were not inspected.
The 2026-10-12 constructor metadata extension preserves supported declaration data;
executable hooks remain capabilities rather than copied instruction prose.
Native constructor parity passed `93c75a07` / `proc_9cc4`.
The getter counters matched the baseline,
including zero construction-time reads of `execute`,
and separate positive reads verified the counters.
Lifecycle and declared/hidden/replaced/raw-mutated description consumers passed `proc_a37a`.
Action/source parents passed `proc_50d1`;
all 154 integrated input cases passed `proc_a226`;
original-judgment variants passed `proc_d5fb`;
the runtime suffix passed 39 complete outputs in `proc_f646`.
Complete native parent `89a504de` / `proc_7042` passed in 860 seconds:
82 complete outputs,
157 integrated input cases,
and 69 section-source cases.
This is the restored implementation with the accepted processing-receipt controls.
Required pre-implementation description-loss reds are retained;
no duplicate omission campaign is opened.
The first parity baseline `proc_0081` stopped at an unresolved relative module import before execution;
its fresh copy now keeps the original staged-module targets.

This covers selected constructor descriptions,
not earlier SDK option mutations,
extension registrations,
schemas and other metadata,
deep mutation,
or applicability across manager-root resets.
The manager-only post-emission growth predicate is unchanged.
Constructor-source additions after emission cannot silently gain that handoff.
No installed activation,
semantic qualification,
or governing-domain closure follows.

## Current manager inputs at judgment start

This private checkpoint passed native,
action-policy,
policy-source,
and sixteen-document verification in its selected scope.
The manager observes original `appendMessage` inputs both before and after Session context attachment.
The current phase uses `native-manager-input`;
the initial phase keeps its distinct origin.
Actual argument frames remain input-only publications,
without fabricated ordinals,
authority,
human attribution,
or payload eligibility.

The original request receipt,
emitted body,
emission snapshot,
and material contributions remain unchanged.
The existing collector owns a separate current judgment snapshot through its local `snapshots` map.
Its one-time handoff requires unchanged resource,
base,
and policy identities,
unchanged copied material and run fields,
and an exact original-reference prefix extended only by unregistered manager inputs.
Binding updates judgment indexing,
assessment wire,
and manager source views together.
Later source changes remain strictly invalidating;
failed or reentrant handoffs cannot revive eligibility.
The original budget and cancellation still apply.
Emission-only material associations are not transferred to newly captured input.
The narrow `system-view.mjs` profile refuses source growth rather than claiming broader coverage.

Required post-attachment red `b650661` / `proc_72ea` lost an appended input hidden by a context edit.
Connected `6f71097` / `proc_dc02` passed that native consumer.
Timing controls `a4a7b43f` / `proc_9b65` passed hidden input,
host-called `message_end`,
post-emission text and system-tool inputs,
and post-binding retirement.
They check the exact original request body and native response persistence:
new inputs reach judgment and wire without appearing in the emission snapshot.
No protected execution or provider requests occurred.

Source review identified omitted system `toolsAdded` and `toolsRemoved` fields.
Owner red `d8175179` / `proc_3b4d` demonstrated the loss;
`1c6985cf` / `proc_88a2` preserved the received declaration and removal records.
The manager selector uses one existing copier budget and `native-content-json`,
without another normalization or request parse.
Descriptions,
schema prose,
and constrained-sampling data remain unregistered source data.
Invalid JSON values and accessor fields latch failure.
Loaded,
Agent prompt,
and setter profiles are unchanged.

Owner and handoff controls passed `proc_4412`,
including the emission-only profile refusal with labelled adapters.
Native lifecycle `12ea892b` / `proc_83f6` passed the handoff clock positive control,
expiry during source comparison before judgment publication,
unchanged-view capture failure,
shared occurrence exhaustion,
and late reset,
disposal,
reload,
and oversized input.
The clock is an explicit fixture;
the Session and judgment paths are native.
The positive control retains the original 5000-millisecond deadline.
Expired handoff work sends no assessment and publishes no judgment.
`7e7edd94` / `proc_2872` also retained a post-emission append rejected by the private clone.
No entry insertion,
authority,
or successful persistence is inferred from capturing that input.

Constructed-input regression `e32496ec` / `proc_a9e3` passed 133 outputs with empty child stderr.
The newly integrated lifecycle and clone cases still need the full native parent.
Indexed-question positive control `fb444692` / `proc_4150` passed.
Omitting observation or current message-source binding lost the original input.
Omitting the handoff or current governing-source binding caused stale-snapshot failure.
Omitting current indexing failed wire source/ordinal coherence.
Omitting copied-material equality admitted changed adapter material.
All omissions are restored in `54575a9a`,
with an empty scoped runtime diff against `7e7edd94`.
The complete action-policy and policy-source suites passed `proc_417a`,
with 29 successful result records.
Full native `proc_a3df` stopped in `default-payload-copy` because no review was returned.
Its original exception was not retained,
so no cause is established.
The isolated native-default/copy pair passed `96a0c52e` / `proc_e195` without a runtime change;
that later pass is not a diagnosis or fix.
The second parent `proc_b1f3` stopped in the distinct `carrier-bound` case,
whose original exception was also not retained.
Both fixtures now retain batch errors and report total judgment-start to reviewed time.
Repeated configurations passed `bef690b2` / `proc_a7bd`:
carrier-bound totals were 4984.015445,
5402.251908,
and 5789.265434 milliseconds;
copy-case totals were 971.362350,
914.014512,
and 1024.266656 milliseconds.
These include work after evidence closure and are not deadline-failure measurements.
Neither failed parent has an established cause or a claimed runtime fix.
Parent `proc_05da` passed 35 complete prefix outputs,
including 142 constructed-input cases and 69 section cases,
then stopped at an older original-judgment source-list expectation.
`adfb8fd9` / `proc_a45e` passed the corrected source-list assertions in all four variants.
The assertions retain exact manager source references,
roles,
content,
and emission-versus-judgment membership.
Prompt and handler fixtures also needed their newly captured persistence inputs counted.
Those corrections passed `proc_5415` and `proc_7fa1` before the next boundary was reached.
Diagnostic `9f8e832f` / `proc_b4f5` measured 69 frozen sources in a shared-session carrier case,
exceeding the unchanged 64-source limit.
Independent carrier cases now use disposable sessions;
the original shared-session rejection is retained as a separate bound control.
Runtime suffix `eaa78d4a` / `proc_5185` passed all 39 result records,
including the retained bound rejection and all six full-policy profiles.
The final one-shot native parent `3ea75e62` / `proc_d937` then passed:
82 complete result records,
142 constructed-input cases,
69 section cases,
and all six full-policy profiles.
The parent had no nonroutine stderr;
child matrices reported empty stderr.
Complete action-policy and policy-source verification passed `proc_417a`.
All deliberate omissions remain restored.
Only documentation and standalone,
non-wired intake tests changed during the final parent.
The earlier unattributed parent failures remain historical failures,
not explained by this later pass.
Earlier complete suites cover the pre-attachment checkpoint,
not these changes.
All original bounds and installation,
domain,
delegation,
and semantic-qualification holds remain.
Tool-definition inputs before native declaration production remain a separate gap.

## Independent in-place message intake

The user subsequently selected receipt at host processing boundaries.
The historical requirement to retain undone direct state edits is superseded,
not a remaining release prerequisite.
Controls `a3387df9` / `proc_c726` passed push/pop and direct-index exclusion
plus an actual Agent queue receipt followed by clearing.
Only the genuine API receipt is retained and indexed.
This changes no authority assignment or completeness claim.
State actually processed by the host still requires accounting.
The following intake records explain the superseded assumption.

While manager-source verification runs,
the next source gap has an independent native red.
Pi agent-core 1.1.0 returns its live message array from the state getter;
observing the setter does not observe element writes.
`95ad85c6` / `proc_4292` inserted and removed original text with `push` and `pop`.
`39d61ef4` / `proc_9e5b` repeated the loss with direct index insertion and deletion.
Both restored the original array before prompting,
completed native request/review,
then failed the required source-retention assertion.
Neither performed protected execution or provider requests.

The standalone tests use the existing staged SDK without regenerating it.
They remain intentionally outside the manager increment's passing suite.
No proxy,
array wrapper,
constructor change,
or replacement-state implementation is adopted.
A later snapshot cannot recover the removed input from the restored array.
Native event processing itself uses array writes,
so a mutator name or callsite is not authorship or authority evidence.

## Original manager messages before context attachment

The completed initial checkpoint was limited to `appendMessage` calls before the manager's first
original Session context-copy observer attaches.
It is not coverage of all current manager appends.

Required red `4c74fc7` / `proc_5d44` appended original user text to an owned in-memory manager,
hid it with a null context edit,
then constructed the real SDK session.
The native request and assessment completed,
but canonical review lacked the original append input.
Capturing another constructor snapshot would only see the edited context.

The pre-body hook retains the actual `arguments` frame through the existing manager source owner.
Its origin is `native-manager-initial-input`,
with metadata-only `managerInput` location,
no invented entry ordinal,
and no message or payload alias.
All existing source and copier bounds remain shared.
The selected content profile keeps admitted text/thinking/tool-call fields,
including opaque signatures and argument data,
without assigning meaning,
authority,
or execution facts.
Prompt/setter text-only profiles are unchanged.
Image blocks,
unknown fields,
and unsupported roles still latch guarded failure.

The unrestricted first implementation `bf5aff8` failed before review.
Diagnostic `0f05a19` / `proc_323e` showed one request,
zero assessments,
and no content-profile failure.
Native response persistence changed retained input membership,
which republished the run view and correctly invalidated the original request snapshot.
Policy source binding and rule indexing also read that original snapshot,
so merely tolerating new retained inputs would not make them enter policy review.

The narrowed hook uses the existing one-way `contextCopyAttached` lifetime state,
not a temporary mute window or a claim that native persistence is already accounted for.
`740ffae` / `proc_22d7` passed the pre-attachment canonical consumer with two local requests,
one assessment,
and no protected execution/provider calls.
That checkpoint did not account for post-attachment direct or native appends.
No request freshness or source-binding rule was relaxed.

Canonical `ffa4e19` / `proc_f1e3` passed both hidden-input and private clone-rejection cases.
A rejected append inserts no entry,
but its original selected text still reaches review.
Owner `9c9885b` / `proc_0d67` passed block/companion selection,
opaque-getter exclusion,
malformed inputs,
shared bounds,
original errors,
and JSON-value preservation.
The negative-zero red required rejection of an argument that serialization would change to zero.

Native `3e4dfd2` / `proc_7543` passed actual unmapped frames,
private owned-copy preservation,
clone reentry,
getter/proxy error parity,
unexpected falsy observer errors,
and root-reset/one-way attachment controls.
Its baseline is a disposable private generated manager with only the new observation hook removed,
not the installed manager's different aliasing behavior.
Post-attachment coverage and qualified continuation are not claimed.

Integrated `b62201f` / `proc_4882` passed 128 constructed-input outputs with empty child stderr,
including the hidden assistant block consumer.
Observation omission `proc_d58c` lost the original source.
Post-clone placement `proc_1c79` preserved successful appends but lost rejected input.
Attachment-phase omission `proc_2dbc` reproduced the diagnosed pre-review snapshot failure.
Content-profile omission `proc_a769` admitted an unsupported block.
Every omission is restored;
the restoration's runtime diff against `9c9885b` was empty.
Complete one-shot native parent `proc_9e75` passed in 624 seconds:
81 successful JSON outputs,
the 128-output constructed-input matrix,
all six full-policy profiles,
and empty reported child stderr.
Complete action-policy and policy-source suites passed `proc_3bea`.
Only documentation changed during these regressions.
This verifies the bounded pre-attachment checkpoint,
not later manager-input accounting.
Outcome document checkpoint `proc_a64d` passed all 14 documents with zero diagnostics.
The incumbent private manager copy performs `structuredClone`;
the installed native manager does not.
Parity claims must distinguish those baselines.
The completed setter checkpoint predates this new runtime change.

## Original Agent state assignments

Direct `agent.state.messages` assignments use a native setter,
not a `replaceMessages` method in Pi 1.1.0.
Required red `cd5fd32` / `proc_896e` assigned original text,
verified the native shallow copy,
cleared it,
and then lost the input in canonical review.

Private `7d42311` / `proc_1f5b` now retains the actual setter arguments frame before native `slice`.
It reuses the existing Agent observer,
manager input owner,
array-content profile,
and shared limits.
The borrowed array and message references are unchanged;
the frame's input-only publication grants no payload capability or authority.

Native reset and Session projection assignments use an instance-local internal function.
It supplies a private invocation token only to the exact original state setter,
after checking target and descriptor identities/flags.
A changed descriptor takes ordinary native assignment,
without receiving the token.
There is no dynamic mute flag:
reentrant ordinary writes are still captured.
This identifies native derivation,
not authorship or complete source coverage.

The existing one-time Agent attachment returns that function.
SDK construction passes it as a separate validated constructor argument before user callbacks can run.
No new borrowed constructor-option getter or global registry is added.
The early installation matters:
`prepareLoadout` can call `pi.sendMessage` during Session construction.

Native `proc_67c2` passed frame/copy,
reentry,
forged-token,
descriptor,
foreign-target,
proxy,
and error controls.
`proc_91c7` exercised real constructor reentry without reminting the native projection.
`proc_7faa` passed erased/late/oversize/reset/disposal paths,
including original cancellation cause and zero protected execution/provider requests.

Integrated `proc_a3d0` passed 123 constructed-input outputs with empty child stderr.
The observation,
token identity,
setter identity,
constructor routing,
and constructor-validation omissions each failed their intended control.
All were restored in `8cb4a62`.
The runtime diff against `7d42311` is empty;
the exhaustive changed-path inventory contains only tests,
task configuration,
and documentation.

The legacy compaction fixture directly assigned a manager projection,
which now correctly failed the public setter's selected text-message profile.
Diagnostic `proc_8730` exposed the unsupported-role cause.
The fixture now uses actual synchronous `Session.refreshContext()`,
not a public-write exemption.
Reconciled `proc_eff0` passed all 43 expanded/retained modes;
the intentional stale-path clear still has its own captured occurrence.

Extended `69deaca` / `proc_be71` passed shared loaded/queued/setter occupancy,
serialized-record overflow,
and direct setter calls inside real constructor callbacks.
It also exercised real direct Sessions sharing an attached Agent.
Without the original private constructor argument,
the second Session's writes take the ordinary observed route.
They stay with the Agent's fixed input owner and do not establish caller identity,
transfer scope,
or acquire authority.
Shared-Agent multi-Session execution remains outside the qualified profile.

The setter requires an ordinary dense array using the existing selected message fields.
Non-text content and unsupported message roles latch capture failure without replacing native assignment.
The selected array,
including JSON wrappers and escaping,
shares the 65,536-byte source bound.
All manager inputs still share 64 source occurrences and the four-MiB serialized-record bound.
Equal arrays and empty clears remain separate occurrences.

Complete native parent `proc_428d` passed in 719 seconds:
81 successful JSON outputs,
the 123-output constructed-input matrix,
all six full-policy profiles,
and the remaining native program cases.
This is a passing one-shot parent invocation,
not segmented coverage.
All reported child matrices had empty stderr.
Complete action-policy and policy-source suites passed `proc_b3aa`.
Only documentation changed during these regressions.
The private setter checkpoint is verified.
Outcome document checkpoint `proc_99f8` passed all 14 documents with zero diagnostics.
Constructor inputs,
in-place changes to getter-returned arrays,
state-object or descriptor replacement,
and broader model/tool/image instruction sources remain unfinished.
No domain closure,
human origin,
or new policy admission is inferred.

## Original direct Agent message and array prompts

The next private increment extends the existing Agent call-frame owner beyond primitive strings.
Single-message and message-array consumers first failed the required original-source assertion
in `proc_4517` and `proc_1356`.
Initial implementation `08389a1` / `proc_4ff8` passed both actual direct request consumers,
with no Session input events,
protected executions,
or provider requests.

Single-message inputs reuse selected content and companion metadata.
An array retains selected message fields as one original call source in `native-content-json`.
Its original arguments frame carries the input-only publication.
Neither the array nor its messages acquire aliases or payload capability.
Opaque provider/tool records are not copied.
The array uses one shared incumbent copier budget,
and wrapper fields plus escaped text count toward the original source-byte bound.

Native `AgentSession.prompt` also forwards an array.
The initial queue-role subset missed a system-section delta prepended by the Session.
`proc_bc9b` exposed its capture failure before any request or assessment.
The correction reuses the existing loaded-system content/sections profile,
including null section deletion,
without inspecting tool declarations or assigning system authority.
Queue role support is unchanged.

Owner controls `proc_fe45` and native normalization controls `proc_a349` passed
before the system-section correction.
The normalization delegate remains a labelled double,
separate from actual request consumers.
Corrected `e1fbda6` / `proc_e97e` passed owner system fields and real late,
oversize,
unchanged-view,
and reset controls.
Direct system-message and system-array canonical wire consumers passed `proc_f007`.
System `inputFields` metadata and original section content remain unregistered.
Integrated `proc_a44a` passed 116 outputs with empty child stderr.
Non-string observation omission `proc_0e00` and system-section omission `proc_883a`
lost the required original source.
Proxy-guard omission `proc_7d1b` invoked a caller trap and propagated its `false` value.
All guards were restored;
the implementation has an empty scoped diff against `e1fbda6`.

Every native case is verified across phases,
not a single successful parent invocation:
`proc_5868` completed the source prefix before its old full-policy fixture failed;
`proc_45e7` passed the complete corrected runtime suffix.
Diff inspection from `06ed9d3` shows only the full-policy fixture,
its added standalone task,
and progress notes changed between those runs.
Full-policy `proc_b0c7` separately passed all six profiles.
Complete action-policy `proc_4f7e` and policy-source `proc_0091` passed.
The final document check is tracked in `current-progress.json`.

System-role support does not close tool instruction-source ownership.
Mixed/image content and unsupported shapes still fail guarded source use,
including internal Session array forwarding.
Every internal forwarding adds one original call occurrence and can exhaust unchanged limits sooner.
Boxed strings and null inputs now latch unsupported-profile failure instead of remaining unobserved;
normalization parity is checked separately.
The native prompt body,
borrowed inputs,
source limits,
and installed plugin remain unchanged.
The evidence for this message/array increment is distinct from the primitive-string checkpoint.
Other direct Agent state producers,
tool instruction-source ownership,
image governance,
applicable scope/delegation,
and complete domain accounting remain unfinished.

## Original direct Agent string prompts

String-form `Agent.prompt(input, images)` is now observed before its active-run check and normalization.
Required red `8e56006` / `proc_b58b` completed the direct request without session input events,
then found its original source absent from canonical review.

The existing private Agent observer and manager owner retain the actual call frame's string input.
The source remains `native-agent-input` at `agentInput`,
with method `prompt`,
no role-derived authority,
and no fabricated user-message publication.
The observer does not read the image argument.
Message/array prompt forms are deliberately untouched and remain a separate gap.

Canonical `b192dc9` / `proc_960b` passed direct source/reference/wire assertions
and real active-run rejection retiring the earlier review.
`proc_f1aa` passed native entry-frame,
empty/adversarial/UTF-8 bound,
strict/unmapped,
image-reference,
non-string branch,
oversize,
and error controls.
Its normalization delegate is a labelled double;
the canonical consumer separately exercises real request and busy-run behavior.
No image governance or full producer coverage is inferred.

The integrated string-prompt matrix passed `proc_0117` with 103 outputs and empty child stderr.
It includes oversize input before transport,
oversize busy rejection with an unchanged constructed view,
and reset retiring the earlier review.
Predicate omission `8e52673` / `proc_b636` poisoned non-string capture as expected;
observation omission `0734902` / `proc_58f8` lost the original source.
Both are restored.

Full regression `proc_931a` stopped in preparation on retained import path spelling
following a filesystem move.
Build-only correction `b60415d` / `proc_0064` passed same-file alias and wrong-target controls,
then actual SDK preparation.
Target-check omission `e38b148` / `proc_cfc3` failed the equal-byte wrong-file control;
the check is restored.
Complete native `proc_6645` passed after restoration and the build-path correction,
including the current 103-output input matrix with empty child stderr.
Complete action-policy `proc_379b` and policy-source `proc_580e` passed.
The separate earlier full-policy failure remains unattributed;
these later passes establish no cause or fix.
Fourteen-document lint and rendering passed `proc_4b01` with zero diagnostics
using the current repository linter and per-path clean/error controls.
The document runner was adapted after the old Markdown CLI was removed;
that verification-only change does not replace the qualified runtime parser.
All source limits,
budgets,
and installed files remain unchanged.

## Original direct Agent queue-call inputs

Direct `Agent.steer(message)` and `followUp(message)` bypass `AgentSession` text entry.
Required red `c6bc256` / `proc_d56b` queued the exact supplied message,
cleared it through native `clearAllQueues`,
then found zero original direct-Agent sources during later review.

The existing SDK generator now attaches a fixed observer immediately after constructing each private Agent.
It closes over that SDK construction's original manager.
The Agent stores the callback in a once-assignable private field.
No borrowed constructor option,
new registry,
or source-domain flag is used.
Each actual queue-call arguments frame gets an input-only source,
so an already-published message keeps its prior alias.
Observation precedes enqueue and proves neither enqueue success nor material contribution.

`native-agent-input` uses `agentInput`,
with method and selected message metadata.
Supported content uses the incumbent bounded copier and text-content profile for user,
custom,
assistant,
tool-result,
and legacy hook-message roles.
System and non-text/mixed representations remain unsupported.
Expected capture failures latch while native queue behavior continues.
Unexpected observer failures latch and rethrow their original value.

`f58fd92` / `proc_afcc` passed the cleared-input consumer.
`proc_0d29` passed native queue/reference parity,
one-time attachment,
unchanged constructor-option reads,
two-SDK-manager isolation,
publication preservation,
and native/capture error boundaries.
The owner phase of `proc_c501` passed content and shared-bound controls.
`proc_7763` passed delivery,
clearing,
late changes,
overflow,
reset,
and disposal cancellation.

This observation also sees internal `AgentSession` forwarding.
Mixed image messages can therefore latch the text-only source-profile failure even when their earlier text input
was captured.
Native messages/images remain unfrozen and queue behavior remains unchanged;
this does not complete image governance.
All caps remain unchanged.

Queue fixture `proc_77d0` failed because a preceding private-only injected enqueue failure left native steering
bookkeeping on only one side.
An erroneous progress sentence marking that run as passed was retracted.
`proc_54e8` now tests the preceding failure on both SDKs,
verifies and clears identical bookkeeping,
and passes the supported-text and explicit mixed-image cases.
No runtime guard was weakened.

Integrated `proc_b210` passed 97 constructed-input outputs with empty child stderr,
including actual settled-event queueing before clear.
Rethrow omission `92fe738` / `proc_7cf0` failed with `Missing expected exception.`
SDK attachment omission `b3c1431` / `proc_7234` and queue-observation omission `d34d75c` / `proc_cd4a`
each lost the original cleared input.
All guards and observations are restored.
Complete native `proc_c273` passed in 428 seconds.
Complete action-policy `proc_43c5` and policy-source `proc_89b0` also passed.
These local checks establish the bounded mechanism,
not empirical qualification or a production admission policy.
Direct Agent prompt/state producers and governing-domain closure remain unfinished.

## Original public API input before native branching

Public `AgentSession.prompt`,
`steer`,
and `followUp` can reject or defer before the already-observed input constructions.
Required red `c05cf9d` / `proc_30b3` rejected a known command through the real queue API,
with no command handler,
input event,
or transport,
then found its original text absent from later canonical review.

The maintained generator now observes each actual method-entry `arguments` object synchronously.
The existing manager owner selects only its own text data descriptor at index zero.
`native-call-input` uses `callInput` with fixed method metadata.
No option/image fields are read or serialized.
No call inventory,
synthetic event,
run,
entry ordinal,
authority,
or human origin is invented.
The opaque frame remains mutable and input-only.

Engine/owner probe `proc_dd75` was not SDK coverage.
Actual native `proc_2d41` subsequently verified strict/unmapped entry frames for all supported methods,
original publication identity,
native argument behavior after frame-only mutation,
and installed/private option-getter,
rejection,
and oversize parity.
These are `AgentSession` text APIs;
direct `Agent` message APIs remain separate.

Canonical `3dbf0f6` / `proc_0ff4` passed rejected steering,
rejected follow-up,
and original prompt-option getter failure.
Owner/queue controls `proc_48a4` passed shared bounds and explicit source ordering.
`proc_118a` had exposed an obsolete fixture index:
the new call input precedes the native input event.
The fixture now verifies both rather than dropping either source.

Successful calls also add original invocation occurrences.
Existing constructor/event occurrences are not merged by equal bytes.
The same source-count,
content,
serialized-record,
and snapshot caps apply,
so supported occupancy can be exhausted earlier.
No bound or judgment deadline was enlarged.

Native settled deferral/replay and reset/disposal checks passed `proc_1842`.
The deferred entry reads no options;
replay has its own physical invocation,
and prior root eligibility is not restored.
Late rejected-call and unchanged-view failure controls passed `proc_d2ba`.
Original-judgment profiles passed within `proc_0abb`;
subsequent prompt/handler fixtures now verify API-call and return sources separately.

Runtime `proc_5939` passed those custody checks,
then the full-policy fixture failed to reach review during independent-copy preparation.
Diagnostic-only retry `proc_9a1e` passed with unchanged runtime and the same policy input.
That non-reproduction proves neither cause nor fix.
The five-second budget remains unchanged,
and nested failure diagnostics are now preserved.
Integrated `proc_4a74` passed 85 constructed-input outputs with empty child stderr.
Rethrow omission `0d7076e` / `proc_82cb` failed with `Missing expected rejection.`
Entry-observation omission `5a3020f` / `proc_c64f` lost the original rejected-call source.
Both guards are restored;
no deliberate omission remains.
Complete native `proc_834c` passed in 428 seconds,
including the full-policy fixture.
Complete action-policy `proc_fcb7` and policy-source `proc_e74d` also passed.
The prior full-policy incident remains unattributed;
these later passes are not evidence of a runtime fix.
All guards and original bounds remain intact.
No production admission or governing-domain closure is claimed.

## Original known-command input before interception

Known extension commands can return before input-event observation.
Required red `1e2add0` / `proc_d845` ran an inert command with no input event or initial transport;
later canonical review lacked its original text.

The existing manager owner now records the original full text and native parsed command name
beside the actual freshly constructed command context,
before attempting the handler call.
`native-command-input` uses location `commandInput`.
The context is only an opaque input-only identity witness:
its getters,
methods,
and other fields are neither read nor serialized.
The handler still receives native arguments,
not a reconstructed command.
There is no invented event,
run,
ordinal,
authority,
or successful-execution fact.

`3356afe` / `proc_e670` passed the canonical source/publication and assessment-wire checks.
Corrected owner controls `proc_24cc` preserve the existing `toJSON` exclusion,
zero getter/proxy reads,
distinct equal occurrences,
first-failure latching,
and shared bounds.
The initial positive getter fixture in `proc_d918` incorrectly included `toJSON`;
the guard was preserved and the fixture now tests that hook as rejected.

Installed/private parity `proc_cfac` preserved native parsing,
context construction,
handler receiver,
lazy getter behavior,
unknown-command absence,
and native handler/reporter/context-factory failures.
Expected capture-profile failure does not replace native handler behavior.
Unexpected observer failure retains the original thrown value,
latches guarded source failure,
and is not reported as a command-handler failure.

`proc_797b` passed pre-run command-triggered delivery,
overlapping equal commands,
reentrancy,
late non-transmitted input,
oversized late capture,
and reset before consumption.
Integrated `proc_375b` passed 71 constructed-input outputs with empty child stderr,
including reset/disposal during awaited command handlers.
Rethrow omission `b30b5b0` / `proc_1854` failed with `Missing expected rejection.`
Observation omission `9bebebc` / `proc_e106` lost the original command source after real handler entry.
Both guards are restored;
no deliberate omission remains.
Complete native `proc_1736` passed in 411 seconds.
Complete action-policy `proc_4dfe` and policy-source `proc_666d` also passed.
These local checks do not qualify semantic estimates or admit production policy.
Deferred or rejected calls before this observed handoff,
direct-agent inputs,
image governance,
delegation,
and domain closure remain unfinished.

## Final transformed input before expansion

The final input handler can create text that no later input event sees.
A native template can then discard that text.
Required red `91b4edb` / `proc_2bf2` isolated a restriction-like argument created only by the final handler;
canonical review lacked its original source despite successful native termination.

The maintained generator now observes the actual aggregate transform-return construction.
The existing manager owner retains `native-transformed-input` at `transformedInput`,
with text and source/streaming metadata.
It returns the identical native object,
does not inspect or freeze borrowed images,
and invents no handler event,
run,
entry ordinal,
or authority.
This is not the handler's borrowed result object.
Native aggregate continue and handled outcomes create no transformed-return publication.
Equal text with changed images can still select the native transform branch.

`80fe2f5` / `proc_03ab` passed the canonical source/publication and assessment-wire assertions.
`proc_83f6` passed installed/private return parity,
original failure latching,
shared source and serialized-record bounds,
and reset/disposal during awaited handlers.
The original root lease remains ineligible after reset;
disposal still rejects guarded source access.
All source bounds remain unchanged.
These checks do not establish image governance or payload eligibility.

Integrated `proc_1851` passed 60 constructed-input outputs with empty child stderr,
including queued/cleared,
chained,
equal-byte,
oversized,
and rejected-streaming transformed inputs.
Omitting only this native observation failed `c0e74fc` / `proc_166f` at the original-source assertion.
The observation is restored in `6e373a9`;
no deliberate omission remains.
Standalone source profiles with explicit preparation and the complete native parent suite passed in `proc_3e8e`
(426 seconds for the combined command).
Complete action-policy `proc_db21` and policy-source `proc_8595` also passed.
These remain local mechanism checks,
not empirical semantic qualification or a production admission policy.
The earlier unintercepted-input closeout remains a separate verified checkpoint,
not verification of this new construction edge.

## Original unintercepted input before expansion

The private SDK now observes the actual `{text,images}` return object from the
no-input-handler branch before skill/template expansion.
It returns that same object unchanged and records only original text and source/streaming metadata
through the existing manager owner.
The source is `native-unintercepted-input`,
not a fabricated input event,
run,
entry position,
human issuer,
or payload capability.

Required red `c2ca326` / `proc_785c` used a native template without argument placeholders.
The expanded request omitted the raw invocation and its arguments.
`1509002` / `proc_bd69` preserved that original input;
the installed template parser and expander remain unchanged.

`proc_af25` matched installed/private return values,
borrowed image references,
empty and UTF-8 boundary inputs,
reset,
and oversized capture preserving the native return.
`proc_a904` covered steering,
follow-up,
cleared queues,
a raw input rejected before transport,
and a rejected streaming prompt that still retires earlier source evidence without queuing anything.
Expanded strings and equal raw strings remain distinct source occurrences;
the raw input never borrows the expanded prompt's material association.

The ordinary source-content cap remains 65536 bytes,
with the same combined 64-source/four-MiB record limits.
Raw text can now fail the source profile even if later expansion shortens it.
The first capture failure remains latched across a later valid input.
A fresh native input owner can clear that capture failure,
but does not revive the old root lease.
Wrong-manager observation rejects without publishing a source.

`proc_801f` exposed a loaded-source fixture that counted every manager-owned source as loaded.
Selecting `native-load` explicitly restored the original load assertions in `proc_e82c`.
Expanded-prompt regression `proc_3db1` also passed.

Metadata validation already rejected nested getters before cloning.
Proxy traps were a distinct gap:
`bfb344f` / `input-metadata-proxy-read-required-red` observed two trap calls before rejection.
The incumbent `copyMessageData` now checks Node's `types.isProxy` before reflection.
`1583671` / `proc_5258` passed the zero-side-effect controls.
No additional copier or registry was introduced.

Integrated `proc_753b` passed with empty child stderr,
including placeholders,
equal raw/expanded values,
persistent failure,
foreign-manager rejection,
and reset before consumption.
Observation-edge omission `proc_8292` lost the original raw source and failed;
the edge is restored.
The first complete run `proc_5485` stopped at an obsolete built-in source offset in
`original-judgment.test.mjs`.
Its wire correctly included the original raw prompt before generated fragments.
Narrow red `proc_28b6` reproduced the assertion;
`cf238c5` now verifies the raw source and its original publication separately.
Restored `proc_5f2e` and all source profiles in `proc_1be7` passed.
No runtime or generator code changed for this fixture correction.
Complete retry `proc_efa9` passed the source profiles but exposed another stale assumption:
settled prompt/handler snapshots expected raw inputs to disappear with the run.
`proc_18dd` also exposed stream assertions converted into native terminal error messages.
The fixtures now require successful native termination and compare original run/raw source references separately.
The existing task sequence is split at `test:runtime`,
without dropping cases,
so fixture retries do not repeat the source phase.
Runtime stage `proc_1cea` passed in 39 seconds.
Complete staged parent run `proc_075f` passed in 431 seconds.
Complete action-policy `proc_843e` and policy-source `proc_87c4` also passed.
All deliberate omissions are restored;
these are local mechanism checks,
not semantic qualification or production policy admission.

This does not cover commands or deferred/rejected calls before the observed branch,
last-handler output before further expansion,
direct-agent producers,
image origin,
or complete governing scope.
Skill expansion is not separately exercised here.
`request-source-domain`,
semantic qualification,
and independent approval sufficiency remain unresolved.
No installed change or provider trial occurred.

## Original native input-event text

The private emitter now observes each actual native `input` event before its handler receives it.
The existing manager owner retains original text and source/streaming metadata as
`native-input-event`,
using the same constructed-input reference view and source limits.
It does not read image data,
freeze the borrowed event,
infer human origin from `source:'interactive'`,
or grant payload eligibility.

The maintained generator copies the current SDK's input emitter,
including its handler snapshot,
ordering,
transform chain,
handled short-circuit,
and native error reporting.
Equal text passed to distinct handler events keeps distinct original occurrences.
Handled or transformed text remains represented even when absent from the model request.

Required red `b4c72b5` / `proc_dae2` lacked the original intercepted input.
`09cb90e` / `proc_7ee7` passed handled and transformed consumers.
`proc_64d3` passed chained transforms,
equal events,
handled-after-transform,
and late handled/oversized input.
A handled input can invalidate earlier source evidence without ever becoming a queued message.

`proc_5822` exposed an obsolete fixture count:
an intercepted queue call now has its actual input-event occurrence as well as queue construction.
The corrected parity assertions in `proc_7530` distinguish both original publications.
Pure source controls `proc_6bb9` cover UTF-8 bounds,
metadata,
borrowed-event mutation,
getter rejection,
image-read exclusion,
and shared load/append/queue/custom/event limits.

Independent review identified a private diagnostic error:
observation was inside the native handler catch.
Structural red `a0e0071` / `proc_c956` showed an observer's falsy throw was swallowed and attributed to the extension.
`9cde800` moves construction/observation before the handler try.
Native handler failures keep their own catch and reporting;
unexpected observation failures propagate as themselves.
Ordinary source-profile failures still latch for guarded use.
The initial consumers passed `proc_be20` after the change.

Integrated `proc_a059` passed with empty child stderr,
including the absent-handler negative and separate observer/handler error paths.
Omitting the actual observation edge failed `proc_e954`;
the edge is restored.
Complete native `proc_a1d5` passed in 365 seconds;
action `proc_d1a1`,
source-policy `proc_62bd`,
and 14-document rendering `proc_e9ec` also passed.
No-handler fast paths and commands handled before the input emitter construct no event under this profile.
Their absence is not proof of non-governing input.
Images,
direct-agent producers,
authenticated callers,
applicable governing scope,
and domain closure remain unfinished.
No provider trial or installed change is authorized.

## Original current custom-message construction inputs

The private native `sendCustomMessage` constructor now captures its normalized
`appMessage` through the existing manager input owner.
Text stays text;
text-block arrays stay structured source data.
`customType` and `display` remain metadata,
not authority.
Opaque `details` are not read into instruction metadata.
The source uses `native-custom-construction`,
a fresh manager input occurrence,
and no fabricated run or entry ordinal.

The shared reference view is now named `readConstructedInputs`.
Its native reader is `readNativeConstructedInputOrigins`.
These replaced the private `readQueuedInputs` and `readNativeQueuedInputOrigins` names on 2026-10-09.
Existing queued-user source metadata is unchanged.
This remains one view of original source/publication records,
not a second message inventory.

Actual custom-entry construction now propagates input-only identity,
including the immediate `_appendCustomMessage` path.
Projection and conversion retain that identity without payload eligibility.
Native normalization,
queue behavior,
and dispatch errors remain native.

Required red `0abb416` / `proc_2d96` lacked the original constructed source.
`74e6e10` / `proc_fb7c` then exposed the missing input-only entry edge.
`d7c857e` / `proc_07f8` passed immediate,
next-turn,
and hidden inputs,
but exposed first-turn `triggerTurn` without a native run composition.

The incumbent policy-input collector now retains original constructed inputs directly when no native run exists.
Its fixed native session reader checks liveness and capture failure.
The snapshot carries `nativeConstructedInputs`;
the source view invents neither `runOccurrence` nor `currentRunOccurrence`.
Changed source references or capture failure retire earlier request evidence.
Once a native run exists,
the original run-retention path remains the carrier.

A copied resource-owner attempt failed `proc_15b4` at
`Native base composition lacks its exact owned resource binding`.
That duplicate-owner route,
generator,
and unused generated artifact were removed.
The correction in `6fe0552` extends the incumbent collector and preserves its original binding map.

`proc_f9b4` passed immediate,
next-turn,
hidden,
initial trigger,
and streaming steering/follow-up/deferred construction.
`proc_d5cb` rejected late pre-run changes,
oversize capture,
and reset.
`proc_fb31` passed installed/private normalization parity and retained-input controls.
`proc_bbe9` passed the integrated user/custom matrix with empty child stderr,
including actual continuation delivery and original entry aliases.

The pre-run snapshot keeps the existing one-MiB base-linked bound,
separate from the shared 64-source/four-MiB owner limits.
The allowed control retained eight pending 63000-byte inputs;
18 such inputs failed before transport.
Unsupported custom image content and oversized immediate input also failed before transport.
The disposal fixture initially expected the wrong error:
`proc_685d` showed original `JudgmentCancelledError` with its `AbortError` cause.
The corrected fixture preserves that cause identity and separately rejects reads after disposal.

Entry omission `proc_2dde` and pre-run freshness omission `proc_c28d` failed as required.
The byte-bound omission first survived `proc_ef01`:
a later check emitted the same error after one original transport.
The strengthened before-transport assertion failed `proc_0fdc` with `1 !== 0`.
All guards are restored.
Full native `proc_d3b5` passed in 359 seconds;
action `proc_f8cd`,
source-policy `proc_21a7`,
and 14-document render `proc_3378` also passed.
Runtime and generator source is unchanged after that full-native checkpoint.
Direct agent producers,
pre-handler inputs,
mixed-content origin,
and complete governing-domain coverage remain unfinished.
Native settled-callback control `proc_3e24` also passed:
it observed actual `agent_settled`,
queued one deferred custom action while not streaming,
and verified later delivery and original entry identity.
This test-only addition is in the integrated matrix.
No installed activation,
semantic qualification,
policy grant,
or provider request is implied.

## Original native queued user text

The private consumer captures the actual user-message construction in
`AgentSession._queueSteer` and `AgentSession._queueFollowUp`,
after native input handlers,
skill expansion,
and prompt-template expansion.
The existing manager input owner retains the original text and input-only publication.
It assigns `inputOrigin:'native-queue'`,
a fresh input occurrence,
and delivery metadata,
not an emitting run,
a saved issuer,
or a session-entry ordinal.

A frozen view holds references to those original source/publication records.
It is not another transcript,
message inventory,
parser,
or serialized request.
Loaded,
appended,
and queued sources share the existing 64-source and four-MiB serialized-record limits.
The source remains unregistered and cannot authenticate a payload contribution.
Queue clearing and delayed transmission do not independently prove governing-policy revocation.

Context retention incorporates these known inputs independently of transmission.
A source read refreshes nonempty changed queue views and rechecks deferred capture failure.
Empty queues do not manufacture a native context observation.
The queued-view marker is committed only after successful retention or unchanged-view reuse.
Late input changes invalidate the original request source view rather than borrowing its earlier assessment.

Required red `7047d65` / `proc_10e8` lacked the original queued occurrence.
The first implementation failed `proc_1693` because a late generator insertion used
`_sessionManager` instead of the already-private `#sessionManager`.
Corrected native delivered/cleared cases passed `proc_43ef`.
Late-arrival controls `proc_f702` and `proc_14cd` closed the original judgment with
`close-failed` and all members unentered.
They cover explicit queue calls,
streaming prompt/extension routes,
oversized capture with unchanged queue-view identity,
and native reset before merge.

Empty-view red `ddea9bd` / `proc_3fab` was corrected in `242d62c`.
`proc_cd98` passed queue-owner controls only:
mise treated the other task name as an argument.
Separate `proc_a0bc` passed the 23 selected-path,
append,
and queue-owner controls.
`proc_60ef` exposed a fixture assertion against an ordinary entry wrapper;
the actual copy observer associates `entry.message`.
Corrected native aliases passed `proc_83d8`.

This is construction-field custody,
not complete original-input coverage.
Direct `Agent.steer` / `Agent.followUp` calls,
custom queue messages,
pre-handler inputs,
and independent image/mixed-content origins remain unestablished.
Native delivery metadata is not governing applicability.
`request-source-domain`,
authority,
delegation,
semantic qualification,
and independent approval sufficiency remain separate unresolved premises.
Installed/private queue API parity passed `proc_6162`,
including images without origin claims,
transformed/handled input,
and unchanged falsy native enqueue errors.
Omissions `proc_d12b`,
`proc_2f0b`,
and `proc_88a1` independently lost pending-input membership,
late-input source retirement,
and unchanged-view capture-failure rechecking.
All were restored;
integrated controls passed `proc_2ae8` with empty child stderr.
Complete native `proc_33e4` passed in 346 seconds;
complete action `proc_593c`,
source-policy `proc_188a`,
and the 14-document render `proc_b538` also passed.
Nothing is installed or activated.

## Original current-session summary inputs and append membership

Current compaction and branch-summary arguments now use the incumbent manager input owner.
Each is a fresh `native-append` input.
The current derived system checkpoint is not recaptured as a new independent instruction source;
reopening its saved file creates a separate `native-load` input.
This does not prove complete composition or governing scope.

Capturing the argument alone was insufficient when a context handler appended it
without returning a rebuilt projection.
Required red `5f576bb` / `proc_6f5a` exposed that untransmitted-input gap.
The actual native leaf assignment now extends the existing root-scoped known-input reference set.
No additional tree walk,
message inventory,
authority registration,
or budget was added.

The append observation happens after native in-memory index/leaf publication and before persistence.
Source-set overflow is deferred to source use and latched for that root,
so an observation-bound failure does not replace a native append error.
A new root can start a new observation set;
that does not revive the retired runtime.

Summary required red `e539d3c` / `proc_02e3`
became native green `901ee9d` / `proc_3775`.
Untransmitted edit/summary consumers passed `466cb3d` / `proc_7838`.
Summary/derived-checkpoint,
branch/reset,
and 43-mode native controls passed `proc_eecb`.
Root,
duplicate,
foreign,
and deferred-bound controls plus loaded-input regression passed `proc_1bea`.
Combined capture-overflow and native write-error controls passed `proc_f207`.

Leaf-observation omission `proc_19e4` and failure-latch omission `proc_40b7` each failed their control.
Both guards are restored.
Current summary manager controls are included in routine verification;
complete native `proc_7a67`,
action `proc_a32c`,
source-policy `proc_42e6`,
and 14-document rendering `proc_62a0` / `proc_28b9` passed.
Queued inputs,
mixed representations,
tool instruction-source ownership,
applicable governing scope,
and domain closure remain unfinished.
No provider trial or installed activation occurred.

## Original current-session context-edit inputs

The existing manager input owner now observes native context-edit appends after defensive cloning.
Each actual append gets a fresh input occurrence with `inputOrigin: native-append`;
file-loaded inputs remain `native-load`.
No earlier run,
historical issuer,
authority,
or payload eligibility is invented.
Only supported edit and summary inputs are newly captured;
known run publications and derived system checkpoints are not recaptured.

Context handlers can create an edit after initial context observation.
The private native context return therefore rechecks the original run handle and current manager input owner.
It unions newly observed inputs with the original pre-handler membership.
Removing visible output does not remove the original input;
an unchanged published view is reused without another observation charge.
First empty observation still publishes its empty view.

Between-prompt required red `e1ab1be` / `proc_0fe4`
became native green `b2cd8aa` / `proc_398f`.
Owner,
append/load distinction,
shared-limit,
failure-latching,
and failed-native-write controls passed `proc_61e1`.
Same-request red `b2d9480` / `proc_cc5e`
identified the missing final context boundary.
Superseded-run red `a1c3a1a` / `proc_db41` required exact run-handle revalidation.

The no-op optimization initially skipped the first empty observation (`proc_5206`);
that is fixed.
The image fixture initially expected the nested capture error's text,
then attempted to inspect the failed run after prompt reset (`proc_ef57` / `proc_6aad`).
It now intercepts and rethrows the actual native context error before cleanup,
checking the exact original nested failure.
Corrected owner checks and the 41-mode native matrix passed `proc_b11b`.

Append omission `proc_7ed6`,
final-context omission `proc_0903`,
and prior-input-union omission `proc_a4c1` each failed their required control.
All omitted code is restored.
Creation-owner and manager controls are integrated into the routine suite;
complete native `proc_8bde`,
action `proc_9601`,
source-policy `proc_f730`,
and 14-document rendering `proc_a31c` / `proc_c9e9` passed.
The native ordinal assertion passed without adding one to the header-inclusive position.
Other current inputs,
queued inputs,
mixed representations,
tool instruction-source ownership,
governing scope,
and domain closure remain unfinished.
No provider trial or installed activation occurred.

## Original loaded context-edit inputs

The existing load owner captures original `context_edit` replacement content and null omissions.
Displaced target inputs and shadowed edits remain separate original occurrences.
Actual native replacement construction propagates the edit's input-only identity,
not the displaced target's identity.
Unapplied replacements do not authenticate unchanged or unrelated outputs.

Target IDs remain metadata.
Neither context suppression nor replacement establishes policy revocation,
authority,
permission,
execution facts,
or complete composite origin.
The shared source and byte limits remain unchanged.

Required red `4f164d6` / `proc_d556`
preceded consumer `f001a43`.
Native editable-role and owner checks passed inside `proc_2061`,
but its matrix phase failed.
The equal-edit fixture incorrectly searched array content only;
the native serializer also preserves scalar user text.
After that correction,
`proc_871e` exposed a separate fixture flag collision:
`edit-hidden` accidentally enabled compaction-system hidden behavior.
The fixture now keeps those families separate and explicitly checks the displaced target's content.

Corrected 36-mode matrix,
19 native edit-manager cases,
and owner controls passed `proc_da68`.
Construction omission `ddc6f13` / `proc_e0e8` lost the edited message's original publication.
The observer is restored.
Complete native `proc_36c9`,
action `proc_fafc`,
source-policy `proc_80b6`,
and 14-document rendering `proc_18b1` passed.
Mixed or unsupported representations,
current-run generated inputs,
queued transformations,
tool instruction-source ownership,
governing scope,
and domain closure remain unfinished.
No provider trial or installed activation occurred.

## Original loaded bash record inputs

The existing load owner captures native `bashExecution` command and output text.
Reported exit,
cancellation,
truncation,
path,
and context-exclusion fields remain metadata,
not historical execution facts.
These fixtures store authored records;
they never execute the recorded shell commands.

Actual native formatting propagates input-only identity to the resulting user message.
Excluded and compacted records remain original inputs without inventing visible contributions.
The source does not acquire authority,
permission,
text-part eligibility,
or payload eligibility.
The existing source and byte limits still apply.

Required red `a7a1966` / `proc_f853`
became canonical native green `646453a` / `proc_5aac`.
Formatting variants,
owner shape/getter controls,
the 31-mode native input matrix,
and manager regression passed `proc_589c`.
Conversion omission `1316895` / `proc_30e1` lost the formatted message's original publication.
The observation is restored.
Complete native `proc_aa93`,
action `proc_25cc`,
source-policy `proc_7e06`,
and 14-document rendering `proc_e36e` / `proc_0f9f` passed.

Loaded context-edit replacement inputs,
mixed or unsupported representations,
current-run generated inputs,
queued transformations,
tool instruction-source ownership,
governing scope,
and domain closure remain unfinished.
No provider trial or installed activation occurred.

## Original loaded assistant and tool text

The incumbent text-content profile now includes stored `assistant`,
`toolResult`,
and message-form `custom`/`hookMessage` inputs.
These use the same original load occurrence,
source limits,
and input-only publication mechanism.
Saved role labels,
tool names,
call IDs,
and reported error flags establish neither authority nor execution facts.

Opaque details,
usage,
and nested-call records are not copied into semantic source material.
Legacy `hookMessage` capture occurs before native migration to `custom`;
the saved role remains source metadata while actual native conversion preserves input-only identity.
Mixed assistant thinking/tool-call content and media remain outside this text-only profile.
Unsupported content fails closed at guarded source use,
not during native file parsing.

Required red `b56aa77` / `proc_1259`
became native consumer green `4086f0a` / `proc_b37f`.
The first broader matrix,
`proc_d69d`,
exposed a fixture assumption that the requested source was always first.
Capturing the seed assistant changed that order for branch summaries.
The corrected fixture selects the original role/content and keeps exact source-count assertions.
Corrected 28-mode matrix,
native manager checks including legacy custom migration,
and owner controls passed `816732c` / `proc_0bf6`.
Text-profile omission `e938c5d` / `proc_2f73` produced native `stop` instead of the required source-use `error`.
Validation is restored.
Complete native `proc_1011`,
action `proc_a1d6`,
source-policy `proc_c54d`,
and 14-document rendering `proc_5570` passed.

Loaded context-edit inputs,
mixed or unsupported representations,
queued inputs,
tool instruction-source ownership,
governing scope,
and domain closure remain unfinished.
No provider trial or installed activation occurred.

## Original loaded system prose

The existing load owner now captures original system-message `content` and optional `sections`.
It also captures a compaction entry's stored `systemMessage` prose as its own input,
separate from the summary.
The already-observed native selected path retains each original capability,
including non-transmitted older checkpoint inputs.

Scalar content remains text.
Combined content and sections use the incumbent `native-content-json` source representation,
not a rebuilt request body.
The observer does not read or serialize `toolsAdded` or `toolsRemoved`.
Tool schemas,
tool instruction sources,
and prose origins remain separate responsibilities.
All message and companion records share the existing source and byte limits.

A stored `system` role or section name grants no authority.
Original input identity remains input-only,
without text-part or payload capability.
Native replay may remove null section values before storing a checkpoint;
the original delta and the checkpoint are distinct inputs,
not equal source records.
Native deletion of a section is not policy-revocation evidence.

Required red `2c57586` / `proc_147b`
preceded consumer `4289eec`.
Initial run `proc_3666` passed plain,
section,
and companion cases but exposed an incorrect fixture expectation for replayed null sections.
The corrected fixture checks each original stored field separately.
Owner and native companion controls passed `proc_950c`.
Corrected 22-mode native matrix and manager regression passed `proc_69e4`.
Native reset,
selected-path root retirement,
null sections,
and getter non-invocation controls passed `proc_db9e`.
Retention omission `f29b4be` / `proc_aef6` lost the hidden older companion (`4 !== 5` sources).
The edge is restored.
Complete native `proc_c526`,
action `proc_1b97`,
source-policy `proc_e729`,
and 14-document rendering `proc_949d` / `proc_9332` passed.
The routine native matrix now also covers a native-written 32-compaction history exceeding the shared source cap.

Other loaded roles,
current-run generated inputs,
queued or unobserved transformations,
tool instruction-source ownership,
governing scope,
and domain closure remain unfinished.
No provider trial or installed activation occurred.

## Original loaded summary inputs

The existing load owner captures original `compaction.summary` and `branch_summary.summary` text,
including empty inputs.
Summaries share the user/custom source and byte limits.
Equal text keeps distinct source occurrences.
No saved summary authenticates prior human instructions or receives inherited authority.

A native compaction entry can project to a stored system message and a summary.
The private constructor observation attaches the summary input only to the actual summary output.
Native summary-to-user conversion keeps input-only identity,
not text-part or payload capability.
The summary publication never authenticates the system companion.
Its separate capture is described in “Original loaded system prose”.

Required red `fced351` / `proc_640e`
became canonical native green `cc6c408` / `proc_1efc`.
The 14-mode loaded-input matrix,
including empty summaries,
passed `proc_503b`.
Explicit two-output row,
legacy migration,
failed-rewrite withholding,
native conversion parity,
shared bounds,
and occurrence controls passed `proc_047a`.
Constructor omission `076347b` / `proc_de3c` lost the summary identity;
that edge is restored.
Companion-attribution omission `f165d21` / `proc_9769` incorrectly marked the stored system message.
The single-output guard is restored.
Complete native `proc_18d1`,
action `proc_06a7`,
source-policy `proc_62b3`,
and 14-document rendering `proc_7bd6` passed.

Current-run generated summaries,
other loaded roles,
queued or unobserved inputs,
applicable governing scope,
and domain closure remain unfinished.
No provider trial or installed activation occurred.

## Original loaded custom-message inputs

The existing manager-load owner now captures supported `custom_message` inputs as well as stored user messages.
It retains bounded content and native `customType`/`display` companion data.
Opaque `details` are neither read for source assessment nor serialized as semantic metadata.
An authority-looking custom label or details object grants no authority.

Input-only aliases follow the actual native custom-to-user conversion.
This preserves original input identity without producing text-part or payload capabilities.
User and custom inputs share the original source and byte limits.

Required red `a7b820e` / `proc_e115`
became native green `e50abe1` / `proc_71df`.
Native matrix and manager/conversion parity passed `proc_f925`,
covering string,
structured,
empty,
nullish,
unsupported,
and over-bound inputs.
Shared-bound and opaque-metadata owner controls passed `proc_8cb1`.
Conversion omission `cdbbdf8` / `proc_6d32` lost the original alias in the actual SDK converted message.
The omitted branch is restored.
Full native `proc_297d`,
action `proc_7871`,
source-policy `proc_de4f`,
and 14-document rendering `proc_4922` passed.

Other loaded message roles,
queued inputs,
historical issuer authentication,
and governing-domain closure are still unfinished.
The separate loaded-summary increment is described in “Original loaded summary inputs”.
This is another original input consumer,
not a production qualification or installed activation.

## Original owned session-load inputs

The next accepted-domain increment records a new native load occurrence,
not the historical issuer of a saved message.
Required red `ae9d582` / `proc_ddb2` opened a native-written session through a distinct owned manager
and reached canonical review with the prior user text in the body,
but without an original source view.

The existing private manager now owns capture of supported loaded user-message content
after its defensive clone and before migration.
Each record is unregistered,
with historical issuer,
authority,
and priority unestablished.
Saved roles,
IDs,
timestamps,
paths,
and equal bytes do not carry an older capability into this load.
Loaded sources have their own session-input occurrence,
not an invented emitting run.

A default message-copy observer preserves input-only aliases during initial SDK projection,
before `AgentSession` attaches its run observer.
The fixed manager reader supplies original loaded-source lookup to the existing source collection.
It stores source/publication references,
not a parallel transcript.
Loaded input aliases do not grant text-part or serialized-payload eligibility.

Capture failures remain owner-local until source use.
The native file loader still runs its parser,
migration,
and original error paths.
The source-use assertion fails on capture errors rather than publishing a truncated source set as complete.
The incumbent data copier,
source limit,
source-byte limit,
and snapshot ledger are retained.

Initial stage `proc_0bb0` rejected the injected helper imports.
Explicit mappings in `342b64f` repaired that import allow-list without accepting arbitrary file URLs.
Native consumer `proc_397c` and publication regression `proc_9794` passed.
Pure identity,
bounds,
and deferred-failure controls passed `proc_4bbb`.
Native manager identity/value parity passed `proc_3dd2`;
native migration and invalid-file controls passed `proc_68f4`.
Capture-failure omission `342436b` / `proc_647a`
and pre-attachment observer omission `4e579c9` / `proc_1f27` failed and were restored.
Full native `proc_2f89`,
action `proc_0d50`,
and policy-source `proc_1b35` passed.

Failed in-place loading exposed an incorrect fixture expectation in `proc_12a4`.
Corrected `proc_84c8` follows surviving native state or withholds provenance;
it does not relabel prior indexed values as a newly attempted load.
The details are in main `doc/troubleshooting/pi-sdk-staging.md`.
Material-guard omission `9e05514` / `proc_9357` incorrectly accounted input-only membership;
the guard is restored and complete action regression `proc_eb8e` passed.
The earlier attempt `proc_8568` stopped at a Git lock collision and ran no test.
Runtime/generator sources match full-native checkpoint `a3d2086` by scoped Git diff.

This initial profile covers supported stored user-message text content.
It does not authenticate a prior human,
inherit a registration,
establish other loaded message kinds,
or close the governing domain.
No provider trial or installed activation occurred.

## Original selected-path inputs after native projection

Native compaction and context edits can remove an input from the rendered transcript
without establishing that its instructions ceased to govern.
The selected-path increment retains known original run inputs through that boundary,
without assigning them authority or scope.

The maintained private manager generator observes the path already computed by native `buildSessionProjection`,
before compaction and edit projection.
It adds no second traversal,
JSON parse,
or retained entry tree.
The run owner keeps original source/publication references and the manager's original root occurrence.
At context consumption,
it re-reads that root and verifies original tuple membership.
Payload eligibility is checked separately.

Native root occurrences rotate on branch,
reset,
fork,
and load operations.
Ordinary append-only extensions preserve the observed selected path as a prefix.
The stored observation therefore does not claim to describe every entry in the later current path.
Missing or unknown input origins still keep `request-source-domain` unresolved.

Required compaction red `53b92f5` / `proc_a393`
became native green `a3ffc46` / `proc_e0fd`.
The expanded native matrix `proc_d92e` passed original prompt and handler retention after compaction,
pre-capture context removal and equal replacement,
and reset without a refreshed path observation.
Native root controls and installed/private projection parity passed `proc_501a`.
The first native reset omission `proc_95d4` passed,
so it was not claimed as a sensitivity witness.
The original-owner interleaving control `65ec659` / `proc_6875` then failed as intended
when a changed root retained one old source instead of zero.
That is an authored owner-boundary control,
not proof of a native scheduling interleaving.
The guard is restored;
owner controls and the 40-mode native matrix passed `proc_70e3`.
Runtime and generator sources match full-native checkpoint `71d727c` byte-for-byte by scoped Git diff.
Full native `proc_c35b`,
action `proc_066c`,
and 14-document rendering `proc_aa7e` passed.

Known originals are retained even when they have no current serialized contribution.
This does not establish the origin or authority of an unknown compaction summary,
a disk-loaded history,
queued input,
or an unobserved transformation.
The accepted all-host-input domain remains unfinished.

## Input-origin-only links for unsupported prompt representations

A native prompt containing an image or normalization hint still has an original expanded-text input.
It must not disappear from source assessment merely
because its representation is outside the text-only association profile.
Required control `1c03dff` / `proc_1b9c` failed on that missing retained input.

The existing publication alias index now distinguishes `input-only` links
from text-part and payload capabilities.
They retain the original input through actual native copies without claiming unchanged current content.
Changing such a message into an equal text-only shape does not upgrade its capability.
Independent copies still have no origin until an actual native copy edge connects them.
Original occurrence collisions must not overwrite another publication.

The native image/hint retention controls passed in the 35-mode matrix `proc_0441`.
Pure publication controls also check that observers do not invoke getters
and that input-only aliases cannot become text contributions.
Collision-guard omission `7bd4d85` / `proc_e92d` overwrote another original occurrence and failed its control.
The guard is restored.
Complete native `proc_0f88`,
action `proc_df6e`,
and 14-document rendering `proc_1715` passed.
This adds neither authority nor request-domain completeness.

## Retained original run inputs after another prompt

The user accepted governing-domain option A on 2026-10-09:
account for all original host instruction inputs within their applicable scope,
without treating unregistered possible instructions as absent.
The accepted scope is recorded in
`doc/planning/pi-auto-mode-governing-domain-closure.md`
in the main repository.
It is not a completeness declaration.

The first implementation increment retains known original prompt and handler inputs
that are actually present at the native context-emitter entry after another prompt.
The existing run-scope lookup and publication alias index supply the original references.
The current run stores frozen `retainedMessageOrigins` links;
it does not promote the earlier emitting run into active ancestry.
The combined `runMessageInputs` collection retains those same source records,
not copied text or a second message inventory.

Source retention and serialized contribution eligibility remain separate.
A context handler can remove or independently copy the old message:
its original input remains available for assessment,
but the replacement acquires no contribution capability.
Observed publication retirement remains permanent.
Known native copy edges preserve message aliases even while a new prompt has no current run;
system-publication copying still requires its current run.

Required consumer red `db94d91` / `proc_b7c9`
became native green `387c717` / `proc_0333`.
Prompt and handler matrices passed `proc_93b2`;
canonical source-binding controls passed `proc_a3ce`.
Retained-membership omission `4057cef` / `proc_0e9a`
incorrectly accounted a nonmember and is restored.

The first preparation-copy fixture `proc_9841` was invalid evidence:
a nonexistent `replaceMessages` call was swallowed by the native handler catch.
The setter control `proc_2904` did not prove its copied objects reached conversion.
The refined fixture actually routes its saved native preparation copy into context conversion.
Restoring the old no-current-run gate then failed as `b164094` / `proc_9d78`;
the message copy path is restored.

Full native `proc_dd40` reached the original handler-input fixture
and failed its obsolete count after another prompt:
the correct retained source count was 12,
not 6.
The fixture now checks exact distinct original source references rather than enlarging bounds;
targeted owner regressions passed `proc_c5e4`.
Full action `proc_4576` and source-policy `proc_9fad` passed.
The 33-mode expanded-prompt/retained-input matrix passed `proc_b46c`.
Actual native branch departure/return and manager reset/fork controls observed their input boundaries,
but the original bound policy runtime correctly refused new transport with `Root lease has been retired`.
Branch return preserves observed original input links without reviving the retired root.
The reset and uninstrumented fork-copy paths do not borrow the earlier run's origins.
These are manager/projection controls,
not qualification of the full host resume or fork workflow.
The restored full native rerun `proc_3da2` passed.

The initial owner-local route did not establish disk-loaded,
queued,
compacted,
or pre-capture-excluded origins.
The selected-path increment now retains known original inputs omitted by native projection;
unknown summary and historical origins remain unresolved.
It does not prove revocation when a source is absent from the current transcript.
Registration,
applicable governing scope,
and request-domain completeness remain independent.
No new authority,
permission,
provider trial,
or installed activation occurred.

## Original expanded-prompt input and contribution

The incumbent run owner now captures the actual native `expandedText` handoff
after input,
skill,
and template expansion.
This establishes that native occurrence,
not unmodified caller wording,
human origin,
or instruction authority.
The input remains unregistered.

The existing source collection now exposes `runMessageInputs`
(renamed from the collector snapshot's `handlerMessageInputs` on 2026-10-09).
It references the same run-owned expanded-prompt and handler records.
Per-run `handlerMessageInputs` remains the emitter-only collection.
No parallel message inventory,
second request parse,
or request re-encoding was added.

The existing publication/copy index now supports the actual text-only native user construction.
Native identity pass-through checks known original aliases before retaining them.
Independent deep copies,
equal context-edit replacements,
and replaced text parts do not inherit lineage.
Wrappers retaining the actual original part capabilities remain distinct from independent material copies.
Payload custody still owns fresh arrays and text blocks across awaited authentication
and discards transient aliases at sealing.

Image-containing and hint-modified constructions retain the original expanded text
but remain outside this association profile.
Empty and malformed text receives no invented contribution.
Active ancestry is supported;
prior prompt occurrences cannot supply the new prompt's source association.

Missing origin failed `proc_2f9f`;
retained origin without association failed `proc_6dd4`;
initial native consumer passed `proc_3e36`.
Native input-owner controls passed `proc_1190`,
conversion parity `proc_f69b`,
source-policy `proc_898f`,
and action `proc_dd50`.
The 23-mode native input/representation matrix passed `proc_7109`,
including actual native image and normalization-hint paths.
Original scalar-publication controls passed `proc_b338`.
The pass-through retirement positive control passed `proc_eebb`.
Pass-through omission `proc_334e` revived invalid prompt lineage on a later review;
scalar-publication omission `proc_b17c` admitted a copied publication;
hint-profile omission `proc_7ea3` changed unsupported native output into an exception.
All guards are restored.
Full native `proc_5d3b`,
full action `proc_15a2`,
and 12-document rendering `proc_2db5` passed.
Source accounting remains partial and `request-source-domain` remains unresolved.

## Original orphan-heading directive use

Original top-level orphan headings now use the existing clause-operation consumer
when their directive meaning is separately admitted and their original source has registered request scope.
The existing parser visits root children only;
a heading inside a blockquote,
fence,
or list remains inside that different original candidate.
Both ATX and setext heading forms use the same original heading capability.

Heading syntax supplies neither authority nor meaning.
Priority still comes from the exact original registered source,
and copied or unregistered sources receive no substitute priority.
Unknown and abstained meanings remain unresolved;
an admitted `none` adds no directive.
Contextual-heading aggregates remain separate:
`potential-instruction` is not a directive and cannot use this route.
Structured JSON is not reinterpreted as Markdown.

Required native red `proc_9b23` lacked the registered heading's prohibition.
Initial native `proc_3043` and action/source `proc_c39d` passed.
The expanded compiler matrix passed `proc_1da2`,
including quoted,
fenced,
list,
unregistered,
abstained,
and setext controls.
No source scope,
question budget,
production admission,
or request-domain completeness is enlarged.
Quoted-scope omission `proc_5954` wrongly lowered a quoted heading;
source-authority omission `proc_9adf` invented priority for an unregistered heading;
admission omission `proc_18d4` treated an abstained heading as interpreted.
All guards are restored.
Full native `proc_b8db` and action/source `proc_39cd` passed.
The expanded 12-document check,
including the effect contract,
passed `proc_40a3` with zero diagnostics.

## Original complete-group scheduling and multi-program application

The original dispatcher now adds its effective `groupExecutionMode`
to the existing frozen prepared-member records.
The value comes from the same latched `serial` decision that controls native entry turns,
including a sequential tool forcing the whole group to run sequentially.
The existing prepared-manifest serialization carries it;
there is no new parser,
source inventory,
or scheduling owner.

Missing original scheduling failed `proc_c205`.
Explicit modes,
per-tool forcing,
borrowed argument mutation,
live configuration retirement,
and original error-class identity passed `proc_5337`.
The live configuration negative preserves
`Native tool "codemode" or its context owner changed after request preparation`;
it is not a reason to relax freshness.

Missing complete-group application failed `proc_e3d5`.
The compiler now consumes admitted whole-proposal relationships for matching original inspected programs
with a consistent original scheduling fact and no bypassed independent fact owner.
Application records retain the original `preparedOperations` array.
They do not turn effect estimates into code-established effects.
Mixed or uninspected programs,
missing/inconsistent schedules,
unknown meanings,
abstention,
and independent fact-owner gaps remain unresolved.
Same-source incorporation remains single-member.

Parallel/sequential native consumers passed `proc_5a03`;
full action tests passed `proc_e8e3`;
the 12-mode native scheduling/application matrix passed `proc_bce2`.
The current dispatcher also passed actual fixture-authorized execution,
queue,
abort,
and disposal controls in `proc_c317`.
All deliberate guard omissions are restored.
Producer `proc_e6ad` misreported tool-forced sequential scheduling;
consumer `proc_b025` accepted inconsistent schedules;
`proc_54ee` erased an independent fact-owner gap;
`proc_9907` accepted a wrong inspected profile.
Each control failed.
Restored full native `proc_c1a5`,
action `proc_2992`,
and eleven-document render `proc_f581` passed.
Production admission and installed activation remain unchanged.

## Express same-source quoted-unit incorporation

The user accepted option A on 2026-10-08.
The new `unitAdoptionAdmissionPolicy` fixes the `unit_adoption` definition,
model,
inspected program profile,
and synchronous acceptance function separately from clause interpretation.
No production semantic admission policy is supplied.

The original judgment selects a bounded opportunity only when one existing source has one tagged adopter
and one blockquote target,
and the operation has one original prepared member.
That shape is not authority.
The existing instruction-binding and program-selection maps retain the exact pair and member.
All original clause candidates still receive their independent assessments;
pair questions join the same batch,
deadline,
and existing question bounds.

A positive admitted pair must represent the complete adopting instruction as exact,
unmodified incorporation.
Code separately requires its original registered authority and original-request scope.
A conflicting admitted primitive effect on the adopter prevents incorporation.
An amendment,
ambiguous reference,
extra independent norm,
unknown condition,
unregistered source,
or unadmitted pair does not establish target scope.

For a selected quoted target,
the clause estimator reports conditional directive meaning,
not operative authority.
The target must independently pass its own clause-admission policy.
Only that target relationship supplies a permit,
prohibit,
or prerequisite effect.
Incorporation itself supplies no permission or additional directive.
The adopter's original primitive estimate remains unchanged;
`representedByAdoption` identifies the separate admitted control meaning.

The consumer discharges named clause-meaning and clause-operation obligations for that adopter
and the exact quotation-use restriction for that target.
Independent implementation-fact owners retain their operation obligations
and receive no estimated incorporation operation marker.
Other source,
heading,
request-domain,
authorization-satisfaction,
and approval-channel obligations remain.

The actual-serializer control failed `proc_e0f1` with zero pair estimates instead of one.
Initial implementation passed `proc_d760`.
Expanded native `proc_84c5` passed 24 modes;
the mixed-use,
priority,
fact-owner,
and target-provenance expansion passed `proc_7c37`.
Original selection,
wire,
request-scope,
and priority controls passed `proc_8f62`.
Guard-removal controls rejected missing positive meaning,
foreign request scope,
unadmitted target effects,
foreign members,
copied projections,
conflicting adopter meanings,
and unregistered authority.
Every omitted guard is restored.
Full native `proc_3c3a` passed with 36 incorporation scenarios;
full action `proc_0b25` also passed.

The full-policy control first exposed 674 pre-fact dependency checks in `proc_8a16`.
Pair derivation now uses the incumbent synchronous binding segment with entry and exit checks.
Restored measurements were 38 checks for the short-host profile and 41 for native-default.
No cached freshness crosses an `await`.
The existing source,
candidate,
question,
and byte bounds are unchanged.
These are authored mechanism controls,
not model-accuracy evidence or installed activation.

## Native source association through the real serializer

The SDK 1.1.0 private adapter passed full native regression `proc_a814`.
The new `serializer-association.test.mjs` uses the installed OpenAI serializer and local SSE responses,
not a manually assembled model-request body.
Missing-association control `proc_6ab2` reached canonical compilation and failed at the required source-use assertion.

The existing run owner now retains the original sections publication.
Its source capability is explicitly carried through the private manager's defensive copies
and the SDK's initial context clone.
Independent equal-text replacements remain unassociated.
The initial request-observer profile qualified only the fixed installed serializer route,
a full current-run sections publication,
and the existing linked context source.
The expanded profiles described in "Original rendered prompt-input contributions" also consume base,
replacement,
forced,
and custom-section contributions.
Additional independent system content,
sampling overrides,
and unrelated stream wrappers remain outside association.

A payload callback can retain mutable aliases across the SDK's asynchronous body construction.
The adapter therefore copies the ordinary payload root,
message array,
and original associated message after the callback.
Only these newly owned containers are frozen.
Borrowed callback objects remain mutable;
changes to discarded aliases after materialization do not invalidate the historical request.
Accessors,
serialization hooks,
and independently copied message nodes remain unsupported for association.

The observer reuses its existing JSON validation to check the particular terminal field.
It retains no parsed request tree and performs no second parse or request re-encoding.
The original receipt owns the resulting contribution and its original request reference.
The canonical consumer reports `requestSourceAssociationUses`.
An accounted use covers only the particular original context contribution,
not the full composite system message,
authority,
valid delegation,
or the request's governing-source domain.
The existing `request-source-domain` obligation remains.

Matrix `proc_6043` passed 11 local modes,
including source-copy rejection,
changed content,
callback hooks,
opaque streams,
post-materialization alias mutation,
and permanent retirement after restoration.
These controls performed no provider request or native tool execution.
Equality substitution `34a0355` failed `proc_92e5`:
an independent equal-byte payload became accounted.
The original identity guard is restored.
Expanded matrix `proc_2731` passed 15 modes,
including accessor substitution,
getter-call parity,
and callback-alias replacement during actual client authentication before body materialization.
Custody omission `8bc2eb4` failed that timing control in `proc_7738`;
the owned copy is restored.
Native context-transform parity `proc_3ece` compared 10 installed/private SDK sessions.
Full restored native `proc_c4d5` and action `proc_2fa4` passed on SDK 1.1.0.

The association currently requires explicit bootstrap selection of `nativeOpenAIStream`
and a full current-run sections publication.
The default model-runtime wrapper and later runs without that publication remain outside this bounded association.
Only content origin is associated:
the complete message node and its other fields are not claimed semantically unchanged.

Independent review exposed input-envelope accessors and copy-observer getter side effects.
The native controls reproduced both in `proc_0731` and `proc_98b2`.
The implementation now requires ordinary source-bearing data fields
and observes copies through descriptors without invoking accessors.
Early seal exits also discard transient payload references.

## Whole-request and contextual-heading follow-up

Whole-request identity omission `73fda0d` failed `proc_d064`;
restoration passed `proc_de54`.
Removing the independent request-domain obligation in `29272de`
made `proc_e7ca` approve and enter an inert tool body.
That omission is restored;
full native `proc_8a24` and action `proc_74d6` passed.

Contextual headings now have a separate original aggregate content use,
`heading_instruction_content`,
under their own captured startup policy.
Implementation `2465042` preserves one group per original source.
An admitted `none` clears only that group's independent heading-content obligation.
Potential or unknown content supplies no directive,
authority,
or governing-domain completeness.
Potential-as-resolved omission `ebe2514` failed `proc_9d89`.
Restoration `e9161b3` passed native `proc_6fd2` and action `proc_a023` on SDK 1.0.4;
the current SDK 1.1.0 full regressions also cover these consumers.

The user accepted express same-source incorporation in
[scoped incorporation of quoted policy text](../planning/pi-auto-mode-quoted-unit-incorporation.md).
Its private consumer is implemented and under expanded verification.
Q21's governing-instruction principle remains settled;
no production semantic admission profile or installed activation is adopted.

## Configured native clause compiler

Private implementation `f172216` connects original typed estimates to the existing clause lowerer and resolver.
The ordinary native reviewer can now produce nonempty permission/prohibition inputs
and separate authorization requirements,
without an injected final decision.
The startup `semanticAdmissionPolicy` fixes model,
relation definition,
inspected program profile,
and a synchronous acceptance function.
No production policy or probability band has been adopted.
The unconfigured runtime still sends no semantic requests.

The configured raw transport reuses the existing wire and response parser.
It verifies the reported model and complete answer set
before the original question-selection owner retains model attribution.
Legacy normalized strings supply no model attribution and cannot satisfy the configured compiler.
There is no new evidence registry,
source-hash lock,
tools-field extraction,
or serialized answer envelope.
Original `semanticQualification: 'not-established'` records remain unchanged.
That tag limits what transport establishes;
it is not a permanent veto on downstream code admission policy.
Authored test acceptance rules establish no empirical model quality.

The compiler authenticates the original batch,
uses code-registered original source priorities,
and preserves uncertainty instead of admitting unsupported or unknown meanings.
Its acceptance function cannot return authority,
complete coverage,
authorization satisfaction,
or an action verdict.
An inspected outer-program shape is not qualified callee effects.
Consumed original candidates now discharge their supported interpretation and operation uses.
Remaining request-domain,
contextual-heading,
authority/delegation,
and unsupported operation uses stay explicit.
All current native compiler cases therefore withhold execution.
Production activation and the full migration remain unfinished.

Initial canonical control `proc_b9d8` passed 17 modes.
Action suite `proc_83cd` passed the new policy-capture,
model-attribution,
and original-evidence controls.
Expanded full native regression `proc_daa0` passed 20 compiler modes
plus the existing runtime suites.
Those additional controls cover repeated codes across sources,
multiple clauses with different relationships,
reordered answer-object fields,
exact original clause-projection association,
mixed direct/program membership,
and missing raw transport before native hook installation.

Model-match omission `7e570a2` failed `proc_885f`:
a mismatched model reached the acceptance callback.
Restoration passed `proc_9198`.
Abstention omission `004091c` failed `proc_6270`:
`admitted` was returned instead of `abstained`.
Both guards are restored.
Full restored native `proc_d5a8` and action `proc_059a` passed.
No semantic provider request,
genuine approval,
or installed change occurred.

### Inspected-effect rubric correction

The permission rubric previously required every relevant condition to be established by supplied code facts.
That excluded Q13 B's accepted use of qualified inspected-form effect estimates.
A permission conditional on a read-only inspected script exposed the contradiction.
Independent transcript review confirmed the mismatch against
[the accepted effect contract](../planning/pi-auto-mode-effect-contract.md).

The rubric now separates estimated effects from external runtime facts.
Effect estimates remain subject to code-owned admission;
canonical paths,
file contents,
account ownership,
and existing authorization cannot be invented by the estimator.
Full native `proc_968f` passed 22 compiler modes;
action `proc_d032` also passed.
The new local cases exercise an authored estimated-effect relationship
without promoting it to `establishedEffectFacts`.
The missing-external-fact case supplies `unknown` explicitly:
it verifies preservation of uncertainty,
not independent enforcement against an incorrect model answer.
No semantic accuracy or production profile was established.

The unconditional operation premise in `compiler.mjs` was another concrete consumer defect.
Canonical control `proc_7bd1` failed because an admitted relationship to the whole original inspected program
still could not satisfy its operation-evidence consumer.
The compiler now records per-clause `operationApplication` using the original member and program inspection.
Supported admitted relationships replace the universal static-effect demand for one enclosing inspected program
without a separate implementation-fact owner.
At that checkpoint,
unknown uses,
multi-member composition,
and independent fact-owner gaps remained unresolved.
The current bounded multi-program route is recorded in
[Original complete-group scheduling and multi-program application](#original-complete-group-scheduling-and-multi-program-application).
Full native `proc_7b44` passed 23 compiler modes;
action `proc_b3d0` passed.
The estimated relationships remain estimates,
not code facts or complete governing-source proof.

Already-retained unclassified source spans now have original selectable capabilities
through the existing index,
source set,
judgment,
and typed wire.
The canonical regression failed `proc_3ca8` with zero estimates instead of two
for equal untagged prohibitions in independent original sources.
Implementation passed `proc_b70d` and full native `proc_7e1b`.
Source-owner and action checks passed `proc_db47`;
native quotation control `proc_95a9` passed.
Quotation-scope omission `8a03068` failed `proc_4ab6` by lowering quoted data with developer priority.
The guard is restored and full native `proc_5707` passed.
Structured content remains exact JSON,
not reconstructed Markdown.
Copied diagnostic spans do not authenticate candidates.

The updated full-policy control measured 212 and 217 retained candidates
for the short-host and native-default linked profiles,
including 207 tagged rules.
The corresponding typed requests were 208,145 and 216,999 bytes.
Existing bounds still reject independent copies;
no source equality merge,
batch split,
or limit increase occurred.

Authority need is now derived from consumed original relationships.
Sources whose consumed candidates are all admitted `none` no longer impose blanket authority premises.
Operative,
unknown,
abstained,
unrepresented-source,
and unit-level delegation cases retain their gaps.
The canonical control failed `proc_9516` before this change;
full native `proc_7fff` and action `proc_66e7` passed after it.
This is not a declaration that the complete source or request has no other governing instructions.

Orphan headings and source-local coverage uses are now implemented in `6ca7009`.
The missing original heading control failed `proc_7986`;
heading-none,
heading-unknown,
and heading-operative controls passed `proc_2e5a`.
Source-local candidate use is discharged only for admitted supported interpretations.
Heading strings supplied only as ancestry retain a separate normative-use obligation.
The original request observation remains attached to its independent request-domain issue.

Omission `d8baccb` failed `proc_110b` by removing the original orphan-heading candidate.
Restoration `69e69d2` passed full native `proc_44b5` and action `proc_9324`.
The configured-compiler matrix now has 26 scenarios,
and the linked full-policy counts remain 212 and 217 candidates.
No production policy,
paid semantic trial,
new source inventory,
or installed activation was introduced.

The initial request-source proposal incorrectly required another grant for an already-retained input.
That requirement is withdrawn in
[the corrected scope proposal](../planning/pi-auto-mode-batched-assessment.md#proposed-expansion-of-request-source-accounting).
The original observer already owns the complete exact request bytes;
selecting and consuming them through the same request/judgment owner is authorized implementation,
not a new source origin or parallel message inventory.
That consumer is now implemented through the original judgment's existing binding and assessment owners.
The absent selection failed `proc_bd4a`;
initial canonical none,
potential-instruction,
and unknown controls passed `proc_86c2`.
A separately captured `requestMaterialAdmissionPolicy` selects `request_instruction_content`,
not the clause-authority rubric.
Independent clause and whole-request questions share the same original batch and deadline.
No observer change,
extra parsing,
message inventory,
or request re-encoding was needed.

Expanded full native `proc_53ff` and action `proc_5aed` passed.
The controls preserve complete whitespace,
escaped text,
and structured message content,
and distinguish independent requests with identical bytes.
They cover request-only operation,
missing or mismatched request policy,
abstention,
bootstrap-copy isolation,
and rejection of member/clause substitution.
Whole-request operative answers acquire no single authority tier or directive lowering,
and `none` still cannot certify request-wide governing completeness.
Origin association,
contextual-heading meaning,
and valid delegation remain separate obligations.

### Review disposition and remaining work

The default Advisor call reported an operation deadline and unconfirmed remote settlement;
it supplied no review.
Explicit alternate `hyper/deepseek-v4-pro-0813` returned a transcript-based review,
not an independent execution.
Its missing-transport and multi-clause verification suggestions are covered by the expanded native controls.
The existing issuer maps the immutable candidate list in order;
the original selection owner preserves that exact issued order.
No new identity registry was needed for correlation.

Do not adopt the suggested direct lowering of `states_approval_prerequisite`.
Its actual definition in private
`contract/research/instruction-prerequisite-relation/cases.mjs`
expressly excludes current-operation applicability and authorization satisfaction.
A positive scalar cannot establish an applicable authorization requirement.
Likewise,
a single native-read member does not expand the existing execute-body fact
into complete callback,
transfer,
or whole-operation coverage.

Remaining authorized work is decision-specific source/delegation and operation coverage through existing owners.
The compiler mechanism is no longer the unconditional empty-program placeholder.
Do not mistake its configured test policy for production qualification,
or treat the remaining implementation as automatically a blocker.

## Continuation correction: predecision program inspection

The assistant incorrectly stopped after verifying partial helpers.
The remaining consumer implementation is authorized work,
not a reason to wait for another continuation prompt.
The protected `AGENTS.md` remains unchanged;
the existing unapplied `PXQ` proposal is tightened to distinguish missing implementation from a genuine blocker.

The immediate consumer requirement is concrete:
the native reviewer must receive the validated original program declaration before permission or execution.
Before `0381bdc`,
the program owner parsed that declaration only inside `openProgram`,
after an execution token became active.
Execution controls that independently allowed a fixture do not verify this earlier policy-consumption point.

Work now reuses the existing declaration decoder and ownership check,
the original callable-tool snapshot,
and the original judgment member records.
The declaration must be available before review and reused by identity during execution,
under the same preparation-inclusive budget and currentness checks.
No new registry,
source-hash lock,
semantic qualification,
or provider trial follows from this work.
Validated outer syntax still does not establish complete effects or instruction meaning.

The missing predecision reader failed `proc_c2f6`.
Implementation `0381bdc` now compiles through the existing decoder before review,
binds its original declaration to the judgment's prepared member,
and requires execution to reuse it by identity.
Canonical control `proc_063c` passed without an injected Allow or model request.
Execution and shared-queue controls `proc_3f37` passed.
The source remains private;
no installed SDK or plugin was changed.

The policy inputs now retain `programInspection`
and revalidate it through the original judgment.
Referenced callable metadata participates in the original dependency checks.
Unsupported shapes close evidence before review;
changed callees permanently retire eligibility.
Expanded canonical controls `proc_af20` passed.
`proc_355e` was a test expecting the original decoder diagnostic in a rendered native outcome;
the corrected control forwards the canonical handler unchanged
and checks its retained thrown error instead.

Required-inspection omission `003d6ec` failed `proc_6aa8` after running three inert children.
Restoration `33e37ca` passed the full native suite in `proc_4785`.
Document check `proc_3acf` also passed.
This verifies the preparation dependency,
not completion of the remaining consumer work.

## Accepted policy-consumer slice: partial implementation

The user accepted the focused policy-consumer slice on 2026-10-08.
Its contract is recorded in
[the batched-assessment plan](../planning/pi-auto-mode-batched-assessment.md).
The initial implementation established clause lowering and original-owner premise diagnostics.
The configured native clause compiler now consumes those components;
the historical controls in this section concern the initial partial implementation.

Private `contract/integration/action-policy/clause-lowering.mjs`
maps authored established permission and prohibition relationships to directives.
Prerequisites become only authorization constraints.
Satisfaction and approval-channel sufficiency remain independent code inputs.
Unknown and unsupported meanings remain gaps;
`none` adds no permission.
Raw probability vectors,
qualification flags,
unsupported fields,
and accessors cannot stand in for established relationships.

Implementation `4591d78` passed `proc_4b77`,
including 22 malformed-input controls
and the incumbent Allow/Deny/Stop binder with controlled responses.
These are authored relationships and fixture selections,
not model-quality evidence or genuine human approvals.

Private `premises.mjs` and `evaluate.mjs`
now return frozen `premiseIssues` alongside the derived `missingPremises`.
Issues retain original source,
clause,
and prepared-effect references.
They distinguish registered source/request bindings from unregistered or mismatched sources.
Equal text cannot borrow registration.
Unclassified material and member-only facts remain visible gaps.
Unsupported implementation-evidence kinds reject instead of being described as established member facts.

Premise implementation `427416d`,
tests `ff9f110`,
action `proc_2503`,
and canonical native control `proc_c8a8` passed.
The canonical test uses the ordinary native reviewer,
not an injected final Allow.
It verifies withholding and retained identity,
not operative approval.

The incorrect prerequisite-to-`require-approval` mutation `16232a0`
failed `proc_bf63` at the outcome assertion:
`ask` instead of `deny` for an independent prohibition.
The original constraint lowering is restored.
Restored action suite `proc_0816` and full native regression `proc_72fa` passed.
Document check `proc_9293` passed before this final result note.

The remaining limit is explicit:
source/delegation coverage and complete-operation coverage remain missing under their current contracts.
The unconfigured evaluator sends no inference.
The configured compiler calls the lowerer using original estimates admitted by fixed code policy,
but the resolver's complete-premises branch is still unreachable with the current coverage producers.
The lowerer's defensive branch for a future catalog label is also unexercised.
Do not call this a completed policy consumer or claim that it can now approve ordinary native work.

## Current native program integration

Current SDK program composition is implemented privately in
`contract/integration/native-batch/program-source.mjs`,
`tools.mjs`,
and `prepare.mjs`.
It reuses the original judgment,
program and result owners,
native nested runner,
queue,
and recorder.
The actual sandbox and native script conversion run against the original request's executable-tool snapshot.
The installed plugin and SDK remain unchanged.

Initial control `proc_a75a`,
execution matrix `proc_20e0`,
and subsequent full native suite `proc_2615` passed.
These fixtures independently authorize authored inert programs.
They do not demonstrate native policy admission,
complete semantic effects,
or genuine human approval.
Preparation now emits nine SDK-derived modules.

Currentness follow-up found and repaired public runner shadowing
(`proc_305f` to `proc_5adc`,
fix `87b8e7e`).
Transcript-based Advisor review then identified mutable prototype routing and shallow schema comparisons.
Prototype control `proc_6072` reproduced redirection;
input-schema control `proc_c467` reproduced execution after in-place mutation.
Fix `3100219` passed `proc_9d00`.
The existing tool owner now retains schema data descriptors,
including TypeBox symbols,
without freezing borrowed objects or extracting another JSON representation.
The fixed request-wide profile limits property and object entries to 10,000.
This protects runtime schema dependencies,
not interpretation or permission.

Expanded verification now asserts retained original failure causes,
settled child execution,
and one conversion per returned native result.
`proc_3948` first exposed a test counting all declared groups as reached;
the test was corrected against the original owner's `unreached` state.
The corrected 18-case matrix `proc_5caf`,
schema-owner checks `proc_a52b`,
full native suite `proc_c1b1`,
action `proc_edaa`,
and source `proc_bdbb` passed.
Schema unit checks include permanent retirement after restoration,
but use execution-position facades rather than claiming native authentication.

Shared-queue control `proc_0d8b` passed release,
queued abort,
and disposal while a child was suspended.
Release executed six children across separate parent recorders;
abort and disposal each entered only the initially suspended child.
The driver verified empty child stderr,
original judgment reuse,
settled children,
parent-specific usage and updates,
late-context rejection,
and refusal to detach before the agent run settles.
`proc_7f41` was an inaccurate test regex for that existing detach diagnostic.

A concrete missing consumer requirement then emerged:
native Codemode can call inactive deferred tools,
but the adapter had captured only active tools.
`proc_7bfe` measured that limitation;
the required callable behavior failed `proc_f19b`.
Implementation `7d9bdeb` extends the existing per-request execution-tool owner.
It captures native callable names before the model request
and reuses each wrapper across the active and callable views.
`proc_65de` passed active,
inactive-but-originally-callable,
and later-exposed cases.
Later additions remain excluded.
Inactive callable declarations are not added to the main model request.

Native loadout descriptions were also being replaced by unprojected definition text.
`proc_3145` reproduced the lost Codemode return-type guidance.
The wrapper now retains the native projected description;
`proc_8ebd` passed.
The earlier `proc_622a` failure was a test reading `tools` from the normalized stream context.
The corrected test observes the original execution context instead.

Final suite `proc_a91d` passed after original-callable capture and description repair.
It ran 21 program cases,
22 schema mutation cases,
and the shared-queue release,
abort,
and disposal controls.
Document check `proc_659d` passed before this final result note.

The bounded native-program checkpoint is complete.
Independent transcript-based review found no further concrete defect in the actual issued-context path.
It did not independently execute the tests or qualify semantic decisions.
Forged direct calls to internal SDK glue are not a public admission endpoint;
the fixed consumer callback supplies its captured session and tool view.
Do not turn hypothetical trusted-helper misuse into another mandatory capture or hardening queue.

Arbitrary programs and identifier forms outside the existing bounded profile remain unsupported.
The substantive unresolved frontier is established actual clause interpretation/application
and enough decision-specific instruction/operation coverage to exclude outcome-changing alternatives.
The full migration and installed cutover remain incomplete.
Keep the canonical bound evaluator withholding.
No semantic provider trial,
threshold adoption,
or installed activation occurred.

## Continuation decision: no additional live trial

The user declined an additional trial on 2026-10-07.
A stale frozen input is not a reason to repeat a paid experiment.
The user also directed faster implementation without hash locking or unnecessary validation machinery.
Do not add source-freeze manifests,
admission pipelines,
or repeated qualification runs as prerequisites for ordinary code changes.
Use existing evidence and targeted tests.
Retain the successful full-rule interface call and the local native controls at their proven scopes.
Refresh current policy and source inputs,
then verify changed implementation paths deterministically.
The prepared native-live snapshot was never dispatched and is not admitted for use.
Live native end-to-end behavior,
semantic accuracy,
and production readiness remain unproven.

Read-only intake `proc_48d7` found installed Pi SDK 1.0.4
and 86 unavailable paths in the historical native source manifest.
The protected policy now has 207 lexical rule markers and 31,867 bytes;
its text is not merely whitespace-normalized equivalent to the captured policy.
The Node binary still matches its pin.
No SDK was imported,
no credential was read,
and no provider request was made by this intake.
Re-index the current policy and rebase deterministic source checks;
do not restore removed dependencies or reinterpret old receipts as current verification.

## Simplification correction: reuse the original request

The user asked why JSON was being parsed and serialized
and directed the implementation not to overengineer.
The tools-field extraction detour duplicated evidence already retained as `originalRequestBody`
in `contract/integration/action-policy/wire.mjs`.
Decoding and re-encoding that field introduced representation problems without supplying a missing policy premise.
The detour was removed,
including the separate source/binder,
receipt projection,
and raw-span regression.
No raw-span parser was implemented.
The existing request bytes and one-time JSON-format validation remain.

Do not expand the source inventory merely for completeness.
Before adding parsing or another evidence representation,
identify the specific missing requirement in the actual policy consumer
and check whether existing captured data already satisfies it.
The migration remains incomplete;
qualified interpretation and complete-operation policy are the substantive open areas.

The assessment helper's operation-only call was then replaced with the existing original mixed batch
so requested assessment includes both rule meaning and whole-operation relations.
Action `proc_fde4` and native `proc_e5ac` passed.
The bound evaluator remains incomplete;
this is not a claim that native automatic decisions now work.
The next substantive gap is admitted instruction effects and applicability from existing inputs.
Do not introduce an unqualified-estimate-to-approval fallback or re-open that settled behavior.

### Partial supported instruction effects

The existing code resolver now accepts an explicit unresolved effect among its supported instruction effects.
It retains the priority of a known-applicable rule whose effect is unknown,
rather than falling through to a weaker known rule.
Unknown meaning blocks only when it can change the outcome.
Known-inapplicable rules and weaker rules do not defeat an established stronger result.
This partial state does not stand for unclassified prose,
unknown delegation,
or unsupported conditions.
Source admission and complete-operation requirements remain unchanged.

`proc_b90a` reached the expected rejection before implementation.
The preceding `proc_176a` committed the test but did not run it:
its repository-root `//contract` task invocation required monorepo configuration absent in the private repository.
The existing directory-local invocation was used instead.
Implementation `27ad6ac` passed `proc_5a09`,
including 14,400 partial programs checked against 112,896 complete worlds.
Omission `2f409d4` made `proc_af31` return resolved instead of unresolved at the guard-outcome boundary.
The guard is restored in `41d9294`;
`proc_1c88` passed.
Action `proc_d1d1`,
native `proc_304b`,
and rendered document checks `proc_dce0` passed.
Routine kernel checks compare behavior instead of enforcing the historical textual source delta;
the old verification record remains intact.

### Operation-bound typed clause estimates

The experimental `clause_operation_relation` separates affirmative permission,
prohibition,
authorization prerequisite,
no operative effect,
unsupported semantics,
and unknown premises.
It estimates one clause's relationship to the complete original operation,
not source authority or the final action outcome.
Authorization prerequisites cannot be treated as conditional permission or proof that session Allow suffices.
Unknown and unsupported meanings do not automatically fit the partial resolver's supported-effect domain.

The existing selection owner,
response parser,
and assessment wire now retain typed choice estimates and their probability vectors.
The rubric is included once;
exact original request text is reused.
Native `proc_e737` passed current-policy linked controls with 207 clauses and 207 questions per local response.
Request bodies were 204,917 bytes with the host prompt and 209,700 bytes with native defaults.
Normalized local answers were 34,253 bytes.
Independent-source failures retain the existing bounds.
No provider request or semantic qualification occurred.
The default mixed helper and incomplete bound evaluator have not adopted these estimates as facts.
Omission `1c41a06` failed `proc_a4ae` because choice probabilities were not frozen.
Restoration `88f3635` passed `proc_c971`.
Expanded action `proc_3e68` and complete native `proc_5ed5` passed.

### Authorization prerequisites remain constraints

Prerequisite-aware composition is implemented in the existing code-owned consumer.
A higher-tier instruction requiring authorization does not itself override a lower-tier independent prohibition.
The legacy authored `require-approval` effect acts as conditional authorization through the approval adapter;
a bare prerequisite cannot be lowered into it directly.
`authorizationRequirements` now retain applicability,
satisfaction,
and eligible-channel facts separately.
Known denial remains denial;
unknown satisfaction is not absence,
and unsupported channels do not open the UI.
Only an otherwise-permitted operation with fully satisfiable remaining requirements reaches approval.

Naive lowering failed `proc_d724` with approve instead of deny.
The missing composition also failed `proc_ae02` by skipping required UI.
Implementation `7bad409` and interaction controls `fb203a1` passed action `proc_f0d3`
and native `proc_e801`.
Channel-eligibility omission `2e19c00` failed `proc_ff0d`;
the guard is restored and action `proc_e1a6` passed.
These are authored-fact and fixture-approval controls,
not model qualification or genuine human approvals.
The partial-effect truth table proves its bounded algebra,
not semantic lowering from prose.
The native evaluator still withholds until admitted interpretation and complete-operation premises are available.

### Unapplied instruction clarification

A proposed tightening of the existing `RCI` rule in `AGENTS.md` is:

> Before adding capture,
>  parsing,
>  validation,
>  or an owner,
>  inspect existing generators,
>  managers,
>  and retained data.
> Extend their owner only for a specific missing consumer requirement.

This proposal is recorded only;
`AGENTS.md` remains unchanged.
The overlapping Git inspection and index-lock incident also motivates this unapplied `CPN` tightening:

> Commit pathspecs name every new file.
> Serialize Git calls per repository,
> including status,
> and check status after the commit finishes.

## Direct integration progress

Private implementation lives in `contract/integration/` in the retained consumer-contract repository.
The installed plugin is unchanged.
The public relevance hook passed its provider-free current-policy cases in `proc_10ca`.
The bound action review now retains original source and effect evidence through closure.
It refuses caller-supplied completeness and invented permits,
and selects the incumbent original-group approval binder directly.
The combined action-policy suite passed in `proc_c7b7`.
Its real native-read check used SDK 1.0.4 and a disposable file;
request/judgment and approval doubles are not live sessions or genuine human confirmations.

The new `policy-source` integration links the original policy capture to the actual native system-input callback.
It preserves other context and appended inputs rather than deleting them to satisfy a profile.
A bounded system-view adapter checks the original resource,
base,
run,
and final request links.
The initial numeric-priority claim was an overreach and has been removed:
observed system placement does not establish instruction authority.
`proc_6295` passed the initial source-publication tests using actual resource/base/run owners
and the installed prompt builder,
with a test-double final request boundary.
Action-review wiring passed `proc_ad11` and `proc_2de4`.
A fabricated input-owner regression failed in `proc_e1d6`;
private instance authentication fixed it,
and `proc_f52c` passed the original/copy/proxy controls.
The corrected system-view adapter now imports the native renderer directly,
rather than accepting a caller renderer.
Corrected system-view tests passed `proc_4a3d`,
action-policy tests passed `proc_d73e`,
and the ordinary document render passed `proc_5031`.
The request-owner integration test passed in `proc_8d6d` with one local stub terminal send and no provider calls.
It uses the real producer,
observer,
collector,
and budget owner,
while retaining stream/transport/session-reader and judgment-facade doubles.
Instruction authority and priority,
additional governing-source coverage and delegation,
qualified rule interpretation,
and complete-operation coverage remain unresolved.
Neither source identity nor a narrow read fact supplies those premises.
The user accepted [trusted-bootstrap authority registration](../planning/pi-auto-mode-instruction-authority-root.md)
on 2026-10-07.
The existing source owner's private state now retains an optional immutable startup declaration.
Registered tiers and original-request scope are separate from observed wire placement.
A user-tier registration remains user-tier in a system message;
per-action metadata cannot register an unregistered source.
The new authority binding retains the exact original source and request identities.
The ordinary-host registration profile passed `proc_a0b3`,
and action-policy regression passed `proc_61e6`.
Registration now preserves the native host prompt,
normal context files,
and appended instructions instead of requiring a sole system carrier.
`createBoundPolicyReviewer` fixes owner capabilities at bootstrap
and takes only the original response,
obtaining its existing judgment internally.
Its entry-point controls passed `proc_9c85`.
An ordinary installed SDK 1.0.4 session retained the registered source in `proc_292a`,
using an isolated home and local stream/transport doubles with no network requests.
`proc_3a8c` verified judgment start before native argument preparation,
original signal identity,
and simulated expiry within the original five-second budget.

Private `contract/integration/native-batch/` now copies only the SDK factory,
Agent,
and loop modules to add complete-group dispatch.
It uses the incumbent prepared dispatcher and execution-context/cancellation wrappers.
`proc_7fcb` passed 11 native SDK-session cases,
including parallel/sequential ordering,
whole-group blocking,
preparation expiry,
implementation-change retirement,
unsupported nesting,
multiple groups,
and native termination requests.
Positive reads were independently authorized disposable fixtures;
judgment facades and local model-stream/transport doubles remain.
No installed source,
provider trial,
source-hash lock,
or source manifest was introduced.
The unsupported session-runtime replacement exports are intentionally absent from this private factory.

The current copy then added the incumbent manager copy/private-field/root-occurrence mechanisms
and session liveness/disposal-completion mechanisms,
for five copied modules.
`proc_e023` passed the batch matrix and a real-owner composition:
the original judgment constructor,
constructor/root-lifecycle owners,
registered source authority,
and native read-fact owner ran through a current SDK session.
The judgment closed and both members remained unentered because policy premises were incomplete.
Native root reset/restoration did not revive an old lease;
copied leases,
protected manager shadows,
and disposal were also exercised.
This composition no longer uses the judgment facade,
but its model stream and transport remain local doubles.
`createRootSessionFactory` now reuses constructor and factory-cleanup owners as a reusable entry.
`bindNativePolicyRuntime` composes the actual source,
request,
judgment,
review,
and execution owners.
`proc_5be8` passed construction cleanup controls and the incumbent default scope cases;
`proc_4ddb` passed runtime composition and active/duplicate binding controls.
`proc_328f` passed the action-policy regressions after the effect-owner argument change.
Independent review identified missing rollback after interrupted runtime installation
and a provisional session association that could overwrite an existing association before rejection.
`proc_87bc` reproduced retained hooks after interrupted binding;
`proc_3b1b` reproduced loss of the previous session association after rejected reuse.
Both were corrected,
and `proc_fecd` passed the combined suite,
including partial-registration cleanup and retryable detachment with falsy causes preserved.
The current SDK copy now reuses incumbent native base/run prompt custody.
It contains five whole SDK modules plus the extracted native handler emitter.
`bindPolicySessionCollector` retains those publications in the original request snapshot.
The existing suite passed in `proc_e079`,
and source regressions passed in `proc_c172`.
`proc_8791` exposed an owned test-phase error:
native settlement clears the run publication,
so request-time freshness does not survive `prompt()` returning.
The corrected active-phase checks passed in `proc_a6fe`,
including actual handler capture,
late-alias independence,
private-reader shadow resistance,
reset/reload/disposal,
and original request-publication links.
Captured input still does not establish authority or complete instruction coverage.
Optional `customPromptAuthority` now registers explicitly configured host prompt text within the existing source owner.
It requires the original resource/base publication and does not assign a tier to the composite system message,
context files,
appendices,
or handler outputs.
The file registration and host registration remain separate original-request bindings.
Native tests passed in `proc_9463`,
source tests in `proc_109e`,
action tests in `proc_6e61`,
and additional copy/per-action override controls in `proc_2961`.
The original judgment now indexes the selected policy and explicit fixed host prompt text,
even when the latter has no authority registration.
It preserves duplicate codes as distinct source-owned handles,
rejects ambiguous bare-code selection,
and retains unclassified prose per source.
Whole-program and bounded prerequisite wire adapters retain all captured source text.
Wire ordinals are representation data,
not original-occurrence authentication.
`proc_2fe0` passed an original judgment through the whole-program wire with one local relation-response double,
retaining the native signal and original five-second deadline;
its estimates remained unqualified and execution remained withheld.
Source and action regressions passed in `proc_c356` and `proc_bf22`.
No provider request was sent.
`bindRuleAssessmentWire` then added the fixed meaning and operation definitions in one request.
`proc_2371` passed a single original mixed batch through one local response double.
The original owner now also declares and issues that fixed batch within its existing synchronous binding segment;
`proc_7f48` passed that original-owner helper composition with one local relation dispatch,
no provider requests,
and no execution release.
Broader native/delegated source inventory,
qualified interpretation,
provider acceptance and full-policy capacity of the mixed representation,
and complete-operation policy remain unfinished.

`proc_1c65` then reproduced an owned review-lifecycle defect:
a failed fact reader withheld execution but left its prepared judgment open.
The bound reviewer now rejects phase/concurrency misuse before acquiring cleanup responsibility.
On an accepted review failure,
it uses the existing finalizer to close still-open evidence,
then rethrows the original failure unchanged.
If closure also fails,
both occurrences are retained in order.
`proc_0871` passed real-SDK zero/false/undefined/error and changed-dependency cases,
with no tool execution or provider calls.
`proc_979e` passed phase/copy/repeat controls and `proc_1597` passed action regressions.
Evidence-only closure still supplies no policy outcome or permission.
The full native suite passed in `proc_7d3e`.
Native append outputs now enter the original source index and wire without acquiring authority.
Repeated text keeps distinct original resource-frame positions,
not invented raw-file identities.
The current source controls passed in `proc_8684`;
the initial failure in `proc_7df2` was an outdated authored snapshot missing the new append-input link,
corrected without weakening the binding.
The tagged-appendix original mixed-batch test passed in `proc_658a`.
Native context entries then entered the source-owned index without path-derived authority;
native/source/action tests passed in `proc_ecf4`,
`proc_4314`,
and `proc_da14`.
The full current-policy check caught repeated per-clause dependency checks in `proc_03b7`.
Using the original synchronous binding segment passed `proc_18ea`,
with 207 rules,
414 unlinked occurrences,
and 19 checks before fact collection.

The ordinary-host loader now also preserves its configured context callback and emits fresh unchanged entries
from the original selected policy capture.
Original native returned-array/entry/frame checks establish these as carriers of that source,
not proof of the displaced read.
Unmatched entries stay independent,
including equal text at another path.
Source publication and failure controls passed in `proc_b756`.
`proc_9e95` passed the current-policy comparison:
207 canonical clauses and 414 mixed questions with the explicit carrier link;
414 clauses and no dispatch with an additional independent copy.
The negative fixture explicitly ends after its mixed-bound check.
A later enlarged-ancestry request exceeded the existing private collection byte bound in `proc_052e`;
that bound was not raised and later-request support is not claimed.
A separate owned hook defect masked native terminal errors as missing producer observations in `proc_edda`.
Reordering the non-executing terminal check fixed it in `proc_f814`;
unowned executable responses still require authentication.
The full carrier suite passed in `proc_690f`.

The incumbent run owner now captures forced prompt output and generic prompt-guideline inputs
as original run-publication sources.
Empty forced text is retained,
and repeated guideline positions remain distinct.
Neither system placement nor capture supplies an authority tier or proves rendered visibility.
Late callback aliases do not alter published inputs.
The original judgment-to-mixed-wire case still makes one local relation dispatch and releases no tools.
Native tests passed in `proc_fbd4`,
source tests in `proc_2341`,
and action tests in `proc_7e0e`.
Native resource-system output is now retained when not already represented by a fixed host input
or the selected-policy system carrier.
Discovered `SYSTEM.md` and replacement callbacks do not establish tier or raw-file provenance.
`proc_5351` passed discovered and empty system text through the original mixed wire,
forced rendered-head isolation,
and an in-place empty forced override.
`proc_013e` passed reload,
replacement,
empty/absent,
and existing owner rejection of invalid replacement types.
Tool-guideline entries now retain their native map key and list position,
including unselected tools and repeated text.
Those positions survive the mixed and legacy wires as data,
not authority or applicability.
Native `proc_1eb2`,
source `proc_34c5`,
and action `proc_bea7` passed.
The full current-policy local control still carries 207 clauses and 414 questions.
Tool-snippet and section-map inputs now retain original field/key positions too,
including empty section entries and repeated text under different keys.
Native `proc_cdd9`,
source `proc_0537`,
and action `proc_4646` passed.
The expanded original-owner fixture carries 24 questions,
or 22 with an empty discovered-system source,
through one local mixed response.
The full current-policy control remains 207 clauses and 414 questions.
Scalar and context run inputs now preserve unchanged values through explicit original-base republication.
Field and position select the original source link;
no byte-keyed source search or append-subset decomposition occurs.
Changed or added values remain independent unregistered run sources.
`proc_a309` passed the complete native suite,
including present/absent system inputs,
empty values,
removals,
reordering,
late aliases,
and copied-snapshot rejection.
The original replacement-input batch carries 30 questions through one local response.
Source `proc_47a1` and action `proc_cfcf` passed.
The first matrix failure was a fixture assumption about configured empty append strings;
its native override-boundary correction passed `proc_26ba`
and is recorded in `doc/troubleshooting/pi-sdk-staging.md`.
The full-policy control remains 207 clauses and 414 questions.
Skill descriptions now retain their original positions and serialized native metadata,
including disabled or empty entries.
Metadata scope and paths do not supply authority or authenticate skill files.
Native `proc_8037`,
source `proc_db99`,
and action `proc_c1af` passed.
The initial failure was an owned JSON-boundary assertion,
corrected without loosening source validation.
Handler output now retains original emitter occurrences through current-run ancestry.
String content is indexed;
text-block arrays are retained as JSON and explicitly unclassified,
without guessing clause boundaries.
Labels,
display flags,
and user-role transport do not grant authority or human origin.
Native `proc_9d95` passed string and structured content through the original mixed wire,
opaque-details exclusion,
late aliases,
and another next-turn ancestry level.
Source `proc_d3ea` and action `proc_827f` passed.
This does not establish provenance of older persisted messages after a new prompt resets ancestry.
The private structured-content guard omission failed its regression in `proc_1d20`;
the guard is restored.
Native prompt construction now captures built-in instruction fragments through the original run owner.
The private copy includes seven SDK modules,
with the native prompt builder and skill formatter retained in the new builder module.
Rendered-section parity passed `proc_bc87`.
Native `proc_997c`,
source `proc_6abc`,
and action `proc_2e52` passed.
The native path requires its exact construction input and render occurrence;
fragments remain unregistered and do not prove final visibility.
`proc_a631` confirmed the linked default prompt supports 207 clauses and 414 questions
at 34,332 rendered bytes.
Only the independent-copy default case exceeded the unchanged 65,536-byte source cap,
at 66,405 bytes,
before judgment creation.
The failure must not be generalized to the linked default case.
The render-requirement omission failed its committed regression in `proc_8931`;
the requirement was restored in private `df8e07d`,
and `proc_0a4d` passed.
The complete native suite `proc_259b` passed the expanded rendering parity,
original-owner controls,
and both full-policy prompt variants.
Document rendering passed in `proc_9dfa`.
Remaining source-index gaps include general persisted-session messages,
later-read skill-file contents,
tool-schema descriptions,
and broader delegated inputs.
Complete governing-source coverage,
delegation,
qualified interpretation,
complete-operation policy,
and broader nested/runtime support remain unfinished.
Continue other authorized implementation without a new paid trial or source-locking pipeline.

## Current direction: incremental rule relevance

The user selected policy-change-time relevance classification
and clarified that changed rules go in one API call,
with independent questions inside that batch.
Do not reopen the one-question-versus-one-call discussion.
Use the [incremental cache contract](../planning/pi-auto-mode-batched-assessment.md#incremental-rule-relevance-cache-direction).

The private cache now extends the incumbent policy selector and exact Markdown index.
`proc_7d12` passed 12 control groups.
Its first control used the actual protected policy read-only:
205 indexed rules in one synthetic classifier batch,
then no classifier call on warm refresh or on a new owner loading the persisted entries.
Single-rule edits and additions selected only those rules;
deletion required no new classification.
Heading changes invalidated affected descendants,
and changed unclassified context widened invalidation.

Controls also covered worktree overlays,
namespace/profile changes,
same-source single-flight refresh,
stale asynchronous results,
invalid replies,
cancellation,
diagnostic-sink failure,
malformed cache entries,
and duplicate rule IDs rejected by the incumbent indexer.
These are cache mechanics,
not semantic relevance qualification.
All indexed rules remain candidates;
no low score is admitted as an exclusion.

Git namespace controls passed in `proc_481b` on Git 2.55.0,
using disposable repositories and a linked worktree.
The original SSH-hostname normalization failure `proc_cd11` remains preserved.
No network fetch or production Git mutation belongs in namespace lookup.
The fixed transport passed `proc_d04a`;
native resource ownership and drain passed `proc_2dbb` and exact omission control `proc_b05b`.

The private hook refreshes lazily at `before_provider_request`,
where the native active-run cancellation signal is available.
Read-only reconciliation `proc_e53e` verified the completed `proc_6478` worker:
one SDK session,
two runs,
four local main requests,
question counts `[205, 0, 1, 0]`,
and four native fixture reads.
The edited-rule batch contained only `LN7`.
Both tool judgments retained zero semantic attempts;
cache preprocessing did not reopen their budgets.
The worker's copied terminal summary incorrectly said five sessions and ten requests.
Its actual artifacts establish one and four;
no SDK replay was used to correct that reporting error.

The first real relevance request is now complete.
`proc_f465` verified one 255,046-byte Jev request,
all 205 raw answers,
and zero additional calls for same-owner warm and restarted-owner persistent reads.
Returned usage was 59,382 input tokens and 3,729 output tokens.
At the retained 42 nanoUSD input-token rate,
modeled maintenance cost was US$0.002494044.
This is neither an invoice nor a representative per-action average.
At completion of the cold relevance canary,
the cumulative recorded semantic subtotal reached US$0.055516382.
No SDK AgentSession or native tool action was created by this live cache canary.

The outer metadata checker separately emitted a logger verification timeout after reporting completion.
The native worker and its immediate controller had empty stderr.
`proc_6308` reproduced the pending-file-sink plus blocking-child mechanism
and verified awaiting the incumbent logger's `flush()` before blocking.
The paid canary was not replayed.
See [staging diagnostics](../troubleshooting/pi-sdk-staging.md#relevance-canary-and-native-hook-diagnostics).

The retained-score diagnostic `proc_2dfd` used a post-collection authored subset,
not a held-out accuracy study.
`LN7` scored `0.53` and remains included.
The current guard must not discard uncertain rules merely because they fail a high positive threshold.

Original action-time candidate selection also passed its native consumer,
`proc_1bd1`:
all 205 indexed handles stayed owned by the original judgment,
copied handles were refused,
and selection after evidence closure was refused.
The fixture used one session,
two local main requests,
one fake guard request,
and two independently authorized reads.
`proc_fa56` verified foreign/copy rejection,
source invalidation that restoration cannot revive,
and the exact enumeration-freshness omission witness.
An empty index still does not establish complete governing-source coverage.

The original claim-to-answer handoff is now rebased onto the candidate-aware judgment.
`proc_d959` verified canonical evidence capture,
equal estimates remaining distinct across claims,
copied/foreign/cross-claim rejection,
and malformed/discarded/unissued refusal.
The exact canonical-identity omission failed its authored assertion.

Native checking exposed distinct incidents.
`proc_b5b2` called evidence validation after the next request generation had retired the original;
that rejection was correct.
The active-request control `proc_f221` then found that restoring a changed dependency
revived evidence eligibility.
`proc_7bc6` verified the repair:
the original judgment permanently retires evidence eligibility after a genuine dependency failure,
including other claims sharing those dependencies.
Canonical identity is checked first,
so a rejected copied record does not poison genuine evidence.
Successful evidence closure still permits validation after its inference clock expires.
The repaired fixture executed no tools and made no provider request.
`proc_f9b4` reconciled the failed artifacts read-only.

Next:
connect original candidate-bound evidence to rule-based action assessment and code-owned policy premises.
Do not promote retained estimates,
candidate enumeration,
or source designation into missing policy premises.
Semantic relevance qualification and exclusion-cutoff adoption remain separate.
The accepted direction does not authorize production cutoff adoption,
installed-plugin cutover,
or cached action permission.

## Current whole-program rule-assessment boundary

The private experimental predicate `operation_violates_rule`
binds each original indexed rule to the complete original parent program and prepared group.
Generic member selectors cannot create another sampling opportunity for the same program/rule pair.
The fixed question preserves the original rule text,
heading context,
complete policy,
and original main request.
A negative binary score merges inapplicability and compliance;
it cannot establish permission or instruction-free fallback.
No additional code-effect facts are bound by this initial wire,
and it explicitly marks effect-fact coverage incomplete.

`proc_aaa5` passed 14 synthetic wire modes with all 205 policy rules.
The authored complete-program request was 452,279 bytes.
The first native attempt `proc_e496` stopped during evidence capture;
its wrapper did not retain the underlying cause.
The separate instrumented attempt `proc_7e9b` recorded
`DependencyDeadlineError: Original dependency assessment deadline expired`
at capture index 45,
with 26,485 dependency-trace callbacks.
Do not retrospectively assign that cause to the earlier unretained error.

The source owner now performs full original dependency checks around synchronous binding segments,
keeping per-rule identity and budget checks inside them.
Freshness is not retained across transport awaits or separate calls.
An exact issued batch owns its canonical captured evidence array;
copied and proxied arrays cannot authenticate by their contents.
`proc_ca71` completed all 205 fake estimates and original evidence records under the unchanged five-second budget:
one SDK session,
two local main requests,
one fake guard attempt,
628 dependency-trace callbacks,
and zero tool executions.
Its native serialized request was 454,012 bytes.
These counts are scoped observations,
not a wall-time speedup or real-provider latency claim.

`proc_b707` verified postcheck source change,
source change across transport await,
clock expiry,
reentry rejection,
and the exact postcheck omission witness.
`proc_94d8` checked canonical batch identity and inert foreign/proxy inputs.
`proc_8424` then verified centralized original dependency retirement:
direct,
per-claim,
and batch failures remain retired after source restoration,
while rejected closed-phase collection calls do not poison valid captured evidence.
The original failure cause remains stable.

Actual native cancellation at the postcheck was retained from `proc_75d4`.
The await-cancellation fixtures initially expected the wrong top-level error messages.
`proc_7d8a` checked the transport/budget aggregate,
the original observer's cancellation cause,
the stable retirement cause,
and unentered members on the corrected graph.
No provider call or fixture tool execution occurred in these native qualification controls.

The separately frozen copied-input interface request was then sent once.
`proc_48e7` reconciled its terminal HTTP 400 response,
`max_tokens_exceeded`,
with no scores or usage returned.
The 454,012-byte namespace is consumed and was not retried.
After that first failure,
known modeled semantic spend was US$0.055516382,
plus that failed request with unquantified billing.

Current TypeSafe documentation specifies both 64k tokens for state plus all questions
and 32k for state plus the longest question.
The response did not identify which bound failed or return an exact token count.
The client now has a separately versioned encoding that removes only duplicated native binding bookkeeping
from model state.
`proc_ba2b` measured 315,353 bytes,
with every question,
criterion,
selected rule,
policy byte,
and program/request field unchanged.
The separate 315,353-byte trial also returned HTTP 400 with `max_tokens_exceeded`.
`proc_e51c` reconciled that consumed namespace without replay.
Neither rejected representation returned scores or usage;
there are now two unquantified failed-request charges alongside the known subtotal.

The next local revision keeps the exact question sentence,
criteria,
and selected rule inside every head,
and moves the remaining 373-byte instruction guidance verbatim into shared state.
Every question explicitly references that guidance.
`proc_0c8c` measured 256,710 bytes and verified reconstruction of the full original instruction string.
It did not establish token fit,
provider acceptance,
or equivalent model interpretation.
The separately frozen shared-guidance trial then succeeded once.
`proc_ca7b` verified all 205 estimates,
62,053 input and 3,729 output tokens,
and 1,001.814279 ms including sample preparation within the five-second budget.
Modeled cost was US$0.002606226;
the known semantic subtotal is US$0.058122608 plus the two unquantified failed requests.
No SDK session or tool execution occurred in this copied-input trial.
Transport acceptance does not establish accuracy,
calibration,
or workload-average cost.

`proc_64bf` passed synthetic transport controls,
`proc_6062` passed the current native owner graph with fake estimates,
and `proc_7d8d` passed actual frozen-worker launcher controls.
`proc_eda4` additionally reconstructed the full original body,
with positive controls for a missing instruction sibling and changed criteria.
The independent review's generic bytes-per-token prediction was not adopted.

Prospective native-input qualification also passed in `proc_4a49`:
a fresh original judgment produced exactly the predicted 256,710-byte body
and captured 205 fake estimates without executing tools.
The first preparation stopped before worker creation because its planner omitted
working-directory text embedded in the complete main request.
`proc_cd8e` identified that field;
the corrected sibling preserves it through a scoped rebase.
`proc_6418` rejected a prospective-body mismatch before fake network dispatch,
and `proc_8812` retained the exact guard-omission witness.
These remain local mechanical controls,
not a live native guard call.

Next:
continue private implementation with current policy and source inputs,
using deterministic checks rather than another live qualification trial.
Keep exact input checks and the original preparation-inclusive deadline.
Continue semantic and source/policy work without treating existing transport evidence as final permission.
Do not truncate required inputs,
assume moving guidance preserves calibration,
reopen the original judgment,
or replay rejected bodies.
No semantic cutoff,
source-authority admission,
cache exclusion,
or production change follows.

## Current approval UI correction

Use the working auto-mode Pi TUI,
not the separate external-editor experiment.
The user explicitly corrected that integration choice.
`package/pi-plugin/auto-mode/src/ask-user.ts` owns the existing `askUser` interaction:
`ctx.ui.select()` presents Allow,
Deny,
and Stop.
`evaluate.ts` already routes its ask branch there.
The notification,
session-verdict recording,
Stop abort,
and no-UI handling are incumbent behavior to preserve.

The external-editor interaction `proc_60f9` failed.
Read-only diagnostic `proc_1fff` found a captured response rejected at the group-scope-binding boundary;
the fixture then masked that failure by reading `approval.status` from an absent decision.
No approval was established.
Do not replay that interaction,
promote its raw answer,
or build more editor forms as a migration prerequisite.
Those artifacts remain historical evidence only.

The private adapter now reuses the actual existing `askUser` export.
`proc_91cb` exercised its incumbent choice,
notification,
Stop,
and headless behavior.
`proc_c8c5` confirmed that the ordinary native verdict append preserves original-judgment currentness
in the admitted profile;
no persistence bypass was needed.

`proc_8452` passed controlled native Allow,
Deny,
Stop,
and source-change cases:
four sessions,
seven local main-model requests,
four canned assessments,
and native read counts `[2, 0, 0, 0]`.
The adapter forwards original cancellation to the existing selector
and checks currentness before releasing an owned Allow.
It does not build a new UI.

`proc_c4c9` separately showed that restoring source after a failed Allow does not revive it.
Removing the phase guard made the exact assertion fail.
Headless blocking and late Allow after cancellation also passed their named controls.
Retained diagnostics are not release eligibility.
These controls used synthetic UI backends,
not genuine human approval.
General prompt-size and actual viewport qualification remain open.

Continue policy and semantic integration using this incumbent path.
No production cutover or threshold adoption is authorized by the correction.

Proposed guidance clarification,
not applied to protected `AGENTS.md`:
expand the existing owner-discovery rule to cover UI and workflow implementations,
so incumbent discovery precedes replacement work.

## Current batched instruction-meaning integration

The private `batched-prerequisite-wire` adapter now carries distinct original clause claims
in one request,
using fixed pre-authored wire slots.
Slots correlate answers;
they are not new semantic definition IDs or permission records.
The exact complete policy and original parent,
prepared group,
and main request remain in the assessment.

The fixed `states_approval_prerequisite` predicate asks about clause meaning,
not member applicability.
The original judgment now shares that claim across prepared-member selectors.
Generic member claims cannot issue this predicate,
and other predicates retain member-specific bindings.
This avoids resampling the same clause meaning through another member or wire slot.

`proc_5b5e` passed 32 synthetic transport controls,
including swapped clauses,
reversed response-key order,
malformed answers,
original-deadline expiry,
cancellation,
and credential redaction.
`proc_0eb5` then passed one native SDK session with two local main requests,
one fake guard attempt,
two original clause estimates,
and two independently authorized fixture reads.
Estimates `0.37` and `0.61` were canned controls,
not semantic evidence or release decisions.

The exact shared-map omission produced the intended
`AssertionError: Missing expected exception.`
The first verifier `proc_60d2` incorrectly expected extension callbacks for admission-blocked calls.
Its SDK worker exited successfully;
read-only reconciliation `proc_cc9f` passed by checking persisted results without replaying that worker.
See [the verification-boundary incident](../troubleshooting/pi-sdk-staging.md#owned-admission-verifier-confused-extension-callbacks-with-persisted-results).

The fixed live encoding diagnostic is now complete:
`proc_51ef` verified four real requests,
eight estimates,
and no retries.
Each request retained 75,060 bytes of archived native input,
including the full current policy.
Forward and reversed clause orders each had byte-identical repeats.
`LN7` returned `0.97` throughout.
`HRM` returned `0.09`,
`0.10`,
`0.09`,
and `0.09`.
The observed `0.01` spread on repeated reverse-order input prevents an order-effect claim.
No cutoff is adopted.

Modeled spend was US$0.003214512 total,
US$0.000803628 mean,
at the refreshed public input rate.
The recorded semantic-study subtotal is US$0.053022338.
This is returned-usage accounting,
not invoice or representative workload evidence.
The research replay created no SDK sessions,
tool executions,
or current grants;
it did not reopen the archived native judgment.
The namespace is consumed.

Managed launch `proc_1077` stopped at its credential check before creating a live namespace.
The existing Bash credential environment then completed the first actual dispatch.
Only the credential's variable name was exported;
its value was not printed or passed as an argument.

Return to policy integration,
not more prerequisite-only sampling.
Applicability,
authority,
full policy coverage,
current grants,
and production adoption remain separate.
Keep using the incumbent Pi approval TUI.

## Settled design and authorization

- Models estimate narrow semantic axioms;
  code owns facts,
  instruction authority,
  policy,
  and approve/deny/ask decisions.
- All question wordings are pre-baked.
  Calls two and three adaptively select relevant questions from that library;
  they do not invent or paraphrase wording.
- Questions within a Drex call are parallel and do not affect one another,
  as clarified by the user.
- Parallel tool calls must receive one combined judgment before any member executes.
  Interacting effects belong in that judgment.
  Preserve the parent program and dataflow;
  isolated concrete arguments can omit effects established by preceding operations.
  Runtime value arrival or a worker queue drain does not itself create a new judgment or budget.
  The blanket nested-overlap A/B question is withdrawn,
  not an adopted policy.
- At most three client decision-model attempts per combined judgment,
  with earliest possible termination.
  The preparation-inclusive five-second total deadline remains.
  The existing one automatic transport retry consumes the same slots;
  Gateway-internal retries remain separately accounted.
- Target mean cost is no greater than US$0.001 per combined judgment.
  Tell the user if measurements indicate the target cannot be met without sacrificing required evidence or quality.
- Jev through LLM Gateway remains an accepted qualification direction.
  Task-relevant private inputs are already authorized there.
- The user separately authorized task-relevant private Drex inputs,
  including previous real Pi transcripts.
  Do not repeat either provider-consent question.
  Keep raw transcripts out of Git and omit credentials or irrelevant sensitive details.
- Production adoption,
  thresholds,
  cutover,
  real grants,
  and represented actions remain unauthorized.

The [batching contract](../planning/pi-auto-mode-batched-assessment.md) is the focused design reference.
The completed qualification phases pinned `AGENTS.md` at SHA-256
`1f5c31c969c5a18aa994465baf7afe25f00a7a9ab11d6190bcab56746732bd4b`.
Concurrent commit `6adccddd80c7717904daad6d0529131e814e06f1`,
dated 2026-10-02,
added the historical-identifier annotation rule `WR6`.
The freshly read current policy hashes to
`15890c665cdb1c054f8c56c2f212cb75297292b812dc0a2e8065e21156483c39`.
This migration did not edit or revert the protected policy.
`proc_75be` stopped before document rendering because its historical policy pin no longer matched.
A new document-only verification phase uses the reviewed current bytes;
it does not qualify current guard semantics or reinterpret earlier experiments.

## Native-manager qualification completed in finite scope

`proc_3b91` exited 0 for the frozen private SDK 0.87.1 manager phase.
The phase accepted three success outcomes and nine expected rejections.
It observed 12 manager creations,
12 message appends,
and 130 captured getter returns.
An additional 11 constructor-owner ID reads are source-derived,
not separately instrumented native returns.

Post-exit checks established:

- All 46 current source bodies equal the same independently reviewed original strings.
- Prepared and consumed input match byte-for-byte at 349,933 bytes.
- Entry and worker stdout equal the complete expected projection.
- Worker stderr is empty;
  entry stderr contains only the expected Mise command echo.
- All ordered completed-case prefixes match.
- Runtime,
  policy,
  manifest,
  file-descriptor identity,
  stream modes,
  and scoped tracked Git checks pass.
- Fixture inventory is 24 directories,
  16 regular files,
  and no symlinks.

No model,
`AgentSession`,
genuine original,
current permission,
grant,
or represented action was involved.
`actualSDKRootAdapterQualified` remains false:
this is manager mechanics,
not actual agent/session/resource-loader producer integration.
The phase is consumed;
never rerun its entry gate or delete its markers.

Private evidence:
`contract/lifecycle/native-manager-dependency-controls/result-disposition.json`
in `~/temp/agent/auto-mode-consumer-contract.mDLkyNoP`.
The document-tracked milestone formerly numbered 95 is complete in that finite scope.

## Drex 1.5 recheck and cost sample

Authenticated metadata lists `drex-v1.5`.
Current documentation raises the state limit to 131,072 tokens
and says long inputs no longer fall back to 1.0.
The [Drex recheck](../planning/pi-auto-mode-drex-1-5-recheck.md) retains primary sources and contract inconsistencies.

One frozen full-policy call returned exact model `drex-v1.5`:

- Six pre-baked questions,
  no retries.
- 8,664 reported input tokens.
- US$0.0004332 modeled cost at the published 1.5 input rate.
- `873.6587979999999` ms on the preparation-through-freshness clock.
- At 80/20:
  four correct,
  zero wrong,
  two unresolved.
- At 90/10 and 95/05:
  three correct,
  zero wrong,
  three unresolved.

This is below the cost target for one observed batch,
not a verified invoice rate,
workload-average guarantee,
semantic qualification,
or threshold adoption.
The explicitly requested read and `README.md` transfer remained unresolved.
The canary is consumed and must not be repeated to obtain more favorable scores.
New private-input authorization does not rewrite its historical public/synthetic scope.

## Composed-effect wording experiments

The user directed trying different wording on cases that are answered poorly.
This permits controlled diagnostic prompt development,
not erasing failures or inventing production questions during a decision.

Five synthetic programs held the invoked script path fixed while changing written source,
which file was written,
or write/execution order.
A read-only control separated protected reads from transfers.
Inert reference execution and state-leak checks passed before provider calls.
Every request included the full parent program,
relevant initial files,
primitive definitions,
and current `AGENTS.md` bytes.
Local expected labels and descriptive case IDs were excluded.

Three pre-authored wording versions each made one request per case,
with no retries.
At 90/10,
initial wording produced nine correct,
five wrong,
and one unresolved scalar;
invocation-time wording produced eight correct,
five wrong,
and two unresolved;
short direct wording produced nine correct,
five wrong,
and one unresolved.
The unused-file and write-after-execution cases remained wrong.
No version qualifies this composed-effect representation.
These are semantic-estimate outcomes,
not final-policy approval measurements or proof of an internal model mechanism.

Modeled means per version were US$0.00043813,
US$0.00044073,
and US$0.00043428.
Including all three versions gives US$0.00131314 per case in development spend.
That is not a measured production workload mean or a per-case spending violation.
The user clarified that US$0.001 is an average target;
individual hard judgments may cost more.
The live three-call ceiling limits one guard decision,
not offline experimentation.
The three variants remain historical completed runs,
not a prohibition on fresh research.

The user also directed testing more than wording.
Next work must compare different axiom decompositions,
input representations,
and suitable question types,
while retaining full composed context and code-owned decisions.
Measure unchanged-input variability before claiming a design improves scores.
See the [Drex recheck](../planning/pi-auto-mode-drex-1-5-recheck.md#composed-program-effects-and-wording-development).

## Factorized diagnostic result

The subsequent diagnostic tested source selection separately from hypothetical source effects,
not just alternative wording.
Ten requests covered two predetermined passes over the five known programs,
with zero retries.
The caller exited 0;
offline reconciliation `proc_01ca` exited 0 with all 41 records and private streams verified.

At 90/10,
all 60 hypothetical effect estimates matched their references.
Source selection was wrong four times:
both passes selected the written source for the unrelated-file and write-after-execution cases.
Direct whole-program estimates remained 18 correct,
10 wrong,
and two unresolved.
This locates observed task errors,
not an internal mechanism or general model capability.
Source-occurrence selection remains code-owned when mechanically established;
this diagnostic did not adopt model authority over it.

Mean modeled cost was US$0.00047233 per one-call diagnostic request.
Both passes returned identical estimates;
uncached determinism and calibration are not established.
The private evidence directory is `contract/research/drex-composed-methods/`.
The ordered-operation comparison subsequently completed 10 requests with zero retries;
`proc_d04e` reconciled all 41 records and exited 0.
Only the ordered input view changed;
the admitted runtime and questions were reused.
Source-selection labels were correct eight times instead of six.
The unrelated-file trap selected the initial source correctly;
the write-after-execution trap remained wrong,
with only 0.5344 probability on the selected wrong occurrence.

At 90/10,
ordered-input direct effects were 22 correct,
four wrong,
and four unresolved;
hypothetical effects were 58 correct,
zero wrong,
and two unresolved.
Mean modeled cost was US$0.00047443 per one-call diagnostic request.
Evidence is in `contract/research/drex-composed-ordered/`.
The offline binding replay then passed as `proc_7856`:
10 source-occurrence bindings matched references,
nine invalid-profile cases were rejected,
and deliberately faulty write/order/path selectors were detected.
Selecting the corresponding retained model heads produced 30 correct effects,
zero wrong,
and zero unresolved at all recorded diagnostic bands,
with no new provider calls.
This covers only the declared literal virtual-file profile and inspected bodies,
not arbitrary JavaScript,
real-file identity,
fresh selected-only inference,
or a production guard.
Private evidence is `contract/research/composed-binding-replay/`.
Read-only inventory completed as `program-fact-owners`.
The user removed the subagent extension afterward;
continue directly without that extension.
The installed `yuku-parser` API is already used by `package/git-policy/cli/src/trust/mjs-validator.ts`.
The parser-backed literal-profile fact producer subsequently passed as `proc_ce56`.
It derives operation records from complete source using installed `yuku-parser` 0.14.0,
retains exact source/literal spans,
and binds exactly one invocation to its last preceding virtual write or initial occurrence.
Child source remains opaque and has no whitelist.

Revised controls passed the five originals,
ten literal-boundary cases,
and seven syntax/version variants;
40 invalid-source and 12 invalid-initial-input cases were rejected.
Disposable extra-argument,
path-binding,
and coverage mutations failed their checks while intact counterparts passed.
The parser-derived replay selected 30 correct retained effects at all diagnostic bands,
with zero wrong or unresolved and no new inference.

Evidence is in `contract/research/composed-literal-facts/`.
The result depends on caller-owned virtual primitives,
successful prior writes,
initial bytes,
and no external mutation.
It is not real-file identity,
host integration,
fresh selected-only inference,
held-out qualification,
or production permission.
Next work must establish those consumer/environment bindings before using these facts in an actual guard.
No production adoption or threshold choice follows from these results.

## Fresh inference using parser-derived input

`contract/research/drex-composed-bound/` completed 10 requests with zero retries.
Only a code-derived invocation fact was added to the ordered study's state;
the same questions and full parent/policy context remained.
Parsing,
fact construction,
and parser/native freshness checks were inside each request's assessment clock.
The supplied source-choice fact is excluded from model accuracy claims.

The first verifier failed on its handling of absolute parser paths,
not on an inference error.
`proc_8fa2` completed corrected offline reconciliation with exit 0;
all 41 records and complete streams matched,
and no paid inference was replayed.
At 90/10,
direct effects were 26 correct,
zero wrong,
and four unresolved.
The selected conditional effects were 30 correct,
zero wrong,
and zero unresolved at all diagnostic bands.
Mean modeled cost was US$0.00048863 per request.
These remain known development cases and conditional virtual facts,
not held-out or production qualification.

The relevant-only follow-up then completed ten fresh requests with three effect questions each.
The full model-visible state was unchanged;
source-choice and unused-source questions were removed.
`proc_9369` reconciled all 41 records and exited 0.
At 90/10,
all 30 effects were correct with zero wrong or unresolved.
At 95/05,
28 were correct and two unresolved,
with zero wrong.
Mean modeled cost was US$0.00046188 per request.
Evidence is `contract/research/drex-composed-selected/`.
No threshold or operational mean is established by these known cases.

The broader child-semantics phase then completed ten requests with the same questions and runtime.
Inert reference execution `proc_b7fd` and offline reconciliation `proc_07e1` exited 0.
The original reserved bank remained unopened.
At 90/10,
20 estimates were correct,
six wrong,
and four unresolved.
An uncalled function was treated as executed;
a protected read followed by an unrelated upload payload was treated as protected transfer.
The disabled branch and computed protected path/URL cases passed.
Mean modeled cost was US$0.00047017 per request.
Evidence is `contract/research/composed-child-semantics/`.

The original-case successes do not qualify this broader profile.
The call-site decomposition subsequently completed ten requests with five or seven questions per case.
`proc_37fd` passed inventory/claim controls,
`proc_a0bb` passed the changed runner's fault controls,
and `proc_dff7` completed offline reconciliation with exit 0.
Actual-observable heads at 90/10 were 44 correct,
four wrong,
and two unresolved;
12 counterfactual heads were reported separately and all matched their scoped references.
Matched-site effects were 24 correct,
four wrong,
and two unresolved.
The uncalled function still had incorrect reachability estimates;
the unrelated constant upload payload remained unresolved.
Mean modeled cost was US$0.00051826 per request.
Evidence is `contract/research/composed-callsite-diagnostic/`.

The same-input Jev/Gateway comparison then completed ten requests with zero client retries.
Fresh metadata and the already settled route authorization were used;
no historical Jev phase was replayed.
`proc_955d` reconciled all 41 records and exited 0.
At 90/10,
Jev composed effects were 18 correct,
zero wrong,
and 12 unresolved,
compared with Drex's 24 correct,
four wrong,
and two unresolved.
Observable and counterfactual head counts remain separate in the receipt.

Modeled returned-input usage averaged US$0.0004385304 per request.
Unknown Gateway-internal attempts/charges and invoices remain separate;
this is not an operational workload mean or weight attestation.
Four-second research gaps were outside each five-second assessment clock.
Evidence is `contract/research/jev-callsite-recheck/`.
No provider or threshold adoption follows from this finite comparison.

Next resume the unfinished instruction-source/SDK consumer bridge,
keeping current permission unavailable until its new SDK-linked human original exists.
Preserve all failed results and avoid threshold changes or probability multiplication.
No subagent extension is used.

## Parallel-tool bug and private prototype

The actual built extension reproduced two judgments for a parallel two-call group.
The callback-only red test was `proc_20ac`.
The real Pi 1.0.0 scheduler red test was `proc_2172`.
Both had injected evaluators and no provider calls.

The private shared-admission prototype passed `proc_46b7`:
11 real-scheduler cases plus duplicate-ID rejection.
It demonstrates one assessment before member execution,
combined-effect rejection,
and zero execution for group denial,
changed input,
cancellation,
stale context,
or assessment failure.

Prepared-input dispatch passed 16 controls in `proc_0340`,
including native preparation,
extension transformations,
after hooks,
updates,
and execution-entry staleness.
Private Pi 1.0.0 host copies passed nine controls in `proc_669f`
and 13 controls in `proc_75fb`.
The latter keeps one admission for a declared group even when member execution is serialized.
No installed Pi source was edited.

`proc_1819` then passed two real SDK session controls using the first private host/Agent copy
and native `AgentSession`,
`SessionManager`,
`DefaultResourceLoader`,
and extension hooks.
Each two-tool batch received one assessment after input transformations.
The allow case executed both inert tools;
the deny case executed neither.
Both own sessions completed,
reached idle,
and freshly persisted their final and tool-result entries.
Extension contexts returned the exact supplied manager;
the session retained the supplied loader and its prompt marker.
No remote model,
genuine human input,
current permission,
or represented action was involved.
The historical SDK-session ledger is not reconciled by these two new completions.

Production extension integration is unfinished.
Nested calls need an explicit complete group boundary;
a shared parent ID or timing window is not enough.
Final effective inputs and later argument mutations have finite prepared-dispatch controls,
not arbitrary-tool qualification.
Subsequent asynchronous controls found a preparation-order defect in the private helper.
`proc_66a7` exited 1 at the expected assertion;
`proc_700d` exited 0 for nine controls after separating preparation and execution turns.
Removing the execution wait triggered the same overlap assertion,
proving the control could detect its absence.
Ordinary native execution errors preserved continuation;
staleness and cancellation while waiting prevented the next execution.

`proc_1505` then passed four new native SDK sessions with the corrected frozen host/helper:
parallel preparation,
global serial mode,
member-selected serial mode,
and serial denial.
Each group received one assessment after both transformations.
The serial cases waited for the first asynchronous result hook;
denial executed nothing.
All newly persisted entries matched their in-memory JSON values.
These are four additional own completions,
not historical-ledger reconciliation or complete native event/cancellation parity.
Private `contract/diagnostic/parallel-tool-batch/integration-frontier.json`
is the detailed result and next-work receipt.
Do not implement a barrier that waits inside the first per-call hook for sibling hooks:
the current host prepares those hooks serially before parallel execution.

See the [parallel batch plan](../planning/pi-auto-mode-parallel-batch.md).

## Request-stage instruction-source controls

`contract/collector/request-stage-controls/` completed as `proc_74bc`,
exit 0:
six new scripted Pi 1.0.0 `AgentSession` completions,
zero external model requests,
and no current permission or human witness.
Full private worker streams,
exact callback order,
manager/loader identity,
JSON-value persistence,
and cleanup were checked.

The current getter missed request-local context changes and final payload-only additions.
Persisted entries retained only the base marker,
not forced,
request-local,
or native-payload markers.
The ordinary collector remained fresh on its observations while retaining all three coverage gaps.
The omitted-payload-hook control was rejected by the exact final-payload validator;
a disposable removal of that validator caused its consumer test to fail.

The built-in serializer follow-up completed as `proc_3c50`,
exit 0,
with seven new SDK completions and captured `openai-completions` requests.
It delegated through the stock SDK/ModelRuntime/OpenAI 7.19.0 client,
used injected fetch responses,
and kept global fetch blocked.
Context/forced/payload differences persisted at the actual serialized JSON boundary.
Async payload-handler order and quote/backslash/newline/Unicode/SSE-sentinel boundaries passed.
The omitted-hook case was again rejected by the final-payload validator after normal SDK completion.

Evidence is `contract/collector/request-wire-controls/`.
This qualifies the finite text-only serialization observation,
not hosted behavior,
all providers,
instruction authority,
or current permission.
The consumer-owned observer passed `proc_c523`:
25 intact interface cases plus five exact guard-removal failures,
with original source unchanged.
Its stock SDK binding then passed `proc_cf38`:
eight completed sessions,
16 serialized requests,
one inert tool execution,
and no external model requests.
Evidence is `contract/collector/request-observation-owner/` and `contract/collector/request-observation-sdk/`.

Actual decoded response identity reached `beforeToolCall` and matched its serialized request receipt.
Native preparation,
asynchronous permission hooks,
and result hooks remained active.
A native denial bypassed lookup and execution.
Missing binding,
copied/foreign identity,
fixture epoch drift,
actual owned-loader reload,
and explicit invalidation blocked execution.
Copy/foreign controls verified the original remained valid;
stale controls began with a valid lookup.
A subsequent model request invalidated the older receipt.
The source-stage and wire documentation passed native lint/render/readback in `proc_2269` before this owner update.

The owner still carries every collector coverage gap.
Production lifecycle-event wiring,
stop-turn receipt consumption,
complete native parity,
instruction authority,
and current permission remain unqualified.
The complete-group binding subsequently passed `proc_70b6` in `contract/collector/request-observation-batch/`:
five SDK sessions,
ten serialized requests,
no external model calls.
Parallel and sequential groups shared one assessment after both members' final preparation.
Missing ownership and preparation-time source change stopped both members before assessment.
A source change after the first inert execution stopped the waiting second member precisely at execution entry,
retaining the first success and an explicit second-member error.
Assessment counts were `1, 1, 0, 0, 1`;
execution counts were `2, 2, 0, 0, 1`.
This used the existing private SDK/group-dispatch copies,
not installed patches or production approval.
The owner SDK documentation passed lint/render/readback in `proc_9613` before this group update.

Next qualify actual SDK/root lifecycle mutation ownership rather than the fixture epoch counter.
Current Pi 1.0.0 routes session replacement through `AgentSessionRuntime`;
`AgentSession.navigateTree` handles in-session tree navigation.
The actual tree diagnostic subsequently passed `proc_1da7` in `contract/collector/request-tree-lifecycle/`:
six completed SDK sessions,
six injected requests,
no tools or external model requests.
Stop-response lookup was fresh after normal completion.
No-op and cancelled navigation preserved the receipt as expected.
Unwired real navigation and direct manager branching changed/restored the leaf while the constant-epoch observer
still reported freshness.
A `session_tree` invalidator rejected the receipt and prevented revival after restoration,
but the receipt remained usable after native mutation until that callback invalidated it.
Direct `SessionManager.branch()` calls emitted no tree events.
No production transition-window or direct-manager coverage is established.

The existing native-manager registry owns per-manager epochs and issuer-local branch snapshots,
but requires a fixed planned session ID and persistent path and only advances epochs through explicit invalidation.
Do not duplicate that ownership responsibility without inspecting its existing contract.
Runtime replacement/fork controls then passed `proc_7d74` in `contract/collector/request-runtime-lifecycle/`:
14 constructed SDK sessions,
eight owned completed responses,
eight injected requests,
no tools or external model requests.
Cancelled new-session flow preserved the original runtime and fresh receipt.
New/switch/persisted-fork flows changed manager identity;
in-memory forks reused the manager but replaced `AgentSession`.
The unwired in-memory case rejected the old receipt through the stale extension-context check,
not manager identity.
Receipts remained fresh in shutdown and immediately before the host invalidation callback;
explicit invalidation then rejected them before disposal.
The exact deliberate factory failure was retained after old-session disposal,
with no replacement constructed and no old-receipt revival.
Every constructed session completed delegated disposal once.

Next review a lifecycle-only lease extension of the existing ownership registry.
It must account for pre-mutation windows,
direct manager mutation,
and mutable references returned by native getters,
without invalidating every normal model/tool-result append or treating IDs as authority.
The getter/dispatch matrix passed `proc_152e` with 18 native managers,
no SDK sessions,
and no external requests.
All tested getter paths and the original `appendMessage` argument shared the native message object;
controlled mutation changed active graph and projection without changing persistent JSONL bytes.
An in-place method hook did intercept same-object property calls,
but cached pre-install methods and prototype dispatch bypassed it.
Positive property-call controls proved interception was working.
Evidence is `contract/collector/manager-mutation-coverage/`.

No production mechanism is adopted.
The [private lifecycle-only lease proposal](../planning/pi-auto-mode-root-lifecycle-lease.md)
is separate from unresolved writer custody.
The user accepted A:
prototype that contract first,
then address SDK writer custody separately.
Do not treat the lease as proof that every native mutation is observed.
Implementation is authorized only for the private prototype;
production deployment and permission creation remain unapproved.
The private lifecycle path passed `proc_ca4b`:
24 intact cases and five guard-removal failures,
with historical registry source unchanged,
no SDK sessions,
and no model requests.
Evidence is `contract/lifecycle/root-lifecycle-lease/`.
The observer bridge passed `proc_6f61`:
ten intact cases and three guard-removal failures.
Temporary suspension did not latch stale state,
and a refused stream during suspension did not supersede a receipt that could resume after cancellation.
Permanent source/root changes still rejected older receipts.
The real SDK consumer then passed `proc_4b83` in `contract/lifecycle/root-lifecycle-sdk/`:
five completed sessions,
five injected requests,
no tools or external model requests.
Controlled no-op/cancel resumed the original receipt;
managed navigation retired it before native mutation,
and restoration did not revive it.
The exact controlled mutator-entry failure survived retirement with the leaf unchanged.
Prototype dispatch still bypassed the selected hooks and left the receipt active,
explicitly retaining the custody gap.
SDK state-custody intake inspected native own fields,
borrowed projection/entry identities,
message persistence,
and pre-persistence in-place message replacement.
A paired copy-in/copy-out fork experiment is now prepared,
not adopted as complete encapsulation.
Its first staging attempt `proc_c72a` failed before the worker because the stager assumed an absent installed
license path.
Original sources and partial generated output remain intact;
`contract/lifecycle/manager-copy-boundary-v2/` validates the upstream-tag license blob before staging.
The recovery phase then passed `proc_0468`:
30 paired cases using 32 owned native managers,
no SDK sessions or model requests.
The copy variant isolated the tested getter,
append-argument,
and preloaded-input aliases from the canonical graph/projection.
Direct raw-state mutation remained effective in both variants.
The actual SDK consumer then passed `proc_5f26` in `contract/lifecycle/manager-copy-sdk/`:
four completed sessions,
eight injected requests,
two inert tool executions,
no external model calls.
Both variants preserved final-message rewrites,
native preparation/result hooks,
before-settle preview,
append-only context edits,
provider continuation inputs,
and persisted JSON values.
Public event identity stayed intact;
stored-message identity was shared in the baseline and separate in the copy variant.
Only these inspected JSON-like consumers are qualified;
retry/omission paths and arbitrary extension payloads remain unexercised.

A separate private-repository Git recovery blocked commands on the superseded `a673638` transaction.
Explicitly authorized `proc_aac3` preserved and archived 21 journal files without changing index or refs;
normal wrapper status/log calls then passed.
Evidence is `contract/diagnostic/superseded-index-recovery/`.
The first recovery script parse failure remains retained separately.
The private-field experiment then passed `proc_50e1`:
58 observations,
four native managers,
no SDK sessions/model requests.
Its 11 private data fields resisted tested public-field shadows,
but cached/prototype branch calls,
getter overrides,
and public `_loadEntries` remained open routes.
This isolates data;
it does not establish method custody.
`contract/lifecycle/manager-private-fields-sdk/` then passed `proc_809e`:
two changed-variant sessions,
four injected requests,
one inert execution,
no external calls.
Its outcomes matched retained copy-boundary consumer records without replaying unchanged sessions.

The next structural choice is
[the private SDK-owned custody proposal](../planning/pi-auto-mode-sdk-state-custody.md).
Option A prototypes intrinsic lifecycle occurrences and a protected method surface inside the SDK owner.
Option B stops structural SDK work with production mutation custody still unqualified.
The user accepted A:
implement the private SDK-owned custody prototype.
Native qualification now has 24 intact cases and 12 guard-removal outcomes,
reconciled by `proc_2023` with the four-case own-definition supplement `proc_04d7`.
The initial verifier failure `proc_b8f3` remains retained;
no completed native worker was replayed.

`proc_51fd` passed selected SDK consumers with the protected manager:
two sessions,
four injected requests,
one inert execution,
no external calls.

`root-lifecycle-native/` then passed `proc_9258`:
24 legacy,
17 native-reader,
and nine callback/publication cases,
plus 11 guard-removal variants.
`proc_f13e` first reproduced the nine callback/finalization defects on the preserved draft.
The remedy validates after callbacks and leaves failed preparation cleanup available.

`root-lifecycle-intrinsic-sdk/` passed `proc_2e77`:
six sessions,
six injected requests,
no tools or external calls,
and no instance mutator hooks.
No-op/cancel resumed receipts;
managed,
failing,
cached,
and prototype mutation retired them permanently.
Five suspended stream attempts did not advance request generation.

The native-manager occurrence and tested request-observer binding are qualified in this private profile.
The next intake,
`session-disposal-intake/`,
passed `proc_7f17` with one SDK session/request and one disposal.
During the owned abort hook,
both lifecycle-only and full-receipt checks were active.
After disposal,
lifecycle-only assertion and recapture still succeeded;
the full observer rejected through the SDK's stale extension context.
That is a session-lifetime gap,
not a manager-custody safety pass.

The exact-pair reader extension passed `proc_59d2`:
56 pure cases and 12 guard-removal variants.
The private SDK session-liveness controls then passed `proc_deae`:
six sessions/requests,
four intact cases and two exact guard failures,
no tools or external calls.
Direct/cached/prototype disposal rejected old roots before abort hooks;
cleanup and the controlled abort error remained observable.
The earlier `proc_f507` preparation failure was a stale source-hash path,
not an SDK execution;
its original artifacts are retained.

The changed-factory compatibility phases passed:
`proc_7c32` completed two content-consumer sessions/four requests/one inert execution;
`proc_791c` completed six navigation-consumer sessions/six requests/no tools.
Both made no external model calls.

The verified private profile now covers manager occurrences,
session liveness/pairing,
and the stated content/navigation consumers.
Runtime-owned operations were accepted as a separate extension in
[the custody plan](../planning/pi-auto-mode-sdk-state-custody.md).
The user accepted runtime option A,
and the declared private runtime profile is now verified:

- `proc_68df`:
  12 terminal-close cases and three guard removals.
- `proc_ef9b`:
  native callback semantics in three cases/eight constructed sessions/no model requests.
- `proc_a3be`:
  23 constructor/operation-owner cases and six guard removals.
- `proc_07ea`:
  12 actual SDK cases/21 constructed sessions/no model requests or tools.
- `proc_ebd6`:
  ten initial-admission cases,
  three guard removals,
  a retained-source red control,
  and six SDK cases/11 constructed sessions/no model requests.

Runtime operations suspend before before-switch/fork handlers,
retire before shutdown/disposal,
and route manager creation through the declared owner.
Post-apply nesting and original callback errors preserve applied outcomes.
Terminal disposal uses a completion receipt and does not repeat completed cleanup in the tested schedules.
The initial-admission update validates result records before capture and managers before factory execution.
Native incomplete cleanup was retried without reviving the retired root.
Nested factory failure left the applied-but-disposed root unavailable;
nested post-apply callback failure preserved the latest live root.
The original 12-case schedule and its sources remain unchanged.

This profile requires owned non-proxy factory results and a fixed callback-free publisher.
`proc_4f1c` records excluded-contract gaps,
not safety passes.
Constructor-phase requests/protected actions,
cleanup after idle external disposal or mutation,
other mutable state and instruction producers,
current human permission,
and production integration remain unqualified.
Do not infer permission or production adoption from this development authorization.
Runtime operation-entry suspension,
replacement constructors,
content freshness,
and current human permission remain separate.
Installation,
production adoption,
and permission creation remain unauthorized.
Canonical intake is `contract/collector/request-lifecycle-intake/progress.json`.
The runtime documentation passed lint/render/readback in `proc_0b4d` before this getter update.
See [instruction snapshots](../troubleshooting/pi-instruction-snapshots.md#pi-100-terminal-stage-controls).

## Tooling and workflow

The user removed the `todo` extension.
Do not use or reinstall it.
Track work in committed documents.

A disposable replay test diagnosed the old todo/codemode persistence mismatch.
See [the troubleshooting note](../troubleshooting/pi-todo-codemode-replay.md).
The codemode storage limit and later missing Wasm module errors are separate incidents.
Pi updates removed older package paths during this session;
the host scheduler probe pinned installed 1.0.0,
while native qualification deliberately used the unchanged staged 0.87.1 SDK.

When codemode's Wasm load failed,
direct tools remained usable.
Standalone `pi-codemode` 1.0.0 subsequently loaded its native worker and completed two inert calls in `proc_1e95`.
A separate live-tool probe still emitted `Cannot find module 'quickjs-wasi/quickjs.wasm'`,
with a 0.99.2 bundle path in its require stack.
The standalone success does not establish live-tool recovery or a common root cause.
No installed loader was patched.
The native phase used a separately reviewed,
checked,
private stdin file instead of `process.write`.
Do not reconstruct or replay the consumed launch.

The user requested faster progress.
Batch related changes and minimize bookkeeping between meaningful acceptance or irreversible boundaries,
while preserving actual verification and authorization constraints.

## Native request producer evidence

`proc_941b` measured ordinary prompt,
`sendUserMessage`,
manual compaction,
branch summary,
and bug-report summary.
The auxiliary paths reuse the session stream function but bypass ordinary run/payload hooks.
This is a producer-coverage distinction,
not evidence of instruction authority.

`proc_24bc` passed 18 pure cases and six guard removals for a private producer broker.
It binds one-use invocation claims to exact consumers and associates origin with exact returned responses.
Copied or foreign responses reject;
reused response/stream identities invalidate their associations.

`proc_e862` then passed five actual SDK cases/five injected requests with that broker at native call sites.
Main-agent responses pass the main-agent origin predicate;
auxiliary responses fail it.
The existing wire observer still rejects the branch-summary receipt after navigation.
No external model or tool executed.
The collector's unresolved coverage labels remain unresolved.

`proc_c139` failed source staging because its selected historical license had 1070 bytes,
not the pinned upstream blob's 1069 bytes.
Its generated files and failure remain preserved.
Fresh staging `proc_f14b` reused the previously verified SDK-custody license and passed.
No SDK worker ran in the failed staging phase.

Private evidence is under `contract/collector/request-producer-claims/`,
`request-producer-sdk-copy-v2/`,
and `request-producer-native-controls/`.
The next consumer boundary is native complete prepared-group admission,
followed by policy/axiom and finalization coupling.
These producer facts create neither a human grant nor a new judgment budget.

`proc_1aaa` qualified the producer/wire gate at the actual complete prepared-group consumer:
six SDK sessions,
13 injected requests,
and five inert executions.
Parallel and sequential groups each assessed once before both members executed.
Missing wire evidence,
a copied response,
and a genuine same-session auxiliary-summary response each blocked both members before assessment.
When the source changed after the first serial result,
the second member was blocked and the completed first result remained.
This still uses a fixed inert assessment,
not a qualified policy or semantic verdict.

`proc_bed1` reconciled the separate pure consumer controls without replaying workers:
three cases and two guard removals.
The original `proc_6333` summary incorrectly reported six removals from a copied literal count;
its source and result remain preserved.
The next boundary is one enclosing judgment's budget,
code-owned evidence/decision handling,
and dependency finalization.
Start its clock before provenance checks and preparation,
not after either gate.

The separate native-host omission control `proc_3b7a` used one session/two injected requests.
Removing the host checkpoint callback reached assessment where the fixture expected none,
producing exact `AssertionError [ERR_ASSERTION]: 1 !== 0`.
A separate assessment receipt assertion still prevented execution;
this proves omission detection,
not an unsafe execution.
No intact SDK worker was replayed.

## Shared judgment accounting prototype

`proc_7ec7` passed 28 pure cases and ten guard removals for `contract/lifecycle/judgment-budget/budget.mjs`.
The constructor starts one absolute five-second deadline before caller preparation.
The same owner counts at most three initiated client attempts,
including at most one code-classified transport retry.
Returned unresolved values do not trigger retries.
Overlap,
reentry,
late success,
clock regression,
cancellation,
and finished-boundary reuse reject in the declared profile.
Attempt records retain final status and original operation failures.
Classifier failure after expiry retains the transport,
classifier,
and deadline errors together.

This is cooperative accounting,
not five-second handback under a hard stall or ignored cancellation.
It creates no permission and imposes no tool-execution runtime deadline.
Question identity/anti-fishing enforcement,
selection qualification,
one-time enclosing/nested scope construction,
and existing dependency-finalizer integration remain next work.
No SDK session or model ran in this phase.

`proc_035e` subsequently measured a failure-retention gap in the consumed budget source:
separate operation/clock or classifier/clock failures could compare equal and be collapsed.
Four paired contrasts qualify a fresh candidate that distinguishes a new retirement from an already-latched one.
Equal `undefined` values and repeated references now retain separate new occurrences;
rethrowing the already-latched retirement still preserves the original exception.
The earlier matrix was not replayed or rewritten.
Use `contract/lifecycle/judgment-budget-error-occurrences/controls-private/candidate.mjs` for further integration.

## Fixed question selection prototype

`proc_c02b` passed 31 pure cases and 11 guard removals for `contract/lifecycle/judgment-questions/selection.mjs`.
Pinned question definitions supply the exact emitted text.
An owned occurrence-resolver double canonicalizes admitted aliases;
copied or foreign claim/batch handles reject.
Distinct occurrences may use the same predicate,
but an issued claim cannot be issued again after uncertainty or malformed output.
Whole-batch validation precedes issuance and answer publication.

Current-use reads remain source/deadline-gated.
Separate retained-only inspection preserves late or stale diagnostics without reviving usable evidence.
Late responses are recorded as discarded,
not accepted estimates.
The finite profile bounds total declarations and issued batches.
No model,
SDK session,
truth band,
policy verdict,
or human permission was involved.

This does not qualify a native binding resolver,
semantic equivalence across differently named definitions,
or one-time construction across adaptive rounds and nested work.
The next integration extends the existing owned producer response record with one enclosing-judgment reservation,
then binds declared/final prepared members and existing dependency finalization to that same owner and budget.
See private `contract/lifecycle/runtime-continuation/enclosing-judgment-next.json`.

## Enclosing judgment reservation prototype

`proc_757f` passed 23 pure cases and eight guard removals for `request-producer-judgments/`.
The existing producer response record reserves one enclosing judgment;
no separate session/root identity registry was added.
Duplicate construction and reentry reject,
failed construction retains its original cause and cannot reset that response's budget,
and separate responses remain independent.
The original start is captured before entry getters and constructor work.

This is a pure reservation profile,
not native SDK group-entry or finalization integration.
A subsequent private delta attaches each fixed session-specific constructor to the existing stream-consumer identity.
Its goal is to avoid caller-selected factories or a competing stream/session lookup registry.
The fixed creator and occurrence resolver were subsequent composition work;
actual SDK integration remains next work.

`proc_c390` passed the constructor-binding delta:
eight pure cases and three guard removals.
The exact existing consumer identity holds its fixed constructor;
call-entry metadata cannot select another constructor.
`proc_17a2` then staged nine private SDK artifacts without importing the SDK.
The protected copy always uses complete-group admission and requires synchronous enclosing-judgment entry.
Missing hooks cannot select independent per-tool fallback.
The SDK staging is source/syntax evidence only;
actual SDK consumer integration is not yet qualified.

## Fixed judgment composition controls

`proc_6147` passed 20 new composition cases for the private fixed creator.
The tests use owned producer/root/source doubles and real descriptor-bound disposable target files,
not actual SDK sessions.
The creator combines the original-start budget,
complete ordered prepared membership,
canonical member claims,
one-use assessment dispatch,
and dependency finalization.
Failed or late responses remain discarded;
failed close retains its original cause,
including `throw undefined`,
without becoming current evidence or permission.

`proc_a447` passed seven intact delta cases and six exact guard mutants in 13 sequential workers.
The controls cover the shared attempt ceiling including a transport retry,
the four MiB serialized request bound,
and guard sensitivity for original start,
member order,
target freshness,
callback reentry,
pre-dispatch terminal retention,
and request size.
The parent program,
final prepared group,
and observed request body appear once in the shared request envelope,
not once per question.
After evidence closes,
continuation checks retain root/source/target/cancellation checks without imposing the decision-model deadline
on tool execution.
No external model requests,
SDK imports,
semantic calibration,
policy verdicts,
or human permission were established.
The native consumer result is recorded in [Native fixed-judgment consumer](#native-fixed-judgment-consumer).

## Native fixed-judgment consumer

`proc_e427` passed eight actual private SDK cases:
eight sessions,
16 injected wire requests,
eight inert executions,
and five canned semantic attempts.
The exact producer response/consumer reserved one judgment before native preparation.
The final sealed member group,
canonical question bindings,
canned transport,
finalization,
and execution freshness checks consumed that same owner.

Missing beginning or combined-admission hooks rejected the whole group with the historical opt-in flag disabled.
Preparation deadline expiry prevented semantic dispatch and execution.
Advancing the decision clock after evidence closed did not impose a tool-execution deadline.
Separate source-change and target-change cases retained the first completed serial effect and rejected the next member.
Native persisted outcomes,
complete streams,
source hashes,
and disposal completion were reconciled.
`proc_0c33` separately passed four preparation-checkpoint and JSON-profile controls;
unsupported `undefined` and negative-zero tool values cannot silently change during JSON snapshotting.

This qualifies the configured fixed-bootstrap profile,
not arbitrary callback providers or nested semantic groups.
The native freshness callback was optional in the `proc_e427` SDK graph.
`proc_a8bf` staged a fresh graph requiring and capturing that callback.
`proc_6979` passed its three native delta cases:
three sessions,
six injected requests,
three inert executions,
and two canned semantic attempts.
Absent freshness support rejected the complete group before preparation or assessment.

`proc_5749` then omitted only the actual native `member-execute` freshness invocation.
The unchanged target-change fixture observed two inert executions instead of one and failed with
`AssertionError [ERR_ASSERTION]: 2 !== 1`.
The first completed effect and both native results were retained;
other checkpoints and fixture release synchronization remained intact.
The guard worker used one session,
two injected wire requests,
and one canned semantic attempt.
Its original assertion,
cleanup completion,
and full stream/source evidence were checked.
No completed intact SDK worker was replayed.
Native nested execution and complete enclosing-program linkage remain the next mechanical area.
`proc_4202` completed its corrected path/lifetime diagnostic:
three sessions,
six injected requests,
three outer inert executions,
and seven nested inert executions.
Children retained ordinary native hooks and parent records but did not enter outer complete-group judgment hooks.
A saved context executed another inert child after the parent result and final assistant response,
reused the `/1` child label,
and left the persisted parent record unchanged.
The original `proc_8724` failure excluded the leaf through the fixture's SDK allow-list;
its two `Tool owned_leaf not found` records remain preserved,
not counted as nested guard rejections.
See [the nested-context diagnosis](../troubleshooting/pi-nested-tool-context.md).
The reviewed next design carries exact execution occurrences through existing contexts and scope owners,
without equating root liveness,
parent return,
recorder consumption,
or a worker drain with complete semantic membership.

`proc_cb12` passed seven interface controls for a per-invocation context-lifetime prerequisite,
using the native tool-definition wrapper.
`proc_01d5` passed an intact control and two guard removals.
`proc_7b35` then staged ten private SDK artifacts.
`proc_fc01` passed three actual SDK cases:
three sessions,
six injected requests,
three outer inert executions,
six nested inert executions,
and three canned semantic attempts.
Live parallel and sequential children retained native hooks and parent records.
The saved context rejected `NestedContextClosedError` before another child invocation.
The wrapper preserves lazy native context descriptors and closes only new nested invocation after parent return;
it neither rolls back an already-started child nor proves successful drain.
Original judgment/cancellation linkage,
queued-child release checks,
and complete nested semantic membership remain unfinished.
`proc_8bea` separately measured cancellation while the parent remained active.
Both cases aborted the original parent signal;
the inherited signal prevented the child,
but a substituted live signal allowed one inert child execution.
The diagnostic used two SDK sessions,
two injected requests,
and two canned semantic attempts.
A fresh wrapper derivative composes original and additional cancellation instead of substituting them.
`proc_4be3` passed ten native signal/helper controls;
`proc_4dbb` passed independent original/child cancellation guard removals.
`proc_2cf6` then passed four actual private SDK cases:
four sessions,
four injected requests,
four canned semantic attempts,
and child execution counts `[0, 0, 1, 0]`.
Original-aborted inherited and substituted signals both prevented the child;
live parent/child signals allowed it;
a separately aborted child still prevented it.
This qualifies that cancellation profile without introducing an execution deadline.
The next occurrence/carrier phase completed its declared root-member profiles.
`proc_ae68` passed 13 occurrence-interface cases;
`proc_89b5` passed seven focused guard mutations.
`proc_8b95` passed 17 context-carrier interface cases,
`proc_85d5` passed ten fixed-session reader cases,
and `proc_9ed1` passed ten carrier/reader guard mutations.
`proc_2da7` passed ten native invocation/retirement helper cases,
including distinct original and cleanup `throw undefined` occurrences.

`proc_b813` staged eleven private SDK artifacts.
`proc_681e` then passed seven actual SDK cases:
seven sessions,
14 injected requests,
six canned semantic attempts,
eight root definition entries,
and no child executions.
The existing producer response and prepared member own the execution occurrence;
the fixed session reader supplies its exact native context at actual entry.
Result hooks observed returned occurrences,
including exceptional exits.
Missing context factories rejected the complete group before preparation;
copied contexts entered no definition;
ordinary tool errors retained native serial continuation;
and source change retained the first result while blocking the next member.
Missing complete nested-group support now rejects explicitly instead of falling back to independent child execution.
`proc_6251` omitted issuer retirement in the actual native return path.
The worker retained active issuer states where returned states were required,
and original execution origins remained readable at result hooks.
The expected assertion verified that local context return is not enough to retire the original occurrence.
It added one session,
two injected requests,
one canned semantic attempt,
and two unchanged root definition entries;
no extra effect or child execution was claimed.

The focused stop-latch omission `proc_8319` removed only the stop-remaining check.
The first member guard-failed;
the otherwise intact second member then entered.
The expected Node assertion was `AssertionError [ERR_ASSERTION]: 2 !== 1`.
This added one session,
two injected requests,
one canned semantic attempt,
and no child executions.

`proc_b861` passed five changed-graph deadline/freshness regressions:
five sessions,
ten injected requests,
two canned semantic attempts,
and definition-entry counts `[0, 0, 0, 2, 1]`.
Missing beginning and combined hooks,
plus expired preparation,
entered no definition.
Execution after evidence close was not capped by the decision deadline;
target change after the first effect stopped the next member.

`proc_6894` passed the mixed-failure profile in one session with two injected requests and one canned attempt.
The first member raised an ordinary tool error;
the second completed;
the third encountered a nested-support guard failure;
the final intact member remained unentered.
Native error flags were `[true, false, true, true]`,
and issuer states were `[returned, returned, returned, unentered]`.
The completed second result remained in native persisted outcomes.
No child executed.

A separate bounded Codemode declaration prototype now has native-worker evidence.
`proc_fda3` passed four positive source cases,
31 rejected source cases,
three catalog rejections,
and a compiled-source size rejection.
It retains the original parent source,
reparses generated callsite aliases,
records canonical argument JSON,
and reconstructs the original source exactly.
These structural checks do not estimate effects.

`proc_6838` passed nine run-local collection cases.
Each callback belongs to a private declared-member record;
two callbacks for the first member do not complete a missing second member.
Equal alias names from separate runs do not combine their membership.
Original cancellation,
native-call cancellation,
parent return,
and staleness retain their separate rejection paths.

`proc_68c2` passed seven actual Codemode 1.0.0 sandbox executions:
original/compiled pairs for ASCII and escaped Unicode parents,
plus duplicate-member,
swapped-argument,
and cancelled incomplete-set controls.
Original and compiled forms retained the tested argument JSON,
call ordering,
and ordered return values under controlled reverse settlement.
`proc_5d76` then omitted only actual-argument versus declared-callsite comparison.
Its expected native assertion was `AssertionError [ERR_ASSERTION]: 1 !== 0`:
the changed collector reached a group consumer that the intact guard prevented.
This added one sandbox execution,
not an SDK session or model request.

The prototype has not yet been connected to SDK child preparation,
the original judgment's child occurrences,
or the native nested recorder and queue.
It is not a workaround for the current closed SDK nested path.
Private evidence is in `contract/lifecycle/codemode-declared-group/`;
the concrete integration work is recorded in
`contract/lifecycle/runtime-continuation/native-declared-group-integration-next.json`.

The unsupported-child recorder/event check `proc_4d8b` passed one SDK session with two injected requests.
An installed native-runner positive control produced child events and a recorder entry.
The protected SDK case produced no child event,
recorder start,
or persisted nested record.
This pins the refusal boundary,
not admitted-child parity.

Collector lifecycle diagnostics `proc_398f` then reproduced post-settlement cancellation relabeling,
a native abort during the final parent-reader callback,
and a separately thrown consumer failure lost behind an existing cancellation.
The fresh `codemode-group-lifecycle` collector passed 17 controls in `proc_2404`,
three actual native Codemode lifetime cases in `proc_3c7d`,
and four exact guard omissions in `proc_3102`.
Transport delivery and eventual completion are now separate:
the native fail-fast case retained a delayed successful completion after its transport call was cancelled.
Collector states do not establish actual SDK execution start.

The subsequent `proc_62e1` contrasts preserved two original failure witnesses and passed four fresh candidate cases.
A synchronous group-consumer callback could capture `drain()` before `work = dispatch()` assigned its result.
The original callback control failed with `AssertionError [ERR_ASSERTION]: true !== false`;
the candidate publishes its drain ticket before entering callbacks.
The original malformed completion array also caused Node to report
`Error: Owned unbound completion failure`.
The candidate observes and drains short,
sparse,
and extra completion arrays,
retaining their outcomes as unbound diagnostics rather than admitted member completions.
The initial review's no-window conclusion covered ordinary callers after invocation return,
not the synchronous callback exercised by this contrast.

The current collector source is
`contract/lifecycle/codemode-group-drain/collect.mjs`.
Consumed predecessors remain preserved.
These checks assume ordinary finite arrays from the fixed native consumer;
they do not qualify hostile publishers or callbacks that await their own drain to produce its pending completions.
No new judgment,
model attempt,
permission,
or runtime execution deadline was introduced.

The first connected literal SDK profile passed in `proc_5927`:
two sessions,
four injected requests,
and one original canned assessment per session.
Both native child preparations completed before either child definition entered.
The valid program entered both children;
a native hook changing final child inputs entered neither.
Original child names,
text results,
update events,
recorder entries,
and persisted messages were checked.
No child judgment or model attempt was created.

`proc_994f` removed only the post-preparation input binding.
Its expected native assertion was `AssertionError [ERR_ASSERTION]: 2 !== 0`:
both inert children entered with transformed inputs where the intact group rejected them.
Returned child/root issuer states,
native outcomes,
complete streams,
and disposal were retained.

`proc_79f0` passed three additional SDK cases:
a copied definition with the same name,
schema,
and execute function rejected;
advancing the owned clock after evidence close still allowed both children;
changing a disposable target after the first serial child retained its result and blocked the second.
These cases used three sessions,
six injected requests,
three canned assessments,
and child-entry counts `[0, 2, 1]`.

The graph is the private `contract/collector/native-program-sdk-copy-v3/` derivation.
Its 21 artifacts bind parser dependencies explicitly,
so SDK workers keep their cleared `HOME`.
The failed original parser intake and a separate generator-escaping error remain preserved;
see [SDK staging](../troubleshooting/pi-sdk-staging.md).
The installed SDK and production auto-mode were not changed.

`proc_15e7` added native `Promise.allSettled` child-failure controls.
An ordinary child error allowed the next serial child to complete;
a required-group guard failure left the next child unentered.
The two sessions used four injected requests and two original canned assessments,
with child-entry counts `[2, 1]`.
This avoids confusing ordinary errors with `Promise.all` fail-fast transport cancellation.

The first queue fixture `proc_ff76` remains failed:
it treated the current `queueTail` as the active first lease.
Fresh `proc_bb45` observed actual queue publications and their predecessor identity instead.
Two parallel program parents contended for one sequential-child queue under one judgment.
It checked four child definitions,
separate recorder summaries,
opaque owner-key consumption before agent end,
four exact update payloads,
and synthetic per-parent usage totals.
This is not a claim about provider billing or arbitrary recursive queue ownership.

`proc_dd3b` cancelled the original agent while the first child definition was running.
The child observed its composed signal abort;
the second definition remained unentered.
Both already-prepared native attempts retained their start/end records.
One SDK session,
one injected request,
and one canned assessment completed with returned parent/child issuer state and clean stderr.
A recorder start is not a definition entry,
and retained diagnostics are not fresh cancelled evidence.

`proc_f6a8` forwarded the non-program root regressions onto the changed wrapper.
Two sessions and four injected requests retained definition-entry counts `[2, 1]`
for execution after evidence close and target change after the first effect.

This qualification covers model-issued native Codemode parents with the declared literal group shapes.
Full image/store parity,
arbitrary recursive queue ownership,
body-return/queued-release combinations beyond the named cases,
and general dynamic groups remain open.
The result-derived native profile then passed `proc_0963`:
three SDK sessions,
six injected requests,
one original canned assessment per session,
and child-definition counts `[3, 1, 1]`.
A producer generated its string only after evidence collection closed;
two later sinks received that exact owned value after complete native preparation.
Advancing the owned clock did not create an execution deadline.
An ordinary producer error left the successor group unreached,
with no successor handles or attempts.
Changing final successor inputs blocked both sink definitions.
The full parent source and one original judgment remained in use.

A separate boundary gap was measured in `proc_0b5a`.
Refusing a converted non-string value became an ordinary VM script error,
so an otherwise unstarted serial root tool still ran.
The fresh guard category and original-program latch passed `proc_408c`:
normal results,
binding refusal,
and ordinary producer failure produced tail-entry counts `[1, 0, 1]`.
Ordinary conversion errors remain distinct from code-owned result guards.

`proc_57e1` omitted only the final guard relay during program drainage.
The expected assertion `AssertionError [ERR_ASSERTION]: 1 !== 0`
showed that retaining a guard marker alone does not stop the sibling root.
`proc_b8ff` refused a copied native producer outcome even though its actual conversion succeeded:
the successor stayed unreached and the sibling root stayed unentered.
`proc_abc3` omitted only the final receipt-derived input comparison.
Its expected assertion `AssertionError [ERR_ASSERTION]: 3 !== 1`
retained the successful producer and two transformed inert sink entries.

Current private sources are
`contract/lifecycle/result-binding-guard/`
and `contract/collector/result-program-sdk-copy-guarded/`.
This is a finite string-result producer/successor profile,
not universal program analysis,
qualified semantic effect estimates,
governing-instruction authority,
human permission,
or production adoption.
Current-use checks remain separate from retained result and cleanup records.

`proc_4ede` then cancelled the original run at a native producer-result update,
after the owned string receipt existed and before the successor activated.
One SDK session made one injected request and one canned assessment.
The completed producer retained its `ok` outcome and unchanged converted value;
no sink started,
the successor stayed unreached,
and the parent issuer returned after drainage.
Receipt readiness is a recorded observation of the owned program,
not authority derived from an event label.
The independent guard review and source-reconciled findings are retained in
`contract/lifecycle/runtime-continuation/result-guard-review-resolution.json`.
Definition ownership is factory-closure ownership,
not session or human authority;
the fixed session reader and registered receiver separately bind each execution.

The literal native group profile is mechanically connected;
general parent-program/dataflow coverage and dynamic child groups remain unfinished.
The accepted preparation requirement applies to each actual execution group,
not every future dynamic descendant at once.
Whole-parent semantic coverage may include result-derived bindings;
this does not require code-only effect proof.
Later mechanical validation neither creates another judgment nor reopens its inference budget.
A bounded static extraction profile is a prototype mechanism,
not a newly adopted universal policy.
No external inference,
semantic accuracy,
instruction authority,
policy verdict,
current human permission,
or production adoption was established.

## Owned resource inputs and request freshness

The private loader factory in `contract/collector/resource-input-custody-v2/`
now retains inputs before configured transformations and outputs after them.
It composes with the existing collector rather than replacing policy-byte retention,
request ownership,
or the original judgment.
Its resource records do not create another session/root registry or inference budget.

`proc_5602` passed 17 pure controls.
`proc_6b95` passed four actual SDK 1.0.0 loader cases,
including original-input retention,
same-byte reload invalidation,
forged callback refusal during an owned reload,
and copied-output refusal.
These loader-only cases created no `AgentSession`.
`proc_0116` removed only the raw output-identity comparison:
the copied-output assertion failed with
`AssertionError [ERR_ASSERTION]: Missing expected exception (ResourceInputCustodyError).`
Value and occurrence checks remained intact.

The actual-session integration retained the completed baseline from `proc_9b98`
and qualified the changed-source case with `proc_edba`.
The intended pair used two SDK sessions,
four injected requests,
and two original canned assessments,
with root-definition counts `[2, 1]`.
A same-byte native resource reload invalidated the original request occurrence;
the first completed outcome stayed recorded and the second root remained unentered.
The original failed suffix is separate evidence:
including it,
these executions used three sessions,
six injected requests,
and three canned assessments.

The first SDK fixture expected a leaf `SourceCollectionError`
instead of the observer's `RequestObservationStaleError` wrapper.
That failed namespace remains unchanged;
only the affected case was rerun with the outer error and cause asserted.
The [instruction snapshot report](../troubleshooting/pi-instruction-snapshots.md)
also records the unsupported-callback cleanup correction.

The next private derivative binds collection through intrinsic session readers,
not a caller-supplied context/loader pair.
`proc_8417` passed seven controls using two actual SDK 1.0.0 sessions:
public loader/prompt shadows did not redirect the native readers,
copied sessions and foreign-owner substitutions rejected,
and disposal blocked current use while old observations remained inspectable.
`proc_7ed2` then passed the real original-judgment consumer:
one session,
two injected requests,
one canned assessment,
and two root definitions after evidence collection closed.
This still observes a native prompt view;
it does not establish the complete derivation of base/run options or their authority.

A later host update installed Pi 1.0.2 and parser 0.17.0.
Fresh source comparison and relocation preserved the completed 1.0.0 records.
`proc_89f9` rechecked the paired original-judgment consumer,
and `proc_35e7` rechecked result-derived successor execution:
three updated-SDK sessions,
six injected requests,
child entries `[3, 1, 1]`,
and sibling-root entries `[1, 0, 1]`.
The [SDK staging report](../troubleshooting/pi-sdk-staging.md)
retains the source delta,
separate failed setup attempts,
and exact verification scope.

This qualifies pre-transform observation custody and named freshness paths,
not instruction authority,
raw-file identity,
legitimate delegation,
all prompt transformations,
semantic calibration,
or current human permission.
Next connect these owned inputs to native prompt composition and the final request,
then apply code-owned source authority and policy without promoting filenames,
role labels,
or equal rendered text.

## Native base-prompt derivation

The private `contract/collector/base-prompt-sdk-copy/` graph now links actual native base normalization
to a constructor-issued resource binding.
Canonical base options live in native private state and are immutable owned copies.
Command-context readers and `before_agent_start` receive separate mutable copies,
including nested skill-source metadata.
These records establish observed derivation,
not instruction authority or legitimate delegation.

The first helper had two measured gaps:
`proc_14bd` retained a base as current after its resource occurrence changed,
and `proc_8da2` accepted a same-value accessor replacing an output data field.
The corrected helpers passed 17 pure cases in `proc_3378`.
`proc_d6a3` removed the resource-occurrence comparison and data-descriptor guard separately;
both reached their exact missing-exception witnesses.
Original diagnostics and review corrections remain preserved.

The actual SDK controls in `proc_516c` used two sessions and one injected request,
with no semantic assessment.
They confirmed actual normalizer field shape,
isolated command copies,
mutable run-hook copies reaching the wire,
unchanged canonical base,
missing-binding refusal,
and disposal-aware current reads with retained inspection.
The earlier `proc_c33b` fixture failed before any wire request
because it bypassed the existing producer binding;
that one-session failed prefix remains separate.

`proc_02ab` then passed the original-judgment consumer:
two sessions,
four injected requests,
two original canned assessments,
and root-definition counts `[2, 1]`.
A native same-value base rebuild invalidated the original source snapshot.
The first completed outcome remained recorded,
and the second root stayed unentered.
Post-deadline execution still used the closed original budget.

This is the declared base-derivation profile,
not a universal rule about every legitimate dynamic source transition.
Run-handler ancestry,
forced projections,
other prompt producers,
governing-source authority,
qualified semantic applicability,
and final policy decisions remain open.
The next source step must preserve the actual handler snapshot and native working-copy flow,
not infer source membership from callback counts or equal text.

## Owned native run publication and next-turn refresh

The private Pi 1.0.2 run derivative now records the actual ordered handler snapshot,
handler processing and failures,
an owned session reconciliation copy,
and the final immutable run options.
Native next-turn refresh retains its actual prior run and current base as separate ancestors.
This extends source observations,
not governing authority or policy decisions.

`proc_1b81` passed 21 pure controls.
Independent review and targeted diagnostics then found additional copy and serialization boundaries:
`proc_73ca` let a retained handler alias change options between emitter return and final publication;
`proc_9709` invoked a raw thrown value's `toJSON` during source-size accounting.
The fresh v3 helper transfers a separate reconciliation copy before subsequent native awaits.
Published failure records contain inert occurrence descriptors;
retained inspection still contains the original thrown values.
`proc_e503` passed the changed-boundary controls.
`proc_3237` verified separate single-expression omission witnesses for both repairs.

`proc_0a79` qualified the actual native handler path:
one session,
one injected request,
three ordered handlers,
one expected ordinary error report,
and no semantic assessment.
Forced text and partial mutations were retained;
a same-tool base rebuild preserved its original causal base;
a borrowed alias mutation after image normalization did not reach the request.
Public run/runner shadows were ignored.
Reset,
persisted entries,
disposal,
and retained inspection were checked.

`proc_d157` connected the run observation to the original judgment:
two sessions,
four injected requests,
two canned assessments,
and root-definition counts `[2, 1]`.
The positive case executed after the original decision deadline.
The negative case injected a real native refresh operation after the first inert root,
changing the run occurrence without changing its bytes or base.
The original source check stopped the second root,
retaining the first outcome and closed budget.
That injection is not evidence of a scheduled second turn.
Both sessions separately reached a genuine next model response,
where the native loop performed its scheduled refresh.

The failed `proc_602f` remains separate:
one session and one injected request,
with completed cleanup,
but no completed-case receipt.
It incorrectly expected a next-turn refresh in a run containing only one text response.
The native loop requires a completed turn and another loop iteration.
The hooks-only recovery did not repeat the failed empty-case prefix.

Current native graph:
`contract/collector/run-prompt-sdk-copy-v2/stage-private/manifest.json`.
The first staging attempt,
`proc_f30f`,
used the wrong source ledger and imported no SDK.
The corrected graph derives the installed runner from the checked session import.

Session-long history retention,
hard stalls,
images,
pending-hook session/runner replacement,
and arbitrary publishers remain unqualified.
The next substantive work is host-source authority and deterministic policy integration,
not another unchanged runtime matrix.
These observations do not authenticate instructions,
create permission,
adopt thresholds,
or complete the migration.

## Deterministic policy consumer with explicit fixture premises

The first policy-consumer path now runs through actual private SDK judgments,
rather than ending at source-custody checks.
It is still a fixture trial:
instruction effects and applicability are authored test premises,
not admitted real-world authority or qualified model estimates.

`proc_dfd6` compared the deterministic kernel against 6,175 authored rule programs
and 14,425 complete-fact evaluations.
The finite domain covered up to three rules,
three effects,
two priority values,
and three applicability states.
An initial `proc_9cec` failure retained 373 completed programs before exposing
an unnecessary question about an already-settled equal-priority conflict.
The corrected kernel no longer asks about additions that cannot repair that conflict.

Independent review distinguished settled effects from settled controlling-instruction identity.
A higher unresolved rule with the same effect can change which instruction controls
without changing the effect.
The clarified v3 metadata therefore names highest-known applicable support,
not a unique winner.
`proc_ec0f` checked that delta,
priority gaps,
multiple supports,
and the 256-rule boundary.
It also verified the native trial's distinct refusal diagnostics without rerunning SDK sessions.

`proc_7931` checked the fixture adapter's original-response binding,
one-shot resolution,
original deadline,
post-close freshness,
copied-receipt rejection,
and refusal to promote any fixture receipt into host admission.
`proc_7efd` then ran five actual SDK 1.0.2 sessions,
ten injected requests,
and five canned assessment attempts.
Root-definition counts were `[2, 0, 0, 0, 0]`
for permission,
prohibition,
required approval,
unresolved applicability,
and no applicable fixture rule.
The source program reached the actual wire;
policy computation preceded finalization inside the original deadline.
The permitting fixture's second inert body ran after the closed decision clock advanced.

The canned numeric estimates were not treated as qualified policy facts.
The harness independently restricted its own inert definitions.
All non-permitting states used the same native block transport,
with distinct recorded states and diagnostics.
This does not test an interactive approval workflow
or adopt a real instruction-free fallback.
Every fixture receipt remained inadmissible for real host execution.

Current kernel:
`contract/lifecycle/instruction-policy-consumer-v3/resolve.mjs`.
The native trial used the v2 kernel and remains effect-only evidence.
Next:
establish configured-host governing-source selection,
bind qualified semantic relation evidence,
and connect actual admission and approval paths.
The existing source collector still correctly reports authority as not established.

## Real policy binding and live instruction-condition study

The private host path now explicitly designates a governing-policy input
through the existing constructor-owned resource owner,
rather than promoting a loaded filename or caller `readPolicy` label.
`proc_2d5b` carried all 31,421 bytes of the protected current `AGENTS.md`
through the native base,
wire request,
and original estimator envelope.
It used one SDK 1.0.2 session,
two injected requests,
one canned assessment,
and two inert root definitions.
The file stayed unchanged.
Priority,
rule interpretation,
delegation,
total instruction coverage,
and actual permission were not established by that designation.

`proc_db07` then bound exact `PX3` and `LN7` clauses to original prepared-member identities
using the incumbent native Markdown parser.
Unknown clauses,
copied handles,
and replaced source occurrences rejected.
The full policy is now an explicit envelope field,
not replaced by clause excerpts.
This remained a mechanical SDK control with canned estimates.

The live Jev study was different:
`proc_e22b` verified 14 actual requests,
no retries,
and two fixed passes over seven development programs.
The predicate estimated whether a complete program weakened effective lint enforcement
under a declared in-memory configuration contract.
Expected labels came from independent reached-state execution before inference.
The actual full policy and exact `LN7` text were included;
no reserved cases were opened.

Results were:

- At 80/20:
  ten correct,
  three wrong,
  one unresolved.
- At 90/10:
  six correct,
  three wrong,
  five unresolved.
- At 95/05:
  six correct,
  zero wrong,
  eight unresolved.

An uncalled weakening function scored `0.90` and `0.93`.
A write to an unused configuration scored `0.93` and `0.16` on identical input,
an absolute spread of `0.77`.
The stricter band abstained on every negative example;
it is not an adopted remedy or held-out qualification.

Published returned-input pricing gives study total US$0.005012028
and mean US$0.000358002 per request/judgment.
The combined retained semantic-study subtotal is US$0.044820032.
These exclude unknown gateway charges and agent/reviewer costs;
they do not establish invoices or representative production mean cost.

The next change is decomposition,
not new wording or threshold selection.
The finite `lint-write-absence-v2` profile can establish absence of an active write
for the documentation,
uncalled-function,
disabled-branch,
and unused-target examples.
Active writes or unsupported syntax remain unresolved.
A first implementation's hoisted-shadowing error was measured by `proc_3cb1`
before admission and corrected.
`proc_dcb4` and `proc_74a8` retained the scoped controls and read-only replay.
Replay keeps code facts distinct from unqualified estimates;
it does not erase the 14 requests or lower their actual recorded spend.

The integrating consumer must still own the exact tool contract and active configuration identity.
Names such as `tools.write` and a supplied path cannot establish that ownership.
No real action permission,
semantic threshold,
or production deployment follows from these results.

## Owned condition facts in the native consumer

`proc_4670` connected the conservative condition fact to an actual SDK judgment.
The existing native tool-definition registry became private,
with a fixed reader returning the actual registered object.
A factory owned both the concrete in-memory implementation and its active configuration.
The condition owner read the canonical prepared program from the original judgment;
it did not accept a caller's target path or profile label.

The control used two sessions,
four injected requests,
zero semantic attempts,
and root-definition counts `[2, 0]`.
The code-resolved batch executed in the owned reference runtime,
including after the original decision clock closed.
The unresolved batch was withheld.
Copied definitions and evidence rejected;
public method/map shadows did not replace the intrinsic definition source.
Persistence,
full streams,
source identities,
and disposal were checked.

This is a real consumer integration of a closed reference contract,
not native filesystem or linter parity,
arbitrary program execution,
or real action permission.
The next contract must be established from actual built-in construction and behavior,
not a matching tool name.
No additional Jev request was made for this control.

## Native text-read implementation facts

`proc_5efd` passed the first real built-in read profile:
five SDK sessions,
ten injected requests,
zero semantic attempts,
and completed native reads `[2, 0, 1, 0, 1]`.
Default text reads returned their actual disposable file contents.
Custom same-name reads remained unqualified.
Replacing the original execute method or changing an observed file after the first result stopped the remaining read.
Image-selected input stayed outside the text profile.

The native default factory branch captures the original definition before extension setup.
The existing private registry supplies the current definition.
The original judgment's dependency facade supplies the observed file bytes,
not a caller-provided text-file label.
Public shadows,
copied evidence,
and copied member handles cannot replace those sources.
Post-closure freshness checks retain the original budget and cancellation lineage.

`proc_c618` checked the changed boundary without more SDK sessions or inference.
Removing the execute-field comparison made the exact guard assertion fail.
The actual native file MIME sniffer matched the snapshot-prefix detector in the named controls;
a positive control proved full-buffer detection can differ from the native `4100`-byte prefix.
The helper also left normalized alternate paths,
controlled observers,
undeclared files,
images,
and non-UTF-8 inputs unresolved.

The emitted fact concerns the default native text-read body only.
It explicitly does not establish the whole operation's lint-policy condition,
callback effects,
later transfers,
atomicity,
or permission.
The next semantic work separates a clause's stated approval prerequisite from program-effect prediction.
A positive prerequisite relation must not become a verdict or a demand for a new human grant:
applicability,
existing governing authorization,
scope,
approval type,
and hierarchy remain separate.

## Instruction-prerequisite relation result

`proc_ef2e` verified a separate rule-meaning diagnostic:
12 live Jev requests,
two fixed identical passes over six actual policy clauses,
and no retries.
The predicate asks whether the selected clause states an approval
or authorization prerequisite for any described action.
It does not ask whether the proposed action is covered,
whether authorization is already supplied,
who can supply it,
or whether a new human grant is needed.
Every request retained the complete current policy,
the exact indexed clause,
and the complete fixed enclosing program.
Reference labels were written before inference and independently reviewed.

Scores in the respective passes were:

- `LN7`:
   `0.98 / 0.98`.
- `PX2`:
   `0.97 / 0.97`.
- `PX3`:
   `0.96 / 0.96`.
- `HRM`:
   `0.08 / 0.08`.
- `LN3`:
   `0.13 / 0.11`.
- `GCE`:
   `0.08 / 0.08`.

The unadopted 80/20 diagnostic band matched all 12 development references.
At 90/10,
ten matched and two remained unresolved.
At 95/05,
six matched and six remained unresolved.
None of these results adopts a threshold or establishes held-out correctness.
The target changed from predicting program effects to extracting a narrow relation from a selected clause;
the failed broad effect head remains unqualified.

Modeled returned-token spend was US$0.004224444 total
and US$0.000352037 mean,
using the retained published rate.
The combined recorded semantic-study subtotal is US$0.049044476.
These values exclude reviewer/agent cost,
unobserved gateway attempts or charges,
and invoice verification.
The live namespace is consumed.
Next,
connect the fixed relation to the actual original SDK judgment beside the owned code-effect facts,
while keeping estimates unqualified for policy permission.

## Live native assessment and pending approval

`proc_f481` verified one real Jev request inside an actual SDK original judgment,
with two local injected main-model requests and two completed disposable-file reads.
The fixed prerequisite question assessed the exact `LN7` clause
and returned `0.97`.
Complete policy,
original main-request bytes,
parent program,
prepared group,
and clause binding reached the guard request.
The original deadline was retained.
The estimate remained unqualified,
and the reads relied on independent fixture-only authorization,
not on its value.

Modeled returned-token cost was US$0.00076335.
The combined recorded semantic subtotal is US$0.049807826.
Neither figure establishes a representative workload mean,
invoice totals,
or gateway-internal attempt counts.
This live namespace is consumed.

The next boundary is a genuine scoped-response handoff after model evidence closes.
`proc_92b4` passed scripted presentation,
original-capture,
and pending-group controls.
It also reproduced a dead Boolean-versus-number consistency check in the first capture helper:
a nonblank captured approval could be paired with a cancelled requester result.
The corrected helper rejects that inconsistency.
It retains response diagnostics when source,
member,
or cancellation checks later prevent release.

`proc_3e9a` then exercised the actual current requester,
default launcher,
request-owned helper,
and original raw capture with a disposable scripted terminal/editor.
Approved,
denied,
and blank/cancelled cases completed,
with private workspace removal and empty helper stderr.
Pre-spawn intent validation and post-spawn occurrence checks stayed separate.
These were scripted controls,
not genuine human responses.
The current source is prepared for native pending-group integration;
no new genuine approval has been captured.

## Next work

The runtime result was a checkpoint in the migration,
not completion of the authorized mechanical queue.
The user had to request continuation after that premature stop.
Continue directly through the remaining authorized work;
keep adoption and human-permission gates separate from mechanical progress.

Current independently verifiable areas:

- [x] Measure constructor-phase requests and root publication using the existing observer and actual SDK callbacks.
  `proc_587f` passed five cases/nine sessions/nine injected requests.
  New-manager startup requests created a root that later publication rejected;
  failed factories left that candidate live until explicit fixture cleanup.
- [x] Integrate factory-occurrence ownership into the existing registry,
  preserving valid startup requests while retiring failed candidates.
  `proc_0c7e` reconciled nine intended cases/15 sessions/14 injected requests.
  Original failed fixture/verifier processes remain preserved;
  completed workers were not replayed.
- [x] Qualify runtime cleanup after idle external disposal or structural mutation.
  `proc_2d14` measured the cleanup refusal;
  `proc_4ad5` passed four changed-runtime SDK cases/four injected requests.
  Complete direct disposal was not repeated,
  partial disposal retried,
  and ordinary replacement admission remained closed.
  Shutdown then host invalidation order was preserved;
  after direct disposal these callbacks observed stale extension context.
- [x] Bind the original judgment budget,
  fixed questions,
  final prepared members,
  and dependency finalization to the declared native SDK group profile.
  `proc_e427` qualified the configured consumer;
  `proc_6979` and `proc_5749` covered required freshness support and execution-checkpoint omission.
- [x] Extend native nested execution for the declared literal and string-result profiles,
  with exact parent occurrence ownership and complete enclosing-program/group linkage.
  `proc_0963`,
  `proc_408c`,
  and `proc_4ede` retain the original judgment across reached groups,
  result guards,
  cancellation,
  and native drainage.
  General dynamic orchestration and additional output modalities remain unqualified;
  these finite profiles do not become universal policy restrictions.
- [ ] Complete instruction-source collection and authority admission beyond the finite observed producer profiles.
- [ ] Finish the code-owned policy consumer without adopting diagnostic model thresholds or historical policy defaults.
- [ ] Prepare the new SDK-linked human-original confirmation once its mechanical prerequisites are ready.
  Only this step requires the genuine human input;
  historical witnesses do not supply permission.

Proposed tightening of `AGENTS.md` rule `PXQ`,
not applied because that file remains protected:

> Tracked work remains:
>  start the next authorized item after verification.
>  A missing implementation is work,
>  not a blocker;
>  stop only for missing authorization,
>  unavailable input,
>  or completion.

- Establish the composed operation and relevant effect scope before mapping nested transport groups to judgments.
  Do not infer missing safety information merely from a runtime-dependent value.
- Finish the real policy consumer and explicit nested-group orchestration seam.
  Existing private host and SDK session controls do not integrate production auto-mode.
- Extend the measured agent/session/resource-loader identity bridge to full instruction collection,
  root snapshotting,
  freshness,
  and finalization.
  The native-manager phase remains its separate frozen 0.87.1 qualification.
- Then prepare the new frozen SDK-linked configured-host confirmation,
  naming the planned not-yet-constructed root.
  Earlier witnesses are not reusable current permission.
- Only after that new original is admitted,
  perform the already-authorized disposable current-permission tests.
- Use the retained composed-effect failures to test decomposition and representation changes in new research phases.
  Completed artifacts remain unchanged;
  the live call ceiling is not a blanket research-attempt limit.
  Preserve whole-program interactions;
  do not reinstate a code-only effect-proof requirement or infer new budgets from callbacks.
- Qualify the intended pre-baked question library and adaptive selection on representative inputs,
  now including task-relevant private history where needed.
  Keep cost accounting and full-policy freshness intact.

No new user input is needed for the remaining mechanical implementation.
Ask only when the genuine SDK-linked confirmation or another actual user-only decision is ready.
