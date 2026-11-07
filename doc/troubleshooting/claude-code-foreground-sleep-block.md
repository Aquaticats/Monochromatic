# Claude Code 2.1.274 blocks a leading foreground `sleep` of 25 seconds or more in the Bash tool

The Bash tool rejects the call before the shell runs,
 with `errorCode:10` and a `Blocked:` message.
The block ships in Anthropic's signed binary,
 it is gated behind a server-side feature flag,
 and it inspects only the first command of the chain.

## Symptom

A Bash tool call whose first command is a bare `sleep` of 25 seconds or more never reaches the shell.
Two message variants exist,
 depending on whether anything follows the sleep.

Standalone form,
 from `sleep 25`:

```text
Blocked: standalone sleep 25. To wait for a condition, use Monitor with an until-loop
(e.g. `until <check>; do sleep 2; done`). To wait for a command you started, use
run_in_background: true. Do not chain shorter sleeps to work around this block.
```

Chained form,
 from `sleep 25 && echo reached`:

```text
Blocked: sleep 25 followed by: echo reached. To wait for a condition, use Monitor with an
until-loop (e.g. `until <check>; do sleep 2; done`). To wait for a command you started, use
run_in_background: true. Do not chain shorter sleeps to work around this block.
```

When the same flag is on,
 the Bash tool description in the system prompt also carries this sentence:

```text
Foreground `sleep` is blocked; use Monitor with an until-loop to wait on a condition.
```

That sentence states neither the 25 second threshold nor the first-command-only scope,
 so the rule reads as absolute while the enforcement is narrow.

## Root cause

### Provenance: the block is Anthropic's, not a local patch

The installed binary is byte-identical to Anthropic's published release,
 so no local tool (tweakcc or otherwise) contributed the text or the check.

```bash
sha256sum /home/user/.local/share/claude/versions/2.1.274
# 15e2d05148f801b5774032faad87e624ecd172e9903288bda448b892eb58fa07

dist=https://storage.googleapis.com/claude-code-dist-86c565f3-f756-42ad-8dfa-d59b1c096819
curl -sS "${dist}/claude-code-releases/2.1.274/manifest.json"
# "linux-x64": { "checksum": "15e2d05148f801b5774032faad87e624ecd172e9903288bda448b892eb58fa07",
#                "size": 230580536 }
```

The local tweakcc state carries no matching customization:
 `~/.tweakcc/config.json` contains no occurrence of the sentence,
 and the `~/.tweakcc/system-prompts/tool-description-bash-sleep-*.md` segments hold the older
 Claude Code 2.1.53 wording ("If you must sleep,
keep the duration short (1-5 seconds)...")
 with no applied-hash entries.

### Source trace

Claude Code 2.1.274 is a Bun bytecode build,
 so there is no `path/to/file.js:LINE` to cite.
The reproducible substitute is a byte-level extraction from the binary named above,
 whose checksum is pinned in the provenance section.
Each excerpt below is the literal output of the command shown.

Detector and threshold:

```bash
rg -a -U -o -m2 '(?s).{300}standalone sleep .{300}' /home/user/.local/share/claude/versions/2.1.274 | tr -d '\000'
rg -a -U -o -m3 '(?s)Wsn=.{0,120}' /home/user/.local/share/claude/versions/2.1.274 | tr -d '\000'
```

```js
function Ops(e){
  let n=jf(e);
  if(n.length===0)return null;
  let r=n[0]?.trim()??"",s=/^sleep\s+(\d+(?:\.\d*)?)\s*$/.exec(r);
  if(!s)return null;
  let m=parseFloat(s[1]);
  if(m<Wsn)return null;
  let h=n.slice(1).join(" ").trim();
  return h?`sleep ${m} followed by: ${h}`:`standalone sleep ${m}`
}
// Wsn=25
```

Three properties follow directly from `Ops`:

- The threshold is 25 seconds,
   and the comparison is `m < Wsn`,
   so 25 blocks and 24 passes.
- Only `n[0]`,
   the first command of the parsed chain,
   is tested.
- The duration regex is `^sleep\s+(\d+(?:\.\d*)?)\s*$`,
   which accepts bare decimal seconds only,
   so any suffix form such as `sleep 1m` fails the match and is never blocked.

Two enforcement sites exist,
 one per shell surface,
 and both sit in the Bash tool's `validateInput`:

```bash
rg -a -U -o -m1 '(?s).{230}let h=Ops\(n\.command\).{60}' \
  /home/user/.local/share/claude/versions/2.1.274 | tr -d '\000'
rg -a -U -o -m2 '(?s).{320}Do not chain shorter sleeps to work around this block\..{40}' \
  /home/user/.local/share/claude/versions/2.1.274 | tr -d '\000'
```

Site one,
 the bash surface,
 which produced every message in "Verification":

```js
async validateInput(n,r){
  let s=r;
  if(SL()&&!cc()&&s.remoteCall?.constraints.background!=="forbidden"&&!n.run_in_background){
    let h=Ops(n.command);
    if(h!==null)return{result:!1,message:`Blocked: ${h}. To wait for a condition, use Monitor with
      an until-loop (e.g. \`until <check>; do sleep 2; done\`). To wait for a command you started,
      use run_in_background: true. Do not chain shorter sleeps to work around this
      block.`,errorCode:10}
  }
