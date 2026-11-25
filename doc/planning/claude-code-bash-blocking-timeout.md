# Claude Code Bash blocking timeout planning

Purpose:
the design tree for replacing forced immediate backgrounding in the Claude Code Bash hook
with a one-second foreground timeout.
Grilling is complete.
The user confirmed shared understanding on 2026-11-25T18:02:17-05:00
and authorized implementation.
The accepted contract is recorded in
`doc/decision/claude-code-bash-blocking-timeout.md`.

State recorded 2026-11-25T18:00:40-05:00.

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
Claude Code can move it to the background and return the native task ID and output path.
The configured foreground timeout is `1000` milliseconds.
The user accepted in `Q21 A` that native results can arrive around two seconds
and that commands finishing between the nominal timeout and native handoff
can return complete output with a background notice.
The configured value is not a strict one-second result-delivery guarantee.
The timeout is not intended as a general execution-lifetime limit.
A successful `cd` can now persist when the command returns in the foreground.
A `cd` in a command that returns as a background task does not persist.
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
Its real wrapped and unwrapped behavior is recorded under native probe evidence.
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
The user explicitly accepted this limitation in `Q20 A`,
which was returned with the label `Q28 A` and interpreted as the only open question's option A.
The filter retains read-to-EOF behavior.
The interim-output and termination-loss consequences must be documented,
and no streaming-filter redesign belongs to this change.

Only `agent_id` identifies a native subagent for this boundary.
A main session started with `--agent` can carry `agent_type` without `agent_id` and remains in scope.
A separate `spawn-claude` child session is also a main session in its own process and remains in scope.
Malformed non-Bash inputs and Bash inputs without a string `command` remain outside the handler.

### Completion notice

Claude Code's native moved-to-background result is sufficient.
The hook adds no second completion notice.

## Open frontier

The design frontier is empty.
The user accepted the measured native result timing in `Q21 A`.
Immediate backgrounding and a custom strict one-second runner were presented and not chosen.

The final cross-vendor review also found that the scratch sleep predicate rejects `.5` and `5.`,
which production accepts.
The tested `sleep 3` evidence remains valid,
but implementation tests must use the production predicate and include those differential cases.
The scratch wrapper probes exercise the real filter executable,
not production allowlist and denylist composition.

The final fresh Claude Sonnet acceptance review returned **READY** on 2026-11-25.
It found no design blocker,
missing user decision,
or critical untested mechanism.
Its only correction was a missing derived unwrapped measurement file,
now created from the raw log.
Implementation stays unauthorized until the user confirms shared understanding.

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
A final Claude Sonnet review examined the record and raw probe evidence.
It returned not ready because nominal `1000` millisecond timeouts did not always produce results at one second.
Several commands returned full output with a background notice after completion.
The controlled duration probes then confirmed that one-second result delivery is not guaranteed.
The user accepted those measurements and the native behavior in `Q21 A`.

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
Claude Code returned background task `blhskwhvo` with `timedOutAfterMs: 1000`.
The native result explicitly said no output had been read when it moved.
The final review calculated a later result arrival,
so this evidence does not establish a one-second return.
The task later exited 0.
A retained copy of its output at
`/home/user/temp/agent/claude-bash-timeout-native-probes-2026-11-25/keep/wrapped-chained-sleep.output`
contains `done`.
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
A subsequent `cd ../cwd-moved && node ...` returned full output with a background task ID.
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
A resumed session emitted no SessionStart hook event,
consistent with matcher `startup`.
With the policy also disabled,
the startup hook ran and returned `{}`.

A slow unwrapped command with native background tasks disabled and our policy enabled
still received `timeout: 1000` after the startup warning.
It returned exit code 143 with `Command timed out after 1s`,
confirming the accepted no-fallback termination behavior.
Clear and compact exclusion follows from the `startup` matcher;
only startup and resume received live event probes.

Noninteractive text output did not display the warning and returned only `ok`.
An interactive PTY session using a disposable HOME did display
`SessionStart:startup says: Claude Code background tasks are disabled ...`
with the full consequence and remedies.
The evidence therefore supports interactive warning visibility and headless structured-hook output,
not a claim that every output surface renders `systemMessage`.

### Controlled headless durations

Each command wrote its own start time.
`resultAfterMs` below is the interval from that marker to the tool result's timestamp,
not a claim about every stage of native delivery.
The measurements are single samples per arm,
so they establish that one-second blocking is not guaranteed but do not establish an exact native floor.

Unwrapped results were:

- Duration 1200:
  1277 ms,
  complete output with a background ID and no `timedOutAfterMs`.
- Duration 1800:
  1829 ms,
  complete output with a background ID and no `timedOutAfterMs`.
- Duration 2500:
  1953 ms,
  `timedOutAfterMs: 1000` and the moved-to-background message.
- Duration 5000:
  1836 ms,
  `timedOutAfterMs: 1000` and the moved-to-background message.
- Caller timeout 500 with duration 5000:
  1953 ms,
  `timedOutAfterMs: 500` and a message rounded to `1s`.

Wrapped results were:

- Duration 1200:
  1297 ms,
  complete output with a background ID and no `timedOutAfterMs`.
- Duration 1800:
  924 ms,
  `timedOutAfterMs: 1000`.
  This timestamp anomaly is retained rather than explained away.
- Duration 2500:
  1949 ms,
  `timedOutAfterMs: 1000`.
- Duration 5000:
  1924 ms,
  `timedOutAfterMs: 1000`.
- Caller timeout 500 with duration 5000:
  1969 ms,
  `timedOutAfterMs: 500` and a message rounded to `1s`.

The measurement files are `duration-unwrapped-measurements.jsonl`
and `duration-wrapped-measurements.jsonl` in the probe evidence folder.

### Controlled interactive durations

The interactive wrapped arm used a positional prompt and a disposable HOME.
Earlier PTY driver attempts that never submitted the prompt were discarded as evidence.
The valid session produced these results:

- Duration 1200:
  1322 ms,
  complete output with a background ID and no `timedOutAfterMs`.
- Duration 1800:
  1875 ms,
  complete output with a background ID and no `timedOutAfterMs`.
- Duration 2500:
  1959 ms,
  `timedOutAfterMs: 1000` and the moved-to-background message.
- Duration 5000:
  1848 ms,
  `timedOutAfterMs: 1000` and the moved-to-background message.

The shorter-timeout control ran only in the headless arms.
After the 5000 ms task completed,
the interactive model did not start the control turn,
so the probe was stopped rather than treating model inactivity as a native timing measurement.
That separate observation does not change the four completed tool-result measurements above
and is not diagnosed as a notification defect.
The valid measurements are in `duration-interactive-measurements.jsonl`.

## Verification still owed

The final fresh Claude Sonnet reviewer read this record,
the timing evidence,
and the user's `Q21 A` acceptance,
and returned **READY**.
Implementation verification remains an implementation-time concern,
not a design blocker.

After acceptance,
implementation verification must cover the filter-skipped composition branch,
both existing `run_in_background` rewrite sites,
legacy-variable ignoring,
exact lowercase false-like values,
case-sensitive non-matches,
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

Present the assembled contract for explicit shared-understanding confirmation.
Do not implement before that confirmation.

[tools-reference]: https://code.claude.com/docs/en/tools-reference#foreground-commands-that-move-to-the-background
