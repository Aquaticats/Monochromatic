# Chromium live histogram upstream contribution

## Active work at 2026-11-03T09:40:00+00:00

The user requested:
"Lets do the 'Chromium's live histogram map ping' upstream contribution now.
Flag any blockers."
They subsequently required collecting missing fresh upstream reproduction evidence now,
rather than treating its absence as a reason to stop.
No upstream message has been sent.

## Work items

- Fresh Chromium reproduction:
  downloaded official Linux snapshot `1733707`,
  Chromium `159.0.8141.0`,
  source revision `986947eb4afc18471e51b7cb473f7c2f779f62bf`.
  Run a headed browser with a disposable Btrfs profile and correlate browser-main write faults
  with that process's live `BrowserMetrics` mapping.
  A safe native recording route was source-checked and smoke-tested in a separate older test browser:
  `chrome.metricsPrivate.recordBoolean` and `recordSparseValue` from `chrome://settings/`.
  That smoke test is not the fresh-build reproduction.
- Upstream message:
  [draft](../planning/browser-disk-responsiveness/chromium-metrics-ping.md)
  and [sanitized historical evidence](../planning/browser-disk-responsiveness/chromium-metrics-ping-evidence.json)
  were committed in `364e8a8bb9`.
  Replace the missing-reproduction wording with actual fresh-build findings after the run.
  The source-checked route is `Internals>Metrics`,
  component `1456863`,
  through <https://issues.chromium.org/new>.
  Related issue `580929788` denies anonymous access;
  its discussion remains unknown.
- Submission access:
  the user explicitly declined signing into a testing browser,
  citing concern for their Google Account.
  Close the isolated sign-in window.
  Do not ask them to authenticate in a testing browser again.
  Prepare the verified report for their normal browser.
  A human maintainer reply must receive a human reply under Chromium's policy.
- Agent behavior:
  the user approved tightened `PX1` and `CB1` wording.
  `AGENTS.md` and generated `CLAUDE.md` were committed in `93a9b867d5`.
  The user also requested disabling the bundled Browser plugin.
  User Codex configuration now sets `browser@openai-bundled.enabled = false`
  and `mcp_servers.node_repl.enabled = false`.
  `codex mcp get node_repl --json` confirms the latter.
  Continue using `agent-browser`.

## Evidence and limits

The historical Helium capture directly correlates its main-thread write fault with a live `.pma` mapping.
The observed `6622 ms` interval is sampled blocked time,
not exact fault latency or input-to-presentation latency.
Ordinary Chromium's historical filesystem waits have no mapping attribution;
keep that observation separate until fresh evidence establishes one.
The [route research](../planning/browser-disk-responsiveness/chromium-metrics-ping-route.md)
distinguishes current mapped-read and sparse-lock changes from the live-write case.

The downloaded snapshot lives under `${HOME}/temp/agent/chromium-live-repro-1733707`.
The bounded collector is `${HOME}/temp/agent/chromium-live-fault-trace.ts`.
Only disposable state may be mutated for verification.
Synthetic host load remains prohibited.
Source research uses a same-vendor subagent,
not independent review.

## Remaining verification

Complete the fresh-browser capture and inspect the consumer result,
then update the public draft and sanitized artifact.
Finish scoped Markdown lint and rendered checks,
collect every command and cell result,
close owned browser sessions,
stop owned tracing and cgroups,
and record final status and commits.
