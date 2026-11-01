# Plan: repository-owned Pi goal stop hook

Status:
 implemented in `package/pi-plugin/goal/`.
 The primary-model `goal_complete` interface and completion sections are superseded by
 `doc/planning/pi-goal-tool-free-completion.md` after the 2026-08-26 session diagnosis.
 This file remains the design record for unaffected lifecycle and reviewer behavior.

Tracking issue:
 [#360](https://github.com/Aquaticat/Monochromatic/issues/360).

## Goal

Replace the globally installed `npm:@narumitw/pi-goal` package with a private,
repository-owned Pi extension at `package/pi-plugin/goal/`.
The package name will be `@monochromatic-dev/pi-goal`.

The extension is a stop hook for one active objective.
It starts work on an explicit `/goal <objective>` command and starts another turn
when an active goal's model run settles without completing the goal.
It is not an interruption manager,
a task list,
a background worker,
or a policy layer for unrelated tools.

Completion remains explicit through `goal_complete`,
but the primary model cannot approve its own completion claim.
A different model must approve the submitted evidence,
subject to the manual availability fallback specified in this plan.

## Authority and supersession

This plan records the settled design from the 2026-07-15 grilling session.
It supersedes conflicting details in issue #360 while preserving requirements that were not changed.

The main issue changes are:

- Remove `/goal pause` and `/goal resume`.
- Remove `/goal edit <objective>`.
- Remove bare `/goal` and `/goal status`.
- Remove the complete token-budget subsystem,
  including parsing,
  accounting,
  status,
  prompts,
  transitions,
  commands,
  and tests.
- Treat goal continuation as a stop hook,
  not a paused-state or interruption-recovery state machine.
- Require approval from a different model before successful completion.
- Add a manual interactive fallback only when every eligible reviewer attempt fails.

The issue requirements that remain authoritative include:

- Repository ownership under `package/pi-plugin/goal/`.
- Session custom-entry persistence and active-branch reconstruction.
- Generation validation for stale completion and delayed callbacks.
- No goal-state `tool_call` blocker.
- Terminal `/goal clear` behavior.
- No duplicate continuation turns.
- Current Pi lifecycle APIs and stale-context restrictions.
- Side-effect-free package exports with registration only in the default factory.
- No npm publication.
- Disposable verification state.
- Global migration from the npm package to the repository package path without changing
  project-local `.pi/settings.json`.

## Verified source facts

The plan is based on these inspected sources:

- Installed `@narumitw/pi-goal` version `0.12.0`:
  `/var/home/user/.pi/agent/npm/node_modules/@narumitw/pi-goal/`.
- Installed stale blocker implementation:
  `/var/home/user/.pi/agent/npm/node_modules/@narumitw/pi-goal/src/goal.ts`.
- Upstream source clone at commit `45c290b36e2090d60e871d6d1bc8831ba83bce01`:
  `/var/home/user/temp/agent/pi-extensions-2026-07-15/`.
- Pi extension documentation for installed Pi `0.80.6`:
  `node_modules/.pnpm/@earendil-works+pi-coding-agent@0.80.6/node_modules/
  @earendil-works/pi-coding-agent/docs/extensions.md`.
- Pi session format and active-branch APIs:
  `node_modules/.pnpm/@earendil-works+pi-coding-agent@0.80.6/node_modules/
  @earendil-works/pi-coding-agent/docs/session-format.md`.
- Pi compaction behavior:
  `node_modules/.pnpm/@earendil-works+pi-coding-agent@0.80.6/node_modules/
  @earendil-works/pi-coding-agent/docs/compaction.md`.
- Pi package behavior:
  `node_modules/.pnpm/@earendil-works+pi-coding-agent@0.80.6/node_modules/
  @earendil-works/pi-coding-agent/docs/packages.md`.
- Existing repository Pi package conventions under `package/pi-plugin/`.
- Existing highest-expected-cost secondary-model selection in
  `package/pi-plugin/advisor/` and
  `package/pi-shared/model-selection/`.
- Structured judge transport and fallback behavior in
  `package/pi-plugin/auto-mode/src/judge.ts`,
  `package/pi-plugin/auto-mode/src/judge-stream.ts`,
  and `package/pi-plugin/auto-mode/src/judge-fallback.ts`.
- Prior failed grilling assumptions and user corrections in
  `doc/limitation/kimi.md`.

Pi `0.80.6` exposes the lifecycle and control primitives this design needs:

- `agent_end` reports low-level run messages.
- The `AgentSettledEvent` declaration in Pi `0.80.6` states that `agent_settled` fires after the run has fully settled
  and no automatic retry,
  compaction,
  or queued continuation will run.
- `ctx.isIdle()` and `ctx.hasPendingMessages()` gate continuation.
- `session_start`,
  `session_compact`,
  and `session_tree` expose the reconstruction points needed here.
- `pi.appendEntry()` stores branch-aware custom state outside model context.
- `pi.sendMessage()` can add visible extension-authored context and trigger a turn.
- Pi `0.80.6` passes the current `ExtensionContext` as the final `ToolDefinition.execute` parameter,
  so `goal_complete` can read the current branch,
  model registry,
  mode,
  and UI without a `tool_call` capture hook.
- `executionMode: 'sequential'` on one tool makes its complete assistant tool batch execute in source order.
- `ctx.ui.custom()` can host a combined selector and editor in TUI mode;
  Pi's bundled `questionnaire.ts` example demonstrates that composition.

## Product contract

### Public command surface

Register exactly one command name,
`goal`,
with these accepted forms:

```text
/goal <objective>
/goal clear
```

Register exactly one model-callable tool:

```typescript
// doc/planning/pi-goal-stop-hook.md
type GoalCompleteInput = {
  readonly goal_id: string;
  readonly summary: string;
};
```

Do not expose any of these removed forms:

```text
/goal
/goal status
/goal edit <objective>
/goal pause
/goal resume
/goal --tokens ...
```

Removed forms must return a direct usage diagnostic rather than being reinterpreted as objectives.
In particular,
`status`,
`edit`,
`pause`,
`resume`,
and `--tokens` remain reserved rejected prefixes so an old command does not silently start a goal named after the
removed operation.

### Starting and replacing a goal

`/goal <objective>` trims surrounding whitespace and rejects an empty objective.
Retain the incumbent objective-length guard of 4,000 characters.
Longer instructions should live in a file referenced by the objective.

Starting never asks for replacement confirmation.
If any active or terminal goal record exists,
the command immediately supersedes it with a fresh goal run.
One persisted start event records both the new run and the superseded run identifier,
so reconstruction cannot observe an intermediate state where the old run ended but the new run did not start.

Each new run receives:

- A fresh run identifier.
- A fresh generation identifier used as `goal_id`.
- The exact normalized objective.
- A start timestamp.
- A stable start-boundary marker for reviewer context.
- A continuation sequence starting at zero.

The start command updates the active footer and emits a visible,
extension-authored goal kickoff message.
The kickoff includes the exact objective and generation identifier.
It triggers an agent turn when Pi is idle and no message is pending.

If a start command is handled while Pi is busy,
do not put an irrevocable custom message into Pi's follow-up queue.
Record a generation-scoped kickoff request in the controller instead.
The next safe settlement may emit it only if that run and generation are still active.
Clear,
replacement,
session shutdown,
and branch reconstruction invalidate an old kickoff request.

### Clearing a goal

`/goal clear` is terminal for the current run and generation.
It must:

- Append a branch-local clear tombstone.
- Remove active in-memory state.
- Invalidate kickoff and continuation latches.
- Clear the footer.
- Make every delayed callback for the cleared generation a no-op.
- Avoid aborting an in-flight Pi turn.
- Avoid blocking or modifying any tool call from that turn.
- Avoid triggering another agent turn.

Clearing with no active goal is an idempotent no-op with an informational notification.
It may append no additional tombstone when the active branch is already clear.

The clear event may have a transcript renderer,
but it must remain a custom entry outside model context.
A clear command must not masquerade as a human message.

### Goal footer

Show a footer status only while a goal is active.
The format is:

```text
goal <objective-preview>
```

The objective preview has at most 10 displayed grapheme clusters.
If the objective is longer,
use its first nine displayed grapheme clusters followed by `…`.
The `goal ` prefix is outside this 10-character objective limit.

Clear the footer for every terminal state:

- successful completion
- `/goal clear`
- non-interactive `review_unavailable`
- replacement before the new active footer is installed

Do not leave historical `complete` or `review_unavailable` text in the live footer.

### Extension-authored messages

Kickoff and automatic continuation use visible custom messages with a goal-specific custom type.
They must never use `pi.sendUserMessage()` and must never appear as human-authored input.

Each message details object carries:

- run identifier
- generation identifier
- continuation sequence
- unique message marker
- kind,
  either `kickoff` or `continuation`

Each model-visible body includes:

- exact objective
- exact current generation identifier
- explicit statement that the identifier is only the stale-completion guard
- instruction to continue until the objective is fully complete
- instruction to call `goal_complete` only after requirement-by-requirement verification

The extension registers a renderer that labels these messages as `goal`.
Continuation messages remain visible so the user can audit why another turn started.

### Active system prompt

On every `before_agent_start`,
append goal guidance only when the exact reconstructed state is active.
The guidance includes the exact objective and generation identifier.

The guidance tells the primary model:

- Continue until the objective is complete.
- Do not redefine the objective into a smaller task.
- Use current files,
  command output,
  tests,
  and external state as authority.
- Do not stop at a plan or partial result.
- Expect another stop-hook continuation while the same goal remains active.
- Call `goal_complete` only as the final action after verification.
- Pass the exact current `goal_id`.
- Never reuse an identifier from a replaced,
  cleared,
  terminal,
  or restored generation.

No goal prompt is injected for terminal or absent state.

## Domain model

### Goal run and generation

A goal run is the user-visible lifetime beginning at one `/goal <objective>` command.
A generation is the stale-callback lifetime within that run.

The run identifier stays stable for reviewer context and audit.
The generation identifier rotates when stale work must become invalid without changing the original start boundary.

Create a new run and generation on every start or replacement.
Rotate only the generation when active state is reconstructed into a fresh runtime after:

- startup of a resumed session
- `/reload`
- session resume
- session fork or clone
- active tree navigation

A session replacement must not reuse an old `ExtensionAPI`,
`ExtensionContext`,
`SessionManager`,
or closure-owned runtime epoch.
The new extension instance reconstructs plain persisted data,
creates a fresh generation,
and appends that rotation to the selected branch.

Compaction within the same session runtime does not itself rotate the generation.
It invalidates no active run,
and Pi-owned retry settlement governs continuation.

### Persisted states

The active branch reduces to one of these states:

```typescript
// doc/planning/pi-goal-stop-hook.md
type GoalState =
  | { readonly phase: 'absent'; }
  | ActiveGoalState
  | CompletedGoalState
  | ReviewUnavailableGoalState;
```

`ActiveGoalState` retains:

- run identifier
- generation identifier
- objective
- original start timestamp
- start-boundary marker
- continuation sequence
- latest transition timestamp

`CompletedGoalState` retains:

- run and final generation identifiers
- objective
- completion summary
- approval source,
  either model or manual fallback
- approving reviewer identity when model-approved
- reviewer feedback
- completion timestamp

`ReviewUnavailableGoalState` retains:

- run and final generation identifiers
- objective
- submitted completion summary
- attempted reviewer identities
- normalized terminal diagnostic
- terminal timestamp

A clear tombstone reduces to `absent` rather than a queryable terminal state.
The tombstone remains in session history so old branch state cannot reappear accidentally.

There is no paused state,
resume state,
budget-limited state,
interruption state,
or edit transition.

### Persisted event stream

Use one custom entry type for goal state events.
The reducer understands these semantic events:

- run started,
  optionally naming a superseded run
- generation rotated after active restoration
- review denied,
  retaining feedback while state remains active
- run completed by model approval
- run completed by manual approval
- reviewer unavailable terminal transition
- run cleared

Persist immutable event payloads.
Do not mutate objects already appended to Pi.
The current state is always derived from the active branch's ordered events.

### Branch reconstruction

Reconstruct only from `ctx.sessionManager.getBranch()`.
Never use `getEntries()` as a fallback because it includes abandoned branches.

Reconstruct on:

- `session_start`
- `session_tree`
- `session_compact`

Terminal branch state remains terminal.
Active branch state rotates generation on a fresh extension runtime or tree navigation,
updates the footer,
and does not trigger a model turn merely because it was restored.

Selecting a branch before the run-start event yields absent state.
Selecting a branch after start but before completion restores active state with a fresh generation.
Selecting a branch after a terminal event restores that terminal event without prompt injection or footer state.

## Stop-hook continuation policy

### Governing rule

Continuation is owned by final Pi settlement,
not by interruption state.
Use `agent_end` only to capture the latest low-level outcome and runtime evidence.
Use `agent_settled` as the sole automatic-continuation decision point.

At settlement,
emit at most one continuation when all conditions hold:

- The same run and generation are still active.
- The runtime epoch is current.
- Pi is idle.
- Pi has no steering or follow-up message pending.
- No live `@aliou/pi-processes` process is observed in the current runtime.
- No kickoff or continuation for the same settlement is already emitted.
- The final outcome was not a user abort.

The continuation is a visible custom goal message sent with `triggerTurn: true` only after those checks.
Do not queue it early with `deliverAs: 'followUp'`.
This makes clear and replacement able to invalidate pending intent before any Pi message exists.

### Natural stop and output exhaustion

Continue after an ordinary model `stop`.

Continue after output-length exhaustion.
Pi `0.80.6` rejects possibly truncated tool calls rather than executing them,
so the next goal turn can safely resume from the persisted transcript.

### Errors and compaction

A model error normally lets Pi perform its own provider retry or overflow compaction before `agent_settled`.
The goal extension must not enqueue a competing continuation during `agent_end`,
`session_before_compact`,
or `session_compact`.

After Pi-owned compaction and retry finish,
`agent_settled` reevaluates the exact current generation and continues if it remains active.
This creates one continuation after recovery rather than one before and one after it.

If an errored run reaches `agent_settled` without any compaction having occurred,
log that rare path at debug level and continue anyway.
Do not create an error state or process-global latch.

### User abort

A user abort causes no goal-owned action:

- no transition
- no continuation
- no clear
- no reactivation marker
- no tool policy
- no footer change beyond existing active state

The active goal remains persisted.
A later ordinary agent turn still receives the active goal system prompt.
Restoring the state alone does not trigger that turn.

### Pending human input

Pending steering or follow-up input always wins over goal continuation.
When `ctx.hasPendingMessages()` is true,
do not enqueue a goal message.
The pending turn receives the active goal system prompt naturally.

### Live background processes

Observe successful `process` start and list tool results plus `ad-process:update` lifecycle messages.
Treat `running`,
`terminating`,
and `terminate_timeout` as live states in the current extension runtime.
When any observed process remains live,
do not enqueue a goal continuation.
Keep the goal active and let process notification preferences decide whether process completion starts a later turn.
Do not poll or override those preferences.

### Duplicate prevention

Use generation-scoped message markers and a settlement sequence in the controller.
A callback may emit only after comparing its captured run,
generation,
runtime epoch,
and settlement sequence with current state.

The custom continuation starts one new agent run.
That later run may produce one later continuation at its own settlement.
This repeating stop-hook cycle is intended.
Two continuations for one settlement are not.

## No unrelated tool blocking

The extension must never register a `tool_call` handler for goal state.
It must never return `{ block: true }` for `read`,
`bash`,
`edit`,
`write`,
or any custom tool.

Pause,
resume,
interruption,
and stale-tool latches do not exist in this package.

A stale `goal_complete` call is rejected by the tool's own execution logic.
That rejection has no effect on sibling or later unrelated tools.

The built-extension verifier must assert that the registered event inventory contains no `tool_call` handler.
The crash-regression integration test must exercise ordinary built-in and custom tools after aborted
and errored goal turns.

## Completion contract

### Tool execution discipline

Register `goal_complete` with `executionMode: 'sequential'`.
Pi then executes the complete assistant tool batch in source order.

Require `goal_complete` to be the final tool call in its assistant message.
Earlier sibling calls finish first and their finalized results become reviewer evidence.
If a later sibling call exists,
reject completion before reviewer invocation and instruct the model to submit completion only after every remaining
action.
This prevents approval from racing work scheduled after the completion request without forbidding already ordered
evidence-producing calls.

### Local preflight

Before spending a reviewer call:

1. Require an active goal.
2. Trim and require non-empty `goal_id` and `summary`.
3. Require exact equality with the active generation identifier.
4. Reject a plainly contradictory summary.
5. Capture the current run,
   generation,
   runtime epoch,
   branch leaf,
   and tool-call identifier.

Use a bounded lexical check for explicit contradictions such as:

- `not complete`
- `not done`
- `not finished`
- `still incomplete`
- `tests still fail`
- `tests failing`

Implement the bounded phrase check with string scanning rather than an unbounded regular expression.
The secondary reviewer remains responsible for semantic contradictions and inadequate evidence.

### Reviewer context boundary

The reviewer receives only active-branch context recorded after the current run started.
Edits no longer exist,
and generation rotation after restoration does not reset this run boundary.
Replacement starts a new run and therefore a new reviewer boundary.

Build context from `ctx.sessionManager.getBranch()`:

- Find the current run's persisted start event.
- Exclude every earlier entry.
- Exclude abandoned branches by construction.
- Exclude custom goal state entries from model-visible context.
- Include visible goal kickoff and continuation messages.
- Include assistant messages and finalized tool results.
- Exclude the assistant message containing the pending `goal_complete` call.
- Supply the submitted completion summary separately.

Do not add pre-goal session history to fill space.
If post-start context exceeds the selected reviewer model's request budget,
retain the objective and completion summary outside the truncatable transcript,
then truncate transcript history deterministically to fit.
Record truncation in review details and logs.

Compaction does not authorize adding pre-start material.
The reviewer serializer may use raw post-start branch entries rather than a compaction summary that also covers
pre-goal history.
It must preserve finalized recent evidence and make any older-context omission explicit to the reviewer.

### Reviewer rubric

The reviewer has no tools and judges only supplied objective,
summary,
and post-start session evidence.
The system prompt requires an independent completion decision.

The reviewer approves only when:

- Every objective requirement visible in the evidence is complete.
- The summary is consistent with the transcript.
- Claimed verification is supported by finalized tool output or other visible evidence.
- No known failure,
  blocker,
  TODO,
  or remaining required work appears.
- Completion does not rely on an abandoned branch.

The reviewer rejects when evidence is incomplete,
contradictory,
unverified,
or still reports required work.
Feedback must tell the primary model what remains.

### Reviewer model selection

Resolve the effective Pi model scope through
`@monochromatic-dev/pi-shared-model-selection`.

Exclude the active primary model by exact `provider/id` identity.
The initial reviewer is the eligible scoped model with the highest expected call cost for its model-specific serialized
input estimate and fixed reviewer output reserve.
This is the same default-selection principle used by Advisor.

Goal review has no configuration file.
Use these repository precedents as fixed review constants:

- 10,000 milliseconds for each complete model attempt,
  matching auto-mode's default judge timeout.
- 16,384 output tokens for request budgeting and cost ranking,
  matching Advisor's default maximum secondary-review output.
- 256 reserved framing tokens and four estimated characters per token,
  matching Advisor's context-budget calculation.

The 16,384-token value is a reviewer request reserve,
not the removed goal-work token budget.
Derive each model's maximum serialized-context characters from its context window after these reserves.
Do not add global or project goal configuration for these values in this implementation.

If no distinct authenticated model is eligible,
treat model review as exhausted and enter the manual or non-interactive fallback.
Never silently reuse the active primary model.

Every fallback resolver also excludes:

- the active primary model
- the failed initial reviewer
- every previously selected fallback reviewer

### Structured verdict

The reviewer contract is strict structured data:

```typescript
// doc/planning/pi-goal-stop-hook.md
type GoalReviewVerdict = {
  readonly approved: boolean;
  readonly feedback: string;
};
```

Both fields are required.
A malformed object is a reviewer attempt failure,
not a denial and never an approval.
A valid `approved: false` is a completed denial and does not trigger model fallback.
A valid `approved: true` is approval subject to final stale-generation revalidation.

The forced reviewer tool should have a goal-specific name,
not auto-mode's user-facing `render_verdict` name,
while the transport machinery accepts the tool name as a contract input.

### Auto-mode transport and fallback policy

Extract and reuse auto-mode's structured-judge transport and fallback orchestration rather than copying it into the
goal package.
Preserve these behaviors:

1. Call the selected reviewer with forced structured-tool choice.
2. If the model omits the tool,
   retry without tools and request direct JSON.
3. If that JSON retry returns no text,
   retry direct JSON once more.
4. Treat unexpected tools,
   malformed structured output,
   timeout,
   auth failure,
   and exhausted transport as attempt failures.
5. After the initial reviewer exhausts its complete attempt,
   resolve up to two distinct authenticated fallback reviewers before sending fallback requests.
6. Run selected fallback attempts concurrently.
7. The first fallback promise fulfilled with a successfully parsed valid verdict wins,
   whether approval or denial.
8. A fallback rejected by transport or contract failure does not settle the race
while another fallback can still return a verdict.
   In auto-mode's current source,
   a valid denial is a fulfilled verdict and therefore can win;
   `rejected contender` refers to a rejected promise,
   not a denial verdict.
9. If no fallback exists or every fallback attempt fails,
   enter the manual or non-interactive fallback.

This is availability fallback,
not consensus voting.
Do not call another model merely because a valid reviewer denied completion.

### Stale result revalidation

After every asynchronous model attempt,
fallback race,
and manual dialog,
re-read current controller state before changing it.
Approval has no effect unless all captured values still match:

- run identifier
- generation identifier
- runtime epoch
- active phase
- selected active branch

Clear,
replacement,
reload,
session replacement,
and tree navigation make an old approval a stale no-op.
Return an explicit stale-completion result to the old tool call without touching current state.
Do not use captured old session-bound Pi objects after replacement.

### Model approval

A valid model denial:

- leaves the goal active
- appends a branch-local review-denied event
- returns reviewer feedback to the primary model
- does not set `terminate: true`
- allows the primary model to address the feedback in the current or next goal turn

A valid model approval after stale revalidation:

- appends the terminal completed state
- records approving reviewer identity and feedback
- invalidates continuation latches
- clears the footer
- returns a successful completion result
- sets `terminate: true`

### Exhausted reviewer fallback

When every model attempt fails,
the behavior depends on Pi mode.

In TUI mode,
show one custom combined dialog:

```text
Accept
Reject [optional reason]
```

The interface has two semantic outcomes only.
Escape does not close or settle the custom dialog.
The user must explicitly activate `Accept` or `Reject`.

`Accept` manually approves completion after stale revalidation.
Persist the terminal completion with approval source `manual` and the normalized model-failure diagnostic.

`Reject` keeps the goal active.
If the user entered a reason,
return it to the primary model as completion-rejection feedback.
If the reason is empty,
return a generic manual-rejection message.
A manual rejection may be followed by ordinary stop-hook continuation.

Pi's custom UI is unavailable in RPC mode.
For this selected combined-dialog design,
treat RPC,
JSON,
and print modes as non-interactive completion modes.
Do not substitute a different multi-dialog interaction.

In non-interactive completion modes,
reviewer exhaustion:

- rejects completion
- appends terminal `review_unavailable`
- appends a renderable terminal custom entry so the diagnostic remains visible after footer clearing
- records attempted reviewers and normalized diagnostics
- invalidates continuation latches
- clears the footer
- stops injecting the goal prompt
- stops keeping the goal active
- does not trigger another turn

Starting `/goal <objective>` later replaces this terminal record with a fresh run.

## Shared model-review module

This section is the implementation architecture selected by repository analysis,
not an additional grilled product behavior.
It may change internally if implementation proves a deeper existing seam,
but it must preserve the settled auto-mode transport and fallback behavior without duplication.

### Why a shared module is required

Auto-mode and goal now need the same nontrivial transport and fallback behavior.
Copying that policy would let retries,
provider support,
timeout handling,
and fallback races drift.
Depending directly on auto-mode's extension entry point would couple one deployable plugin to another plugin's command
and security-policy surface.

Create reusable Pi infrastructure instead:

```text
package/pi-shared/model-review/
```

Package name:

```text
@monochromatic-dev/pi-shared-model-review
```

This placement follows the documented `package/pi-shared/` rule:
it is Pi extension infrastructure intended for at least two Pi packages,
not an extension itself.

### Shared interface

Keep the external interface deep and goal-agnostic.
Expose only the orchestration needed by callers:

```typescript
// doc/planning/pi-goal-stop-hook.md
type StructuredReviewContract<TVerdict> = {
  readonly toolName: string;
  readonly tool: unknown;
  readonly parse: (value: unknown) => TVerdict;
  readonly buildJsonRetryPrompt: (input: StructuredReviewPromptInput) => StructuredReviewPrompt;
};

async function runStructuredReviewAttempt<TVerdict>(
  options: StructuredReviewAttemptOptions<TVerdict>,
): Promise<TVerdict>;

async function runReviewWithFallback<TCandidate, TVerdict>(
  options: ReviewWithFallbackOptions<TCandidate, TVerdict>,
): Promise<ReviewWithFallbackResult<TCandidate, TVerdict>>;
```

The exact TypeScript spelling may change during lint-driven design,
but the interface must keep these responsibilities behind the seam:

- Pi AI provider stream dispatch
- forced tool selection
- timeout and abort propagation
- stream collection
- direct-JSON retry sequence
- balanced JSON extraction
- contract parsing
- initial attempt handling
- distinct fallback resolution
- concurrent fallback race
- winning reviewer identity
- complete failure diagnostics

Callers retain:

- model-selection policy
- model auth resolution
- reviewer rubric and prompts
- verdict schema
- interpretation of approval or denial
- manual user interaction
- domain-state transitions

The production adapter uses Pi AI provider streams.
Tests inject a stream adapter and fallback resolver.
These are real seams because production and deterministic test adapters both exist.

### Auto-mode migration

Refactor auto-mode to consume the shared module before goal depends on it.
Auto-mode retains its current user-facing behavior,
verdict type,
messages,
config,
model selection,
and ask-user flow.

Move or generalize the implementation currently owned by:

- `package/pi-plugin/auto-mode/src/judge-fallback.ts`
- shared transport portions of `judge.ts`
- shared stream collection portions of `judge-stream.ts`
- shared JSON extraction portions of `judge-json.ts`
- shared timeout and stream-option portions of `judge-runtime.ts`

Do not move auto-mode-specific action prompts,
trust directives,
batch context,
`approve`/`deny`/`ask` interpretation,
or tool-call blocking decisions.

Characterization tests must prove byte-equivalent verdict behavior and the same fallback selection count,
exclusions,
concurrency,
first-valid-winner rule,
and all-fail diagnostics before deleting old copies.

Advisor's separate full-context review client is not part of this extraction.
A later effort may migrate it if a genuinely shared interface emerges.

## Goal package shape

Create `package/pi-plugin/goal/` with:

- Package name `@monochromatic-dev/pi-goal`.
- `private: true`.
- `sideEffects: false`.
- Root built export from `dist/final/node/index.mjs`.
- `/ts` export from `src/index.ts`.
- `package.json#pi.extensions` containing only `./dist/final/node/index.mjs`.
- Default factory as the only registration entry point.
- No module-scope registration,
  timer,
  process,
  session object,
  or mutable global goal state.
- No npm publication configuration.

Expected workspace dependencies include:

- `@monochromatic-dev/caught-value-core`
- `@monochromatic-dev/logger-core`
- `@monochromatic-dev/ownership-marker-foreign-borrowed`
- `@monochromatic-dev/pi-shared-model-review`
- `@monochromatic-dev/pi-shared-model-selection`

Expected peer dependencies include Pi-provided runtime packages imported by the extension:

- `@earendil-works/pi-ai`
- `@earendil-works/pi-coding-agent`
- `@earendil-works/pi-tui`
- `typebox`

Development dependencies follow sibling built Pi packages:

- Pi peer packages from the pnpm catalog
- `@monochromatic-dev/config-tsdown`
- `@monochromatic-dev/config-typescript`
- `@monochromatic-dev/test-harness-core`
- `@types/node`

Add the normal package files:

- `README.md`
- `package.json`
- `mise.toml`
- `tsconfig.json`
- `tsdown.node.config.ts`

### Internal modules

Keep `src/index.ts` as a shallow Pi adapter around deeper modules.
Split implementation by role before max-lines pressure appears.
The expected concepts are:

- constants and domain types
- persisted state events and active-branch reducer
- generation and runtime-epoch controller
- command parser and handlers
- active prompt construction
- custom goal message construction and rendering
- footer preview formatting
- stop-hook lifecycle policy
- completion-summary preflight
- reviewer context serialization
- reviewer model selection
- structured completion review
- manual fallback UI
- `goal_complete` tool definition
- fake Pi runtime harness
- built-extension verifier
- Pi discovery and lifecycle verifier

File names should describe these concepts directly.
Do not force all logic into one incumbent-style `goal.ts` file.

The controller should be the deep module.
Its interface accepts semantic events and returns effects such as persist,
set footer,
emit message,
notify,
log,
or no-op.
The Pi adapter executes those effects against the current context.
Tests drive the same controller interface with in-memory adapters.

## Logging

Use tagged loggers from `@monochromatic-dev/logger-core`.
Log at module and function boundaries.

Required debug or higher records include:

- extension registration
- branch reconstruction result
- run start and replacement
- generation rotation cause
- clear
- kickoff decision
- settlement outcome
- every continuation gate decision
- compaction observed for an errored run
- errored settlement without compaction
- reviewer scope and selected identity
- context truncation
- each structured transport attempt
- fallback selection and race winner
- reviewer denial
- stale approval suppression
- model approval
- manual approval or rejection
- terminal reviewer unavailability
- session shutdown invalidation

Never log full session context,
secrets,
auth headers,
or provider keys.
Completion summaries and objectives may be logged only in bounded,
non-sensitive diagnostic form;
prefer identifiers and lengths over raw text.

## Test plan

All behavior tests import built output from `dist/final/node/index.mjs` unless they are fixtures or test-only adapters.
Use `@monochromatic-dev/test-harness-core/ts`.
Run builds and tests through package-scoped mise tasks.

### Command tests

Cover:

- empty objective rejection
- objective trimming
- 4,000-character acceptance
- over-limit rejection
- start with no prior state
- immediate replacement of active state
- replacement of terminal state
- idempotent clear with no goal
- clear of active goal
- rejection of bare `/goal`
- rejection of removed `status`,
  `edit`,
  `pause`,
  `resume`,
  and `--tokens` forms
- no replacement confirmation

### State and branch tests

Cover:

- active start event reduction
- replacement in one persisted event
- clear tombstone reduction
- model-approved terminal completion
- manually approved terminal completion
- terminal `review_unavailable`
- review denial retaining active state
- reconstruction from active branch only
- abandoned-branch goal exclusion
- tree navigation before start
- tree navigation into active run
- tree navigation into each terminal state
- generation rotation on active runtime restoration
- stable run start boundary across rotations
- terminal restoration without rotation
- session replacement with no captured old Pi objects

### Footer and rendering tests

Cover:

- active footer format
- objective preview shorter than limit
- objective preview at limit
- truncation to nine graphemes plus ellipsis
- Unicode grapheme safety
- footer clear on every terminal transition
- visible custom kickoff rendering
- visible custom continuation rendering
- extension provenance distinct from human messages

### Stop-hook tests

Cover every settlement branch:

- ordinary model stop continues
- output-length exhaustion continues
- user abort does nothing
- provider retry does not create a competing continuation
- overflow compaction does not create a competing continuation
- post-compaction settlement continues once
- errored settlement without compaction logs debug and continues once
- pending steering suppresses continuation
- pending follow-up suppresses continuation
- non-idle context suppresses continuation
- each live background-process status suppresses continuation
- process lifecycle completion permits a later eligible settlement to continue
- restored active state does not trigger
- one settlement produces at most one message
- later settlement may produce the next sequence
- clear invalidates delayed callback
- replacement invalidates delayed callback
- tree navigation invalidates delayed callback
- session shutdown invalidates delayed callback
- stale kickoff request becomes a no-op

Lifecycle tests capture stderr and must not emit bare shutdown errors such as `context canceled`.

### Completion preflight tests

Cover:

- no active goal
- missing generation identifier
- stale generation identifier
- empty summary
- each bounded contradiction phrase
- non-contradictory evidence
- later sibling tool-call rejection
- earlier sibling calls allowed before final completion
- reviewer not called for local rejection

### Reviewer context tests

Cover:

- context begins after current run start
- pre-goal entries excluded
- abandoned branches excluded
- old replaced run excluded
- generation rotation does not move start boundary
- visible goal messages included
- custom state entries excluded
- current completion-call assistant message excluded
- finalized recent tool results included
- truncation preserves objective and summary
- truncation is disclosed
- compaction cannot reintroduce pre-goal context

### Reviewer selection tests

Cover:

- active primary model excluded
- exact provider/id identity comparison
- highest expected-call-cost eligible reviewer selected
- model-specific context estimate used
- deterministic tie breaking from shared model selection
- no eligible distinct reviewer enters availability fallback
- fallbacks exclude current and prior models

### Shared structured-review tests

Preserve and extend auto-mode coverage for:

- forced tool success
- unexpected tool failure
- missing tool followed by valid direct JSON
- empty direct JSON followed by final valid retry
- malformed JSON failure
- strict contract parser failure
- initial reviewer success without fallback
- initial reviewer denial without fallback
- one available fallback
- two distinct fallbacks selected before requests
- concurrent fallback execution
- first valid fallback winner
- one failed contender with another valid contender
- all fallback attempts failed
- no fallback available
- timeout and abort propagation
- complete diagnostics naming attempted models

### Completion outcome tests

Cover:

- model approval terminal transition
- model denial keeps active and returns feedback
- approval after clear is stale
- approval after replacement is stale
- approval after generation rotation is stale
- valid fallback approval
- valid fallback denial
- TUI manual dialog after all failures
- manual accept completes
- manual reject without reason keeps active
- manual reject with reason keeps active and returns exact reason
- Escape leaves the custom dialog open
- RPC reviewer exhaustion becomes `review_unavailable`
- JSON reviewer exhaustion becomes `review_unavailable`
- print reviewer exhaustion becomes `review_unavailable`
- non-interactive terminal state stops future prompt injection and continuation

### No-blocker regression tests

The fake extension API must prove the default factory registers no `tool_call` handler.

A reproduction harness must drive an aborted and an errored goal run,
then execute representative unrelated tools:

- `read`
- `bash`
- `edit`
- `write`
- another custom tool

Every tool must reach its own implementation without a goal-state block result.
Repeat after clear,
replacement,
review denial,
and `review_unavailable`.

### Built and Pi-runtime verification

Add package tasks equivalent to sibling Pi plugins:

```toml
# package/pi-plugin/goal/mise.toml
[tasks."verify:extension"]
depends = ["build"]
run = "node src/mise.verify-extension.ts"

[tasks."verify:pi-runtime"]
depends = ["build"]
run = "node src/mise.verify-pi-runtime.ts"
```

The built-extension verifier imports `dist/final/node/index.mjs` and checks:

- side-effect-free named imports
- one `/goal` command
- one `goal_complete` tool
- expected lifecycle handlers
- no `tool_call` handler

The Pi-runtime verifier uses Pi's real package discovery against a disposable agent directory.
It must exercise:

- package manifest discovery
- start and immediate replacement
- active footer
- kickoff custom message
- natural settlement continuation
- compaction settlement continuation
- abort no-op
- ordinary tools after abort and error
- matching completion review with fake reviewer transport
- reviewer denial feedback
- reviewer approval termination
- clear during delayed callbacks
- session reload and tree reconstruction without automatic trigger

Use disposable Pi session files,
disposable config,
and fake reviewer/provider adapters.
Never mutate real sessions,
real global settings,
or project-local `.pi/settings.json` during tests.

## Package verification commands

After implementation,
run targeted tasks through mise.
The shared extraction must be verified before goal consumes it.

```bash
mise run //package/pi-shared/model-review:build
mise run //package/pi-shared/model-review:lint:types
mise run //package/pi-shared/model-review:lint:oxlint
mise run //package/pi-shared/model-review:test:unit

mise run //package/pi-plugin/auto-mode:build
mise run //package/pi-plugin/auto-mode:lint:types
mise run //package/pi-plugin/auto-mode:lint:oxlint
mise run //package/pi-plugin/auto-mode:test:unit

mise run //package/pi-plugin/goal:build
mise run //package/pi-plugin/goal:lint:types
mise run //package/pi-plugin/goal:lint:oxlint
mise run //package/pi-plugin/goal:test:unit
mise run //package/pi-plugin/goal:verify:extension
mise run //package/pi-plugin/goal:verify:pi-runtime
```

Run `mise run //package/pi-plugin/goal:lint:types` manually after every TypeScript editing stage.
Do not substitute raw `tsc`,
`tsdown`,
or `bun test`.

## End-user verification

Verification must cross the built artifact and Pi consumer boundary.
Use a disposable home and agent directory for state-mutating scenarios.

Exercise these flows through Pi's actual extension loader:

- Start a goal and inspect the visible custom kickoff.
- Let the model stop without completion and observe one continuation.
- Start a disposable background process,
  let the model settle,
  and observe no goal continuation while the process remains live.
- Finish the disposable process and verify its configured alert behavior owns any later turn.
- Force output exhaustion and observe continuation.
- Force overflow compaction and observe no pre-compaction duplicate,
  then one post-recovery continuation.
- Force a settled model error without compaction,
  observe the debug diagnostic,
  and observe continuation.
- Abort a goal turn and verify no continuation occurs.
- Run unrelated built-in and custom tools after abort and error.
- Replace an active goal and prove the old generation cannot complete.
- Clear while a callback and reviewer result are delayed,
  then prove both become no-ops.
- Restore an active session and verify no turn starts from restoration alone.
- Navigate branches and verify state comes only from the selected branch.
- Approve,
  deny,
  malformed-output retry,
  model fallback,
  manual accept,
  manual reject with reason,
  and non-interactive exhaustion paths.

The crash/interruption regression is not complete merely because the extension loads.
A real post-interruption `read`,
`bash`,
`edit`,
`write`,
and custom tool execution must succeed through Pi.

## Global migration

Do not change real global settings until all disposable verification passes.
Do not change project-local `.pi/settings.json` at any point.

Document the local package installation in `package/pi-plugin/goal/README.md`.
The final global package list must remove:

```text
npm:@narumitw/pi-goal
```

and add:

```text
/var/home/user/Monochromatic/package/pi-plugin/goal
```

Use Pi's package commands or an equivalent scoped settings edit that preserves every unrelated package entry.
Inspect the resulting settings diff.
Do not hand-edit package-manager lockfiles.

After migration:

- Run `pi list` and verify the local package path is present and the npm package is absent.
- Load Pi with no persisted session mutation and confirm only one `/goal` command and one `goal_complete` tool exist.
- Confirm project-local `.pi/settings.json` is unchanged.
- Keep the package private and unpublished.

## Documentation deliverables

Implementation is incomplete until it includes:

- `package/pi-plugin/goal/README.md`
- shared model-review `README.md`
- updated auto-mode documentation naming shared review infrastructure
- troubleshooting documentation for the stale global tool blocker
- local installation and rollback instructions
- exact command and completion contracts
- reviewer cost and provider-data warning
- mode-specific manual fallback behavior
- verification commands and disposable-state guarantees

The goal README must warn that completion review sends post-goal session context to another configured provider
and may incur provider cost.
It must explain that no pre-goal or abandoned-branch context is sent.

## Sequencing and commits

Implementation should proceed in independently verifiable stages.
Commit each stage before starting the next.

1. Add `package/pi-shared/model-review/` with generic structured-review and fallback behavior.
2. Migrate auto-mode to the shared module without changing observable behavior.
3. Add goal package scaffold,
   README,
   pure state events,
   reducer,
   and controller tests.
4. Add command,
   footer,
   custom-message,
   and active-prompt adapters.
5. Add stop-hook lifecycle policy and every continuation regression test.
6. Add reviewer context,
   model selection,
   strict verdict,
   and shared transport integration.
7. Add manual TUI fallback and non-interactive terminal behavior.
8. Add built-extension and Pi-runtime verification.
9. Complete disposable end-user scenarios.
10. Migrate global Pi settings to the local package path and verify the actual consumer boundary.
11. Commit the final migration with `Closes #360` in the commit body.

Earlier commits must not close issue #360 while acceptance criteria remain incomplete.

## Implementation result

Implementation and global migration completed on 2026-07-17.
The global Pi package list contains exactly one repository goal source:

```text
/var/home/user/Monochromatic/package/pi-plugin/goal
```

It contains no `npm:@narumitw/pi-goal` entry.
A fresh global resource load found one `/goal` command,
one `goal_complete` tool,
and no goal-owned `tool_call` handler.
Project-local `.pi/settings.json` remained unchanged at SHA-256
`18e283b90ecb233a9f35bfa23febd572f16586323dfaf4000b39121e1e2fca89`.

The targeted build,
type lint,
Oxlint,
unit test,
built-extension verification,
disposable Pi-runtime verification,
and Markdown lint tasks passed for the changed packages and documentation.
The real `AgentSession` regression exercised built-in and custom tools after abort,
settled error,
replacement,
and clear.

## Explicit non-goals

Do not add:

- multiple concurrent goals
- goal lists or backlog management
- pause or resume
- objective edit
- status query commands
- token budgets or usage accounting
- a goal-owned interruption state
- current-turn abortion
- any global or scoped unrelated-tool blocker
- automatic work on session restoration
- consensus voting among reviewers
- fallback after a valid reviewer denial
- reviewer tools or independent worktree inspection
- project-local goal configuration
- npm publication
- project-local package installation
- compatibility fields that silently accept removed command forms

Do not fix the upstream npm package in place.
The repository-owned extension is the local product path.

## Completion criteria for issue #360

Issue #360 is complete only when:

- The shared model-review module exists and auto-mode consumes it without behavior drift.
- `@monochromatic-dev/pi-goal` exists as a private built Pi package.
- The reduced command and tool surfaces match this plan exactly.
- Goal state reconstructs from the active branch only.
- Every generation-sensitive callback revalidates current state.
- The stop hook continues exactly once at eligible settlement.
- A live background process makes settlement ineligible without terminating the goal.
- User abort produces no goal-owned action.
- Compaction and retry produce no duplicate continuation.
- Reviewer approval,
  denial,
  fallback,
  manual,
  stale,
  and non-interactive paths pass.
- No `tool_call` blocker is registered.
- The real interruption regression proves ordinary tools remain usable.
- README,
  lint,
  type checks,
  unit tests,
  built verification,
  and Pi consumer verification pass.
- Global settings use the repository path and no longer use `npm:@narumitw/pi-goal`.
- Project-local `.pi/settings.json` is unchanged.
- A final commit closes issue #360.

## Open decisions

No user-facing design decisions remain open from the grilling session.

Implementation may reveal a source-level Pi constraint that conflicts with this plan.
If that happens,
record the exact source path and behavior,
then return to the user for the smallest affected user-facing decision.
Do not silently substitute different behavior.
