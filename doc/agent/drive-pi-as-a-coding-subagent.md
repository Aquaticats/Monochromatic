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

The `git worktree add` call itself returned only after minutes,
on both worktrees made this way,
although the directory,
the branch,
and the copied `node_modules` were usable within seconds.
Redirecting its output to a file did not change that,
so a held pipe is not the cause.
The same README paragraph says the wrapper starts each tool's offline repair in the new worktree after the copy
(`pnpm install --offline --frozen-lockfile`,
a Cargo build directory clone,
and `mpm-shadow install --frozen-lockfile`).
That the add waits for those repairs is inferred from the delay and is not stated in the README.
Run the add in the background with its output in a file,
and check the worktree directly.

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
Every timing in this doc was measured on the `synthetic` provider on 2026-11-07,
which the owner describes as a relatively slow provider for this model,
with roughly three times the prefill and generation speed expected later.
Read the seconds as that provider's,
  not as the model's.
The counts of tool calls and turns do not depend on speed.
On 2026-11-08 the owner reported that `deepseek-v4.1-flash` is available through the `neuralwatt` provider.
`pi --list-models flash` lists `neuralwatt` with `deepseek-v4.1-flash`,
`deepseek-v4.1-flash-flex`,
and `deepseek-v4.1-flash-speed`,
each with a 1.0M context and images,
next to the plain and `-flex` and `-speed` variants of `deepseek-v4-flash`.
No request has been sent to any of them from a driven session,
so their speed and behavior under `xhigh` are unmeasured.
A driven session needs `neuralwatt` credentials in its isolated Pi home before it can use one.
- A second task sent through the inbox while the agent was idle became a new prompt in the same session,
  and the model started on it by reading the files its context did not yet hold.
- No dialog request arrived in the runs so far.

## The detailed-brief run

Task:
change the goal reviewer rubric and add tests.
The brief named the files,
the exact new sentences,
the function names,
and the test cases.

- From prompt to settle:
  about 250 seconds on the `synthetic` provider.
- 49 assistant messages,
  67 tool calls (29 `read`,
  25 `grep`,
  6 `edit`,
  5 `ls`,
  2 `find`),
  and 2 tool errors:
  the `find` with a bad directory,
  and a `read` with an offset past the end of the file.
- Usage as reported:
  85006 input tokens,
  26904 output tokens,
  0.136 in cost.
  The input figure is the `usage.input` field summed over messages,
  and whether it excludes cached tokens was not checked.
- No dialog request.
- The edits matched the brief exactly,
  including the verbatim rubric sentences.
  `lint:types`,
  `test:unit` and `lint` all exit 0 after a rebuild,
  with no correction round.
- One review finding:
  its tests sit in a second `describe` that reuses the existing suite name.

### The stale-build trap

Both the type check and the unit tests failed on the model's correct edit before a rebuild.
The test files import the built artifact (`../dist/final/node/index.mjs`),
so until `mise run //package/pi-plugin/goal:build` runs,
the type check reports the old signature (TS2353 on the new field)
and the tests see the old prompt (assertion failures that look like logic errors).
This was the lead's verification mistake,
not the model's.
Run `build` before `lint:types` and `test:unit` in this package,
and check that the new test names appear in the output.

## The unassisted run

The owner's calibration is that DeepSeek V4.1 Flash at `xhigh` is about a Sonnet 5.5 low-level model.
The owner asked whether it works satisfactorily with less handholding.
The brief gave the problem,
the evidence,
and the owner's intent,
and nothing about files,
names,
wording,
steps,
or tests.
The model had a shell,
the repository's own checks to find,
and the read-only git shim below.

What the feed showed:

- It oriented for about three minutes before the first edit,
  reading nine source files,
  the existing tests,
  and the oxlint configuration.
  It found the reviewer prompt,
  the call site,
  and the `mode` signal without pointers.
- It applied the repository's rules from `AGENTS.md`:
  every command starts with `cd -- <path> &&`,
  and its code follows the repository's TSDoc and destructured-parameter style.
- It also found and read this repository's own write-up of the problem under `doc/`.
  The unassisted run therefore had the diagnosis from the repository,
  not only from the brief.
- It chose a different design from the detailed-brief run:
  a post-filter module that replaces reviewer guidance matching phrase lists with a fixed instruction,
  a mode-keyed rubric builder,
  audit fields on the persisted denial event that keep the reviewer's original text,
  validation of those fields in `events.ts`,
  and a decision not to rewrite a human's own manual-rejection text.
- It wrote a dedicated unit test file for its new module,
  extended the built-artifact runtime verifier,
  and made the fake session context take a mode so the tests reach the production wiring.
