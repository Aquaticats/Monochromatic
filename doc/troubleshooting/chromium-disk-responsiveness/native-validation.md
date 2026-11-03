# Chromium metrics candidate validation drivers

These sources exercise the [candidate patch](reporting-ineligible-storage.patch)
against the compiled Chromium browser and native test executables.
The [verification record](reporting-ineligible-storage-verification.json)
contains the successful native tests,
failing control,
restored-candidate retest,
and live browser results.
The [candidate status](reporting-ineligible-storage.md) records their scope and limits.

The container-owned checkout is `/work/check/src`,
the build directory is `out/metrics`,
and logs are under `/work/logs`.
The source revision is `360d943d418b3d6645fea86766f5dcc4a7cc3b2e`.
A fresh run applies the delivered formatted patch to the recorded upstream revision
and builds the browser and native test executables using the recorded GN arguments.
Keep the checkout HEAD at that upstream revision:
the guard-control driver reads its original startup file from HEAD.
The final-source driver records the formatting transition used during this validation.
Run the native tests,
then the failing startup control,
then the live browser check on the restored candidate.
The failing control replaces only the startup file with its unmodified source,
requires the persistent-location assertion to fail,
then restores and rebuilds the candidate before rerunning the native tests.
A forcibly terminated control requires checking its saved candidate source before resuming.

All execution uses disposable container state with external networking disabled.
The live browser uses a container-owned X display and no browser account.
Its first launch is the forced-reporting positive control for the mapping probe.
The normal and recording-only launches follow only after that control passes.
The browser driver uses the [retained recording client](live-histogram-reproduction.md)
as `/work/recording-client.ts`.
The test evaluates backing selection and sample counts;
it does not benchmark latency or inject filesystem stalls.

## finalize-source.ts

Source SHA-256:
`ab770bf63817efbaecf81efe805dd51c50ab9cba5f9ba53ca8ffbd247f4b28ca`.

```typescript
// /work/finalize-source.ts
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { finished } from 'node:stream/promises';

//region Apply the formatter's reviewed delta only after the original build has ended.
const source = '/work/check/src';
const directory = await mkdtemp('/work/logs/final-source-');
const revision = '360d943d418b3d6645fea86766f5dcc4a7cc3b2e';
const environment = { ...process.env, TMPDIR: '/work/tmp', DEPOT_TOOLS_UPDATE: '0',
  DEPOT_TOOLS_METRICS: '0', PATH: `/work/depot_tools:${process.env.PATH}` };
class FinalSourceError extends Error {}

/** Retain each actual command result and return its complete output for small checks. */
async function run({ name, command, args, capture = false }: {
  name: string; command: string; args: string[]; capture?: boolean;
}): Promise<string> {
  const started = Date.now();
  const path = `${directory}/${name}.log`;
  const log = createWriteStream(path);
  console.log(JSON.stringify({ event: 'step_start', name, at: new Date(started).toISOString() }));
  const child = spawn(command, args, { cwd: source, env: environment, stdio: ['ignore', 'pipe', 'pipe'] });
  child.stdout.pipe(log, { end: false });
  child.stderr.pipe(log, { end: false });
  const [code, signal] = await once(child, 'close');
  log.end();
  await finished(log);
  const record = { name, command, args, code, signal, elapsedMs: Date.now() - started,
    at: new Date().toISOString() };
  await writeFile(`${directory}/${name}.json`, JSON.stringify(record, null, 2) + '\n');
  console.log(JSON.stringify({ event: 'step_end', ...record }));
  if (code !== 0 || signal) {
    process.stderr.write((await readFile(path, 'utf8')).slice(-24000));
    throw new FinalSourceError(`${name}: exit ${code}, signal ${signal}`);
  }
  return capture ? await readFile(path, 'utf8') : '';
}

const actual = await run({ name: 'source-revision', command: 'git', args: ['rev-parse', 'HEAD'], capture: true });
assert.equal(actual.trim(), revision, 'Source revision changed before finalization');
await run({ name: 'formatting-check', command: 'git', args: ['apply', '--check', '-p0', '/work/formatting.patch'] });
await run({ name: 'formatting-apply', command: 'git', args: ['apply', '-p0', '/work/formatting.patch'] });
await run({ name: 'source-identity', command: 'sha256sum', args: ['--check', '/work/candidate-source.sha256'] });
const sourceChecksums = await readFile('/work/candidate-source.sha256', 'utf8');
const expectedPaths = sourceChecksums.trim().split('\n').map(line => {
  const match = /^[a-f0-9]{64}  (.+)$/u.exec(line);
  assert(match, 'Malformed source checksum entry');
  return match[1];
});
const changed = await run({ name: 'changed-paths', command: 'git',
  args: ['diff', '--name-only', '-z', 'HEAD'], capture: true });
const actualPaths = changed.split('\0').filter(Boolean);
assert.deepEqual(actualPaths.toSorted(), expectedPaths.toSorted(), 'Build checkout has unexpected source changes');
await run({ name: 'patch-whitespace', command: 'git', args: ['diff', '--check'] });
const style = await run({ name: 'chromium-style', command: 'git',
  args: ['cl', 'format', '--dry-run', '--diff', '--upstream=HEAD'], capture: true });
assert.equal(style.trim(), '', 'Formatter still produced output after applying its delta');
const patch = await readFile('/work/candidate-final.patch');
const inputs = { revision, candidatePatchSha256: createHash('sha256').update(patch).digest('hex'),
  sourceChecksums,
  gnArguments: await readFile(`${source}/out/metrics/args.gn`, 'utf8') };
await writeFile(`${directory}/inputs.json`, JSON.stringify(inputs, null, 2) + '\n');
await run({ name: 'final-native-build', command: '/work/depot_tools/autoninja',
  args: ['-C', 'out/metrics', '-j', '28', 'chrome', 'chrome/test:unit_tests', 'chrome/test:browser_tests'] });
await writeFile('/work/logs/latest-final-source.json', JSON.stringify({ directory,
  completedAt: new Date().toISOString(), ...inputs }, null, 2) + '\n');
console.log(JSON.stringify({ event: 'final_build_complete', directory }));
//endregion
```

