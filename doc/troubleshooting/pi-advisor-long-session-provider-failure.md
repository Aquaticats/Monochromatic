# Pi Advisor 0.0.1 with Pi 0.82.1: long compacted sessions duplicate failed calls and render unreadable errors

## Symptom

Session `019fb34b-1fa3-7b1e-b6bf-e4ce7f632feb` reached Task 8 after two Pi compactions.
The visible sequence was:

```text
advisor synthetic/hf:moonshotai/Kimi-K3 125.5s synthetic 1512228/2029572 chars 378312 tokens full
(advisor returned no text)

advisor undefined NaNs undefined undefined/undefined chars undefined tokens full
advisor: advisor call was aborted

advisor undefined NaNs undefined undefined/undefined chars undefined tokens full
advisor: advisor call was aborted

Operation aborted
```

The persisted session gives the following exact call sequence:

- The default Kimi-K3 call ran for `125548` ms.
  It returned `stopReason: "error"`,
   zero reported usage,
   and no text.
  Advisor nevertheless stored a successful tool result containing `(advisor returned no text)`.
- The explicit `openai-codex/gpt-5.6-sol` call ran for `240011` ms and stored
  `advisor: advisor call was aborted` as an error result.
- The next default call ran for `240049` ms and stored the same error.
  Its selected model cannot be recovered because the thrown-result path discarded Advisor details.
- The primary Pi run then ended with assistant `stopReason: "aborted"` and `errorMessage: "Operation aborted"`.

The final abort is separate from the two Advisor tool timeouts.

A second live reproduction occurred while diagnosing this incident.
Session `019fb733-a7f7-7fda-baba-e3ecfb77c287` explicitly selected
`openai-codex/gpt-5.6-luna` with `215130` serialized context characters and
`53791` estimated input tokens.
It ran from `2026-07-31T08:24:46.097Z` to `2026-07-31T08:28:34.991Z`,
then stored the same `advisor: advisor call was aborted` error with empty details and no top-level usage.
The smaller uncompacted input proves that context inflation is not required to trigger a provider failure.
The retry and error-handling defects amplify any provider failure.

## Root cause

Source findings for upstream Pi refer to tag `v0.82.1`,
 commit
`b4f293684bba718d59cc1157679bcf6157b3a7f5`.
The relevant contracts are unchanged at Pi `v0.83.0`,
 commit
`845d6ff1f6643aba440341cce877ce1c43ebbc39`.

### Advisor bypasses Pi's compaction-aware context boundary

`package/pi-plugin/advisor/src/tool.ts:273-277` passes the complete active path from `getBranch()`:

```typescript
const selectionContext = selectAdvisorRunContext({
  branch: ctx
    .sessionManager
    .getBranch(),
```

Pi documents `getBranch()` as the full path and directs LLM consumers to the resolved context.
Upstream `packages/coding-agent/src/core/session-manager.ts:1255-1277` says:

```typescript
/**
 * Walk from entry to root, returning all entries in path order.
 * Includes all entry types (messages, compaction, model changes, etc.).
 * Use buildSessionContext() to get the resolved messages for the LLM.
 */
getBranch(fromId?: string): SessionEntry[] {
  // ...
}

/**
 * Build the active, compaction-aware entry list for context/rendering.
 */
buildContextEntries(): SessionEntry[] {
  return buildContextEntries(this.getEntries(), this.leafId, this.byId);
}
```

Upstream `packages/coding-agent/src/core/session-manager.ts:410-452` explains what the latter removes:

```typescript
/**
 * Build the active, compaction-aware session entry list.
 *
 * This follows the current leaf path. If the path contains compaction entries,
 * the latest compaction is represented by the compaction entry itself, followed
 * by the kept entries starting at firstKeptEntryId and all entries after the
 * compaction entry. Older summarized entries are omitted.
 */
export function buildContextEntries(
```

The incident measurement at Task 8 was:

- `getBranch()`:
   `2284` entries,
   `1668` included messages,
   `1512228` serialized characters;
- `buildContextEntries()`:
   `534` entries,
   `403` included messages,
   `363290` serialized characters.

