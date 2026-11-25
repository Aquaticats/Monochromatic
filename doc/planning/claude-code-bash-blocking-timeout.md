# Claude Code Bash blocking timeout planning

Purpose:
the design tree for replacing forced immediate backgrounding in the Claude Code Bash hook
with a one-second foreground timeout.
Grilling is in progress.
This record is unapproved,
and no implementation is authorized until the user confirms a shared understanding.

State recorded 2026-11-25T17:30:26-05:00.

## Requirement as stated

The initial request on 2026-11-25 was:

> We want to change our Claude Code hook that turns every command not ran by a subagent background
> to giving every command a 1s blocking timeout.

The user then added:

> Must not intervene with Claude Code native subagents.

The settled execution-policy boundary is that the new timeout rewrite must not affect native subagents.
Their existing output-filter rewrite remains intact.

## Current behavior

`package/claude-code-plugin/source/src/handler/bash-output-filter/force-background.ts`
forces `run_in_background: true` for main-agent Bash calls,
except for bare numeric sleeps and sessions using the documented opt-out.
The `agent_id` field identifies native subagent calls.

`package/claude-code-plugin/source/src/handler/bash-output-filter/index.ts`
still applies the output-filter command rewrite to subagent calls.
Only forced backgrounding excludes them.

`package/claude-code-plugin/bash-output-filter/README.md`
records the existing owner decision,
the opt-out,
the subagent boundary,
and the result-collection contract.

## Mechanism evidence

Installed Claude Code reports version 2.1.332 through `claude --version`.

The current [Claude Code tools reference][tools-reference],
fetched 2026-11-25,
states these rules:

- A foreground command that reaches its timeout moves to the background instead of stopping,
  unless the command starts with `sleep`.
- Disabling background tasks or using bare mode makes the timeout stop the command instead.
- A moved command returns a task ID and output path,
  with an explicit message that it moved after its timeout.
- A successful foreground `cd` can persist,
  while a `cd` in a moved command cannot.
- In unattended sessions,
  `timeout` on a command started with `run_in_background: true` is its background lifetime,
  not a foreground blocking threshold.
- A command moved from the foreground receives the background lifetime default from the moment it moves.

The hook can wrap a command in an output-filter pipeline before Claude Code executes it.
The exact leading-sleep behavior therefore needs a real hooked-session probe before final acceptance;
the original command text alone does not prove which native branch will run.

The prior feature-flagged sleep block in
`doc/troubleshooting/claude-code-foreground-sleep-block.md`
is a different mechanism and is not evidence for this timeout design.

## Settled decisions

The user answered the first frontier on 2026-11-25.
The selected options were `Q1 B`,
`Q2 A`,
`Q3 A`,
`Q4 B`,
`Q5 A`,
`Q6 B`,
`Q7 A`,
`Q8 A`,
and `Q9 A`.
Their combined behavior follows.

### Native subagent boundary

Native subagent commands must not receive execution-field rewrites from the new policy.
Their `timeout` and `run_in_background` fields remain untouched.
The existing output-filter rewrite continues to apply to them.

### Foreground timeout semantics

An eligible main-agent command starts in the foreground.
If it finishes within one second,
it returns its normal synchronous result.
If it remains running,
Claude Code moves it to the background and returns the native task ID and output path.
The timeout is not intended as a general execution-lifetime limit.
A successful foreground `cd` can now persist when the command finishes before the threshold.
A `cd` in a command that moves to the background still does not persist.
This timing-dependent working-directory behavior differs from the old always-background policy
and requires an explicit consumer-boundary probe.
The user accepted this consequence in `Q2 A`,
whose option text explicitly named working-directory persistence at the threshold.
Later relative-path commands can therefore run in a different directory depending on command duration;
this is an accepted native consequence,
not an unresolved design branch.

### Explicit background calls

A call already carrying `run_in_background: true` keeps that flag and its original `timeout`,
whether present or absent.
The hook never converts the field into a one-second background lifetime.
Output filtering remains independent.

### Caller-supplied foreground timeouts

A foreground call with any caller-supplied `timeout` keeps that value,
including values shorter or longer than one second.
The hook supplies `1000` only when `timeout` is absent.
This preserves explicit caller control rather than imposing a universal override.
The value is a default,
not a cap:
a caller can still request a ten-minute foreground wait.
Zero,
negative,
and other caller-supplied values are preserved for native validation and semantics;
only `undefined` means absent.

### Explicit foreground calls

A call carrying `run_in_background: false` is eligible for the default timeout.
If its `timeout` is absent,
the hook supplies `1000`,
and native auto-backgrounding can still move it when that timeout expires.
A caller-supplied timeout still wins under the caller-timeout decision.

### Sleep boundary

