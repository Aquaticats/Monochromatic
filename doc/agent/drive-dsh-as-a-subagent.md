# Driving dsh as a subagent from another agent

Purpose:
record how an agent in another harness starts a `dsh` session,
sends it messages,
steers it mid-turn,
and reads its events back.
Everything here was run against dsh 0.2.2-rc.1 on 2026-11-07,
except where a line says it was not run.

## Which mode to use

### Web profile over HTTP

`dsh --profile web`.
Run end to end with real model replies.
It is the only mode run that steers a turn already in progress.
Use it when the controller needs to send a follow-up while the agent works.

### SDK JSON-RPC

`dsh --profile sdk`,
newline-delimited JSON-RPC on stdio.
Initialize,
two prompts,
wait,
and shutdown were run.
The second prompt was queued as `next-turn`,
so it does not steer.
The model call failed for lack of a key for the `deepseek-official` route,
so no model reply was seen in this mode.

### ACP

`dsh --profile acp`.
Only `--help` was run.
Its README allows one in-flight prompt per session and cancel,
so it does not steer.
Not run beyond the help text.

### Headless

`dsh --profile headless "task"`.
Read from `--help` only.
One task per process,
resumable with `--session-id`,
with no way to message a running task.
Not run.

## Web profile over HTTP

### Start a server

```sh
# run from the directory that becomes the default workspace, inside a capped unit
systemd-run --user --unit=dsh-web-trial --collect \
  --property=MemoryMax=3G --property=CPUQuota=200% --property=TasksMax=400 --property=IOWeight=1 \
  --property=StandardOutput=file:/path/to/server.log --property=StandardError=file:/path/to/server.err \
  --setenv=PATH="$PATH" --setenv=HOME="$HOME" \
  dsh --profile web --no-open --port 0
```

`--port 0` lets the OS choose a free port,
which is what lets several instances run at once.
The server prints one line to stdout:
`dsh web: http://127.0.0.1:<port>/?token=<launch token>`.
Each process mints its own launch token.
The token is a bearer secret for that process,
so read it from the log inside the client and never echo it into a transcript.

### Sign in as an API caller

`GET /connect?token=<token>&interactions=none` answers 302 with a signed cookie in `Set-Cookie`.
The `none` value means unary HTTP calls only,
with no WebSocket.
Documented in `packages/client/connection/README.md`,
section "Browser authentication and request trust",
of the `deepseek-ai/deepseek-harness` repository.

### Call an endpoint

`POST /api/<endpoint>` with `content-type: application/json`,
the cookie,
and this body:

```json
{ "type": "client-request", "rpcId": "<uuid>", "method": "<endpoint>", "payload": { "args": { } } }
```

The response is `{ "type": "server-response", "rpcId": ..., "result": { "ok": true, "value": ... } }`
or `ok: false` with `error.code` and `error.message`.

`args` is keyed by the Host method's parameter names.
A wrong shape returns `gateway/arguments-invalid` naming the missing and unexpected keys,
so the server tells you the right key.
Most session methods take one parameter named `request`,
and `session/list` names it `_request` because it ignores it.

### Calls that were run

- `session/list` with `{"_request":{}}`.
- `session/create` with `{"request":{"cwd":"<absolute path>","sessionId":"<your id>"}}`
  returns `{ sessionId, agentPreset }`.
  A `sessionId` with no stored session becomes the new session's id.
- `session/prompt` with
  `{"request":{"requestId":"<unique>","sessionId":"<id>","mode":"queue"|"steer","content":[{"type":"text","text":"..."}]}}`
  returns `{ accepted: true }`.
  `requestId` makes a retry return the original acceptance.
- `session/wait` with `{"request":{"sessionId":"<id>"}}`
  returns when the session goes idle,
  with `reason`,
  `state`,
  and `output`.
- `session/state` with the same shape returns `cursor`,
  `running`,
  and the pending `queue`.
- `session/events` with
  `{"request":{"address":{"kind":"session","sessionId":"<id>"}}}`
  returns `records` of `{ type: "event", event: { type, seq, time, data } }`.
  The address needs `kind: "session"`.

`session/follow` is a live stream over the WebSocket and was not run.

### Queue versus steer

Observed in the event log of a session that took a queued prompt and then a steer prompt 320 ms later:

- The `queue` prompt entered the inbox with `target: "next-turn"` and started turn 1.
- The `steer` prompt entered the inbox with `target: "next-step"` at seq 7,
  before step 1 began at seq 8,
  so the agent read it at its next step boundary inside the same turn.
  The agent's reasoning at step 2 already mentioned the steered instruction,
  and the final reply followed it.
- When the first turn failed before consuming the steer prompt,
  the steer prompt stayed in `session/state` `queue` with `placement: "steering"`.

### Reading a reply

Fetch `session/events` and take the last `assistant/message` event.
Its `data.message.content` holds `text` parts.
`turn/end` carries `reason.kind`,
`completed` on success and `error` with a message on failure.

### Client script

