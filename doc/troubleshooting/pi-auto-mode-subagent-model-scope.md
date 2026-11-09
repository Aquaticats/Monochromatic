# Pi 0.87.1 subagents expose an empty live scope that prevented auto-mode judge selection

Note 2026-11-09T16:51:44-05:00:
`package/pi-shared/model-selection/src/sdk-scope.unit.test.ts` is now
`package/agent-harness-shared/model-selection/src/sdk-scope.unit.test.ts`,
moved 2026-10-20 in `b710410e81`.

## Symptom

The supplied `Screenshot_20260925_231044.png` shows auto-mode repeatedly logging:

```text
judge model resolution failed: Tried to auto-detect a fast judge model for a background task, but couldn't find one.
Reason: no fast judge models with API keys found across any provider
To fix: authenticate a model in Pi or expand Pi's effective model scope.
```

A child tool call then receives a fail-closed denial because its session has no approval UI.
The screenshot also shows the parent bypass banner while child judge failures continue.
These are distinct boundaries:
model selection fails before authentication,
whereas approval UI and bypass are session-local.

## Root cause

### The child has no explicit live cycle list

Installed `@tintinweb/pi-subagents` is version `0.19.0`.
The inspected source clone is commit `e955e29c51b7a6cce37e1108cd2d6c57a77e151c`.
Its `src/agent-runner.ts` matches the installed file byte-for-byte:
SHA-256 `88e61481fd627254ff3ac0a27e1991136c41e103fac847dd06444c8ac3d3eab1`.

`src/agent-runner.ts:969` obtains the parent's runtime,
then constructs the SDK child without `scopedModels`:

```ts
// Upstream src/agent-runner.ts:969 and sessionOpts fields at 982.
const parentModelRuntime = (ctx.modelRegistry as unknown as { runtime?: unknown }).runtime;
// Other session construction fields omitted.
modelRegistry: ctx.modelRegistry,
...(parentModelRuntime !== undefined && { modelRuntime: parentModelRuntime as never }),
model,
tools: sessionTools,
customTools: [...nestedTools, ...structuredTools],
resourceLoader: loader,
```

The Pi source clone is version `0.87.1`,
commit `d6af72e1857cfb10b41d8ff8e69f0d72b4cf6d31`.
`packages/coding-agent/src/core/agent-session.ts:419` supplies this default:

```ts
// Pi packages/coding-agent/src/core/agent-session.ts:419.
this._scopedModels = config.scopedModels ?? [];
```

This is not a deny-all restriction.
Pi's `packages/coding-agent/src/core/agent-session.ts:2160` treats the empty list as unrestricted cycling:

```ts
// Pi packages/coding-agent/src/core/agent-session.ts:2160.
async cycleModel(
  direction: "forward" | "backward" = "forward",
  options: ModelMutationOptions = {},
): Promise<ModelCycleResult | undefined> {
  if (this._scopedModels.length > 0) {
    return this._cycleScopedModel(direction, options);
  }
  return this._cycleAvailableModel(direction, options);
}
```

### Our resolver interpreted that default as an authoritative empty restriction

At regression commit `fc5aa9f0e`,
`package/pi-shared/model-selection/src/scope-resolver.ts:232` rejected only non-array values:

```ts
// Historical readLiveScope implementation.
if (!Array.isArray(rawScope,))
  return NO_LIVE_SCOPE;
```

The empty array therefore remained live scope,
and `resolveEffectiveScope` returned before reconstructing settings.
The current `package/pi-shared/model-selection/src/scope-resolver.ts:138` retains this precedence:

```ts
// package/pi-shared/model-selection/src/scope-resolver.ts:138.
const liveScope = readLiveScope<TModel>(ctx,);
if (liveScope !== NO_LIVE_SCOPE) {
  return {
    source: 'live',
    entries: liveScope,
  };
}
```

The correction at `package/pi-shared/model-selection/src/scope-resolver.ts:235` is:

