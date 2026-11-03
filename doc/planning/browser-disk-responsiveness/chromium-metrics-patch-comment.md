# Candidate for reporting-ineligible unbranded Chromium

The accompanying patch selects the existing local-memory histogram allocator
when an unbranded Chromium process cannot report metrics.
It makes that choice before publishing histogram pointers.
Forced reporting and test overrides installed before allocator creation retain mapped storage.
Histogram recording remains enabled.

Base revision:
`360d943d418b3d6645fea86766f5dcc4a7cc3b2e`.
Patch SHA-256:
`57a30501ab2cba3fb21d815b0a750f5ce3c50d596a6ea154fd41bc28914e2a18`.

Validation used the actual Linux Chromium build:

- `chrome`,
  `chrome/test:unit_tests`,
  and `chrome/test:browser_tests` built successfully.
- All `6` accessor-policy tests and `4` selected browser tests passed.
- Removing only the startup storage choice made the new startup regression fail
  because `HasPersistentLocation()` returned `true`.
  Restoring and rebuilding the candidate made all selected tests pass again.
- In standalone Chromium `159.0.8141.0`,
  forced reporting retained a live `BrowserMetrics` file mapping.
  Normal reporting-ineligible and recording-only launches had no such mapping.
  Each mode recorded `60` boolean and `60` sparse samples through the native WebUI API;
  counts,
  sums,
  and buckets matched the inputs.
  All browsers closed normally.
- Tests ran with external networking disabled and disposable profiles.

This candidate covers the reporting-ineligible unbranded case.
Reporting-capable Chrome is outside this fix's scope.
The patch leaves delayed spare-file creation and legacy-file cleanup code unchanged.
Other operating systems and Google-branded builds were not tested.
This validation establishes storage selection and sample retention;
it does not measure a latency improvement.

Prepared with AI assistance:
Codex implemented the candidate and performed the listed checks.