The current bare numeric-sleep exemption remains exactly as implemented.
A command such as `sleep 30` receives no new default timeout.
Output filtering remains independent,
so a filtered sleep can still be wrapped in the pipeline that starts with `set -o pipefail`.
The exemption therefore must not be described as proof that native Claude Code sees a leading `sleep`.
Its real wrapped and unwrapped behavior remains a required probe.
A chained command such as `sleep 2 && make` is eligible when its timeout is absent.
The user accepted in `Q6 B` that chained sleeps can terminate.
Because filtering can change the leading command seen by Claude Code,
a filtered chain can move while an unwrapped leading-sleep chain terminates.
That native divergence is accepted and must be probed in both forms.
Unit suffixes,
extra arguments,
redirects,
and chaining remain outside the bare numeric-sleep exemption.

### Opt-out and threshold

An environment-controlled off switch remains.
When off,
it leaves main-agent execution fields to native Claude Code while retaining output filtering.
The enabled policy uses a fixed `1000` millisecond default,
not a configurable duration.
The user's request is the decision behind that shipped value;
no workload measurement is claimed as its basis.

### Shell surface

The change remains scoped to the existing Bash hook.
PowerShell behavior does not change.

### Legacy opt-out and primary name

The new primary variable is `MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED`.
Its exact lowercase values `0`,
`false`,
and `off` disable the policy.
An unset variable and every other value leave it enabled.

The user first chose to preserve the existing `MONOCHROMATIC_BASH_FORCE_BACKGROUND=off` alias,
then explicitly retracted that choice.
The legacy variable has no effect.
Only `MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED` controls the new policy,
so no primary-versus-legacy precedence branch remains.
Sessions still carrying the old `off` setting return to the enabled default unless they migrate.

### Disabled native background tasks

`CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` is a misconfiguration for this policy.
The hook warns the user once per session but otherwise applies the selected policy normally.
A synchronous `SessionStart` handler in this same plugin emits `systemMessage` on startup.
Its registration uses matcher `startup`,
not `startup|resume`,
and the plugin needs an entry point or event dispatch that handles this event separately from PreToolUse.
The warning detector is the exact value `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` named by the user;
other values and native modes are not claimed to trigger it.
It has no persistent warning marker and does not re-warn on resume or clear.
The warning fires only when our timeout policy is enabled.
If both native background tasks and our policy are disabled,
there is no warning because this hook introduces no one-second termination consequence.
It does not suppress the default timeout,
change the requested background mode,
block the command,
or add a fallback execution path.
Consequently,
a long foreground command can still terminate at its timeout when native auto-backgrounding is unavailable.
No special bare-mode fallback is added.
Bare mode has the same termination consequence without the environment-variable warning;
this limitation must be documented beside the policy.

### Eligibility and composition

Timeout eligibility is independent of output-filter allowlist and denylist decisions.
A filter-skipped command keeps its original command text but still receives the default timeout when eligible.
Both handler return paths must compose the same execution-policy result with or without the filter rewrite.

Only the literal `run_in_background: true` field is the explicit-background branch.
Shell background syntax is not a substitute for that field.
The rewrite spreads the complete original Bash input so `description` and unrelated fields survive.
The output-filter script reads stdin to EOF before emitting filtered text.
A moved filtered command therefore has no filtered output available mid-run,
and a filtered command terminated at its foreground timeout can lose output buffered in the filter.
This is an accepted limitation of retaining the existing filter under the chosen timeout policy,
not a reason to invent a streaming-filter redesign in this change.

Only `agent_id` identifies a native subagent for this boundary.
A main session started with `--agent` can carry `agent_type` without `agent_id` and remains in scope.
A separate `spawn-claude` child session is also a main session in its own process and remains in scope.
Malformed non-Bash inputs and Bash inputs without a string `command` remain outside the handler.

### Completion notice

Claude Code's native moved-to-background result is sufficient.
The hook adds no second completion notice.

## Open frontier

The design frontier is empty.
No behavior remains silently assumed from the answered branches.

A cross-vendor review and native runtime probes remain required before the user is asked
to confirm shared understanding.
Implementation stays unauthorized until that confirmation.

## Review record

A fresh DeepSeek V4.1 Flash review on 2026-11-25 examined the assembled design.
Its valid findings led to the explicit composition,
`agent_id`,
cwd,
caller-value,
bare-mode,
and verification requirements in this record.
Its concern that filter-skipped commands currently lose all execution rewrites was not borne out by
`package/claude-code-plugin/source/src/handler/bash-output-filter/index.ts`:
the current denylist branch already returns `updatedInput` when forced backgrounding applies.
The new implementation must preserve that composition rather than introduce it as new behavior.
A Claude Sonnet gate review then examined snapshot `cd9231b72b`.
It judged that snapshot not ready.
Its composition and verification-list findings overlapped changes already made in `ccfa538859`,
which the review worktree did not contain.
Its sleep-termination and cwd questions were already covered by `Q6 B` and `Q2 A`.
Those answers and the accepted consequences are now explicit in this record.
Its new output-buffering and startup-registration findings are folded in.
A fresh cross-vendor re-review of the final record and probe evidence remains required before confirmation.

