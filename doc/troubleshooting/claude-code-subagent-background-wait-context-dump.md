# Claude Code 2.1.316: a background subagent's Bash calls move to the background after 180 s, whatever the timeout

## Symptom

A subagent that hands a long run (a mutation campaign,
a container gate) to the shell cannot wait for it in the
foreground,
and once it waits in the background it costs an abnormal amount of compute.
The owner's words,
2026-11-01:

- "Because of how a few Claude Code quirks work together,
  long-running mutation runs when handed to subagents would
  burn abnormally amount of compute,
  so they'd have to be done by the main agent."
- "The core Claude Code problem:
  subagent context would be auto dumped into main agent's context (expensive!)
  if the
  subagent doesn't have any foreground work.
  Foreground bash is limited to max 3 min by Claude Code and the tuner
  doesn't work."

The tool result a subagent sees,
as quoted in
[anthropics/claude-code#105928](https://github.com/anthropics/claude-code/issues/105928):
"Command was moved to the background (ID:
…) after running for 180s".

## Root cause

Traced in the installed executable,
`2.1.316 (Claude Code)`
(`/var/home/user/.local/share/claude/versions/2.1.316`,
a native ELF with its JavaScript bundled).
The excerpts below were extracted with `rg --text --only-matching` from that file;
names are minified.

### The timeout setting is applied, and is not the limit

`BASH_MAX_TIMEOUT_MS` was not renamed.
It still sets the ceiling a Bash call may ask for:

```js
// claude 2.1.316 bundle
var a=120000,s=600000;function mpe(n=process.env){let e=n.BASH_DEFAULT_TIMEOUT_MS;if(e){let t=Zu(e);if(!isNaN(t)&&t>0)return t}return a}function gpe(n=process.env){let e=n.BASH_MAX_TIMEOUT_MS;if(e){let t=Zu(e);if(!isNaN(t)&&t>0)return Math.max(t,mpe(n))}return Math.max(s,mpe(n))}
```

### A separate cap moves the call to the background

Before the call runs,
the requested timeout passes through an auto-background cap:

```js
// claude 2.1.316 bundle
var i=2000;function jnr({requestedTimeoutMs:n,isMainAgent:e,canAutoBackground:t,subagentCapMs:o=0,env:u=process.env}){if(!t)return n;let r=e?Zu(u.CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS):o;if(isNaN(r)||r<=0)return n;return Math.min(n,Math.max(r,i))}
```

For the main agent the cap is the undocumented `CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS`;
for a subagent it is the
`subagentCapMs` argument,
which no environment variable reaches.
A larger `timeout` or `BASH_MAX_TIMEOUT_MS` only raises `requestedTimeoutMs`,
and the `Math.min` keeps the cap.

### The subagent cap is a remote feature flag

The Bash tool passes `subagentCapMs:CWr(s)`:

```js
// claude 2.1.316 bundle
function CWr({agentId:e,agentContext:n,innerCall:r,hookCaller:s,pluginUnnamed:h}){if(r===!0||s!==void 0||h===!0||!Bk(n)||n.isMainSession===!0||n.isAsync!==!0||n.agentId!==e||CI()||M5o())return 0;let y=v("tengu_maple_ferry",0);return Number.isFinite(y)&&y>0?y:0}
```

The cap applies only to a subagent running in the background (`isAsync`),
and its value is the feature flag
`tengu_maple_ferry`.
This machine's cached flags in `~/.claude.json` hold `"tengu_maple_ferry": 180000`,
the three minutes observed.
`M5o` is `function M5o(){return Y8()||GMe()}`;
`CI` has several unrelated minified definitions in the bundle,
so which
one this call reaches was not established.

### Why waiting costs the main agent

