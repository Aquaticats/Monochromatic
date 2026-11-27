# bash-output-filter

Claude Code plugin whose PreToolUse hook filters Bash output,
holds main-session Bash calls to a one-second foreground timeout,
and whose SessionStart hook warns when native background tasks are disabled.

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

The hook gives every eligible main-agent Bash call a foreground timeout of at most `1000` milliseconds.
It no longer forces `run_in_background: true`.
A quick command returns synchronously.
A longer command uses Claude Code's native auto-background result with a task ID and output path,
and keeps running in the background.

### Accepted decision

The 2026-11-25 decision in
[`doc/decision/claude-code-bash-blocking-timeout.md`](../../../doc/decision/claude-code-bash-blocking-timeout.md)
supersedes the blanket forced-background decision of 2026-10-21.
Its amendment of 2026-11-27 clamps caller timeouts and prefixes a leading `sleep`,
after main sessions were measured waiting in front for up to 58 minutes on timeouts they passed themselves
([`doc/planning/claude-code-bash-caller-timeout-cap.md`](../../../doc/planning/claude-code-bash-caller-timeout-cap.md)).
The value comes from the owner's request,
not a workload measurement.

Native delivery is not guaranteed at one second.
Controlled headless and interactive probes on Claude Code 2.1.332 returned some commands around two seconds.
Commands finishing between the nominal timeout and native handoff can return complete output
with a background notice.
The owner accepted that behavior.

### Eligibility and caller intent

The policy changes a call only when every condition holds:

- The event has no `agent_id`.
- `run_in_background` is not `true`.
- The policy is enabled.

Such a call keeps a caller timeout above 0 and at most `1000`;
every other value,
absent and `0` included,
becomes `1000`.
`run_in_background: false` counts as a foreground call.
A bare numeric sleep has no exemption:
it receives `1000` and,
being filter-wrapped,
moves to the background like any other command.

Explicit background calls preserve their flag and their present or absent timeout,
because that timeout is how long the command may run in the background.
When `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` holds a value Claude Code reads as true
(`1`,
`true`,
`yes` or `on`,
in any case),
a caller timeout stays as given and no prefix is added,
because a timeout there stops the command instead of moving it;
only an absent timeout becomes `1000`.

### Leading sleep prefix

Claude Code moves a foreground command to the background at its timeout,
unless the command starts with `sleep`,
which it stops instead (exit 143,
"Command timed out after 1s").
A filter-wrapped command starts with `set -o pipefail`,
so it always moves.
For a policy call the filter leaves unwrapped,
the hook adds `true && ` in front when the first command word,
after blanks,
backslash-newlines and whole comment lines,
is `sleep`,
or when the first statement starts with a redirection such as `> log`,
which Claude Code skips when it looks for the first word:
`sleep 20 && check > out` runs as `true && sleep 20 && check > out`
and moves instead of being stopped.
Output,
exit status and working directory are unchanged;
only `$!` after a prefixed `sleep N &` names the background subshell instead of the `sleep`.
Claude Code itself refuses a leading `sleep` of 25 seconds or more before the hook runs
(probed live on 2026-11-27;
see [`doc/troubleshooting/claude-code-foreground-sleep-block.md`](../../../doc/troubleshooting/claude-code-foreground-sleep-block.md)).
The rule follows Claude Code 2.1.332's own check,
traced in
[`doc/troubleshooting/claude-code-bash-timeout-moves-or-stops-a-command.md`](../../../doc/troubleshooting/claude-code-bash-timeout-moves-or-stops-a-command.md);
re-probe it after a Claude Code upgrade.

Native subagent execution fields and command text remain untouched,
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
In an unattended session (`-p`,
SDK,
CI or cloud),
Claude Code stops a moved command 30 minutes after the move,
or 10 in a `-p` run with a text prompt;
start a longer run there with `run_in_background: true` and its own `timeout`.
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

On 2026-11-27 a real Claude Code 2.1.332 session using the rebuilt bundle moved every main-session case
(a filtered command with `timeout: 3480000`,
unwrapped `sleep`-first commands with a long timeout,
none,
or a comment line first,
and a bare `sleep 5`) at about 2.1 s,
and left an explicit background call and a native subagent's calls untouched;
`doc/planning/claude-code-bash-caller-timeout-cap.md` records the rows.

Real Claude Code 2.1.332 probes verified quick and moved commands,
caller values,
explicit flags,
wrapped and unwrapped sleeps,
`agent_id` versus `agent_type`,
working-directory persistence,
and startup-warning behavior.
The full evidence and final **READY** review are recorded in
[`doc/planning/claude-code-bash-blocking-timeout.md`](../../../doc/planning/claude-code-bash-blocking-timeout.md).
