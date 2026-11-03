# Chromium live histogram reproduction

## Verified upstream run at 2026-11-03T10:03:12+00:00

This record retains the tested observation and recording clients.
The [public report](../../planning/browser-disk-responsiveness/chromium-metrics-ping.md)
and [sanitized evidence](../../planning/browser-disk-responsiveness/chromium-metrics-ping-evidence.json)
contain the measured result.

Official Chromium snapshot `1733707` reported `159.0.8141.0`.
Its main thread faulted twice while writing inside its live disk-backed `BrowserMetrics` mapping.
Both faulting instructions resolve to `base::SampleVectorBase::Accumulate(int, int)`.
The instruction at ELF virtual address `0xce44e00` is `lock xadd %r12d,(%rax)`.
The code segment's virtual address exceeds its file offset by `0x1000`;
symbolization used the virtual address,
not file offset `0xce43e00`.
`addr2line --functions --demangle` returned `sample_vector.cc:?`,
so no source line number is claimed.

The native recording probe completed with `60` ordinary and `60` sparse samples.
The same counts appeared in the rendered histogram page with subprocess imports disabled.
The trace read `4922` write-fault events;
every per-CPU trace buffer reported zero overruns and dropped events.
The first observed metrics wait was in writeback;
the second was in Btrfs metadata reservation.

The recorded blocked spans were `104 ms` and `1484 ms`.
A browser-window command took `1159.963 ms` inside the latter span.
These are sampled spans and a protocol round trip,
not exact fault durations or input-to-presentation measurements.
The observer logs blocked samples and recovery observations.
Scheduling and observation work can widen the gap between them.
The evidence does not attribute the entire command delay to one fault.
The trace ended before the recording loop's final calls.
Its second correlated fault falls within their overlap.
An unmatched read-fault observation remains excluded from metrics attribution.

## Input and isolation

The selected archive was
<https://storage.googleapis.com/chromium-browser-snapshots/Linux_x64/1733707/chrome-linux.zip>.
The archive and executable hashes are retained in the sanitized evidence.
The test used kernel `7.2.15-ogc1.1.fc45.x86_64`,
Wayland,
and a newly created profile on Btrfs.

The original launch used a disposable `--user-data-dir`.
Its crash handler nevertheless selected the default Chromium crash-report directory.
A separate launch verified that setting `CHROME_CONFIG_HOME` to the fixture's `isolated-config`
moved the crash-report database under that directory.
That placement check did not repeat the latency measurement.
The existing default crash-report directory was not deleted or restored.

The browser launcher scope used `CPUQuota=400%`,
`MemoryMax=4G`,
`TasksMax=512`,
and `IOWeight=10`.
Chromium moved its main process into a desktop scope;
the same limits were then applied to that actual scope.
The collector used one CPU,
`512M`,
`64` tasks,
and I/O weight `10`.
No synthetic host load,
global cache dropping,
or injected storage delay was used.
The recording calls were a finite functional test at a `1000 ms` cadence.

## Running the retained clients

These clients require Linux procfs,
tracefs exposing `exceptions/page_fault_user`,
permission to create a trace instance,
Node with TypeScript stripping,
and the pinned Chromium build.
The recorded run used Node `26.13.0`.
No browser account is needed.

The clients derive paths from the current user's home.
They expect the unpacked build and disposable profile under
`${HOME}/temp/agent/chromium-live-repro-1733707`.
The browser needs a non-default `--user-data-dir`,
`--remote-debugging-port=0`,
`--no-first-run`,
`--no-default-browser-check`,
and `--ozone-platform=wayland`.
Use disposable `CHROME_CONFIG_HOME` and `XDG_CACHE_HOME` directories for a repeat.

Save the fenced sources to their named files.
Select the browser main PID from the complete command line:
it must name the disposable profile and contain no `--type=` subprocess flag.
The observer accepts that PID,
a private output path,
and a duration in milliseconds.
The recorded duration argument was `180000`.
The observer needs permission to read the main thread's kernel stack and create its trace instance.

