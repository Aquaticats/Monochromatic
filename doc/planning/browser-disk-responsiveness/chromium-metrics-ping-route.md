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
The absence of a latest-build reproduction is an evidence limit for this initial owner ping;
the ping must not describe the Helium observation as a newly reproduced current-main Chromium bug.

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
The public [issue page][segment-issue] fetch returned only the application shell;
the issue's contents and access policy were not established by that response.
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
The missing latest-build reproduction and deterministic regression fixture must remain explicit.
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

## Verification handoff

The source and Gerrit checks were read-only.
Nontrivial commands used `systemd-run --user --scope`
with `CPUQuota=200%`,
`MemoryMax=2G`,
`TasksMax=128`,
and `IOWeight=10`.
The main task owns the combined Markdown lint,
rendered-document inspection,
and browser verification of the source selections before publishing this document.

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
