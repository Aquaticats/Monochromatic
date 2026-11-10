# Depot native macOS provenance artifact, verified 2026-11-10T14:49:50-05:00

## Why

Issues #1175 and #1176 record two transport failures inside one native macOS job on 2026-10-17:
the after-consumer provenance fetch timed out against `api.github.com`,
and the provenance upload exhausted five `CreateArtifact` attempts.
Both issues ask for the failed attempt to stay in the evidence
and for the recovery to be verified from an actual provider attempt.
This records that verification.

## The recovery execution

Job `117581375857` ("Native package gates (depot-macos-15)") in run `39095637308`,
workflow `depot-native-runner-followup-hosts`,
started 2026-10-18T04:01:18Z and completed 2026-10-18T04:08:19Z.
It ran the frozen source `cb95cf6d785c3073c192b95f0632cdae7b910894`
on the `.github/workflows/music-player-windows.yml` harness revision `3e51a06848ba363a90d4ce30816760042153f349`,
whose base revision is the failing run's `467dc0002d6080f8aa951f488430450debf3df3c`.
`doc/audit/depot-native-runner-followup-2026-10-17.evidence.json`
records both revisions under `harnesses.native`,
with `originalStepsByteValuesPreserved: true`.

The two steps that failed on 2026-10-17 succeeded in this job,
as `doc/audit/depot-native-consumers-and-variation-2026-10-18.evidence.json`
records under `proofs` for route `native.hosts`:

- Step 18,
  `Record frozen source provenance after consumers`,
   log SHA256 `db873e69d184029ea1edd47309c918aaea7e340b5ab4b7d5adb2ed5350dd9daa`.
- Step 19,
  `Preserve frozen source provenance`,
   log SHA256 `cebbe0084353b52e9518ba2b6063217237f306a51cd15089476a3d2d5d8a27d1`.

The job as a whole still failed,
at step 14 (all-target Clippy) and step 15 (the nextest body);
`gh api repos/Aquaticats/Monochromatic/actions/jobs/117581375857`
lists those two step conclusions as `failure` and steps 18 and 19 as `success`.

## The artifact

`gh api repos/Aquaticats/Monochromatic/actions/runs/39095637308/artifacts`
returns two artifacts for the run;
the macOS one is:

```text
id 12124706529
name depot-native-provenance-cb95cf6d785c3073c192b95f0632cdae7b910894-macOS-ARM64
size 11058
digest sha256:7e8c86173cc3300464144fb20aa81ea92d451d41307db1f0eecf9f25fd2a3162
created 2026-10-18T04:08:16Z
expired false
```

Downloading `repos/Aquaticats/Monochromatic/actions/artifacts/12124706529/zip`
and hashing the received bytes gives
`7e8c86173cc3300464144fb20aa81ea92d451d41307db1f0eecf9f25fd2a3162`,
so the provider digest names the bytes actually delivered.

The archive holds exactly two members,
`before.json` (17088 bytes) and `after.json` (17089 bytes).
Both parse as `event: source-provenance` records and agree on:

- `commonManifest.sha256` `1ca74543a11ad55be26851ed6f3c768b0113f4dd2237d4db7550f10d5369830f`,
  from `doc/audit/depot-complete-cohort-native-recovery-20261018T035939491Z.json`.
- `case.sourceRevision` `cb95cf6d785c3073c192b95f0632cdae7b910894`.
- `route.harnessRevision` `3e51a06848ba363a90d4ce30816760042153f349`.
- The whole `identity` object,
  including `sourceTree`
  `ee83d524cb9dbd385bddd2e745b3624873290f58`
  and the per-file hashes under `identity.inputs`.
- `expectedSource`,
  which is the same revision string.

They differ only in `phase` (`before` then `after`) and `observedAt`
(2026-10-18T04:02:08.416Z then 2026-10-18T04:08:16.030Z).
Identical identity across the two records is the source-immutability check:
the consumers ran between them and changed no tracked byte.

## The failed attempt stays recorded

`doc/audit/depot-coherent-github-gates-2026-10-17.evidence.json`
keeps the first attempt's boundary at top level as
`nativeMacFirstAttemptPairedProvenance: false`,
and its payload holds the raw logs:

- `ConnectTimeoutError: Connect Timeout Error (attempted address: api.github.com:443, timeout: 10000ms)`
  under the `Record frozen source provenance after consumers` step.
- Four `Attempt N of 5 failed with error: Request timeout: .../ArtifactService/CreateArtifact`
  lines plus the terminal `Failed to make request after 5 attempts`,
  under the `Preserve frozen source provenance` step.

## Limits

- One recovery execution and one artifact show the paths completing once,
  not a success distribution or a latency figure.
- The transport cause of the 2026-10-17 failures is not attributed.
  The recovery ran later,
  so it neither proves nor disproves a provider-side cause.
- The harness that produced both runs is gone from the tree:
  commit `5ebb947a9d`,
  2026-11-01,
  `chore(depot): delete the .depot folder`,
  removed `.depot`,
  and the trial branch the runs used is not a branch of this repository.
- The artifact expires under the provider's retention policy,
  so the digest above is the durable record of its contents.