## native-test.ts

Source SHA-256:
`0c7c452e975763a404210f0f1b39686664581b73b4aae0f7b73f08128a8165ce`.

```typescript
// /work/native-test.ts
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createWriteStream } from 'node:fs';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { finished } from 'node:stream/promises';
import { withDisplay } from './display.ts';

//region Native test execution with no account data or external networking.
const source = '/work/check/src';
const output = `${source}/out/metrics`;
await mkdir('/work/logs', { recursive: true });
const directory = await mkdtemp('/work/logs/native-tests-');
const expectedFailure = process.argv[2] === 'expect-guard-failure';
assert(!process.argv[2] || expectedFailure, 'Unknown native-test mode');
assert.notEqual(process.getuid?.(), 0, 'Native tests must run as the isolated build user');
class NativeTestError extends Error {}

/** Read actual test observations, rejecting absent or malformed suite output. */
function statuses(text: string): Map<string, string[]> {
  const value: unknown = JSON.parse(text);
  if (!value || typeof value !== 'object' || !('per_iteration_data' in value)
    || !Array.isArray(value.per_iteration_data)) throw new NativeTestError('Missing native test iterations');
  const results = new Map<string, string[]>();
  for (const iteration of value.per_iteration_data) {
    if (!iteration || typeof iteration !== 'object' || Array.isArray(iteration)) throw new NativeTestError('Invalid native test iteration');
    for (const [name, observations] of Object.entries(iteration)) {
      if (!Array.isArray(observations) || observations.length === 0) throw new NativeTestError(`Missing observations for ${name}`);
      for (const observation of observations) {
        if (!observation || typeof observation !== 'object' || !('status' in observation)
          || typeof observation.status !== 'string') throw new NativeTestError(`Invalid observation for ${name}`);
        const entries = results.get(name) ?? [];
        entries.push(observation.status);
        results.set(name, entries);
      }
    }
  }
  if (results.size === 0) throw new NativeTestError('Native test launcher ran no tests');
  return results;
}

/** Verify the binary emitted every selected source test and the intended result. */
async function run({ name, binary, filter, expectedNames, display, failure = false }: {
  name: string; binary: string; filter: string; expectedNames: string[]; display: string; failure?: boolean;
}): Promise<void> {
  assert(expectedNames.length > 0, `No expected tests found for ${name}`);
  const summary = `${directory}/${name}.summary.json`;
  const log = createWriteStream(`${directory}/${name}.log`);
  const args = [
    `--gtest_filter=${filter}`, '--test-launcher-retry-limit=0', '--no-sandbox',
    `--test-launcher-summary-output=${summary}`,
  ];
  const child = spawn(`${output}/${binary}`, args, {
    cwd: source, env: { ...process.env, DISPLAY: display, TMPDIR: '/work/tmp' },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  child.stdout.pipe(log, { end: false });
  child.stderr.pipe(log, { end: false });
  const [code, signal] = await once(child, 'close');
  log.end();
  await finished(log);
  if (signal || (!failure && code !== 0) || (failure && code === 0)) {
    process.stderr.write((await readFile(`${directory}/${name}.log`, 'utf8')).slice(-24000));
    throw new NativeTestError(`${name}: unexpected process result ${code}, signal ${signal}`);
  }
  const observed = statuses(await readFile(summary, 'utf8'));
  const expectedStatus = failure ? 'FAILURE' : 'SUCCESS';
  for (const test of expectedNames) {
    const entries = observed.get(test);
    if (!entries || !entries.every(status => status === expectedStatus)) {
      throw new NativeTestError(`${test}: expected ${expectedStatus}, observed ${JSON.stringify(entries)}`);
    }
  }
  for (const [test, entries] of observed) {
    if (!entries.every(status => status === expectedStatus)) throw new NativeTestError(`${test}: ${JSON.stringify(entries)}`);
  }
  const outputText = await readFile(`${directory}/${name}.log`, 'utf8');
  const persistenceFailure = /Value of: allocator->HasPersistentLocation\(\)\s+Actual: true\s+Expected: false/u.exec(outputText)?.[0];
  if (failure && !persistenceFailure) {
    throw new NativeTestError(`${name}: failed without the intended persistent-location assertion`);
  }
  const record = { at: new Date().toISOString(), name, binary, args, code, signal,
    expectedStatus, expectedNames, observed: Object.fromEntries(observed),
    ...(failure ? { persistenceFailure } : {}) };
  await writeFile(`${directory}/${name}.verified.json`, JSON.stringify(record, null, 2) + '\n');
  console.log(JSON.stringify(record));
}

await withDisplay({ directory, run: async display => {
  const accessorSource = await readFile(`${source}/chrome/browser/metrics/chrome_metrics_service_accessor_unittest.cc`, 'utf8');
  const accessorTests = [...accessorSource.matchAll(/TEST_F\(\s*ChromeMetricsServiceAccessorTest,\s*(\w+)\s*\)/gu)]
    .map(match => `ChromeMetricsServiceAccessorTest.${match[1]}`);
  const browserSource = await readFile(`${source}/chrome/browser/metrics/metrics_service_browsertest.cc`, 'utf8');
  const browserTests = [...browserSource.matchAll(/IN_PROC_BROWSER_TEST_F\(\s*(NonReportingPersistentHistogramsBrowserTest|MetricsServiceBrowserDoUploadTest|MetricsServiceBrowserNoUploadTest|MetricsServiceBrowserSampledOutTest),\s*(\w+)\s*\)/gu)]
    .map(match => `${match[1]}.${match[2]}`);
  const guardTests = browserTests.filter(name => name.startsWith('NonReportingPersistentHistogramsBrowserTest.'));
  if (expectedFailure) {
    await run({ name: 'startup-guard-control', binary: 'browser_tests',
      filter: 'NonReportingPersistentHistogramsBrowserTest.*', expectedNames: guardTests, display, failure: true });
  } else {
    await run({ name: 'accessor-policy', binary: 'unit_tests',
      filter: 'ChromeMetricsServiceAccessorTest.*', expectedNames: accessorTests, display });
    await run({ name: 'startup-and-file-policy', binary: 'browser_tests',
      filter: 'NonReportingPersistentHistogramsBrowserTest.*:MetricsServiceBrowserDoUploadTest.*:MetricsServiceBrowserNoUploadTest.*:MetricsServiceBrowserSampledOutTest.*',
      expectedNames: browserTests, display });
  }
  await writeFile('/work/logs/latest-native-tests.json', JSON.stringify({ directory, expectedFailure, completedAt: new Date().toISOString() }, null, 2) + '\n');
} });
//endregion
```