Once the call is in the background,
the subagent has no foreground work and ends its turn.
The harness then notifies the main agent,
whose next turn re-reads its whole context.
This session's harness text on every subagent completion:
"A task-notification fires each time this agent stops with no live background children of its own."
Issue #105928 documents the repeat:
a background subagent that waits on its own background shell or Monitor wakes the
parent once per event,
each wake marked `completed` with the summary "is waiting on its own background work".

### What this session's main agent shows

Every Bash call in the coordinating session returned "Command running in background with ID:
...".
That is the owner's own tool,
which backgrounds every main-agent command and is set not to apply to subagents;
it is
not the Claude Code cap above.

## Verification

- Version:
  `2.1.316 (Claude Code)`;
  the bundle excerpts above come from
  `/var/home/user/.local/share/claude/versions/2.1.316`.
- Setting in force:
  `~/.claude/settings.json:13` sets `"BASH_MAX_TIMEOUT_MS": "86400000"`;
  `printenv` in the session
  shows `BASH_MAX_TIMEOUT_MS=86400000` and no `CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS`.
- Flag value:
  `"tengu_maple_ferry": 180000` in `~/.claude.json`.
- Documentation,
  read 2026-11-01 from <https://code.claude.com/docs/en/env-vars.md>:
  `BASH_MAX_TIMEOUT_MS` is "Maximum timeout the model can set for a foreground Bash or PowerShell tool command";
  `CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS` and the subagent cap are absent;
  `CLAUDE_SUBAGENT_BG_SHELL_MAX_MS` is "Removed in v2.1.260 and now a no-op",
  and it capped how long a subagent's
  background shell could run,
  not when a call is moved to the background.
- Not run here:
  a subagent `sleep 200` with `timeout: 600000` to watch the move at 180 s.
  Issue #105928 reports that
  run on 2.1.314.

## Verified workarounds

- The main agent starts every long run itself as its own systemd user unit
  (`systemd-run --user --unit=<name> --collect` with CPU,
  memory,
  task and disk-weight caps),
  waits on it with one
  background Bash command that ends at a terminal state,
  and reads the result when that command's notification
  arrives.
  Tradeoff:
  the main agent's own context carries the waiting and the result,
  and only one session follows the run.
- Where this repository applies it:
  mutation campaigns only,
  by the owner's decision of 2026-11-01 ("anything lower
  than 1h should be fine-ish ... Limit it to mutation runs only for now"):
  `package/mutation-test/cli-cargo-mutants/README.md` ("Who runs a campaign").

## Untested options, read from the source and the docs

- A subagent running in the foreground is exempt (`n.isAsync!==!0` returns 0).
  The docs say fork mode,
  on by default,
  stops Claude waiting for a subagent,
  and `CLAUDE_CODE_FORK_SUBAGENT=0` turns it
  off.
  Tradeoff:
  the main agent blocks while that subagent runs.
- `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` is documented to stop "moving subagents,
  long MCP tool calls,
  and timed-out
  shell commands to the background automatically",
  and it also removes `run_in_background` everywhere.
  Tradeoff:
  the main agent loses its background commands.

## What does not work

- Raising `BASH_MAX_TIMEOUT_MS` or passing a larger `timeout`:
  the subagent cap is applied with `Math.min` after them.
- Setting `CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS`:
  read only when `isMainAgent` is true.
- `CLAUDE_SUBAGENT_BG_SHELL_MAX_MS`:
  a no-op since v2.1.260,
  and it governed a different limit.
- Letting the subagent background the run and end its turn:
  that is what wakes the main agent.

## Upstream filing decision

Not filed.
`.out-of-scope/claude-code-upstream-bugs.md` exempts Claude Code defects from upstream filing and tracking
("This project does not file or track Claude Code bugs as GitHub issues");
this file is the canonical record instead.
The existing report [anthropics/claude-code#105928](https://github.com/anthropics/claude-code/issues/105928) (open,
read 2026-11-01) already carries the 180 s observation and the wake cost;
the source trace here would be additive,
but
the exemption covers comments too,
so nothing is drafted.
