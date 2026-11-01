# Depot Test feature trial on 2026-10-16

## Result and scope

This is an isolated trial of Depot's Test feature,
not a CI migration.
The trial branch is `trial/depot-tests-20261016`.
Its workflow is `.depot/workflows/depot-tests-trial.yml`.
It exercises repository Rust tests through nextest,
JUnit ingestion,
controlled failure,
skipped tests,
same-commit retry,
CLI retrieval,
analytics,
and timing-based splitting.

The tested reporting,
failure history,
CLI retrieval,
analytics,
and timing-based splitting paths work with real repository Rust tests.
The final split run returned every expected test identity exactly once.
Repeated same-key upload is a confirmed integration risk:
Depot stored duplicate rows.
No production migration,
speedup claim,
or actual billing-total conclusion follows from this trial.

The [machine-readable evidence][evidence] records every attempt,
complete result counts,
candidate assignments,
source hashes,
and timestamp arithmetic.

## Initial report evidence

[Run `lw9hrglt00`][initial-run] used trial commit `2e1c21b01f`
as a local patch over `46effff65ab4193b4311d7675e301c97007176d1`.
The first attempt is `cp6tk4gdh6`.
The task changed an existing `window_frames` assertion inside the disposable sandbox,
ran nextest with default crate features,
and restored the source in `finally`.
Nextest reported 58 cases:
57 passed and the intended assertion failed.
The action uploaded native nextest JUnit after the failing step.

Uploading that same file again with the same `audio-core` key produced another accepted upload.
`depot tests cp6tk4gdh6 --ci --org l4zncjjbj5 --page-size 500 --output json`
returned 116 rows,
58 distinct test names,
114 passing results,
and 2 failing results,
with no continuation token.
See the [duplicate-ingestion investigation][duplicates].

The initial skip filter named the source file's module rather than the compiled Rust module.
It matched no test,
so this run does not establish skipped-test handling.
The corrected fixture excludes `gain::tests::silence_gives_unity_gain`.

The first retry retained the deliberate failure after a branch-scoped trial variable was created.
This does not isolate selector matching from the possibility of retained compiled workflow values.
`depot ci run show` reported a SHA in `ref`,
but the subsequent analytics query filtered by `refs/heads/trial/depot-tests-20261016`
returned the intended failed/pass pair.
The API source defines its requested branch ref as test-results attribution metadata.
The retry log did establish `GITHUB_RUN_ATTEMPT=2`,
so the subsequent fixture uses the observed attempt number directly.
The temporary trial variable was removed.

## Failure, skip, retry, and analytics

[Run `qlmb7szd5v`][retry-run] executed commit `edc7269534`
with one report upload per attempt.
Attempt `n3236mwkmj` returned 58 distinct rows:
56 passing,
1 intentional assertion failure,
and 1 deliberately skipped test.
Its same-commit retry `m02sjlh616` returned the same 58 identities:
57 passing and the same skipped test.
`GITHUB_RUN_ATTEMPT` changed from `1` to `2`,
and the fixture deliberately stopped mutating the assertion on the retry.

The `depot tests --status failed` and `--status skipped` queries each returned the expected single case.
The failed result retained the exact assertion values,
failure type,
and stack text.
The complete page-size-500 retrievals had no continuation token.

`depot tests analytics --ranking flaky`
identified `window::tests::window_frames_floors_and_clamps`
with 1 passing and 1 failing result,
flake rate `0.5`,
1 mixed-outcome revision,
and the correct failing attempt as `latestFailure.ownerId`.
Filtering analytics by the trial branch returned that same pair.
This validates the analytics grouping mechanism;
the test was deliberately controlled and is not evidence of a naturally flaky repository test.