```

Site two,
 the PowerShell-capable surface,
 whose message adds "Monitor runs bash" and whose detector is `Ks`:

```js
if(SL()&&hi()&&!cc()&&!n.run_in_background){
  let a=Ks(n.command);
  if(a!==null)return{result:!1,message:`Blocked: ${a}. To wait for a condition, use Monitor with an
    until-loop (e.g. \`until <check>; do sleep 2; done\` — Monitor runs bash). To wait for a command
    you started, use run_in_background: true. Do not chain shorter sleeps to work around this
    block.`,errorCode:10}
}
```

`Ks` is the `Start-Sleep` twin of `Ops`,
 case-insensitive,
 splitting the chain on `[;|&\r\n]` instead of parsing it,
 and sharing the same `Wsn` threshold:

```js
function Ks(e){
  let n=e.trim().split(/[;|&\r\n]/)[0]?.trim()??"",
      r=/^(?:start-sleep|sleep)(?:\s+-s(?:econds)?)?\s+(\d+(?:\.\d*)?)\s*$/i.exec(n);
  if(!r)return null;
  let s=parseFloat(r[1]);
  if(s<Wsn)return null;
  let a=e.trim().slice(n.length).replace(/^[\s;|&]+/,"");
  return a?`Start-Sleep ${s} followed by: ${a}`:`standalone Start-Sleep ${s}`
}
```

The gate helpers both sites call:

```js
function SL(){return I("tengu_amber_sentinel",!1)}
function hi(){if(M()!=="windows")return!0;return Ij()!==null}
function cc(){return wY().backgroundTasksDisabled||a.CLAUDE_CODE_DISABLE_BACKGROUND_TASKS}
```

So the bash block is live only when all of these hold:

- `SL()`,
   the `tengu_amber_sentinel` feature flag,
   is on for the session.
   Its local default is false,
   which makes this a server-controlled rollout rather than a fixed property of the release.
- `!cc()`,
   so background tasks are enabled for the session.
   Either `wY().backgroundTasksDisabled` or `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` turns the guard off.
- The remote call does not already forbid background work,
   that is `s.remoteCall?.constraints.background!=="forbidden"`.
- The call did not already set `run_in_background: true`.

Site two swaps the remote-call condition for `hi()`,
 which is true on every non-Windows host and on Windows hosts with bash or PowerShell present.

The same flag drives the prompt sentence:

```js
let ye="- `run_in_background` runs the command detached: it keeps running across turns and "
      +"re-invokes you when it exits. No `&` needed.";
if(SL())ye+=" Foreground `sleep` is blocked; use Monitor with an until-loop to wait on a condition.";
```

### An earlier reading of the gate was wrong

`CLAUDE_CODE_SLEEPY_SNOWFLAKE` and its flag `tengu_sleepy_snowflake` look like the sleep gate by name.
They are not.
The surrounding code shows the value is a kept-reminder delivery scope,
 accepting `"off"`,
 `"threads"` or `"all"`,
 and its warning text names reminders,
 not sleeps:

```js
var IRn="tengu_sleepy_snowflake";
function XWo(e){
  let n=zn.CLAUDE_CODE_SLEEPY_SNOWFLAKE;
  if(n!==void 0)return{scope:n,source:"env"};
  // ...
  t(`[reminders] ignoring client_data ${IRn} for ${e}: got ${...}; expected a map from model
     pattern to "off", "threads" or "all"`,{level:"warn"});
}
```

Setting that variable does not affect the sleep block.
The gate is `tengu_amber_sentinel`.

## Verification

Version under test:
 Claude Code 2.1.274,
 linux-x64,
 sha256 `15e2d05148f801b5774032faad87e624ecd172e9903288bda448b892eb58fa07`,
 commit `1efcc1361e649ab98800b43a7df307043397a9ba`,
 build date 2026-09-16.
Harness:
 issue each command as a Bash tool call in a session where the sentence is present in the
 Bash tool description,
 which is the observable proof that `SL()` is on.

Blocked,
 standalone variant:

```bash
sleep 25
# Blocked: standalone sleep 25. ...
```

Blocked,
 followed-by variant:

```bash
sleep 25 && echo reached
# Blocked: sleep 25 followed by: echo reached. ...
```

Allowed:

```bash
sleep 24 && echo 'allowed at 24'
# allowed at 24

