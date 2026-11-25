# Claude Code Bash blocking timeout planning

Purpose:
the design tree for replacing forced immediate backgrounding in the Claude Code Bash hook
with a one-second foreground timeout.
Grilling is in progress.
This record is unapproved,
and no implementation is authorized until the user confirms a shared understanding.

State recorded 2026-11-25T17:04:31-05:00.

## Requirement as stated

The initial request on 2026-11-25 was:

> We want to change our Claude Code hook that turns every command not ran by a subagent background
> to giving every command a 1s blocking timeout.

The user then added:

> Must not intervene with Claude Code native subagents.

The settled execution-policy boundary is that the new timeout rewrite must not affect native subagents.
Whether the constraint also removes the existing output-filter rewrite for subagents remains open.

## Current behavior

`package/claude-code-plugin/source/src/handler/bash-output-filter/force-background.ts`
forces `run_in_background: true` for main-agent Bash calls,
except for bare numeric sleeps and sessions using the documented opt-out.
The `agent_id` field identifies native subagent calls.

`package/claude-code-plugin/source/src/handler/bash-output-filter/index.ts`
still applies the output-filter command rewrite to subagent calls.
Only forced backgrounding excludes them.

`package/claude-code-plugin/bash-output-filter/README.md`
records the existing owner decision,
the opt-out,
the subagent boundary,
and the result-collection contract.

## Mechanism evidence

Installed Claude Code reports version 2.1.332 through `claude --version`.

The current [Claude Code tools reference][tools-reference],
fetched 2026-11-25,
states these rules:

- A foreground command that reaches its timeout moves to the background instead of stopping,
  unless the command starts with `sleep`.
- Disabling background tasks or using bare mode makes the timeout stop the command instead.
- A moved command returns a task ID and output path,
  with an explicit message that it moved after its timeout.
- A successful foreground `cd` can persist,
  while a `cd` in a moved command cannot.
- In unattended sessions,
  `timeout` on a command started with `run_in_background: true` is its background lifetime,
  not a foreground blocking threshold.
- A command moved from the foreground receives the background lifetime default from the moment it moves.

The hook can wrap a command in an output-filter pipeline before Claude Code executes it.
The exact leading-sleep behavior therefore needs a real hooked-session probe before final acceptance;
the original command text alone does not prove which native branch will run.

The prior feature-flagged sleep block in
`doc/troubleshooting/claude-code-foreground-sleep-block.md`
is a different mechanism and is not evidence for this timeout design.

## Settled decisions

### Agent execution policy

Native subagent commands must not receive the new timeout policy.
Main-agent commands remain the target.

### Threshold

The requested blocking threshold is one second,
which is `1000` milliseconds in the Bash tool input.

## Open frontier

These questions are independent unless their descriptions say otherwise:

- `subagent-filter-boundary`:
  does non-intervention cover only execution fields,
  or must the whole hook pass native subagent calls through without output filtering?
- `timeout-semantics`:
  is the intended result foreground execution for up to one second followed by native auto-backgrounding,
  rather than termination?
- `explicit-background`:
  do calls already requesting `run_in_background: true` keep their original timeout and mode?
- `caller-timeout`:
  does the hook overwrite caller-supplied foreground timeout values,
  including shorter and longer values?
- `explicit-foreground`:
  does the policy apply even when the caller explicitly supplies `run_in_background: false`?
- `sleep-policy`:
  should sleeps retain an exemption,
  or should they receive the same timeout even where native behavior terminates rather than backgrounds them?
- `opt-out-existence`:
  does the new policy retain an environment-controlled off switch?
- `threshold-configurability`:
  is one second fixed,
  or merely the default for a configurable threshold?
- `shell-surface`:
  does the change remain scoped to the existing Bash hook,
  or expand to PowerShell?

Questions about opt-out naming,
migration of existing opt-outs,
notification wording,
verification cases,
and behavior when native backgrounding is unavailable wait for the answers that determine their prerequisites.

## Current recommendations

These are recommendations to grill,
not accepted design:

- Preserve explicit background calls unchanged because rewriting their timeout to one second
  can become a one-second execution lifetime.
- Apply the one-second timeout to every main-agent foreground call,
  including caller-supplied foreground flags and timeout values,
  if the universal policy is confirmed.
- Keep the change on the existing Bash surface.
- Keep one second fixed unless the user wants a new configuration dimension.
- Decide the sleep and opt-out branches explicitly rather than inheriting them silently.

## Rejected recommendations

Including native subagents in the new timeout policy was suggested during the initial draft
and is retracted after the user's correction.

## Verification still owed

Before shared understanding,
a cross-vendor reviewer must re-review the assembled design.
Before implementation is complete,
a real Claude Code session must exercise quick and slow foreground commands,
explicit background calls,
the chosen sleep cases,
and native subagent pass-through.
The probe must use the actual output-filter rewrite so wrapped and unwrapped commands cannot be conflated.

## Next action

Collect the open-frontier answers,
then recompute the design tree and ask only the newly unblocked questions.

[tools-reference]: https://code.claude.com/docs/en/tools-reference#foreground-commands-that-move-to-the-background