The slowest-test query returned 20 ranked results and `hasMore: true`.
That verifies the ranking API path,
not an exhaustive performance analysis.
The CLI/API was exercised through the authenticated CLI.
Dashboard badges and interactive filters were not verified because this session had no usable signed-in browser.
The account-access limitation is already tracked in [issue #1040][browser-issue].

## Timing-based split and run

The trial used `depot tests split` to inspect assignments
and `depot tests run` to execute the selected units and upload native nextest JUnit.
Each job used the `depot-ubuntu-24.04` size:
2 vCPUs and 8 GB of memory,
with a 12-minute job timeout for split trials.
No local unbounded compilation or load test was used.

The first successful matrix,
[run `hn8scrvkf1`][split-run],
used 3 crate candidates.
Depot found prior timings for the audio candidate only.
Both shards passed and uploaded 375 rows:
373 passing and 2 skipped.
A same-run rerun preserved that assignment and reported the same history coverage.

A [fresh run at the same commit][fresh-run]
found timings for all 3 candidates and changed the assignment:
`forbidden-regex` ran alone,
while audio and JSONC tests shared the other shard.
This establishes learning from uploaded history across fresh runs.
It does not establish why the same-run rerun retained its original assignment.

The crate adapter exposed a mismatch:
`forbidden-regex` also executes a separate integration-test binary,
which has its own JUnit classname.
The final adapter at commit `b1d360002d` lists all 4 binary identities
and executes each through nextest's `binary_id(=...)` filter.
Its separate JUnit profile reports ignored tests
without emitting filtered tests belonging to other shards.

[Final run `wf9xzbwfph`][binary-run] passed both shards.
Depot found history for every candidate:

- Shard `0` selected `forbidden-regex` and reported 211 passing tests.
- Shard `1` selected `truepeak-core`,
  `monochromatic-jsonc-edit`,
  and `forbidden-regex::integration`.
  It reported 163 passing tests and 1 existing ignored integration test.

The combined final response contains 375 unique identities:
374 passed and 1 skipped.
The retained validation compared that identity set with the earlier complete crate runs,
required no overlap between shards,
and checked that every result classname belongs to its shard's selected candidates.

The actual nextest summaries in the final jobs span `0.045` to `0.334` seconds per selected binary.
The job timestamps span 49 and 53 seconds,
including setup and compilation.
This workload demonstrates correctness and timing-history use,
not a measured end-to-end speedup from splitting.

## Test billing and observed trial extent

[Current public pricing][pricing] includes 1,000,000 passing results each month
on Developer and Startup plans.
Additional passing results cost USD `0.000001` each.
Failed,
errored,
and skipped results are free.
Compute remains a separate charge.

Across all launches,
including setup mistakes,
intentional failures,
retries,
and duplicate uploads,
this trial created 14 job attempts.
Complete result retrieval found 1,848 stored rows,
of which 1,834 passed.
The published passing-result overage rate would price those rows at USD `0.001834`
if no included allowance remained and every stored passing row reached billing.
That is arithmetic over retrieved rows,
not an observed charge.

The API attempt timestamps sum to 682 job-seconds on 2-vCPU jobs,
or 1,364 vCPU-seconds.
Applying the published USD `0.00005` per vCPU-second overage rate gives USD `0.0682`.
Those rounded API timestamps are not the billing meter,
and included allowances or other account rules may change the charge.
Authenticated account usage was unavailable;
the broader trial tracks that boundary in [issue #999][usage-issue].

Duplicate ingestion must be reconciled before using result counts as invoice evidence.
The trial did not change billing settings or resource caps.

## Problems and verification

- [Issue #1046][duplicates-issue] remains open:
  repeated same-key uploads store duplicate rows.
  Single-upload follow-ups returned one row per identity.
- [Issue #1048][skip-issue] is closed:
  the initial filter used the wrong compiled Rust identity.
  Commit `f23707b723` corrected it,
  and the clean failure/retry run verified the skipped case.
- [Issue #1049][flag-issue] is closed:
  the initial split command used the run subcommand's `--split-key` flag.
  Commit `0822456eb7` changed the split call to `--key`,
  and subsequent matrix jobs passed.
- [Issue #1050][identity-issue] is closed:
  crate candidates omitted a distinct integration-binary timing identity.
  Commit `b1d360002d` corrected the adapter,
  and the final run verified exact binary coverage.

The isolated workflow and TOML files passed scoped dprint verification.
The audit and troubleshooting Markdown passed scoped repository lint
and rendered inspection through Pandoc.
The evidence summarizer checked complete pagination,
duplicate counts,
same-commit failure/pass identity equality,
status filters,
analytics links,
candidate coverage,
and the final 375-case identity set.
No production workflow or test source was changed in the main worktree.
The disposable worktree was removed after checking its tracked state,
ignored cache entries,
and absence of stray git-directory files.
The pushed trial branch retains the executable reproduction.

The trial did not exercise errored JUnit cases,
test-runner-internal retry extensions,
GitHub Actions runner ingestion,
multi-axis splitting,
or large-report limits.
Rust tests used the default crate features;
the wider trial separately owns feature-combination coverage.

## Sources

[Depot's test-results documentation][test-docs] describes JUnit ingestion,
dashboard history,
CLI filters,
and billing.
[The splitting guide][split-docs] describes candidate identities,
historical timing assignment,
and `depot tests run`.
The installed CLI used for local collection is Depot `2.102.22`.
The report action is pinned to `2ad5779840a9c63e884fa26f11d019768a27a941`.

[initial-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/lw9hrglt00
[duplicates]: ../troubleshooting/depot-test-report-duplicate-ingestion.md
[test-docs]: https://depot.dev/docs/ci/observability/depot-ci-test-results
[split-docs]: https://depot.dev/docs/ci/how-to-guides/split-tests
[evidence]: depot-test-feature-2026-10-16.evidence.json
[retry-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/qlmb7szd5v
[split-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/hn8scrvkf1
[fresh-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/b8k8plzdsk
[binary-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/wf9xzbwfph
[pricing]: https://depot.dev/pricing
[duplicates-issue]: https://github.com/Aquaticats/Monochromatic/issues/1046
[skip-issue]: https://github.com/Aquaticats/Monochromatic/issues/1048
[flag-issue]: https://github.com/Aquaticats/Monochromatic/issues/1049
[identity-issue]: https://github.com/Aquaticats/Monochromatic/issues/1050
[usage-issue]: https://github.com/Aquaticats/Monochromatic/issues/999
[browser-issue]: https://github.com/Aquaticats/Monochromatic/issues/1040
