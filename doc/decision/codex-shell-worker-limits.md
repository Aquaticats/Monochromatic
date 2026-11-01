# Automatic resource limits for Codex shell commands

Status:
 accepted by the user's implementation request on 2026-10-13.

## Purpose

Apply the existing worker budget automatically before Codex executes a shell command.
The user requested this before continuing the Pi migration because repeating a systemd launcher
in every tool call wastes context.
This request authorizes this focused exception to the general Codex integration exclusion.

## Implementation

The maintained hook is the Rust `codex-bash-limit` binary in
[`codex-plugin/bash-limit`](../../package/codex-plugin/bash-limit/src/main.rs).
Codex's native Bash `PreToolUse` hook adds a cgroup membership write before the original command.
The command cannot continue when attachment fails.
The hook retains native sandbox and approval handling.

The shared `piwork3285273.slice` has these aggregate limits:

- Memory:
  4 GiB.
- Swap:
  zero.
- CPU quota:
  200%,
  with explicit user authorization to increase only this limit if needed.
- Tasks: 2048, increased from 256 at the user's request on 2026-10-15.

Only the slice's `codex` child is writable inside the sandbox.
The parent budget remains read-only.
Codex itself remains outside the worker cgroup.
Child processes inherit their shell's membership.

The installed user systemd slice must persist across reboots.
The hook starts that slice if absent and verifies its limits before permitting a command.
The exact installed hook definition receives Codex's normal persisted trust entry;
global hook-trust bypass is unnecessary.

## Evidence

On 2026-10-15 the user increased task capacity to 2048 after concurrent verification and a browser preview
exhausted the former 256-task limit.
Commit `561ac46f20` updates the hook's exact check and disposable fixtures.
The installed helper, persistent slice definition, runtime parent, and existing `codex` child use 2048 tasks.
Memory, swap, and CPU limits are unchanged.
The package's native test task passes five tests with the opt-in host test skipped;
Clippy passes with warnings denied.
Subsequent real shell calls pass through the installed hook and read both live task limits as 2048.
These checks do not repeat the historical overhead benchmark.

The installed CLI is `codex-cli 0.162.1`.
Its source maps `exec_command` to the native Bash hook,
then changes only the command string when consuming `updatedInput`.
The [official hooks documentation](https://learn.chatgpt.com/docs/hooks)
documents this rewrite contract and persisted hook trust.

The user's Claude settings contain `CLAUDE_CODE_TOOL_MEMORY_LIMIT=8000000000`.
The running `claude-code-bash` cgroup reports that memory limit.
This is precedent for applying limits at the shell boundary;
it does not change the separately authorized Pi worker memory budget.

A disposable Codex sandbox successfully joined a separate test slice's child and reported
`memory.max=4294967296`,
`memory.swap.max=0`,
`cpu.max=200000 100000`,
and `pids.max=256` from its parent.
The initial TypeScript implementation was rejected after 30 unchanged-input measurements:
62.26 ms median,
67.51 ms at the 95th percentile,
and 167.34 ms maximum.
The shell-only control measured 2.40 ms median.
The measurement includes process startup and hook execution;
it does not isolate TypeScript processing from Node startup.

The user requires per-command hook overhead below 30 ms and requested a Rust implementation.
The suggested `jq` shortcut was unnecessary and was never installed as a hook.
The Rust helper uses `serde_json` directly.
The Rust integration fixture runs the installed Codex app-server against a local deterministic Responses endpoint.
It discovers the hook,
trusts its exact hash through the native configuration API,
and verifies actual shell output and membership-file writes.
No model or external provider is involved.

Two host runs executed 40 commands each.
Both reported 2 to 4 ms hook duration,
with a 3 ms median and 3 ms at the 95th percentile.
These are host-reported observations,
not a guarantee against operating-system scheduling delays.
The native binary is installed at `$HOME/.local/libexec/codex-bash-limit`.
`$HOME/.codex/hooks.json` contains trusted,
enabled `SessionStart` and Bash `PreToolUse` hooks.
The original user configuration is backed up under
`$HOME/temp/agent/codex-bash-limit-install-20261013/`.
The persistent user slice definition is `$HOME/.config/systemd/user/piwork3285273.slice`.

The running daemon accepted a configuration reload through its Unix WebSocket control socket.
A command without a manual systemd wrapper reported membership in `piwork3285273.slice/codex`
and the expected parent limits.
The existing turn retained its old sandbox snapshot:
a default sandboxed probe stopped with status 70 and `Read-only file system` before running its payload.
The updated thread settings preserve workspace-write restrictions and add only the child cgroup directory.
The native API documents these settings as applying to subsequent turns.
The user offered to restart this session;
the restart checkpoint requires a default sandboxed membership probe before further builds.

## Package and test ownership

The user specified `package/codex-plugin/bash-limit` on 2026-10-13.
This is a standalone Rust crate.
Rust integration tests run through nextest;
TypeScript does not test this crate.
The earlier mixed-language placement in `task-util` was rejected and removed.
Native fixtures now invoke the built Rust binary from Rust integration tests.
The native fixture suite passed through nextest with default concurrency.
Clippy passes with warnings denied.
The opt-in host fixture also passes.
Removing the attachment guard in a disposable worktree made the committed guard test fail:
the payload exited with status 0 instead of the required 70.

The installed nextest help identifies `--jobs` as an alias of `--test-threads`.
Compilation concurrency uses `--build-jobs`.
The user authorized nextest's default build and test concurrency on 2026-10-13.
The package test task supplies no job or test-thread overrides.
The aggregate worker cgroup still bounds resource use.

The user approved making the native-test distinction explicit in `AGENTS.md`
and restricting the TypeScript test skill's scope.