Start the observer before the recording client.
The recording client opens a settings target,
records samples,
checks histogram counts,
and records browser-window command timing.
The native recording functions have no completion callback;
histogram results verify completion.

A run without a correlated blocked sample is an observation with no captured stall.
The fixture does not force a disk wait,
so absence of a captured stall does not disprove the mechanism.
This is an intermittent reproduction rather than a deterministic regression fixture.

The observer disables and removes only its own trace instance at its deadline.
Close the owned browser through `Browser.close`,
wait for its process to exit,
then stop any remaining owned scopes.
Raw captures contain private paths,
PIDs,
addresses,
and instance identifiers;
keep them private.
The public evidence contains selected sanitized fields.

## Cleanup verification at 2026-11-03T10:15:25.739+00:00

The primary measurement browser exited with code zero.
The separate profile-placement probe emitted Chromium's
`FATAL:content/common/zygote/zygote_communication_linux.cc:176`
with `NOTREACHED hit. Did not receive ping from zygote child`
and exited with code `134` when its desktop and launcher scopes were stopped together.
That is a separate teardown observation,
not part of the metrics fault evidence.

A subsequent isolated lifecycle check sent `Browser.close`,
received its acknowledgment,
and waited for browser exit before scope cleanup.
It exited with code zero,
with neither `FATAL` nor `NOTREACHED` in its retained stderr.
The final procfs executable scan found no processes from the owned snapshot directory.
The observer's tracefs instance was absent after its normal cleanup.

## Document verification command

Run `mise run verify:metrics-ping` from `doc/planning/browser-disk-responsiveness`.
The root-qualified lookup returned `mise ERROR no task` because this document configuration
is outside the repository's package discovery roots.
The invocation from the owning directory passed,
following the [recorded Mise workaround](../mise-excluded-research-task-discovery.md).

## Observer source

