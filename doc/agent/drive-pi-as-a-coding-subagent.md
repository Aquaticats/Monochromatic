# Driving Pi as a coding subagent and watching it work

Purpose:
record how to run Pi (DeepSeek V4.1 Flash) as an implementer that edits files in a worktree,
how to watch it live,
how to steer it between turns,
and the quirks seen while doing so.
It builds on `doc/agent/drive-pi-as-a-subagent.md`,
which covers the RPC protocol itself.
The runs were made on 2026-11-07 against Pi 1.5.0 with `synthetic` / `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`.
The task they served is in `doc/handover/pi-goal-reviewer-human-input-rubric.md`.

## Setup that worked

### A worktree with dependencies

`git worktree add -b <branch> .claude/worktrees/<name> HEAD` in this repository gives a worktree that already has
`node_modules` and the git-ignored `dist` folders,
because the git-policy tooling copies ignored files in
(`package/git-policy/cli/README.md`,
the paragraph on what happens after `git worktree add` has filled a worktree).
`mise run //package/pi-plugin/goal:lint:types` and `:test:unit` ran in it with no install step.

Piping that command into `tail` made the shell call hang,
although the worktree and branch existed within seconds.
A long-lived git-policy process holds the pipe open.
Send its output to a file,
or check the worktree directly instead of waiting on the pipe.

### A baseline before the subagent edits anything

Run the type check and the unit tests in the worktree first.
Here the baseline caught a failure that predated the work:
`src/review-contract.ts` imported `TSchema` from `@earendil-works/pi-ai`,
and pi-ai 1.5.0 no longer exports it,
so `lint:types` failed with TS2305 in the main checkout too.
Without the baseline,
that failure would have been blamed on the subagent's first edit.

### An isolated Pi home

`PI_CODING_AGENT_DIR=<dir>` makes Pi use that directory instead of `~/.pi/agent`
(`packages/coding-agent/src/config.ts`,
`ENV_AGENT_DIR`).
A directory holding only `settings.json` and a copy of `auth.json` was enough:

- `settings.json` with `packages` listing the installed provider package by path,
  here `~/.pi/agent/npm/node_modules/@benvargas/pi-synthetic-provider`.
  `pi list` showed exactly that package.
- `pi --list-models DeepSeek-V4.1-Flash` then listed `synthetic hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`.
- The copy of `auth.json` needs mode 600 in a mode 700 directory,
  and must be deleted afterwards.

This drops every other extension of the owner's setup,
including `thinking-default`,
which overrides `--thinking`.
In the isolated home `--thinking xhigh` is therefore the level that applies.
The level was set on the command line and was not read back with `get_state` in these runs.

The `commands` list of the isolated home still contained many `skill:` entries.
Where they were loaded from was not determined,
so do not assume the isolated home carries no skills.

### Tools

`--tools` is an allowlist of tool names or patterns to enable,
built-in and extension tools alike
(`pi --help`:
"Comma-separated allowlist of tool names or patterns (*) to enable").
The implementer ran with `--tools read,edit,write,grep,find,ls`:
no shell,
so it cannot run tests or use git.
The lead ran the checks and committed.
That kept the model inside the worktree by construction,
not by instruction.

## The driver

`pi-rpc-agent.ts` starts one Pi process in RPC mode and records everything worth watching.
It differs from the client in `doc/agent/drive-pi-as-a-subagent.md` in four ways:

- It writes a compact feed of tool calls with their arguments,
  tool results,
  assistant text,
  and token cost.
- It reads steers from an inbox folder.
  A file dropped there is sent as `steer` while the agent runs and as a new `prompt` while it is idle.
- It answers any dialog request with `{ type: 'extension_ui_response', id, cancelled: true }` and records it,
  so a dialog cannot stall the session.
- It stays alive across tasks until an idle timeout,
  a stop file,
  a turn cap,
  or a hard wall time.

