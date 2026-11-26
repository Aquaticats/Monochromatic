# Claude Code Bash blocking timeout decision

Accepted by the user on 2026-11-25T18:02:17-05:00
after the final shared-understanding check in
`doc/planning/claude-code-bash-blocking-timeout.md`.
This decision supersedes the blanket forced-background policy recorded in
`package/claude-code-plugin/bash-output-filter/README.md`.

## Scope

The change applies to the Bash tool only.
PowerShell is unchanged.

Only hook events without `agent_id` receive execution-field changes.
Native subagent commands keep their `timeout` and `run_in_background` fields untouched,
but retain output filtering.
A main session started with `--agent`,
and separate `spawn-claude` child sessions,
remain in scope.

## Timeout policy

For an eligible main-agent Bash call,
the hook supplies `timeout: 1000` only when all of these hold:

- `MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED` does not contain exact lowercase `0`,
  `false`,
  or `off`.
- The event has no `agent_id`.
- `run_in_background` is not `true`.
- The caller did not supply `timeout`.
- The command is not a bare numeric sleep under the existing production predicate.

The old forced `run_in_background: true` rewrite is removed without a replacement background flag.
Quick commands return synchronously.
Longer commands use Claude Code's native auto-background result.
The configured timeout is not a strict result-delivery guarantee.
The user accepted measured results around two seconds
and complete output with a background notice for commands finishing between the nominal timeout
and native handoff.

Caller intent has these boundaries:

- Explicit `run_in_background: true` preserves the flag and the caller's present or absent timeout.
- Any caller-supplied timeout is preserved,
  including `0`,
  `500`,
  and `600000`.
- `run_in_background: false` receives the default only when timeout is absent.
- The fixed default is not a cap and not a configurable duration.

The value `1000` comes from the user's request,
not a workload measurement.

## Sleep and filter behavior

The existing bare numeric-sleep exemption remains exactly as implemented.
Chained sleeps are eligible.
The user accepted that an unwrapped leading-sleep chain can terminate at its timeout,
while a filtered chain can auto-background because the wrapper changes the leading command.

Output-filter eligibility is independent of timeout eligibility.
A filter-skipped command retains its original command text and receives the timeout when eligible.
All rewrites preserve the complete original Bash input,
including `description`.

The filter continues to read stdin to EOF.
The user accepted that a moved filtered command can lack filtered interim output
and a terminated filtered command can lose buffered diagnostics.
No streaming-filter redesign belongs to this change.

## Configuration and migration

`MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED` is the only policy variable.
Exact lowercase `0`,
`false`,
and `off` disable it.
Unset and every other value enable it.

`MONOCHROMATIC_BASH_FORCE_BACKGROUND` has no effect.
Implementation,
tests,
and user-facing documentation remove it.
Sessions using the old `off` setting return to the enabled default unless they migrate.

## Disabled native background tasks

When `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` and the timeout policy is enabled,
a synchronous `SessionStart` handler in the same plugin emits one `systemMessage` at startup.
Its registration uses matcher `startup`,
not `startup|resume`.
It has no persistent marker and does not re-warn on resume or clear.

No warning appears when the timeout policy is disabled.
After any warning,
the selected policy proceeds normally:
no timeout suppression,
background-mode rewrite,
command block,
or fallback execution path.
A timeout can therefore terminate the command.
Bare mode has no special fallback or promised warning.

Interactive warning visibility is verified.
Headless evidence is structured hook output,
not proof of visible text on every output surface.

## Results and working directory

Claude Code's native completion result is sufficient.
The hook adds no second notice.

A successful `cd` in a foreground result can persist.
A `cd` in a command returning as a background task does not persist.
The user accepted the resulting timing-dependent working-directory behavior.

## Evidence

The planning record contains the full question history,
review verdicts,
and native probe evidence.
The final fresh Claude Sonnet acceptance review returned **READY**.

Real Claude Code 2.1.332 probes verified:

- bare-sleep exemptions and the wrapped/unwrapped chained-sleep divergence;
- quick and moved foreground commands;
- caller timeout and background-flag preservation;
- `agent_id` versus `agent_type` behavior;
- quick and moved `cd` persistence;
- startup,
  resume,
  policy-disabled,
  and background-disabled warning behavior;
- interactive warning visibility;
- headless and interactive result timing beyond the nominal one-second timeout.

The final review's missing derived measurement file was created from the raw unwrapped log.

## Implementation requirements

- Add the synchronous startup registration and event dispatch without requiring the startup `model` field.
- Replace the old forcing helper and tests,
  preserving the production sleep predicate and its edge cases.
- Cover both previous execution-rewrite sites and filter-skipped composition.
- Verify exact enabled values,
  case sensitivity,
  legacy-variable ignoring,
  and all input-field preservation.
- Supersede the old README decision and update affected anchors.
- Rebuild the tracked bundle and verify the installed plugin at the consumer boundary.
