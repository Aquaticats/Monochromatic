# Chromium live histogram mapping contribution route

## Source check at 2026-11-03T09:27:54+00:00

The authorized task is the initial upstream ping about the observed live histogram mapping stall.
This source check does not approve or implement a metrics persistence redesign.
No upstream message was sent by this research pass.
The parallel source check uses the same model vendor as the main agent;
it is not an independent review.

Current source was checked at `1ff392dbf4e593ae50b633f6327c06365fd5b8d9`,
whose commit time is `2026-11-03T09:16:44Z`.
The [incident evidence](../../troubleshooting/helium-window-freezes.md)
and [upstream proposal](chromium-upstream.md)
remain the sources for the observed mechanism and proposed follow-up work.

## Reporting route

[Chromium's contribution guidance][contributing]
asks contributors to discuss existing-code changes with owners before going far.
It identifies bugs as the place for history,
discussion,
attachments,
and links to related issues.

The [current bug reporting guidelines][reporting]
link the public tracker at <https://issues.chromium.org/new>.
They require a Google Account and ask reporters to verify against the latest Chromium or Chrome Canary.
The main task is now acquiring and exercising a fresh upstream Chromium build.
The ping must use the resulting measured evidence;
the Helium observation alone is not a newly reproduced current-main Chromium bug.

The owning issue component is `Internals>Metrics`,
public component ID `1456863`,
as declared in [the metrics common metadata][metadata].
Both [base metrics metadata][base-metadata]
and [components metrics metadata][components-metadata] include that declaration.
The public tracker is the documented route for a durable issue.

An anonymous HTTP request to the legacy new-issue URL from `docs/contributing.md`
redirected to Google sign-in with `https://issues.chromium.org/issues/new` as the continuation.
The constructed URL `https://issues.chromium.org/issues/new?component=1456863`
also redirected to sign-in and preserved the component parameter.
This does not verify component-prefilling behavior after authentication.
Use the documented tracker route and the source-verified component name until the form is inspected.

[Metrics owners][owners] prefer `chromium-metrics-reviews@google.com`
for code-review assignment.
The file does not establish that alias as a public bug-reporting mailbox.
[Fallback metrics owners][fallback-owners] include Alexei Svitkine,
author of the related system-profile scanning change,
but also say to prefer the review alias for reviews.

## Existing work and mechanism boundaries

[CL 8707393][segment-cl],
"Cache system profile segment refs in RecordAllocator",
remained `NEW` when fetched from Gerrit's change-detail API.
Its last update was `2026-11-02T20:57:57Z`.
The description connects cold mapped-page reads during ChromeOS shutdown/profile updates
to main-thread shutdown hang crashes.
It caches system-profile segment references to avoid scanning unrelated allocations.
Its `Bug:` footer is `580929788`.
The public [issue page][segment-issue] fetch returned only the application shell.
The main task's subsequent `agent-browser` inspection rendered
"Access is denied to this issue"
and stated that access "may be resolved by signing in".
The issue's contents remain unavailable in that anonymous session;
the page does not prove that a signed-in account would have access.
This is related evidence,
not proof that the observed live mapped-write fault has already been reported or fixed.

[CL 8688624][sparse-cl],
"[Metrics] Batch persistent sparse histogram imports during merges",
remained `NEW`,
last updated `2026-10-30T23:03:01Z`.
It moves allocator scanning outside `SparseHistogram::lock_`
and batches imports to reduce UI/IO contention.
Its current commit message has no `Bug:` footer.
Lock contention during import is distinct from a caller faulting on a live regular-file-backed write.

The broadened Gerrit searches also found:

- [CL 8041741][readonly-cl],
  still `NEW`,
  about a segmentation fault from writing a read-only histogram mapping.
  Its described mechanism is memory protection,
  not filesystem writeback latency.
- [CL 8454728][torn-cl],
  still `NEW`,
  about reading torn system-profile records after an unclean termination.
  Its described mechanism is recovery consistency,
  not the live recording stall.

The searches covered `message:BrowserMetrics`,
open persistent changes under `base/metrics`,
and `message:histogram message:fault` in `chromium/src`.
The Gerrit result sets had no `_more_changes` marker.
Broader web searches included `BrowserMetrics`,
`btrfs_page_mkwrite`,
`mmap`,
freeze,
and UI hangs.
Directory-growth and disk-full reports were not treated as the same mechanism.
No assertion is made that an exact duplicate or unpublished fix does not exist.

## Human obligations and evidence limits

The [current Chromium AI policy][ai-policy] explicitly discusses an AI agent filing a bug.
It does not categorically forbid an agent-authored initial report.
Its rule for subsequent discussion is:
"a human reply must get a human reply".
If a maintainer responds,
the human operating the account must reply to that feedback.
The policy's separate self-review requirement applies to code and documentation updates sent for review.
Do not transform that clause into an unsupported universal approval requirement for initial bug filing.

The current contribution is an evidence-bearing owner ping.
The main task completed a fresh upstream reproduction in Chromium `159.0.8141.0`.
The [public report](chromium-metrics-ping.md)
and [sanitized evidence](chromium-metrics-ping-evidence.json) contain the result.
The deterministic regression fixture remains a separate deliverable;
do not substitute a static call-chain analysis for the running-browser result.
The incident captured Helium's browser main thread writing through a regular-file-backed `BrowserMetrics` mapping;
ordinary Chromium filesystem waits without an identified mapping are separate observations.
A future patch needs its own tests,
human understanding,
and the applicable review requirements.
No such patch is claimed here.

Account access and the human operator's future replies remain practical dependencies of an upstream conversation.
Public HTTP reads do not prove access to a signed-in browser or permission to post.
The main task owns authentication checks,
the final sanitized message,
submission,
and any user-facing blocker question.

## Reproduction preparation at 2026-11-03T09:37:40+00:00

The [official download guide][downloads] identifies the Chromium snapshot archive
and its `LAST_CHANGE` pointer.
The Linux x64 pointer returned `1733707` when this source pass selected a build.
Its [revision manifest][snapshot-revisions] identifies
`986947eb4afc18471e51b7cb473f7c2f779f62bf`,
commit position `refs/heads/main@{#1733707}`.
The corresponding [version source][snapshot-version] is `159.0.8141.0`.
The main task reported that a later pointer read returned `1733712`;
`1733707` therefore means latest at selection,
not a permanently latest build.
The selected archive is
<https://storage.googleapis.com/chromium-browser-snapshots/Linux_x64/1733707/chrome-linux.zip>.
The guide warns that snapshots are best effort builds from arbitrary revisions and lack symbols.
The tested Linux binary nevertheless retained function symbols:
`addr2line` resolved the captured instruction to `base::SampleVectorBase::Accumulate(int, int)`
and `sample_vector.cc`,
without a source line number.

The selected build already has a browser-main recording entry point without a source patch.
Its [API availability declaration][api-availability] exposes `chrome.metricsPrivate`
to trusted WebUI pages including `chrome://settings/` and `chrome://extensions/`.
Its [recording implementation][api-recording] directly calls `UmaHistogramBoolean`
and `UmaHistogramSparse` and responds synchronously on the native side.
The [dispatcher contract][api-dispatcher] says it lives entirely on the UI thread,
and the [WebUI dispatch branch][api-webui-dispatch] invokes the function inline.
This is not an ordinary website API.

A disposable `agent-browser` session exercised the route on its bundled
`HeadlessChrome/149.0.0.0` runtime.
On `chrome://settings/`,
`typeof chrome.metricsPrivate.recordBoolean`
and `typeof chrome.metricsPrivate.recordSparseValue` both returned `function`.
`getHistogram` returned `undefined`;
the API availability file restricts that function separately.
The finite probe recorded `true` in `Monochromatic.BrowserMetricsRepro.Boolean`
and `42` in `Monochromatic.BrowserMetricsRepro.Sparse`.
The rendered histogram page showed exactly `1` sample at boolean bucket `1`
and exactly `1` sample at sparse bucket `42`.
The same counts remained after disabling subprocess imports.
The recording API has no JavaScript completion callback;
renderer evaluation finishing only proves dispatch,
so the histogram count and a browser-main responsiveness probe must verify completion separately.
This is an API smoke check,
not the fresh headed Chromium reproduction or proof of a regular-file mapping.
The main task owns that measurement on the selected upstream binary.

[The histogram page handler][histogram-handler] runs on the UI thread
and supports observation and monitoring,
not a direct sample-recording message.
Its optional subprocess import can run synchronously.
Disable that checkbox when isolating browser-process observations.
The [page source][histogram-query] documents `chrome://histograms/#NAME`
as the canonical filter form;
the historical pathname form can be misclassified by filename suffix.
The runtime smoke used
`chrome://histograms/#Monochromatic.BrowserMetricsRepro`.

## Verification results

The source and Gerrit checks were read-only.
Nontrivial commands used `systemd-run --user --scope`
with `CPUQuota=200%`,
`MemoryMax=2G`,
`TasksMax=128` for source commands or `512` for browsers,
and `IOWeight=10`.
All `13` original source and policy citation targets passed the disposable
`metrics-ping-route-check` browser pass at `2026-11-03T09:36:24.681Z`.
GitHub line selections matched the fetched source and were highlighted.
Other pages contained their expected rendered excerpts,
including the denied-access message for issue `580929788`.
The browser session closed successfully.
The additional `9` reproduction-source citations passed their own browser pass
at `2026-11-03T09:40:12.129Z`.
Its browser and the API-smoke browser closed successfully;
no owned `metrics-ping-*` systemd units remained.
Scoped `mise run lint:monochromatic-lint` passed for this document.
The main task owns the combined rendered-document inspection.

[contributing]: https://github.com/chromium/chromium/blob/1ff392dbf4e593ae50b633f6327c06365fd5b8d9/docs/contributing.md?plain=1#L19-L34
[reporting]: https://www.chromium.org/for-testers/bug-reporting-guidelines/#:~:text=You%20need%20a%20Google%20Account
[metadata]: https://github.com/chromium/chromium/blob/1ff392dbf4e593ae50b633f6327c06365fd5b8d9/base/metrics/COMMON_METADATA#L1-L6
[base-metadata]: https://github.com/chromium/chromium/blob/1ff392dbf4e593ae50b633f6327c06365fd5b8d9/base/metrics/DIR_METADATA#L1
[components-metadata]: https://github.com/chromium/chromium/blob/1ff392dbf4e593ae50b633f6327c06365fd5b8d9/components/metrics/DIR_METADATA#L1-L3
[owners]: https://github.com/chromium/chromium/blob/1ff392dbf4e593ae50b633f6327c06365fd5b8d9/base/metrics/OWNERS#L1-L8
[fallback-owners]: https://github.com/chromium/chromium/blob/1ff392dbf4e593ae50b633f6327c06365fd5b8d9/base/metrics/METRICS_OWNERS#L1-L18
[segment-cl]: https://chromium-review.googlesource.com/c/chromium/src/+/8707393
[segment-issue]: https://issues.chromium.org/issues/580929788
[sparse-cl]: https://chromium-review.googlesource.com/c/chromium/src/+/8688624
[readonly-cl]: https://chromium-review.googlesource.com/c/chromium/src/+/8041741
[torn-cl]: https://chromium-review.googlesource.com/c/chromium/src/+/8454728
[ai-policy]: https://github.com/chromium/chromium/blob/1ff392dbf4e593ae50b633f6327c06365fd5b8d9/agents/ai_policy.md?plain=1#L6-L33
[downloads]: https://www.chromium.org/getting-involved/download-chromium/#:~:text=The%20latest%20one%20is%20mentioned%20in%20the
[snapshot-revisions]: https://storage.googleapis.com/chromium-browser-snapshots/Linux_x64/1733707/REVISIONS
[snapshot-version]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/chrome/VERSION#L1-L4
[api-availability]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/extensions/common/api/_api_features.json#L392-L432
[api-recording]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/extensions/browser/api/metrics_private/metrics_private_api.cc#L193-L207
[api-dispatcher]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/extensions/browser/extension_function_dispatcher.h#L35-L37
[api-webui-dispatch]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/extensions/browser/extension_function_dispatcher.cc#L370-L377
[histogram-handler]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/content/browser/metrics/histograms_internals_ui.cc#L107-L155
[histogram-query]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/content/browser/resources/histograms/histograms_internals.ts#L87-L108
