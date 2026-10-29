import assert from 'node:assert/strict';

import { wait, } from '@monochromatic-dev/module-async-time/ts';

import {
  readFixtureState,
  runBypassOperation,
  runBypassOperationAllowingFailure,
  runNamespaceIp,
  runSudo,
  runSudoAllowingFailure,
  runWgQuickerCli,
  type BypassFixture,
  writeRootFixtureFile,
} from './tunnel-bypass-command-fixture.ts';
import { createBypassFixture, } from './tunnel-bypass.integration-fixture.ts';

/**
 Exempt socket mark used by all routing checks.
 */
const EXEMPT_MARK = 100;

/**
 Maximum bounded route-watcher readiness probes.
 */
const WATCH_PROBE_ATTEMPTS = 100;

/**
 Delay between route-watcher probes.
 */
const WATCH_PROBE_DELAY_MS = 10;

/**
 Adds bypass state through built artifact.
 
 @param fixture - Disposable namespace.
 
 @param watchRouteChanges - Whether detached watcher starts.
 
 @example
 ```ts
 addBypass({ fixture, watchRouteChanges: false });
 ```
 */
async function addBypass(
  {
    fixture,
    watchRouteChanges,
  }: {
    readonly fixture: BypassFixture;
    readonly watchRouteChanges: boolean;
  },
): Promise<void> {
  await runBypassOperation({
    fixture,
    source: `await addExemptRule({ interfaceName: 'wgtest', mark: ${String(EXEMPT_MARK,)}, watchRouteChanges: ${String(watchRouteChanges,)} });`,
  },);
}

/**
 Removes bypass state through built artifact.
 
 @param fixture - Disposable namespace.
 
 @example
 ```ts
 removeBypass({ fixture });
 ```
 */
async function removeBypass(
  { fixture, }: { readonly fixture: BypassFixture; },
): Promise<void> {
  await runBypassOperation({
    fixture,
    source: "await removeExemptRule({ interfaceName: 'wgtest' });",
  },);
}

/**
 Reads detached watcher PID from root-owned sidecar.
 
 @param fixture - Disposable namespace.
 
 @returns Registered watcher PID.
 
 @example
 ```ts
 watcherPid({ fixture });
 ```
 */
async function watcherPid(
  { fixture, }: { readonly fixture: BypassFixture; },
): Promise<number> {
  /**
   Parsed watcher sidecar.
   */
  const value: unknown = JSON.parse(await runSudo({
    args: [
      'cat',
      `${fixture.statePath}.watcher.json`,
    ],
  },),);
  if (((typeof value) !== 'object')
    || (value === null)
    || (!('pid' in value))
    || ((typeof value.pid) !== 'number')) {
    throw new Error('Watcher sidecar lacks numeric PID.',);
  }
  return value.pid;
}


/**
 Starts detached watcher through alternate Node executable argument.
 
 @param fixture - Disposable namespace keeping process and routes isolated.
 
 @returns Registered detached watcher PID for cleanup assertions.
 
 @example
 ```ts
 await startPriorRuntimeWatcher({ fixture });
 ```
 */
