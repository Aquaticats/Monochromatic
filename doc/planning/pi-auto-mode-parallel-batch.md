# Historical parallel-batch judgment proposal

## Superseded by individual judgments

On 2026-10-19, the user changed the requirement:
parallel tool calls must always be judged individually, and `codemode` counts as one tool call.
Each outer invocation owns its prepared input, rule answers, verdict, review allowance, and manual approval.
Parse a complete `codemode` body without creating inner approval units.
The shared-batch design and prototypes in this document are historical, not current implementation requirements.
The [active rebuild plan](../../package/pi-plugin/auto-mode/doc/planning/ground-up-rebuild.md)
and [question inventory](../../package/pi-plugin/auto-mode/doc/planning/rebuild-question-inventory.md)
own the replacement design.

## Current constraint and graceful fallback

On 2026-10-19, the user explicitly prohibits maintaining or copying Pi internals,
duplicating existing Pi functionality, and using private Pi APIs, without exception.
Use supported public plugin APIs and leave execution, scheduling, argument preparation,
session handling, and existing tool behavior owned by Pi.
If a desired review behavior cannot be implemented through those APIs, gracefully fall back.
A missing capability never authorizes another dispatcher, host copy, source transformation, or private hook.

Degrade as little as possible, aiming for no degradation of the end-user experience where possible.
First seek an equivalent supported public-API path; otherwise retain the greatest supported review coverage.
Per-call review is now the intended behavior, not a degraded replacement for batch review.
A missing integration capability does not itself require manual approval, blocked execution, or disabled auto-mode.
Keep ordinary Pi usable and explain material limitations without claiming guarantees the fallback lacks.
Do not deadlock Pi while waiting for an event its scheduler cannot yet deliver.

The [active rebuild plan](../../package/pi-plugin/auto-mode/doc/planning/ground-up-rebuild.md)
owns current grouping, review budgets, and implementation work.
The private prototypes recorded here are withdrawn historical experiments, not integration candidates.
Their passing fixtures do not authorize maintaining them or migrating their functionality into the plugin.
The agent pursued those mechanisms without user approval.

The user also accepts later extensions changing arguments after review as an unsupported case on 2026-10-19.
Design as though those later mutations do not occur; this does not require ordering guards or fallback prompts.
Historical post-guard mutation qualifications in this document are not remaining work.

Before replacing the shared-review requirement, the user clarified that individual calls could be blocked independently.
That correction remains useful evidence about Pi, but no longer implies a shared batch assessment.
The historical shared-decision and whole-group-denial prototypes do not establish a current execution requirement.

## Historical requirement and budgets

On 2026-10-01,
the user requested fixing the latent per-tool judgment behavior:
parallel tool calls must be judged as one.
The assessment input must contain the complete batch and combined effects,
including interactions that are not visible when each call is assessed separately.
The three-call ceiling,
five-second total deadline,
and average US$0.001 cost target apply to that combined judgment,
not separately to each member.

No member may start before the batch decision.
This is an admission requirement,
not a promise to roll back tools that later fail during execution.
A batch decision does not make filesystem or network operations transactional.

## Composed effects before dispatch details

Assess the enclosing program and relevant dataflow,
not isolated concrete argument objects.
A write that creates `a.js` containing `console.log('meow')`,
followed by `node('a.js')`,
illustrates the distinction:
the execution's effect depends on the preceding write,
not just its filename argument.
Qualified inspected-form semantic effect estimates remain eligible under Q13 B.
This does not require code-only proof of all program effects.

A result-derived value is not automatically missing decision-relevant information.
First determine whether the parent program,
context,
and established scope already cover its effects.
Likewise,
a worker queue drain or another nested callback is a transport event,
not by itself a new judgment or a new model-call budget.
Preserve combined interactions and the original assessment scope.
Unresolved required effect coverage still cannot yield approval.

The proposed blanket choice between assessing overlapping successors separately and rejecting them is withdrawn.
Neither option was adopted.
The runtime-value control `proc_ad78` showed that a later argument can be absent from parent source;
it did not show that the parent lacked information needed for judgment.
The native host and SDK controls remain finite dispatch-mechanics evidence,
not proof of where semantic judgment boundaries belong.

## Reproduction

The provider-free probe invoked the real built auto-mode extension's registered `tool_call` handler
with concurrent synthetic read events and an injected evaluator.
No represented read or provider call was executed.

`proc_20ac` exited 1 with:

