# Candidate local histogram storage for reporting-ineligible Chromium

## Candidate status at 2026-11-03T09:29:22-05:00

The [patch](reporting-ineligible-storage.patch) targets upstream Chromium revision
`360d943d418b3d6645fea86766f5dcc4a7cc3b2e`.
It is implemented but has not yet passed native compilation or browser verification.
Do not treat the earlier unmodified Chromium reproduction as validation of this patch.

The user requested a patch to accompany
[issue 581090722](https://issues.chromium.org/issues/581090722).
The [source assessment](../../planning/browser-disk-responsiveness/chromium-metrics-patch.md)
records the storage,
reporting-policy,
consent,
and allocator-lifetime evidence.
The [build research](../../planning/browser-disk-responsiveness/chromium-metrics-build.md)
records the selected native toolchain and test targets.
The [retained validation drivers](native-validation.md) record the actual test and browser checks,
including the failing startup control and candidate restoration.
The [source checksums](reporting-ineligible-storage.sha256)
identify the formatted files that the final build must consume.

## Intended behavior and limits

The patch exposes the existing build/reporting eligibility condition through the metrics accessor.
Chrome's accessor adds its existing test-consent override to that condition.
At initial allocator creation,
Chrome chooses local memory for a process that cannot report metrics.
The choice precedes publication of histogram pointers.

Google-branded builds,
explicitly forced reporting,
and installed test overrides retain their existing persistent-storage selection.
The recording-only switch does not independently enable reporting eligibility.
The candidate preserves the global allocator and ordinary/sparse histogram recording.

This scope addresses the reproduced unbranded Chromium case.
It does not solve regular-file-backed live recording in reporting-capable Chrome.
Anonymous allocation can still encounter memory pressure and swap.
Existing delayed spare-file work and legacy-file cleanup remain unchanged;
the candidate does not promise an absence of all metrics-related disk activity.

## Verification to complete

- Build the actual patched Chromium and its native unit/browser test executables.
- Run the accessor policy cases and the browser-startup regression,
  checking the emitted test names.
- Remove the startup storage choice in the disposable build,
  rebuild,
  and show that the startup regression fails before restoring the candidate.
- Run the patched browser with a disposable profile,
  inspect its active mappings,
  and record ordinary and sparse samples through the browser's real WebUI API.
- Exercise forced reporting with external networking disabled and verify its persistent selection remains.

No passing result is claimed for these pending checks.
The candidate is retained so implementation and its limits remain reviewable while validation proceeds.

## Build preparation verified at 2026-11-03T17:40:12+00:00

The exact target revision was fetched using a verified public-source cache as input.
Dependency synchronization,
patch application,
dependency hooks,
and GN generation passed in the isolated CachyOS container.
Native compilation started at `2026-11-03T17:40:12.725Z`.
The build retains its logs in the named container volume;
the [handover](../../handover/chromium-live-histogram-contribution.md)
records the resource bounds and acquisition evidence.
Compilation and behavioral verification remain pending.

## Formatted candidate prepared at 2026-11-03T18:21:27+00:00

Chromium's formatter produced indentation and line-wrapping changes.
They were applied in the separate prototype and committed as `ab9cdcd4e3`.
The retained patch includes that formatting.
The active compilation still uses its original inputs;
the prepared `build:formatted` task will apply the exact formatter delta after that process stops,
verify every changed file against the retained checksums,
and build the final candidate.

The native test driver now selects the actual `MetricsServiceBrowserDoUploadTest`
and `MetricsServiceBrowserNoUploadTest` fixtures.
Its initial filter named their shared base fixture,
which contains no tests.
No native run had yet used that incomplete filter.
The corrected checks include those file-policy paths,
sampled-out reporting,
and the new reporting-ineligible startup regression.
The final order is native tests,
the deliberately failing control and candidate restoration,
then the live browser check on the restored candidate.
