# Chromium issue 581090722 watcher

## Running watch verified at 2026-11-03T12:12:41+00:00

The user requested at least `24` hours of scripted monitoring for
<https://issues.chromium.org/issues/581090722>.
The user service `chromium-issue-581090722-watch.service` is enabled and running.
Its saved window begins at `2026-11-03T12:07:36.851Z`
and ends at `2026-11-04T12:07:36.851Z`.
The loop checks every `5` minutes and performs a final check at its deadline.

The verification observed `3` successful checks,
no current error,
and an unchanged baseline.
The live report has the expected title,
the `Internals>Metrics` component,
and the sanitized evidence attachment.
The published disclosure says it was prepared with AI assistance;
the local draft subsequently made the agent's role more explicit.

## Runtime behavior

The watcher retrieves public issue,
comment,
and update views through the anonymous read operations used by the tracker.
It uses no signed-in browser or account credentials.
The request operations return the expected issue-fetch and list response kinds.
Two consecutive live reads produced the same normalized snapshot hash.

Changed payloads and read failures produce local desktop notifications.
The watcher retains changes and errors in a private event log,
with snapshots and an atomic checkpoint under
`${HOME}/.local/state/chromium-issue-watch/581090722`.
Pending notifications survive process restarts.
The code does not post issue comments or replies.
Human maintainer feedback requires a human reply.

The service uses one CPU,
`512M` memory,
`64` tasks,
and I/O weight `10`.
It restarts after failures and preserves the saved deadline.
The script disables its own service at completion;
the service also has a `25h` runtime ceiling.
User lingering was already enabled,
so the service does not depend on this chat or an open terminal.

## Validation and retained sources

The strict TypeScript check passed through the local `lint:types` Mise task.
The functional checks exercised live response decoding,
baseline and unchanged observations,
a positive change control,
durable pending notifications,
recovery,
and malformed response rejection.
The operational service then completed actual public tracker reads.

The installed runtime uses the locked Node `26.13.0` executable through local Mise tasks
named `init`,
`watch`,
`test`,
and `lint:types`.
The unit is stored in `${HOME}/.config/systemd/user`.
Raw responses and the checkpoint contain reporter/comment data and remain local.
The source copies in this record contain no account credentials or raw snapshots.


## client.ts

```typescript
// ${HOME}/.local/state/chromium-issue-watch/581090722/client.ts
import { createHash } from 'node:crypto';

//region Read-only wire contract observed in the public tracker page.
export const issueId = 581090722;
export const issueUrl = `https://issues.chromium.org/issues/${issueId}`;
const responseKindSlot = 0;
const responsePayloadSlot = 1;
const issueNumberSlot = 1;
const issueFieldsSlot = 2;
const issueTitleSlot = 5;
const maxResponseBytes = 2 * 1024 * 1024;
const requestTimeoutMs = 30_000;
const xssiPrefix = ")]}'";

export class WatchReadError extends Error {}

export type Snapshot = {
  issueFields: unknown[];
  comments: unknown;
  updates: unknown;
};

/** Validate the response kind before interpreting the captured wire shape. */
export function decodeResponse({ text, kind }: { text: string; kind: string }): unknown {
  const content = text.startsWith(xssiPrefix) ? text.slice(xssiPrefix.length).trimStart() : text;
  let parsed: unknown;
  try { parsed = JSON.parse(content); }
  catch (error) { throw new WatchReadError(`Invalid JSON for ${kind}`, { cause: error }); }
  if (!Array.isArray(parsed)) throw new WatchReadError('Tracker response is not an array');
  const envelope = parsed.find(entry => Array.isArray(entry) && entry[responseKindSlot] === kind);
  if (!envelope) throw new WatchReadError(`Tracker response no longer contains ${kind}`);
  if (!Array.isArray(envelope[responsePayloadSlot])) throw new WatchReadError(`${kind} has no array payload`);
  return envelope[responsePayloadSlot];
}

/** Locate the requested issue without including unrelated tracker configuration. */
export function issueFields(payload: unknown): unknown[] {
  const pending: unknown[] = [payload];
  while (pending.length > 0) {
    const value = pending.pop();
    if (!Array.isArray(value)) continue;
    const fields: unknown = value[issueFieldsSlot];
    if (value[issueNumberSlot] === issueId && Array.isArray(fields)
      && typeof fields[issueTitleSlot] === 'string') return fields;
    for (const child of value) pending.push(child);
  }
  throw new WatchReadError(`Tracker response does not contain issue ${issueId}`);
}

