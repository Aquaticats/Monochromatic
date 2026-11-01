# Depot rolling-budget evaluation

## Recommendation and scope

Keep Scorecard scheduling in its existing GitHub workflow during the Depot trial.
The GraphQL repair preserves the current admission decisions in supplied fixtures
and queries live history without REST history calls.
Its exact script passed with the workflow's own token in run `38831327297`.
The user subsequently requested repair of the quota-related download failure and reiterated the REST-quota objective;
the narrow query replacement landed in `f598404b31` under that repair request.
The production gate then passed in [run `38831778986`][graphql-production-job],
reading 972 historical runs with ten GraphQL requests and no REST history requests.
It skipped analysis at the eight-per-day count.
No migration to a different scheduler has been validated.
This is a recommendation to defer scheduler migration,
not a claim that the current gate correctly enforces every execution limit.
The gate has measured overhead and correctness gaps described in this evaluation.
The [trial plan](depot-ci-trial.md) authorizes evaluation,
not production adoption.

Current provider documentation and public source were checked on October 15,
2026.
This evaluation compares mechanisms against the established requirement;
it does not select a new scheduler vendor or approve a custom controller.
Ranking:
repair the existing history query before migrating scheduling,
because the former has an executable candidate preserving observed policy
while the latter still lacks admission and recovery evidence.

## Required behavior

The [Scorecard workflow](../../.github/workflows/scorecard.yml) specifies
2 analyses per rolling minute,
4 per rolling hour,
and 8 per rolling day.
Excess events are skipped rather than retried after capacity returns.
The [accepted trial context](../handover/depot-trial-grilling.md)
also permits cancelling superseded intermediate checks.
Neither permission removes the rolling limits or authorizes replacing them with a periodic schedule.

A replacement must evaluate all windows together,
preserve the selected commit and triggers,
and make concurrent admission decisions without spending the same capacity twice.
For example,
after two counted executions at 12:00:10 and 12:00:20,
12:00:30 must be rejected even if neither execution is still running.
At 12:01:10,
the first execution falls outside the minute window,
but the hour and day windows still apply.
This example follows the current strict `ageSeconds < seconds` comparison.

The future workload includes newly wired checks and event bursts from agent work,
not just today's successful Scorecard analyses.
Rejected events must therefore be part of any overhead measurement.
An implementation that scans every rejected event can become expensive even when analysis execution remains bounded.

## Existing implementation and measured overhead

The REST baseline inspected before the query repair runs on `ubuntu-latest` with `actions: read`.
Job concurrency serializes the gate under `scorecard-rate-limit` with `cancel-in-progress: false`.
The gate lists workflow runs using a `created` filter,
then awaits a separate jobs request for each visited run until a budget is exhausted.
It counts an analysis when its conclusion is neither `skipped` nor `cancelled`.
Analysis starts in a dependent job after the gate completes.
These observations come from [the workflow source](../../.github/workflows/scorecard.yml).

The completed [gate job 116669815274][gate-job] provides a measured example:

- The job ran from 21:03:07 to 21:08:41 UTC,
  or 334 seconds.
- `Count recent analysis runs` ran from 21:03:09 to 21:08:40 UTC,
  or 331 seconds.
- Its log reported 922 workflow runs and emitted 790 per-run job-query results before finding eight counted analyses.
- It then rejected the current analysis because the day budget was exhausted.

These timings come from the completed jobs API response,
and the counts come from the complete job log fetched with `gh api`.
They replace the handover's preliminary observation that the step was still running after three minutes.
The handover's separate 926-run history query used a different capture interval;
it is not this job's fetched-run count.
This is one completed observation,
not a latency distribution or proof that every second was network latency.
It does establish that the measured delay was predominantly inside the counting step,
after setup.

The disposable `replay-budget` task extracts the production script and injects fixed time and fake GitHub responses.
All 12 cases passed again during this evaluation.
The [retained replay results](../audit/depot-trial-2026-10-15.evidence.json) cover empty history,
each exhausted window,
exact minute expiry,
failed,
cancelled,
skipped,
running,
and absent analysis jobs.
A 926-run skipped fixture issued 926 sequential job-history calls.
The fixture proves request shape and decisions under its supplied responses;
it does not measure live API latency.

## Correctness boundaries

The fixture also exposes a missing admission reservation:
requests that see absent analysis jobs both pass.
GitHub serializes only the gate job,
while the counted analysis is a different dependent job.
There is no persisted reservation between the decision and the downstream job becoming visible.
This is a modeled gap,
not a demonstrated live budget violation.
Making the executor faster does not establish that this gap disappears.
[Source](../../.github/workflows/scorecard.yml)
and [fixture evidence](../audit/depot-trial-2026-10-15.evidence.json).

Additional source and API boundaries require explicit acceptance tests before claiming exact execution accounting:

