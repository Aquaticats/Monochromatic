# guardrail

PreToolUse hook that guards Agent and Bash tool calls with three checks:
resume polling prevention, `bun test` blocking, and process-name wait loop blocking.

## Problem

Claude sometimes polls background agents via `resume` calls instead of waiting
for automatic completion notifications,
 which wastes context tokens on repeated
error messages.
 Separately,
 `bun test` misreports under this repo's custom
`@monochromatic-dev/test-harness-core` harness:
 the harness runs tests as a side
effect of module import,
 so `bun test <file>` prints `PASS` lines and then
reports `0 pass / 0 fail` (bun's runner finds no `bun:test` registrations),
which looks like a broken run when every test passed.

General-purpose Agent calls are no longer blocked here.
 The ban was lifted once
the Claude Code UI let a human observe and message subagent sessions directly;
see `doc/decision/general-purpose-subagent-ban.md`.

## Solution

The handler implements denial responses for three cases:

1. **Resume blocking**:
    when an Agent call includes a `resume` parameter,
   it is denied with a message explaining that background agents notify
   automatically.

2. **`bun test` blocking**:
    when a Bash call invokes `bun test`,
    it is denied
   with a message directing the caller to `mise run //package/<path>:test:unit`
   (or `node <file>` for ad-hoc single-file runs).

3. **Process-name wait loop blocking**:
   when a Bash call runs `pgrep` or `pidof` inside a `while` or `until` loop,
   including inside a `bash -c` script string,
   it is denied
   with a message explaining the self-match and pointing at PID-based waits
   (`timeout 600 tail --pid=<pid> --follow /dev/null`, or `systemd-run --user --wait`).
   Straight-line `pgrep` calls and `kill -0 "$pid"` loops stay allowed.
   `ps | grep` loops are not detected.

## Process-name wait loops

The Bash tool runs each command inside a wrapper shell whose command line contains the command text.
`pgrep --full <pattern>` therefore matches that wrapper,
and a loop waiting for the pattern to disappear never exits.
Measured on 2026-10-19:
`bash -c 'pgrep --list-full --full <unique-string>'` listed both the wrapper shell and the inner `bash -c`.
The check was verified through the live Claude Code host, which denied a bounded probe loop.

## Resume blocking status

On 2026-10-15,
the user reported that resume blocking does not appear to work in Claude Code.
This is a user-observed discrepancy,
not a reproduced failure or an established explanation of the host's behavior.
The handler still returns a denial for an Agent input containing a string `resume` field.
That source behavior alone does not prove Claude Code invokes the hook or enforces its response.

The user wants resumes allowed but deferred removal of the existing check.
Their concern is that Claude Code's non-open-source implementation makes unproven changes undesirable.
Keep the check unchanged until host-level evidence supports a change.
No such runtime verification was performed for this observation.

Follow the [Claude Code upstream bugs policy](../../../.out-of-scope/claude-code-upstream-bugs.md):
do not file or track upstream Claude Code bugs as GitHub issues,
or develop elaborate local workarounds for upstream defects.
This observation does not establish an upstream defect.
Document the uncertainty without treating it as a request for investigation or a speculative workaround.

## Setup

Add to `.claude/settings.json`:

```jsonc
{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [{ "type": "command", "command": "ccgr" }],
      },
    ],
  },
}
```

## Binary

**`ccgr`**:
 Claude Code GuardRail
