# bash-output-filter

Claude Code PreToolUse hook that pipes Bash tool output through a text filter
to strip wasteful patterns before the model sees them.

## Problem

Bash tool outputs contain patterns that waste tokens without providing value to the model:

- **Git commit boilerplate**:
   `create mode 100644` lines repeated per file (~5% of all Bash output)
- **Git transport progress**:
   `Enumerating objects`,
   `Counting objects`,
   `Writing objects` counters
- **Long lines**:
   minified JS/CSS lines thousands of chars long (~5% of output)
- **Repeated diagnostics**:
   linters emitting the same message for every violation (~2%)
- **Trailing whitespace**:
   spaces and tabs at end of lines

Combined,
 these account for ~12% of all Bash tool output tokens across typical sessions.

## How it works

The hook intercepts Bash tool calls at PreToolUse and rewrites the command to pipe
through a TypeScript filter:

```text
original_command 2>&1 | node ccbof-filter.mjs; _bof=${PIPESTATUS[0]}; (exit $_bof)
```

**Key design decisions:
**

- **Filter-by-default with denylist**:
   all commands are filtered unless they match
  a denylist (binary tools,
   file redirects,
   background processes,
   double-wrapping)
- **Runs inside the sandbox**:
   the filter executes as the right side of a pipe,
  inheriting the sandbox's filesystem and network restrictions
- **Preserves exit codes**:
   `PIPESTATUS[0]` captures the original command's exit code
  and replays it via `(exit $_bof)` so Claude Code sees failures correctly
- **Works with the pipe bug**:
   the sandbox's `< /dev/null` redirect
  lands on `(exit $_bof)` (the last simple command),
   not the filter

## Filter transformations

1. **Git file mode lines**:
    strips `create mode`,
    `delete mode`,
    `copy mode`,
   `rename`,
    `mode change` lines from commit output
2. **Git transport progress**:
    strips `Enumerating objects`,
    `Counting objects`,
   `Compressing objects`,
    `Writing objects`,
    `Total`,
    `Resolving deltas`,
   `Unpacking objects`,
    and their `remote:` prefixed variants
3. **Long line truncation**:
    lines over 500 characters are truncated with
   `... [N chars]` marker
4. **Consecutive duplicate collapsing**:
    3+ identical consecutive lines become
   `line (x6 repeated lines)` notation
5. **Trailing whitespace**:
    spaces and tabs at end of lines are removed

## Denylist

Commands matching these patterns are **not** filtered:

- Binary/hex tools (`xxd`,
   `hexdump`,
   `base64`,
   `tar`,
   `gzip`,
   etc.)
- Output redirections (`> file`,
   `>> file`)
- Commands already using the filter (`ccbof-filter`)
- Background commands (`&`,
   `nohup`,
   `setsid`)
- Interactive docker/podman exec/run
- `bun build` (structured output needed for verification)

## Setup

The plugin is registered via `.claude-plugin/plugin.json` and hooks into `PreToolUse`
with a `Bash` matcher.
 No manual configuration needed beyond installing the plugin.

## Force background

Every Bash call the main agent makes gets `run_in_background: true` by default,
 even when the tool input supplies `run_in_background: false`,
 because the hook's `updatedInput` rewrite sets the flag after spreading the original input.

### Owner decision (2026-10-21)

Blanket default-on backgrounding is an owner decision confirmed on 2026-10-21,
 not an accidental default to narrow back to commands expected to exceed a minute.
 Relying on Claude to identify which calls will block leaves that same failure mode in control of the choice,
 so the forcing covers every main-agent Bash call.
 The motivating report is foregrounding work that should background,
 such as watching a GitHub Actions run with `gh run watch 123456789012 --exit-status` while independent work waits.
 That report is observed usage,
 not a controlled model-prevalence evaluation.

### Scope

The forcing is broader than the long-command procedure in
 [`doc/handover/cli-git-rust-implementation.md`](../../../doc/handover/cli-git-rust-implementation.md),
 which backgrounds only tests,
 clippy,
 mutation,
 and commands that may exceed a minute.
 That procedure is narrower,
 and it does not bound this hook's policy.
 Setting `MONOCHROMATIC_BASH_FORCE_BACKGROUND` to `off` in the session environment disables the forcing;
 any other value,
 or an unset variable,
 keeps it on.
 Two kinds of call stay in the foreground:

- Subagent calls stay in the foreground,
   because the hook input carries `agent_id` only inside a subagent,
   and a subagent waiting on a command's output would otherwise lose its synchronous result.
- A bare numeric sleep stays in the foreground,
   because backgrounding a pure delay returns nothing to wait on;
   the whole command is exactly `sleep <number>`,
   with a plain numeric duration and no unit suffix,
   chaining,
   redirects,
   or extra arguments.

 Backgrounding is independent of the output-filter denylist:
 commands the denylist skips still run in the background,
 just without the output filter.

### Results and working directory

A backgrounded call returns a task ID and an output file path instead of an immediate synchronous result,
 and the agent reads the output with `Read` once the exit notification arrives.
 A `cd` inside a backgrounded command does not carry over to later calls.

### Follow-through

Backgrounding does not finish the work:
 retain the task or run identifier,
 collect its terminal result,
 and investigate its failures.
 Starting a watcher is not completion.
 Resource containment and successful follow-through cannot be inferred from background execution alone.

### Verification

A real `claude --print` session with this hook returned `Command running in background with ID ...`
 and then the filtered output (`git version 2.56.0`) through the output file.