async function startPriorRuntimeWatcher(
  { fixture, }: { readonly fixture: BypassFixture; },
): Promise<number> {
  /**
   Alternate executable argument modeling prior mise-managed Node installation.
   */
  const executablePath = `${fixture.stateDirectory}/node-prior-runtime`;
  await runSudo({
    args: [
      'ln',
      '--symbolic',
      process.execPath,
      executablePath,
    ],
  },);
  /**
   Built watcher entry used by production detached process.
   */
  const watcherPath = new URL(
    '../dist/final/node/bypass-watch.mjs',
    import.meta.url,
  ).pathname;
  /**
   Detached watcher PID printed by short-lived privileged launcher.
   */
  const processIdText = await runSudo({
    args: [
      process.execPath,
      '--input-type=module',
      '--eval',
      [
        "const { spawn } = await import('node:child_process');",
        "const watcher = spawn('ip', process.argv.slice(1), { detached: true, stdio: 'ignore' });",
        'watcher.unref();',
        "if ((watcher.pid === undefined) || (watcher.pid <= 0)) throw new Error('Watcher lacks PID.');",
        'console.log(String(watcher.pid));',
      ].join(' ',),
      'netns',
      'exec',
      fixture.namespace,
      'env',
      `WG_QUICKER_RUNTIME_DIRECTORY=${fixture.stateDirectory}`,
      executablePath,
      watcherPath,
      fixture.statePath,
    ],
  },);
  /**
   Positive watcher process identifier returned to caller.
   */
  const processId = Number(processIdText.trim(),);
  if ((!Number.isSafeInteger(processId)) || (processId <= 0))
    throw new Error(`Prior-runtime watcher returned invalid PID: ${processIdText}`,);
  /**
   Bounded sidecar readiness cursor.
   */
  const cursor = { attempt: 0, };
  while (cursor.attempt < WATCH_PROBE_ATTEMPTS) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Detached watcher creates sidecar asynchronously.
    const result = await runSudoAllowingFailure({
      args: [
        'test',
        '-e',
        `${fixture.statePath}.watcher.json`,
      ],
    },);
    if (result.exitCode === 0)
      return processId;
    // oxlint-disable-next-line eslint/no-await-in-loop -- Bounded readiness wait avoids busy spin.
    await wait(WATCH_PROBE_DELAY_MS,);
    cursor.attempt += 1;
  }
  throw new Error(`Prior-runtime watcher ${String(processId,)} did not register.`,);
}

/**
 Polls active `ip monitor` child under watcher.
 
 @param watcherProcessId - Detached watcher PID.
 
 @param previousProcessId - Prior monitor PID excluded after restart.
 
 @returns Positive current monitor PID.
 
 @example
 ```ts
 await watcherMonitorPid({ watcherProcessId: 123, previousProcessId: 0 });
 ```
 */
async function watcherMonitorPid(
  {
    watcherProcessId,
    previousProcessId,
  }: {
    readonly watcherProcessId: number;
    readonly previousProcessId: number;
  },
): Promise<number> {
  /**
   Bounded child-discovery cursor.
   */
  const cursor = { attempt: 0, };
  while (cursor.attempt < WATCH_PROBE_ATTEMPTS) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Each probe observes supervised child replacement after delay.
    const result = await runSudoAllowingFailure({
      args: [
        'pgrep',
        '--parent',
        String(watcherProcessId,),
        '--exact',
        'ip',
      ],
    },);
    if (result.exitCode === 0) {
      /**
       First matching monitor child.
       */
      const pid = Number(result.stdout.trim(),);
      if (Number.isSafeInteger(pid,) && (pid > 0) && (pid !== previousProcessId))
        return pid;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- Bounded child-restart wait avoids busy spin.
    await wait(WATCH_PROBE_DELAY_MS,);
    cursor.attempt += 1;
  }
  throw new Error('Detached watcher did not expose supervised route-monitor child.',);
}

/**
 Waits until watcher table follows secondary physical interface.
 
 @param fixture - Disposable namespace.
 
 @param table - Claimed bypass table.
 
 @example
 ```ts
 await waitForSecondaryRoute({ fixture, table: 52002 });
 ```
 */
async function waitForSecondaryRoute(
  {
    fixture,
    table,
  }: {
    readonly fixture: BypassFixture;
    readonly table: number;
  },
): Promise<void> {
  /**
   Mutable bounded probe count.
   */
  const cursor = { attempt: 0, };
  while (cursor.attempt < WATCH_PROBE_ATTEMPTS) {
    /**
     Current IPv4 bypass defaults.
     */
    // oxlint-disable-next-line no-await-in-loop -- polling must observe route state after each bounded delay.
    const routes = await runNamespaceIp({
      fixture,
      args: [
        '-4',
        'route',
        'show',
        'table',
        String(table,),
      ],
    },);
    if (routes.includes(`dev ${fixture.peerSecondary}`,)
      && (!routes.includes(`dev ${fixture.peerPrimary}`,)))
      return;
    // oxlint-disable-next-line eslint/no-await-in-loop -- Bounded watcher readiness probe avoids busy spin.
    await wait(WATCH_PROBE_DELAY_MS,);
    cursor.attempt += 1;
  }
  throw new Error('Detached bypass watcher did not synchronize changed default route.',);
}

await using fixture = await createBypassFixture();

//region Dynamic ownership and full-tunnel shapes

