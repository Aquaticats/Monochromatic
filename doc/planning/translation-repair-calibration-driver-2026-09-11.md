# Reviewed calibration driver verification

Task 34's transport and evidence checks are complete.
Task 31's actual judge calibration has also passed;
writer and image-reader holds remain for their separate measurements.
The reference package is verified at frozen `993583ad5`;
the paid judge run has completed,
while writer and image calibration remain pending.
The driver lives in private `~/temp/agent/v41-reviewed-*.mts` files,
with `calibrate-v41-judges-20260911.mts` as its explicit-mode entry point.

## Registered inputs

The fixed manifest contains the independently read references and deliberate variants from
[the fixture plan](translation-repair-reviewed-calibration-fixtures-2026-09-11.md).
The driver compares every materialized tuple with that approved artifact before constructing questions.

The measured matrix has twenty-eight logical arrangements,
fourteen message-plus-schema questions,
and 140 model cells across the unchanged nine peers plus V4.1.
Planner-only abstentions are not model evidence.
All models' actual initial requests are checked for equality.
Message and schema positive controls move the question digest.
Equivalent direction bookkeeping is not scored twice.

Frozen output,
manifest and driver-source digests belong to the registered plan.
A live invocation rebuilds and compares the plan before constructing provider clients.
Every driver edit therefore requires a new plan.
Planner,
simulated and live ledgers use separate directories.
The previous invalid-gold driver is retained only as a `.source.txt` artifact.

## Stage depth correction

The original private two-call bound was not a valid reading of the existing stage.
The first simulation exposed identical-prompt retries.
A later source read found another boundary that the four-call correction had missed:
`package/module/translation-repair/src/stage-quorum.ts` (moved on 2026-10-18 to
`package/translation-repair/stage/src/stage-quorum.ts`)
runs its initial call and `STAGE_RETRY_ROUNDS = 3` retries,
then one separate recovery round when unreadable answers remain.
That final round appends the exported `RECOVERY_NUDGE`.

The maximum is therefore five stage calls per single-model cell,
with a 700-call stage ceiling for this fixed matrix.
This preserves existing behavior;
it neither adds a generation round nor shortens the production recovery path.
The four-call simulation proved only the quorum-loop path,
not this post-loop recovery.
It remains historical evidence rather than current launch approval.

Proposed `AGENTS.md` clarification to `TC2`,
not applied:

```text
TC2: Test names aren't coverage. Walk full control flow, including post-loop recovery and delegated retry layers.
Verify each distinct path and derive total bounds from the whole call graph.
```

The expected action is underway:
the complete stage and delegated router/client paths were read,
and the new simulation targets the previously missed recovery branch.

## Physical calls and actual provider bodies

`provider-router.ts` bounds budget-routing attempts by serving reach.
`provider-router-reask.ts` can add a schema re-ask on the direct router API,
but it is not on this production caller path:
`prompt-uniqueness-client.ts` implements `chatJson` by reading or buying raw replies through `inner.chatText`,
then applying the caller's validator.
`createRunClient` installs that wrapper.
A compiled-client simulation with all providers wet confirmed that an initial schema mismatch
was cached and retried without an OpenRouter schema re-ask.
Do not add a production re-ask merely to make an incorrect test expectation pass.

`transient-retry.ts` grants four retries after the initial HTTP attempt.
Each client's `chatText` uses that exchange ladder once.
The current model POST dispatch bound is therefore five stage calls,
times that model's serving-provider count,
times five HTTP attempts.
It is calculated per model and summed over registered cells.
Budget GETs and lower-level redirect transactions are not model POST dispatches in this counter.
The existing global deadline remains another independent bound.

The driver captures both initial and recovery bodies through the actual compiled provider clients,
using a transport that records the request and then refuses any network operation.
Capture does not change the request's completion cap or body schema.
The zero-retry capture transport is not the live retry policy.

Live transport checks are intended to require:

- A registered model cell.
- A registered serving URL and provider spelling.
- The exact compiled body for that cell's initial or recovery request.
- The existing measured completion cap and absence of prohibited reasoning knobs.
- Per-cell and global physical-call bounds.