```ts
// pi-rpc-agent.ts: drive one Pi subagent through RPC, record everything it does live, accept steers from an inbox folder.
// usage: node pi-rpc-agent.ts <out-dir> <tag> <cwd> <prompt-file>
// env: AGENT_DIR (PI_CODING_AGENT_DIR for the child), PI_TOOLS, PI_THINKING, PI_PROVIDER, PI_MODEL, PI_EXTRA_ARGS (space separated),
//      WALL_MS (hard kill), IDLE_MS (exit after this long idle once settled with an empty inbox), TURN_CAP
// Feed: <out-dir>/feed<tag>.jsonl, one compact record per event worth watching.
// Steers: write any text file into <out-dir>/inbox<tag>/ ; it is sent as a steer while the agent runs, else as a new prompt.
// Stop: create <out-dir>/stop<tag> to end the session early.
import { spawn } from 'node:child_process';
import { appendFileSync, existsSync, mkdirSync, readdirSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [outDirectory, tag, workingDirectory, promptFile] = process.argv.slice(2);
if (outDirectory === undefined || tag === undefined || workingDirectory === undefined || promptFile === undefined) {
  throw new Error('usage: pi-rpc-agent.ts <out-dir> <tag> <cwd> <prompt-file>');
}
const inboxDirectory = join(outDirectory, `inbox${tag}`);
mkdirSync(inboxDirectory, { recursive: true });
const feedFile = join(outDirectory, `feed${tag}.jsonl`);
const summaryFile = join(outDirectory, `summary${tag}.json`);
const stopFile = join(outDirectory, `stop${tag}`);
const WALL_MS = Number(process.env.WALL_MS ?? 840_000);
const IDLE_MS = Number(process.env.IDLE_MS ?? 90_000);
const TURN_CAP = Number(process.env.TURN_CAP ?? 80);
const startedAt = Date.now();

const arguments_ = [
  '--mode', 'rpc', '--no-session',
  '--provider', process.env.PI_PROVIDER ?? 'synthetic',
  '--model', process.env.PI_MODEL ?? 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017',
  '--thinking', process.env.PI_THINKING ?? 'xhigh',
  '--tools', process.env.PI_TOOLS ?? 'read,edit,write,grep,find,ls',
  ...(process.env.PI_EXTRA_ARGS ? process.env.PI_EXTRA_ARGS.split(' ') : []),
];
const environment = { ...process.env, ...(process.env.AGENT_DIR ? { PI_CODING_AGENT_DIR: process.env.AGENT_DIR } : {}) };
const child = spawn('pi', arguments_, { cwd: workingDirectory, stdio: ['pipe', 'pipe', 'pipe'], env: environment });

const preview = (value: unknown, length: number): string => {
  const text = typeof value === 'string' ? value : (JSON.stringify(value) ?? '');
  return text.length > length ? `${text.slice(0, length)}...[${text.length}]` : text;
};
const note = (entry: Record<string, unknown>): void => {
  appendFileSync(feedFile, `${JSON.stringify({ t: Date.now() - startedAt, ...entry })}\n`);
};

let exited = false;
let running = false;
let settled = 0;
let turns = 0;
let lastActivity = Date.now();
let costTotal = 0;
let inputTokens = 0;
let outputTokens = 0;
const toolHistogram: Record<string, number> = {};
const toolErrors: Array<Record<string, unknown>> = [];
const dialogs: Array<Record<string, unknown>> = [];
const waiting = new Map<string, (record: Record<string, unknown>) => void>();
let lastAssistantText = '';

child.stderr.on('data', (chunk: Buffer) => appendFileSync(join(outDirectory, `stderr${tag}.log`), chunk));
child.on('close', (code, signal) => {
  exited = true;
  note({ kind: 'exit', code, signal });
  for (const resolve of waiting.values()) resolve({ success: false, error: 'pi exited' });
});

const write = (command: Record<string, unknown>): void => { if (!exited) child.stdin.write(`${JSON.stringify(command)}\n`); };
let counter = 0;
const send = (command: Record<string, unknown>, timeoutMs = 30_000): Promise<Record<string, unknown>> => {
  if (exited) return Promise.resolve({ success: false, error: 'pi exited' });
  const id = `a${++counter}`;
  return new Promise((resolve) => {
    const timer = setTimeout(() => resolve({ success: false, error: `no response in ${timeoutMs}ms` }), timeoutMs);
    waiting.set(id, (record) => { clearTimeout(timer); resolve(record); });
    write({ id, ...command });
  });
};

const handleEvent = (record: Record<string, unknown>): void => {
  const type = String(record.type);
  lastActivity = Date.now();
  if (type === 'agent_start') running = true;
  if (type === 'agent_end') running = false;
  if (type === 'agent_settled') { settled += 1; note({ kind: 'settled', n: settled }); }
  if (type === 'turn_start') { turns += 1; note({ kind: 'turn_start', n: turns }); }
  if (type === 'tool_execution_start') {
    const name = String(record.toolName);
    toolHistogram[name] = (toolHistogram[name] ?? 0) + 1;
    note({ kind: 'tool_start', tool: name, args: preview(record.args, 700) });
  } else if (type === 'tool_execution_end') {
    const isError = record.isError === true;
    const entry = { kind: 'tool_end', tool: record.toolName, isError, result: preview(record.result, 500) };
    note(entry);
    if (isError) toolErrors.push({ t: Date.now() - startedAt, tool: record.toolName, result: preview(record.result, 300) });
  } else if (type === 'message_end') {
    const message = (record.message ?? {}) as Record<string, unknown>;
    if (message.role === 'assistant') {
      const parts = Array.isArray(message.content) ? (message.content as Array<Record<string, unknown>>) : [];
      const text = parts.filter((part) => part.type === 'text').map((part) => String(part.text ?? '')).join('\n');
      const thinkingLength = parts.filter((part) => part.type === 'thinking').reduce((sum, part) => sum + String(part.thinking ?? '').length, 0);
      const usage = (message.usage ?? {}) as Record<string, unknown>;
      const cost = (usage.cost ?? {}) as Record<string, unknown>;
      costTotal += Number(cost.total ?? 0);
      inputTokens += Number(usage.input ?? 0);
      outputTokens += Number(usage.output ?? 0);
      if (text !== '') lastAssistantText = text;
      note({ kind: 'assistant', stop: message.stopReason, text: preview(text, 900), thinkingChars: thinkingLength, toolCalls: parts.filter((part) => part.type === 'toolCall').length, usage: { input: usage.input, output: usage.output, cost: cost.total }, error: message.errorMessage ?? null });
    }
  } else if (type === 'extension_ui_request') {
    const method = String(record.method);
    if (['select', 'confirm', 'input', 'editor'].includes(method)) {
      dialogs.push({ t: Date.now() - startedAt, method, title: preview(record.title, 160), message: preview(record.message, 200) });
      note({ kind: 'dialog', method, title: preview(record.title, 160) });
      write({ type: 'extension_ui_response', id: record.id, cancelled: true });
    }
  } else if (type === 'auto_retry_start' || type === 'compaction_start' || type === 'queue_update') {
    note({ kind: type, detail: preview(record, 300) });
  }
};

let buffered = Buffer.alloc(0);
child.stdout.on('data', (chunk: Buffer) => {
  buffered = Buffer.concat([buffered, chunk]);
  for (let newline = buffered.indexOf(0x0a); newline !== -1; newline = buffered.indexOf(0x0a)) {
    const line = buffered.subarray(0, newline).toString('utf8').replace(/\r$/, '');
    buffered = buffered.subarray(newline + 1);
    if (line.trim() === '') continue;
    let record: Record<string, unknown>;
    try { record = JSON.parse(line) as Record<string, unknown>; } catch { note({ kind: 'non-json', line: line.slice(0, 160) }); continue; }
    if (record.type === 'response' && typeof record.id === 'string') { waiting.get(record.id)?.(record); continue; }
    handleEvent(record);
  }
});

const hardKill = setTimeout(() => { note({ kind: 'hard-kill' }); child.kill('SIGTERM'); }, WALL_MS);

const state = await send({ type: 'get_state' });
const stateData = (state.data ?? {}) as Record<string, unknown>;
note({ kind: 'state', model: (stateData.model as Record<string, unknown> | undefined)?.id, thinkingLevel: stateData.thinkingLevel });
const commands = await send({ type: 'get_commands' });
const commandNames = (((commands.data ?? {}) as { commands?: Array<{ name?: string }> }).commands ?? []).map((command) => command.name);
note({ kind: 'commands', names: commandNames });

const firstPrompt = readFileSync(promptFile, 'utf8');
const firstResponse = await send({ type: 'prompt', message: firstPrompt });
note({ kind: 'prompt_sent', success: firstResponse.success, error: firstResponse.error ?? null, chars: firstPrompt.length });

let endReason = 'idle';
while (!exited) {
  if (existsSync(stopFile)) { endReason = 'stop-file'; break; }
  if (turns >= TURN_CAP) { endReason = 'turn-cap'; break; }
  const inboxFiles = readdirSync(inboxDirectory).filter((name) => !name.startsWith('.')).sort();
  for (const name of inboxFiles) {
    const path = join(inboxDirectory, name);
    const text = readFileSync(path, 'utf8');
    unlinkSync(path);
    const mode = running ? 'steer' : 'prompt';
    const response = await send({ type: mode, message: text });
    note({ kind: 'inbox', file: name, sentAs: mode, success: response.success, error: response.error ?? null, preview: preview(text, 300) });
  }
  if (!running && settled >= 1 && inboxFiles.length === 0 && Date.now() - lastActivity > IDLE_MS) { endReason = 'idle'; break; }
  await new Promise((resolve) => setTimeout(resolve, 300));
}

const stats = await send({ type: 'get_session_stats' }, 15_000);
if (running) await send({ type: 'abort' }, 15_000);
clearTimeout(hardKill);
const summary = {
  tag, endReason, totalMs: Date.now() - startedAt, turns, settled,
  tokens: { input: inputTokens, output: outputTokens, costTotal },
  toolHistogram, toolErrors, dialogs, sessionStats: stats.data ?? stats.error ?? null,
  lastAssistantText: preview(lastAssistantText, 3000),
};
writeFileSync(summaryFile, `${JSON.stringify(summary, null, 2)}\n`);
child.stdin.end();
setTimeout(() => { if (!exited) child.kill('SIGTERM'); }, 5_000);
```

