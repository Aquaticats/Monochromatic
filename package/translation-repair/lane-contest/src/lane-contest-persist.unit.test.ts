/**
 Tests for what a bought contest outcome may be written down as, which is the
 one decision a later run's resume depends on.

 WHAT IS UNDER TEST is what the store refuses. A settled outcome is evidence a
 later run reuses without asking anyone, so an outcome the final publication
 cannot ship, or one short of quorum, must never reach the cache.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  type LaneContestOutcome,
  persistLaneContestOutcome,
} from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Exact caller abort reason the abort case stops the store with.
 */
const CALLER_STOP = new Error('caller stopped lane contest',);

/**
 Settled outcome whose three voices all backed the translate lane.
 */
const SETTLED: LaneContestOutcome = {
  choice: 'translate',
  ballots: [],
  usable: 3,
  findings: [],
};

/**
 Outcome one voice of three heard, which is short of the quorum.
 */
const UNDERSTAFFED: LaneContestOutcome = {
  choice: 'translate',
  ballots: [],
  usable: 1,
  findings: [],
};

/**
 Cache that records every write it is asked to make and resumes nothing.

 @param writes - sink for the keys and serialized outcomes written

 @returns Cache the store persists through

 @example
 ```ts
 const cache = recordingCache({ writes, },);
 ```
 */
function recordingCache(
  { writes, }: { readonly writes: { key: string; serialized: string; }[]; },
): SliceCache<LaneContestOutcome> {
  return {
    resumed: new Map<string, LaneContestOutcome>(),
    persist: async function recordWrite({ key, serialized, },): Promise<void> {
      writes.push({
        key,
        serialized,
      },);
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: persistLaneContestOutcome.name,
      children: [
        it({
          name: 'REFUSES TO PERSIST a settled outcome whose lane the final publication cannot ship, so no later run resumes it',
          fn: async () => {
            /**
             Writes the store makes under this case.
             */
            const writes: { key: string; serialized: string; }[] = [];
            const persisted = await persistLaneContestOutcome({
              key: 'cat-key-unshippable',
              outcome: SETTLED,
              cache: recordingCache({ writes, },),
              choiceMayShip: false,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
            },);
            expect(persisted,).toBe(false,);
            expect(writes,).toEqual([],);
          },
        },),
        it({
          name: 'REFUSES TO PERSIST an outcome short of the quorum, since half a panel is not an answer to keep',
          fn: async () => {
            /**
             Writes the store makes under this case.
             */
            const writes: { key: string; serialized: string; }[] = [];
            const persisted = await persistLaneContestOutcome({
              key: 'cat-key-understaffed',
              outcome: UNDERSTAFFED,
              cache: recordingCache({ writes, },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
            },);
            expect(persisted,).toBe(false,);
            expect(writes,).toEqual([],);
          },
        },),
        it({
          name: 'PERSISTS a settled quorum the final publication can ship, writing its JSON under the key',
          fn: async () => {
            /**
             Writes the store makes under this case.
             */
            const writes: { key: string; serialized: string; }[] = [];
            const persisted = await persistLaneContestOutcome({
              key: 'cat-key-settled',
              outcome: SETTLED,
              cache: recordingCache({ writes, },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
            },);
            expect(persisted,).toBe(true,);
            expect(writes,).toEqual([
              {
                key: 'cat-key-settled',
                serialized: JSON.stringify(SETTLED,),
              },
            ],);
          },
        },),
        it({
          name: 'THROWS THE CALLER\'S STOP REASON before writing anything under an aborted signal',
          fn: async () => {
            /**
             Writes the store makes under this case.
             */
            const writes: { key: string; serialized: string; }[] = [];
            /**
             Signal the caller has already stopped.
             */
            const controller = new AbortController();
            controller.abort(CALLER_STOP,);

            /**
             What the store threw, or the marker that it returned instead.
             */
            const outcome: unknown = await (async function persistUnderStop(): Promise<unknown> {
              try {
                return await persistLaneContestOutcome({
                  key: 'cat-key-stopped',
                  outcome: SETTLED,
                  cache: recordingCache({ writes, },),
                  signal: controller.signal,
                },);
              }
              catch (error) {
                return error;
              }
            })();
            expect(outcome,).toBe(CALLER_STOP,);
            expect(writes,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
