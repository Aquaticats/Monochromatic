# Expanded Depot execution on 2026-10-16

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/logger` is now `package/logger/core`.

## Purpose and scope

This report records actual browser and native checks in an isolated trial branch,
`trial/depot-expanded-20261016`,
based on `d6deca5fa570d6ba71afdab837e508a9f7ff6b0e`.
It does not authorize production routing or a billing-plan change.

The executed scope was:

- The existing logger and filesystem browser suites in Chromium,
  Firefox,
  and WebKit with explicit nonempty result counts.
- The native audio core with default,
  fingerprint-only,
  service-only,
  and all features,
  and its existing Clippy task.
- The existing DOM interaction suites in Chromium and Firefox.
- A deliberately wrong assertion in an existing browser test.

The native feature matrix and DOM suites passed.
The logger/filesystem browser job executed every selected engine,
but retained one failing WebKit availability assertion and one skipped WebKit storage fixture.

## Initial evidence

`pnpm-lock.yaml` resolves `@playwright/test` to `1.64.0` at the trial base,
while `playwright.Dockerfile` selects `1.63.0`.
The experiment used the published `1.64.0-noble` OCI index digest
`sha256:06a9939e57531807f8d5fd76ce44b53165ffb7d7501d87ab10e285c20b1e971f`.
Registry verification required an Accept header that included OCI image indexes;
a bare HEAD request returned 404,
while the OCI-aware request returned 200 and this digest.
The first response does not establish that the image is unavailable.

The first [Depot run][first-run] uses revision
`a21e6369936689a962de528dce0a0f8b9e22f4fe`.
The native job completed independently.
The browser preparation job was cancelled to replace an unnecessary browser-install workaround
with the verified matching image.
No test outcome is claimed from that cancelled browser job.

## Native results

The native job `sfjhrbwvrs`,
attempt `2gv4q0p1bd`,
finished successfully in 157.920 seconds on 4 CPUs and 16 GiB RAM.
The parent run is marked cancelled because its separate browser preparation job was cancelled;
the native job's own terminal conclusion is success.

- Default features:
  58 passed,
  zero skipped.
- Fingerprint only:
  67 passed,
  zero skipped.
- Service only:
  71 passed,
  zero skipped.
- All features:
  80 passed,
  zero skipped.
- Existing all-features,
  all-targets Clippy task:
  passed with warnings denied.

These are 276 test executions covering 80 distinct test names across feature configurations.
The [companion evidence](depot-expanded-execution-2026-10-16.evidence.json)
preserves every test name,
raw resource samples,
API timestamps,
and the log hash.
The 68 historical events already classified as `musicCore` are the corresponding workload selector.
Use this expanded job as a replacement for the previous core all-features baseline when modeling cost;
adding both would count the old all-features test twice.

## Browser results

The [rootful browser run][browser-rootful],
job `pp3m2jb9sj`,
attempt `08s1x6qxrt`,
executed 69 cases with retries disabled.

- Chromium:
  23 passed.
- Firefox:
  23 passed.
- WebKit:
  21 passed,
  one failed,
  one skipped.

The failure is `verify detects OPFS availability` at
`package/module/logger/src/sink/opfs.browser.test.ts:41`:
the test expected `true`,
but sink verification returned `false`.
The filesystem OPFS fixture skipped after an `UnknownError` reported that storage refused the operation.
Those observations do not establish a browser defect or a memory shortage.
[Issue #1052][opfs-issue] tracks the current test's unconditional availability assumption.

The whole browser job took 151.586 seconds;
the Playwright report's test duration was 69.817495 seconds.
This is a measured failing baseline,
not a fully successful three-engine run.
Its job includes logger and filesystem builds,
but does not include the previously measured logger type checks or unit tests.
For the monthly model,
add this separate job to the earlier logger job;
sharing builds or setup across jobs remains unmeasured.

The existing DOM scenarios in [run `zlhbx5k5rq`][followup-run],
job `zw591vv262`,
attempt `106fpb437j`,
passed 34 cases:
17 in Chromium and 17 in Firefox,
with zero skips and failures.
They cover the existing dialog prompt,
redirect,
element duplication,
and CSS parameter helper suites.
The whole job took 117.823 seconds;
the Playwright test duration was 44.274455 seconds.
WebKit is not configured by the repository's e2e config.

Both browser workload selectors match 50 events in the historical audit.
Their stored event lists include shared root inputs and current-manifest reverse consumers
of the packages under test.
This conservative selector is a proposal for modeling,
not evidence that these tests ran historically.

## Failure control

The separate browser control changed a real logger assertion from expecting `function`
to expecting `deliberately-wrong`.
That same test had passed in the unchanged browser baseline.
The control's report contains exactly one Chromium case,
one failed assertion,
zero skips,
and the expected-versus-received mismatch at the changed source line.
The task and job both failed.

The control is attempt `kfnj5n805m` in [run `zlhbx5k5rq`][followup-run].
Its 92.450-second whole-job duration is verification overhead,
not a recurring workload estimate.
The fixture restores the source in a `finally` block.

## Browser preparation and recovery

The [matching-image run][browser-rootless] stopped before browser tests.
Podman exited 125 because `newuidmap` rejected the process identity:
`uid:1001 pw_uid:1001 st_uid:1001, gid:1001 pw_gid:117 st_gid:1001`.
The isolated [rootful Podman probe][browser-rootful] built the container successfully
and ran the browser suites without that user-mapping rejection.
The [troubleshooting record](../troubleshooting/depot-rootless-podman-group-identity.md)
contains the identity probe,
source trace,
verified workaround,
and its storage-ownership tradeoffs.
The repository integration gap is [issue #1047][identity-issue].
The image version mismatch is tracked by [issue #1045][image-issue].

The successful image-preparation job took 137.525 seconds on 4 CPUs and 16 GiB RAM.
It reused the earlier Node/dependency image;
the original creation cost of that image is outside this measurement.
Allocate browser-image preparation once per chosen image rebuild event,
shared by its consumers,
not once per browser job.

## Measurement limits and remaining coverage

Each final job has one measured run.
No claim about run-to-run timing spread,
runner-size improvement,
or warm-versus-cold improvement is made.
The evidence preserves raw API samples and timestamps for later modeling.
The JSON reporter was extracted from job logs by removing timestamp prefixes
and identifiable interleaved task/npm notices;
the remaining JSON parsed successfully,
and individual case counts reconcile with reporter totals.

The measured browser failure remains open.
Other native application packages,
other browser packages,
and missing application user-flow tests remain outside this representative execution.
Existing issues #1023,
#1025,
#1026,
and #1030 retain those broader gaps.
No experiment workflow or rootful Podman wrapper was merged into the production checkout.

[first-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/p9sfmw70wq
[browser-rootless]: https://depot.dev/orgs/l4zncjjbj5/workflows/t3jdjq2nd3
[browser-rootful]: https://depot.dev/orgs/l4zncjjbj5/workflows/5f1hbcxtws
[image-issue]: https://github.com/Aquaticats/Monochromatic/issues/1045
[identity-issue]: https://github.com/Aquaticats/Monochromatic/issues/1047
[opfs-issue]: https://github.com/Aquaticats/Monochromatic/issues/1052
[followup-run]: https://depot.dev/orgs/l4zncjjbj5/workflows/zlhbx5k5rq
