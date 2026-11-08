# Plan: tool-free Pi goal completion

Status:
 implemented and verified on 2026-08-26 after two grilling rounds.

This plan supersedes the primary-model `goal_complete` interface and completion sections in
`doc/planning/pi-goal-stop-hook.md`.
The unaffected command,
persistence,
process-gating,
model-selection,
and reviewer-availability decisions remain authoritative there.

## Decision

Goal completion is a harness-owned stop decision over finalized primary-model output.
The primary model produces a normal response to the user and is not told that completion review exists.
It does not call,
name,
or receive a `goal_complete` tool,
a completion summary field,
a public generation identifier,
reviewer identity,
review outcome,
or stop-hook protocol.

The primary-model interface becomes only:

```text
/goal <objective>
/goal clear
```

The independent reviewer's private structured-verdict mechanism may remain an implementation detail.
It is not registered in the primary model's tool list and does not appear in the primary system prompt.

## Why the tool interface is wrong

A completion protocol is control policy,
not task work.
Making it a primary-model tool puts the stop decision at the wrong seam and expands the package interface with:

- a public tool name
- a public `goal_id`
- a self-authored completion summary
- final-sibling ordering rules
- local contradiction parsing
- a tool result after the user-facing answer

The protocol also has no correct message placement.
If the model calls `goal_complete` before delivering the answer,
the reviewer cannot inspect the answer.
If the answer and call share one assistant message,
the current evidence serializer excludes the answer with the pending call.
If that message is included instead,
the reviewer judges a tool-using turn before its tool result determines whether another primary turn follows.

Session `01a03c23-5f48-778f-8306-b30a1fddddd2` demonstrated the shared-message failure.
The primary model displayed the requested five explanations and called `goal_complete` in the same assistant message.
The reviewer received neither explanation text nor the answer's actual method selection.
It received earlier denials,
an advisor's elliptic-curve proposal,
and an unexplained `points 0` probe instead.

The durable diagnosis is
`doc/troubleshooting/pi-goal-pending-completion-message-omission.md`.

## Visibility seam

Harness enforcement stays outside primary-model context.
Do not encode it in `AGENTS.md`,
tool metadata,
system-prompt guidance,
visible goal messages,
or continuation protocol text.

The primary may receive only task-level information it needs to act:

- the exact user objective
- later user input
- concrete missing requirements or failed evidence

Injected task context must not identify an independent reviewer,
a completion verdict,
a generation,
or the reason another turn was triggered.
Reviewer identity,
verdict,
transport audit,
and lifecycle state remain extension-private custom entries and user-facing TUI state.

## Target lifecycle

### Natural primary stop

The primary model works normally from the objective presented as ordinary task context.
It uses ordinary tools when work requires them and eventually emits a normal assistant response
with no completion marker or awareness of the stop hook.

Pi `0.84.2` persists each finalized message before the agent run reaches `agent_settled`.
Use `agent_settled` as the completion-review trigger after Pi-owned retries,
overflow compaction,
and queued continuation handling have settled.

### Review eligibility

Before spending a reviewer call,
require all of these conditions:

- the exact run and generation remain active
- the settled run was not aborted
- Pi is idle
- no human steering or follow-up message is pending
- no live process is observed by the existing process monitor
- the selected branch leaf has not already been reviewed for this settlement

Capture run identifier,
generation identifier,
runtime epoch,
branch leaf,
and settlement sequence before asynchronous review.
Revalidate every captured identity before applying a verdict.
Clear,
replacement,
reload,
session replacement,
tree navigation,
or new user input makes an old verdict a no-op.

### Evidence

The reviewer receives:

- the exact active objective
- active-branch entries recorded after the run start
- the finalized assistant response that caused settlement
- finalized tool results and visible goal messages

The reviewer does not receive:

- a primary-authored completion summary
- a pending completion call
- pre-goal history
- abandoned branches
- private reasoning
- custom goal state entries

Advisor and other nested-review results are supporting evidence,
not objective amendments.
The reviewer treats the user objective and later user input as requirements authority.

### Verdicts

The reviewer contract has two outcomes:

- `complete`:
  every objective requirement is supported by finalized evidence
- `continue`:
  required work remains and the primary model can proceed autonomously

A denial carries a contracted task-only `remaining_work` value.
It cannot contain reviewer identity,
verdict language,
evidence scoring,
or stop-hook protocol.
A response that violates this contract is a failed reviewer attempt,
not a valid denial.

`complete` appends durable completed state,
records reviewer identity and truncation audit,
clears the footer,
and ends without a primary-model tool result or another model turn.
A TUI notification may report approval without replacing the assistant's answer.

`continue` appends the private verdict audit and injects one task-level continuation
containing only `remaining_work`.
The exact settlement identity prevents duplicate review or duplicate continuation.
There is no arbitrary consecutive-denial cap.
The active run continues until approval,
user abort,
or `/goal clear`.

Human decisions use `ask_user_question` inside the primary run.
That tool blocks indefinitely until the user answers or cancels,
so completion review does not need a second blocked-on-user state.
The reviewer prompt depends on whether the session can reach a user,
which only the terminal UI can.
In RPC,
JSON,
and print sessions the reviewer is told the user cannot be asked,
and never routes work to `ask_user_question`.
In every mode,
hard,
unsolved,
or unfinished work gets one concrete next attempt that differs from the attempts already shown,
never an instruction to stop,
explain impossibility,
or ask for another task.
The measurements that led to this are in `doc/agent/pi-goal-extension-driven-through-rpc.md`.

Reviewer exhaustion keeps the existing TUI manual decision and non-interactive terminal behavior.

## Interface reduction

Remove the primary completion-tool surface and logic:

- `goal_complete` registration and prompt metadata
- public `goal_id` prompt text
- `GoalCompletionParams`
- sibling-tool finality tracking
- completion-summary lexical preflight
- pending-completion-message exclusion
- primary completion tool results and `terminate: true` behavior

Retain and adapt the deep internal modules for:

- active-branch evidence serialization
- distinct reviewer selection
- structured reviewer transport and fallback
- stale asynchronous result validation
- branch-local review and completion events
- manual reviewer-unavailable handling

The external module becomes deeper:
users learn the `/goal` command,
while the primary model sees only task context and normal tools.
Settlement detection,
evidence capture,
review,
staleness,
and continuation remain harness implementation details.

## Required regression coverage

Implementation is incomplete until tests prove:

- no tool named `goal_complete` is registered or present in primary prompt metadata
- a normal finalized answer is included in reviewer evidence
- the exact five-way answer from session `01a03c23-5f48-778f-8306-b30a1fddddd2` can be approved
- the same session without the final answer is denied
- advisor output cannot replace or amend the user objective
- `ask_user_question` can remain pending without completion review or continuation
- one selected branch leaf is reviewed at most once
- denial emits one feedback continuation
- approval emits no primary tool result and no continuation
- pending human input wins over review and continuation
- abort,
  provider retry,
  overflow compaction,
  live process,
  reload,
  replacement,
  clear,
  and tree navigation preserve their current safety properties
- delayed review results cannot mutate a changed run,
  generation,
  runtime,
  or branch

State-mutating verification must use disposable Pi homes and session files.
The final user-boundary check loads the built extension through Pi,
confirms the primary tool inventory has no completion tool,
and drives approval and denial flows end to end.

## Human-only completion presentation

Approval appends a durable TUI-only completion entry excluded from model context.
Its default rendering is only `Goal complete`.
Expanded rendering shows:

- reviewer identity
- approval rationale
- attempted reviewer identities
- whether reviewer evidence was truncated

The entry preserves the normal assistant answer and adds no primary response.