/** Read the same anonymous endpoints used by the public issue page. */
async function read({ endpoint, body, kind }: { endpoint: string; body: unknown[]; kind: string }): Promise<unknown> {
  const response = await fetch(`https://issues.chromium.org/action/issues/${issueId}/${endpoint}`, {
    method: 'POST', headers: { 'Content-Type': 'application/json', Accept: 'application/json', 'Cache-Control': 'no-cache' },
    body: JSON.stringify(body), redirect: 'error', signal: AbortSignal.timeout(requestTimeoutMs),
  });
  if (!response.ok || !response.body) throw new WatchReadError(`${endpoint}: HTTP ${response.status}`);
  const chunks: Uint8Array[] = [];
  let bytes = 0;
  for await (const chunk of response.body) {
    bytes += chunk.byteLength;
    if (bytes > maxResponseBytes) throw new WatchReadError(`${endpoint}: response exceeds ${maxResponseBytes} bytes`);
    chunks.push(chunk);
  }
  return decodeResponse({ text: Buffer.concat(chunks).toString('utf8'), kind });
}

/** Preserve the exact read request tuples captured from the live page. */
export async function fetchSnapshot(): Promise<Snapshot> {
  const results = await Promise.allSettled([
    read({ endpoint: 'getIssue?currentTrackerId=157', body: [issueId, 1, 1], kind: 'b.IssueFetchResponse' }),
    read({ endpoint: 'listComments', body: [issueId, 'DESC', 1], kind: 'b.ListIssueCommentsResponse' }),
    read({ endpoint: 'updates?currentTrackerId=157', body: [issueId, 'ASC', 1100, null, 2], kind: 'b.ListIssueUpdatesResponse' }),
  ]);
  for (const result of results) if (result.status === 'rejected') throw new WatchReadError(String(result.reason), { cause: result.reason });
  const [issue, comments, updates] = results.map(result => {
    if (result.status !== 'fulfilled') throw new WatchReadError('Incomplete tracker response batch');
    return result.value;
  });
  return { issueFields: issueFields(issue), comments, updates };
}