## browser-check.ts

Source SHA-256:
`ea5cac195b7caa3607f9ebb24f80795ffd474ff092294f4059de2063cc0ec252`.

```typescript
// /work/browser-check.ts
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { mkdir, mkdtemp, readFile, readdir, readlink, symlink, unlink, writeFile } from 'node:fs/promises';
import { homedir, networkInterfaces } from 'node:os';
import { join } from 'node:path';
import { finished } from 'node:stream/promises';
import { setTimeout as wait } from 'node:timers/promises';
import { withDisplay } from './display.ts';

//region Exercise the compiled browser through the previously verified WebUI client.
const source = '/work/check/src';
const binary = `${source}/out/metrics/chrome`;
await mkdir('/work/logs', { recursive: true });
const directory = await mkdtemp('/work/logs/live-browser-');
const clientParent = join(homedir(), 'temp/agent');
const clientBase = join(clientParent, 'chromium-live-repro-1733707');
await mkdir(clientParent, { recursive: true });
const client = '/work/recording-client.ts';
const clientSha256 = createHash('sha256').update(await readFile(client)).digest('hex');
const interfaces = Object.values(networkInterfaces()).flatMap(entries => entries ?? []);
assert(interfaces.every(entry => entry.internal), 'Forced-reporting verification requires an isolated network');
assert.notEqual(process.getuid?.(), 0, 'Browser verification must run as the isolated user');
class BrowserCheckError extends Error {}

/** Await the endpoint written by the real browser within a bounded startup window. */
async function endpoint(profile: string): Promise<{ Browser: string; webSocketDebuggerUrl: string }> {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const names = await readdir(profile);
    if (names.includes('DevToolsActivePort')) {
      const [port, path] = (await readFile(join(profile, 'DevToolsActivePort'), 'utf8')).split('\n');
      if (port && /^\d+$/u.test(port) && path?.startsWith('/devtools/browser/')) {
        const response = await fetch(`http://127.0.0.1:${port}/json/version`, { signal: AbortSignal.timeout(30_000) });
        if (!response.ok) throw new BrowserCheckError(`DevTools metadata: HTTP ${response.status}`);
        const value = await response.json();
        if (typeof value.Browser !== 'string' || typeof value.webSocketDebuggerUrl !== 'string') {
          throw new BrowserCheckError('Invalid browser metadata');
        }
        return value;
      }
    }
    await wait(100);
  }
  throw new BrowserCheckError('Browser did not publish its DevTools endpoint before the startup deadline');
}