Private request and response records omit authorization headers.
Known credential values from those headers are redacted from recorded response text.
No raw traffic or secrets may be printed.
A transport that throws before returning a body cannot provide complete wire evidence;
its missing usage is not zero cost.

## Evidence and failure interpretation

The existing clean-count and damaged-count comparisons keep the fixed peer cohort.
Do not discard incomplete peers or change their quorum after seeing responses.
Candidate coverage and positive candidate/peer evidence are reported separately from those raw count conditions.
An absent candidate must not become admission evidence merely because peer clean counts are zero.

Simulated rows are rejected by admission scoring,
and simulated run reports omit admission scores.
All role changes still require source-reviewed assessment;
no driver writes production role arrays.

Persistence and invariant failures must stop model spending.
Uncaught cell errors and aborted runs must not emit a completed report.
Expected provider absence remains distinct from an infrastructure failure:
the existing pipeline may return a valid terminal outcome with no ballot.

## Verification status

Reference verification and source guard proofs are complete.
The corrected logical driver simulation completed 140 cells with 170 fake calls,
but it bypassed the physical transport wrapper.
It does not establish transport correctness.

The next verification uses the actual compiled router,
clients,
prompt cache and parsers,
replacing only physical transport.
It must exercise the existing HTTP retry ladder,
Hyper payment refusal followed by OpenRouter budget failover,
cached stage retries and final recovery nudge,
then verify one ballot per model identity and no simulated admission score.
The first compiled-client run returned all 140 cells with 144 stage calls,
including the nudge,
but its expected schema re-ask assertion was wrong.
That invocation is not the completed transport verification.

The corrected simulation and negative controls pass against
`~/temp/agent/v41-reviewed-plan-20260911-9eo8in/plan.json`.
Its plan digest is `2df23dc9469dc2a65a21654ac782fe26e2af3f547359723f4f6fcc886b12cf1e`;
its driver digest is `c8498a4d4ca35374cce8121aee0e70e69d5c9b63938c6fd6f920efe71b818daa`.
The compiled-client simulation returned 140 cells,
144 stage calls and 146 simulated model POST dispatches across all four gateways.
It verified the unchanged HTTP retry ladder,
cached stage retries,
final nudge,
Hyper-to-OpenRouter budget failover and one ballot per identity.
It emitted no admission score.
Evidence:
`~/temp/agent/v41-reviewed-simulation-20260911-LZgjFd/wire-simulation-verification.json`.
The controls separately exercise wrong route,
wrong body,
zero global and per-cell model-POST allowances,
a bound-of-one positive control,
per-cell and global bound overflow after one dispatch,
preexisting request-record refusal,
an uncaught post-call error and genuine provider absence.
They require preserved sentinel bytes,
expected delegated-call counts,
nonzero child exit for failed operations,
no score on simulated reports and incomplete rather than completed evidence on failure.
`~/temp/agent/v41-reviewed-boundary-verification-r2-20260911.json`
records all eleven passing positive and negative cases.
Failed operations exit nonzero and persist incomplete reports;
the provider-absence control returns an empty ballot without falsely becoming an infrastructure failure.
Every record is simulated and no score is attached.
These results were read before closing task 34.
No real model call occurred in either verification.

## Paid launch

Task 31 started at 2026-09-11 14:13:40 UTC using the verified registered plan.
`proc_02bc` owns wrapper PID 2149953 and Node PID 2150064.
State is `~/temp/agent/v41-reviewed-run-20260911-vbOoMQ`;
output is redirected to `~/temp/agent/v41-reviewed-judge-calibration-20260911.log`.
The live provider-budget snapshot marks all providers wet.
Launch and a returned score alone changed no production role.
The terminal report,
all individual ballots and cost/provenance records were subsequently read.
V4.1's fourteen clean picks and no damaged pick satisfy the pre-registered comparison
against a peer median of fourteen and maximum damaged count of one.
Judge-only admission and post-change verification are recorded in
[the seating decision](../decision/translation-repair-roster-seating-2026-09-01.md).