```ts
// Empty raw cycle lists permit normal argv/settings reconstruction.
if ((!Array.isArray(rawScope,)) || (rawScope.length === 0))
  return NO_LIVE_SCOPE;
```

A nonempty malformed live list still produces an authoritative empty result.
Explicit settings and argv restrictions that match no models still produce no candidates.
Tests in `package/pi-shared/model-selection/src/sdk-scope.unit.test.ts` cover these boundaries.

`package/pi-shared/model-selection/src/sdk-scope.unit.test.ts` moved on 2026-10-20 to
`package/agent-harness-shared/model-selection/src/sdk-scope.unit.test.ts` in `b710410e81`
(refactor(*):
move model-selection to agent-harness-shared).
The paths this record names under `package/pi-plugin/auto-mode/src/` were removed on 2026-10-19 in
`7ac15b19b4` (refactor(pi-plugin-auto-mode):
remove legacy source for ground-up rebuild);
no file
in the working tree carries them now.

The credential-loss hypothesis was rejected by the real SDK reproduction:

```json
{"hasUI":false,"liveScope":0,"available":true,"authenticated":true}
```

Adding an explicit live model entry made the unchanged pre-fix selector pass.
No key value was logged and no external provider request was needed for this reproduction.

### Headless approval and parent bypass are separate boundaries

`@tintinweb/pi-subagents/src/agent-runner.ts:1018` binds the child's extensions without UI bindings:

```ts
// Upstream src/agent-runner.ts:1018.
await session.bindExtensions({
  onError: (err) => {
    options.onToolActivity?.({
      type: "end",
      toolName: `extension-error:${err.extensionPath}`,
    });
  },
});
```

`package/pi-plugin/auto-mode/src/ask-user.ts:225` still fails closed without UI.
Its updated return preserves the explanation and blocked action,
then tells a child to report them to the parent instead of retrying:

```ts
// package/pi-plugin/auto-mode/src/ask-user.ts:242.
return {
  block: true,
  reason: formatModelBlockReason({
    guardrailReason: `${explanation} No approval UI is available in this session. Blocked action: ${action}`,
    // Guidance directs a subagent to its parent and prohibits retries or rephrasing.
  },),
};
```

`package/pi-plugin/auto-mode/src/index.ts:156` owns bypass state per extension instance:

```ts
let bypassEnabled = false;
```

`package/pi-plugin/auto-mode/src/bypass.ts:223` restores it from that session's branch:

```ts
const branchEntries: SessionCustomEntry[] = ctx.sessionManager
  .getBranch()
```

The fix does not copy parent bypass,
trust rules,
or prior approvals into children.
It does not fabricate a child UI or turn failed judging into permission.

## Verification

### Repeatable regression

Run from the repository root:

```bash
# Build and exercise the actual Pi SDK context against the shipped auto-mode artifact.
mise run //package/pi-plugin/auto-mode:build:js:node
mise run //package/pi-plugin/auto-mode:test:unit -- src/sdk-scope.unit.test.ts src/ask-user.unit.test.ts
```

The SDK test uses a disposable workspace,
file-backed settings,
in-memory session,
local fixture credentials,
and no ambient extensions.
Before the fix,
`uses configured models when SDK live scope is unset` failed with the exact screenshot error.
The explicit nonempty live-scope control passed.
Both now pass.

### Real subagent consumer boundary

A disposable Pi CLI host loaded the installed subagents extension and the rebuilt auto-mode bundle.
Scripted parent and child providers drove the actual `Agent` tool,
`get_subagent_result`,
child `read` tool,
and completion delivery.
A loopback OpenAI-compatible SSE server supplied the judge verdict.
No live credentials or user files were used.
Each child issued one guarded read outside its workspace.
For available-judge scenarios,
the server verified a `render_verdict` request containing that exact path.
Unavailable-judge scenarios verified that no request escaped the configured restriction.

