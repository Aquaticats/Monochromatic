/**
 Watch option behavior: external change detection,
 encrypted changes,
 cache dropping on watched changes,
 and corrupt or deleted file handling.
 
 @module
 */

import {
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createConf, } from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
  writeConfigFile,
} from './test-support.ts';

/**
 Upper bound on watcher waits;
 Linux `fs.watchFile` polls about every 5007 milliseconds.
 */
const WATCH_TIMEOUT_MILLISECONDS = 25_000;

/**
 Poll interval for bounded condition waits.
 */
const POLL_INTERVAL_MILLISECONDS = 100;

/**
 Polls until the condition passes or the deadline expires,
 so watch-based tests stay bounded.
 
 @param check - Condition polled until it passes.
 
 @param timeoutMilliseconds - Upper bound on the wait.
 */
async function waitUntil({
  check,
  timeoutMilliseconds,
}: {
  readonly check: () => boolean;
  readonly timeoutMilliseconds: number;
},): Promise<void> {
  /**
   Moment the bounded wait started.
   */
  const startedAt = Date.now();
  while (!check()) {
    if ((Date.now() - startedAt) >= timeoutMilliseconds)
      throw new Error(`Condition was not met within ${timeoutMilliseconds} milliseconds`);
    /* oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling between condition checks; parallel promises cannot express one bounded polling wait. */
    await wait(POLL_INTERVAL_MILLISECONDS,);
  }
}

/**
 Watches one store for a raw `change` event,
 reporting through a flag the bounded waits poll.
 
 @param events - Store event target dispatching `change`.
 
 @returns Predicate reporting whether the change arrived.
 */
function trackChangeEvent({ events, }: {
  readonly events: EventTarget;
},): () => boolean {
  /**
   Mutable flag the listener flips and the predicate reports.
   */
  const state = {
    changeSeen: false,
  };
  events.addEventListener(
    'change',
    function markSeen(): void {
      state.changeSeen = true;
    },
  );
  return function changeArrived(): boolean {
    return state.changeSeen;
  };
}

