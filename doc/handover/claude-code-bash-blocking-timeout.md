# Claude Code Bash blocking timeout handover

State recorded 2026-11-25T17:14:27-05:00.
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

## Open work

- Obtain a fresh cross-vendor review of the complete design and fold in findings that earn changes.
- Run the real Claude Code 2.1.332 probes for quick and slow foreground commands,
  caller-supplied timeouts,
  explicit background calls,
  wrapped and unwrapped sleep forms,
  native subagent execution-field pass-through,
  and the startup warning.
- Present the final assembled contract for explicit shared-understanding confirmation.

## Start constraint

At 2026-11-25T17:14:27-05:00,
`systemctl --user list-units --no-legend --plain --state=active 'pi-*.service'`
reported 19 active Pi units.
One other Claude agent was also running,
filling the current starting level.
Start one new local Claude reviewer only after the active Pi count falls below 19,
and keep the reviewer and native probe sequential.

The stop condition for the wait is an active Pi count below 19.
The wait has a ten-minute deadline.
If it expires,
reassess the current agent count and report the blocker rather than exceeding the starting level.

## Next step

Run the independent review while the local slot wait proceeds.
When a slot is available,
run the Claude Sonnet gate review and then the native probe,
record their evidence in the planning document,
and commit before asking for confirmation.