```ts
// web-call.ts: sign in to a local `dsh --profile web` server as an API caller, then run unary calls.
// usage: node web-call.ts <server-log> <out-jsonl> <endpoint> <argsJson> [<endpoint> <argsJson> ...]
import { appendFileSync, readFileSync } from 'node:fs';
import { randomUUID } from 'node:crypto';

const [serverLog, outFile, ...callArguments] = process.argv.slice(2);
if (serverLog === undefined || outFile === undefined || callArguments.length === 0 || callArguments.length % 2 !== 0) {
  throw new Error('usage: web-call.ts <server-log> <out-jsonl> <endpoint> <argsJson> [...]');
}
const startedAt = Date.now();
const record = (entry: Record<string, unknown>): void => {
  const line = JSON.stringify({ elapsedMs: Date.now() - startedAt, ...entry });
  appendFileSync(outFile, `${line}\n`);
  console.log(line.length > 1500 ? `${line.slice(0, 1500)}...` : line);
};

// The launch token stays inside this process; it is read from the server log and never printed.
const urlMatch = /(http:\/\/127\.0\.0\.1:\d+)\/\?token=([^\s&]+)/.exec(readFileSync(serverLog, 'utf8'));
if (urlMatch === null) throw new Error('no server URL with token in the server log yet');
const origin = urlMatch[1] as string;
const token = urlMatch[2] as string;

const connect = await fetch(`${origin}/connect?token=${encodeURIComponent(token)}&interactions=none`, { redirect: 'manual' });
const cookie = connect.headers.getSetCookie().map((entry) => entry.split(';', 1)[0]).join('; ');
record({ step: 'connect', status: connect.status, gotCookie: cookie !== '' });
if (cookie === '') throw new Error(`sign-in produced no cookie (HTTP ${connect.status})`);

for (let index = 0; index < callArguments.length; index += 2) {
  const endpoint = callArguments[index] as string;
  const args = JSON.parse(callArguments[index + 1] as string) as unknown;
  const rpcId = randomUUID();
  const response = await fetch(`${origin}/api/${endpoint}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json', cookie, origin },
    body: JSON.stringify({ type: 'client-request', rpcId, method: endpoint, payload: { args } }),
  });
  record({ step: 'call', endpoint, args, status: response.status, body: await response.text() });
}
```

The script sends an `Origin` header equal to the server origin.
Whether the server requires it was not tested.

## Home, credentials, and isolation

The server reads `$DSH_HOME`,
which defaults to `~/.dsh`,
and takes model keys from the credentials service stored in `$DSH_HOME/.credentials.yaml`
or from the launching environment.
A session created under the real home is a real session:
it appears in the owner's session list and writes a log under `$DSH_HOME/sessions/<cwd slug>/<session id>/`.

To run without touching the real home,
create a throwaway `DSH_HOME` and copy the `profiles/web` directory into it.
That copy has no credentials,
so every model call ends with `no credential for provider route`.
Copy the credentials file with a plain `cp` to get real replies,
not `cp --no-clobber`:
the server creates its own small `.credentials.yaml` on first start,
and `--no-clobber` then silently keeps it.
Stop the server before replacing the file,
because the file is loaded at startup,
and delete the copy afterwards.

The user-level `$DSH_HOME/cordis.patch.yml` is not part of the profile directory,
so a throwaway home does not carry it.
That difference is how `doc/troubleshooting/dsh-list-agents-subpath-removed-in-0-2-2-rc-1.md` was found.

## Several instances at once

Two `dsh --profile web --port 0` instances ran at the same time on one real home.
They took ports 37289 and 33039,
printed different launch tokens,
and opened different inspector ports,
9231 and 9232.
Both listed the same 340 sessions.
Each session has one writer:
instance A accepted a prompt for session `trial-real-home-2`,
and instance B's prompt for the same session failed with
`session/writer-held` and the message `already owned by an active write handle`.
A listen failure on a fixed port rejects start-up with the bind diagnostic,
so `--port 0` is the safe default.

## Defaults to know before sending real work

The owner's web profile composes `danger-full-access` and approval policy `never`
into new sessions.
An agent started this way runs commands anywhere without asking.
Create sessions in a disposable working directory,
or choose another permission preset for the session.
How to pick a preset through the HTTP API was not tested.

## Cleanup checklist

- Stop the unit with `systemctl --user stop <unit>`,
  then confirm no `dsh --profile` process or `dsh-*` unit remains.
- Delete any credentials copy from a throwaway home.
- A session created under the real home stays there.
  Remove it only with the owner's say-so,
  since it is session data.
- Cache files at `$DSH_HOME/storages/session_projcache/sessions/<id>.json` are derived data
  that a failed `session/create` can leave behind.

## See also

`doc/agent/drive-pi-as-a-subagent.md` covers the same job for Pi in RPC mode,
and its "Compared with driving dsh" section weighs the two.

## Not verified

- Two sessions running at once inside one web server.
  The ACP README says one connection can run several sessions at once,
  and `session/list` showed 340 persisted sessions.
  Every run here executed one session at a time,
  so concurrent execution in one server is documented and not tested.
- ACP and headless beyond their `--help` text.
- `session/follow` and the WebSocket stream.
- Choosing a permission preset or model per session through the API.
- Whether the HTTP server requires the `Origin` header.