- It hit the stale-build trap,
  read the error as an out-of-date artifact,
  and ran `build` without being told.
- It ran two mutation checks on its own:
  it broke a guard condition with `perl -pi`,
  rebuilt,
  confirmed that tests failed,
  and restored the line.
  It did the same to the prompt builder.
  The repository requires a guard test to be shown to fail.
  After each restore the lead checked the source by hand.
- It ran `lint`,
  tallied the findings by rule,
  fixed them in rounds,
  and reached 0 warnings and 0 errors.
  It captured real exit codes with `${PIPESTATUS[0]}` after noticing that `| tail` hides them.
- One lint run printed a 5924-character stack trace from inside the repository's oxlint plugin
  (`readonlySourceEvidence`,
  `openSemanticFile`) instead of findings.
  The cause of that trace was not read.

Whether the result is satisfactory is judged in `doc/handover/pi-goal-reviewer-human-input-rubric.md`
and recorded in `doc/agent/pi-goal-extension-driven-through-rpc.md` when the comparison is done.

## Tool and feed quirks

- A multi-block `edit` is rejected whole when one `oldText` does not match.
  The model recovered by grepping the file for the current text and sending the blocks again.
- The model guessed file extensions (`.d.mts` for a `.d.ts` package) and got silent empty results,
  then switched to a recursive search.
- The `bash` tool result carries `structuredContent` with `output`,
  `truncated`,
  `exit_code`,
  and `wall_time_seconds`.
  The driver's feed keeps only the first 500 characters of a result and ignores `structuredContent`,
  so a long lint run shows its tail and no verdict.
  Record `exit_code` in the feed next time.
- Two Monitor tasks on one feed deliver every event twice.
  Stop the old one with `TaskStop`.
  `tail -n 0 -F` waits for a feed file that does not exist yet.
- To wait for a subagent without polling,
  read the unit's main process ID with `systemctl --user show --property=MainPID --value <unit>`
  and wait with `timeout <seconds> tail --pid=<pid> --follow /dev/null` in a background command.

## The read-only git shim

A commit in this repository auto-pushes,
and the model has a shell.
A script named `git` first in the unit's `PATH` lets read-only subcommands through to the real git and refuses the rest.
It guards against mistakes only:
an absolute path to the real git bypasses it.
It was checked to refuse `commit` and `push`
(against a path that is not a repository,
so a failure of the shim could not have committed anything)
and to pass `rev-parse`.

```js
#!/usr/bin/env node
// Read-only git shim for an unattended Pi subagent: lets read-only subcommands through to the real git, refuses the rest.
// A commit in this repository auto-pushes, so the subagent must not be able to commit by accident.
// This is a guard against mistakes, not a sandbox: an absolute path to the real git bypasses it.
const { spawnSync } = require('node:child_process');

const REAL_GIT = process.env.REAL_GIT || '/usr/bin/git';
const READ_ONLY = new Set([
  'status', 'diff', 'log', 'show', 'rev-parse', 'ls-files', 'blame', 'grep', 'describe', 'cat-file', 'rev-list',
  'check-ignore', 'shortlog', 'name-rev', 'ls-tree', 'merge-base', 'show-ref', 'for-each-ref', 'version', 'help',
]);
const READ_ONLY_CONFIG_FLAGS = new Set(['--get', '--get-all', '--list', '-l', '--get-regexp']);

const args = process.argv.slice(2);
let index = 0;
while (index < args.length) {
  const token = args[index];
  if (token === '-C' || token === '-c' || token === '--git-dir' || token === '--work-tree') { index += 2; continue; }
  if (token.startsWith('-')) { index += 1; continue; }
  break;
}
const subcommand = args[index];

const allowed = subcommand === undefined
  || READ_ONLY.has(subcommand)
  || (subcommand === 'config' && args.slice(index + 1).some((flag) => READ_ONLY_CONFIG_FLAGS.has(flag)))
  || (subcommand === 'branch' && args.slice(index + 1).every((flag) => flag.startsWith('-') && ['--show-current', '--list', '-l', '-a', '-r', '-v', '-vv'].includes(flag)));

if (!allowed) {
  process.stderr.write(`git ${subcommand} is blocked in this session. The lead handles all git changes; read-only git commands work.\n`);
  process.exit(1);
}
const result = spawnSync(REAL_GIT, args, { stdio: 'inherit' });
process.exit(result.status ?? 1);
```

Start the unit with `--setenv=PATH="<shim dir>:$PATH"` and `--setenv=REAL_GIT=/usr/bin/git`.