await runBypassOperation({
  fixture,
  source: "await using firstInterfaceLock = await claimBypassInterfaceOperation({ interfaceName: 'wgtest' }); const secondInterfaceLock = await Promise.allSettled([claimBypassInterfaceOperation({ interfaceName: 'wgtest' })]); if (secondInterfaceLock[0]?.status !== 'rejected') throw new Error('Concurrent interface lock unexpectedly succeeded.'); await using firstAllocationLock = await claimBypassAllocationOperation(); const secondAllocationLock = await Promise.allSettled([claimBypassAllocationOperation()]); if (secondAllocationLock[0]?.status !== 'rejected') throw new Error('Concurrent allocation lock unexpectedly succeeded.');",
},);
await runNamespaceIp({
  fixture,
  args: [
    '-4',
    'route',
    'add',
    'blackhole',
    '203.0.113.0/24',
    'table',
    '52000',
  ],
},);
await runNamespaceIp({ fixture, args: ['-4', 'rule', 'add', 'pref', '50', 'from', '203.0.113.0/24', 'table', 'main',], },);
await runNamespaceIp({ fixture, args: ['-6', 'rule', 'add', 'pref', '51', 'from', '2001:db8:ffff::/64', 'table', 'main',], },);
await addBypass({
  fixture,
  watchRouteChanges: false,
},);
/**
 State proving occupied table and both-family preferences were skipped.
 */
const firstState = await readFixtureState({ fixture, },);
assert.equal(firstState.table, 52_001,);
assert.equal(firstState.preference, 49,);
await runNamespaceIp({ fixture, args: ['route', 'add', '0.0.0.0/1', 'dev', 'wgtest',], },);
await runNamespaceIp({ fixture, args: ['route', 'add', '128.0.0.0/1', 'dev', 'wgtest',], },);
await runNamespaceIp({ fixture, args: ['-6', 'route', 'add', '::/1', 'dev', 'wgtest',], },);
await runNamespaceIp({ fixture, args: ['-6', 'route', 'add', '8000::/1', 'dev', 'wgtest',], },);
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'get', '8.8.8.8',], },)).includes('dev wgtest',));
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'get', '8.8.8.8', 'mark', String(EXEMPT_MARK,),], },)).includes(`table ${String(firstState.table,)}`,));
assert.ok((await runNamespaceIp({ fixture, args: ['-6', 'route', 'get', '2001:4860:4860::8888',], },)).includes('dev wgtest',));
assert.ok((await runNamespaceIp({ fixture, args: ['-6', 'route', 'get', '2001:4860:4860::8888', 'mark', String(EXEMPT_MARK,),], },)).includes(`table ${String(firstState.table,)}`,));
await runNamespaceIp({ fixture, args: ['route', 'delete', '0.0.0.0/1', 'dev', 'wgtest',], },);
await runNamespaceIp({ fixture, args: ['route', 'delete', '128.0.0.0/1', 'dev', 'wgtest',], },);
await runNamespaceIp({ fixture, args: ['-6', 'route', 'delete', '::/1', 'dev', 'wgtest',], },);
await runNamespaceIp({ fixture, args: ['-6', 'route', 'delete', '8000::/1', 'dev', 'wgtest',], },);
await runNamespaceIp({ fixture, args: ['-4', 'route', 'add', 'default', 'dev', 'wgtest', 'table', '51820',], },);
await runNamespaceIp({ fixture, args: ['-6', 'route', 'add', 'default', 'dev', 'wgtest', 'table', '51820',], },);
await runNamespaceIp({ fixture, args: ['-4', 'rule', 'add', 'not', 'fwmark', '51820', 'table', '51820', 'pref', '1000',], },);
await runNamespaceIp({ fixture, args: ['-6', 'rule', 'add', 'not', 'fwmark', '51820', 'table', '51820', 'pref', '1000',], },);
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'get', '8.8.8.8',], },)).includes('table 51820',));
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'get', '8.8.8.8', 'mark', String(EXEMPT_MARK,),], },)).includes(`table ${String(firstState.table,)}`,));
assert.ok((await runNamespaceIp({ fixture, args: ['-6', 'route', 'get', '2001:4860:4860::8888',], },)).includes('table 51820',));
assert.ok((await runNamespaceIp({ fixture, args: ['-6', 'route', 'get', '2001:4860:4860::8888', 'mark', String(EXEMPT_MARK,),], },)).includes(`table ${String(firstState.table,)}`,));

