# Independent review of Depot orchestration and physical test evidence

## Reviewed scope

This review examines the parent-authored native-orchestration and physical-test receipts,
their original ZIP archives,
and the Mise task-discovery and rerun-artifact troubleshooting notes.
It does not review the platform audit author's own research as independent evidence.
The checks ran on 2026-10-18 at 11:21 UTC without provider mutations or new allocations.

The reviewed evidence files are:

- [Native orchestration](../audit/depot-native-orchestration-2026-10-18.evidence.json),
  SHA256 `a9bf4d6762c2fc2c219f9f08fe80be888f80f7bf2c01189507df63f150cea8bf`.
- [Physical test shards](../audit/depot-physical-test-shards-2026-10-18.evidence.json),
  SHA256 `e09a818e85777eed1547d08196490946b32e85759740001470df1c5d130ef0fa`.
- [Mise task discovery](../troubleshooting/mise-excluded-research-task-discovery.md),
  SHA256 `462bde96fdcdf8ec0c7a87be18d1960e113bec619f93dfb29cd723ff4932303a`.
- [Rerun artifact selection](../troubleshooting/depot-rerun-artifact-id-order.md),
  SHA256 `45ebffd21bf27266c38a4bb15473497cdce1aaa2fc5e076dcb89c06c6d352973`.

Later edits can supersede these findings without changing what these file hashes contain.

## Findings requiring correction or qualification

### Test candidate identities do not match the reports

The physical receipt describes zero timing coverage as an unresolved history identity or availability boundary.
The [deployed splitting contract][split] provides a narrower explanation that must be addressed first:
`classname` candidates must match JUnit `classname` values.
`timings-type: testname` derives candidate durations from individual cases;
it does not add a test-name candidate type.

All five original JUnit files use `monochromatic-git-policy-cli` as their classname.
The harness instead supplies individual test names as `classname` candidates.
This mismatch prevents the zero-coverage observation from establishing a vendor history failure.
An actual classname candidate is the required positive control.
The physical execution and report-ingestion results remain valid within their selected cohort.

### The artifact workaround note has a stale qualification state

The artifact troubleshooting note says the content-bound cloud workaround and full rerun remain pending.
The newer original ZIPs establish passing consumers on both attempts of `w9jdpj5f1l`,
at harness `46fab80ef500e7d857cdb822085c03f899e266ff`.
The note should describe the verified result and retain its original failure evidence.

### Selector behavior and incident causality need separate scopes

The published SDK's descending-ID selector can choose the older of the two actual artifact records.
Both IDs are safe JavaScript integers,
so integer precision does not explain that pair.
The raw downloaded archive also proves that the failing consumer received the older bytes.

Those facts establish the selector vulnerability and the stale-artifact incident.
The reviewed artifacts do not retain the failing Action's unfiltered `ListArtifacts` response.
They therefore do not establish whether both candidate rows were visible to that client before filtering.
Keep that visibility condition explicit when attributing the observed incident to ordering.
No artifact propagation delay or backend ID-construction mechanism is established.

## Independently verified results

Both compressed source archives match their published lengths and SHA256 values.
All 20 original ZIPs,
15 orchestration archives and five physical-cohort archives,
match their recorded byte sizes and SHA256 values and pass `unzip -t` CRC checks.
These are local checks against retained API lengths and published hashes,
not a claim of native server-provided ZIP digests.

For both content-bound orchestration attempts,
each consumer receipt hash matches the original producer JSON,
the corresponding immutable artifact name,
and the current expected producer output hash.
All six producer joins match the frozen application source and current harness.
Each retains the passing source and producer-hash rejection controls.
The latest run metadata contains ten distinct provider sandbox IDs across five jobs and two attempts.
This does not prove ten distinct physical hosts or physical deallocation timing.

The five original JUnit files contain 136 passing execution events for 34 unique test names:
three 34-case seeds and physical shards of 17 cases each.
Original XML hashes and case names match the execution receipts.
The shard union is exact and disjoint,
and the stored-result API rows agree with those passing events and classname values.
The selected tests used an owned 256 MiB tmpfs,
and the retained cleanup receipts report successful unmount and absence of that filesystem afterward.
The full 1,956-case application suite is not qualified by this cohort.

The reviewed Mise launcher source changes the Depot child's working directory to its owning configuration.
That directly addresses the observed missing-task boundary.
It does not establish a vendor defect or repair the independent candidate identity mismatch.

## Remaining review boundary

Ordinary job-level continuation is being tested separately from the intentionally cancelled timeout job.
The mixed orchestration workflow's raw aggregate failure must not be used alone to attribute that behavior.
This review makes no interview-readiness or complete-platform qualification claim.

[split]: https://depot.dev/docs/ci/how-to-guides/split-tests
