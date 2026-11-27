# Claude Code Bash blocking timeout handover

State recorded 2026-11-25T19:04:42-05:00.
Owner acceptance is not recorded.
The user approved the design and authorized implementation on 2026-11-25T18:02:17-05:00.

## Current state

The design tree is recorded in
`doc/planning/claude-code-bash-blocking-timeout.md`.
Its user-question frontier is empty.
Implementation and the post-review fixes are committed.

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
All five changed Markdown documents rendered to HTML through an isolated `mise` task using `pandoc`.
The repository Markdown lint could not run because `monochromatic-lint: command not found`.
A read-only DeepSeek second pass found no blocker and verified the tracked bundle over 30 payloads.
Its earned in-scope findings are fixed.
The post-review plugin rebuild produced no bundle diff,
and the changed Markdown rendered successfully again.
A fresh normal `claude --print` session without `--plugin-dir` applied the live user-level Bash hook:
a 1.5-second command returned native background task ID `bw23gwxe0`.
The plugin is not installed through `claude plugin list`.

## Live configuration

`/home/user/.claude/settings.json` now registers the rebuilt bundle for both `PreToolUse Bash`
and startup-only `SessionStart`.
With `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1`,
a fresh normal `claude --print` session emitted the full warning exactly once
and successfully returned `startupWired` from its Bash command.

## Open work

- Owner review and acceptance are not recorded.
- Caller timeouts and leading sleeps:
  the owner chose the hook clamp with a sleep prefix on 2026-11-27T12:29:58-05:00
  (`doc/decision/claude-code-bash-blocking-timeout.md`,
  "Amendment of 2026-11-27").
  It is implemented and verified on branch `feat/claude-code-plugin-source-bash-caller-timeout-clamp-and-sleep-prefix`
  in `.claude/worktrees/claude-code-bash-caller-timeout-clamp-and-sleep-prefix`,
  not yet merged:
  the merge makes it live for every running session.
  Waiting on the owner's answers about REPL commands and the merge;
  `doc/planning/claude-code-bash-caller-timeout-cap.md` holds the evidence and the open items.

## Next step

The owner can review the committed implementation and live configuration evidence.
