# Claude Code 2.1.332: a Bash command at its foreground timeout moves to the background unless its first word is `sleep`, which is stopped

## Symptom

A main-session Bash call that reaches its foreground timeout ends one of two ways.
Most commands move to the background and keep running:

```text
Command did not complete within its 1s timeout and was moved to the background (ID: b9qz5ezg2).
```

A command whose first command word is `sleep` is stopped,
and whatever was chained after the `sleep` never runs:

```text
Exit code 143
Command timed out after 1s
```

A separate effect of the same rule:
a call that passes its own long `timeout` stays in front until that timeout,
which the terminal shows as

```text
Running… (3m 36s · timeout 58m)
     (ctrl+b to run in background)
```

`doc/planning/claude-code-bash-caller-timeout-cap.md` measured how often that happened here
and records why this repository's hook now clamps such timeouts.

## Root cause

Claude Code is distributed as a native executable with its JavaScript bundled,
not as open source,
so the trace below reads the installed bundle,
`~/.local/share/claude/versions/2.1.332`,
with `rg --text --byte-offset --only-matching`.
Names are minified,
and each excerpt is cited by its byte offset in that file.

### The leading-word check

Byte 345971091 and byte 345978488:

```js
// claude 2.1.332 bundle
var dJi=["sleep"]
function bJi(e){let n=nb(e);if(n.length===0)return!0;let o=n[0]?.trim().split(/\s+/,1)[0];if(!o)return!0;return!dJi.includes(o)}
```

`bJi` splits the command into subcommands with `nb`,
takes the first word of the first one,
and answers `false` when that word is `sleep`.
`nb` was not located in the bundle under that name,
so its splitting rules are known only from the probes under "Verification":
it splits on `;` and `&&`,
it skips a whole comment line,
and it does not see inside `{ ...; }` or `( ... )`.

### Where the check decides the outcome

Byte 346004080,
in the Bash tool's call path:

```js
// claude 2.1.332 bundle
fs=(!Qn||So!==void 0)&&(bJi(ln)||ht===void 0&&So===void 0&&u_r()),Ss=!Qn,Or=kEo({requestedTimeoutMs:Jn,isMainAgent:Ge===!0,canAutoBackground:fs&&So===void 0,subagentCapMs:co?0:ze}),Zo=await $dt(ln,_e.signal,"bash",{timeout:Or,...,shouldAutoBackground:fs,...
```

`shouldAutoBackground` is `false` for a command `bJi` rejects
(the other branch needs the feature flag behind `u_r`,
`tengu_foamy_pond`,
which defaults to `false`),
so its timeout stops it.
The [tools reference][tools-reference-sleep] states the same rule:
"Claude Code moves it to the background instead of stopping it,
unless the command starts with `sleep`."

### The undocumented main-agent cap

Byte 336059910:

```js
// claude 2.1.332 bundle
var i=2000;function kEo({requestedTimeoutMs:n,isMainAgent:e,canAutoBackground:t,subagentCapMs:o=0,env:u=process.env}){if(!t)return n;let r=e?ug(u.CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS):o;if(isNaN(r)||r<=0)return n;return Math.min(n,Math.max(r,i))}
```

