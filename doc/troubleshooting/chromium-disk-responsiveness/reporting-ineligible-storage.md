# Candidate local histogram storage for reporting-ineligible Chromium

## Candidate verified at 2026-11-03T19:37:50+00:00

The [patch](reporting-ineligible-storage.patch) targets upstream Chromium revision
`360d943d418b3d6645fea86766f5dcc4a7cc3b2e`.
Native compilation passed for `chrome`,
`chrome/test:unit_tests`,
and `chrome/test:browser_tests`.
The native tests,
deliberately failing control,
restored-candidate retest,
and live browser checks passed.
The [verification record](reporting-ineligible-storage-verification.json)
retains the actual results for this patch.

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
identify the formatted files consumed by the verified build.

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

## Verification results

- The actual browser and native test executables built successfully.
- The native launcher emitted the expected names and successful results for
  `6` accessor-policy tests and `4` browser tests.
  The browser tests cover startup recording,
  file retention,
  consent-driven file deletion,
  and sampled-out reporting.
- Removing the startup storage choice made
  `NonReportingPersistentHistogramsBrowserTest.RecordsWithoutPersistentLocation`
  fail at `allocator->HasPersistentLocation()`:
  actual `true`,
  expected `false`.
  The candidate was restored and rebuilt;
  all `10` selected native tests passed again.
- Standalone Chromium `159.0.8141.0` ran with fresh disposable profiles.
  The forced-reporting positive control had `1` live `BrowserMetrics` mapping.
  The ordinary reporting-ineligible and recording-only runs each had `0`.
- Each live mode recorded `60` boolean and `60` sparse samples through
  `chrome.metricsPrivate`.
  Counts,
  sums,
  and individual sample buckets matched the generated inputs.
  Each browser acknowledged `Browser.close` and exited `0` without a fatal shutdown diagnostic.
- External networking was disabled for all native tests and live-browser runs.
  Source checksums still matched after candidate restoration and live verification.

This validates unbranded Linux Chromium.
Google-branded builds and other operating systems were not exercised.
No latency comparison or injected filesystem delay was used.

## Submission packet

The [follow-up comment](../../planning/browser-disk-responsiveness/chromium-metrics-patch-comment.md)
is prepared for the existing issue.
No patch attachment or follow-up comment has been posted.
The user declined sign-in through a testing browser;
submission remains through their normal browser.

Chromium's [AI policy][ai-policy] says:
"Authors **must** self-review and understand all code and documentation updates
(with or without AI tooling) before sending them for review".
The human submitter must review and understand this candidate.
The policy also requires a human response to human maintainer feedback.

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
This preparation checkpoint did not yet include completed compilation or behavioral verification.

## Formatted candidate prepared at 2026-11-03T18:21:27+00:00

Chromium's formatter produced indentation and line-wrapping changes.
They were applied in the separate prototype and committed as `ab9cdcd4e3`.
The retained patch includes that formatting.
The original compilation was intentionally interrupted before its inputs changed.
The `build:formatted` task applied the exact formatter delta,
verified every changed file against the retained checksums,
and confirmed the changed-path set and whitespace checks.
Chromium's formatter then exited successfully with no remaining diff.
Compilation resumed at `2026-11-03T18:26:39.515Z` using the retained build outputs.

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

## Native build verified at 2026-11-03T19:21:54+00:00

The final native build exited `0` at `2026-11-03T19:21:54.478Z`.
Its recorded elapsed time was `3314963 ms` after resuming from the retained outputs.
The final-source record confirms the exact candidate patch and source checksums.
This is compilation evidence;
no native test pass or browser behavior is inferred from it.

The [upstream source check](upstream-source-check.json) also compared the patch-base files
against observed main revision `4bc632656c7b95bc4c1c8c6a891e63396bdd0d52`.
All modified source paths retained their original Git blob IDs.
The `DEPS` control had different blob IDs,
demonstrating that the comparison detects a changed input.
The build remains pinned to its recorded target revision.

[ai-policy]: https://github.com/chromium/chromium/blob/1ff392dbf4e593ae50b633f6327c06365fd5b8d9/agents/ai_policy.md?plain=1#L6-L33
