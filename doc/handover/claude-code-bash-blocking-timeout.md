# Claude Code Bash blocking timeout handover

State recorded 2026-11-25T18:11:37-05:00.
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

## Implementation evidence

Implementation and the tracked bundle are committed.
Source formatting,
both type checks,
the full source-package unit suite,
and real built-plugin consumer probes all pass.
The unit output includes every new predicate and handler suite by name.

## Open work

- Complete the DeepSeek second-pass review and address any earned findings.
- Record that `mise run lint:monochromatic-lint` could not run because
  `monochromatic-lint: command not found`;
  the locked repository linter must not be compiled ad hoc.
  A rendered Markdown check remains before documentation completion.

## Next step

Run the DeepSeek second-pass review,
then finish documentation checks and cleanup.
