# Chromium live histogram upstream contribution

## Reproduction verified at 2026-11-03T10:03:12+00:00

The user requested:
"Lets do the 'Chromium's live histogram map ping' upstream contribution now.
Flag any blockers."
They subsequently required collecting missing fresh upstream reproduction evidence now,
rather than treating its absence as a reason to stop.
No upstream message has been sent.

## Work items

- Fresh Chromium reproduction completed:
  official Linux snapshot `1733707`,
  Chromium `159.0.8141.0`,
  source revision `986947eb4afc18471e51b7cb473f7c2f779f62bf`.
  A headed browser with a disposable Btrfs profile produced matching browser-main write faults
  in its live `BrowserMetrics` mapping.
  Both faulting instructions symbolize to `base::SampleVectorBase::Accumulate(int, int)`,
  specifically `lock xadd %r12d,(%rax)`.
  The current build recorded `60` ordinary and `60` sparse samples through
  `chrome.metricsPrivate.recordBoolean` and `recordSparseValue` from `chrome://settings/`.
  Rendered counts matched with subprocess imports disabled.
  Sampled blocked spans were `104 ms` and `1484 ms`;
  a browser-window command took `1159.963 ms` within the latter span.
  These are not exact fault durations or input-to-presentation latency.
- Upstream message:
  [draft](../planning/browser-disk-responsiveness/chromium-metrics-ping.md)
  and [sanitized historical evidence](../planning/browser-disk-responsiveness/chromium-metrics-ping-evidence.json)
  were committed in `364e8a8bb9`.
  Commit `58f62c8c92` replaced the missing-reproduction wording with fresh-build evidence.
  The source-checked route is `Internals>Metrics`,
  component `1456863`,
  through <https://issues.chromium.org/new>.
  Related issue `580929788` denies anonymous access;
  its discussion remains unknown.
- Submission access:
  the user explicitly declined signing into a testing browser,
  citing concern for their Google Account.
  The isolated sign-in window was closed.
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
The [retained reproduction clients](../troubleshooting/chromium-disk-responsiveness/live-histogram-reproduction.md)
preserve the executed source and its timing/isolation limits.
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
