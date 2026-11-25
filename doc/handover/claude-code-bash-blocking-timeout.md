# Claude Code Bash blocking timeout handover

State recorded 2026-11-25T17:31:28-05:00.
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

## Open work

- Obtain a fresh cross-vendor review of the final record and probe evidence.
- Fold in any findings that earn changes.
- Present the final assembled contract for explicit shared-understanding confirmation.
- Implement only after that confirmation.

## Next step

Request the final review,
record its verdict in the planning document,
and commit before asking for confirmation.