The session-local harness is retained at
`~/temp/agent/auto-mode-subagent-host.mjs`
and `~/temp/agent/auto-mode-subagent-fixture.ts`.
It invokes finite scripted prompts,
caps each host at 20 seconds,
closes piped stdin,
and disposes its temporary homes and server.
The final host run also captured stderr and rejected bare `context canceled` shutdown errors.
The permanent SDK regression is the repository-owned reproduction;
the session-local harness additionally verifies the installed third-party integration.

Observed results:

```text
approve foreground: one judge request, read allowed, parent received child result
approve background: one judge request, read allowed, parent received child result
deny foreground: one judge request, read blocked, parent received child result
deny background: one judge request, read blocked, parent received child result
ask foreground: one judge request, read blocked with parent guidance, parent received child result
ask background: one judge request, read blocked with parent guidance, parent received child result
unavailable judge foreground: no judge request, read blocked with selection error and parent guidance
unavailable judge background: no judge request, read blocked with selection error and parent guidance
```

Working catalog:

- Empty SDK live list with configured authenticated judge.
- Nonempty live scope overriding settings.
- Empty SDK list with argv restrictions,
  settings restrictions,
  or no configured restriction.
- Foreground and background subagents receiving each verdict kind.
- Advisor dispatch after Pi's live cycling restriction is cleared.

Fail-closed catalog:

- Nonempty live scope whose candidates cannot authenticate.
- Nonempty live list containing only malformed records.
- Explicit empty or unmatched argv/settings restrictions.
- Judge denial.
- Approval-required action in a headless child.
- No eligible judge in a real foreground or background child;
  its result preserves the original selection error,
  action,
  and no-retry guidance.

Not exercised:
external paid-provider availability,
interactive forwarding of child approval requests,
or automatic inheritance of parent bypass and trust.
The latter integrations were not added.

All affected-package unit suites,
TypeScript checks,
and Oxlint checks passed for auto-mode,
shared model selection,
and Advisor.
The changed Markdown files passed lint and were rendered with Marked;
rendered headings,
code blocks,
and lists were inspected.

## Verified workarounds

For an SDK caller that already owns the intended scope,
passing an explicit nonempty `scopedModels` list avoids the pre-fix empty-list path.
The regression's positive control verifies this:

```ts
// createAgentSession options, using the caller's intended authenticated model.
scopedModels: [{ model, },],
```

Tradeoff:
a fixed list must be refreshed when the intended parent scope changes.
The local resolver correction is preferable to patching an installed third-party package.

## What does not work

- Re-authenticating an already available model does not populate an empty candidate list.
  The pre-fix reproduction authenticated successfully and still failed selection.
- Treating all empty results as unrestricted would widen explicit restrictions.
  The fix checks the raw live list before filtering and leaves configured empty results authoritative.
- Parent bypass does not change child session state.
  The screenshot's banner therefore does not establish that the child guard was bypassed.
- A headless approval fallback cannot obtain interactive consent.
  It must remain blocked and report the action rather than recommend repeated calls to its own approval tool.

## Upstream filing decision

The `.out-of-scope/` inventory contains no matching exemption for this integration.
Issue and pull-request searches in `tintinweb/pi-subagents` for `auto-mode scopedModels` returned no matches.
No upstream report or patch was sent.

1.  Upstream fault:
    no.
    Our resolver imposed deny-all semantics on Pi's empty cycling list.
2.  Upstream fixability:
    a caller could pass a scope,
    but our consumer must still understand the valid SDK default.
3.  Supported use case:
    the subagents README documents inherited extensions and independent sessions;
    Pi's SDK documents optional `scopedModels`.
4.  Contribution acceptance:
    not evaluated because no upstream defect or contribution is proposed.
5.  Likelihood of an upstream fix:
    not applicable to the local semantic mismatch.
6.  Prototype:
    the local correction passed the SDK and installed-extension tests;
    no third-party source modification was necessary.

Upstream filing artifact:
nothing to add.
The durable fix belongs to this repository.
