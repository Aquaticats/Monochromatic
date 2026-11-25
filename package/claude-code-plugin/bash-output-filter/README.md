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
its merged output through a filter beside the hook bundle:

```text
set -o pipefail && original_command 2>&1 | node filter.mjs && true
```

**Key design decisions:**

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
   `set -o pipefail` makes the pipeline report the original command's status
  rather than the filter's,
   so a failure still reaches Claude Code while the filter reads the whole stream
- **Works with the pipe bug**:
   the sandbox's `< /dev/null` redirect
  lands on `true` (the last simple command),
   not the filter

The retired wrapper replayed `PIPESTATUS[0]` after a `;` boundary instead;
[`TROUBLESHOOTING.md`](TROUBLESHOOTING.md) records why the sandbox made that form fail
(its "Constraint 2").

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
- Commands already piping through the filter (`filter.mjs`,
   `filter.ts`)
- Background commands (`&`,
   `nohup`,
   `setsid`)
- Interactive docker/podman exec/run
- `bun build` (structured output needed for verification)

## Setup

The plugin is registered via `.claude-plugin/plugin.json`.
It hooks into `PreToolUse` with a `Bash` matcher
and `SessionStart` with a `startup` matcher.
No manual configuration is needed beyond installing the plugin.

## Blocking timeout

The hook supplies `timeout: 1000` for eligible main-agent Bash calls that have no caller-supplied timeout.
It no longer forces `run_in_background: true`.
A quick command returns synchronously.
A longer command uses Claude Code's native auto-background result with a task ID and output path.

### Accepted decision

The 2026-11-25 decision in
[`doc/decision/claude-code-bash-blocking-timeout.md`](../../../doc/decision/claude-code-bash-blocking-timeout.md)
supersedes the blanket forced-background decision of 2026-10-21.
The value comes from the owner's request,
not a workload measurement.

Native delivery is not guaranteed at one second.
Controlled headless and interactive probes on Claude Code 2.1.332 returned some commands around two seconds.
Commands finishing between the nominal timeout and native handoff can return complete output
with a background notice.
The owner accepted that behavior.

### Eligibility and caller intent

The default applies only when every condition holds:

- The event has no `agent_id`.
- `run_in_background` is not `true`.
- The caller did not supply `timeout`.
- The command is not a bare numeric sleep under the existing production predicate.
- The policy is enabled.

Explicit background calls preserve their flag and their present or absent timeout.
Every caller-supplied timeout is preserved,
including `0`,
`500`,
and `600000`.
`run_in_background: false` receives the default only when timeout is absent.

Native subagent execution fields remain untouched,
but output filtering still applies to subagents.
A main session started with `--agent` has `agent_type` without `agent_id`
and remains in scope.
Separate `spawn-claude` child sessions are main sessions in their own processes and remain in scope.
PowerShell is unchanged.

### Output filtering and buffering

Timeout eligibility is independent of the output-filter allowlist and denylist.
A filter-skipped command retains its original command text and still receives the timeout when eligible.
All rewrites preserve the complete original input,
including `description`.

The filter reads stdin to EOF before emitting filtered text.
A moved filtered command therefore has no filtered interim output,
and a filtered command terminated at its foreground timeout can lose buffered diagnostics.
The owner explicitly accepted this limitation.

### Configuration

`MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED` controls the policy.
Exact lowercase `0`,
`false`,
and `off` disable it.
Unset and every other value enable it.
The duration is fixed,
not a configurable setting.

`MONOCHROMATIC_BASH_FORCE_BACKGROUND` has no effect.
Sessions using the old `off` setting return to the enabled default unless they migrate.

### Disabled native background tasks

When `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` and the timeout policy is enabled,
a synchronous `SessionStart` handler emits one `systemMessage` at startup.
It does not re-warn on resume or clear.
No warning appears when the timeout policy is disabled.

The warning does not change the command,
timeout,
or background mode.
A timeout can therefore terminate the command when native auto-backgrounding is unavailable.
Bare mode has the same termination risk without a promised warning.
Interactive warning visibility is verified.
Headless evidence is structured hook output,
not a claim that every output surface displays the message.

### Results and working directory

A moved command returns a task ID and output path.
The agent reads output with `Read` after the exit notification arrives.
Claude Code's native result is sufficient;
the hook adds no second completion notice.

A successful `cd` in a foreground result can persist.
A `cd` in a command returning as a background task does not persist.
The owner accepted this timing-dependent behavior.

### Follow-through

Backgrounding does not finish the work:
 retain the task or run identifier,
 collect its terminal result,
 and investigate its failures.
 Starting a watcher is not completion.
 Resource containment and successful follow-through cannot be inferred from background execution alone.

### Verification

Real Claude Code 2.1.332 probes verified quick and moved commands,
caller values,
explicit flags,
wrapped and unwrapped sleeps,
`agent_id` versus `agent_type`,
working-directory persistence,
and startup-warning behavior.
The full evidence and final **READY** review are recorded in
[`doc/planning/claude-code-bash-blocking-timeout.md`](../../../doc/planning/claude-code-bash-blocking-timeout.md).