Advisor therefore resent messages that Pi had already summarized.
It serialized `1148938` more characters than the compaction-aware boundary.
This explains the oversized Task 8 request,
 but the later Luna reproduction shows it is not the sole precondition for a provider failure.

The default also retains prior Advisor results.
`package/pi-plugin/advisor/src/config.ts:37-42` sets:

```typescript
export const DEFAULT_CONFIG: Omit<AdvisorConfig, 'source'> = {
  enabled: true,
  timeoutMs: DEFAULT_TIMEOUT_MS,
  maxAdvisorOutputTokens: DEFAULT_MAX_ADVISOR_OUTPUT_TOKENS,
  includePriorAdvisorResults: true,
};
```

Disabling prior results on the raw Task 8 branch reduced the serialized context from `1512228` to
`1503716` characters.
This is not the primary inflation source.
The measurement establishes retained review text,
not whether those reviews are unrelated or whether excluding them improves judgment.
See the [prior-review policy reassessment](../planning/pi-advisor-prior-review-context.md)
for the distinction between artifact exclusion and blind assessment.

### Pi returns terminal provider errors as values

Pi's stream contract does not reject `result()` on a provider error.
Upstream `packages/ai/src/utils/event-stream.ts:69-78` resolves both successful and error terminal events:

```typescript
export class AssistantMessageEventStream extends EventStream<AssistantMessageEvent, AssistantMessage> {
  constructor() {
    super(
      (event) => event.type === "done" || event.type === "error",
      (event) => {
        if (event.type === "done") {
          return event.message;
        } else if (event.type === "error") {
          return event.error;
        }
```

The returned `AssistantMessage` carries `stopReason: "error" | "aborted"` and `errorMessage`.
This is a documented Pi API contract,
 not an upstream defect.

Advisor checks only for text before retrying.
`package/pi-plugin/advisor/src/advisor-client.ts:319-345` does not inspect the terminal state:

```typescript
const firstResponse = await completeModel({
  // ...
  providerOptions: createProviderOptions(),
});
const responseHasText = firstResponse
  .content
  .some(/* text block check */);
if (responseHasText)
  return firstResponse;

return await completeModel({
  // ...
  providerOptions: createProviderOptions(),
});
```

A minimal provider-seam reproduction returned `error` and `aborted` messages without text.
Both caused two provider invocations:

```text
{"stopReason":"error","calls":2,"returnedStopReason":"error","returnedErrorMessage":"fixture error"}
{"stopReason":"aborted","calls":2,"returnedStopReason":"aborted","returnedErrorMessage":"fixture aborted"}
```

Each attempt receives a fresh timeout signal.
`package/pi-plugin/advisor/src/advisor-client.ts:275-290` states and implements that reset:

```typescript
/**
 * Build provider options for one provider attempt.
 *
 * @returns provider options with fresh timeout signal
 */
function createProviderOptions(): SimpleStreamOptions {
  return {
    signal: combinedSignal({
      timeoutMs: options.config.timeoutMs,
    }),
    timeoutMs: options.config.timeoutMs,
```

The default `timeoutMs` is `120000`.
An error or timeout without text can therefore consume two separate 120-second windows.
That produced both approximately 240-second failures in this session.

### Advisor turns a terminal `error` into a successful empty result

`package/pi-plugin/advisor/src/tool.ts:322-358` replaces missing text and stores the provider stop reason only
 in details:

```typescript
const text = extractAdvisorText(response,)
  || '(advisor returned no text)';

return {
  text,
  details: {
    // ...
    stopReason: response.stopReason,
    usage: response.usage,
  },
};
```

`package/pi-plugin/advisor/src/tool.ts:147-158` throws only for `aborted`:

```typescript
if (result.details.stopReason === 'aborted')
  throw new Error('advisor: advisor call was aborted',);

return {
  content: [{ type: 'text', text: result.text }],
  details: result.details,
};
```

The first Kimi-K3 response had `stopReason: "error"`,
 so the tool result was incorrectly marked successful.