//endregion Dynamic ownership and full-tunnel shapes

//region Exact teardown ownership

await runNamespaceIp({ fixture, args: ['-4', 'route', 'add', 'blackhole', '198.18.0.0/15', 'table', String(firstState.table,), 'proto', 'boot',], },);
await runNamespaceIp({ fixture, args: ['-4', 'route', 'add', 'blackhole', '192.0.2.0/24', 'table', String(firstState.table,), 'proto', '201',], },);
await removeBypass({ fixture, },);
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'show', 'table', '52000',], },)).includes('blackhole 203.0.113.0/24',));
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'show', 'table', String(firstState.table,),], },)).includes('blackhole 198.18.0.0/15',));
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'show', 'table', String(firstState.table,),], },)).includes('blackhole 192.0.2.0/24',));
assert.equal((await runNamespaceIp({ fixture, args: ['-4', 'rule', 'show',], },)).includes(`fwmark 0x22b8 lookup ${String(firstState.table,)}`,), false,);
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'rule', 'show',], },)).includes('50:',));
assert.ok((await runNamespaceIp({ fixture, args: ['-6', 'rule', 'show',], },)).includes('51:',));

//endregion Exact teardown ownership

//region Prior-runtime watcher cleanup

await addBypass({
  fixture,
  watchRouteChanges: false,
},);
/**
 Watcher launched through executable argument differing from current runtime.
 */
const priorRuntimeWatcherPid = await startPriorRuntimeWatcher({ fixture, },);
/**
 Guaranteed cleanup preserving disposable fixture when regression assertion fails.
 */
await using priorRuntimeWatcherCleanup: AsyncDisposable = {
  async [Symbol.asyncDispose](): Promise<void> {
    await runSudoAllowingFailure({
      args: [
        'kill',
        '--signal',
        'KILL',
        '--',
        `-${String(priorRuntimeWatcherPid,)}`,
      ],
    },);
  },
};
/**
 Sidecar text rewritten to legacy shape from incident before command persistence.
 */
const priorRuntimeWatcherSidecarText = await runSudo({
  args: [
    'cat',
    `${fixture.statePath}.watcher.json`,
  ],
},);
/**
 Parsed prior-runtime watcher identity used for backward-compatibility fixture.
 */
const priorRuntimeWatcherSidecar: unknown = JSON.parse(priorRuntimeWatcherSidecarText,);
if (((typeof priorRuntimeWatcherSidecar) !== 'object')
  || (priorRuntimeWatcherSidecar === null)
  || (!('ownerId' in priorRuntimeWatcherSidecar))
  || (!('pid' in priorRuntimeWatcherSidecar))
  || (!('startTime' in priorRuntimeWatcherSidecar))
  || ((typeof priorRuntimeWatcherSidecar.ownerId) !== 'string')
  || ((typeof priorRuntimeWatcherSidecar.pid) !== 'number')
  || ((typeof priorRuntimeWatcherSidecar.startTime) !== 'string')) {
  throw new Error('Prior-runtime watcher sidecar has invalid shape.',);
}
assert.equal(priorRuntimeWatcherSidecar.pid, priorRuntimeWatcherPid,);
await writeRootFixtureFile({
  path: `${fixture.statePath}.watcher.json`,
  contents: JSON.stringify({
    ownerId: priorRuntimeWatcherSidecar.ownerId,
    pid: priorRuntimeWatcherSidecar.pid,
    startTime: priorRuntimeWatcherSidecar.startTime,
  },),
},);
await removeBypass({ fixture, },);
assert.notEqual((await runSudoAllowingFailure({
  args: [
    'test',
    '-e',
    `/proc/${String(priorRuntimeWatcherPid,)}`,
  ],
},)).exitCode, 0,);

//endregion Prior-runtime watcher cleanup

//region Detached route-change watcher

await addBypass({
  fixture,
  watchRouteChanges: true,
},);
/**
 State after two deliberately occupied bypass tables.
 */
const watcherState = await readFixtureState({ fixture, },);
assert.equal(watcherState.table, 52_002,);
/**
 Detached watcher PID expected to disappear during teardown.
 */
const detachedPid = await watcherPid({ fixture, },);
/**
 Initial route-monitor child intentionally killed to verify watcher supervision.
 */