/** Send the same acknowledged Browser.close operation used by the lifecycle reproduction. */
async function closeBrowser(url: string): Promise<void> {
  const socket = new WebSocket(url);
  try {
    await new Promise<void>((resolve, reject) => {
      socket.addEventListener('open', () => resolve(), { once: true });
      socket.addEventListener('error', event => reject(new BrowserCheckError(String(event))), { once: true });
    });
    const requestId = 1;
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      await new Promise<void>((resolve, reject) => {
        timer = setTimeout(() => reject(new BrowserCheckError('Browser.close acknowledgement timed out')), 30_000);
        socket.addEventListener('message', event => {
          const response = JSON.parse(String(event.data));
          if (response.id !== requestId) return;
          if (response.error) reject(new BrowserCheckError(JSON.stringify(response.error)));
          else resolve();
        });
        socket.send(JSON.stringify({ id: requestId, method: 'Browser.close' }));
      });
    } finally { clearTimeout(timer); }
  } finally { socket.close(); }
}

/** Retain only the backing property, excluding process addresses and fixture identifiers. */
async function histogramMappings(pid: number): Promise<Array<{ permissions: string; deleted: boolean }>> {
  const command = (await readFile(`/proc/${pid}/cmdline`, 'utf8')).split('\0');
  assert(!command.some(argument => argument.startsWith('--type=')), 'Selected process is a browser subprocess');
  const mappings = (await readFile(`/proc/${pid}/maps`, 'utf8')).split('\n');
  return mappings.filter(line => line.includes('/BrowserMetrics/') && line.includes('.pma'))
    .map(line => ({ permissions: line.trim().split(/\s+/u)[1], deleted: line.endsWith(' (deleted)') }));
}