```typescript
// ${HOME}/temp/agent/chromium-live-fault-trace.ts
import { spawn } from 'node:child_process';
import { appendFileSync, mkdirSync, readFileSync, readdirSync, rmdirSync, writeFileSync } from 'node:fs';
import { setTimeout as wait } from 'node:timers/promises';

const [pidText, output, durationText] = process.argv.slice(2);
const pid = Number(pidText);
const durationMs = Number(durationText);
if (!Number.isSafeInteger(pid) || pid <= 1 || !output || !Number.isSafeInteger(durationMs) || durationMs <= 0) {
  throw new Error('Expected browser PID, output path and bounded duration in milliseconds');
}
const instance = `/sys/kernel/tracing/instances/chromium-live-${process.pid}`;
const deadline = Date.now() + durationMs;
let stopped = false;
let lastFault = '';
let pendingText = '';
let blockedAt = 0;
let matchedSamples = 0;
let faultEvents = 0;
let reader: ReturnType<typeof spawn> | undefined;

function read(path: string): string {
  try { return readFileSync(path, 'utf8').trim(); }
  catch (error) { return `unavailable: ${(error as NodeJS.ErrnoException).code}`; }
}
function log(data: object): void {
  appendFileSync(output, JSON.stringify({ at: new Date().toISOString(), mono: Number(process.hrtime.bigint()) / 1e9, pid, ...data }) + '\n', { mode: 0o600 });
}
function mappingFor({ address, maps }: { address: string | undefined; maps: string }): string {
  if (!address) return 'no address';
  const value = BigInt(address);
  for (const line of maps.split('\n')) {
    const range = /^([0-9a-f]+)-([0-9a-f]+)/u.exec(line);
    if (range && value >= BigInt(`0x${range[1]}`) && value < BigInt(`0x${range[2]}`)) return line;
  }
  return 'unmapped';
}

mkdirSync(instance);
try {
  writeFileSync(`${instance}/tracing_on`, '0');
  writeFileSync(`${instance}/buffer_size_kb`, '64');
  writeFileSync(`${instance}/trace_clock`, 'mono');
  writeFileSync(`${instance}/set_event_pid`, String(pid));
  writeFileSync(`${instance}/events/exceptions/page_fault_user/filter`, 'error_code & 2');
  writeFileSync(`${instance}/events/exceptions/page_fault_user/enable`, '1');
  reader = spawn('/usr/bin/cat', [`${instance}/trace_pipe`], { stdio: ['ignore', 'pipe', 'pipe'] });
  reader.stdout?.setEncoding('utf8');
  reader.stdout?.on('data', (chunk: string) => {
    pendingText += chunk;
    const lines = pendingText.split('\n');
    pendingText = lines.pop() ?? '';
    for (const line of lines) if (line.includes('page_fault_user:')) { lastFault = line; faultEvents++; }
  });
  reader.stderr?.on('data', chunk => log({ state: 'reader-error', detail: String(chunk) }));
  reader.on('exit', () => { stopped = true; });
  process.on('SIGTERM', () => { stopped = true; });
  process.on('SIGINT', () => { stopped = true; });
  writeFileSync(`${instance}/tracing_on`, '1');
  log({ state: 'armed', instance, durationMs, mappings: read(`/proc/${pid}/maps`).split('\n').filter(line => line.includes('BrowserMetrics')) });
  while (!stopped && Date.now() < deadline) {
    const stat = read(`/proc/${pid}/stat`);
    if (stat.startsWith('unavailable')) break;
    if (stat.includes(') D ')) {
      if (!blockedAt) blockedAt = Date.now();
      const stack = read(`/proc/${pid}/stack`);
      const syscall = read(`/proc/${pid}/syscall`);
      const fault = lastFault;
      const address = /address=(0x[0-9a-f]+)/u.exec(fault)?.[1];
      const ip = /ip=(0x[0-9a-f]+)/u.exec(fault)?.[1];
      const maps = read(`/proc/${pid}/maps`);
      const addressMapping = mappingFor({ address, maps });
      const instructionMatches = Boolean(ip) && ip === syscall.split(/\s+/u).at(-1);
      const metricsWrite = instructionMatches && stack.includes('btrfs_page_mkwrite') && addressMapping.includes('/BrowserMetrics/');
      if (metricsWrite) matchedSamples++;
      log({ state: 'blocked', stack, syscall, fault, addressMapping, instructionMatches, metricsWrite });
    } else if (blockedAt) {
      log({ state: 'recovered', sampledBlockedMs: Date.now() - blockedAt });
      blockedAt = 0;
    }
    await wait(100);
  }
} finally {
  writeFileSync(`${instance}/tracing_on`, '0');
  const statistics = readdirSync(`${instance}/per_cpu`).map(cpu => ({ cpu, stats: read(`${instance}/per_cpu/${cpu}/stats`) }));
  reader?.kill('SIGTERM');
  await wait(100);
  writeFileSync(`${instance}/events/exceptions/page_fault_user/enable`, '0');
  rmdirSync(instance);
  log({ state: 'stopped', matchedSamples, faultEvents, statistics });
  console.log(JSON.stringify({ output, matchedSamples, faultEvents }));
}
```

## Native recording and browser-window timing source