const firstMonitorPid = await watcherMonitorPid({
  watcherProcessId: detachedPid,
  previousProcessId: 0,
},);
await runSudo({
  args: [
    'kill',
    '--signal',
    'KILL',
    String(firstMonitorPid,),
  ],
},);
/**
 Replacement monitor child started by persistent watcher.
 */
const replacementMonitorPid = await watcherMonitorPid({
  watcherProcessId: detachedPid,
  previousProcessId: firstMonitorPid,
},);
assert.notEqual(replacementMonitorPid, firstMonitorPid,);
await runNamespaceIp({ fixture, args: ['route', 'replace', 'default', 'via', '198.51.101.1', 'dev', fixture.peerSecondary, 'metric', '50',], },);
await runNamespaceIp({ fixture, args: ['route', 'delete', 'default', 'via', '198.51.100.1', 'dev', fixture.peerPrimary, 'metric', '100',], },);
await runNamespaceIp({ fixture, args: ['-6', 'route', 'replace', 'default', 'via', '2001:db8:2::1', 'dev', fixture.peerSecondary, 'metric', '50',], },);
await runNamespaceIp({ fixture, args: ['-6', 'route', 'delete', 'default', 'via', '2001:db8:1::1', 'dev', fixture.peerPrimary, 'metric', '100',], },);
await waitForSecondaryRoute({
  fixture,
  table: watcherState.table,
},);
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'get', '8.8.8.8', 'mark', String(EXEMPT_MARK,),], },)).includes(`dev ${fixture.peerSecondary}`,));
assert.ok((await runNamespaceIp({ fixture, args: ['-6', 'route', 'get', '2001:4860:4860::8888', 'mark', String(EXEMPT_MARK,),], },)).includes(`dev ${fixture.peerSecondary}`,));
await runNamespaceIp({ fixture, args: ['-4', 'route', 'add', 'default', 'via', '198.51.101.1', 'dev', fixture.peerSecondary, 'metric', '900', 'table', String(watcherState.table,), 'proto', 'boot',], },);
/**
 Synchronization collision rejection for externally added default.
 */
const externalDefaultCollision = await runBypassOperationAllowingFailure({
  fixture,
  source: `await synchronizeBypassRoutes({ state: await readBypassStatePath({ path: ${JSON.stringify(fixture.statePath,)} }) });`,
},);
assert.notEqual(externalDefaultCollision.exitCode, 0,);
assert.ok(externalDefaultCollision.stderr.includes('unowned default route',));
/**
 Original watcher sidecar restored after wrong-owner cleanup probe.
 */
const originalWatcherSidecar = await runSudo({
  args: [
    'cat',
    `${fixture.statePath}.watcher.json`,
  ],
},);
/**
 Parsed sidecar used to change only owner identity.
 */
const watcherSidecarValue: unknown = JSON.parse(originalWatcherSidecar,);
if (((typeof watcherSidecarValue) !== 'object') || (watcherSidecarValue === null))
  throw new Error('Watcher sidecar is not object.',);
await writeRootFixtureFile({
  path: `${fixture.statePath}.watcher.json`,
  contents: JSON.stringify({
    ...watcherSidecarValue,
    ownerId: 'wrong-owner',
  },),
},);
/**
 Cleanup failure caused by mismatched sidecar owner.
 */
const wrongOwnerCleanup = await runBypassOperationAllowingFailure({
  fixture,
  source: "await removeExemptRule({ interfaceName: 'wgtest' });",
},);
assert.notEqual(wrongOwnerCleanup.exitCode, 0,);
assert.equal((await runSudoAllowingFailure({ args: ['test', '-e', fixture.statePath,], },)).exitCode, 0,);
assert.equal((await runSudoAllowingFailure({ args: ['test', '-e', `/proc/${String(detachedPid,)}`,], },)).exitCode, 0,);
await writeRootFixtureFile({
  path: `${fixture.statePath}.watcher.json`,
  contents: originalWatcherSidecar,
},);
await removeBypass({ fixture, },);
assert.notEqual((await runSudoAllowingFailure({ args: ['test', '-e', `/proc/${String(detachedPid,)}`,], },)).exitCode, 0,);
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'show', 'table', String(watcherState.table,),], },)).includes('metric 900',));

//endregion Detached route-change watcher

//region Missing physical defaults and single-family fail-closed route