```text
# contract/diagnostic/parallel-tool-batch/reproduce.mjs:38
AssertionError [ERR_ASSERTION]: Parallel batch must be assessed once, not once per tool
2 !== 1
```

The returned evidence recorded two evaluator invocations,
each containing only one call's input.
The first received empty sibling context;
the second received an already-completed approval for the first.
This is not one combined judgment.
The real host's parallel scheduler was not exercised by this initial callback reproduction.

The tested built artifact was `package/pi-plugin/auto-mode/dist/final/node/index.mjs`,
SHA-256 `de44e07774252c3268bc893c6493bf02c539ef95243255e3edb107e50c010185`.
The local fixture and retained result live in `contract/diagnostic/parallel-tool-batch/`
of the separate qualification repository.

## Real scheduler reproduction and private fix prototype

A second reproduction used installed pi-agent-core 1.0.0's actual `runAgentLoop`,
a scripted assistant response,
the same built auto-mode extension,
an injected evaluator,
and inert tools.
`proc_2172` exited 1 at the expected assertion:
two evaluations instead of one.
Both low-level hook invocations had access to the complete two-call assistant manifest.
No provider call or real read operation occurred.

The private `batch-admission.mjs` prototype captures immutable member identities and serialized inputs,
shares one in-flight assessment,
and never waits inside the first hook for sibling hooks to arrive.
A changed,
unknown,
or reused member cancels the group.
It is not installed into production auto-mode.

`proc_46b7` exited 0 after 11 controls through the real 1.0.0 scheduler,
plus duplicate-identity rejection:

- Single write and single Bash fixtures each execute their inert tool after one assessment.
- A parallel positive group receives one assessment before both inert executions.
- The combined write-plus-Bash fixture is blocked despite both singleton positives.
- Whole-group denial executes no members.
- Changed second-member input cancels the first member before it can execute.
- Cancellation,
  stale context,
  assessment failure,
  malformed decision,
  and an unknown second member execute nothing.

The tested host-loop SHA-256 is
`65def8c7f3fa01e38fe05467520efc8673c22ea8197833a3b04b29c8a60cb1e3`.
These controls do not qualify nested grouping,
post-guard input mutation,
production extension wiring,
human approval,
or provider calls.
No `AgentSession` was constructed.

## Prepared inputs and private host integration

The prepared dispatcher validates the complete declared set,
finishes native argument preparation and extension hooks,
seals the resulting inputs,
and obtains one shared decision before execution.
`proc_0340` exited 0 for 16 controls,
including after-result hooks,
partial updates,
cancellation,
invalid members,
late mutation,
and execution-entry freshness.
A stale execution throws while retaining its prior assessment and native outcomes;
it does not return an accepted group result.

A private copy of the Pi 1.0.0 dispatcher then passed nine host controls in `proc_669f`.
The strengthened copy passed 13 controls in `proc_75fb`,
also with exit 0.
The strengthened path keeps one admission for a declared group even when a host setting
or member forces serial execution.
It preserves ordered serial execution,
blocks the entire group on denial,
and fails closed when required batch admission is unavailable.
The original unprotected dispatch path remains only when batch admission is not requested.
Start,
update,
end,
and result-delivery behavior was checked in the finite fixtures.
No installed Pi file was edited;
the copies retain Pi's MIT notice.

## Actual SDK session consumer

`proc_1819` exited 0 for two new SDK session controls:
combined allow and combined deny.
The fixture used the private host/Agent forwarding copy with native Pi 1.0.0 `AgentSession`,
`SessionManager`,
`DefaultResourceLoader`,
and extension hooks.
The environment was cleared and the home was disposable.

Each two-member group received one code-owned assessment after native and extension input transformations.
The allow case executed both inert tools;
the deny case executed neither.
Both sessions completed their own scripted final response,
returned from `prompt`,
reached idle,
and persisted their own final and tool-result entries.
The observed extension contexts returned the exact supplied session manager.
The session retained the supplied resource loader,
and the effective prompt contained that loader's marker.

These are two newly observed own completed sessions,
not a reconciliation of the historical session ledger.
The original SDK session controls used the first private host copy.
A subsequent asynchronous-order phase found and corrected concurrent native preparation:
`proc_66a7` exited 1 at the exact preparation-order assertion,
then `proc_700d` exited 0 for nine controls.
Removing the execution-turn wait made the same non-overlap assertion fail.
The corrected implementation separates preparation turns from execution turns:
it releases the next preparation after hooks and sealing,
before waiting for combined admission.
Early failures also release preparation progression.

