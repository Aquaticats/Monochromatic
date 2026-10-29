#!/usr/bin/env node

/**
 Persistent route-change watcher for application bypass table.
 
 @module
 */

import {
  spawn as spawnChild,
  type ChildProcess,
} from 'node:child_process';
import { once, } from 'node:events';
import { createInterface, } from 'node:readline';
import { text, } from 'node:stream/consumers';

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { BypassRouteError, } from './errors.ts';
import { readBypassStatePath, } from './tunnel-bypass-state.ts';
import { registerBypassWatcher, } from './tunnel-bypass-watch-service.ts';
import { synchronizeBypassRoutes, } from './tunnel-bypass-route.ts';
import type { BypassState, } from './tunnel-bypass-types.ts';

/**
 Delay before replacing an unexpectedly exited route monitor.
 */
const MONITOR_RESTART_DELAY_MS = 100;

/**
 Sentinel representing no active route-monitor child.
 */
const MONITOR_ABSENT: unique symbol = Symbol('route monitor is absent',);

/**
 Module logger for watcher lifecycle.
 */
const l = tagged({ tag: 'bypass-watch', },);

/**
 Mutable watcher lifecycle shared by signal handler and monitor session.
 */
type WatchLifecycle = {
  monitor: ChildProcess | typeof MONITOR_ABSENT;
  requested: boolean;
};

/**
 Reports whether route-monitor line can reflect main-table default change.
 
 Events for claimed bypass table are excluded to prevent synchronization loop.
 
 @param line - One `ip monitor route` line.
 
 @param state - Claimed table identity.
 
 @returns Whether watcher should resynchronize both families.
 
 @example
 ```ts
 isPhysicalDefaultEvent({ line: 'default via 192.0.2.1', state });
 ```
 */
function isPhysicalDefaultEvent(
  {
    line,
    state,
  }: {
    readonly line: string;
    readonly state: BypassState;
  },
): boolean {
  /**
   Trimmed event with optional deletion marker removed.
   */
  const event = line.trim()
    .startsWith('Deleted ',)
    ? line.trim()
      .slice('Deleted '.length,)
    : line.trim();
  if (!event.startsWith('default ',))
    return false;
  if (event.includes(` table ${String(state.table,)}`,))
    return false;
  /**
   Table attribute when event explicitly names one.
   */
  const tableIndex = event.indexOf(' table ',);
  if (tableIndex === (-1))
    return true;
  return event.slice(tableIndex + ' table '.length,)
    .startsWith('main',);
}

/**
 Runs one route-monitor child until stop or unexpected exit.
 
 Monitor starts before synchronization,
 so events occurring during initial copy remain queued for line processing.
 
 @param state - Persisted ownership state.
 
 @param lifecycle - Shared stop request and active child.
 
 @example
 ```ts
 await runMonitorSession({ state, lifecycle });
 ```
 */
async function runMonitorSession(
  {
    state,
    lifecycle,
  }: {
    readonly state: BypassState;
    readonly lifecycle: WatchLifecycle;
  },
): Promise<void> {
  /**
   Route netlink monitor covering both address families.
   */
  const monitor = spawnChild(
    'ip',
    [
      'monitor',
      'route',
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  lifecycle.monitor = monitor;
  /**
   Spawn failures retained for explicit diagnostic.
   */
  const spawnFailureMessages: string[] = [];
  monitor.once(
    'error',
    function captureMonitorError(error: Readonly<Error>,): void {
      spawnFailureMessages[0] = error.message;
    },
  );
  /**
   Close event promise registered before synchronization and line iteration.
   */
  const closed = once(
    monitor,
    'close',
  );
  /**
   Captured monitor diagnostics consumed concurrently.
   */
  const stderr = text(monitor.stderr,);
  /**
   Line iterator over queued netlink events.
   */
  const lines = createInterface({ input: monitor.stdout, },);
  try {
    await synchronizeBypassRoutes({ state, },);
    for await (const line of lines) {
      if (!isPhysicalDefaultEvent({
        line,
        state,
      })) {
        continue;
      }
      l.debug(`physical default changed: ${line}`,);
      await synchronizeBypassRoutes({ state, },);
    }
    await closed;
  }
  catch (error) {
    monitor.kill('SIGTERM',);
    await closed;
    l.error(`route monitor session failed: ${String(error,)}`,);
    throw error;
  }
  lifecycle.monitor = MONITOR_ABSENT;
  /**
   Captured monitor diagnostics after child close.
   */
  const diagnostic = await stderr;
  /**
   First spawn failure when monitor could not start.
   */
  const [spawnFailureMessage,] = spawnFailureMessages;
  if (spawnFailureMessage !== undefined)
    throw new BypassRouteError(`Route monitor spawn failed: ${spawnFailureMessage}`,);
  if (!lifecycle.requested) {
    throw new BypassRouteError(
      `Route monitor exited unexpectedly with ${String(monitor.exitCode,)}: ${diagnostic}`,
    );
  }
}

/**
 Watches route events and replaces failed monitor children until terminated.
 
 @param state - Persisted ownership state.
 
 @param statePath - State path whose watcher sidecar signals readiness.
 
 @example
 ```ts
 await watchBypassRoutes({ state, statePath });
 ```
 */
export async function watchBypassRoutes(
  {
    state,
    statePath,
  }: {
    readonly state: BypassState;
    readonly statePath: string;
  },
): Promise<void> {
  /**
   Function-scoped logger for interface watcher.
   */
  const fl = tagged({
    tag: watchBypassRoutes.name,
    l,
  },);
  /**
   Mutable stop request and active monitor reference.
   */
  const lifecycle: WatchLifecycle = {
    monitor: MONITOR_ABSENT,
    requested: false,
  };
  /**
   Requests watcher shutdown and terminates active monitor.
   */
  function stopMonitoring(): void {
    fl.debug('received termination signal',);
    lifecycle.requested = true;
    if ((typeof lifecycle.monitor) !== 'symbol')
      lifecycle.monitor
        .kill('SIGTERM',);
  }
  process.once(
    'SIGTERM',
    stopMonitoring,
  );
  process.once(
    'SIGINT',
    stopMonitoring,
  );
  /**
   Process identity registration removed on clean or failed exit.
   */
  await using registration = await registerBypassWatcher({
    statePath,
    ownerId: state.ownerId,
  },);
  while (!lifecycle.requested) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Each monitor session must finish before supervised replacement.
      await runMonitorSession({
        state,
        lifecycle,
      },);
    }
    catch (error) {
      fl.error(`route monitor will restart after failure: ${String(error,)}`,);
    }
    if (!lifecycle.requested) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Bounded restart delay prevents failure spin.
      await wait(MONITOR_RESTART_DELAY_MS,);
    }
  }
  fl.debug('route watcher stopped',);
}

/**
 State path supplied by detached watcher launcher.
 */
const [statePath,] = process.argv
  .slice(2,);
if (statePath === undefined)
  throw new BypassRouteError('Usage: bypass-watch <state-path>',);

/**
 Validated persisted state watched by detached process.
 */
const state = await readBypassStatePath({ path: statePath, },);
await watchBypassRoutes({
  state,
  statePath,
},);
