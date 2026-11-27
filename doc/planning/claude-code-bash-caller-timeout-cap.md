# Claude Code Bash caller-timeout cap proposal

Purpose:
explain why main-session Bash calls still wait in the foreground for minutes
under the one-second policy in `doc/decision/claude-code-bash-blocking-timeout.md`,
and propose how to stop it.
Status:
the owner chose "Clamp + prefix" on 2026-11-27T12:29:58-05:00;
the accepted contract is the amendment in `doc/decision/claude-code-bash-blocking-timeout.md`,
and "Implementation and verification" records the work,
which the owner has not reviewed.
First committed 2026-11-27T12:19:19-05:00 (`1f2f98005c`) by a Claude Code session.
A Claude Sonnet gate review of that first version returned at 2026-11-27T12:44:44-05:00;
"Gate review findings" weighs it.

## Question as asked

The owner on 2026-11-27:

> Our bash-output-filter is supposed to auto background Bash calls whenever they run in front in the main session
> for longer than 1min, but in practice a lot of the calls are like:

```text
Running… (3m 36s · timeout 58m)
Async hook PostToolUse completed, Async hook PreToolUse completed
     (ctrl+b to run in background)
```

> Why and what should we do?

and then:

> Sorry, I typoed in my initial message this session. minute -> second.

## Why calls still wait

### The hook skips calls that carry their own timeout

The hook works as the accepted decision wrote it.
`doc/decision/claude-code-bash-blocking-timeout.md`,
section "Timeout policy",
supplies `timeout: 1000` only when "The caller did not supply `timeout`",
and "Any caller-supplied timeout is preserved,
including `0`,
`500`,
and `600000`."
`shouldApplyBlockingTimeout` in
`package/claude-code-plugin/source/src/handler/bash-output-filter/blocking-timeout.ts`
returns `false` when `timeout` is present,
and the committed bundle the live settings run has the same branch.

Without a manual move (Ctrl+B,
a queued user message or a turn abort),
Claude Code moves a foreground command to the background only when the command reaches its own timeout.
A call carrying `timeout: 3480000`,
which the terminal shows as `timeout 58m`,
therefore stays in front for up to 58 minutes.

### How often callers pass a timeout

Measured from the main-session transcripts under `~/.claude/projects`
for assistant messages after 2026-11-25T18:10:00-05:00,
just after the policy landed in `bf59b3e994`:

- 4898 main-session Bash calls,
  1428 of them with a caller timeout.
- `3480000` appeared 87 times;
  caller values ranged from `5000` to `21600000`.
- 17 of 694 main-session REPL scripts passed a timeout through `.timeout(...)` or `$(cmd, ms)`,
  which reaches the hook the same way.

Foreground durations,
measured as the gap between a call's `tool_use` and its `tool_result` timestamps,
which also counts hook,
permission and queueing time and so is an upper bound:

- Calls without a caller timeout:
  2988 matched,
  80 over 3 s,
  2 over 60 s.
- Calls with a caller timeout over 3 s:
  1393 matched,
  484 over 3 s,
  119 over 60 s,
  5 over 10 minutes,
  about 480 minutes in front beyond each call's first 3 s.

The calls without a caller timeout are the control:
the hook's default moves them,
so caller timeouts are the cause of the long waits.
The control is not a matched sample,
since callers choose a timeout for commands they expect to run long;
the hook source and the probes carry the causal claim.

The long waits are concentrated.
The gate review's rerun found one orchestrating session behind 119 of the 120 caller-timeout calls over 60 s
and 1115 of 1462 caller-timeout calls,
so the counts describe that session's habit more than every session's.
It also found 371 caller-timeout calls that took 3 to 60 s;
under the clamp each of those returns a task ID instead of its output,
and its result arrives with the task notification.

In this session,
a REPL command without a timeout moved to the background after 2062 ms,
while REPL commands given `.timeout(300000)` stayed in front until they finished.

### Why callers pass long timeouts

The Bash tool schema in Claude Code 2.1.332 describes the field as
"Optional timeout in milliseconds (max ${...} for a foreground command)",
and `BASH_MAX_TIMEOUT_MS` is `86400000` in `~/.claude/settings.json`,
so the model is told it may wait in front for a day.
The longest-timeout samples are wait loops
(polling run markers,
`timeout 7500 tail --pid=...`)
whose tool timeout sits just above the loop's own bound.
The value `3480000` appears in no string of the 2.1.332 executable,
and in this repository only as a digit run inside unrelated numbers in evidence files;
it is the models' own choice.

