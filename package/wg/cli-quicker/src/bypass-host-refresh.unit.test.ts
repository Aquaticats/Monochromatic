import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CLEARNET_HOSTS,
  FAILED_REFRESH_RETRY_SECONDS,
  runHostRefreshLoop,
} from '../dist/final/node/bypass-host-refresh.mjs';
import type { BypassState, } from '../dist/final/node/tunnel-bypass-hosts.mjs';

/**
 State claiming table 52000 and preference 50.
 */
const STATE: BypassState = {
  version: 2,
  interfaceName: 'wg0',
  mark: 100,
  table: 52_000,
  preference: 50,
  ownerId: 'owner',
  routes: [],
};

/**
 One reading of the data file the watcher follows.
 */
type Reading =
  | { readonly kind: 'read'; readonly hosts: readonly string[] }
  | { readonly kind: 'failed'; readonly failure: string };

/**
 Records what one run of the loop did.

 @param readings - What each read of the data file returns, in order;
   the last one repeats.
 @param stopAfter - How many reconciliations run before the watcher is asked to stop.

 @returns The loop promise, the hosts each reconciliation carried, the waits, and the delivered lines.

 @example
 ```ts
 await recordingLoop({ readings, stopAfter: 2 });
 ```
 */
async function recordingLoop(
  {
    readings,
    stopAfter,
  }: {
    readonly readings: readonly Reading[];
    readonly stopAfter: number;
  },
): Promise<{
  readonly reconciled: readonly (readonly string[])[];
  readonly waits: readonly number[];
  readonly delivered: readonly string[]
}> {
  /**
   Aborted in place of a signal from a person.
   */
  const controller = new AbortController();
  if (stopAfter === 0)
    controller.abort();
  /**
   Hosts each reconciliation carried.
   */
  const reconciled: (readonly string[])[] = [];
  /**
   Seconds each wait was asked for.
   */
  const waits: number[] = [];
  /**
   Lines delivered where a person sees them.
   */
  const delivered: string[] = [];
  /**
   Reads made, so the last reading repeats.
   */
  let reads = 0;
  await runHostRefreshLoop({
    state: STATE,
    signal: controller.signal,
    deliver: ({ line, },) => {
      delivered.push(line,);
      return Promise.resolve();
    },
    readHosts: () => {
      /**
       Reading for this round.
       */
      const reading = readings[Math.min(reads, readings.length - 1,)];
      if (reading === undefined)
        throw new Error('A recording loop needs at least one reading.',);
      reads += 1;
      return Promise.resolve(reading,);
    },
    refresh: ({ hosts, },) => {
      reconciled.push(hosts,);
      if (reconciled.length >= stopAfter)
        controller.abort();
      return Promise.resolve(0,);
    },
    pause: ({ seconds, },) => {
      waits.push(seconds,);
      return Promise.resolve();
    },
  },);
  return {
    reconciled,
    waits,
    delivered,
  };
}

await describe({
  name: runHostRefreshLoop.name,
  children: [
    it({
      name: 'reconciles with the list the data file holds now, not the one the watcher started with',
      fn: async () => {
        /**
         What one run of the loop did.
         */
        const run = await recordingLoop({
          readings: [
            { kind: 'read', hosts: ['crates.io',], },
            { kind: 'read', hosts: ['crates.io', 'pypi.org',], },
          ],
          stopAfter: 2,
        },);
        expect(run.reconciled,).toEqual([
          ['crates.io',],
          ['crates.io', 'pypi.org',],
        ],);
      },
    },),
    it({
      name: 'keeps the list it read last, and reports loudly, when the data file cannot be read',
      fn: async () => {
        /**
         What one run of the loop did.
         */
        const run = await recordingLoop({
          readings: [{ kind: 'failed', failure: 'the data file is not there', },],
          stopAfter: 2,
        },);
        expect(run.reconciled,).toEqual([CLEARNET_HOSTS, CLEARNET_HOSTS,],);
        expect(run.delivered.length,).toBe(2,);
        for (const line of run.delivered) {
          expect(line,).toContain('POLICY NOT IN FORCE',);
          expect(line,).toContain('the data file is not there',);
          expect(line,).toContain('stays in force',);
        }
        expect(run.waits,).toEqual([FAILED_REFRESH_RETRY_SECONDS, FAILED_REFRESH_RETRY_SECONDS,],);
      },
    },),
    it({
      name: 'applies the list of a data file that became readable again',
      fn: async () => {
        /**
         What one run of the loop did.
         */
        const run = await recordingLoop({
          readings: [
            { kind: 'failed', failure: 'the data file is not there', },
            { kind: 'read', hosts: ['api.deepseek.com',], },
          ],
          stopAfter: 2,
        },);
        expect(run.reconciled,).toEqual([CLEARNET_HOSTS, ['api.deepseek.com',],],);
        expect(run.waits,).toEqual([FAILED_REFRESH_RETRY_SECONDS, 0,],);
      },
    },),
    it({
      name: 'reads nothing and reconciles nothing once the watcher is asked to stop',
      fn: async () => {
        /**
         What one run of the loop did, over an already stopped watcher.
         */
        const run = await recordingLoop({
          readings: [{ kind: 'read', hosts: ['crates.io',], },],
          stopAfter: 0,
        },);
        expect(run.reconciled,).toEqual([],);
        expect(run.waits,).toEqual([],);
      },
    },),
  ],
},);
