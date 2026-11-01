# Depot CLI 2.102.23 finds no timing history when test names are submitted as class candidates

## Symptom

Physical test run `fm9d4wkbj9` successfully executed and ingested 136 passing events
for 34 unique frozen Nextest cases.
Its two shard commands nevertheless reported timings for zero candidates.
The source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`,
with harness `e1b2f76f62fbb7fea99eef4fca88443d54308455`.
The [original receipt](../audit/depot-physical-test-shards-2026-10-18.evidence.json)
retains all five execution reports and stored result rows.

The initial unresolved-history interpretation was incomplete.
The candidate file contains full Rust test names,
but all actual JUnit `classname` values are `monochromatic-git-policy-cli`.

## Root cause

The current [splitting guide](https://depot.dev/docs/ci/how-to-guides/split-tests.md)
requires runnable candidates to match JUnit file or class identities.
Its test-name timing granularity computes candidate weights from individual cases;
it does not introduce test-name candidates.

The inspected CLI is version `2.102.23`,
source `1f96d6d488f938de97ea640b115e9567ebfd6fbf`.
In `proto/depot/testresults/v1/test_results.proto:145`,
the candidate enum has file and classname variants:

```text
# depot/cli, proto/depot/testresults/v1/test_results.proto
TEST_CANDIDATE_IDENTITY_FILENAME = 1;
TEST_CANDIDATE_IDENTITY_CLASSNAME = 2;
```

`pkg/cmd/tests/selection.go:189` permits test-name timing with either candidate identity.
It does not rewrite candidate strings into another identity:

```text
# depot/cli, pkg/cmd/tests/selection.go
if timingIdentity != testresultsv1.TestTimingIdentity_TEST_TIMING_IDENTITY_TESTNAME &&
    timingIdentity != timingIdentityForCandidate(candidateIdentity) {
```

This excerpt ends at the condition;
it does not reproduce the function body.
`selection.go:227` sends both identities and the unchanged candidates to `SplitTests`.
The private backend's history implementation is not in this public source.

## Verification

The owning task is `.depot/research/test-shards:history-lookup`
at harness `f446852c42d4a382ba7931e6730a17cb3d8308e0`.
It ran inside native run `kjf78stmkb`,
using the actual locked CLI and its job-scoped OIDC context.
It made eight bounded read-only split requests and performed no test execution or upload.
The [public receipt](../audit/depot-job-policy-history-2026-10-18.evidence.json)
retains each argument vector,
input candidate set and JSON response.

The working catalog submits `monochromatic-git-policy-cli` as its classname candidate.
Depot found timing data for that candidate under both `testname` and `classname` timing granularity.
A report-key-shaped split-key control also found the actual class's timing.

The failing catalog submits the 34 full test names as classname candidates.
Both shard indexes found zero timed candidates.
Class-name timing and a report-key-shaped split key still found zero for those same names.
The deliberately absent classname negative also found zero.
This positive/negative catalog validates history reach before interpreting the zero.

## Verified workaround

Use actual JUnit class values as candidates for binary-level splitting.
The owning task's working request uses the following CLI argument form,
with the real classname supplied on stdin:

```sh
# .depot/research/test-shards/mise.toml, history-lookup task
depot tests split --candidate-type classname --timings-type testname \
  --index 0 --total 2 --key research-physical-locks-e02be6c8-20261018:actual-class --output json
```

The tradeoff is binary-level assignment:
the actual Nextest JUnit groups this executable's cases under one classname.
The physical 17-plus-17 execution result remains valid,
but it is not proof of timing-balanced per-test assignment.
A separate research adapter may deliberately produce one class per runnable test;
that requires its own raw-to-derived report provenance and consumer checks.

## What does not work

Changing `--timings-type` alone does not change candidate identity.
Using an invocation-shaped key does not repair the wrong candidate values in this observed scope.
Split and upload keys are separate client fields:
`SplitTestsRequest` has `split_key`,
while `ReportTestResultsRequest` has `invocation_id`.
The source does not establish a private backend coupling between those fields.

No unavailable-history or vendor timing defect follows from the original zero result.
The actual class positive control demonstrates available historical data.

## Upstream filing decision

Nothing to add upstream.
The measured failure is a consumer identity mismatch against the documented contract.

- Responsibility:
  the original research candidate adapter.
- Fixability:
  actual classname requests work without a provider patch.
- Supported use:
  file/class candidates are explicitly documented;
  arbitrary test names are not a separate candidate type.
- Contribution policy:
  no upstream change is proposed.
- Maintainer intent:
  the public enum and guide agree on candidate identity.
- Minimal patch:
  the read-only owning-task control verifies the narrower actual-class route.

No upstream message was sent.
