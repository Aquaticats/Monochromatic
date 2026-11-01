# Nextest 0.9.148 binary classnames miss per-test Depot split candidates

## Symptom

The original frozen-source physical cohort passed its selected tests and uploaded JUnit successfully,
but splitting with individual Rust test names found no matching candidate timings.
The report had 34 distinct test names and one classname,
`monochromatic-git-policy-cli`.
Successful ingestion did not prove that candidate identities matched report identities.

The later actual-binary-classname control found timing data.
That positive control rules out treating the original per-test miss as a general history outage.
The [original physical evidence](../audit/depot-physical-test-shards-2026-10-18.evidence.json)
and [identity controls](../audit/depot-job-policy-history-2026-10-18.evidence.json)
retain those separate observations.

## Root cause

Nextest `0.9.148`,
commit `cd1d6d5467322dbc0d39a80cb671d000ac302798`,
creates the case name from the runnable test name and the classname from its binary ID.
`nextest-runner/src/reporter/aggregator/junit.rs:217` contains:

```text
// nextest-runner/src/reporter/aggregator/junit.rs:217
let mut testcase = TestCase::new(test_instance.test_name.as_str(), testcase_status);
testcase
    .set_classname(test_instance.binary_id.as_str())
    .set_timestamp(main_status.start_time)
    .set_time(main_status.time_taken);
```

The skipped-case path also uses the binary ID at `junit.rs:262`.
The inspected configuration in `nextest-runner/src/config/elements/junit.rs:155`
exposes `path`,
`report_name`,
`store_success_output`,
`store_failure_output`,
`report_skipped`,
and `flaky_fail_status`.
It has no classname-format field.
The source fixes the classname independently of those settings:

```text
// nextest-runner/src/reporter/aggregator/junit.rs:262
testcase.set_classname(test_instance.binary_id.as_str());
```

Current main `6c2ab7e669761d9ef063bf00b2440c0fb5cca1ac`
has byte-identical deciding reporter,
configuration and JUnit documentation files.
The current latest release is still `0.9.148`.

The [deployed Depot split contract][split] says a runnable candidate must map to either
the report's filename or classname.
It separately says `timings-type: testname` calculates candidate durations from individual cases.
That timing option does not reinterpret testcase names as classname candidates.
The original full runnable names therefore differed from the reported binary classname.

Nextest documents post-processing at `site/src/docs/machine-readable/junit.md:77`:

```text
site/src/docs/machine-readable/junit.md:77
You can post-process the JUnit file in such cases.
```

The linked `circleci-junit-fix` precedent was inspected at
`24f9b6e46476b243f0e6f3c7422286c9347991a4`.
It repairs failure-message/output handling;
it does not implement this classname mapping.

## Verification

The [adapter evidence](../audit/depot-tests-identity-adapter-2026-10-18.evidence.json)
and [source archive](../audit/depot-tests-identity-adapter-2026-10-18.sources.json.gz)
retain exact source,
raw and derived reports,
original ZIPs,
API rows,
and control outcomes.
The producer executable's SHA256 is
`53470cfc9b5cbcd96780db2d348700d76af962467503791f1f8a8824d8967e5c`.
It reports the matching Nextest version and commit.

The passing catalog consists of the original 34-case XML,
both original 17-case XML reports,
and an attribute-order variation.
Every resulting byte matched an independent classname-only rewrite.
The original bytes remained unchanged.

The rejected catalog contains 24 duplicate,
unknown-name,
syntax-boundary,
cross-binary,
attribute,
duration,
structure,
character-set and size controls.
These are rejected by the narrow adapter;
some are otherwise valid JUnit shapes outside its admitted scope.
Removing the committed selected-name membership guard made an unknown case succeed,
so the control failed.
Restoring the exact source restored the guard.

Actual native run `st40hvmw90`,
fixture source `c6d25d092678b0933f1e3b43765fc46c02593a9e`,
passed three unchanged seed executions,
two fresh physical shard jobs,
and the downstream consumer.
Each physical shard reported timings for all 34 candidate identities.
They ran disjoint 17-case sets.
Attempt-scoped API reads retained 136 passing rows,
with classname equal to the unchanged exact runnable name on every row.
Four provider sandbox IDs were distinct.
No physical-host,
full-suite,
or elapsed-time improvement claim follows from this cohort.

## Verified workaround

The [source-bound fixture patch](depot-nextest-test-identity.patch)
preserves the raw XML and uploads a separately derived report.
In its `.depot/research/test-identity-adapter/mise.toml:347`,
the mapper validates exact selected names,
uniqueness,
the expected binary,
and finite nonnegative case durations before changing an attribute.
At line 352,
the only replacement is:

```typescript
// .depot/research/test-identity-adapter/mise.toml:352, fixture source c6d25d0926
replacement = tag.slice(0, attribute.start) + escapeAttribute(name) + tag.slice(attribute.end);
```

The committed workflow invokes `run-cohort` from its owning configuration directory,
passes exact candidates to Nextest,
and uploads the derived XML only after the mapper succeeds.
The original suite,
name,
duration,
raw hash and derived hash remain in its mapping receipt.

This changes classname grouping.
It qualifies only the selected ordinary passing ASCII report from one binary.
Failing/output-bearing reports,
Unicode test identifiers,
setup-script suites,
cross-binary name collisions,
production history continuity and the complete suite remain unqualified.
The patch is an isolated research fixture,
not a production reporter migration.

## What does not work

Changing only `timings-type` does not repair an unmatched candidate classname.
Successful test ingestion alone does not validate the split lookup.
The inspected built-in Nextest configuration does not provide this mapping.
The community CircleCI post-processor performs a different transformation.

Launching the local control from the repository root failed task discovery before the mapper ran.
The [existing Mise diagnosis](mise-excluded-research-task-discovery.md)
explains the explicit configuration-root boundary.
Launching from the owning configuration passed.
No extra cloud allocation was used for that local recovery.

## Upstream filing decision

Nothing to add upstream.
This was a consumer identity mismatch,
and the inspected Nextest reporter implements its documented grouping.

- Responsibility:
  no Nextest or Depot contract violation was established.
- Fixability:
  a source-owned post-processor passed actual consumers;
  changing Nextest's representation is unnecessary for this diagnosis.
- Supported use:
  Nextest documents post-processing,
  and Depot documents matching filename/classname identities.
- Contribution policy:
  current `site/src/docs/contributing.md` welcomes careful LLM-assisted code,
  but explicitly rejects primarily LLM-authored issues and bug reports.
  No primarily generated upstream draft or message is proposed.
- Maintainer intent:
  the current reporter remains unchanged in the deciding paths.
  The documented grouping is not evidence that maintainers rejected every possible future option.
- Minimal patch:
  the actual consumer-side fixture and its controls are retained;
  no upstream prototype is required when no upstream defect is established.

The `.out-of-scope/` inventory had no matching Nextest or Depot exemption.
Issue searches covered `classname`,
`junit`,
and `"class name"`;
the PR search covered `junit`.
The `classname` hit,
[issue 831][issue-831],
was read with its comments and concerns historical scheduling rather than this identity mismatch.
No issue,
comment or external message was sent.

[split]: https://depot.dev/docs/ci/how-to-guides/split-tests
[issue-831]: https://github.com/nextest-rs/nextest/issues/831