The waits are deliberate.
In the interactive session (transcript `entrypoint` `cli`) that issued most of the long timeouts,
descriptions read like "Wait until the gate run reaches a terminal state"
and "Wait until all three gate runs report a terminal result".
They echo the rule at `CLAUDE.md:17` ("wait in one command that ends at a terminal state")
and the bounded PID wait `timeout 600 tail --pid=<pid> --follow /dev/null` at `AGENTS.md:682`;
`doc/handover/lint-sweep.md` lines 386 to 390 prescribe `timeout 3600 tail --pid=<pid>`.
None of these says whether the wait runs in front,
and agents keep it in front by giving the tool a timeout above the wait's own bound.

### The async hook lines

`Async hook PreToolUse completed` and `Async hook PostToolUse completed` come from `cccd`,
registered with `"async": true` for `PreToolUse` and `PostToolUse` in `.claude/settings.local.json`.
They report that hook finishing and do not hold the call.

## A native cap in Claude Code 2.1.332

The installed executable `~/.local/share/claude/versions/2.1.332`
bundles this function,
extracted with `rg --text --only-matching`;
names are minified:

```js
// claude 2.1.332 bundle
var i=2000;function kEo({requestedTimeoutMs:n,isMainAgent:e,canAutoBackground:t,subagentCapMs:o=0,env:u=process.env}){if(!t)return n;let r=e?ug(u.CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS):o;if(isNaN(r)||r<=0)return n;return Math.min(n,Math.max(r,i))}
```

The Bash tool passes its requested timeout through `kEo` before running the command.
For the main agent,
when the command may move to the background,
the timeout becomes the smaller of the requested timeout
and the larger of `CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS` and 2000 ms.
Subagents get `subagentCapMs` instead and never read the variable.
`doc/troubleshooting/claude-code-subagent-background-wait-context-dump.md`
traced the same function in 2.1.316.

The variable is undocumented:
<https://code.claude.com/docs/en/env-vars.md>,
fetched 2026-11-27,
lists `CLAUDE_AUTO_BACKGROUND_TASKS` and `CLAUDE_CODE_MCP_AUTO_BACKGROUND_MS` but not this one.

## Probe of the native cap

### Method

Two disposable `HOME` folders ran on Claude Code 2.1.332 on 2026-11-27 at about 12:14 -05:00.
Each held a copy of the login with its refresh token removed,
the live hook bundle,
`BASH_MAX_TIMEOUT_MS=86400000`,
and a `PreToolUse` logger recording each call's input and `agent_id`.
The capped arm's settings `env` added `CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS=1000`,
the channel this proposal would use;
the control arm left it unset.
Headless `claude -p` sessions on Sonnet ran each case once per arm,
so each result is a single sample.
The disposable folders were deleted afterwards.

### Results

Each item gives the capped arm first and the control arm second.

- Main agent,
  an 8-second `node` command with caller `timeout: 600000`:
  moved at 2245 ms with "Command did not complete within its 2s timeout";
  output in front at 8204 ms.
- Main session started with `--agent probe`
  (`agent_type` set,
  no `agent_id`),
  same command:
  moved at 2193 ms;
  output in front at 8240 ms.
- Native subagent (`agent_id` set,
  started by a foreground `Agent` call),
  same command:
  output in front at 8166 ms and 8156 ms.
  A background subagent was not probed;
  the source reads the variable only for the main agent.
- `sleep 5 && printf m2-done > m2.out` with caller `timeout: 600000`
  (the redirect makes the filter skip it,
  so Claude Code sees a leading `sleep`):
  finished in front at 5084 ms and 5076 ms.
  The cap does not apply to such a command,
  and it was not stopped.
- `sleep 5 && printf m6-done` with caller `timeout: 600000`
  (wrapped by the filter):
  moved at 2070 ms;
  output in front at 5088 ms.
