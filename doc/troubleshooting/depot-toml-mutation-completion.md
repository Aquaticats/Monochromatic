# Depot TOML mutation completion

## Observed timeout and diagnosed boundary

Development run `p9r4xl6n12` canceled container attempt `ltfw0kr3cd` after its 30-minute job limit.
The file-enforcer container check had already failed after `79620.70287899999` milliseconds (`#1070`).
TOML mutation started and had no terminal result;
independent file-enforcer mutation never ran.
The [TOML audit](../audit/depot-toml-reconciliation-2026-10-17.md) retains exact identities and provider usage.

At frozen source `525d2e09f8168a3bdd7e513e26d4c80b363123e5`,
`package/mutation-test/cli/src/host/cli-options.ts` permits each container session to last `1800` seconds.
`package/mutation-test/cli/src/host/orchestrate.ts` can schedule further sessions and retries.
The original trial executor waits for child exit without a per-check deadline,
buffers output to check logs,
and sequences independent gates in one container job.
The outer job budget includes setup and the preceding check.
Cancellation skipped active-log upload and the final source immutability proof.
The mutation campaign's internal cause is not established.

## Trial continuation and evidence retention

The isolated workflow `.depot/workflows/depot-complete-toml-bounded.yml` on
`trial/depot-toml-continuation-20261017` preserves source tasks and flags.
Complete scope runs the original core/generated consumers plus independent container and mutation jobs.
Development mutation-only scope omits the already completed container failure.
The mutation matrix disables fail-fast.
A check deadline or source failure cannot cancel its independent sibling.

The wrapper budgets one consumer for `3600000` milliseconds in a `90`-minute job.
It retains incremental output and correlated container state/events,
sends SIGTERM and then SIGKILL to the owned process group,
and kills/removes only containers carrying its random private label.
Container shutdown operations have independent `60000`-millisecond budgets.
A timed-out started gate remains timed out;
a gate never reached behind a failed prerequisite stays unexecuted.
Cleanup or instrumentation failures remain failed harness outcomes.

The frozen mutator's terminal formatter lists totals,
survivors,
and timeouts,
while complete observed identities reside in shard manifest/report JSON.
The wrapper sets a private `TMPDIR` outside source and retains those raw files and hashes.
The source command receives no extra report flag or changed assertion.
Source identity proofs retain HEAD,
tree,
tracked status,
tool/task hashes,
and available Cargo inputs
before and after consumers,
tied to shared cohort/hash/request-time inputs.

## Verified controls and limits

Exact-source positive controls and a mutated-source rejection control pass.
Both local fixture overrides reject CI mode.
The real container timeout control records `2147483648` memory bytes,
`2` CPU quota,
and `64` PID capacity.
SIGTERM-resistant child and grandchild processes terminate,
and the correlated container is removed with its engine events retained.
A normal source exit `0` passes;
an exit `7` remains failed.
Synthetic shard identity files survive and match their retained hashes.

The initial short cleanup budgets left stopped containers present and correctly failed the controls.
Separate `60000`-millisecond shutdown budgets passed the same boundary.
The cause of Podman's shutdown acknowledgement delay is not established.
Consumer,
process-signal,
shutdown-operation,
and total wrapper intervals are separate evidence fields.
Local controls prove the adapter behavior;
they do not prove source mutation success.
Issue `#1113` remains open pending actual retained consumer outcomes.
Final coherent dispatch is held.

## Default native report

The full CLI emitter writes `cwd()/mutation-report.json` after orchestration.
The TOML source task changes into its monorepo root before invoking that CLI.
The prepared wrapper copies available default reports into the existing external evidence directory.
It hashes each copy and records source mtime and whether the file existed before the consumer.
A report unchanged from before the consumer remains explicitly stale.
This preserves the original command and all assertions.

A native-report positive fixture and a stale-report marking control pass.
The development request predates this copy adapter.
Its separate read-only `depot sandbox exec` observer uses the original VM session and a native command timeout.
The VM `/bin/pwd` probe succeeded;
the observer retained the complete native report with a verified digest.

## Final source guard scope

The [source guard controls](../audit/depot-route-source-guards-2026-10-17.md)
verify a separate final-harness repair under existing issue `#1114`.
The guard rejects both Git index exclusions across tracked source
and compares configured metadata bytes directly to `HEAD`.
Its inventory includes root `.cargo/` configuration and every tracked Cargo manifest and lockfile,
including nested Android Rust metadata.
Untracked and ignored output remains outside this inventory.
The development mutation run retains its original harness identity.

## Actual retained infrastructure outcome