## Rejected recommendations

Including native subagents in the new timeout policy was suggested during the initial draft
and is retracted after the user's correction.

## Native probe evidence

Disposable hooks and session logs live under
`/home/user/temp/agent/claude-bash-timeout-native-probes-2026-11-25/keep/`.
All probes ran on Claude Code 2.1.332.

### Bare numeric sleeps

Unwrapped and output-filter-wrapped `sleep 3` calls both remained exempt from the injected default.
Their hook outputs contained no `timeout`,
and both completed normally with no output.
This confirms the chosen exemption but does not bound a bare sleep to one second.

### Chained leading sleeps

The unwrapped `sleep 3 && printf done` hook output added `timeout: 1000`.
Claude Code returned exit code 143 with `Command timed out after 1s`.
The chained `printf` never ran.

The identical command through the real output-filter wrapper also received `timeout: 1000`,
but the executed command began with `set -o pipefail`.
Claude Code moved it to background task `blhskwhvo` at one second.
The native result explicitly said no output had been read when it moved.
The task later exited 0,
and its output file contained `done`.
This proves the accepted filter-dependent termination versus auto-background divergence.

### Timeout input matrix

A disposable session exercised six input shapes through the real filter wrapper:

- `printf quick` with neither execution field received `timeout: 1000`
  and returned `quick` synchronously.
- A caller's `timeout: 500` survived unchanged.
  Its command returned output and a background task ID rather than being rewritten to `1000`.
- A caller's `timeout: 600000` survived unchanged and returned its quick output synchronously.
- A caller's `timeout: 0` survived unchanged and returned its quick output synchronously.
- Explicit `run_in_background: true` with no timeout stayed without a timeout.
  It returned a background task ID immediately and later exited 0.
- Explicit `run_in_background: false` with no timeout received `timeout: 1000`
  and returned its quick output synchronously.

Every rewrite preserved the caller's `description`.

### Working-directory persistence

A quick `cd .../cwd-quick && pwd` received `timeout: 1000`,
finished synchronously,
and the next `pwd` returned `cwd-quick`.
A subsequent `cd ../cwd-moved && node ...` moved to the background at its timeout.
Its own output showed `cwd-moved`,
while the native result said session cwd remained `cwd-quick`.
The next `pwd` confirmed `cwd-quick`.
This verifies the accepted timing-dependent persistence consequence.

### Main custom agent versus native subagent

A real native-subagent Bash event carried both `agent_id` and `agent_type: custom`.
The hook left `timeout` and `run_in_background` absent while retaining the output-filter rewrite.
The command returned normally.
The main session was itself started with `--agent custom`.
Its Bash event carried `agent_type: custom` but no `agent_id`,
so the hook added `timeout: 1000` and retained filtering.
This proves that `agent_id`,
not `agent_type`,
is the correct execution-policy boundary.

### Startup warning

With `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` and the policy enabled,
the synchronous `SessionStart:startup` hook returned the expected `systemMessage`
through the real Claude Code hook stream.
Resume and policy-disabled probes remain.

## Verification still owed

Before shared understanding,
the Claude Sonnet gate review must return and any earned findings must fold into this record.
A real hooked Claude Code 2.1.332 session must then prove these critical mechanisms:

- A hook-injected `timeout: 1000` returns a quick command normally
  and moves a still-running command to a native background task.
- Wrapped and unwrapped sleep forms exhibit their actual behavior;
  no behavior is inferred from the original command text alone.
- A filter-skipped command keeps its command text and receives the eligible execution rewrite.
- Caller timeouts `500`,
  `600000`,
  and `0` survive unchanged.
- Explicit `run_in_background: true` keeps its original mode and absent or supplied timeout.
- `agent_id` suppresses execution-field changes,
  while `agent_type` without `agent_id` does not.
- The startup-only `systemMessage` renders once with the policy enabled,
  does not render with it disabled,
  and does not reappear on resume or clear.
- A quick foreground `cd` and a moved `cd` demonstrate their different persistence rules.

Implementation verification must also cover both existing `run_in_background` rewrite sites,
legacy-variable ignoring,
exact lowercase false-like values,
case-sensitive non-matches,
filter-wrapped sleep,
and the actual output-filter bundle.
The wrapper's share of the one-second budget needs an interleaved unwrapped control;
absolute wrapper timing alone is not a finding.

The existing `MONOCHROMATIC_BASH_FORCE_BACKGROUND` sweep currently finds only implementation,
tests,
the old README,
and this planning record outside bundles.
After acceptance,
the old owner decision in `package/claude-code-plugin/bash-output-filter/README.md`
must be superseded in place and all implementation and documentation references updated together.

## Next action

Finish the independent review and critical native probes,
record their evidence,
then present the assembled contract for explicit shared-understanding confirmation.

[tools-reference]: https://code.claude.com/docs/en/tools-reference#foreground-commands-that-move-to-the-background