```typescript
// ${HOME}/temp/agent/chromium-live-recording-probe.ts
import { appendFileSync, readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';

const base = join(homedir(), 'temp/agent/chromium-live-repro-1733707');
const [port] = readFileSync(join(base, 'profile/DevToolsActivePort'), 'utf8').split('\n');
const metadata = await (await fetch(`http://127.0.0.1:${port}/json/version`)).json();
const protocol = await (await fetch(`http://127.0.0.1:${port}/json/protocol`)).json();
const browserCommands = protocol.domains.find((domain: any) => domain.domain === 'Browser').commands;
for (const command of ['getHistogram', 'getWindowForTarget']) {
  if (!browserCommands.some((entry: any) => entry.name === command)) throw new Error(`Missing Browser.${command}`);
}
const output = join(base, 'recording-probe.jsonl');
function record(data: object): void {
  appendFileSync(output, JSON.stringify({ at: new Date().toISOString(), mono: Number(process.hrtime.bigint()) / 1e9, ...data }) + '\n', { mode: 0o600 });
}
const socket = new WebSocket(metadata.webSocketDebuggerUrl);
await new Promise<void>((resolve, reject) => {
  socket.addEventListener('open', () => resolve(), { once: true });
  socket.addEventListener('error', event => reject(new Error(`WebSocket open failed: ${String(event)}`)), { once: true });
});
let sequence = 0;
const pending = new Map<number, { resolve: (value: any) => void; reject: (error: Error) => void; timer: ReturnType<typeof setTimeout> }>();
socket.addEventListener('message', event => {
  const response = JSON.parse(String(event.data));
  const request = pending.get(response.id);
  if (!request) return;
  pending.delete(response.id);
  clearTimeout(request.timer);
  if (response.error) request.reject(new Error(JSON.stringify(response.error)));
  else request.resolve(response.result);
});
async function command({ method, params = {}, sessionId }: { method: string; params?: object; sessionId?: string }): Promise<any> {
  const id = ++sequence;
  const started = performance.now();
  const startedAt = new Date().toISOString();
  const result = await new Promise<any>((resolve, reject) => {
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`Timed out: ${method}`)); }, 30000);
    pending.set(id, { resolve, reject, timer });
    socket.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
  });
  record({ kind: 'command', method, startedAt, elapsedMs: performance.now() - started, result });
  return result;
}

try {
  record({ kind: 'metadata', metadata });
  const { targetId } = await command({ method: 'Target.createTarget', params: { url: 'chrome://settings/' } });
  const { sessionId } = await command({ method: 'Target.attachToTarget', params: { targetId, flatten: true } });
  const readyDeadline = Date.now() + 15000;
  while (true) {
    const response = await command({ method: 'Runtime.evaluate', sessionId, params: { expression: "({boolean:typeof chrome.metricsPrivate?.recordBoolean,sparse:typeof chrome.metricsPrivate?.recordSparseValue})", returnByValue: true } });
    if (response.result?.value?.boolean === 'function' && response.result?.value?.sparse === 'function') break;
    if (Date.now() >= readyDeadline) throw new Error('Native metrics API did not become ready');
    await wait(250);
  }
  const names = { boolean: 'Monochromatic.BrowserMetricsRepro.CurrentBoolean', sparse: 'Monochromatic.BrowserMetricsRepro.CurrentSparse' };
  const recordingWindowMs = 60000;
  const recordingCadenceMs = 1000;
  const samples = Math.floor(recordingWindowMs / recordingCadenceMs);
  record({ kind: 'test-input', targetId, names, recordingWindowMs, recordingCadenceMs, samples });
  for (let sample = 0; sample < samples; sample++) {
    await command({ method: 'Browser.getWindowForTarget', params: { targetId } });
    const expression = `chrome.metricsPrivate.recordBoolean(${JSON.stringify(names.boolean)},${sample % 2 === 0}); chrome.metricsPrivate.recordSparseValue(${JSON.stringify(names.sparse)},${sample}); 'dispatched'`;
    const evaluation = await command({ method: 'Runtime.evaluate', sessionId, params: { expression, returnByValue: true } });
    if (evaluation.exceptionDetails) throw new Error(JSON.stringify(evaluation.exceptionDetails));
    await wait(recordingCadenceMs);
  }
  for (const name of Object.values(names)) {
    const { histogram } = await command({ method: 'Browser.getHistogram', params: { name, delta: false } });
    if (histogram.count !== samples) throw new Error(`Unexpected count for ${name}: ${histogram.count}, expected ${samples}`);
  }
  record({ kind: 'complete', samples });
  console.log(JSON.stringify({ output, samples, version: metadata.Browser }));
} finally {
  socket.close();
}
```
