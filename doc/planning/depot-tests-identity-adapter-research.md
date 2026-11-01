# Depot test identity adapter research

## Status and admitted scope

The serial native CI profile completed successfully from frozen application source
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Run `st40hvmw90` completed at research source `c6d25d092678b0933f1e3b43765fc46c02593a9e`.
The purpose is to test historical weighting for individual runnable Nextest test names
without changing production test reports or routing.

The parent admitted the existing source-bound 98,192,385-byte Nextest/Git/tools producer,
34 unchanged selected cases,
three seed repetitions,
two distinct physical shard jobs,
and a downstream consumer.
The profile may consume at most one concurrent VM.
Its full configured-window compute reservation is USD 0.174.
Prior physical-profile reservations total USD 0.348,
and a separately admitted read-only history follow-up reserves USD 0.012.
The resulting conservative campaign reservation is USD 0.534 under the shared USD 1 ceiling.
These are scoped reservation proxies,
not an invoice or account-wide cap.

The transport admission permits one new exact repository secret variant containing only a
five-minute signed GET for the owned producer object.
The controller must verify exact variant absence,
bind through standard input,
retain no signed query in public evidence,
and delete that variant with an absence check after terminal evidence retention.
No broad credential is sent to a job.

## Deciding source and existing evidence

The [Depot split contract][split] permits filename or classname candidate identities.
Individual-case timing granularity does not change candidate identity.
The [raw physical cohort](../audit/depot-physical-test-shards-2026-10-18.evidence.json)
has distinct test names but one binary classname.
The parent's later actual-classname positive control establishes history reach;
it does not establish per-test balancing with arbitrary unmatched classnames.

Nextest `cargo-nextest-0.9.148`,
commit `cd1d6d5467322dbc0d39a80cb671d000ac302798`,
unconditionally sets ordinary testcase classnames from `test_instance.binary_id` in
`nextest-runner/src/reporter/aggregator/junit.rs:219`.
Its skipped-case path does the same at line 262,
and its source test explicitly expects the binary ID at line 921.
`nextest-runner/src/config/elements/junit.rs:155` exposes report path/name,
output storage,
skipped-case,
and flaky-status controls,
without a classname-format field.
The versioned [JUnit documentation][nextest-junit] describes each binary as a suite
and recommends post-processing for tools with different expectations.

The actual producer executable reports Nextest `0.9.148`,
commit `cd1d6d5467322dbc0d39a80cb671d000ac302798`,
and has SHA256 `53470cfc9b5cbcd96780db2d348700d76af962467503791f1f8a8824d8967e5c`.
The current latest release remains `cargo-nextest-0.9.148`,
published `2026-10-08T03:32:39Z`.
Current main `6c2ab7e669761d9ef063bf00b2440c0fb5cca1ac` has byte-identical deciding reporter,
configuration,
and JUnit documentation files.
A built-in classname format control is absent from those inspected versioned and current paths.

## Local adapter qualification

Research source `c6d25d092678b0933f1e3b43765fc46c02593a9e`
adds only the isolated `.depot/research/test-identity-adapter` fixture and its manual workflow.
The application remains frozen.
The adapter accepts the selected passing ASCII Nextest report shape,
one expected binary suite,
and exact unique runnable names from the tracked candidate universe.
It writes separate derived XML and a mapping receipt;
it does not overwrite the original report.

Actual mapper invocations passed four positive controls:
the original 34-case report,
both original 17-case reports,
and a reordered-attribute report.
An independently constructed expected result matched every output byte.
Only validated classname attribute values changed.
Original suite/name metadata,
durations,
and all other XML bytes remained unchanged.
Raw and derived hashes are retained independently.

Actual mapper invocations rejected 24 controls,
including duplicate or unknown selections/cases,
quote,
newline,
escape,
delimiter,
traversal,
command-separator,
source-escaped and entity boundaries,
cross-binary classes,
duplicate or unknown attributes,
negative or nonfinite durations,
missing cases,
nested failure output,
DTD markup,
extra roots/text,
non-ASCII input,
and an oversized report.
Rejected reports produced no derived XML or mapping receipt.

The guard-removal control used committed source `237254fdd14661c9274617bc1896e495a2082dbc`.
Removing the selected-name membership guard made an unknown report case incorrectly succeed.
The negative control then failed as required.
The recipe was restored byte for byte,
and its Git diff was empty.
The mapper section remains identical in the later source with expanded controls.

The first local controller invocation selected the repository root,
where explicit configuration roots exclude `.depot`.
It failed before executing the mapper.
Launching from the owning configuration directory passed,
following the existing [Mise task-discovery diagnosis](../troubleshooting/mise-excluded-research-task-discovery.md).
No cloud allocation or test-ingestion failure receives credit from that local diagnostic.

This qualification is deliberately limited to the passing report shape and selected single-binary cohort.
It does not qualify arbitrary JUnit,
failing/output-bearing cases,
setup-script suites,
cross-binary duplicate names,
or a production reporter change.

## Cloud consumer and cleanup

Both fresh physical shard jobs found timings for all 34 exact runnable-name candidates.
They executed disjoint 17-case sets,
and the downstream consumer passed the exact 34-case union.
Attempt-scoped API reads retained 136 passing rows across the three seeds and two shards,
with `className` equal to the unchanged exact `testName` on every row.
The original and derived XML matched the independently constructed rewrite byte for byte.
Four provider sandbox IDs were distinct.
This does not identify physical hosts or measure improved elapsed time.

All five original ZIPs were retained privately with API-size,
CRC and independently computed SHA256 checks.
The four report/consumer ZIPs are also included byte for byte in the public compressed source artifact.
The separate 98 MB input ZIP is represented by exact metadata and hashes rather than duplicated in Git.
The exact owned transport variant was deleted,
absence was verified,
and the private signed URL file was retired.
No follow-up history allocation was needed.

The [public evidence](../audit/depot-tests-identity-adapter-2026-10-18.evidence.json)
records individual report hashes,
lookup counts,
source/resource admission and cleanup.
The [source artifact](../audit/depot-tests-identity-adapter-2026-10-18.sources.json.gz)
retains deciding sources,
full fixtures,
original report ZIPs and raw API/log evidence.
This selected cohort does not qualify the complete 1,956-case suite or a speed benefit.

[split]: https://depot.dev/docs/ci/how-to-guides/split-tests
[nextest-junit]: https://github.com/nextest-rs/nextest/tree/cargo-nextest-0.9.148/site/src/docs/machine-readable
