# General-purpose subagent ban

Decision record for whether Claude Code may use the general-purpose (catch-all) Agent subagent.
Decision:
 lifted on 2026-06-12.
General-purpose subagents are allowed,
 as a peer to `spawn-claude` child sessions.
The supervision restriction (interactive sessions with a human watching the Claude Code UI) was removed on 2026-10-29:
the user reported that Claude Code updates fixed most of the problems behind it,
so the `CLAUDE.md` preamble no longer carries a subagent rule.
The sections below record the original reasoning as history.

## Background

The repo previously blocked general-purpose Agent subagents at three layers:

- `.claude/settings.local.json` permission deny `Agent(general-purpose)`,
   the hard gate.
- The `ccgr` PreToolUse guardrail (`package/claude-code-plugin/source/src/handlers/guardrail.ts`),
  which denied the call and redirected to `spawn-claude`.
- A line in the generated `CLAUDE.md` preamble (source `file-enforcer.config.ts`):
  "General purpose agents are banned because of bugs.
  "

Specialized subagents (Explore,
 Plan,
 and peers,
 selected by a named `subagent_type`)
were never banned;
 only the catch-all type was.

## Why the ban existed

The "bugs" were never written down anywhere;
 this reconstruction comes from the people who set the ban:

- The parent agent is blind to how many and which subagents are running,
  and Claude Code provides no tool for the agent to enumerate them.
- Programmatic steering via `SendMessage` is unreliable;
  in the last test it did not work even though it is documented to.
- The ban routed delegation to `spawn-claude` on the premise that a child session's results
  forward back to the parent automatically.
  In fact `spawn-claude` forwarding is unreliable and needs manual monitoring,
  while in-process subagents (the banned path) forward their results reliably.
  The ban pushed delegation toward the less reliable forwarding path.

## Why it is lifted now

The Claude Code UI now lets a human navigate to a running subagent session,
see how many are running,
 and message them directly.
This does not fix the agent-side blindness and does not make `SendMessage` work;
it moves the observe-and-steer role to the human.
With a human supervising in the UI,
 general-purpose subagents are no riskier than
`spawn-claude` child sessions,
 and they forward results reliably,
 which `spawn-claude` does not.

## Scope and boundary

Superseded on 2026-10-29 for the supervision requirement; the list records the boundary as of 2026-06-12.

- Allowed in interactive sessions where a human watches the Claude Code UI.
- Autonomous and headless contexts are out of scope and unsupported in this repo:
  Anthropic now bills autonomous Claude Code contexts at very high rates,
  so the repo does not run them,
   and the agent-side blindness there is moot because the contexts are unused.
- Re-banning should require a concrete,
   reproducible bug,
   not a vague "buggy",
  so the repo does not oscillate on undocumented grounds.

## Terminology

- **General-purpose subagent**:
   the catch-all in-process delegate Claude Code runs
  when no specialized subagent type is named.
- **Specialized subagent**:
   a named in-process delegate (Explore,
   Plan,
   and peers) selected by type.
- **Child Claude session**:
   an independent Claude Code session launched in a visible terminal
  via `spawn-claude`,
   monitored manually for its results.

## What changed

- Removed `Agent(general-purpose)` from the `deny` list in `.claude/settings.local.json`.
- Removed the general-purpose deny branch and its `isGeneralPurpose` helper from `ccgr`;
  the guardrail still blocks Agent `resume` polling and `bun test` invocations.
- Rewrote the `CLAUDE.md` preamble to present in-process subagents and `spawn-claude` child sessions
  as peer mechanisms,
   noting that in-process subagents forward results reliably
  while `spawn-claude` child sessions need manual monitoring.
