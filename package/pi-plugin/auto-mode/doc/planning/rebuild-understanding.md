# Auto-mode working understanding

Recorded on 2026-10-19 after repeated corrections exposed assumptions in the rebuild proposal.
This record distinguishes accepted meaning,
verified Pi behavior,
proposed mechanisms,
and unresolved design.
The [active plan](ground-up-rebuild.md) and [review round](rebuild-review-round-2.md) depend on these distinctions.
No replacement implementation is authorized by this record.
The [complete question inventory](rebuild-question-inventory.md) distinguishes current choices,
dependent design work, and facts to measure.

## Graceful fallback and degradation

The user's accepted meaning is to degrade as little as possible,
aiming for no degradation of the end-user experience where possible.
An equivalent supported path is preferable to reducing functionality.
When equivalent behavior is unavailable,
the fallback preserves the greatest supported functionality
and makes material remaining limits clear.

Example:
if relevance classification fails but fresh rules can be captured,
review all those rules rather than requiring human approval solely because relevance filtering failed.
Each tool call is reviewed individually through public APIs.
Cross-call interaction analysis is not a separate shared review or approval requirement.

The agent's former interpretation,
automatically escalating or withholding execution whenever integration coverage
was incomplete,
was too restrictive and was explicitly corrected.
A missing API alone is not a rule requiring human confirmation.
An actual instruction requiring confirmation still applies.

Degradation concerns the experienced outcome,
including retained functionality,
correctness,
responsiveness,
extra prompts,
and interruption of otherwise permitted work.
Changing an internal mechanism is not necessarily degradation.
No numeric latency or fidelity target is implied by that definition.

## Individual tool-call review

The latest user requirement on 2026-10-19 is that parallel tool calls are always judged individually.
Every outer invocation has its own immutable evidence snapshot,
rule answers,
review allowance,
and human decision.
This supersedes the earlier whole-parallel-batch review requirement.
That earlier requirement is historical,
including shared budgets and joint-interaction attribution questions.

`codemode` counts as one tool call.
Parse its complete body and execution structure,
but do not create inner approval units for its contained operations.
Its body can contain many actions while its review still returns one outer-call disposition.

Parallel calls can produce different outcomes.
One denied call does not inherently require canceling its allowed siblings or aborting the run.
For independent local tests and a prohibited public upload,
tests may remain permitted in the same batch
while the upload is blocked.
The agent's earlier requirement to reject both and request a new tests-only batch was incorrect.

Pi retains scheduling and execution ordering.
Independent judgments do not promise that a permitted call starts while another call's review is pending.
Do not add a group verdict,
sibling-conflict branch,
or shared manual approval to recover the superseded design.

### Verified public Pi behavior

Installed Pi coding agent 1.3.0 exposes `ToolCallEventResult` in
`package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/types.d.ts:1089`:

```typescript
// Pi 1.3.0: dist/core/extensions/types.d.ts:1089
export interface ToolCallEventResult {
    /** Block tool execution. To modify arguments, mutate `event.input` in place instead. */
    block?: boolean;
    reason?: string;
}
```

This excerpt omits the interface's separate `terminate` hint;
blocking and termination are not the same operation.
`dist/core/agent-session.js:333` forwards each event with the current `toolCall.id`:

```javascript
// Pi 1.3.0: dist/core/agent-session.js:333
return await runner.emitToolCall({
    type: "tool_call",
    toolName: toolCall.name,
    toolCallId: toolCall.id,
    ...(parentToolCallId ? { parentToolCallId } : {}),
    input: args,
});
```

`dist/core/extensions/runner.js:965` returns that event's blocking result:

```javascript
// Pi 1.3.0: dist/core/extensions/runner.js:965
if (result.block) {
    return result;
}
```

The public-only disposable consumer in `~/temp/agent/pi-public-plugin-probe-20261019/`
passes `mise run lint:types` and `mise run probe`,
with 20 asserted cases.
Its scripted provider and inert tools do not invoke a remote model or perform the represented real actions.
These historical selective-blocking fixtures include the whole declared batch in review:

- `member-allow-a-block-b`:
  review both IDs,
  allow A,
  block B,
  receive B's blocked result,
  execute A,
  receive A's successful result,
  and continue to the scripted final response.
- `member-block-a-allow-b`:
  review both IDs,
  block A,
  receive A's blocked result,
  allow and execute B,
  receive B's successful result,
  and continue to the scripted final response.

Neither calls `ctx.abort()`.
The traces are in `results.json`;
the earlier 18-case output is retained as `results-18-cases.json`.
The all-run abort discussed previously belonged to a prototype that had already returned approval for earlier hooks.
It is not a Pi parallel-batch requirement.
These fixtures prove per-call blocking;
their shared-review prototype does not implement the latest review unit.