/** Run the retained recording client and require its actual completion record. */
async function recordSamples(fixture: string): Promise<{ samples: number; histograms: unknown[] }> {
  const log = createWriteStream(join(fixture, 'client.log'));
  const child = spawn(process.execPath, [client], { cwd: '/work', stdio: ['ignore', 'pipe', 'pipe'] });
  child.stdout.pipe(log, { end: false });
  child.stderr.pipe(log, { end: false });
  const [code, signal] = await once(child, 'close');
  log.end();
  await finished(log);
  if (code !== 0 || signal) throw new BrowserCheckError(`Recording client exited ${code}, signal ${signal}`);
  const events = (await readFile(join(fixture, 'recording-probe.jsonl'), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
  const complete = events.find(event => event.kind === 'complete');
  const input = events.find(event => event.kind === 'test-input');
  if (!complete || !input) throw new BrowserCheckError('Recording client omitted its input or completion record');
  const histograms = events.filter(event => event.kind === 'command' && event.method === 'Browser.getHistogram')
    .map(event => event.result.histogram);
  assert.equal(histograms.length, Object.keys(input.names).length);
  assert(histograms.every(histogram => histogram.count === complete.samples));
  return { samples: complete.samples, histograms };
}

const modes = [
  { name: 'forced-reporting-control', flags: ['--force-enable-metrics-reporting'], mapped: true },
  { name: 'reporting-ineligible', flags: [], mapped: false },
  { name: 'recording-only', flags: ['--metrics-recording-only'], mapped: false },
];
const results: object[] = [];
let mappingControlPassed = false;
await withDisplay({ directory, run: async display => {
  for (const mode of modes) {
    if (!mode.mapped) assert(mappingControlPassed, 'Mapping positive control must pass first');
    const fixture = await mkdtemp(`${directory}/${mode.name}-`);
    const profile = join(fixture, 'profile');
    await mkdir(profile);
    await symlink(fixture, clientBase, 'dir');
    const stdout = createWriteStream(join(fixture, 'browser.stdout.log'));
    const stderr = createWriteStream(join(fixture, 'browser.stderr.log'));
    const args = [
      `--user-data-dir=${profile}`, '--remote-debugging-port=0', '--no-first-run', '--no-default-browser-check',
      '--no-sandbox', '--disable-background-networking', '--disable-gpu', '--ozone-platform=x11',
      ...mode.flags, 'about:blank',
    ];
    const browser = spawn(binary, args, {
      cwd: source, env: { ...process.env, DISPLAY: display, TMPDIR: '/work/tmp',
        CHROME_CONFIG_HOME: join(fixture, 'config'), XDG_CACHE_HOME: join(fixture, 'cache') },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    browser.stdout.pipe(stdout, { end: false });
    browser.stderr.pipe(stderr, { end: false });
    const exit = once(browser, 'close');
    let exitObserved = false;
    try {
      if (!browser.pid) throw new BrowserCheckError('Browser launch did not provide a process identifier');
      const metadata = await endpoint(profile);
      const before = await histogramMappings(browser.pid);
      assert.equal(before.length > 0, mode.mapped, `${mode.name}: wrong initial backing`);
      const recording = await recordSamples(fixture);
      const after = await histogramMappings(browser.pid);
      assert.equal(after.length > 0, mode.mapped, `${mode.name}: wrong backing after recording`);
      await closeBrowser(metadata.webSocketDebuggerUrl);
      let forcedTermination = false;
      let force: ReturnType<typeof setTimeout> | undefined;
      const deadline = setTimeout(() => {
        forcedTermination = true;
        browser.kill('SIGTERM');
        force = setTimeout(() => browser.kill('SIGKILL'), 5000);
      }, 30_000);
      const [code, signal] = await exit;
      clearTimeout(deadline);
      clearTimeout(force);
      exitObserved = true;
      stdout.end();
      stderr.end();
      await Promise.all([finished(stdout), finished(stderr)]);
      const diagnostics = await readFile(join(fixture, 'browser.stderr.log'), 'utf8');
      const fatalDiagnostic = /FATAL|NOTREACHED|Did not receive ping from zygote child/u.test(diagnostics);
      if (code !== 0 || signal || fatalDiagnostic || forcedTermination) throw new BrowserCheckError(`${mode.name}: browser shutdown failed: ${code}, ${signal}, fatal=${fatalDiagnostic}, forced=${forcedTermination}`);
      const record = { mode: mode.name, version: metadata.Browser, expectedMapped: mode.mapped,
        before, after, recording, exit: { code, signal }, fatalDiagnostic, browserCloseAcknowledged: true };
      results.push(record);
      console.log(JSON.stringify({ mode: mode.name, mappingCount: after.length, nativeRecording: 'passed', cleanExit: true }));
      if (mode.mapped) mappingControlPassed = true;
    } finally {
      try {
        if (!exitObserved) {
          const force = setTimeout(() => browser.kill('SIGKILL'), 5000);
          browser.kill('SIGTERM');
          await exit;
          clearTimeout(force);
          stdout.end();
          stderr.end();
          await Promise.all([finished(stdout), finished(stderr)]);
        }
      } finally {
        assert.equal(await readlink(clientBase), fixture, 'Recording-client fixture changed unexpectedly');
        await unlink(clientBase);
      }
    }
  }
} });
const result = { at: new Date().toISOString(), clientSha256, mappingControlPassed, network: 'loopback-only', results };
await writeFile(join(directory, 'verified.json'), JSON.stringify(result, null, 2) + '\n');
await writeFile('/work/logs/latest-live-browser.json', JSON.stringify({ directory, completedAt: result.at }, null, 2) + '\n');
console.log(JSON.stringify({ artifact: join(directory, 'verified.json'), modes: results.length }));
//endregion
```

## display.ts

Source SHA-256:
`133e59a77e6dbcea94d206a4ded36ff1a11c1d309a695cae6eea51cbef31918a`.

```typescript
// /work/display.ts
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createWriteStream } from 'node:fs';
import { createInterface } from 'node:readline';
import { Readable } from 'node:stream';
import { finished } from 'node:stream/promises';

//region Container-owned display lifetime, independent of the user's desktop.
class DisplayError extends Error {}

/** Wait for Xvfb's readiness descriptor before launching a browser. */
async function displayNumber(pipe: Readable): Promise<string> {
  const lines = createInterface({ input: pipe, signal: AbortSignal.timeout(30_000) });
  try {
    for await (const line of lines) {
      if (!/^\d+$/u.test(line)) throw new DisplayError(`Invalid Xvfb display: ${line}`);
      return `:${line}`;
    }
  } finally { lines.close(); }
  throw new DisplayError('Xvfb did not provide a ready display');
}

/** Retain display diagnostics and stop the owned server on every exit path. */
export async function withDisplay({ directory, run }: {
  directory: string; run: (display: string) => Promise<void>;
}): Promise<void> {
  const displayDescriptor = 3;
  const log = createWriteStream(`${directory}/xvfb.log`);
  const server = spawn('Xvfb', ['-displayfd', String(displayDescriptor), '-screen', '0', '1280x720x24', '-nolisten', 'tcp'], {
    cwd: '/work', stdio: ['ignore', 'ignore', 'pipe', 'pipe'],
  });
  server.stderr?.pipe(log, { end: false });
  const serverExit = once(server, 'close');
  try {
    const pipe = server.stdio[displayDescriptor];
    if (!(pipe instanceof Readable)) throw new DisplayError('Missing Xvfb readiness pipe');
    await run(await displayNumber(pipe));
  } finally {
    server.kill('SIGTERM');
    await serverExit;
    log.end();
    await finished(log);
  }
}
//endregion
```

## guard-control.ts

Source SHA-256:
`ffd8201f8bbac6c218657134d34acc57166890d56704b929c921d39bf51a8dde`.

```typescript
// /work/guard-control.ts
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { finished } from 'node:stream/promises';

//region Remove only the candidate's startup choice, then restore it on every exit path.
const source = '/work/check/src';
const startupPath = 'chrome/app/chrome_main_delegate.cc';
const startup = `${source}/${startupPath}`;
const directory = await mkdtemp('/work/logs/startup-control-');
const environment = { ...process.env, TMPDIR: '/work/tmp', DEPOT_TOOLS_UPDATE: '0',
  DEPOT_TOOLS_METRICS: '0', PATH: `/work/depot_tools:${process.env.PATH}` };
class GuardControlError extends Error {}

/** Retain the native build or test boundary before deciding whether to continue. */
async function run({ name, command, args }: { name: string; command: string; args: string[] }): Promise<void> {
  const started = Date.now();
  const logPath = `${directory}/${name}.log`;
  const log = createWriteStream(logPath);
  const child = spawn(command, args, { cwd: source, env: environment, stdio: ['ignore', 'pipe', 'pipe'] });
  child.stdout.pipe(log, { end: false });
  child.stderr.pipe(log, { end: false });
  const [code, signal] = await once(child, 'close');
  log.end();
  await finished(log);
  const result = { name, command, args, code, signal, elapsedMs: Date.now() - started,
    at: new Date().toISOString() };
  await writeFile(`${directory}/${name}.json`, JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify(result));
  if (code !== 0 || signal) {
    process.stderr.write((await readFile(logPath, 'utf8')).slice(-24000));
    throw new GuardControlError(`${name}: exit ${code}, signal ${signal}`);
  }
}

/** Rebuild the affected browser and regression executable without reacquiring dependencies. */
async function rebuild(name: string): Promise<void> {
  await run({ name, command: '/work/depot_tools/autoninja',
    args: ['-C', 'out/metrics', '-j', '28', 'chrome', 'chrome/test:browser_tests'] });
}

const candidate = await readFile(startup, 'utf8');
assert(candidate.includes('ChromeMetricsServiceAccessor::IsMetricsReportingSupported()'),
  'Startup source does not contain the expected candidate');
await writeFile(`${directory}/candidate-startup.cc`, candidate);
await run({ name: 'original-startup', command: 'git', args: ['show', `HEAD:${startupPath}`] });
const original = await readFile(`${directory}/original-startup.log`, 'utf8');
assert(original.includes('/*storage=*/kPersistentHistogramStorageMappedFile'), 'Unexpected original startup source');
assert(!original.includes('ChromeMetricsServiceAccessor::IsMetricsReportingSupported()'),
  'Original startup source already contains the candidate');
await writeFile(`${directory}/inputs.json`, JSON.stringify({ startupPath,
  candidateSha256: createHash('sha256').update(candidate).digest('hex'),
  originalSha256: createHash('sha256').update(original).digest('hex') }, null, 2) + '\n');
let controlPassed = false;
try {
  await writeFile(startup, original);
  await rebuild('baseline-rebuild');
  await run({ name: 'baseline-regression', command: process.execPath,
    args: ['/work/native-test.ts', 'expect-guard-failure'] });
  controlPassed = true;
} finally {
  await writeFile(startup, candidate);
  await rebuild('candidate-restored-rebuild');
  assert.equal(await readFile(startup, 'utf8'), candidate, 'Candidate restoration changed its bytes');
}
assert(controlPassed, 'Baseline regression did not fail at the intended assertion');
await run({ name: 'candidate-restored-tests', command: process.execPath, args: ['/work/native-test.ts'] });
await writeFile('/work/logs/latest-startup-control.json', JSON.stringify({ directory, controlPassed,
  candidateRestored: true, completedAt: new Date().toISOString() }, null, 2) + '\n');
//endregion
```