- Bare `sleep 5` without a timeout
  (exempt from the hook's default but wrapped by the filter):
  moved at 2065 ms;
  finished in front at 5077 ms.
- Explicit `run_in_background: true`:
  a background task at once in both arms.
- No caller timeout:
  moved in both arms,
  at 2067 ms and 2058 ms,
  under the hook's `timeout: 1000`.
- A command the filter skips that does not start with `sleep`
  (`node ... > m7.out`,
  an 8-second command with caller `timeout: 600000`,
  run in a second round at about 12:22 -05:00):
  moved at 2145 ms;
  finished in front at 8202 ms.
- REPL code mode:
  not exercised.
  `claude -p` with `CLAUDE_CODE_EXPERIMENTAL_CODE_MODE=1` loaded no REPL tool.

Measured use bearing on the sleep results:
among 4405 foreground main-session Bash calls since the policy landed,
none was a bare numeric sleep and 18 began with a `sleep` chain.

## Options

### Native cap through settings

Add `"CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS": "1000"` to the `env` of `~/.claude/settings.json`.

Pros:
no code change;
probed on the installed version;
moves filter-wrapped and filter-skipped main-session calls alike,
whatever timeout the caller passes,
except a command Claude Code sees starting with `sleep`
(REPL commands are expected to follow but were not probed);
leaves subagents alone;
a command Claude Code sees starting with `sleep` keeps its timeout instead of being stopped.

Cons:
undocumented,
so a Claude Code release can drop it without notice and it needs a re-probe after upgrades;
the floor is 2000 ms,
not 1000;
a filter-wrapped bare `sleep` moves after 2 s instead of waiting;
an unwrapped command starting with `sleep` and carrying its own timeout still waits in front;
a `cd` in a command lasting over 2 s does not persist,
as the decision already accepts for calls without a timeout.

### Cost shared by every option that moves these calls

A moved command runs on in the background,
but in an unattended session it then has a time limit.
The [tools reference][tools-reference-background-limit],
fetched 2026-11-27,
states:
"A command that starts in the foreground and then moves to the background at its timeout gets 30 minutes from the move",
10 minutes in a `-p` run whose prompt is passed as text,
and "A local session you work in from a terminal,
the desktop app,
or the VS Code extension has no time limit on background commands."
The 2.1.332 bundle matches:
the default `QCt()` starts from `PEl=1800000`,
and deadlines apply only when `JCt()` holds.
So in a `-p`,
SDK,
CI or cloud main session,
a long wait that the cap or a clamp moves is stopped at that limit,
where today its own timeout would have kept it running in front.
Such sessions would need to start long waits with `run_in_background: true` and their own `timeout`.
Raising `BASH_DEFAULT_TIMEOUT_MS` would lift the limit,
but it also changes the default foreground timeout of subagents,
which the decision's "Scope" forbids.

### Hook clamps caller timeouts

The hook replaces a main-session caller timeout above 1000 with 1000.

Pros:
uses the documented hook input rewrite;
stays under this repository's tests.

Cons:
on its own,
an unwrapped `sleep`-first chain with a caller timeout is then stopped at one second
(exit 143,
measured on 2026-11-25 in `doc/planning/claude-code-bash-blocking-timeout.md`,
"Chained leading sleeps",
and again as `m9` under "A prefix keeps sleep-first commands running");
the prefix in that section removes this;
new code,
tests and a decision amendment;
the native handoff measured about 2 s anyway,
so the nominal one second makes no practical difference.

### Agent rule only

An `AGENTS.md` rule telling main agents never to pass `timeout`.

Pros:
no harness or hook change.

Cons:
nothing enforces it,
while the Bash tool schema itself advertises a day-long maximum;
1428 of 4898 recent calls carried a timeout.

### Rejected while designing

- Lowering `BASH_MAX_TIMEOUT_MS` and `BASH_DEFAULT_TIMEOUT_MS`:
  documented,
  but the source reads them for every Bash call,
  subagents included,
  which breaks the recorded requirement "Must not intervene with Claude Code native subagents";
  the env-vars page also ties a ceiling above 2 hours to the background time limit in unattended sessions.
- Forcing `run_in_background: true` on caller-timeout calls:
  quick commands would lose their synchronous result,
  the behavior the 2026-11-25 decision replaced.

### A prefix keeps sleep-first commands running

The owner asked on 2026-11-27T12:26:54-05:00,
as a note on the first question:
"Surely hook clamp can leave them running in background?"
It can.
The [tools reference][tools-reference-foreground-move] says a foreground command that reaches its timeout moves
to the background "unless the command starts with `sleep`",
and the filter wrapper already avoids that by starting the command with `set -o pipefail &&`.
A hook that also starts each unwrapped `sleep`-first command with a no-op such as `true &&`
gets the same treatment.

Probe on Claude Code 2.1.332 at about 12:27 -05:00,
one disposable session without the native cap,
the live hook passing both commands through unwrapped because of their redirects,
caller `timeout: 1000` standing in for the clamp:

- `m8`,
  `true && sleep 5 && printf m8-done > m8.out`:
  moved at 2133 ms with "Command did not complete within its 1s timeout";
  `m8.out` later held `m8-done`.
- `m9`,
  `sleep 5 && printf m9-done > m9.out`:
  "Exit code 143 Command timed out after 1s" at 1082 ms;
  `m9.out` was never written.

A single sample;
the prefix relies on Claude Code checking only the first command of the text,
which a release could change,
so it needs the same re-probe after upgrades as the native cap.
If that check changed,
the prefixed command would be stopped at its timeout,
where the native cap's failure is a call left waiting in front.

### Ranking

Four combinations remain:

- Hook clamp with the sleep prefix:
  the hook lowers every main-session caller timeout above 1000 to 1000
  and starts each unwrapped `sleep`-first command with `true &&`.
- Native cap with the sleep prefix:
  the settings variable,
  plus the hook prefix for the `sleep`-first commands the cap leaves in front.
- Native cap alone.
- Agent rule.

Hook clamp with prefix > native cap with prefix > native cap alone > agent rule.
The clamp with prefix beats the cap with prefix because both move the same calls,
including `sleep`-first ones,
but the clamp rests on the documented hook input rewrite and the documented `sleep` rule,
while the cap rests on a variable no documentation names and a release can drop without notice.
The cap with prefix beats the cap alone because the prefix also moves filter-skipped `sleep`-first commands,
which the cap alone leaves waiting in front for their whole timeout.
The cap alone beats the agent rule because it is enforced,
while the rule depends on models that the tool schema invites to set long timeouts.
Building the hook code is a cost of the first two,
not a reason to rank them lower.

## Separate choices

### Bare sleeps

Under the cap,
a filter-wrapped bare `sleep N` moves to the background after 2 s.
Accepting that keeps the hook unchanged and matches "auto background whenever in front over one second".
The alternative is to stop wrapping bare numeric sleeps in the filter,
so Claude Code sees the leading `sleep` and leaves it waiting,
as the decision's sleep exemption intends.
Recommendation:
accept,
because no bare sleep was measured in the recent calls.
Under the hook clamp the same answer means the hook drops its bare-sleep exemption,
so a bare sleep receives `timeout: 1000` and moves like any other command.

### The hook's own default

Correction recorded 2026-11-27:
the pair first cited here (2067 ms and 2058 ms) compares the capped and uncapped arms,
both of which ran the hook's `timeout: 1000`;
no arm ran the cap without the hook's default.
The owner's "Keep both" answer was given on that mislabeled premise.
Under the hook clamp the owner then chose,
the hook's default stays in any case,
so the answer changes nothing.
Recommendation:
keep the hook's default,
because it is the documented fallback if a release stops reading the undocumented variable.
Under the hook clamp the default stays in any case.

### Owner answers

On 2026-11-27T12:26:54-05:00,
asked through the question tool with both choices framed as mattering only under the native cap:

- Bare sleeps:
  "Accept the move (Recommended)".
- The hook's own default:
  "Keep both (Recommended)".
- The first question (which fix) received no selection,
  only the note quoted under "A prefix keeps sleep-first commands running",
  and is asked again with the four combinations.

On 2026-11-27T12:29:58-05:00:

- Which fix:
  "Clamp + prefix (Recommended)".

On 2026-11-27T12:49:24-05:00,
after the gate review raised REPL commands and the mislabeled pair was disclosed:

- REPL commands:
  "Clamp REPL too (Recommended)".
- Merge timing:
  "Merge now (Recommended)",
  with the second-pass review's findings to follow as fixes.

## Changes once a choice is accepted

For the native cap:

- `~/.claude/settings.json` gains the variable in `env`.
- `doc/decision/claude-code-bash-blocking-timeout.md` and
  `package/claude-code-plugin/bash-output-filter/README.md`
  ("Eligibility and caller intent") say that a caller timeout no longer keeps a main-session call in front past 2 s.
- A troubleshooting record notes the variable,
  its 2000 ms floor and the re-probe after each Claude Code upgrade.
- Wait loops return a task ID after about 2 s,
  and the agent learns completion from the task notification,
  the pattern the "Verified workarounds" of
  `doc/troubleshooting/claude-code-subagent-background-wait-context-dump.md` already uses.
- The wait guidance in `AGENTS.md` (the rules behind `CLAUDE.md:17` and `AGENTS.md:682`)
  says that such a wait runs in the background and ends with its notification;
  otherwise agents can keep waits in front through the one shape the cap leaves alone,
  a filter-skipped command starting with `sleep`.

## Gate review findings

The Claude Sonnet review reviewed the native-cap version of this proposal,
before the owner chose the clamp.
Each finding,
weighed against the accepted design:

- REPL commands:
  the REPL's inner Bash calls reach the hook as main-session calls
  (this session's REPL probe moved at 2062 ms under the hook's default),
  so the clamp lowers `.timeout(ms)` there too,
  and a REPL script can no longer wait for a long command and act on its exit code in the same script.
  Put to the owner,
  see "Owner answers".
- A clamp with native background tasks disabled would stop commands at one second,
  because `kEo` skips its cap only when a move is impossible and the hook had no such check.
  Fixed:
  the hook now keeps a caller timeout and adds no prefix when `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1`.
  Bare mode sets no variable the hook can read,
  as the decision already records.
- The caller's timeout is not carried into the background:
  in a local terminal session a moved command has no limit,
  as before the change;
  in an unattended session it gets the 30-minute limit described under
  "Cost shared by every option that moves these calls".
- The mislabeled pair under "The hook's own default" is corrected there.
- The cap's other overrides (the opt-out variable,
  PowerShell,
  a user-global setting,
  silent failure after an upgrade) applied only to the native cap,
  which was not chosen.