await runNamespaceIp({ fixture, args: ['route', 'delete', 'default', 'via', '198.51.101.1', 'dev', fixture.peerSecondary, 'metric', '50',], },);
await runNamespaceIp({ fixture, args: ['-6', 'route', 'delete', 'default', 'via', '2001:db8:2::1', 'dev', fixture.peerSecondary, 'metric', '50',], },);
/**
 Expected setup failure with no physical default in either family.
 */
const missingDefault = await runBypassOperationAllowingFailure({
  fixture,
  source: `await addExemptRule({ interfaceName: 'wgtest', mark: ${String(EXEMPT_MARK,)}, watchRouteChanges: false });`,
},);
assert.notEqual(missingDefault.exitCode, 0,);
assert.ok(missingDefault.stderr.includes('no IPv4 or IPv6 physical default route exists',));
assert.notEqual((await runSudoAllowingFailure({ args: ['test', '-e', fixture.statePath,], },)).exitCode, 0,);
await runNamespaceIp({ fixture, args: ['route', 'add', 'default', 'via', '198.51.100.1', 'dev', fixture.peerPrimary, 'metric', '100',], },);
await addBypass({
  fixture,
  watchRouteChanges: false,
},);
/**
 State for IPv4-only physical connectivity.
 */
const singleFamilyState = await readFixtureState({ fixture, },);
assert.ok((await runNamespaceIp({ fixture, args: ['-6', 'route', 'show', 'table', String(singleFamilyState.table,),], },)).includes('unreachable default',));
await removeBypass({ fixture, },);

//endregion Missing physical defaults and single-family fail-closed route

//region Built CLI up and changed-config down lifecycle

await runNamespaceIp({ fixture, args: ['link', 'delete', 'wgtest',], },);
/**
 Valid private key generated inside disposable namespace.
 */
const privateKey = (await runSudo({
  args: [
    'ip',
    'netns',
    'exec',
    fixture.namespace,
    'wg',
    'genkey',
  ],
},)).trim();
/**
 Explicit config path consumed by built CLI.
 */
const configPath = `${fixture.stateDirectory}/wgtest.conf`;
await writeRootFixtureFile({
  path: configPath,
  contents: [
    '[Interface]',
    `PrivateKey = ${privateKey}`,
    'Address = 10.200.0.1/32',
    `ExemptMark = ${String(EXEMPT_MARK,)}`,
  ].join('\n',),
},);
/**
 Captured activation with configured mark and no missing-mark warning.
 */
const cliActivation = await runWgQuickerCli({
  fixture,
  operation: 'up',
  configPath,
},);
assert.equal(cliActivation.stderr.includes('has no ExemptMark',), false,);
assert.ok((await runNamespaceIp({ fixture, args: ['link', 'show', 'dev', 'wgtest',], },)).includes('wgtest',));
/**
 CLI-created bypass state and watcher identity.
 */
const cliState = await readFixtureState({ fixture, },);
const cliWatcherPid = await watcherPid({ fixture, },);
await runNamespaceIp({ fixture, args: ['-4', 'route', 'add', 'blackhole', '198.19.0.0/16', 'table', String(cliState.table,), 'proto', '201',], },);
await writeRootFixtureFile({
  path: configPath,
  contents: [
    '[Interface]',
    `PrivateKey = ${privateKey}`,
    'Address = 10.200.0.1/32',
    'ExemptMark = 9999',
  ].join('\n',),
},);
await runWgQuickerCli({
  fixture,
  operation: 'down',
  configPath,
},);
assert.notEqual((await runSudoAllowingFailure({ args: ['ip', 'netns', 'exec', fixture.namespace, 'ip', 'link', 'show', 'dev', 'wgtest',], },)).exitCode, 0,);
assert.notEqual((await runSudoAllowingFailure({ args: ['test', '-e', fixture.statePath,], },)).exitCode, 0,);
assert.notEqual((await runSudoAllowingFailure({ args: ['test', '-e', `/proc/${String(cliWatcherPid,)}`,], },)).exitCode, 0,);
assert.ok((await runNamespaceIp({ fixture, args: ['-4', 'route', 'show', 'table', String(cliState.table,),], },)).includes('blackhole 198.19.0.0/16',));

//endregion Built CLI up and changed-config down lifecycle

console.log('wg-quicker bypass integration passed',);