await describe({
  name: 'watch option',
  children: [
    it({
      name: 'watches config file changes made by another process',
      fn: async () => {
        /**
         Directory shared by the watching and the writing store.
         */
        const directory = createTempDirectory();
        /**
         Store watching the config file.
         */
        const watcher = createConf({
          cwd: directory,
          watch: true,
        },);
        watcher.set({
          key: 'foo',
          value: '👾',
        },);
        /**
         Second store standing in for another process.
         */
        const writer = createConf({
          cwd: directory,
        },);
        expect(writer.get('foo',),).toBe('👾',);
        expect(writer.path,).toBe(watcher.path,);

        /**
         Changes the watcher reported for `foo`.
         */
        const history: Record<string, unknown>[] = [];
        watcher.onDidChange({
          key: 'foo',
          callback: function recordChange(change: Record<string, unknown>,): void {
            history.push(change,);
          },
        },);
        /**
         Predicate reporting the watcher's raw change event.
         */
        const changeArrived = trackChangeEvent({
          events: watcher.events,
        },);

        await wait(50,);
        writer.set({
          key: 'foo',
          value: '🐴',
        },);
        await waitUntil({
          check: changeArrived,
          timeoutMilliseconds: WATCH_TIMEOUT_MILLISECONDS,
        });

        expect(history,).toEqual([
          {
            newValue: '🐴',
            oldValue: '👾',
          },
        ],);

        watcher.closeWatcher();
        writer.closeWatcher();
      },
    },),

    it({
      name: 'watches config file changes made by a direct file write',
      fn: async () => {
        /**
         Directory whose config file is rewritten directly.
         */
        const directory = createTempDirectory();
        /**
         Store watching the config file.
         */
        const conf = createConf({
          cwd: directory,
          watch: true,
        },);
        conf.set({
          key: 'foo',
          value: '🐴',
        },);

        /**
         Changes reported for `foo`.
         */
        const history: Record<string, unknown>[] = [];
        conf.onDidChange({
          key: 'foo',
          callback: function recordChange(change: Record<string, unknown>,): void {
            history.push(change,);
          },
        },);
        /**
         Predicate reporting the watcher's raw change event.
         */
        const changeArrived = trackChangeEvent({
          events: conf.events,
        },);

        await wait(50,);
        writeConfigFile({
          directory,
          data: {
            foo: '🦄',
          },
        },);
        await waitUntil({
          check: changeArrived,
          timeoutMilliseconds: WATCH_TIMEOUT_MILLISECONDS,
        });

        expect(history,).toEqual([
          {
            newValue: '🦄',
            oldValue: '🐴',
          },
        ],);

        conf.closeWatcher();
      },
    },),

    it({
      name: 'detects encrypted changes',
      fn: async () => {
        /**
         Directory shared by the watching and the writing store.
         */
        const directory = createTempDirectory();
        /**
         Store watching the encrypted config file.
         */
        const watcher = createConf({
          cwd: directory,
          watch: true,
          encryptionKey: 'secret-key',
        },);
        /**
         Second store writing encrypted data behind the watcher.
         */
        const writer = createConf({
          cwd: directory,
          encryptionKey: 'secret-key',
        },);
        writer.set({
          key: 'foo',
          value: 'bar',
        },);

        /**
         Changes the watcher reported for `foo`.
         */
        const history: Record<string, unknown>[] = [];
        watcher.onDidChange({
          key: 'foo',
          callback: function recordChange(change: Record<string, unknown>,): void {
            history.push(change,);
          },
        },);
        /**
         Predicate reporting the watcher's raw change event.
         */
        const changeArrived = trackChangeEvent({
          events: watcher.events,
        },);

        writer.set({
          key: 'foo',
          value: 'baz',
        },);
        await waitUntil({
          check: changeArrived,
          timeoutMilliseconds: WATCH_TIMEOUT_MILLISECONDS,
        });

        expect(history,).toEqual([
          {
            newValue: 'baz',
            oldValue: 'bar',
          },
        ],);

        watcher.closeWatcher();
        writer.closeWatcher();
      },
    },),

    it({
      name: 'reports an external change to onDidChange',
      fn: async () => {
        /**
         Directory shared by the watching and the writing store.
         */
        const directory = createTempDirectory();
        /**
         Store watching the config file with its cache enabled.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          watch: true,
        },);
        conf.set({
          key: 'foo',
          value: '🐴',
        },);

        await wait(250,);
        expect(conf.get('foo',),).toBe('🐴',);

        /**
         Changes reported for `foo`.
         */
        const history: Record<string, unknown>[] = [];
        conf.onDidChange({
          key: 'foo',
          callback: function recordChange(change: Record<string, unknown>,): void {
            history.push(change,);
          },
        },);
        /**
         Predicate reporting the watcher's raw change event.
         */
        const changeArrived = trackChangeEvent({
          events: conf.events,
        },);

        await wait(50,);
        writeConfigFile({
          directory,
          data: {
            foo: '🦄',
          },
        },);
        await waitUntil({
          check: changeArrived,
          timeoutMilliseconds: WATCH_TIMEOUT_MILLISECONDS,
        });

        expect(history,).toEqual([
          {
            newValue: '🦄',
            oldValue: '🐴',
          },
        ],);

        conf.closeWatcher();
      },
    },),

    it({
      name: 'drops the cache when watch sees the config file deleted',
      fn: async () => {
        /**
         Directory whose config file is deleted behind the watcher.
         */
        const directory = createTempDirectory();
        /**
         Store whose watcher must drop the cache on deletion.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          watch: true,
        },);
        conf.set({
          key: 'foo',
          value: '🐴',
        },);
        expect(conf.get('foo',),).toBe('🐴',);

        /**
         Changes reported for `foo`.
         */
        const history: Record<string, unknown>[] = [];
        conf.onDidChange({
          key: 'foo',
          callback: function recordChange(change: Record<string, unknown>,): void {
            history.push(change,);
          },
        },);
        /**
         Predicate reporting the watcher's raw change event.
         */
        const changeArrived = trackChangeEvent({
          events: conf.events,
        },);

        await wait(50,);
        unlinkSync(conf.path,);
        await waitUntil({
          check: changeArrived,
          timeoutMilliseconds: WATCH_TIMEOUT_MILLISECONDS,
        });

        expect(history,).toEqual([
          {
            oldValue: '🐴',
          },
        ],);
        expect(conf.get('foo',),).toBeUndefined();

        conf.closeWatcher();
      },
    },),

    it({
      name: 'throws instead of serving the cache when watch sees the config file become invalid',
      fn: async () => {
        /**
         Directory whose config file is corrupted behind the watcher.
         */
        const directory = createTempDirectory();
        /**
         Store whose dropped cache must not hide the corruption.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          watch: true,
        },);
        conf.set({
          key: 'foo',
          value: '🐴',
        },);
        expect(conf.get('foo',),).toBe('🐴',);

        /**
         Predicate reporting the watcher's raw change event.
         */
        const changeArrived = trackChangeEvent({
          events: conf.events,
        },);

        await wait(50,);
        writeFileSync(
          conf.path,
          '🦄',
        );
        await waitUntil({
          check: changeArrived,
          timeoutMilliseconds: WATCH_TIMEOUT_MILLISECONDS,
        });

        expect(function readCorruptStore(): unknown {
          return conf.store;
        },).toThrow(SyntaxError,);

        conf.closeWatcher();
      },
    },),
  ],
},);
