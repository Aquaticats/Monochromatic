/**
 Coalescing and teardown behavior of `createConfigWatcher` over a real
 config file in a disposable temp directory.
 
 @module
 */

import path from 'node:path';
import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ConfigWatcher,
  createConfigWatcher,
  type FileState,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
  writeConfigFile,
} from './test-support.ts';

/**
 Upper bound for waiting on an `onChange` call:
 on Linux the watcher polls file signatures at Node's `fs.watchFile`
 interval and then debounces one second,
 so a call can arrive several seconds after the write. The bound exits
 early when the call arrives,
 so the slack only pays on slow schedulers.
 
 @example
 ```ts
 CALL_WAIT_LIMIT; // 25000
 ```
 */
const CALL_WAIT_LIMIT = 25_000;

/**
 Polling step between bounded-wait observations.
 
 @example
 ```ts
 POLL_INTERVAL; // 100
 ```
 */
const POLL_INTERVAL = 100;

/**
 Quiet window proving no trailing `onChange` call arrives after the debounced
 burst settles.
 
 @example
 ```ts
 QUIET_WINDOW; // 3000
 ```
 */
const QUIET_WINDOW = 3_000;

/**
 Quiet window covering a whole polling interval plus debounce,
 so a close that failed to drop in-flight work would still surface a call.
 
 @example
 ```ts
 CLOSE_DROP_WINDOW; // 8000
 ```
 */
const CLOSE_DROP_WINDOW = 8_000;

/**
 One watcher fixture: watched config path,
 recorded calls,
 and the live watcher.
 
 @example
 ```ts
 const fixture = createWatcherFixture();
 fixture.watcher.close();
 ```
 */
type WatcherFixture = {
  /**
   Disposable directory holding the watched config file.
   */
  readonly directory: string;
  /**
   Absolute path of the watched config file.
   */
  readonly filePath: string;
  /**
   Timestamps recorded by `onChange` calls,
   one entry per call.
   */
  readonly calls: number[];
  /**
   Watcher under test.
   */
  readonly watcher: ConfigWatcher;
};

/**
 Creates one temp config file with a live watcher recording `onChange` calls.
 
 @returns Fixture with directory,
 config path,
 call log,
 and watcher.
 
 @example
 ```ts
 const fixture = createWatcherFixture();
 ```
 */
function createWatcherFixture(): WatcherFixture {
  /**
   Disposable directory holding the watched config file.
   */
  const directory = createTempDirectory();
  /**
   Absolute path of the watched config file.
   */
  const filePath = path.join(
    directory,
    'config.json',
  );
  writeConfigFile({
    directory,
    data: {
      run: 'baseline',
    },
  },);
  /**
   Call log filled by the watcher's `onChange`.
   */
  const calls: number[] = [];
  /**
   Watcher under test recording one timestamp per `onChange` call.
   */
  const watcher = createConfigWatcher({
    path: filePath,
    onChange: function recordChange(): void {
      calls.push(Date.now(),);
    },
  },);
  return {
    directory,
    filePath,
    calls,
    watcher,
  };
}

/**
 Waits until the recorded call count reaches the wanted count,
 polling with small sleeps until the bound runs out.
 
 @param calls - Call log recorded by the watcher fixture.
 
 @param wantedCount - Call count to wait for.
 
 @returns Whether the count was reached before the bound.
 
 @example
 ```ts
 const arrived = await waitForCallCount({ calls, wantedCount: 1, });
 ```
 */
async function waitForCallCount({
  calls,
  wantedCount,
}: {
  readonly calls: readonly unknown[];
  readonly wantedCount: number;
},): Promise<boolean> {
  /**
   Bound for this wait;
   observed calls keep arriving past it only on failure.
   */
  const deadline = Date.now() + CALL_WAIT_LIMIT;
  while (Date.now() < deadline) {
    if (calls.length >= wantedCount)
      return true;
    /* oxlint-disable-next-line eslint/no-await-in-loop -- a bounded poll must observe between sleeps, so the sleeps cannot run in parallel. */
    await delay(POLL_INTERVAL,);
  }
  return calls.length >= wantedCount;
}