Run it as its own capped user unit,
never in the shared command cgroup:

```sh
# run from anywhere; the child's cwd is the fourth argument
systemd-run --user --unit=pi-goal-impl --collect \
  --property=RuntimeMaxSec=2500 --property=MemoryMax=3G --property=CPUQuota=200% --property=TasksMax=300 --property=IOWeight=10 \
  --setenv=PATH="$PATH" --setenv=HOME="$HOME" --setenv=AGENT_DIR=<isolated home> \
  --setenv=WALL_MS=2400000 --setenv=IDLE_MS=1500000 --setenv=TURN_CAP=120 \
  node pi-rpc-agent.ts <out-dir> _impl <worktree> <prompt-file>
```

## Watching it live

Tail the feed through `jq` and let the Monitor tool turn each line into a notification:

```sh
tail -n 0 -f feed_impl.jsonl | jq --unbuffered --raw-output \
  'select(.kind=="tool_start" or .kind=="tool_end" or .kind=="assistant" or .kind=="settled" or .kind=="dialog") |
   "\(.t) \(.kind) \(.tool // "") \(.isError // "") \(.args // .result // .text // "" | tostring | .[0:220])"'
```

Use `tail -n 0 -f` when re-arming after an expiry.
`tail -n +1 -f` replays the whole feed.
Two monitors on the same feed each deliver every event,
so let the old one expire before judging the volume.

