# Driving Pi as a subagent from another agent

Purpose:
record how an agent in another harness starts a `pi` session in RPC mode,
sends it a prompt,
steers it while it works,
and reads the reply back.
Everything here was run on 2026-11-07 against Pi 1.5.0
(`@earendil-works/pi-coding-agent`,
installed through this repository's pnpm store),
except where a line says it was not run.
The dsh equivalent is `doc/agent/drive-dsh-as-a-subagent.md`.

## Start RPC mode

```sh
# run from the directory that becomes the working folder
pi --mode rpc --provider synthetic --model 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017'
```

None of `--no-session`,
`--thinking`,
or `--tools` is required.
Pi saves the session by default and the owner keeps that on purpose,
so the run can be read back or resumed with `--session <path|id>`.
Most runs recorded in this doc also passed `--no-session` (no session file),
`--thinking xhigh`,
and `--tools read,bash`
to keep each experiment small and repeatable;
the client script below passes them for that reason.
Drop them for real work.
Pi reads JSONL commands from stdin and writes JSONL records to stdout,
and it exits when stdin closes.
Keep extensions loaded:
some providers exist only because an extension registers them (see "Traps found").

The `pi` on `PATH` in this repository is the pnpm shim at
`package/agent-harness-shared/advisor/node_modules/.bin/pi`.
The protocol reference is in the installed package,
`docs/rpc.md` and `docs/rpc-commands.md`,
under `~/.local/share/pnpm/store/v11/links/@earendil-works/pi-coding-agent/1.5.0/<hash>/node_modules/@earendil-works/pi-coding-agent/`.

## Framing

One JSON object per line,
terminated by LF.
Split stdout on LF only,
never with a reader that also splits on the Unicode line and paragraph separators,
because those are valid inside JSON strings.
Every command takes an optional string `id`,
and its `response` record repeats it.
Correlate by `id`,
not by arrival order.

In the run,
every stdout line parsed as JSON.
Pi's `[info]` log lines went to stderr.

## Send, steer, and read

Commands that were run:

- `{"id":"c1","type":"get_state"}` returns the active model under `data.model`,
  with its provider,
  context window,
  output limit,
  and cost fields.
- `{"id":"c2","type":"prompt","message":"..."}` returned `success: true` with `data.disposition: "started"`.
  A successful response means the prompt was accepted,
  not that the model finished.
- `{"id":"c3","type":"prompt","message":"...","streamingBehavior":"steer"}` while the run was active
  returned `data.disposition: "queued"`.
  Without `streamingBehavior`,
  a prompt sent during a run is rejected (documented in `docs/rpc-commands.md`,
  not run).
- `{"type":"steer","message":"..."}` is a separate command that queues a steering message.
  It returned `data.disposition: "queued"` during a run.
- `{"type":"follow_up","message":"..."}` returned `data.disposition: "queued"` during a run.
  See "Follow-up,
  abort,
  and clear_queue" for what happened next.
- `{"type":"abort"}` and `{"type":"clear_queue"}` are covered in the same section.
- `{"id":"c4","type":"get_last_assistant_text"}` returns `data.text`.
- Closing stdin ends the process.

Wait for the `agent_settled` event before reading the final text,
and install the stdout listener before sending the first prompt so a fast run cannot be missed.
`agent_end` marks only the end of one low-level run.

### What the steer looked like

In the event stream of one run that took a prompt and then a steer,
and that lasted four turns:

- The steer prompt produced a `queue_update` record when it was accepted during the first turn.
- A second `queue_update` came at the start of the second turn,
  when the queue drained,
  so the instruction was delivered after the first turn's tool calls finished and before the next model call.
  That matches the documented delivery point.
- The final reply listed both fixture files and ended with `PINEAPPLE`,
  the word the steer asked for.
- The stream held `agent_start` once,
  `turn_start` and `turn_end` four times each,
  three tool executions,
  seventy-two `message_update` records,
  and `agent_settled` once.

### Follow-up, abort, and clear_queue

Each ran in its own Pi process on 2026-11-07,
with the `synthetic` model started with `--thinking low`
(the effective level is inferred to have been `xhigh`,
see "The thinking-default plugin overrides --thinking").
The prompt asked for `sleep 20` through `bash`,
then two file reads,
as separate tool calls,
and the command under test was sent once the first `tool_execution_start` arrived,
so the run was known to be active.
The source behind them is `packages/coding-agent/src/modes/rpc/rpc-mode.ts` lines 437 to 454 and
`packages/coding-agent/src/core/agent-session.ts` lines 2363 to 2385,
2569 to 2577,
and 2601 to 2611,
at commit `6e2222a8dda06e3d69fad46d630f152f5873567a` of `earendil-works/pi` (package version 1.5.0).

- `follow_up`:
  the command returned `queued`,
  and `get_state` during the run showed `pendingMessageCount: 1` with `isStreaming: true`.
  The follow-up user message appeared after the assistant's first complete reply,
  one more turn started,
  and that turn's reply was `FOLLOWED`.
  The source describes it as delivered only when the agent has no more tool calls or steering messages.
- `clear_queue`:
  a `steer` and a `follow_up` were queued during the run,
  and `get_state` showed `pendingMessageCount: 2`.
  `clear_queue` returned both texts as `{"steering":[...],"followUp":[...]}`,
  and the count dropped to 0 while `isStreaming` stayed true.
  The run finished without either word in its reply.
- `abort`:
  the response came back with `success: true` 29 ms after it was sent,
  `get_state` then showed `isStreaming: false`,
  and `get_messages` ended with a tool result of `Command aborted`
  and an assistant message with `stopReason: "error"`.
  The source awaits `waitForIdle()` inside [`abort()`][abort-source],
  so the response is sent after the agent is idle,
  and `agent_settled` can arrive before the response does.
  A client that starts waiting for `agent_settled` after the `abort` response returns will wait forever.
  The first run of this scenario did exactly that,
  and registering the waiter before sending `abort` fixed it.

### Commands that need no model call

One idle Pi process (`--no-session`,
the `synthetic` model) took these commands in sequence on 2026-11-07,
with no prompt sent,
and each returned `success: true` unless a line says otherwise.
The sweep took under two seconds.

- Reading state:
  `get_state`,
  `get_available_models` (99 models in the data),
  `get_available_thinking_levels` (5 levels),
  `get_commands` (15 commands),
  `get_messages`,
  `get_session_stats` (message,
  token,
  and cost counters,
  all 0 on an idle session),
  `get_entries` (an `entries` array and a `leafId`),
  `get_tree`,
  `get_fork_messages` (an empty array),
  and `get_last_assistant_text` (an empty object when there is no assistant message).
- Changing settings:
  `set_thinking_level` (null data),
  `cycle_thinking_level` (returns `{"level":"xhigh"}`),
  `set_model` (returns the model object,
  here the same model),
  `set_steering_mode`,
  `set_follow_up_mode`,
  `set_auto_compaction`,
  `set_auto_retry`,
  and `set_session_name`.
- `bash` with `{"command":"echo pi-bash-ok","excludeFromContext":true}` returned
  `{"output":"pi-bash-ok\n","exitCode":0,"cancelled":false,"truncated":false}`.
- `abort_bash` and `abort_retry` with nothing active returned `success: true` with null data.
- `clone`,
  `switch_session`,
  and `new_session` each returned `{"cancelled":false}`.

Two failed,
and for understandable reasons:

- `fork` with an unknown `entryId` returned `Invalid entry ID for forking`.
- `export_html` returned `Cannot export in-memory session to HTML`,
  because the process ran with `--no-session`.

One result needs care.
`switch_session` with a path to a file that does not exist reported success.
The source (`packages/coding-agent/src/core/agent-session-runtime.ts` lines 196 to 224) opens the path with
`SessionManager.open(sessionPath, ...)` and returns `{ cancelled: false }` after replacing the runtime.
`SessionManager.open` (`packages/coding-agent/src/core/session-manager.ts` lines 1777 to 1797)
reads a header only when the file exists,
takes the working directory from the process otherwise,
and builds a manager bound to that resolved path.
So the process continues in a fresh session for that path.
No file appeared at that path,
and no file appeared in Pi's real session store (`~/.pi/agent/sessions`) during the sweep.
Do not treat a `success: true` from `switch_session` as proof that the session existed.
The next section switches to a path that does exist.

### Session commands on a persisted session

One Pi process ran with `--session-dir <scratch folder>` and no `--no-session`,
the `synthetic` model,
`--thinking low`,
and a prompt of `Reply with exactly: OK`,
which makes a real user message.
The whole run took 35 seconds.

- After the prompt,
  `get_state` showed `messageCount: 3` and a `sessionFile` path inside the scratch folder,
  and a session file of 4694 bytes was on disk.
- `get_fork_messages` returned `{"messages":[{"entryId":"8916c48f","text":"Reply with exactly: OK"}]}`.
  Those ids are what `fork` accepts.
- `export_html` with an `outputPath` returned `{"path":"..."}` and wrote a 294687-byte file.
- `compact` failed with `Nothing to compact (session too small)`.
  So compaction itself was not exercised.
- `fork` with that `entryId` returned `{"text":"Reply with exactly: OK","cancelled":false}`.
  `get_state` afterwards showed a new `sessionFile` and `messageCount: 1`.
  At the end,
  that forked file was not among the two files on disk,
  which fits Pi writing a session file lazily.
  That reading was not checked in the source.
- `switch_session` to the original file returned `{"cancelled":false}`,
  and `get_state` then showed the original `sessionFile` and `messageCount: 3`.
- `clone` showed a new `sessionFile` with `messageCount: 3`,
  and that file (5426 bytes) was on disk.
- `cycle_model` moved the model to `synthetic` / `hf:openai/gpt-oss-120b`,
  one of the owner's scoped models.
  It chooses the next model for you,
  so read `get_state` afterwards.
- `set_model` with `{"provider":"opencode-go","modelId":"deepseek-v4.1-flash"}` returned that model object,
  and `get_state` showed the `opencode-go` provider.

### Extension records on stdout

Extensions also write `extension_ui_request` records to stdout,
for widgets,
titles,
status entries,
and notices.
In the first full run,
18 of 131 records were these,
about 14 percent,
and 7 of them arrived back to back before `agent_start`.
Examples were `setWidget` (`processes-dock`,
`processes-status`),
`setTitle`,
`setStatus` (`pi-statusline.usage`,
`hyper`,
`goal`),
and a `notify` warning that MCP tools are only reachable from the codemode or tool_search tool.
A client has to recognize and skip them.
Those that were seen need no reply.
A dialog-style request does,
and none appeared in any run.

### Dialog requests block until answered

`docs/rpc-extension-ui.md` splits the methods in two.
The dialog methods `select`,
`confirm`,
`input`,
and `editor`
"block until the client sends back an `extension_ui_response`" with the matching `id`.
The fire-and-forget methods `notify`,
`setStatus`,
`setWidget`,
`setTitle`,
and `set_editor_text`
expect no response.
If a dialog carries a `timeout` field,
the agent side auto-resolves with a default value when it expires.

The source agrees (`packages/coding-agent/src/modes/rpc/rpc-mode.ts`,
[`createDialogPromise`, lines 94 to 135][dialog-source]).
The awaiting extension code resumes on a matching response,
on its abort signal,
or on a timeout the extension chose.
With none of those,
the promise stays pending forever.
That blocks whatever extension code awaits it,
and a tool call,
command,
or run that depends on that code waits too.
No run here produced a dialog request,
so the stall itself was not observed.

A client should answer every `extension_ui_request` whose `method` is a dialog method
with an `extension_ui_response` carrying the same `id`.

### Which of the owner's extensions can raise a dialog

Pi loaded 18 packages from `~/.pi/agent/settings.json`:
13 from this repository and 5 from npm
(`pi-btw`,
`pi-processes`,
`pi-synthetic-provider`,
`pi-hyper-provider`,
`pi-radius`).
A static search of their sources for calls to the UI context found these.

- Can block,
  and only when a command is sent:
  `@earendil-works/pi-radius` runs `ctx.ui.select("Select provider for <model>")`
  (`extensions/radius-routing.ts` lines 474 and 556)
  inside its `/radius` command when the action is `force` or `prefer` and no provider id is given.
  `@charmland/pi-hyper-provider` runs `ctx.ui.select("Hyper status settings", ...)` in a loop
  and then `ctx.ui.confirm("Save Hyper status settings?")`
  (`src/credits.ts` lines 324 and 355)
  inside its `/hyper-status` command (`src/index.ts` line 199).
  Neither fires by itself.
  Both need a `prompt` whose message starts with the command,
  and extension commands execute immediately even while a run is streaming.
- Cannot block through the protocol:
  `ui.custom()` returns `undefined` at once in RPC mode
  (`rpc-mode.ts` lines 232 to 235,
  "Custom UI not supported in RPC mode").
  `pi-btw`,
  `pi-processes`,
  and the `pi-synthetic-provider` commands only use `ui.custom`.
  The repository's `goal` plugin
  (tested under RPC in `doc/agent/pi-goal-extension-driven-through-rpc.md`)
  also uses it for a manual review dialog,
  and returns `review_unavailable` before reaching it when `context.mode !== 'tui'`
  (`package/pi-plugin/goal/src/review-unavailable.ts` line 20).
  The `ask-user-question` plugin works only in interactive TUI mode and answers RPC with an unavailable-tool error
  (its README,
  and `src/tool.ts` line 155).
- The other 11 packages:
  no dialog calls found.

The search looked for `select`,
`confirm`,
`input`,
`editor`,
and `custom` on the UI context.
It would miss a dialog created through a wrapper or an alias,
and code reached only through dependencies of dependencies.

### One session per process

A Pi process holds one active session at a time.
`fork`,
`clone`,
`switch_session`,
and `new_session` replace it:
`get_state` showed a different `sessionFile` after each,
and `switchSession` tears down the current runtime before applying a new one
(`packages/coding-agent/src/core/agent-session-runtime.ts` lines 196 to 224).
To run N agents in parallel,
start N processes,
each with its own model connection and extensions.

## Choosing a model

`pi --list-models` prints a table of provider,
model id,
context window,
output limit,
thinking,
and images.
`pi --list-models <search>` filters it.
Six rows matched `deepseek-v4.1-flash`:

- `opencode-go` / `deepseek-v4.1-flash`.
- `radius` / `deepseek-v4.1-flash`.
- `openrouter` / `deepseek/deepseek-v4.1-flash`.
- `hyper` / `deepseek-v4.1-flash-1017`.
- `openrouter` / `deepseek/deepseek-v4.1-flash-1017`.
- `synthetic` / `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`.

The owner prefers the `-1017` variants,
and the run used the `synthetic` one.
Its `get_state` showed a context window of 524288,
an output limit of 65536,
and listed cost fields of 0.6 for input,
1.2 for output,
and 0.03 for cache reads,
with `supportsReasoningEffort: true` and an `xhigh` level mapped to `xhigh`.
The units of the cost fields were not checked.

## Traps found

### Extensions provide some providers

`--no-extensions` removes the `synthetic` provider:

```text
Error: Unknown provider "synthetic". Use --list-models to see available providers/models.
```

The process exited with code 1 at startup.
Start with extensions on when the model comes from one.
A first run that passed `--no-extensions` failed this way,
and its client then waited forever for a reply,
because it had no exit handler.

### A colon in a model id is fine, a bad suffix is not

Synthetic ids start with `hf:`,
and Pi's `--model` syntax also uses `:<thinking>` as a suffix.
The first revision of this doc,
commit `111426a476`,
called the colon a trap.
That was wrong.
The `:xhigh` failure it was based on came from `--no-extensions` being on in the same run,
which makes the `synthetic` provider unknown.

The resolver,
in `packages/coding-agent/src/core/model-resolver.ts`,
tries the whole text as a model reference first,
then,
if that fails,
splits at the last colon
([`lastIndexOf(":")`, lines 121 to 126][resolver-source],
and lines 207 to 213)
and accepts the suffix only when it is a valid thinking level.
Startup was tested without a prompt,
so no model call was made,
with extensions on:

- `--provider synthetic --model hf:deepseek-ai/DeepSeek-V4.1-Flash-1017` started (exit 0).
- `--model synthetic/hf:deepseek-ai/DeepSeek-V4.1-Flash-1017` started (exit 0).
- `--model synthetic/hf:deepseek-ai/DeepSeek-V4.1-Flash-1017:xhigh` started (exit 0).
- `--model synthetic/hf:deepseek-ai/DeepSeek-V4.1-Flash-1017:bogus` also exited 0,
  with a warning instead of an error:
  `Model "hf:deepseek-ai/DeepSeek-V4.1-Flash-1017:bogus" not found for provider "synthetic". Using custom model id.`

The last case is the trap:
a mistyped suffix does not fail,
it becomes a custom model id sent to the provider.
Check `get_state` `data.model.id` after startup,
or pass the thinking level with `--thinking` and leave the suffix off.

The `Invalid thinking level "nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4"` warning on every start
comes from a configured pattern,
`synthetic/hf:nvidia/...`,
that matches no model in the catalog.
The fallback branch at lines 216 to 226 then splits at the last colon,
finds a suffix that is not a thinking level,
and warns.
It is noise from that pattern and unrelated to the model in use.

### The thinking-default plugin overrides --thinking

The owner's `thinking-default` plugin (`package/pi-plugin/thinking-default`) calls `pi.setThinkingLevel(...)`
on `session_start` and on every model change
(`src/index.ts` lines 101 to 142;
its README,
"Behavior" and "Level selection").
It picks `xhigh` when the model supports it and `high` otherwise,
whatever the command line says.
Tested with `get_state` after startup and `--thinking low`:

- `synthetic` / `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`,
  extensions on:
  `xhigh`.
- `openrouter` / `deepseek/deepseek-v4.1-flash`,
  extensions on:
  `high`.
- The same `openrouter` model with `--no-extensions`:
  `low`.

The last two differ only in whether extensions load,
so the plugin is what changed the level.
The scenario and persisted-session runs in this doc were started with `--thinking low`.
`get_state` was not read in those runs,
so their effective level is inferred to have been `xhigh` from this test.
The owner finds DeepSeek V4.1 Flash works poorly below `xhigh`,
so leave the level to the plugin:
pass no `--thinking` and send no `set_thinking_level`.
The README says a manual change stays in effect until the next session start or model selection.
That was not tested.

### Project trust is silent in RPC mode

A folder with a project `.pi/settings.json`
(or `mcp.json`,
`extensions`,
`skills`,
`prompts`,
`themes`,
`SYSTEM.md`,
`APPEND_SYSTEM.md`,
or an `.agents/skills` folder in it or an ancestor)
needs a trust decision.
In RPC mode Pi does not ask.
Its trust context returns `undefined` from `select` and `false` from `confirm` unless the mode is interactive
([`packages/coding-agent/src/cli/project-trust.ts` lines 11 to 30][trust-source]),
and `resolveProjectTrusted` treats an unanswered prompt as not trusted
(`packages/coding-agent/src/core/project-trust.ts` lines 9 to 18).
Tested in a throwaway folder whose `.pi/settings.json` set steering mode `one-at-a-time` and follow-up mode `all`,
against a baseline of `all` and `one-at-a-time` in a plain folder:

- No flag:
  baseline values,
  so the project settings were ignored.
  No dialog record.
- `--approve`:
  the project values.
  No dialog record.
- `--no-approve`:
  baseline values.
  No dialog record.

`~/.pi/agent/trust.json` kept its modification time,
so no decision was saved.
To use a folder's project-level settings or packages from a subagent,
pass `--approve`.
Without it they are ignored with no message in the stream.
Only a project `settings.json` was tested.
A first attempt that used `defaultThinkingLevel` showed no effect even with `--approve`,
because of the override in "The thinking-default plugin overrides --thinking",
and was discarded.

### The readiness check: right about two providers, one open discrepancy

`pi auth check --provider <p> --model <m> --json --no-refresh` prints `status` and `provider`
without a credential.
The code is [`packages/coding-agent/src/cli/auth-check.ts` lines 22 to 53][auth-check-source].
It builds a model runtime over an in-memory models store with network access off (lines 66 to 73),
returns `invalid` with reason `invalid_state` if that runtime has any error or `checkAuth` throws,
and returns `not_ready` with reason `credentials_not_configured` if `checkAuth` finds nothing.
It does not load extensions.
Credentials come from `~/.pi/agent/auth.json`
(`packages/coding-agent/src/core/auth-storage.ts`)
or a provider's environment variable.

Over one session its answers changed for two providers,
and the cause was a credential change,
not a fault in the check:

- `openrouter` printed `ready` (an API key) and `radius` printed `ready` (OAuth) in the first runs at 22:09:44 and 22:10:05.
  From 22:15:20 on,
  both printed `not_ready` with reason `credentials_not_configured`,
  and a real prompt on `openrouter` failed with `No API key found for openrouter`.
  `~/.pi/agent/auth.json` was rewritten at 22:10:22,
  between those two groups of runs,
  and holds no entry for either provider.
  The owner then said they may have removed both providers during the session.
  That fits the evidence,
  and the contents of the file before 22:10:22 were never read,
  so the exact edit is not shown.
- `opencode-go` printed `ready` on every run.

One discrepancy stays open:
`synthetic` printed `invalid`,
reason `invalid_state`,
and the model then worked in real runs.
The same `auth.json` read cleanly for `opencode-go` in the same period,
so a malformed credential file does not explain it
(`ReadOnlyAuthStorage` in `auth-storage.ts` lines 205 to 258 throws on any invalid entry).
The remaining paths are a runtime error or a thrown `checkAuth` for `synthetic` specifically,
and the error message was not captured.
Because the check loads no extensions and `synthetic` needs one in the real run,
that difference is the likeliest place to look next.
This is an untested guess.

Use the check as a hint for providers with stored credentials,
and trust a prompt that gets a reply.
The command was run without `--credentials`,
and no credential was opened or printed in this work.

### Rejected prompts come back as a response

A prompt that cannot start returns a `response` with an error text instead of events.
For a missing key,
the text names the provider and points to Pi's `docs/providers.md` and `docs/models.md`.
A client that only waits for `agent_settled` will wait forever,
so give it a deadline and an exit handler.

## Compared with driving dsh

Both were driven from this repository on 2026-11-07,
with one run per scenario and no spread measured,
so the timings in this section are single runs.
The dsh side is in `doc/agent/drive-dsh-as-a-subagent.md`.

Pi is the easier of the two to drive as a throwaway subagent:

- One process and stdin and stdout JSONL,
  with no token,
  cookie,
  or port.
- A documented protocol in the installed package.
  dsh's HTTP call shapes had to be read from source.
- A full control set that behaved as documented:
  `steer`,
  `follow_up`,
  `abort`,
  `clear_queue`,
  `fork`,
  `clone`,
  `switch_session`.
- Fast:
  the 27-command idle sweep took under two seconds and the persisted-session run 35 seconds,
  while dsh's SDK server took about 18 seconds to answer `initialize`.
- Nothing persists with `--no-session`,
  while a dsh session lands in the owner's real session list.

dsh's web profile wins when a long-lived server with durable,
resumable sessions and pageable history is wanted.
Its docs say one connection can run several sessions at once.
That was not tested here:
the dsh runs executed one session at a time,
and the Pi side runs one active session per process.

Part of Pi's advantage is that its protocol is documented,
and that would shrink the gap if dsh documented a stable HTTP API.

## Client script

```ts
// pi-rpc-client.ts: drive `pi --mode rpc` over JSONL on stdio.
// usage: node pi-rpc-client.ts <out-dir> <cwd> <provider> <model>
// Sends one prompt, then a second prompt with streamingBehavior "steer" while the first run is still active,
// waits for agent_settled, reads the last assistant text, and closes stdin for an orderly shutdown.
import { spawn } from 'node:child_process';
import { appendFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const [outDirectory, workingDirectory, provider, model] = process.argv.slice(2);
if (outDirectory === undefined || workingDirectory === undefined || provider === undefined || model === undefined) {
  throw new Error('usage: pi-rpc-client.ts <out-dir> <cwd> <provider> <model>');
}
mkdirSync(outDirectory, { recursive: true });
const recordLog = join(outDirectory, 'pi-records.jsonl');
const deadlineMilliseconds = 240_000;
const startedAt = Date.now();

// A provider of "-" means the model argument already carries "provider/id".
// Keep extensions on: some providers only exist because an extension registers them.
const child = spawn('pi', [
  '--mode', 'rpc', '--no-session',
  ...(provider === '-' ? [] : ['--provider', provider]), '--model', model,
  '--thinking', 'xhigh',
  '--tools', 'read,bash',
  '--no-skills', '--no-mcp', '--no-prompt-templates', '--no-themes', '--no-context-files',
], { cwd: workingDirectory, stdio: ['pipe', 'pipe', 'pipe'] });

const log = (direction: string, record: unknown): void => {
  appendFileSync(recordLog, `${JSON.stringify({ elapsedMs: Date.now() - startedAt, direction, record })}\n`);
};

const waiting = new Map<string, (record: Record<string, unknown>) => void>();
const eventTypes: string[] = [];
let settled: (() => void) | undefined;
const settledPromise = new Promise<void>((resolve) => { settled = resolve; });
let buffered = Buffer.alloc(0);

child.stderr.on('data', (chunk: Buffer) => appendFileSync(join(outDirectory, 'pi-stderr.log'), chunk));
// If Pi exits before agent_settled, release the wait so the summary is still written.
child.on('close', (code, signal) => {
  log('exit', { code, signal });
  settled?.();
});

// Strict JSONL framing: split on LF only, strip an optional CR, never on Unicode line separators.
child.stdout.on('data', (chunk: Buffer) => {
  buffered = Buffer.concat([buffered, chunk]);
  for (let newline = buffered.indexOf(0x0a); newline !== -1; newline = buffered.indexOf(0x0a)) {
    let line = buffered.subarray(0, newline).toString('utf8');
    buffered = buffered.subarray(newline + 1);
    if (line.endsWith('\r')) line = line.slice(0, -1);
    if (line.trim() === '') continue;
    let record: Record<string, unknown>;
    try {
      record = JSON.parse(line) as Record<string, unknown>;
    } catch (error) {
      log('in-not-json', { line: line.slice(0, 300), error: String(error).slice(0, 100) });
      continue;
    }
    log('in', record);
    if (record.type === 'response' && typeof record.id === 'string') waiting.get(record.id)?.(record);
    else if (typeof record.type === 'string') {
      eventTypes.push(record.type);
      if (record.type === 'agent_settled') settled?.();
    }
  }
});

let counter = 0;
const send = (command: Record<string, unknown>): Promise<Record<string, unknown>> => {
  const id = `c${++counter}`;
  const frame = { id, ...command };
  log('out', frame);
  return new Promise((resolve) => {
    waiting.set(id, resolve);
    child.stdin.write(`${JSON.stringify(frame)}\n`);
  });
};

const deadline = setTimeout(() => {
  log('deadline', { note: 'hard deadline reached, killing child' });
  child.kill('SIGTERM');
}, deadlineMilliseconds);

// The stdout listener is installed before any prompt is sent, so a fast completion cannot be missed.
const state = await send({ type: 'get_state' });
const first = await send({
  type: 'prompt',
  message: 'Use read-only tools: first list the files in the current directory, then read each file one at a time, and report each file\'s contents. Modify nothing.',
});
const steer = await send({
  type: 'prompt',
  message: 'Also end your final reply with the single word PINEAPPLE.',
  streamingBehavior: 'steer',
});
await settledPromise;
const last = await send({ type: 'get_last_assistant_text' });
child.stdin.end();
clearTimeout(deadline);

const countsByType: Record<string, number> = {};
for (const type of eventTypes) countsByType[type] = (countsByType[type] ?? 0) + 1;
const stateData = (state.data ?? {}) as Record<string, unknown>;
appendFileSync(join(outDirectory, 'pi-summary.json'), `${JSON.stringify({
  getState: { success: state.success, model: stateData.model ?? null },
  firstPrompt: { success: first.success, data: first.data, error: first.error },
  steerPrompt: { success: steer.success, data: steer.data, error: steer.error },
  lastAssistantText: (last.data as Record<string, unknown> | undefined)?.text ?? null,
  eventCounts: countsByType,
}, null, 2)}\n`);
```

The script has no handling for Pi exiting between `agent_settled` and the final `get_last_assistant_text` call.
That path was not exercised,
and the script is shown as it ran.

## Safety and cleanup

- The run used `--tools read,bash` in a throwaway folder with `--no-session`.
  `bash` can modify files,
  so a real task wants a disposable working directory.
- Extensions stay loaded,
  so the owner's extension set runs.
  That set includes a status line and process widgets,
  whose records appear in the stream.
- Run the client in a capped systemd user unit,
  stop it when done,
  and confirm no `pi --mode rpc` process or `pi-*` unit remains.
  The run did this.

## Scenario and sweep scripts

The scenarios and sweeps in "Follow-up,
abort,
and clear_queue" and the two sweep sections used the same plumbing as the client script:
spawn Pi,
split stdout on LF,
correlate responses by `id`,
and register an event waiter before sending the command that can trigger the event.
Each scenario ran in its own Pi process.

```ts
// Sequences used with the same client plumbing. nextEvent(type) resolves on the next record of that type, or when Pi exits.
const longTask = 'Use bash to run `sleep 20`, then read alpha.txt, then read beta.txt, and report both contents. Do each step as a separate tool call.';
const toolStarted = nextEvent('tool_execution_start'); // register first, then prompt

// follow-up: the follow_up arrives while a tool is running and is delivered after the agent stops
await send({ type: 'prompt', message: longTask });
await toolStarted;
await send({ type: 'follow_up', message: 'After you finish, reply with the single word FOLLOWED.' });
await send({ type: 'get_state' }); // pendingMessageCount 1, isStreaming true
await nextEvent('agent_settled');

// abort: register the settle waiter BEFORE sending abort, because abort() waits for idle and
// agent_settled can arrive before the abort response does
await send({ type: 'prompt', message: longTask });
await toolStarted;
const settledAfterAbort = nextEvent('agent_settled');
await send({ type: 'abort' });
await settledAfterAbort;

// clear-queue: both queued messages come back from clear_queue and never run
await send({ type: 'prompt', message: longTask });
await toolStarted;
await send({ type: 'steer', message: 'Also end your reply with the word STEERED.' });
await send({ type: 'follow_up', message: 'Then reply with the word FOLLOWED.' });
await send({ type: 'clear_queue' }); // data: { steering: [...], followUp: [...] }
```

The idle sweep sent this list to one `--no-session` process,
in this order,
waiting for each response:

```ts
const commands = [
  { type: 'get_state' }, { type: 'get_available_models' }, { type: 'get_available_thinking_levels' },
  { type: 'set_thinking_level', level: 'high' }, { type: 'cycle_thinking_level' },
  { type: 'set_model', provider, modelId: model },
  { type: 'set_steering_mode', mode: 'all' }, { type: 'set_follow_up_mode', mode: 'one-at-a-time' },
  { type: 'set_auto_compaction', enabled: false }, { type: 'set_auto_retry', enabled: false },
  { type: 'get_commands' }, { type: 'get_messages' }, { type: 'get_session_stats' },
  { type: 'get_entries' }, { type: 'get_tree' }, { type: 'get_last_assistant_text' },
  { type: 'set_session_name', name: 'pi-rpc-idle-trial' },
  { type: 'bash', command: 'echo pi-bash-ok', excludeFromContext: true },
  { type: 'abort_bash' }, { type: 'abort_retry' }, { type: 'get_fork_messages' },
  { type: 'fork', entryId: 'does-not-exist' }, { type: 'clone' },
  { type: 'export_html', outputPath: '<scratch>/idle-export.html' },
  { type: 'switch_session', sessionPath: '<scratch>/no-such-session.jsonl' },
  { type: 'new_session' }, { type: 'get_state' },
];
```

The persisted sweep started Pi with `--session-dir <scratch>` and no `--no-session`,
sent the prompt `Reply with exactly: OK`,
waited for `agent_settled`,
then sent `get_state`,
`get_fork_messages`,
`export_html`,
`compact` with `customInstructions`,
`fork` with the first `entryId`,
`switch_session` to the `sessionFile` from the first `get_state`,
`clone`,
`cycle_model`,
and `set_model`,
reading `get_state` after most steps.

## Not verified

- Compaction itself:
  `compact` on a one-prompt session returned `Nothing to compact (session too small)`,
  and a larger session was not built.
- `prompt` without `streamingBehavior` during a run is documented as an error and was not run.
- Whether Pi writes a forked session file lazily,
  which fits one forked file being absent from disk and was not checked in the source.
- A dialog request actually stalling a run.
  The behavior is documented and in the source,
  and no run produced a dialog request.
- Project trust for resources other than a project `settings.json`,
  such as project `extensions` or `.agents/skills`.
- Loading only a provider extension with `-e <path>` to avoid the extension noise,
  as an alternative to `--no-extensions`.
- Whether `set_thinking_level` survives until the next model change,
  as the `thinking-default` README says.
- Image prompts,
  `codemode`,
  and MCP tools.
- The cause of `invalid_state` for the `synthetic` readiness check,
  because the runtime error text was not captured.

[dialog-source]: https://github.com/earendil-works/pi/blob/6e2222a8dda06e3d69fad46d630f152f5873567a/packages/coding-agent/src/modes/rpc/rpc-mode.ts#L94-L135
[trust-source]: https://github.com/earendil-works/pi/blob/6e2222a8dda06e3d69fad46d630f152f5873567a/packages/coding-agent/src/cli/project-trust.ts#L11-L30
[resolver-source]: https://github.com/earendil-works/pi/blob/6e2222a8dda06e3d69fad46d630f152f5873567a/packages/coding-agent/src/core/model-resolver.ts#L121-L126
[abort-source]: https://github.com/earendil-works/pi/blob/6e2222a8dda06e3d69fad46d630f152f5873567a/packages/coding-agent/src/core/agent-session.ts#L2601-L2611
[auth-check-source]: https://github.com/earendil-works/pi/blob/6e2222a8dda06e3d69fad46d630f152f5873567a/packages/coding-agent/src/cli/auth-check.ts#L22-L53
