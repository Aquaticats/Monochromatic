# Actions artifact SDK 6.2.1 can select an older same-name artifact on Depot CI reruns

## Symptom

Native run `1jqm5wjmlj` passed its final artifact consumer on its first attempt.
Its full workflow rerun regenerated passing producer receipts,
but the download Action selected the original `allrun` ZIP alongside current state/default ZIPs.
The consumer's producer-hash guard rejected the mixture:

```text
ArtifactAdmissionFailure [Error]: Research artifact differs from its current producer output hash
```

The harness is `bf3c4a98b32826c1c16e8755b8775256340fab79`.
The download Action is `actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c`.
The current-output check prevented a false passing consumer result.

## Root cause and boundary

The pinned Action's `src/download-artifact.ts` requests `listArtifacts({ latest: true })`
before applying its artifact-name pattern.
Its pinned package lock resolves `@actions/artifact` version `6.2.1`.
The official package archive is bound by SHA512:

```text
sha512-sJGH0mhEbEjBCw7o6SaLhUU66u27aFW8HTfkIb5Tk2/Wy0caUDc+oYQEgnuFN7a0HCpAbQyK0U6U7XUJDgDWrw==
```

In that published package,
`lib/internal/find/list-artifacts.js:109` converts the service database ID to a number.
Its `filterLatest` at line 131 sorts descending by ID,
then keeps the first artifact for each name:

```javascript
// @actions/artifact 6.2.1, lib/internal/find/list-artifacts.js:131
function filterLatest(artifacts) {
    artifacts.sort((a, b) => b.id - a.id);
    const latestArtifacts = [];
    const seenArtifactNames = new Set();
    for (const artifact of artifacts) {
        if (!seenArtifactNames.has(artifact.name)) {
            latestArtifacts.push(artifact);
            seenArtifactNames.add(artifact.name);
        }
    }
    return latestArtifacts;
}
```

The original `allrun` artifact has numeric ID `8282037184921433`
and API creation time `2026-10-18T10:05:47Z`.
The rerun upload finalized with ID `6837401340876611`
and API creation time `2026-10-18T10:13:49Z`.
The older artifact has the larger numeric ID.
The consumer's download log records its original ZIP digest,
`9983d9eb096ff0b722e097b422f8dac0d79e1c9fb0c5c4a952ada12c34818d0f`,
rather than the new ZIP digest,
`4027aee6a01033fcca9578f8545764a75974ddef9c9afe41cb411f060ca501b0`.
These observations match the deciding SDK selector.
The Action's raw pre-filter list response was not captured.
The new upload's earlier finalization and later API visibility do not prove that both same-name rows
were visible to the failing consumer's list request.
The exact selector controls establish the numeric-ordering vulnerability;
they do not isolate it from a visibility race in that specific cloud incident.

Both observed IDs are safe JavaScript integers.
Numeric precision loss is not the explanation for this pair.
The private backend's ID-construction mechanism is not established.
This result also does not establish an artifact-list propagation delay.

## Verification

The exact published selector was evaluated without modifying third-party source.
The positive catalog supplies increasing IDs with increasing creation times;
the selector returns the newest artifact.
The failing catalog supplies the actual observed IDs and times;
the selector returns the older artifact.
A distinct-name control preserves both artifacts.

The actual cloud catalog includes the successful first consumer,
the full rerun's hash rejection,
and all seven original producer/consumer ZIPs retained across those attempts.
The first attempt's directory was copied before the rerun,
so the subsequent status and log collection did not replace its only evidence.
ZIP lengths,
local SHA256 values and CRC checks passed.
No native server-provided ZIP hash is claimed.

## Verified protection and workaround

The producer writes its raw receipt SHA256 to its job output.
The consumer compares the downloaded receipt's bytes with that current output.
This protection passed on the fresh run and rejected the observed stale rerun receipt.
Its tradeoff is a failing consumer when selection returns an older receipt;
it prevents false acceptance rather than completing the rerun.

Research commit `46fab80ef5` adds the producer receipt hash to each artifact name
and downloads the exact names supplied by current dependency outputs.
Actual run `w9jdpj5f1l` passed the downstream consumer on both its first attempt and full workflow rerun.
Each consumer checked the exact current producer bytes and passed six source/hash rejection controls.
The second attempt's artifacts join their current API attempt IDs;
the five jobs across both attempts have ten distinct provider sandbox IDs.
It avoids asking a same-name selector to infer chronology from IDs.
It retains separate immutable names instead of replacing old artifacts.
The [public receipt](../audit/depot-native-orchestration-2026-10-18.evidence.json)
retains fifteen original ZIPs across the failing fixed-name and passing content-bound campaigns.
Raw Action logs remain private because they contain signed download URLs.
The workaround trades unchanged names for content-addressed names;
a content-identical receipt legitimately retains its name.

## What does not work

A broad pattern with `merge-multiple: true` does not select a particular attempt.
Successful download digest checks only prove that the selected ZIP matches its advertised bytes.
They do not prove that those bytes belong to the current producer.
The same SDK's `get-artifact.js:83` also chooses the highest database ID when a fixed name has multiple matches:

```javascript
// @actions/artifact 6.2.1, lib/internal/find/get-artifact.js:83
artifact = res.artifacts.sort((a, b) => Number(b.databaseId) - Number(a.databaseId))[0];
```

Changing a broad pattern to an unchanged exact name therefore retains this selector risk.

## Upstream filing decision

No upstream message was sent.
This remains an integration finding for the current repository.

- Responsibility:
  the observed Depot IDs do not preserve the chronology assumed by the SDK selector.
  The backend mechanism is unknown.
- Fixability:
  content-bound naming is an owned consumer remedy verified on fresh and full-rerun consumers.
  No public backend patch is available to evaluate.
- Supported use:
  native workflows execute the pinned upload/download Actions in this trial.
- Contribution policy:
  the inspected Toolkit `.github/CONTRIBUTING.md` accepts issues and patches.
  That policy does not authorize an external message in this task.
- Maintainer intent:
  no evidence establishes intended Depot ID-ordering semantics.
- Minimal patch:
  the consumer change is committed and cloud-verified;
  no backend prototype is possible from the inspected public source.

The `.out-of-scope/` inventory has no matching Depot exemption.
Current `depot/cli` issue searches for `artifact rerun` returned no matches.
The broader Toolkit search for `artifact latest id` returned related artifact issues,
but none of the returned titles identifies this exact Depot rerun boundary.
Those searches do not prove the absence of every related report.
