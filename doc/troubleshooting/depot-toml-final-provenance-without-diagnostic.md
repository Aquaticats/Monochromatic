# Depot CLI 2.102.23 retains no diagnostic for a failed final TOML provenance guard

## Symptom

Coherent cohort `complete-20261017T111555806Z` ran the unchanged TOML source
`525d2e09f8168a3bdd7e513e26d4c80b363123e5`
through harness `9cc7f28fb8db201d9774ff7cf5404ebec3ffc260`.
Depot run `4c27t98pxx`,
workflow `zkn999rjbq`,
job `4sx4665mqp`,
attempt `3vxxw8xn21` failed with this structured provider diagnostic:

```text
Step 13 (Verify frozen source and cohort after consumers): script exited with code 1
```

The selected native TOML mutation task reported exit zero and verified cleanup.
Its native report records 921 killed,
256 survived,
3 timeout,
1882 compileError,
0 runtimeError,
and 0 ignored across 463 shards.
Those source-task observations do not establish successful final provenance verification.
Artifact `01a149d8-85ca-7109-9181-b531cb1218db` contains the before record
and no `toml-provenance-after.json`.

Repository issue [#1179][incident] tracks this failed completion boundary.

## Root cause remains unknown

The provider identifies the failed step but supplies no relevant diagnostic lines.
Both finite log retrieval paths stop at `2026-10-17T12:31:25.632Z`,
with the source mutation report-written message.
This does not prove which assertion,
transport operation,
or process failed,
or where output became unavailable.
The provider's `diagnosis` and `possible_fix` strings are suggestions;
they do not establish a source or common-manifest mismatch.

At the frozen harness,
`package/monorepo-task/core-exec/mise.toml:916` requires the committed complete case manifest:

```javascript
// package/monorepo-task/core-exec/mise.toml:916, frozen TOML harness
assert.equal(localCase.completeGraph, true, 'Final route requires a frozen complete case manifest');
```

The final record is written only after the authenticated Contents API request,
common-manifest checks,
and before/after identity comparisons.
`package/monorepo-task/core-exec/mise.toml:941` performs that write:

```javascript
// package/monorepo-task/core-exec/mise.toml:941, frozen TOML harness
writeFileSync(join(directory, 'toml-provenance-' + phase + '.json'), JSON.stringify(record, null, 2) + '\n');
```

The missing record alone cannot identify which preceding operation failed.

## Verification

Installed Depot CLI `2.102.23` reports release tag `v2.102.23`.
The unmodified source clone's `pkg/cmd/ci/logs.go` matches that tag at Git blob
`c263bf341a5abbf20a4ab37e1065c9da7318e7a2`.
These commands were exercised against the completed attempt:

```sh
# Read-only retrieval of the completed failed attempt
depot ci diagnose --attempt 3vxxw8xn21 --org l4zncjjbj5 --output json
depot ci logs 3vxxw8xn21 --org l4zncjjbj5 --output json --output-file logs.jsonl
depot ci logs 3vxxw8xn21 --org l4zncjjbj5 --output json
```

Each log operation returned 120742 records and 41239978 body bytes.
The ordered bodies and timestamps match after lossless normalization of fractional-second trailing zeros.
Removing a record changes the comparison hash.
The raw JSONL files remain distinct and retain their original hashes.

`pkg/cmd/ci/logs.go:154` selects the export operation when an output file is supplied:

```go
// pkg/cmd/ci/logs.go:154, depot/cli v2.102.23
return downloadLogsToFile(ctx, tokenVal, orgID, api.CILogStreamTarget{AttemptID: target.attemptID},
    outputOptions, outputFile, cmd.ErrOrStderr())
```

The excerpt wraps the original expression for readability.
The stdout path calls the historical retrieval helper at `pkg/cmd/ci/logs.go:157`:

```go
// pkg/cmd/ci/logs.go:157, depot/cli v2.102.23
lines, err := ciGetJobAttemptLogs(ctx, tokenVal, orgID, target.attemptID)
```

`pkg/cmd/ci/logs.go:357` explains the stdout timestamp spelling:

```go
// pkg/cmd/ci/logs.go:357, depot/cli v2.102.23
return time.UnixMilli(line.GetTimestampMs()).UTC().Format(time.RFC3339Nano)
```

For example,
the export spells an instant as `2026-10-17T11:19:09.190Z`,
while stdout spells the same instant as `2026-10-17T11:19:09.19Z`.
The initial bytewise timestamp comparison was therefore unsuitable.
Trimming only fractional trailing zeros preserves precision and verifies equal retained sequences.

Working observations are the source mutation completion,
bounded cleanup,
published evidence archive,
and successful final source/common pairs from the other TOML execution waves.
The failing observation is the long mutation job's final guard,
without a diagnostic in either retained log sequence or an after-provenance archive member.

## Verified workarounds

No workaround restoring the missing final guard evidence has been verified.
The two log commands recover the same available records through different retrieval paths.
Their tradeoff is explicit:
they preserve source-task observations but do not recover the missing completion record.
The source task can be reported as observed successful;
the final source/common provenance pair remains unaccepted.

## What does not work

Repeating retrieval through the historical API did not expose the failed guard output.
A read-only `depot sandbox exec` probe used the failed attempt's recorded sandbox and session IDs.
Depot CLI `2.102.23` returned:

```text
sandbox "vxbdgtwhm94dpk9f124k" was not found; check that the sandbox ID is correct and the sandbox is still running
```

The probe started at `2026-10-17T13:11:59.797Z` and returned exit one at `2026-10-17T13:12:00.503Z`.
It enumerated no account-wide resources and recovered no worker diagnostics.
Treating the before record or source mutation success as final provenance would omit the completion boundary.
Comparing raw timestamp spellings rejects equivalent instants.
A diagnosis suggestion is not evidence that the source changed.

## Upstream filing decision

The `.out-of-scope/` inventory and its Depot/log diagnostic search contain no matching exemption.
No upstream defect or fileable submission is established:

1.  Fault attribution remains unknown.
    The failed guard and unavailable diagnostic do not identify an upstream defect.
2.  Fixability cannot be assigned to a code path until the failure is identified.
3.  Historical log retrieval is an intended CLI operation,
    as documented by [log export support][log-export] and [step metadata support][step-metadata].
4.  Contribution eligibility was not assessed because no attributable defect or submission is established.
5.  No refusal or willingness is inferred.
    Searches of open and closed `depot/cli` issues and pull requests for missing logs found no exact match.
    Broader log searches found the merged retrieval and metadata features.
6.  No upstream fix prototype is claimed.
    The reproduced repository guard failure and independent log retrieval probes are retained separately.

The merged features are present in the installed CLI source;
they are not proposed as an absent fix for this incident.
There is no substantiated upstream issue or additive upstream comment to file.
The repository incident remains open.

[incident]: https://github.com/Aquaticats/Monochromatic/issues/1179
[log-export]: https://github.com/depot/cli/pull/500
[step-metadata]: https://github.com/depot/cli/pull/501