Development run `p6vm5s00w6` completed its independent mutation commands.
File-enforcer failed at its existing package-path `ENOENT` boundary.
TOML retained `3062` distinct mutant identities across `9037` shard attempts,
all marked `runtimeError` after source copying failed before assertions.
Each shard reports `rsync` exit `23` and
`rsync: [generator] chgrp "/work/." failed: Operation not permitted (1)`.
This is the diagnosed root-owned work-tmpfs boundary,
separate from issues `#1116` and `#1117`.

The wrapper did not time out.
Its final cleanup proves no owned process group or labeled container remains.
The combined provider interval is `2834043` milliseconds at `16x64`.
The full native report is retained as compressed byte-identical evidence in
[the terminal audit](../audit/depot-toml-reconciliation-2026-10-17.md#terminal-toml-infrastructure-outcome).
These infrastructure-derived statuses do not establish source mutation completion or passing tests.
The complete current container adapter was exercised by the bounded actual TOML pilot
described in the consumer recovery section.

Pilot `0j8zhbn9n0` now exercises that complete adapter through the unchanged source task
with one existing source-file selector and the full unit suite.
Its actual source command completed with exit zero;
the wrapper metadata failure remains separate.

## Consumer recovery and metadata observation failure

Actual pilot `0j8zhbn9n0` ran the unchanged one-file source mutation task
through the complete containers adapter from `d098532a5b6946231daf3c4f43e38444359a307e`.
Its baseline is green and the only selected mutant is `compileError`.
This source-terminal result carries no mutant assertion execution credit.
It verifies the owned work volume and unchanged generated-metadata bind on a real TOML consumer.
It does not complete the 3062-identity campaign.

The provider job failed when metadata collection asserted `null !== 0`.
That harness omitted the operation,
signal,
error code,
and stdout/stderr byte counts,
so a timeout is not yet a diagnosis of the historical operation.
The replacement receipt harness logs those fields incrementally before asserting success.
Its disposable timeout control captures `SIGKILL`,
`ETIMEDOUT`,
and both output streams.
Actual repeat `mj128hqwv0` preserves the eight-second operation budget and original source command.
Collect that result before selecting a remedy or retrying the full campaign.
The [TOML audit](../audit/depot-toml-reconciliation-2026-10-17.md)
retains exact requests,
source/harness revisions,
cleanup intervals,
and provider usage.

## Inherited method lookup emits invalid replacement values

Issue [#1118](https://github.com/Aquaticats/Monochromatic/issues/1118)
is a distinct source producer defect in `package/mutation-test/cli/src/engine/operator/chain-method.ts`.
The unchanged source lookup accepts inherited Object functions as method-swap replacements.
A real source consumer proves mapped-string and unmapped negative controls,
then function-valued `toString`,
`constructor`,
and `hasOwnProperty` replacements.
JSON omits the function-valued replacement property on three retained cloud candidates.
This does not explain the container source-copy failure;
all of those earlier attempts failed before mutant assertions.
The audit retains exact candidate identities and the verified issue body.
No source-body remedy was applied.

## Exact observer deadline receipt

Actual repeat `mj128hqwv0` retains the failing `container-bin/podman ps --all --filter label=... --format json`
operation with null status,
`SIGKILL`,
`ETIMEDOUT`,
three stdout bytes,
and zero stderr bytes.
It observed `8006.465648000001` milliseconds against an `8000`-millisecond deadline.
Stdout `[]\n` establishes output bytes only.
The Node forwarder,
its synchronous sudo child,
and actual Podman exit remain distinct lifecycle boundaries.
The source command still completed with exit zero,
green baseline,
and the same terminal compilation result.
Both original eight-second failures remain retained.
The next bounded trial may extend the metadata-only deadline while preserving source commands,
timeout rejection controls,
complete operation durations,
and scoped cleanup.

## Verified bounded observer recovery

Trial `vlrxjltfkq` finished successfully with the unchanged source consumer,
the same terminal compilation identity,
all ten metadata results,
source Podman correlation,
matching source identities,
and scoped cleanup.
The list observer took `14100.441549` milliseconds once;
its synchronous sudo forwarding span was `14077.079173` milliseconds and returned zero.
The 60000-millisecond bound covers this actual delayed call.
The underlying Podman-versus-sudo cause remains unresolved.
The Node observer's two lifecycle additions are byte-separated from the unchanged consumer forwarder.
Issue [#1130](https://github.com/Aquaticats/Monochromatic/issues/1130)
retains both original eight-second failures and the verified timeout rejection/removal controls.
This recovery permits a full source mutation retry;
the bounded compilation result alone does not complete the campaign.

## Full campaign continuation

The verified bounded recovery is followed by development run `23dk3nr90f`.
It executes only the original full TOML mutation task,
with unchanged source defaults.
The source wrapper permits five hours,
with a 360-minute provider job budget.
This is a declared execution allowance,
not a throughput prediction.
Original failed usage and source incidents remain retained.
The next evidence requirement is complete source,
shard and mutant identity reconciliation,
including actual live resource samples and scoped cleanup.
Observer issue `#1130` is closed after the bounded actual recovery;
full-campaign issue `#1113` remains open.

## Selected files and actual mutation test execution

Source `525d2e09f8168a3bdd7e513e26d4c80b363123e5`
uses `package/mutation-test/cli/src/container/test-run.ts` to run selected files in order.
It stops at the first failure or timeout.
The 37-file full-suite selection is therefore a candidate list,
not a claim that every test file executes for every mutant.
Compiler rejections receive no mutant assertion credit.

`runOneTest` also returns `failed` after a Node spawn exception.
The source mutant loop classifies that result as `killed`.
Do not infer an executed assertion from this native label alone.
Retain actual test output and inspect complete logs for spawn diagnostics.
A separate actual-source disposable control is being prepared to test this classification boundary.
Its preparation does not establish that the active cloud run encountered the failure.

## NanoSpawn inherits an eval carrier

Observation repeat `j03l179f3x` retained the failed fixture typecheck detail:
`Error: EEXIST: file already exists, mkdir '/work/control-bin'`.
The stack points into the control's eval body,
rather than a compiler diagnostic.

Installed `nano-spawn` 2.1.0 implements this behavior in
`node_modules/.pnpm/nano-spawn@2.1.0/node_modules/nano-spawn/source/spawn.js`.
For a literal `node` command it uses `process.execPath`
and prepends the parent's `process.execArgv`.
An inline eval caller therefore re-enters its own eval body when it requests a Node file child.
This explains the disposable control's repeated directory creation.
It does not establish a failure in the separate full mutation campaign.

Actual local controls prove the distinction:
a file caller executes its requested child,
while the same eval caller re-enters the guarded carrier.
The negative fixture has an environment recursion guard and cannot recursively fan out.
The revised cloud control mounts an actual TypeScript module entry file
and verifies an empty `process.execArgv`.
The immutable source dispatcher in root `mise.toml` also calls Node with a file entry.
No application source body was changed.

File-based development control `699f9lg5zm` is dispatched on
`f636b13af902ed6abe5cd8758da4cb8ba99f2175`.
It removes only a disposable PATH symlink named `node` after its green baseline.
NanoSpawn retains its actual absolute Node executable for the compiler.
The actual cloud classification result remains pending.

## Full immutable-source completion

Development run `23dk3nr90f` completed successfully at unchanged source
`525d2e09f8168a3bdd7e513e26d4c80b363123e5` and harness
`0f1ef041a0de2ff6d2db6dd55210b1160c592648`.
The historical wait session `96306` has been reaped with exit zero.
All `3062` selected identities have native terminal outcomes:
`921 killed`,
`256 survived`,
`3 timeout`,
and `1882 compileError`.
Native `runtimeError` and final infrastructure-error counts are zero.
These are native runner classifications;
`killed` does not independently prove an assertion ran.
Compiler outcomes receive no mutant assertion credit.
The source CLI reports raw totals without a mutation score or threshold.

All `463` physical manifests and reports reconcile against `3333` assignments.
The `32` unrun prefix assignments have subsequent final dispositions.
Every selected identity matches its frozen source span,
recomputed native ID,
and a matching result in a physical report with a passing baseline.
The candidate full suite contains `37` files;
the unchanged runner stops after its first failed or timed-out file.
It does not execute every candidate file for every mutant.

The source task took `4392514.76837` milliseconds;
the wrapper took `4394121.711807` milliseconds.
The provider interval was `4498869` milliseconds at `16x64`,
or `71981904` allocated vCPU-milliseconds.
This independently requested development job is not complete-cohort latency.
Its result envelope names the broader case inventory,
but its actual check overlay and result execute only TOML mutation.

The source log is `41142330` bytes with SHA-256
`cb31ec5955c4d97ba6ce6f0b97a461dba37a8cc16536273f4a471f8c45b667c3`.
It has no matching Node-spawn diagnostics.
That observation is limited to `runOneTest`'s caught-spawn-error branch and `rl.error` emitter;
the actual-source emitter control subsequently passed in `rsbr0wrdt0`.
It does not upgrade native `killed` into literal assertion credit.

Of `1302` metadata operations,
four auxiliary inspections returned Podman status `125`
and `Error: no such object`,
with exact arguments,
times,
byte counts,
and stderr retained.
They remain separate from the zero final native infrastructure errors.
The events command succeeded with empty output;
this does not prove complete event retention or establish an inspection failure's cause.
Periodic sampling recorded `858` observations of `460` distinct containers across `441` samples.
All available bounds were `2 GiB`,
`2 CPU`,
and `512 PIDs`;
the four unavailable inspections remain in the sample record.
All `463` original source run commands retain their declared bounds.
All `463` named work volumes have paired successful cleanup and absence receipts,
and all `463` stopped metadata containers have successful removal and absence receipts.
Owned metadata was byte-identical and read-only;
source image identity remained unchanged.
Final scoped containers and process groups were empty.

The complete provider artifact is retained in
[the primary ZIP](../audit/depot-toml-full-mutation-2026-10-17.evidence.zip),
`12057855` bytes with SHA-256
`31e45cf2ad99de72d9253c70e1057f050f985c617c1d69a2ee3402f5ec1fa5e3`.
Its `2337` member hashes are retained in the structured audit.
[The compressed reconciliation](../audit/depot-toml-full-reconciliation-2026-10-17.evidence.json.gz)
retains every identity,
assignment,
report mapping,
source span,
metadata duration,
resource sample,
cleanup receipt,
exact provider usage,
and the reconciliation code.

The independent file-carrier classifier control `699f9lg5zm` failed an ancillary
hardcoded NanoSpawn metadata lookup before mutation assertions.
Its source prerequisite,
exact `ENOENT`,
before/after identity,
cleanup,
and provider usage are retained.
No source classifier result is claimed from that failure.
Repeat `rsbr0wrdt0` uses actual package resolution at harness
`f650e3e2f923bea21a7baf9565bb1ec3efb4fbde`,
with historical terminal wait session `43521`,
now reaped with exit zero.
The final common cohort remains held.

## Actual source spawn classification control

Development control `rsbr0wrdt0` completed successfully at harness
`f650e3e2f923bea21a7baf9565bb1ec3efb4fbde`.
The unchanged source modules match frozen `525d2e09f8168a3bdd7e513e26d4c80b363123e5` byte-for-byte.
The real compiler passed before and after the disposable PATH Node launcher was removed.
The genuine assertion control executed the fixture with value `2` and returned `killed`.
After Node removal,
the unchanged mutant loop again returned `killed` with no anomaly,
but its test-entry marker did not change.
Direct `runTests` returned `kind: failed` and
`spawn error: Error: spawn node ENOENT`.
This demonstrates false mutation kill credit in the source classifier.
It is independent of whether the completed cloud campaign encountered that failure.

The actual source `runOneTest` catch emitted both expected `rl.error` diagnostics:
`spawn failed for subject.unit.test.mjs: Error: spawn node ENOENT`.
The completed campaign's hash-verified full source log contains zero matching diagnostics at that branch.
This observation does not prove that every native `killed` reached an assertion,
or exclude other failures before an assertion.
The `921` campaign outcomes therefore remain native runner classifications.
The source's early-stop positive and negative controls and its timeout control passed.
Source restoration,
unchanged image identity,
before/after source identity,
and scoped cleanup all passed.

The actual fixture interval was `1331` milliseconds inside `2 GiB`,
`2 CPU`,
and `512 PIDs`.
The entire provider job took `165855` milliseconds at `16x64`,
or `2653680` allocated vCPU-milliseconds.
The unchanged bounded source prerequisite remains a single compiler outcome with no mutant assertion credit.
The first three unsuccessful control attempts and their provider usage remain retained.
Terminal wait `43521` is reaped with exit zero.
The actual-source control artifact is retained in
[the primary control ZIP](../audit/depot-toml-spawn-control-2026-10-17.evidence.zip).
No application source was changed.

## Original selected-check terminal accounting

All `24` originally selected task commands now have retained actual terminal outcomes:
`20 passed` and `4 failed`.
This joins historical independent development evidence for coverage accounting only.
The original canceled run remains incomplete in its original record.
The final common cohort remains held.
File-enforcer mutation ran and failed before enumeration with its original `ENOENT` (`#1068`);
it receives no mutant assertion credit.
The original completed file-enforcer container failure remains failed (`#1070`).
The independent actual-source classifier defect is now
[issue `#1142`](https://github.com/Aquaticats/Monochromatic/issues/1142),
verified open with `needs-triage`.
No application source fix was made.
The structured `historicalIndependentCoverage` retains each selected identity,
its exact source task,
its original disposition,
and its actual terminal receipt.
Issue `#1113` is closed against that retained coverage,
with its exact closing comment and closed state re-fetched.
The source failures and native outcome limitations remain explicit.

The final closure receipt is
[comment `6232394931`](https://github.com/Aquaticats/Monochromatic/issues/1113#issuecomment-6232394931).
Source classifier issue `#1142` remains open.
Final route preparation remains unchanged;
complete frozen case bytes and actual canonical retrieval are still required before coherent dispatch.