For the main agent,
`CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS` lowers the foreground timeout to at least 2000 ms,
whatever the call requested.
It is skipped for a command that cannot move (`if(!t)return n`),
so it never shortens a `sleep`-first command.
The [environment variable reference](https://code.claude.com/docs/en/env-vars),
read 2026-11-27,
does not list it.
`doc/troubleshooting/claude-code-subagent-background-wait-context-dump.md`
traced the same function in 2.1.316.

### The background time limit after a move

Byte 336059716 and byte 336074020:

```js
// claude 2.1.332 bundle
var PEl=1800000,JTs=7200000;
function QCt(){return Math.min(MW()?Math.max(Ut,OJ()):IEl(Ne(PEl,"tengu_rustling_gizmo",()=>zt("tengu_rustling_gizmo",0))),cOe())}
```

The [tools reference][tools-reference-limit] describes it:
in an unattended session,
"A command that starts in the foreground and then moves to the background at its timeout gets 30 minutes from the move",
10 minutes in a `-p` run whose prompt is passed as text,
while "A local session you work in from a terminal,
the desktop app,
or the VS Code extension has no time limit on background commands."

### A neighbouring check keyed on the same subcommand

Byte 345978774:

```js
// claude 2.1.332 bundle
function EJi(e){let n=TJi(e);if(n===null||n.seconds<Dws)return null;return n.rest?`sleep ${n.seconds} followed by: ${n.rest}`:`standalone sleep ${n.seconds}`}
```

It reads the same first subcommand,
so anything that hides a leading `sleep` from `bJi` hides it from this description too.

## Verification

Version:
`2.1.332 (Claude Code)` from `claude --version`.

Harness,
run 2026-11-27 between 12:14 and 12:41 -05:00:
a disposable `HOME` holding a copy of the login with its refresh token removed,
a `settings.json` whose `PreToolUse` Bash hooks are this repository's
`package/claude-code-plugin/bash-output-filter/bundle/node/index.mjs`
and a logger that appends each hook input to a file,
and one headless session per case list:

```sh
# run from an empty working folder, HOME pointing at the disposable home
claude -p "<numbered list of exact Bash tool JSON parameters, one call per message>" \
  --output-format stream-json --verbose --dangerously-skip-permissions --model sonnet
```

Timings are the gap between each `tool_use` and its `tool_result` in the session transcript;
each case ran once.
The disposable homes were deleted after each run.

### Stopped at the timeout

Each with `timeout: 1000` and a redirect,
so this repository's hook passes the text through unchanged:

- `sleep 5 && printf c4 > c4.out`:
  exit 143 at 1061 ms;
  `c4.out` never written.
- `sleep 5;printf c7 > c7.out`:
  exit 143 at 1064 ms.
- `# wait for the run` on one line,
  then `sleep 5 && printf c1 > c1.out`:
  exit 143 at 1121 ms.
- `sleep 5 && printf m9-done > m9.out`:
  exit 143 at 1082 ms.

### Moved to the background and finished there

Same settings:

- `X=1 sleep 5 && printf c2 > c2.out`:
  moved at 2062 ms;
  `c2.out` written.
- `{ sleep 5; printf c5; } > c5.out`:
  moved at 2063 ms;
  written.
- `(sleep 5; printf c6) > c6.out`:
  moved at 2055 ms;
  written.
- `true && sleep 5 && printf m8-done > m8.out`:
  moved at 2133 ms;
  written.

### Timeout zero

`node -e "setTimeout(function(){},8000)" > c3.out` with `timeout: 0`
stayed in front and finished at 8090 ms:
zero does not mean "move at once".

### The undocumented cap

With `CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS=1000` in the settings `env`
and an 8-second command carrying `timeout: 600000`,
the main session moved it at 2245 ms against 8204 ms without the variable,
a `--agent` main session at 2193 ms against 8240 ms,
and a native subagent finished in front in both arms (8166 ms and 8156 ms).
`sleep 5 && printf m2-done > m2.out` with `timeout: 600000` finished in front in both arms
(5084 ms and 5076 ms).
The full matrix is in `doc/planning/claude-code-bash-caller-timeout-cap.md`,
"Probe of the native cap".

## Verified workarounds

### A no-op first command

Sending `true && <command>` makes the first word `true`,
so the command moves at its timeout instead of being stopped (`m8` against `m9`).
This repository's hook applies it to main-session commands it does not wrap,
in `withSleepPrefix` in
`package/claude-code-plugin/source/src/handler/bash-output-filter/blocking-timeout.ts`,
and the hook's own filter wrapper (`set -o pipefail && ...`) has the same effect on the commands it wraps.
Tradeoffs:
the transcript shows the prefixed text;
Claude Code's sleep-specific descriptions (`EJi`) no longer see the `sleep`;
and the workaround depends on `bJi` reading only the first word,
so a release that read further would stop prefixed sleeps,
which is why the probes in "Verification" are re-run after each Claude Code upgrade.

### The main-agent cap variable

`CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS` moves every main-session command that can move after at least 2 s,
whatever timeout it passed.
Tradeoffs:
undocumented,
so a release can drop it silently;
floored at 2000 ms;
leaves `sleep`-first commands to their own timeout.
This repository does not set it;
the owner chose the hook clamp with the prefix on 2026-11-27
(`doc/decision/claude-code-bash-blocking-timeout.md`,
"Amendment of 2026-11-27").

## What does not work

- Lowering `BASH_MAX_TIMEOUT_MS` or `BASH_DEFAULT_TIMEOUT_MS` to cap main-session waits:
  the functions that read them (`cue` and `OJ`,
  beside `kEo` at byte 336059910) take no main-agent argument,
  so subagents would change too,
  and the background limits derive from the same values.
- The cap variable alone for `sleep`-first commands:
  `kEo` returns the requested timeout when the command cannot move.
- `timeout: 0` as "move at once":
  it kept the command in front for its full 8 s.

## Upstream filing decision

`.out-of-scope/claude-code-upstream-bugs.md` exempts Claude Code from upstream filing and tracking
("This project does not file or track Claude Code bugs as GitHub issues"),
so nothing is drafted or filed.
For the record:

1. Upstream's fault:
   the `sleep` rule is documented behavior,
   not a defect;
   the missing documentation of `CLAUDE_CODE_AUTO_BACKGROUND_TIMEOUT_MS` is a documentation gap.
2. Upstream can fix it:
   yes,
   the gap is documentation.
3. Supported use case:
   the timeout and background behavior are documented in the tools reference.
4. Contribution welcome:
   not assessed,
   because the exemption applies.
5. Likely to fix:
   not assessed,
   for the same reason.
6. Prototype:
   none,
   because there is no defect to fix.

[tools-reference-sleep]: https://code.claude.com/docs/en/tools-reference#:~:text=unless%20the%20command%20starts%20with
[tools-reference-limit]: https://code.claude.com/docs/en/tools-reference#:~:text=gets%2030%20minutes%20from%20the%20move
