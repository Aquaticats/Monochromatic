# Native instruction capture capacity repair

## Problem

Normal prompt submission in the restarted native Pi host fails with bypass off.
The displayed `RunPromptCustodyError` hides the deciding `SourceCollectionError`.
The small-workspace CLI test did not cover repository instructions and the full normal tool set.

The initial failure is the entire serialized callback-option bundle being subjected to a single-text bound:
`native/contract/collector/run-prompt-custody-v3/handler-option-inputs.mjs:16`
within `package/pi-plugin/auto-mode` rejects 83938 bytes against 65536 bytes.
The real configured tool-loadout bundle is 113905 bytes,
also rejected by `loaded-pool.mjs:42`.
Their upstream copying paths already admit complete bundles up to one MiB.

With process-local controls,
the next deciding checks are:

- The base-linked source snapshot in
  `native/contract/collector/rule-relevance-sdk-copy/input/resource-bindings.mjs:136`:
  2850095 bytes against one MiB.
- With user and project skills,
  event receipts in `run-prompt-custody-v3/loaded-event-inputs.mjs:44` exceed the single-record limit.
- A follow-up prompt reaches 5282773 bytes against the run owner's four-MiB charge limit.
- The shared check in `src/input-receipt-budget.ts:171` rejects
  13 resource records occupying 158177 bytes plus 110 session records occupying 4298044 bytes.
  The count fits 128;
  the byte total exceeds four MiB.

The byte counts include repeated serialized observations and references.
They do not establish actual live heap use or provider request size.
Repeatedly raising the thresholds moves the failure and leaves the same product limitation.

## Accepted repair

On 2026-10-15 the user explicitly selected removal of arbitrary capture limits.
This supersedes the earlier fixture-derived capture ceilings, including the 128-receipt limit.
Implementation removes the capture and source-index ceilings.
The configured native host now completes initial and follow-up prompts without capacity overrides.
Configured-workspace tool controls now pass complete assessment,
Allow execution,
prohibition,
denial,
cancellation,
and unsatisfied-prerequisite handling with authored provider responses.
Final regression checks and documentation rendering remain in progress.

The user subsequently rejected workspace-size-triggered abandonment of automatic assessment.
On 2026-10-15 they authorized three sequential bundles of up to three parallel provider calls,
and removed only the average USD 0.001 per judgment cost limit.
The five-second deadline remains.
The measured 617-question workspace is partitioned into four complete attributed requests across two bundles.
The unverified manual-capacity fallback in `8fe7e33410` was withdrawn by `95bd6db768`.

Remove arbitrary byte and occurrence ceilings from normal instruction capture and source accounting.
Preserve data-shape checks,
ownership and freshness validation,
accessor/proxy rejection,
cycle handling,
cancellation,
and actual provider/request constraints.
Keep unrelated program execution limits and semantic acceptance thresholds outside this change.
Where a depth/count limit currently doubles as cycle protection,
replace that role with explicit cycle detection before removing the incidental capacity restriction.

Document the captured objects and their retaining owners,
then measure memory with unchanged repeated prompts in bounded disposable processes.
Address unnecessary retained copies at their owner rather than silently dropping source material.
This preserves complete input evidence without treating fixture-sized snapshots as user-facing prompt limits.

Surface the retained deciding error in the user-facing diagnostic.
Do not make the user inspect an aggregate whose causes the UI omits.

## Verification

`test:native:prompt-capacity` was committed in `49b0dd1b08`
and reproduces the original callback-options failure against the built consumer.
It checks that complete policy text,
forced prompt text,
tool-loadout sources,
and aggregate snapshot material remain present.

The scratch RPC fixture loads the actual configured packages with synthetic credentials,
read-only repository mounts,
copied instructions,
and user/project skill mounts.
Networking is isolated and completions are local SSE responses.
It must pass initial and follow-up prompts,
then the original allowed/blocked tool controls,
without process-local capacity overrides.
The normal native bypass regression must remain green.

The standard SDK plus legacy extension passes the initial/follow-up prompt control;
no installed host or settings were changed.
That recovery option still needs tool-guard verification before activation.

## Verification guidance gap

Proposed tightening of `AGENTS.md`'s consumer-boundary verification rule:
after host integration changes,
exercise the user's configured resources and a follow-up interaction before claiming normal functionality.
Keep reduced fixtures as regression tests,
but label their narrower surface explicitly.