`proc_1505` subsequently exited 0 for four new native SDK sessions with the corrected frozen helper.
The cases covered parallel execution with ordered asynchronous preparation,
global serial mode,
member-selected serial mode,
and serial group denial.
Each group received one assessment after both transformations.
Serial execution waited for the first tool's asynchronous result hook;
denial executed no tools.
Every newly persisted native entry matched its in-memory entry after JSON serialization.
These controls do not establish arbitrary-tool compatibility or complete stock event/cancellation parity.
No remote model call,
genuine human confirmation,
current permission,
or represented tool action occurred.
The production auto-mode extension and arbitrary nested grouping remain unintegrated.

Private receipts are `sdk-integration-result.json` and `integration-frontier.json`
in `contract/diagnostic/parallel-tool-batch/`.
The latter supersedes the older progress observations in `prepared-controls-frontier.json`.

## Deciding source and integration gap

`package/pi-plugin/auto-mode/src/index.ts:478` copies `currentTurnBatch`
and calls the injected evaluator for the current event.
It adds that event's verdict only after evaluation returns.
The batch is retrospective context,
not a complete pending execution set.

Installed Pi 0.99.2 `dist/core/agent-session.js:308` forwards a per-call event
containing call ID,
tool name,
input,
and optional parent ID.
It does not forward the low-level hook's complete assistant message as a batch manifest.

Installed pi-agent-core 0.99.2 `dist/agent-loop.js:411` prepares each parallel member serially,
awaiting its hook,
then invokes the prepared execution functions through `Promise.all`.
Consequently,
a first per-call hook that waits for all sibling hooks to arrive can deadlock the preparation loop.
Do not implement that barrier.

The same source at `prepareToolCall` applies tool argument preparation and validation before the hook.
Extension `tool_call` handlers can then mutate input;
`extensions/types.d.ts:939` explicitly says later handlers see earlier mutations.
A raw assistant-message snapshot alone is not proof of the final arguments executed.

## Historical target for admission

The admission seam needs a complete,
immutable pending-batch manifest and a shared result before member execution.
It must preserve:

- Explicit batch identity and exact member identities,
  distinct from timing overlap or a shared parent ID.
- Complete effective inputs after relevant argument transformations.
- One code-owned assessment of all members and combined effects.
- One shared decision and one shared call,
  cost,
  deadline,
  and cancellation budget.
- Fixed blocks before bypass or model estimates.
- Rejection of changed membership or inputs rather than reuse of a stale group result.
- Group-scoped human presentation and approval,
  with no grant inferred from machine verdicts.

The default implementation must not release already-approved siblings after a later sibling invalidates the group.
A cached per-call verdict is not a replacement for group admission.
Unflagged members still belong in the combined assessment scope when another member or their interaction needs judging.
Keep the existing finite supported-tool scope;
this requirement does not silently qualify arbitrary custom or MCP tools.

Top-level sibling calls and nested calls require separately verified integration.
Pi's nested runner receives individual `ctx.executeTool()` requests,
not a declaration of an entire future `Promise.all` set.
A shared `parentToolCallId` can include sequential dependent calls too.
Do not group all descendants together or guess group membership from a short timer.
The orchestration seam must supply the actual complete parallel set.

## Withdrawn private-integration work

The following former work list is withdrawn as of 2026-10-19.
Retain it only to explain the historical experiments; do not execute it.

- Integrate the qualified batch-admission seam into the real extension rather than only the private prototype.
- Extend the verified native preparation and result-hook ordering beyond the finite inert-tool fixtures.
- Keep declared-group admission independent of whether member execution is parallel or serial.
- Verify zero execution on group denial,
  changed inputs,
  cancellation,
  or incomplete manifests.
- Demonstrate a combined-effect rejection that individually permissive judgments would miss.
- Keep separate sessions,
  turns,
  sequential calls,
  and unrelated nested groups isolated.
- Verify one group-level prompt and shared model-call accounting.

A finite private fix prototype passed;
a deployed extension fix is not claimed.
The separate native-manager run `proc_3b91` subsequently passed its finite 12-case contract:
three successes and nine expected rejections,
with no model,
AgentSession,
genuine original,
permission,
grant,
or represented action.
That result does not integrate the batch prototype into production.
Current work must instead qualify supported public plugin behavior and its graceful fallback.