await describe({
  name: createConfigWatcher.name,
  children: [
    it({
      name: 'fires onChange after another writer replaces the config file',
      fn: async () => {
        /**
         Watcher fixture under test.
         */
        const fixture = createWatcherFixture();
        writeConfigFile({
          directory: fixture.directory,
          data: {
            run: 'first-change',
          },
        },);
        /**
         Whether the watcher reported the write within the bound.
         */
        const arrived = await waitForCallCount({
          calls: fixture.calls,
          wantedCount: 1,
        },);
        fixture.watcher
          .close();
        expect(arrived,).toBe(true,);
        expect(fixture.calls,).toHaveLength(1,);
      },
    },),

    it({
      name: 'reads the watched file state before it returns, so a write made in the same tick still differs',
      fn: async () => {
        /**
         Directory whose config file is never created.
         */
        const directory = createTempDirectory();
        /**
         Path the watcher is told to watch and read.
         */
        const filePath = path.join(
          directory,
          'config.json',
        );
        /**
         Paths the injected reader was asked about, in call order.
         */
        const reads: string[] = [];
        /**
         Watcher whose reader records every baseline and poll read.
         */
        const watcher = createConfigWatcher({
          path: filePath,
          onChange: function ignoreChange(): void {
            // This case asserts the baseline read, not delivery.
          },
          readState: function recordRead(
            request: { readonly path: string; },
          ): FileState {
            reads.push(request.path,);
            return { kind: 'absent', };
          },
        },);
        // Asserted without awaiting, and before the first poll is due: the
        // baseline has to be fixed by the time the factory returns, or a write
        // the caller makes next lands before the watcher knows which file it
        // started from and is never reported.
        expect(reads,).toEqual([filePath,],);
        watcher
          .close();
      },
    },),

    it({
      name: 'reports the change a poll sees between two file-state reads',
      fn: async () => {
        /**
         Directory standing in for the watched location; no file is written.
         */
        const directory = createTempDirectory();
        /**
         Path the injected reader answers for.
         */
        const filePath = path.join(
          directory,
          'config.json',
        );
        /**
         State the reader reports once the case flips it.
         */
        const rewritten: FileState = {
          kind: 'present',
          inode: 2n,
          size: 9n,
          modifiedNanoseconds: 1n,
        };
        /**
         Whether the reader already reports the rewrite.
         */
        let isRewritten = false;
        /**
         Timestamps recorded by `onChange`.
         */
        const calls: number[] = [];
        /**
         Watcher under test,
         polling an injected reader instead of a file.
         */
        const watcher = createConfigWatcher({
          path: filePath,
          onChange: function recordChange(): void {
            calls.push(Date.now(),);
          },
          readState: function reportState(): FileState {
            return isRewritten ? rewritten : { kind: 'absent', };
          },
        },);
        isRewritten = true;
        /**
         Whether the poll reported the change within the bound.
         */
        const arrived = await waitForCallCount({
          calls,
          wantedCount: 1,
        },);
        watcher
          .close();
        expect(arrived,).toBe(true,);
        expect(calls,).toHaveLength(1,);
      },
    },),

    it({
      name: 'coalesces a burst of writes into a single onChange call',
      fn: async () => {
        /**
         Watcher fixture under test.
         */
        const fixture = createWatcherFixture();
        for (const run of [
          'burst-one',
          'burst-two',
          'burst-three',
        ])
          writeConfigFile({
            directory: fixture.directory,
            data: {
              run,
            },
          },);
        /**
         Whether the watcher reported the burst within the bound.
         */
        const arrived = await waitForCallCount({
          calls: fixture.calls,
          wantedCount: 1,
        },);
        await delay(QUIET_WINDOW,);
        fixture.watcher
          .close();
        expect(arrived,).toBe(true,);
        expect(fixture.calls,).toHaveLength(1,);
      },
    },),

    it({
      name: 'stops calling onChange after the watcher closes',
      fn: async () => {
        /**
         Watcher fixture under test.
         */
        const fixture = createWatcherFixture();
        writeConfigFile({
          directory: fixture.directory,
          data: {
            run: 'before-close',
          },
        },);
        /**
         Whether the watcher reported the pre-close write within the bound.
         */
        const arrivedBeforeClose = await waitForCallCount({
          calls: fixture.calls,
          wantedCount: 1,
        },);
        fixture.watcher
          .close();
        writeConfigFile({
          directory: fixture.directory,
          data: {
            run: 'after-close',
          },
        },);
        await delay(CLOSE_DROP_WINDOW,);
        expect(arrivedBeforeClose,).toBe(true,);
        expect(fixture.calls,).toHaveLength(1,);
      },
    },),

    it({
      name: 'drops in-flight change runs when close arrives before the call fires',
      fn: async () => {
        /**
         Watcher fixture under test.
         */
        const fixture = createWatcherFixture();
        writeConfigFile({
          directory: fixture.directory,
          data: {
            run: 'racing-close',
          },
        },);
        fixture.watcher
          .close();
        await delay(CLOSE_DROP_WINDOW,);
        expect(fixture.calls,).toHaveLength(0,);
      },
    },),

    it({
      name: 'tolerates close being called more than once',
      fn: async () => {
        /**
         Watcher fixture under test.
         */
        const fixture = createWatcherFixture();
        fixture.watcher
          .close();
        fixture.watcher
          .close();
        writeConfigFile({
          directory: fixture.directory,
          data: {
            run: 'after-double-close',
          },
        },);
        await delay(QUIET_WINDOW,);
        expect(fixture.calls,).toHaveLength(0,);
      },
    },),
  ],
},);