The exact provider `errorMessage` is no longer recoverable because Advisor never persisted it.

### Thrown tool errors lose details, then the renderer trusts them

Pi correctly converts thrown tool errors into failed tool results.
Upstream `packages/agent/src/agent-loop.ts:692-703` catches the exception,
 and
`packages/agent/src/agent-loop.ts:756-760` creates this result:

```typescript
return {
  result: createErrorToolResult(error instanceof Error ? error.message : String(error)),
  isError: true,
};

function createErrorToolResult(message: string): AgentToolResult<any> {
  return {
    content: [{ type: "text", text: message }],
    details: {},
  };
}
```

Advisor's manual-message renderer validates details,
 but its tool renderer does not.
`package/pi-plugin/advisor/src/rendering.ts:128-140` passes the empty object directly:

```typescript
return new Text(
  renderAdvisorSummary({
    text,
    details: result.details,
    expanded,
    theme,
  }),
```

`package/pi-plugin/advisor/src/rendering-summary.ts:264-310` reads absent fields and divides an absent duration:

```typescript
const model = theme.fg('accent', details.selectedSlug,);
const metadata = theme.fg(
  'dim',
  `${formatDuration(details.durationMs,)} ${formatContext(details,)}`,
);

return durationMs < MILLISECONDS_PER_SECOND
  ? `${durationMs}ms`
  : `${(durationMs / MILLISECONDS_PER_SECOND).toFixed(1,)}s`;
```

That directly produces `undefined NaNs undefined undefined/undefined chars undefined tokens full`.

### Nested Advisor usage is not included in Pi session accounting

Successful Advisor responses keep provider usage inside `details`,
 but
`package/pi-plugin/advisor/src/tool.ts:152-158` does not return the tool result's top-level `usage` field.
Every inspected Advisor tool result in the session therefore had persisted `usage: null`,
 even when
`details.usage` reported provider input and output tokens.
Retries also discard the first attempt's usage.

### Default selection rewards larger requests

`package/pi-plugin/advisor/src/tool-context-selection.ts:226-265` builds a differently truncated context for every
candidate and sends each candidate's input estimate into selection.
`package/pi-shared/model-selection/src/cost-ranking.ts:245-259` computes:

```typescript
const expectedCost = (estimatedInputTokens * inputCost)
  + (maxOutputTokens * outputCost);
```

`package/pi-shared/model-selection/src/cost-ranking.ts:276-293` sorts highest expected cost first.
A model can therefore rank higher because its context window admits more of the oversized history.
For Task 8,
 Kimi-K3 won with `378312` estimated input tokens and an expected-cost score of `1380696`.
This couples default model choice to context inflation rather than observed reliability.

## Verification

### Versions and source identities

- Pi Advisor:
   package version `0.0.1`,
   repository commit `a6cea9ac8` at diagnosis time.
- Pi:
   installed `0.82.1`;
   upstream tag commit
  `b4f293684bba718d59cc1157679bcf6157b3a7f5`.
- Pi comparison release:
   `0.83.0`;
   upstream tag commit
  `845d6ff1f6643aba440341cce877ce1c43ebbc39`.
- Synthetic provider:
   `@benvargas/pi-synthetic-provider@1.2.2`,
   using Pi's
  `openai-completions` stream implementation.
- Incident session file:
   `6705507` bytes,
   `2296` JSONL lines,
   two compaction entries.
- Diagnosis-session reproduction:
   `215130` serialized characters,
   `53791` estimated input tokens,
   no compaction entries before the failed call.

### Session evidence harness

The following extracts Advisor calls and results without printing the serialized conversation:

```bash
# doc/troubleshooting/pi-advisor-long-session-provider-failure.md
session="${HOME}/.pi/agent/sessions/--var-home-user-Monochromatic--/2026-07-30T13-51-12-804Z_019fb34b-1fa3-7b1e-b6bf-e4ce7f632feb.jsonl"
jq --compact-output '
  select(.type == "message")
  | select(
      (.message.role == "toolResult" and .message.toolName == "advisor")
      or (.message.role == "assistant" and any(.message.content[]?; .type == "toolCall" and .name == "advisor"))
    )
  | {
      timestamp,
      role: .message.role,
      stopReason: .message.stopReason,
      isError: .message.isError,
      details: .message.details
    }
' "$session"
```

