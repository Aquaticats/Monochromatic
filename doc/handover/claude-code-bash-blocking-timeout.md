# Claude Code Bash blocking timeout handover

State recorded 2026-11-25T18:00:40-05:00.
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

- The user accepted result arrival beyond the nominal one-second timeout in `Q21 A`.
- The final fresh Claude Sonnet acceptance review returned **READY**.
  Its missing-measurement-file correction is fixed.
- Present the final assembled contract for explicit shared-understanding confirmation.
- Implement only after that confirmation.
- Record that `mise run lint:monochromatic-lint` could not run because
  `monochromatic-lint: command not found`;
  the locked repository linter must not be compiled ad hoc.
  A rendered Markdown check remains before documentation completion.

## Next step

Present the assembled contract and request explicit shared-understanding confirmation.
Implementation remains unauthorized until the user confirms.