- The concentration in one session and the cost to short calls are recorded under "How often callers pass a timeout".
- The REPL,
  `!` commands typed by the user,
  a background subagent and an interactive session were not probed under the cap;
  the verification of the clamp covers a background subagent
  (its `Agent` call launched asynchronously).

## Implementation and verification

Branch `feat/claude-code-plugin-source-bash-caller-timeout-clamp-and-sleep-prefix`,
in the worktree `.claude/worktrees/claude-code-bash-caller-timeout-clamp-and-sleep-prefix`.
`package/claude-code-plugin/source/src/handler/bash-output-filter/blocking-timeout.ts`
holds `isTimeoutPolicyCall`,
`canMoveToBackground`,
`clampedTimeout`,
`startsWithSleepWord` and `withSleepPrefix`;
`index.ts` beside it composes them with the output filter.
The prefix rule follows the probe of sleep shapes recorded in
`doc/troubleshooting/claude-code-bash-timeout-moves-or-stops-a-command.md`:
a comment line before `sleep` and `sleep 5;x` were stopped,
so the check skips whole comment lines and ends a word at shell operators.

Checks in the worktree:
`format:oxlint`,
`lint:types` and `lint:oxlint` for `claude-code-plugin-source`,
`lint:types` and `build` for `claude-code-plugin-bash-output-filter`,
and `test:unit` for `claude-code-plugin-source` with no `[FAIL]` line
and the new suites `isTimeoutPolicyCall`,
`canMoveToBackground`,
`clampedTimeout`,
`startsWithSleepWord` and `withSleepPrefix` present by name.

