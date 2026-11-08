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
pi --mode rpc --no-session --provider synthetic --model 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017' \
  --thinking xhigh --tools read,bash
```

`--no-session` keeps the run from writing a session file.
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
with the `synthetic` model at `--thinking low`.
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
  The source awaits `waitForIdle()` inside `abort()`,
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
`SessionManager.open(sessionPath, ...)` and returns `{ cancelled: false }` after replacing the runtime,
so the process continues in a fresh session bound to that path.
No file appeared at that path,
and no file appeared in Pi's real session store (`~/.pi/agent/sessions`) during the sweep.
Do not treat a `success: true` from `switch_session` as proof that the session existed.

### Extension records on stdout

Extensions also write `extension_ui_request` records to stdout,
for widgets,
titles,
status entries,
and notices.
Those that were seen need no reply.
A dialog-style request would,
and none appeared in the run.

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
splits at the last colon (`lastIndexOf(":")`,
lines 121 to 126 and 207 to 213)
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

### The readiness check disagreed with what worked

`pi auth check --provider <p> --model <m> --json --no-refresh` prints `status` and `provider`
without a credential.
The code is `packages/coding-agent/src/cli/auth-check.ts` lines 22 to 53.
It builds a model runtime over an in-memory models store with network access off (lines 66 to 73),
returns `invalid` with reason `invalid_state` if that runtime has any error or `checkAuth` throws,
and returns `not_ready` with reason `credentials_not_configured` if `checkAuth` finds nothing.
It does not load extensions.

In this session it did not agree with what worked:

- For `synthetic` it printed `invalid`,
  reason `invalid_state`,
  and the model then worked in a real run.
  The code returns that reason for any runtime error or thrown `checkAuth`,
  and the message of the error was not captured,
  so the specific cause is undetermined.
- For `openrouter` it printed `ready` on two runs
  and `not_ready`,
  reason `credentials_not_configured`,
  on every later run.
  A real prompt on `openrouter` failed with `No API key found for openrouter`.
  In the source the only inputs for that provider are a stored credential
  or the `OPENROUTER_API_KEY` environment variable (`packages/ai/src/providers/openrouter.ts` line 68).
  `auth.json` held no `openrouter` entry,
  and the variable was unset in the three later shell calls checked.
  So the early `ready` results can only have come from the variable being present in that call's environment,
  which is an inference from the code,
  since it was not observed.
  The systemd unit,
  which passes only `PATH` and `HOME`,
  had no such variable and failed.
- For `opencode-go` it printed `ready` on every run.

Treat it as a hint,
and trust a prompt that gets a reply.
The command was run without `--credentials`,
and no credential was opened or printed in this work.

### Rejected prompts come back as a response

A prompt that cannot start returns a `response` with an error text instead of events.
For a missing key,
the text names the provider and points to Pi's `docs/providers.md` and `docs/models.md`.
A client that only waits for `agent_settled` will wait forever,
so give it a deadline and an exit handler.

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

## Not verified

- `compact`,
  which needs a model call.
- `fork` with a valid entry id,
  `export_html` with a persisted session,
  and `switch_session` to a real session file.
  All three need a persisted session,
  and the runs used `--no-session`.
- `cycle_model`,
  and `set_model` to a different model than the one in use.
- Extension dialog requests that need a reply.
- Image prompts,
  `codemode`,
  and MCP tools.
- The cause of `invalid_state` for the `synthetic` readiness check,
  because the runtime error text was not captured.
- Whether `OPENROUTER_API_KEY` was really present in the shell when the two early `ready` results came back.
  The source and the later runs point that way,
  and it was not observed.