/** Compare semantic payloads without request timing or unrelated page configuration. */
export function snapshotHash(snapshot: Snapshot): string {
  return createHash('sha256').update(JSON.stringify(snapshot)).digest('hex');
}
//endregion
```

## state.ts

```typescript
// ${HOME}/.local/state/chromium-issue-watch/581090722/state.ts
import { appendFile, readFile, rename, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { issueId, snapshotHash, type Snapshot } from './client.ts';

//region Atomic checkpoints and change detection for one service-owned watch.
export type WatchState = {
  issueId: number;
  startedAt: string;
  endsAt: string;
  checks: number;
  lastAttemptAt: string | null;
  lastSuccessAt: string | null;
  hash: string | null;
  snapshot: Snapshot | null;
  error: string | null;
  pendingNotification: boolean;
  complete: boolean;
};
export type WatchEvent = { kind: string; at: string; detail?: string; hash?: string };
export class WatchStateError extends Error {}

/** Replace a checkpoint without exposing a partly written JSON file. */
export async function save({ directory, state }: { directory: string; state: WatchState }): Promise<void> {
  const temporary = join(directory, `state-${randomUUID()}.new`);
  await writeFile(temporary, JSON.stringify(state, null, 2) + '\n', { mode: 0o600 });
  await rename(temporary, join(directory, 'state.json'));
}

/** Load the existing deadline so a restart cannot reset the monitoring window. */
export async function load(directory: string): Promise<WatchState> {
  const state: WatchState = JSON.parse(await readFile(join(directory, 'state.json'), 'utf8'));
  if (state.issueId !== issueId || !Number.isFinite(Date.parse(state.endsAt))) {
    throw new WatchStateError(`Invalid watch checkpoint for issue ${issueId}`);
  }
  return state;
}

/** Retain actionable observations locally; repeated unchanged checks remain quiet. */
export async function record({ directory, event }: { directory: string; event: WatchEvent }): Promise<void> {
  await appendFile(join(directory, 'events.jsonl'), JSON.stringify(event) + '\n', { mode: 0o600 });
  console.log(JSON.stringify(event));
}

/** Turn a successful read into a baseline, change, recovery, or unchanged checkpoint. */
export function observe({ state, snapshot, at }: { state: WatchState; snapshot: Snapshot; at: string }): { state: WatchState; event?: WatchEvent } {
  const hash = snapshotHash(snapshot);
  const baseline = state.hash === null;
  const changed = !baseline && state.hash !== hash;
  const recovered = state.error !== null;
  const kind = baseline ? 'baseline' : changed ? 'changed' : recovered ? 'recovered' : undefined;
  return {
    state: { ...state, checks: state.checks + 1, lastAttemptAt: at, lastSuccessAt: at, hash, snapshot, error: null,
      pendingNotification: state.pendingNotification || changed || recovered },
    ...(kind ? { event: { kind, at, hash } } : {}),
  };
}
//endregion
```

## watch.ts

```typescript
// ${HOME}/.local/state/chromium-issue-watch/581090722/watch.ts
import { execFile } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { writeFile } from 'node:fs/promises';
import { setTimeout as wait } from 'node:timers/promises';
import { fetchSnapshot, issueId, issueUrl } from './client.ts';
import { load, observe, record, save } from './state.ts';

//region A bounded read-only watch, independent of the chat session.
const directory = dirname(fileURLToPath(import.meta.url));
const unit = `chromium-issue-${issueId}-watch.service`;
const intervalMs = 5 * 60 * 1000;
const stop = new AbortController();
class WatchProcessError extends Error {}
process.on('SIGTERM', () => stop.abort());
process.on('SIGINT', () => stop.abort());

/** Await a native notification or service command with a bounded child lifetime. */
async function command({ executable, args }: { executable: string; args: string[] }): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    execFile(executable, args, { timeout: 10000, maxBuffer: 65536 }, (error, _stdout, stderr) => {
      if (error) reject(new WatchProcessError(`${executable}: ${error.message}; ${stderr}`, { cause: error }));
      else resolve();
    });
  });
}