Consumer check at about 12:41 -05:00:
a disposable session on Claude Code 2.1.332 whose `PreToolUse` hook ran the worktree's rebuilt bundle,
with a `PostToolUse` logger recording the input Claude Code executed:

- A filtered 8-second `node` command with `timeout: 3480000`:
  executed with `timeout: 1000`,
  moved at 2145 ms.
- `sleep 5 && printf v2-done > v2.out` with `timeout: 3480000`:
  executed as `true && sleep 5 && ...` with `timeout: 1000`,
  moved at 2101 ms,
  `v2.out` written.
- The same shape without a timeout:
  prefixed,
  moved at 2100 ms,
  written;
  before the change this shape was stopped.
- A comment line,
  then `sleep 5 && printf v4-done > v4.out`,
  with `timeout: 3480000`:
  prefixed,
  moved at 2103 ms,
  written.
- Bare `sleep 5`:
  filter-wrapped with `timeout: 1000`,
  moved at 2097 ms.
- Explicit `run_in_background: true` with `timeout: 3480000`:
  executed unchanged,
  written.
- A background subagent's `sleep 5 && printf s1-done > s1.out` and a 9-second `node` command,
  each with `timeout: 600000`:
  executed unchanged,
  finished in front at 5241 ms and 9197 ms.

The disabled-background exception was added after this check;
its unit tests cover it,
and it was not run in a live session.

## Open

- A Claude Sonnet second pass on the branch (same vendor,
  not independent) started at 2026-11-27T12:48 -05:00;
  its findings are to be weighed and fixed after the merge.
- Merged as `88632f16aa` at 2026-11-27T12:49:47-05:00.
  Live check at 12:50 -05:00,
  in the interactive code-mode session that wrote this record:
  a REPL command running an 8-second `node` script with `.timeout(600000)` moved at 2058 ms
  and returned a task ID,
  so REPL commands are clamped as the owner chose.
- The Claude Sonnet gate review's findings.
- The REPL surface under the cap stays unprobed until an interactive code-mode session runs with the variable set.
- A background subagent under the cap was not probed;
  the source reads the variable only for the main agent.

[tools-reference-background-limit]: https://code.claude.com/docs/en/tools-reference#time-limit-for-background-commands
[tools-reference-foreground-move]: https://code.claude.com/docs/en/tools-reference#foreground-commands-that-move-to-the-background
