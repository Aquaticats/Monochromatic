# Claude Code Bash blocking timeout decision

Accepted by the user on 2026-11-25T18:02:17-05:00
after the final shared-understanding check in
`doc/planning/claude-code-bash-blocking-timeout.md`.
This decision supersedes the blanket forced-background policy recorded in
`package/claude-code-plugin/bash-output-filter/README.md`.

## Amendment of 2026-11-27: caller timeouts are clamped and leading sleeps are prefixed

Accepted by the owner on 2026-11-27T12:29:58-05:00,
who chose "Clamp + prefix (Recommended)" in the question tool,
together with the answers of 2026-11-27T12:26:54-05:00:
"Accept the move (Recommended)" for bare sleeps
and "Keep both (Recommended)" for the hook's own default.
The cause,
measurements,
probes and ranked options are in `doc/planning/claude-code-bash-caller-timeout-cap.md`.
The amendment replaces the caller-timeout and bare-sleep parts of "Timeout policy"
and the exemption in "Sleep and filter behavior".

For a Bash call without `agent_id`,
without `run_in_background: true`,
and with the policy enabled:

- A caller timeout above 0 and at most `1000` stays.
  Every other value,
  absent included,
  becomes `1000`.
  Zero is included because Claude Code 2.1.332 kept an 8-second command with `timeout: 0` in front.
- A command the output filter leaves unwrapped,
  whose first command word after blanks,
  backslash-newlines and whole comment lines is `sleep`,
  or whose first statement starts with a redirection (`> log`),
  is sent as `true && <command>`.
  Claude Code stops a timed-out command whose first word is `sleep` and moves every other one,
  and it skips a statement made only of redirections when it looks for that word,
  so the prefix makes such a command move;
  a filter-wrapped command already starts with `set -o pipefail`.
- The bare numeric-sleep exemption is removed:
  a bare `sleep N` receives `1000` and moves like any other command.
- When `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` holds a value Claude Code reads as true
  (`1`,
  `true`,
  `yes` or `on`,
  in any case),
  a caller timeout stays as given and no prefix is added,
  because a timeout there stops the command instead of moving it;
  an absent timeout still becomes `1000`.
  The startup warning keeps its owner-chosen trigger,
  the exact value `1`.
  This exception was adopted by the implementing session,
  open to the owner's veto,
  from the owner's note of 2026-11-27T12:26:54-05:00 that the clamp should
  "leave them running in background",
  after the Claude Sonnet review of that day pointed out the stop.
  Settings or remote constraints that disable moves set no variable the hook can read;
  bare mode needs no check because hooks from settings files never run there.

Shell commands that REPL code-mode scripts run reach the hook as main-session Bash calls
and get the same treatment,
by the owner's answer "Clamp REPL too (Recommended)" of 2026-11-27T12:49:24-05:00:
a script's `.timeout(ms)` no longer keeps a long command in front:
the script gets the command's task ID (`.running`) and no exit code,
and the result arrives with the task notification
(live check at 2026-11-27T12:50 -05:00:
an 8-second command with `.timeout(600000)` moved at 2058 ms).

Unchanged:
native subagents keep their execution fields and get no prefix;
explicit background calls keep their flag and timeout;
PowerShell,
the opt-out variable,
and the startup warning stay as decided.

Accepted consequences:

- No main-session Bash call stays in front much past two seconds;
  a long wait goes on in the background and ends with its task notification.
- In an unattended session (`-p`,
  SDK,
  CI or cloud),
  a moved command is stopped 30 minutes after the move,
  or 10 in a `-p` run with a text prompt.
- Claude Code refuses a call whose first statement is `sleep N` with N of 25 or more
  ("Do not chain shorter sleeps to work around this block")
  before any hook runs,
  so the prefix does not get such a call past it:
  live at 2026-11-27T13:19 -05:00,
  `sleep 30 && printf g1-done > g1.out` was refused in 13 ms and never reached a `PreToolUse` logger,
  while `sleep 20 && ...` reached the hook,
  was prefixed and moved
  (`doc/troubleshooting/claude-code-foreground-sleep-block.md` records the guard).
- After a prefixed `sleep N & ...`,
  `$!` names the background subshell rather than the `sleep`.
- The redirection-led and backslash-newline prefix cases and the `$!` difference were adopted by the implementing session
  after the second-pass review of 2026-11-27,
  open to the owner's veto,
  as following from the owner's choice that clamped commands keep running.
- The prefix relies on Claude Code reading only the first command word.
  A release that read further would stop prefixed sleeps at their timeout,
  so the behavior is re-probed after each Claude Code upgrade as
  `doc/troubleshooting/claude-code-bash-timeout-moves-or-stops-a-command.md` describes.
- A `cd` in any command lasting longer than about two seconds does not persist.

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

The caller-timeout and bare-sleep conditions in this section are superseded
by "Amendment of 2026-11-27: caller timeouts are clamped and leading sleeps are prefixed".

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

The bare-sleep exemption in this section is superseded
by "Amendment of 2026-11-27: caller timeouts are clamped and leading sleeps are prefixed".

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

The bare-sleep requirement in this list is superseded
by "Amendment of 2026-11-27: caller timeouts are clamped and leading sleeps are prefixed".

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
