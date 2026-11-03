# Chromium live histogram upstream contribution

## Reproduction verified at 2026-11-03T10:03:12+00:00

The user requested:
"Lets do the 'Chromium's live histogram map ping' upstream contribution now.
Flag any blockers."
They subsequently required collecting missing fresh upstream reproduction evidence now,
rather than treating its absence as a reason to stop.
The user filed [Chromium issue 581090722](https://issues.chromium.org/issues/581090722).
Its public page was verified to contain the report,
the `Internals>Metrics` component,
and the sanitized JSON attachment.

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

## Completion and handoff

The fresh capture,
symbolization,
native histogram counts,
browser rendering of those counts,
trace-loss checks,
and sanitization are complete.
Commit `51da9d59db` retains the reproduction clients and verification tasks.
The source-selection pass verified `22` citations with `agent-browser`.
Scoped Markdown lint and Pandoc rendering passed;
the final verification record records the delivered source hashes.

All owned browser processes and the tracefs instance are gone.
The primary browser exited normally.
A separate simultaneous-scope teardown emitted a zygote fatal error;
the corrected `Browser.close` lifecycle check exited normally without that diagnostic.
The existing user-authorized Helium recorder was not changed.

The bundled Browser plugin and its browser-control server are disabled in user configuration.
A fresh uncached local app-server catalog confirmed no Browser skills.
The approved agent rules and generated copy are committed.

The initial upstream report is posted.
For the live form,
the advised role is `Chromium Contributor`
and the advised template is `Defect on Linux`;
the user supplied those exact available choices.
Do not request sign-in through a testing browser.
Human maintainer feedback requires a human reply.

## Active follow-up work

- The user requested a script to watch the issue for at least the next `24` hours.
  Anonymous read-only HTTP requests used by the public page have passed live checks.
  The watcher is running under `${HOME}/.local/state/chromium-issue-watch/581090722`.
  Its saved deadline is `2026-11-04T12:07:36.851Z`,
  with checks every `5` minutes.
  The [watch record and retained sources](chromium-issue-581090722-watch.md)
  document the active service,
  successful live reads,
  resource bounds,
  and functional/type checks.
- The user then asked why no patch had been prepared and attached.
  Prepare and test a candidate patch.
  The [source assessment](../planning/browser-disk-responsiveness/chromium-metrics-patch.md)
  recommends initial local-memory allocation for reporting-ineligible unbranded processes,
  preserving forced-reporting and test overrides.
  No C++ implementation or test result is claimed yet.
- The user approved the `HQP` readiness-check addition to the troubleshooting skill.
  It requires an attempted authorized fix prototype before a filing handoff,
  with explicit report-only scope or a concrete prototype blocker as the alternatives.
  The source rule is edited and generated copies are synchronized.
  Markdown lint passed;
  the first narrowly targeted skill-lint invocation checked no files and does not count.