The compaction comparison uses Pi's real `SessionManager` and Advisor's real serializer:

```bash
# doc/troubleshooting/pi-advisor-long-session-provider-failure.md
node --input-type=module -e '
import { SessionManager, buildContextEntries } from "./package/pi-plugin/advisor/node_modules/@earendil-works/pi-coding-agent/dist/index.js";
import { buildAdvisorContext, DEFAULT_CONFIG } from "./package/pi-plugin/advisor/dist/final/node/index.mjs";
const session = `${process.env.HOME}/.pi/agent/sessions/--var-home-user-Monochromatic--/2026-07-30T13-51-12-804Z_019fb34b-1fa3-7b1e-b6bf-e4ce7f632feb.jsonl`;
const manager = SessionManager.open(session);
const leaf = "f8e6f369";
const toolCallId = "call_X1pqsrYAHVFohrN5jOWQnrbv|fc_049583c399c269cd016a6c554a3fe08193b7914e6c2bca677d";
const config = {...DEFAULT_CONFIG, source: {globalPath: "", projectPath: "", globalLoaded: false, projectLoaded: false}};
for (const [label, branch] of [["getBranch", manager.getBranch(leaf)], ["buildContextEntries", buildContextEntries(manager.getEntries(), leaf)]]) {
  const result = buildAdvisorContext({branch, config, advisorSystemPrompt: "x", maxContextChars: Number.MAX_SAFE_INTEGER, toolCallId});
  console.log(JSON.stringify({label, entries: branch.length, messages: result.includedMessageCount, chars: result.finalChars}));
}
'
```

### Patterns that work cleanly

- A provider response with `stopReason: "stop"` and a non-empty text block returns after one attempt.
- Pi's `buildContextEntries()` keeps the latest compaction summary and retained entries
 while omitting older summarized entries.
- Explicit Advisor calls that complete before timeout return useful review text.

### Patterns that fail as an empty success

- `stopReason: "error"` plus no text is retried,
   then returned as `(advisor returned no text)` with `isError: false`.
- A second no-text terminal error overwrites the first response's `errorMessage` and usage.

### Patterns that fail as a doubled timeout

- `stopReason: "aborted"` plus no text is retried with a fresh 120-second signal.
- Two timed-out attempts can produce an approximately 240-second tool failure.
- A later explicit Luna call failed after `228894` ms with only `53791` estimated input tokens.
- Shortening the focus question does not remove the resent pre-compaction history.

### Token-estimate check

Successful calls in this session under-estimated provider-reported input by:

- Kimi-K3:
   `3.95%` and `7.39%`;
- gpt-5.6-sol:
   `11.83%`;
- gpt-5.6-luna:
   `10.97%`;
- gpt-5.6-terra:
   `10.11%`.

A context budget that fills the model metadata limit using the four-characters-per-token estimate has no measured safety
 margin for this workload.

## Verified workarounds

### Bound context and omit old Advisor results

Use the existing global configuration boundary:

```json
// ~/.pi/agent/extensions/pi-advisor.json
{
  "maxContextChars": 350000,
  "maxAdvisorOutputTokens": 16384,
  "includePriorAdvisorResults": false
}
```

This bounds the incident's serialized context before provider dispatch.
Tradeoff:
 current truncation keeps character-level head and tail slices,
 so it can still retain old history and remove evidence from the middle.
It does not fix terminal-state handling or the doubled retry deadline.

### Request an explicit alternate model

Use the tool's explicit `model` field after a default model fails:

```json
{
  "model": "openai-codex/gpt-5.6-luna",
  "question": "Review the current task only and cite concrete blockers."
}
```

This bypasses the cost-ranked default selection.
Tradeoff:
 a terminal no-text error on the explicit model still triggers two attempts,
 and selecting the primary model removes reviewer independence.