- Window age uses workflow `created_at`,
  not analysis start time.
  Long queueing can therefore make an analysis appear older than its execution.
- A cancelled analysis is excluded even if it performed work before cancellation.
  Permission to cancel superseded checks does not establish that consumed work should be refunded.
- `filter: latest` observes the most recent job execution rather than all rerun attempts.
  [GitHub jobs API](https://docs.github.com/en/rest/actions/workflow-jobs#list-jobs-for-a-workflow-run).
- GitHub documents a 1,000-result search limit when filtering workflow runs by `created`.
  Pagination alone therefore does not establish a complete day of history once that limit is exceeded.
  No over-limit live sample was tested here.
  [GitHub runs API](https://docs.github.com/en/rest/actions/workflow-runs#list-workflow-runs-for-a-workflow).

The first two observations follow directly from [the gate source](../../.github/workflows/scorecard.yml).
The replay did not exercise GitHub visibility timing,
reruns,
history truncation,
or restart recovery.
Those gaps prevent treating its passing cases as a complete admission proof.

## GraphQL query repair

The user subsequently requested repair of failures associated with organization transfer
and reiterated the objective of avoiding the shared REST quota.
The isolated candidate in commit `024cbb224e` changes only Scorecard's history read.
Commit `2700e15025` corrects its concurrency comment to acknowledge the existing reservation gap.
It resolves the workflow by URL through GraphQL `resource`,
then requests `Workflow.runs` ordered by creation time in pages of 100,
with the latest check named `Scorecard analysis` nested in each run.
This removes the separate per-run REST jobs request and requires no REST call to discover a workflow identifier.
The live probe verified this exact query through GitHub's API;
the [GraphQL object reference][github-graphql-objects] documents the fields.

The candidate deliberately retains the current policy:
workflow creation time determines window age;
failed and unfinished checks count;
skipped and cancelled checks do not;
an absent analysis does not reserve capacity;
and the latest matching check represents reruns.
The original triggers and analysis job remain unchanged.
This repair therefore does not resolve the accounting boundaries in Correctness boundaries.
The baseline's REST filtered-search ceiling is replaced by an explicit 32-page bound.
If history is incomplete at that bound,
the candidate throws without producing an allow output.
It also refuses admission for malformed history,
unordered dates,
incomplete check connections,
nonadvancing cursors,
API errors,
or fewer than 100 remaining GraphQL points.

The committed isolated task at `cebc724b44` extracts and executes both the old main-worktree script
and the actual candidate YAML script with fixed time and supplied API responses.
All 13 named policy comparisons and 32 deterministic variations produced matching admission decisions.
All 11 incomplete-history and quota cases rejected admission.
The [retained fixture results][graphql-fixtures] include each named policy comparison.
For the 790-run fixture,
the baseline made 790 per-run jobs requests and the candidate made eight GraphQL requests.
The baseline fixture count excludes its initial workflow-list request.
For 926 skipped runs,
the corresponding counts were 926 per-run jobs requests and ten GraphQL requests.
These are request-shape measurements under identical supplied history,
not live latency measurements.

The quota test was checked with a negative control:
temporarily disabling the candidate's `remaining < 100` guard caused the low-quota assertion to fail.
Restoring the guard returned the complete fixture task to passing.
Only this guard received a removal control;
the other rejection cases were exercised without separately removing their guards.

The exact candidate script's read-only live probe at `2026-10-16T00:41:24.221Z`
visited 959 historical runs,
used ten GraphQL requests and ten points,
made zero REST requests,
and rejected the event after counting eight day-window analyses.
Elapsed time was 33.2418 seconds.
The [retained live result][graphql-live] is a single sample against changing history
using the local authenticated CLI token.
It is not a controlled latency comparison with the 331-second baseline
and does not establish a billed-cost reduction.

A separate [workflow-token query probe][graphql-token-probe] passed with `actions: read`,
`checks: read`,
and `contents: read`.
The candidate uses those same permissions.
The exact candidate script then passed in
[the consumer-boundary trial](https://github.com/Aquaticats/Monochromatic/actions/runs/38831327297/job/116750532503).
It visited 966 historical runs using ten requests and ten GraphQL points,
made no REST history requests,
and correctly skipped analysis at the existing eight-per-day count.
The committed replacement then passed in [production run `38831778986`][graphql-production-job]
at head `0baff85defe4109be16a1ba15e96712d8a85b49a`.
The gate visited 972 historical runs using ten GraphQL requests and ten points,
made no REST history requests,
and skipped analysis after counting eight analyses in the day window.
The jobs API records the counting step from `00:51:04` to `00:51:29` UTC on October 16,
2026,
or 25 seconds at that API's timestamp precision.
The gate job succeeded and its dependent analysis job was skipped.
The [retained production evidence][graphql-production-evidence] contains the run and job identifiers,
timestamps,
and credential-free runtime log excerpts.
This is one production observation against changing history,
not an unchanged-input latency comparison or proof of billed savings.
The observed production branch is day-budget rejection;
the local fixtures exercise admission and the other budget branches.
The isolated candidate is not a Depot migration or a new scheduler.

A stricter experimental counter used actual step activity,
including cancelled attempts and reruns,
but that would change the current policy.
Its 27 local fixtures passed;
its live run stopped on HTTP 502 after nine completed pages and made no admission decision.
The source does not expose an updated-time ordering or filter for this workflow-runs connection,
so establishing old workflows rerun recently would require scanning their older history.
That experimental policy is not the recommended query repair and has not been adopted.

## Native controls evaluated

### GitHub concurrency and Depot GitHub runners

GitHub concurrency controls active and pending jobs or workflows.
The default retains one pending item and replaces it when another arrives.
Current documentation also offers `queue: max` for up to 100 pending items.
Neither control counts starts in the preceding minute,
hour,
and day.
The two-completed-executions example in Required behavior is outside concurrency's active-work accounting.
[GitHub concurrency documentation][github-concurrency].

Depot GitHub runners replace the execution machine after GitHub emits a `workflow_job` event.
They do not replace this workflow's admission script or its GitHub history queries.
The documented integration is a runner-label change;
this evaluation contains no measurement showing that moving the gate to a Depot runner reduces its counting time.
[Depot runner architecture](https://depot.dev/docs/github-actions/overview).

### Depot CI

Depot documents workflow and job concurrency,
cron,
push,
and dispatch triggers.
It explicitly lists `on.branch_protection_rule` as unsupported,
although the current Scorecard workflow uses that trigger.
A complete migration must account for that event as well as the budgets.
[Depot compatibility reference][depot-compatibility].

Depot's orchestration article explains that a concurrency-group completion wakes waiting workflows.
It documents suspended orchestration between callbacks,
not a user-configurable rolling admission counter.
[Depot orchestration](https://depot.dev/blog/building-ci-with-durable-lambda).

The public `CIService` source exposes run,
dispatch,
retry,
cancel,
and inspection operations.
Its `RunRequest` and `DispatchWorkflowRequest` contain no rolling-window budget or reservation field.
This was checked at `depot/cli` revision `1f96d6d488f938de97ea640b115e9567ebfd6fbf`.
[CI API source][depot-ci-api].
Together with the compatibility reference,
this establishes no validated public configuration for this requirement;
it does not prove that Depot has no internal capability or future extension.

Copying the existing gate into Depot CI would retain queries against GitHub workflow history.
Those queries do not account for Depot CI executions.
Replacing them with Depot list calls would still require a separately validated admission design.
The API source describes `ListRuns` as recent discovery,
not historical search;
`ListWorkflows` has a bounded result size without a pagination token.
[CI API source][depot-ci-api].

### Depot sandboxes and periodic execution

The Sandbox SDK documents creation,
commands,
listing,
termination,
and lifetime controls.
Its creation options do not include rolling admission rules.
A process kept alive to maintain counters would be application scheduling code,
not a native policy established by these APIs.
The documented maximum sandbox lifetime is 24 hours,
so persistent accounting would also need a state and restart design.
[Sandbox SDK](https://depot.dev/docs/api/sandbox-sdk-reference).

A cron-only replacement would change when checks respond to pushes and branch-protection changes.
It would not by itself account for manual runs,
retries,
or actual start times after scheduling delays.
Depot documents independent UTC cron scheduling without a start-time SLA.
Periodic execution is therefore not an equivalent implementation of the accepted event-driven policy.
[Depot schedule behavior][depot-compatibility].

## Comparable existing mechanisms

The repository already uses concurrency in other workflows,
including [Alpine checks](../../.github/workflows/music-player-alpine.yml)
(moved into the `test-alpine` job of `.github/workflows/music-player-release.yml` on 2026-10-22).
Its [debounce utility](../../package/upstream-fork/conf/src/debounce.ts) waits for a quiet period;
its [p-limit fork](../../package/upstream-fork/p-limit/src/index.ts) bounds simultaneous calls.
Neither source defines shared rolling execution accounting.
They are related incumbents,
not replacement schedulers demonstrated by this trial.

The primary-source comparison was widened beyond Depot to avoid mistaking an empty native-feature search for
the absence of an established mechanism:

- Inngest documents skipping excess events before function starts,
  matching the desired rejection behavior.
  Its rate limiter uses the generic cell rate algorithm and exposes a limit and period.
  That documentation does not establish an atomic combination of the required exact historical windows.
  No equivalent-policy execution was tested.
  [Inngest rate limiting](https://www.inngest.com/docs/durable-execution/flow-control/rate-limiting).
- Upstash's TypeScript `slidingWindow` is explicitly a weighted approximation that assumes uniform arrivals
  in the preceding fixed window.
  Its name alone is insufficient evidence for exact rolling counts during bursts.
  [Upstash algorithms](https://upstash.com/docs/redis/sdks/ratelimit-ts/algorithms).
- Upstash also documents a timeout that can allow a request regardless of the current limit.
  Any future evaluation would need to account for that failure behavior explicitly.
  [Upstash timeout behavior](https://upstash.com/docs/redis/sdks/ratelimit-ts/features#timeout).

These examples establish useful precedent for admission before compute starts,
but neither is an approved dependency or a validated replacement.
No service was provisioned and no custom scheduler was built.
Vendor selection would require the repository's full technology evaluation process and execution evidence.

## Evidence scope and adoption conditions

Local inspection covered `.github/`,
root `mise.toml`,
the trial plan and handover,
retained trial evidence,
and related utilities under `package/module/`.
The workflow search included `rate.limit`,
`rolling`,
`debounce`,
`concurrency`,
`TODO`,
`FIXME`,
`workaround`,
and `suppress`.
It found the Scorecard history workaround and other concurrency controls;
it did not establish another CI budget owner.
An initial search included a nonexistent main-worktree `.depot/` path;
the existing paths were searched again successfully without that path.
This is a scoped inventory,
not a repository-wide claim that no scheduling utility exists.

External queries covered Depot's compatibility reference,
CI API,
Sandbox SDK,
runner architecture,
and orchestration article;
GitHub concurrency and run/job APIs;
and Inngest and Upstash admission documentation.
The generated Depot CI API web page exceeded the browsing fetch limit,
so the public CLI's owning protocol definitions were fetched through `gh api` instead.
No conclusion relies on that failed page fetch.

Adoption remains gated on an implementation that demonstrates the requested windows together,
including overlapping arrivals and exact boundary expiry;
documents what counts when work is cancelled or retried;
preserves required triggers;
and survives state loss or incomplete provider history without silently admitting excess work.
Measure admission separately from execution using unchanged-input repetitions,
including a rejected-event history comparable to the observed workload.
Record request count,
elapsed spread,
and actual billed usage before describing the replacement as inexpensive.

Until that evidence exists,
the scheduling portion of the trial concludes **no validated scheduler replacement**.
The GraphQL repair has passing local policy comparisons,
live read-only queries,
and exact-script workflow-token execution.
It is committed as the quota repair;
its production gate completed successfully and skipped an over-budget analysis.
The repair does not adopt a scheduler replacement or correct the separate accounting-policy gaps.
The measured 331-second counting step does not justify relaxing the accepted budgets.

## Verification

`mise run replay-budget` passed all 12 fixtures.
The scoped `lint:monochromatic-lint` task passed,
and the Pandoc-rendered HTML was inspected for headings,
lists,
links,
code spans,
and the emphasized conclusion.
`git diff --check` passed.
The research was first committed as `22c3f3d65e` before document verification.

The GraphQL repair evidence was committed as `56119c773e` before document verification.
Scoped `lint:monochromatic-lint` passed for the updated evaluation and handover.
Pandoc rendering of both documents was inspected for the query-repair prose,
code spans,
evidence links,
and workflow-token result link.
`git diff --check` passed.
The isolated candidate had no tracked modifications after restoring the quota negative control.

Production evidence was committed as `2ac43c7cbd` before document verification.
Scoped `lint:monochromatic-lint` passed for this evaluation and the updated quota troubleshooting record.
Their Pandoc-rendered production-result paragraphs,
links,
and query excerpt were inspected.
The retained JSON parsed successfully;
its runtime excerpts matched the privately downloaded complete job log,
and the logged page count and summed costs both matched ten.
The recorded API timestamps yielded 25 seconds.
`git diff --check` passed.

[gate-job]: https://github.com/Aquaticats/Monochromatic/actions/runs/38806855827/job/116669815274
[github-concurrency]: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency
[depot-compatibility]: https://depot.dev/docs/ci/compatibility
[depot-ci-api]: https://github.com/depot/cli/blob/main/proto/depot/ci/v1/ci.proto
[github-graphql-objects]: https://docs.github.com/en/graphql/reference/objects#workflow
[graphql-fixtures]: ../audit/depot-scorecard-graphql-fixtures-2026-10-15.evidence.json
[graphql-live]: ../audit/depot-scorecard-graphql-live-2026-10-15.evidence.json
[graphql-token-probe]: https://github.com/Aquaticats/Monochromatic/actions/runs/38830412356/job/116747659391
[graphql-production-job]: https://github.com/Aquaticats/Monochromatic/actions/runs/38831778986/job/116751956364
[graphql-production-evidence]: ../audit/depot-scorecard-graphql-production-2026-10-15.evidence.json