/** Keep pending notifications durable so a process restart cannot silently lose one. */
async function main(): Promise<void> {
  let state = await load(directory);
  if (state.complete) {
    await command({ executable: '/usr/bin/systemctl', args: ['--user', 'disable', unit] });
    return;
  }
  await record({ directory, event: { kind: 'watch_started', at: new Date().toISOString(), detail: `Checking ${issueUrl} every ${intervalMs} ms through ${state.endsAt}` } });
  while (!stop.signal.aborted) {
    const at = new Date().toISOString();
    try {
      const snapshot = await fetchSnapshot();
      const observed = observe({ state, snapshot, at: new Date().toISOString() });
      state = { ...observed.state, lastAttemptAt: at };
      if (observed.event) {
        await writeFile(join(directory, `snapshot-${Date.now()}.json`), JSON.stringify(state.snapshot, null, 2) + '\n', { mode: 0o600 });
        await record({ directory, event: observed.event });
      }
    } catch (error) {
      const detail = String(error);
      if (state.error !== detail) await record({ directory, event: { kind: 'read_failed', at, detail } });
      state = { ...state, checks: state.checks + 1, lastAttemptAt: at, error: detail,
        pendingNotification: state.pendingNotification || state.error === null };
    }
    await save({ directory, state });
    if (state.pendingNotification) {
      try {
        await command({ executable: '/usr/bin/notify-send', args: ['--app-name=Chromium issue watch', '--urgency=normal',
          `Chromium issue ${issueId}`, state.error ? 'A tracker read failed. The watcher will retry.' : `The issue changed or monitoring recovered. Review ${issueUrl}`] });
        state = { ...state, pendingNotification: false };
        await save({ directory, state });
      } catch (error) {
        await record({ directory, event: { kind: 'notification_failed', at: new Date().toISOString(), detail: String(error) } });
      }
    }
    const remainingMs = Date.parse(state.endsAt) - Date.now();
    if (remainingMs <= 0) {
      state = { ...state, complete: true };
      await save({ directory, state });
      await record({ directory, event: { kind: 'watch_complete', at: new Date().toISOString(), detail: state.endsAt } });
      await command({ executable: '/usr/bin/systemctl', args: ['--user', 'disable', unit] });
      return;
    }
    try { await wait(Math.min(intervalMs, remainingMs), undefined, { signal: stop.signal }); }
    catch (error) {
      if (!stop.signal.aborted) throw error;
      await record({ directory, event: { kind: 'watch_interrupted', at: new Date().toISOString(), detail: String(error) } });
    }
  }
}
await main();
//endregion
```

## init.ts

```typescript
// ${HOME}/.local/state/chromium-issue-watch/581090722/init.ts
import { appendFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { fetchSnapshot, issueId, snapshotHash } from './client.ts';
import type { WatchState } from './state.ts';

const directory = dirname(fileURLToPath(import.meta.url));
const attemptedAt = new Date().toISOString();
const snapshot = await fetchSnapshot();
const started = Date.now();
const hourMs = 60 * 60 * 1000;
const durationMs = 24 * hourMs;
const state: WatchState = {
  issueId, startedAt: new Date(started).toISOString(), endsAt: new Date(started + durationMs).toISOString(),
  checks: 1, lastAttemptAt: attemptedAt, lastSuccessAt: new Date(started).toISOString(),
  hash: snapshotHash(snapshot), snapshot, error: null, pendingNotification: false, complete: false,
};
await writeFile(join(directory, 'state.json'), JSON.stringify(state, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
const event = { kind: 'watch_initialized', at: state.startedAt, endsAt: state.endsAt, hash: state.hash };
await appendFile(join(directory, 'events.jsonl'), JSON.stringify(event) + '\n', { mode: 0o600 });
console.log(JSON.stringify(event));
```

## test.ts

```typescript
// ${HOME}/.local/state/chromium-issue-watch/581090722/test.ts
import assert from 'node:assert/strict';
import { mkdtemp } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { decodeResponse, fetchSnapshot, issueFields, issueId, snapshotHash } from './client.ts';
import { observe, load, save, type WatchState } from './state.ts';

const fixture = await fetchSnapshot();
const at = new Date().toISOString();
const state: WatchState = { issueId, startedAt: at, endsAt: at, checks: 0, lastAttemptAt: null, lastSuccessAt: null,
  hash: null, snapshot: null, error: null, pendingNotification: false, complete: false };
const baseline = observe({ state, snapshot: fixture, at });
assert.equal(baseline.event?.kind, 'baseline');
assert.equal(baseline.state.pendingNotification, false);
const unchanged = observe({ state: baseline.state, snapshot: fixture, at });
assert.equal(unchanged.event, undefined);
const changedFixture = { ...fixture, issueFields: [...fixture.issueFields, 'isolated positive-control field'] };
const changed = observe({ state: baseline.state, snapshot: changedFixture, at });
assert.equal(changed.event?.kind, 'changed');
assert.equal(changed.state.pendingNotification, true);
const directory = await mkdtemp(join(homedir(), 'temp/agent/chromium-watch-state-'));
await save({ directory, state: changed.state });
const restarted = observe({ state: await load(directory), snapshot: changedFixture, at });
assert.equal(restarted.event, undefined);
assert.equal(restarted.state.pendingNotification, true);
const recovered = observe({ state: { ...baseline.state, error: 'fixture transport failure' }, snapshot: fixture, at });
assert.equal(recovered.event?.kind, 'recovered');
assert.equal(recovered.state.pendingNotification, true);
assert.throws(() => decodeResponse({ text: 'not JSON', kind: 'b.IssueFetchResponse' }));
assert.throws(() => decodeResponse({ text: '[["wrong",[]]]', kind: 'b.IssueFetchResponse' }));
assert.throws(() => issueFields([]));
console.log('PASS live response parsing; baseline; unchanged; positive change; durable pending notification; recovery; rejected malformed responses');
const second = await fetchSnapshot();
assert.equal(snapshotHash(fixture), snapshotHash(second), 'Consecutive live reads changed; inspect the payload differences');
console.log(JSON.stringify({ liveAnonymousReads: 'passed', stableHash: snapshotHash(second) }));
```