### Set the operation deadline to expected provider latency

After the shared-deadline correction,
 `timeoutMs` bounds the complete Advisor operation rather than each attempt.
The user selected a ten-minute default for providers whose valid reviews can exceed two minutes:

```json
// ~/.pi/agent/extensions/pi-advisor.json
{
  "timeoutMs": 600000
}
```

Tradeoff:
 a genuinely stuck provider can now hold the tool call for up to ten minutes.
Terminal provider errors and caller cancellation still fail immediately.

## What does not work

- Before the shared-deadline correction,
   raising `timeoutMs` did not improve failure classification and increased the possible two-attempt wait.
- Retrying the same question manually does not remove pre-compaction entries
 because `runAdvisor()` always calls `getBranch()`.
- Switching from default Kimi-K3 to explicit gpt-5.6-sol does not avoid the duplicate no-text retry.
  The incident's explicit call lasted `240011` ms.
- Treating `(advisor returned no text)` as a valid review hides `stopReason: "error"` and loses the provider diagnostic.
- Updating only to Pi `0.83.0` does not change `AssistantMessageEventStream.result()`,
   compaction APIs,
   or thrown tool error details.
  The plugin must adapt to those contracts.
- Returning an error-looking normal tool result would preserve custom details but would not set Pi's `isError` flag.
  Pi's extension contract requires a throw for tool execution failure.

## Recommended plugin correction

- Build Advisor input from `ctx.sessionManager.buildContextEntries()`,
   not `getBranch()`.
- Historical proposal:
   default `includePriorAdvisorResults` to `false`.
  Withdrawn at the user's request;
   issue `#407` is closed as not planned.
  The [policy reassessment](../planning/pi-advisor-prior-review-context.md) records why the current default remains.
- Check `stopReason` before checking text.
  Treat `error` as provider failure,
   distinguish caller cancellation from the plugin deadline for `aborted`,
   and treat a successful empty response as its own failure class.
- Apply one absolute deadline to the whole Advisor operation.
  Do not create a complete fresh timeout budget for an identical retry.
- For default selection,
   use a configured ordered model preference and fallback list constrained by the live scope.
  Record a failed model in session health so the next default call does not select it again immediately.
  Do not silently fall back when the caller explicitly selected a model.
- Remove request-size-dependent highest-cost ranking.
  At minimum,
   score every candidate against the same bounded input size so larger context windows are not rewarded
    for receiving more history.
- Replace character-only context budgeting with an exact tokenizer where available or a conservative reserve.
  Keep an independent hard input ceiling below provider metadata limits.
- Preserve `errorMessage`,
   diagnostics,
   selected slug,
   timeout source,
   attempt count,
   and per-attempt usage in the surfaced error text.
- Validate `result.details` in the tool renderer.
  On an error result with `{}`,
   render the raw error content without a metadata header.
- Return successful nested model usage through the tool result's top-level `usage` field.
  Aggregate usage across every attempted provider call.
- Use `onUpdate` to show selected model,
   context size,
   attempt,
   elapsed time,
   and fallback state without exposing reasoning text.

Regression coverage must include compaction sentinels,
 every provider stop reason,
 caller abort versus deadline expiry,
 a shared deadline across retries,
 default fallback versus explicit selection,
 malformed error details,
 top-level nested usage,
 and a host-level Pi invocation that renders a failed Advisor call without `undefined` or `NaN`.

## Resolution

The P0 correction landed on 2026-07-31:

- commits `b30861f52`,
   `51a1108ab`,
   and `82361a8ad` classify terminal provider states before retrying and share one absolute operation deadline;
- commit `2f2dc89ba` uses Pi's compaction-aware context entries;
- commits `640a24d41`,
   `b090c6256`,
   `f258ab73e`,
   and `8d25c5535` render missing or malformed error details as raw error text.

Package verification passed the complete Advisor unit suite,
 TypeScript lint,
 Oxlint with zero warnings,
 and built-extension registration verification.
A disposable real Pi host loaded the built extension and a faux provider.
The reviewer returned `stopReason: "error"` with `fixture provider failure`.
Pi recorded one failed Advisor tool result containing:

