# Depot CLI 2.102.23 workflow metadata gives incomplete inputs for allocation accounting

## Symptom

The trial's repository-filtered inventory contains 276 terminal run records.
Its 122 workflow records contain 394 jobs and 391 attempt records.
The other 154 run records have no workflow.
An inventory record alone is not evidence of an allocated worker.

All 391 attempts in `depot ci workflow show --output json` have whole-second timestamp fields.
The metrics endpoint supplies millisecond timestamps for the observed allocation intervals.
93 workflow records have an empty `workflow_path`.
Workflow metadata supplies neither runner sizes nor the uploaded YAML bytes needed to bind historical resources.

Two successful metric records have `availability.code: no_sandbox`,
no start timestamp and equal creation/finish timestamps.
They are retained as provider records without an observed allocation interval.
No zero invoice charge is inferred.

## Source trace and limits of the diagnosis

The installed CLI reports `2.102.23`.
The source clone is unmodified at `1f96d6d488f938de97ea640b115e9567ebfd6fbf`.
The Git blobs for the relevant files match tag `v2.102.23`:

- `pkg/cmd/ci/workflow.go`:
  `7d01a70340086f5f4aedcb5da11dc25e10bd7e39`.
- `pkg/cmd/ci/metrics.go`:
  `1e27e860dbe9ed184a8631f502f46f526b1688bd`.

The source paths in this section refer to the Depot CLI repository.
`pkg/cmd/ci/workflow.go:390` copies attempt timestamp strings directly from the workflow response:

```go
// pkg/cmd/ci/workflow.go
StartedAt:  attempt.GetStartedAt(),
FinishedAt: attempt.GetFinishedAt(),
```

This does not establish client-side timestamp truncation.
The server-side formatting cause is unknown.
The observed precision difference is between the retained endpoint responses.

`pkg/cmd/ci/metrics.go:281` retains the metric endpoint's attempt context separately from sample statistics:

```go
// pkg/cmd/ci/metrics.go
func metricAttemptSummaryJSON(attempt *civ1.CIMetricsAttemptSummary) attemptSummaryJSON {
    return attemptSummaryJSON{
        Attempt:      attempt.GetAttempt(),
        Availability: metricAvailabilityJSON(attempt.GetAvailability()),
        Stats:        metricStatsJSON(attempt.GetStats()),
        Cap:          metricCapJSON(attempt.GetCap()),
    }
}
```

`pkg/cmd/ci/metrics.go:301` maps the explicit no-sandbox availability code:

```go
// pkg/cmd/ci/metrics.go
case civ1.CIMetricsAvailabilityCode_CI_METRICS_AVAILABILITY_CODE_NO_SANDBOX:
    return "no_sandbox"
```

An absent start time is therefore not repaired by substituting its creation time.
The retained response does not establish why those successful records lacked a sandbox.

## Verification

The historical control uses the actual repository-scoped workflow and run IDs:

```bash
# Read-only control against the retained historical run.
depot ci workflow show 7bh1n71pss --org l4zncjjbj5 --output json
depot ci metrics --run knsghczl71 --org l4zncjjbj5 --output json
```

The metrics control reports attempt `snclls3gdd` from `09:57:55.689Z` to `10:00:55.706Z` on 2026-10-17.
The full scoped collection fetched 115 historical run metric responses,
verified their repository and attempt identities,
and reconciled 283 historical attempt records.
The existing 108 coherent attempt metric records were reused.
There were no collection errors.

The working catalog is:

- Metrics contexts bind all 389 observed allocation intervals to exact provider attempt IDs.
- Whole-second workflow timestamps match the second containing each observed metric endpoint.
- Removing all workflow paths still recovers the 108 coherent resource sizes and exact workflow hashes
  by declared workflow name.
- Current Depot sandbox documentation maps the retained label candidates to explicit CPU and memory sizes.

The rejected catalog is:

- Requiring every successful attempt to have a start timestamp.
  The first accounting assertion rejected the two no-sandbox records.
- Pricing from workflow metadata without reading the metrics contexts and resource declarations.
- Treating a same-name workflow at the reported Git head as proof of the exact uploaded historical YAML.

The [accounting evidence](../audit/depot-repository-accounting-2026-10-17.evidence.json)
retains the raw endpoint responses,
source candidates,
individual intervals and overlap calculations.

## Verified consumer workarounds

Use metrics contexts for interval endpoints and preserve no-sandbox records separately.
Join resource candidates by workflow/job identity and exact source hashes.
The coherent cohort has 108 verified resource bindings.
Historical source and label joins remain provisional because uploaded byte identity is not established.
18 observed allocated attempts still have no resource binding and receive no invented size or price.

For overlapping work,
compute the union of intervals and report it separately from their sum.
The private verifier checked overlapping and touching synthetic intervals before using that operation.
The measured union is not a reconstructed workflow dependency critical path.

These workarounds trade a single metadata query for multiple scoped evidence joins.
They improve the observed allocation calculation;
they do not establish an invoice,
an undocumented provisioning charge boundary,
storage cost or allowance consumption by unrelated projects.

## What does not work

The initial assumption that the workflow metadata already retained precise interval endpoints was wrong.
The metrics control disproved it before the historical proration was retained.
Likewise,
the first interval builder failed when it required a start time on every successful record.
Replacing either missing start time with creation time would manufacture allocation evidence.

Historical YAML from a matching Git head is a candidate source,
not proof of what an inline API dispatch uploaded.
Job-key mismatches remain unresolved rather than inheriting another job's resource size.

## Upstream filing decision

1.  An upstream defect is not established by these observed response boundaries.
2.  Changing a response or adding resource provenance may be possible;
    no architectural impossibility is claimed.
3.  The CLI supports workflow inspection and scoped metrics collection,
    both exercised here.
4.  Contribution acceptance was not evaluated because no new upstream defect report is justified.
5.  Expected upstream action is unknown;
    missing fields are not evidence of a maintainer refusal.
6.  No upstream prototype was made because the current evidence supports consumer reconciliation,
    not a demonstrated violation of an upstream contract.

There is no fileable upstream bug report from this accounting observation.
The remaining work is to recover historical uploaded definitions where available
and retain explicit uncertainty where they are absent.
