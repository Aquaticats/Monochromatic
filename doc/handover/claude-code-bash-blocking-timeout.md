# Claude Code Bash blocking timeout handover

State recorded 2026-11-25T17:57:43-05:00.
This work is incomplete and unapproved.

## Current state

The design tree is recorded in
`doc/planning/claude-code-bash-blocking-timeout.md`.
Its user-question frontier is empty.
No implementation is authorized until the user confirms the assembled design after review and probes.

The current checkout has unrelated concurrent edits.
Do not restore,
stash,
or revert them.

## Completed validation

DeepSeek and Claude Sonnet reviewed earlier design snapshots.
Their earned findings are folded into the planning record.
Real Claude Code 2.1.332 probes now cover bare and chained sleeps,
quick and slow foreground commands,
caller-supplied timeouts,
explicit background and foreground flags,
cwd persistence,
native subagent versus `--agent` main-session boundaries,
and startup,
resume,
policy-disabled,
and background-disabled warning behavior.
An interactive PTY probe confirmed that the startup warning is visible,
while noninteractive text mode exposes it only through structured hook output.
The user explicitly accepted the filter's read-to-EOF buffering limitation in `Q20 A`.

## Open work

- Measure controlled command durations and result arrival in wrapped and unwrapped,
  headless and interactive sessions.
- The user accepted result arrival beyond the nominal one-second timeout in `Q21 A`.
- Obtain a fresh final review of the accepted design and complete evidence.
- Present the final assembled contract for explicit shared-understanding confirmation.
- Implement only after that confirmation.

## Next step

Obtain the final review,
record its verdict,
then request explicit shared-understanding confirmation.