## Blocking, cancellation, and human Stop

The two verified cases block before execution at the public `tool_call` boundary.
They do not establish that any arbitrary already-running call can be canceled by ID.
The inspected public interface supplies per-call blocking and a run-abort operation;
an arbitrary post-hook revocation API was not found in the inspected declarations.
That scoped observation is not proof that every other host surface lacks cancellation.

The existing review illustration uses Approve and Stop with an optional reason.
Its Stop action aborts the run;
automatic denial of a call is a different action.
The latest correction does not itself settle a change to the human Stop interaction.
Any manual approval is scoped to the displayed single outer call,
not unrelated siblings or future work.

## Public plugin APIs and ownership

The user permanently prohibits maintaining or copying Pi internals,
duplicating existing Pi functionality,
and using private Pi APIs.
Supported plugin APIs are the integration surface.
Pi retains its tool implementations,
scheduling,
argument preparation,
sessions,
and UI behavior.
Lack of a public capability leads to the least-degrading supported fallback,
not permission to recreate Pi.

Reading source to understand behavior is evidence gathering,
not authorization to integrate with that private code.
The old copied/transformed host and the proposed loading hook were agent choices without user approval.
They are withdrawn,
not accepted alternatives awaiting revival.

## Arguments visible to review and the accepted limitation

The intended review sees preparation and transformations visible before its decision boundary.
The user explicitly accepts later extensions changing arguments after review as an unsupported case.
The design assumes those later mutations do not occur.
They do not justify ordering guards,
extra prompts,
fallback triggers,
or private hooks.

The selective-blocking probes establish public per-call decisions,
not final prepared-input capture in every host mode.
The historical prepared-group probes and the selective-blocking probes are distinct evidence surfaces.
Their results cannot be combined into a claim that one complete replacement implementation already works.

## Rule answer, admission, and individual decision

First-pass coverage includes every effective authored rule and every built-in rule.
Each rule uses the four choices `approve`,
`deny`,
`ask`,
and `not-applicable`.
The accepted Q1 B rule admits a valid unique leading probability without a minimum or winning-margin threshold.
A tie,
malformed answer,
or missing answer remains unresolved.

Every first-pass rule answer belongs to one outer call's review snapshot.
The scalar four-choice answer is attributed by that call identity and rule occurrence.
There is no separate sibling-attribution problem or joint-interaction question schema.
Each call receives its own complete every-rule first pass unless deterministic review already resolves it.

The tests' Approve answer is not a vote capable of canceling an upload's Deny answer.
Q7 concerns how rules governing the same call combine.
Q8 concerns complete all-not-applicable coverage for the affected action.
Neither choice is settled merely by correcting the parallel-batch misconception.

## Resolved Ask and unresolved review

An established Ask means a human decision is required.
It can follow an explicit rule even when all relevant facts are known.
Unresolved review means evidence or interpretation is not yet sufficient to establish a decision.
Those states do not have the same follow-up behavior.

An established Ask is not sent through more model batches to seek a different verdict.
That terminality concerns this call;
other calls retain their separate review state.
Each call has at most three sequential review batches with three parallel Luna requests per batch,
including retries.

An Ask explanation names the action,
target,
relevant rule,
recorded evidence,
the human judgment required,
and the consequences of the available actions.
Probability scores do not reveal the model's reasoning or identify a missing fact.
The Decisions response does not promise free-form rationale.
The explanation and focused-question mechanisms therefore remain explicit design work.

## Snapshot, startup rules, and shadow observation

The pending outer call is separate from the bounded visible-history field.
Q4 A also retains the latest genuine human request in its own field.
Synthetic user-role feedback does not become a new human request.
Private thinking content remains excluded even if its boundary counts as a history unit.

Startup relevance runs on every Pi start;
cached parsing does not authorize skipping classification.
The expected effective-rule count is not a truncation limit.
The built-in catalogue is plural but its count,
members,
and wording remain deferred.

Span 02 observes the primary review and does not own execution permission.
Luna's three-by-three review allowance applies per tool call;
Span 02 observations are counted separately.
The former combined-provider-budget question is withdrawn.
Delivery durability,
scheduling,
and evidence retention remain proposals or open questions.
A logger call or bounded flush does not by itself establish durable replay state.

## Unresolved boundaries and next evidence

- Reduction and follow-up reconciliation for rules governing the same outer call.
- Public behavior for custom global execution modes,
  dynamic loadouts,
  `codemode` outer-call boundaries,
  real TUI cancellation,
  and session transitions.
- Final serialized question capacity with all effective instruction sources and built-ins.
- Shadow scheduling,
  persistence,
  and local evidence retention.

These are named gaps,
not permission to stop all execution or expand into Pi internals.
Their treatment follows the user's least-degrading fallback rule and the current review's explicit decisions.