Record shapes the feed relies on:

- `tool_execution_start` carries `toolName` and `args`.
- `tool_execution_end` carries `isError` and `result.content[].text`.
  An `edit` result also carries `details.diff`,
  which shows the exact change as it happens.
- An assistant `message_end` carries `content` parts of type `text`,
  `thinking`,
  and `toolCall`,
  plus `usage` with a `cost.total`.

## Quirks seen

- The model passed a path as the `pattern` of `find` and got "No files found matching pattern" with `isError` false.
  `find` takes a glob pattern and a separate `path`.
  A soft miss and a success look alike in the feed unless the result text is read.
  The next `find` with a bad directory returned a real error from `fd` with `isError` true.
- After those misses the model listed directories one level at a time and reached the file.
  It did not stop or ask.
- The model issues independent calls together,
  such as a `read` and a `grep` in one assistant message.
- It verified more than a one-line import change needs:
  it confirmed the export in `typebox`,
  read pi-ai's generic `Tool` type,
  edited,
  re-read the file,
  and grepped for other uses.
  A small first task still took 34 seconds from prompt to settle.
- Gaps of 20 seconds or more between events occurred during thinking at `xhigh`.
  Silence in the feed is not a hang.
- A second task sent through the inbox while the agent was idle became a new prompt in the same session,
  and the model started on it by reading the files its context did not yet hold.
- No dialog request arrived in the runs so far.

Results of the larger task,
its cost,
and the correction loop are recorded below when the session ends.
