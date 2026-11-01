# Depot test-report-action v1 stores repeated same-key JUnit uploads twice

## Symptom

In [trial run `lw9hrglt00`][trial],
`depot/test-report-action` commit `2ad5779840a9c63e884fa26f11d019768a27a941`
uploaded the same 58-case JUnit file twice from one job attempt.
Both steps used `key: audio-core`.
Both printed:

```text
files_processed=1 files_skipped=0 tests_reported=58 duplicate_invocation=false
```

Depot CLI `2.102.22` retrieved 116 rows from attempt `cp6tk4gdh6`,
including each test identity twice.
There were 114 passing rows and 2 failing rows,
representing 57 passing tests and 1 intentionally failing test.
The complete response had no continuation token.

## Source boundary and cause

The [documented action key][docs] is intended to distinguish upload invocations for deduplication.
The [action source][action-source] at `src/index.ts:48` sends the explicit key without changing it:

```typescript
// depot/test-report-action/src/index.ts
const request = create(ReportTestResultsRequestSchema, {
  invocationId: inputs.invocationKey,
  files: reportFiles,
})
```

The same source at `src/index.ts:131` recognizes an ignored duplicate response:

```typescript
// depot/test-report-action/src/index.ts
if (response.duplicateInvocation) {
  core.info('Depot ignored this upload because this invocation was already reported.')
  return
}
```

The [published service protocol][protocol] at `proto/depot/testresults/v1/test_results.proto:176`
instead documents the duplicate indicator as always false:

```protobuf
// depot/cli/proto/depot/testresults/v1/test_results.proto
// Whether this invocation was previously reported. Currently always false.
bool duplicate_invocation = 3;
```

The retained API result proves repeated ingestion at the service boundary.
The private backend implementation was not available in the inspected public repositories,
so the server-side storage mechanism is not established.
This is not evidence that the action changed the key or generated distinct XML.

## Verification

Trial commit `2e1c21b01f` contains the executable reproduction.
Its workflow first runs real `truepeak-core` nextest tests with one assertion deliberately changed,
then invokes the pinned report action twice with the same path and key.
The native report contains 58 cases.
Each upload logs 58 accepted cases.
The complete consumer response contains 116 rows.

```bash
# Run from the isolated trial worktree at commit 2e1c21b01f.
depot ci run --org l4zncjjbj5 --repo Aquaticats/Monochromatic \
  --workflow .depot/workflows/depot-tests-trial.yml
```

```bash
# Retrieve every stored case from the observed attempt.
depot tests cp6tk4gdh6 --ci --org l4zncjjbj5 --page-size 500 --output json
```

The working catalog is native nextest JUnit parsing,
passing-result retrieval,
and failed-result retrieval with the assertion text.
The failing catalog is repeating an identical upload with an identical invocation key:
both calls are accepted and both sets remain visible.

## Verified workaround

The clean follow-up workflow contains only one report upload per attempt.
Attempt `n3236mwkmj` returned 58 distinct rows:
56 passing,
1 intentionally failing,
and 1 skipped.
Its same-commit retry `m02sjlh616` returned 58 distinct rows:
57 passing and 1 skipped.
This prevents the tested duplicate call at the workflow boundary,
but does not provide idempotency for transport retries or other callers.

## What did not work

Reusing the documented key did not deduplicate the second upload.
Checking `duplicate_invocation` also did not identify the repeated report:
both responses returned false.

## Upstream filing decision

The user authorized issues in the current repository only.
No upstream issue or comment was sent.

- Responsibility:
  repeated ingestion is observed at Depot's service boundary;
  backend cause is unverified.
- Feasibility:
  no evidence establishes whether service deduplication can be repaired from public source.
- Supported use case:
  the official action and documentation describe invocation keys and deduplication.
- Contribution policy:
  not evaluated because external filing is outside this task's authorization.
- Maintainer intent:
  not established.
- Tested upstream patch:
  none;
  the backend source was not present in the inspected action and CLI repositories.

The `.out-of-scope/` inventory contained no Depot-specific exemption.
The current-repository issue records the consumer risk and required follow-up.
[Issue #1046][issue] tracks the observed behavior.
Passing-test billing may be affected by repeated ingestion,
but this trial has not retrieved an authenticated usage total and does not claim a charge occurred.

[trial]: https://depot.dev/orgs/l4zncjjbj5/workflows/lw9hrglt00
[docs]: https://depot.dev/docs/ci/observability/depot-ci-test-results
[action-source]: https://github.com/depot/test-report-action/blob/2ad5779840a9c63e884fa26f11d019768a27a941/src/index.ts
[protocol]: https://github.com/depot/cli/blob/1f96d6d488f938de97ea640b115e9567ebfd6fbf/proto/depot/testresults/v1/test_results.proto
[issue]: https://github.com/Aquaticats/Monochromatic/issues/1046
