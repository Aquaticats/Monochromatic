# Claude Code Bash caller-timeout cap proposal

Purpose:
explain why main-session Bash calls still wait in the foreground for minutes
under the one-second policy in `doc/decision/claude-code-bash-blocking-timeout.md`,
and propose how to stop it.
Status:
a proposal awaiting the owner's decision,
not accepted.
Recorded 2026-11-27T12:25:00-05:00 by a Claude Code session;
the gate review by Claude Sonnet was started at the same time and its verdict is not yet folded in.

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
The value `3480000` appears in neither the 2.1.332 executable nor this repository;
it is the models' own choice.

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
applies to every main-session Bash call,
REPL commands presumably included,
whatever timeout the caller passes;
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

### Hook clamps caller timeouts

The hook replaces a main-session caller timeout above 1000 with 1000.

Pros:
uses the documented hook input rewrite;
stays under this repository's tests.

Cons:
an unwrapped `sleep`-first chain with a caller timeout is then stopped at one second
(exit 143,
measured on 2026-11-25 in `doc/planning/claude-code-bash-blocking-timeout.md`,
"Chained leading sleeps")
unless the hook gains another exemption;
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

### Ranking

Native cap > hook clamp > agent rule.
The native cap beats the hook clamp because it leaves `sleep`-first commands running instead of stopping them
and needs no code,
while the clamp's one-second nominal value buys nothing over a measured 2-second handoff.
The hook clamp beats the agent rule because it is enforced,
while the rule depends on models that the tool schema invites to set long timeouts.

## Separate choices if the native cap is adopted

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

### The hook's own default

With the cap,
calls without a timeout moved at the same time with or without the hook's `timeout: 1000`
(2067 ms and 2058 ms).
Recommendation:
keep the hook's default,
because it is the documented fallback if a release stops reading the undocumented variable.

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

## Open

- The owner's choice among the options and the two separate choices.
- The Claude Sonnet gate review's findings.
- The REPL surface under the cap stays unprobed until an interactive code-mode session runs with the variable set.