```text
advisor: provider call failed for advisor-p0-fixture/reviewer on attempt 1: fixture provider failure
```

The host then continued to the main model's final response.
The queued faux response sequence proves no identical reviewer retry consumed a second response.

### Continued-session provider variability

On 2026-08-01,
 the same persisted session produced three different provider failures after the P0 correction:

- Spark returned no visible text after two successful terminal responses;
- Luna reached the shared `120000` ms deadline on its first attempt;
- Qwen returned HTTP 400 with `Context limit exceeded`.

The current compaction path was active.
`package/pi-plugin/advisor/src/tool.ts:271` calls:

```typescript
.buildContextEntries(),
```

At the Qwen call,
 the full branch contained `2597` entries while the compaction-aware branch contained `847`.
Advisor serialized `599445` characters and estimated `150061` input tokens.
The same request tokenized locally with the official Qwen tokenizer at commit
[`6a9e13bd6fc8f0983b9b99948120bc37f49c13e9`][qwen-tokenizer]
used `172314` chat-template input tokens.
With the configured `16384` output reserve,
 the total was `188698`,
 which is `73446` below the model configuration's `262144` positions.
The character estimate understated this tokenizer by `14.83%`,
 but the request still fit the documented native window.

The same session also disproves a stable character threshold:
 Luna previously succeeded with `989163` serialized characters but later timed out with `599004`,
 and Spark previously succeeded with `404629` but later returned no text with `444292`.
One variable provider failure therefore does not establish a safe global context cap.
A proposed `300000`-character default was rejected and removed.
Model-aware budgeting remains in place
 while issue `#410` tracks measured reserve improvements without treating one failure as a stable capacity limit.

Commits `7a13ad30c` and `f4d57a2be` raise the tested and documented default operation deadline to `600000` ms.
The user-level `~/.pi/agent/extensions/pi-advisor.json` now carries the same value.
This addresses the Luna wait policy only;
 it does not reinterpret Spark's empty terminal responses or Qwen's transient provider 400 as local timeout failures.

The prior-review default proposal in issue `#407` was withdrawn and closed as not planned.
Other follow-up findings were tracked in issues `#408` to `#411`:
 nested usage accounting,
 model selection,
 input budgeting,
 and progress updates.

## Upstream filing decision

No `.out-of-scope/` entry matches this incident.
Open and closed issue and pull-request searches in `earendil-works/pi` for
`AssistantMessageEventStream result error stopReason` found no duplicate.

1. **Is it really upstream's fault?**
    No.
   Pi explicitly returns terminal error messages from `AssistantMessageEventStream.result()`
    and exposes a separate compaction-aware API.
   Advisor used both APIs incorrectly.
2. **Can upstream fix it?**
    Upstream could change contracts,
    but no upstream change is required.
   The complete correction is inside `package/pi-plugin/advisor`.
3. **Are they supporting this use case?**
    Yes,
    Pi supports nested provider calls and custom tools.
   That does not make Pi responsible for Advisor's terminal-state and context-selection policy.
4. **Would the repo welcome our contribution?**
    `CONTRIBUTING.md` allows understood AI-assisted code but requires issue prose in the contributor's own voice
     or a clearly AI-labelled follow-up.
   New-contributor issues are auto-closed pending maintainer review,
    and PRs require prior `lgtm` approval.
   No upstream contribution is warranted here.
5. **Will they likely fix it?**
    Not applicable because the observed contracts are intentional and remain unchanged in `v0.83.0`.
6. **Have we prototyped a minimal upstream fix?**
    No upstream prototype is appropriate because constraint 1 fails.
   Constraints 1 to 5 do not all hold,
    so the auto-prototype gate does not trigger.

There is nothing additive to file upstream and no issue draft to retain.
The actionable artifact is the repository-local correction plan in this document.

[qwen-tokenizer]: https://huggingface.co/Qwen/Qwen3.6-27B/tree/6a9e13bd6fc8f0983b9b99948120bc37f49c13e9
