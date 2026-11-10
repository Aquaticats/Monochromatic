# Pre-fix `@monochromatic-dev/pi-goal@0.0.1` hides a final answer beside `goal_complete`

Note 2026-11-09T20:41:24-05:00:
 that package is now `@monochromatic-dev/pi-plugin-goal`
 (rule SGD,
 issue #1674).
 The incident record below keeps the name and version it observed.

## Symptom

A primary model can display a complete answer and call `goal_complete` in the same assistant message.
The independent reviewer then denies completion with a diagnostic such as:

```text
Independent reviewer denied completion: Not complete. The evidence contains no finalized response
delivering five distinct explanations that 67 is prime.
```

The terminal wrapped the diagnostic for display.
Session ID `01a03c23-5f48-778f-8306-b30a1fddddd2` contains the concrete incident in
`~/.pi/agent/sessions/--var-home-user-Monochromatic--/`.
The primary model's final assistant message contains both:

- the full answer beginning `67 is prime in these five ways:`
- the pending `goal_complete` call

The next persisted tool result says the reviewer saw no finalized five-part response.
Repeated completion attempts do not repair the evidence.

The later `Operation aborted` text is a separate Pi TUI rendering of the next assistant message's
`stopReason: "aborted"`.
Pi `0.84.2` selects that text in
`packages/coding-agent/src/modes/interactive/components/assistant-message.ts:183-188`:

```typescript
if (message.stopReason === "aborted") {
  const abortMessage =
    message.errorMessage && message.errorMessage !== "Request was aborted"
      ? message.errorMessage
      : "Operation aborted";
```

The session's final entry has that aborted stop reason.
The reviewer denial itself did not emit `Operation aborted`.

## Root cause

### Completion is registered as a primary-model tool

The repository-owned extension puts completion in the same model tool namespace as work tools.

```typescript
// package/pi-plugin/goal/src/completion-registration.ts:101-103
pi.registerTool({
  name: GOAL_COMPLETE_TOOL_NAME,
  label: 'Complete Goal',
```

The interface asks the primary model to decide when to make a completion claim,
provide a summary,
and place the call last in its assistant tool batch.
That lets the answer text and control signal occupy one assistant message.

### Pi persists the combined assistant message before tool execution review

Pi `0.84.2` emits extension message handlers and then persists each finalized assistant message:

```typescript
// packages/coding-agent/src/core/agent-session.ts:633-656 at v0.84.2
// Emit to extensions first
await this._emitExtensionEvent(event);

// Notify all listeners
this._emit(event.type === "agent_end" ? { ...event, willRetry: this._willRetryAfterAgentEnd(event) } : event);

// Handle session persistence
if (event.type === "message_end") {
  // ...
  } else if (
    event.message.role === "user" ||
    event.message.role === "assistant" ||
    event.message.role === "toolResult"
  ) {
    // Regular LLM message - persist as SessionMessageEntry
    this.sessionManager.appendMessage(event.message);
  }
```

Source:
`packages/coding-agent/src/core/agent-session.ts:633-656` at Pi tag `v0.84.2`,
commit `914cf1472e715297caa30db4b9535d534a9eb718`.

The extension's reviewer therefore can read the pending assistant message from the active branch.
The incident session stores the combined text and call at JSONL line 22.

### The evidence serializer drops the whole pending message

The serializer does not remove only the pending tool-call block.
It detects the call identifier anywhere in the assistant message and omits the entire entry:

```typescript
// package/pi-plugin/goal/src/review-context.ts:203-207
if (assistantContainsToolCall({
  message,
  toolCallId,
},))
  return EVIDENCE_ENTRY_OMITTED;
```

That behavior was deliberate in the original plan:

```markdown
<!-- doc/planning/pi-goal-stop-hook.md:583-585 -->
- Include assistant messages and finalized tool results.
- Exclude the assistant message containing the pending `goal_complete` call.
- Supply the submitted completion summary separately.
```

The design assumed the pending message contained only a completion call.
The unit fixture named
`uses only post-start active branch evidence and excludes pending completion message`
constructs a pending assistant message with a tool call but no answer text
(`package/pi-plugin/goal/src/completion.unit.test.ts:380-479`).
It did not cover text and the completion call sharing one message.

### The reviewer correctly rejects the evidence it receives

The reviewer rubric says:

```typescript
// package/pi-plugin/goal/src/review-contract.ts:47
const GOAL_REVIEW_SYSTEM_PROMPT: string = `You are an independent completion reviewer.
```

```typescript
// package/pi-plugin/goal/src/review-contract.ts:50
Reject when evidence is incomplete, contradictory, unverified, or reports remaining work.
```

The final reviewer did not receive the five-part answer.
It received:

- an earlier advisor result saying no actual answer had yet been delivered
- the advisor's proposed elliptic-curve method
- an exploratory elliptic-curve command result of `points 0`
- two earlier completion denials
- a completion summary naming a sieve instead of the advisor's elliptic-curve proposal

The reviewer had no investigation tools and was required to judge only that supplied record.
Its denial was consistent with the evidence serializer's output,
even though it contradicted what the user had just seen.

The immediate defect is evidence blindness.
The deeper defect is exposing completion as a primary-model tool.
A control-plane stop decision and the user-facing answer have no safe shared-message protocol:

- calling before the answer leaves no answer to review
- calling beside the answer creates the omission incident
- including that in-flight tool message asks the reviewer to judge
  before its tool result determines whether another primary turn follows

Goal completion belongs at the harness settlement seam over finalized output,
not in the primary tool interface.

### `points 0` was stale exploratory evidence, not a refutation of the final answer

The advisor proposed an elliptic-curve certificate.
The primary then ran a stricter search that printed `points 0` and abandoned that method.
Its final answer used Eratosthenes' sieve instead.

Because the final answer was omitted,
the reviewer saw the abandoned exploration and only a summary naming the replacement.
The reviewer could not verify that the displayed answer had removed the elliptic-curve method.
The `points 0` result does not contradict the five methods actually displayed;
none of those methods uses an elliptic curve.

## Verification

### Versions and source identity

Verified on 2026-08-26 with:

- `@monochromatic-dev/pi-goal` `0.0.1`
- repository HEAD `f4ed2cb9fb88cf512c87b88673914b7b8754d7f4`
- built extension SHA-256
  `cfea43395480b8c8bbfec2fdaaf79d47dc1abb4a1015bc2825d2319936a71835`
- Pi `0.84.2`
- Pi tag `v0.84.2` at commit
  `914cf1472e715297caa30db4b9535d534a9eb718`
- incident session ID
  `01a03c23-5f48-778f-8306-b30a1fddddd2`

Pi source was cloned read-only with `gh repo clone earendil-works/pi` into a private scratch directory.
The verified origin was `https://github.com/earendil-works/pi.git`.
No upstream file was modified.

### Reproduction harness

Run from the Monochromatic repository root.
The harness reads the existing session and calls the built serializer without mutating session state:

```bash
node --input-type=module <<'NODE'
import { readFileSync } from 'node:fs';
import {
  buildGoalReviewEvidence,
  reduceGoalEvents,
} from './package/pi-plugin/goal/dist/final/node/index.mjs';

const sessionDirectory = `${process.env.HOME}/.pi/agent/sessions/--var-home-user-Monochromatic--`;
const sessionName = '2026-08-26T03-35-46-248Z_01a03c23-5f48-778f-8306-b30a1fddddd2.jsonl';
const file = `${sessionDirectory}/${sessionName}`;
const rows = readFileSync(file, 'utf8').trim().split('\n').map(JSON.parse);
const goal = reduceGoalEvents([rows[5].data]);
const pendingToolCallId = 'call_qa5Inda2jTB9CobHM6E7vlPl|fc_09daec40f3166a8d016a8e6236252c87d19548934ea9ebfe74';
const branchThroughPendingMessage = rows.slice(1, 22);
const baseRequest = {
  goal,
  goalId: goal.generationId,
  summary: 'Delivered five explanations.',
  runtimeEpoch: 'reproduction',
  branchLeafId: '00ac9f66',
};

const actual = buildGoalReviewEvidence({
  branch: branchThroughPendingMessage,
  request: { ...baseRequest, toolCallId: pendingToolCallId },
});
const positiveControl = buildGoalReviewEvidence({
  branch: branchThroughPendingMessage,
  request: { ...baseRequest, toolCallId: 'nonmatching-control' },
});
const containsAnswer = (evidence) => evidence.transcriptChunks.some(
  (chunk) => chunk.includes('67 is prime in these five ways'),
);

console.log(`actual pending id: ${containsAnswer(actual)}`);
console.log(`nonmatching positive control: ${containsAnswer(positiveControl)}`);
NODE
```

Observed output:

```text
actual pending id: false
nonmatching positive control: true
```

The positive control proves the harness can detect the answer when the pending-call exclusion does not match.

A second synthetic control split the same persisted content into an answer-only assistant message
followed by a pending-call-only assistant message.
The serializer then returned:

```json
{
  "containsFinalAnswer": true
}
```

### Patterns that work cleanly

- Standalone finalized assistant text after the goal start is serialized.
- Finalized tool results after the goal start are serialized.
- A pending assistant message containing only the current completion call is omitted
  without losing answer text from an earlier message.
- The nonmatching positive control includes the five-part answer.

### Patterns that fail

- Text followed by the current `goal_complete` call in one assistant message loses both from reviewer evidence.
- Retrying `goal_complete` with a richer summary still leaves the actual answer absent.
- Prior reviewer denials become finalized evidence for later attempts and reinforce the false missing-answer history.

## Verified workarounds

### Separate the answer and completion call across settled turns

The synthetic control verifies that an answer in an earlier finalized assistant entry remains visible
when a later assistant entry contains the pending call.
The current `agent_settled` continuation can create that later turn while the goal remains active.

Tradeoffs:

- It preserves the rejected public completion-tool protocol.
- It adds an otherwise unnecessary model turn.
- It depends on the primary model understanding an internal message-placement rule.
- It can still produce repeated review spending and protocol errors.
- It is an operational escape hatch only,
  not the accepted architecture.

### Clear the goal after the answer

`/goal clear` prevents another completion review and leaves the already displayed answer in session history.

Tradeoffs:

- It does not record independent approval.
- It requires human intervention.
- It discards active goal state instead of completing it.

## What does not work

### Put the final answer beside `goal_complete`

This is the incident trigger.
The serializer omits the entire message.

### Improve only the completion summary

The summary is non-truncatable claim text,
not finalized answer evidence.
The reviewer rubric requires finalized output supporting it.

### Retry without changing message placement

Each retry sees the same absent answer plus another denial result.
The evidence becomes less favorable,
not more complete.

### Reconcile the abandoned elliptic-curve probe only in private reasoning

Private reasoning is serialized as `[private reasoning omitted]`.
It cannot explain `points 0` to the reviewer.
The displayed final answer already avoided the elliptic-curve method,
but that answer was the omitted evidence.

### Treat the reviewer as the root defect

The reviewer followed its supplied rubric and transcript.
Changing reviewer models cannot restore an entry removed before model selection.
The session used `openai-codex/gpt-5.6-sol` for the first denials and later fell back through distinct reviewers,
yet every reviewer received the same evidence omission.

## Durable correction

Remove `goal_complete` from the primary model's tool list and prompt.
Keep the existence of completion review,
reviewer identity,
verdict,
generation,
and stop-hook protocol outside primary-model context.
Review the naturally finalized primary response from a harness-owned `agent_settled` hook.
Approval should append durable completion state without a primary tool result.
Denial should persist its private audit and inject only actionable task-level requirements in one guarded continuation.
A user-question outcome must wait for user input instead of forcing autonomous continuation.

The accepted design is recorded in
`doc/planning/pi-goal-tool-free-completion.md`.
It is implemented in `package/pi-plugin/goal/` and verified through unit,
built-extension,
and disposable Pi runtime checks.

## Upstream filing decision

### Out-of-scope check

The repository's `.out-of-scope/` files were checked on 2026-08-26.
No entry covers the repository-owned goal extension or this evidence omission.

### Duplicate search

Open and closed issue and pull-request searches in `earendil-works/pi` found no match for:

- `goal_complete review pending assistant message`
- pending completion answer omission
- goal reviewer evidence omission

The current repository's open and closed issues also had no matching report.

### Constraint check

#### Constraint 1: Is it really upstream's fault?

No.
Pi persists the finalized assistant message and exposes the active branch to the extension.
The repository-owned serializer deliberately removes the message.

#### Constraint 2: Can upstream fix it?

No external Pi change is required.
The internal extension can move review to `agent_settled` and remove its public completion tool.

#### Constraint 3: Is upstream supporting this use case?

Pi supports extension tools,
session branch access,
and `agent_settled` hooks.
The faulty completion protocol is repository-owned policy built from those mechanisms,
not a Pi-supported completion feature.

#### Constraint 4: Would upstream welcome the contribution?

Not applicable because there is no demonstrated upstream defect to contribute.

#### Constraint 5: Will upstream likely fix it?

Not applicable because the correction belongs in this repository.

#### Constraint 6: Have we prototyped a minimal upstream-compatible fix?

No external prototype is appropriate.
The external contribution gate already fails at constraint 1.
The internal tool-free settlement redesign was subsequently implemented and verified in this repository.

### Filing artifact

Nothing should be filed against Pi upstream.
An upstream report would misattribute a repository-owned evidence policy to the harness.
The internal planning document and this troubleshooting record are the filing artifacts
for the later repository change.
