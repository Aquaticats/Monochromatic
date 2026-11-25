# Claude Code Bash blocking timeout planning

Purpose:
the design tree for replacing forced immediate backgrounding in the Claude Code Bash hook
with a one-second foreground timeout.
Grilling is in progress.
This record is unapproved,
and no implementation is authorized until the user confirms a shared understanding.

State recorded 2026-11-25T17:10:41-05:00.

## Requirement as stated

The initial request on 2026-11-25 was:

> We want to change our Claude Code hook that turns every command not ran by a subagent background
> to giving every command a 1s blocking timeout.

The user then added:

> Must not intervene with Claude Code native subagents.

The settled execution-policy boundary is that the new timeout rewrite must not affect native subagents.
Their existing output-filter rewrite remains intact.

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

The user answered the first frontier on 2026-11-25.
The selected options were `Q1 B`,
`Q2 A`,
`Q3 A`,
`Q4 B`,
`Q5 A`,
`Q6 B`,
`Q7 A`,
`Q8 A`,
and `Q9 A`.
Their combined behavior follows.

### Native subagent boundary

Native subagent commands must not receive execution-field rewrites from the new policy.
Their `timeout` and `run_in_background` fields remain untouched.
The existing output-filter rewrite continues to apply to them.

### Foreground timeout semantics

An eligible main-agent command starts in the foreground.
If it finishes within one second,
it returns its normal synchronous result.
If it remains running,
Claude Code moves it to the background and returns the native task ID and output path.
The timeout is not intended as a general execution-lifetime limit.

### Explicit background calls

A call already carrying `run_in_background: true` keeps that flag and its original `timeout`,
whether present or absent.
The hook never converts the field into a one-second background lifetime.
Output filtering remains independent.

### Caller-supplied foreground timeouts

A foreground call with any caller-supplied `timeout` keeps that value,
including values shorter or longer than one second.
The hook supplies `1000` only when `timeout` is absent.
This preserves explicit caller control rather than imposing a universal override.

### Explicit foreground calls

A call carrying `run_in_background: false` is eligible for the default timeout.
If its `timeout` is absent,
the hook supplies `1000`,
and native auto-backgrounding can still move it when that timeout expires.
A caller-supplied timeout still wins under the caller-timeout decision.

### Sleep boundary

The current bare numeric-sleep exemption remains exactly as implemented.
A command such as `sleep 30` receives no new default timeout.
A chained command such as `sleep 2 && make` is eligible when its timeout is absent.
Unit suffixes,
extra arguments,
redirects,
and chaining remain outside the bare numeric-sleep exemption.

### Opt-out and threshold

An environment-controlled off switch remains.
When off,
it leaves main-agent execution fields to native Claude Code while retaining output filtering.
The enabled policy uses a fixed `1000` millisecond default,
not a configurable duration.

### Shell surface

The change remains scoped to the existing Bash hook.
PowerShell behavior does not change.

### Legacy opt-out and primary name

The existing `MONOCHROMATIC_BASH_FORCE_BACKGROUND=off` setting must remain effective.
The new primary variable is `MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED`.
The accepted values for the new boolean name and its precedence over the legacy alias remain open.

### Disabled native background tasks

`CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` is a misconfiguration for this policy.
The hook warns the user but otherwise applies the selected policy normally.
It does not suppress the default timeout,
change the requested background mode,
block the command,
or add a fallback execution path.
Consequently,
a long foreground command can still terminate at its timeout when native auto-backgrounding is unavailable.
No special bare-mode fallback is added.

### Completion notice

Claude Code's native moved-to-background result is sufficient.
The hook adds no second completion notice.

## Open frontier

The second-round answers unblock these independent questions:

- `enabled-values`:
  which values of `MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED` disable the policy?
- `opt-out-precedence`:
  when the new primary variable and legacy opt-out disagree,
  which one wins?
- `warning-cadence`:
  does the disabled-background-task warning accompany every eligible main-agent Bash call,
  or appear only once per session?

The exact warning text and final verification matrix depend on these answers.

## Current recommendations

These are recommendations to grill,
not accepted design:

- Let the new primary variable win when explicitly present,
  with the legacy `off` alias consulted only when it is absent.
- Accept conventional false-like values for the new boolean variable.
- Use the hook's `systemMessage` warning on every eligible main-agent Bash call
  while the misconfiguration remains,
  so the warning is stateless and appears beside the command that can terminate.

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