echo first && sleep 25 && echo 'allowed after leading command'
# first
# allowed after leading command

sleep 13 && sleep 13 && echo 'chained shorter sleeps allowed'
# chained shorter sleeps allowed

sleep 0.5m && echo 'suffix form allowed'
# suffix form allowed
```

The last three are the scope of the check made visible:
 a leading non-sleep command,
 a chain of sub-threshold sleeps,
 and a suffixed duration each pass,
 even though the block's own message tells the caller not to chain shorter sleeps.

## Verified workarounds

- `run_in_background: true` on the Bash call.
   Tradeoff:
   none for a command that should run detached,
   since the gate skips the check for background calls outright;
   the turn continues and a notification arrives on exit.
- Monitor with an until-loop,
   as the message suggests,
   for example
   `until curl -sf http://localhost:3000 >/dev/null; do sleep 1; done`.
   Tradeoff:
   it needs a real condition to test.
   Pure elapsed time is not a condition,
   so a deliberate delay has to be expressed as a loop around a clock check.
- Keep the leading sleep under 25 seconds.
   Tradeoff:
   the wait itself is still foreground,
   so it still holds the turn,
   and it silently breaks if Anthropic lowers `Wsn`.
- `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` in the Claude Code process environment,
   which makes `cc()` true and skips the guard.
   Tradeoff:
   it is a bad trade.
   It removes `run_in_background`,
   which is the sanctioned escape hatch,
   to recover a blocking primitive,
   and it takes Ctrl+B backgrounding with it.

The first-command-only gap and the suffix gap are real and reproduced,
 but they are not listed as workarounds here.
The block's message explicitly forbids working around it,
 so using them knowingly contradicts an instruction the tool states at the moment of refusal.
They are recorded as scope facts so a future session does not mistake an accidental pass for
 an inconsistent gate.

## What does not work

- `tweakcc unpack` for source-level inspection of this build.
   It writes a 20,777 character stub because the build is `// @bun @bytecode`,
   with the JavaScript compiled rather than embedded as loadable source.
   String extraction from the binary is the working substitute,
   and it is what every excerpt in this document uses.
- Editing `~/.tweakcc/system-prompts/tool-description-bash-sleep-*.md`.
   Those segments carry Claude Code 2.1.53 text,
   none of it is applied to 2.1.274,
   and the block is a runtime permission check rather than prompt text,
   so changing prompt wording would not lift it.
- Repacking patched JavaScript with `tweakcc repack`.
   The bytecode build removes the JavaScript round-trip that path depends on.
- Ctrl+B as a remedy for this specific block.
   Ctrl+B backgrounds a foreground command that is already running,
   and the guard rejects the call before the shell starts,
   so there is nothing to background.
   Ctrl+B does lower the general cost of a long foreground command,
   which is an argument about whether the block should exist,
   not a way around it.
- Setting `CLAUDE_CODE_SLEEPY_SNOWFLAKE`.
   Name collision only;
   see the gate correction in "Root cause".

## Upstream filing decision

Out of scope per `.out-of-scope/claude-code-upstream-bugs.md`,
 which exempts Claude Code from upstream issue filing and from elaborate local workarounds,
 on the recorded ground that upstream is unresponsive to issue reports.
No GitHub issue or comment is drafted here,
 and no fileable draft is kept.

Two further reasons this would fail a filing check even without the exemption:

- It is not a defect.
   The threshold,
   the message,
   and the prompt sentence are all deliberate,
   and they behave as written.
   The complaint is that the rule is tuned tighter than it needs to be,
   which is product feedback rather than a bug report.
- It is flag-gated,
   so the behavior can change server-side without a release,
   and any filed report would describe a configuration that may not be live for the reader.

The durable record is this document.
