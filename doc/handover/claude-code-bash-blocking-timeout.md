# Claude Code Bash blocking timeout handover

State recorded 2026-11-25T18:02:17-05:00.
This work is incomplete.
The user approved the design and authorized implementation at that timestamp.

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

## Accepted design

The canonical contract is `doc/decision/claude-code-bash-blocking-timeout.md`.
Implementation is authorized.

## Open work

- Replace forced backgrounding with the accepted default-timeout policy and startup warning.
- Update tests,
  README decisions and anchors,
  and the tracked bundle.
- Run package type and unit checks,
  then verify the installed plugin in real Claude Code sessions.
- Record that `mise run lint:monochromatic-lint` could not run because
  `monochromatic-lint: command not found`;
  the locked repository linter must not be compiled ad hoc.
  A rendered Markdown check remains before documentation completion.

## Next step

Implement the accepted handler and registration changes,
then run the package verification sequence.
