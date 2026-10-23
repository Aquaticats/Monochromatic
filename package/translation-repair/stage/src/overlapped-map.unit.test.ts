/**
 Tests for running items with a bounded number in flight.

 WHAT THESE PIN: at overlap 1 the helper is the sequential loop it replaced;
 at a higher overlap items start in item order and never more than the
 overlap at once; results come back in item order whatever order the items
 finished in; a failure stops further items from starting, lets the ones in
 flight finish, and throws the lowest position's error; every other failure
 is written in position order, whether or not the caller has stopped, except
 the caller's reason and a failure a lower writer already wrote and marked;
 and an overlap that cannot bound anything is refused before any item runs.

 Every item waits at a gate the test opens, so completion order is the
 test's to choose rather than the scheduler's.

 @module
 */

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import { modelDarkRefusal, } from '@monochromatic-dev/translation-repair-provider/ts/model-dark-refusal.test-fixture.ts';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  mapOverlapped,
  OverlapRefusedError,
  reuseTwinOrBuy,
  type TwinMemo,
  type TwinStored,
} from '../dist/final/node/index.mjs';
import {
  capturingLoggerPair,
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

//region Fixtures
// Gated items: each item's job records that it started, counts itself in
// flight, and returns once the test opens its gate.

/**
 Items every case runs over, in the order results must come back.
 */
const ITEMS = [
  'a',
  'b',
  'c',
  'd',
] as const;

/**
 What one gated run exposes to the assertions.
 */
type Gated = {
  /**
   Positions in the order their jobs started.
   */
  readonly started: readonly number[];

  /**
   Most jobs seen in flight at once.
   */
  readonly inFlight: { readonly peak: number; };

  /**
   One gate per item; a job returns once its gate is opened, or throws once
   its gate is broken.
   */
  readonly gates: readonly PromiseWithResolvers<string>[];

  /**
   Job handed to the helper.
   */
  readonly oneItem: (row: {
    readonly item: string;
    readonly position: number;
  },) => Promise<string>;
};

/**
 Builds the gated job and the records it fills.

 @param count - how many gates to build, one per item

 @returns Records and the job

 @example
 ```ts
 const run = gatedRun({ count: 3, },);
 ```
 */
function gatedRun({ count, }: { readonly count: number; },): Gated {
  /**
   Positions in start order, filled by the job.
   */
  const started: number[] = [];

  /**
   Jobs in flight now and the most seen at once.
   */
  const inFlight = {
    now: 0,
    peak: 0,
  };

  /**
   One gate per item.
   */
  const gates = Array.from(
    { length: count, },
    function openGate(): PromiseWithResolvers<string> {
      return Promise.withResolvers<string>();
    },
  );
  return {
    started,
    inFlight,
    gates,
    oneItem: async function waitsAtGate({
      item,
      position,
    },): Promise<string> {
      started.push(position,);
      inFlight.now += 1;
      inFlight.peak = Math.max(
        inFlight.peak,
        inFlight.now,
      );

      /**
       What the test opened this gate with.
       */
      const answer = await nonNullishOrThrow(gates[position],).promise;
      inFlight.now -= 1;
      return `${item}:${answer}`;
    },
  };
}

/**
 Lets every settled continuation run, so the records reflect what the
 scheduler did with what the test just opened.

 @example
 ```ts
 await settle();
 ```
 */
async function settle(): Promise<void> {
  await wait(0,);
}

/**
 Resolves with what a run threw, or `undefined` when it finished.

 ATTACHED BEFORE ANY GATE IS OPENED in every failing case, so the rejection
 is never unhandled between the moment it happens and the assertion.

 @param run - mapping under test

 @returns What it threw

 @example
 ```ts
 const failure = collected({ run, },);
 ```
 */
async function collected({ run, }: { readonly run: Promise<unknown>; },): Promise<unknown> {
  try {
    await run;
    return undefined;
  }
  catch (error) {
    return error;
  }
}

/**
 Caller's signal for the cases where nobody stops the work.
 */
const NEVER_STOPPED = new AbortController().signal;

/**
 Logger for the cases that read no line.
 */
const QUIET = capturingLoggerPair().logger;

/**
 The twin memo's line for the second cat's buy refused with the provider's
 words, the one line such a failure gets with the caller stopped or not.
 */
const NAPPING_BUY_LINE = 'warn twin memo: the buy for a shared key was abandoned (refused by ProviderHttpError '
  + 'from synthetic with HTTP 503 (the provider said: "the second cat is napping in the sun")), so any twin waiting '
  + 'on it asks for itself';

/**
 Names an item in the line its unthrown failure is written on.

 @param row - item and its position

 @returns The item as a cat's name

 @example
 ```ts
 catName({ item: 'b', position: 1, },); // 'cat b'
 ```
 */
function catName({ item, }: { readonly item: string; readonly position: number; },): string {
  return `cat ${item}`;
}

//endregion Fixtures

await describe({
  name: mapOverlapped.name,
  children: [
    it({
      name: 'runs the items one at a time in item order at overlap 1, which is the loop it replaced',
      fn: async () => {
        const run = gatedRun({ count: 3, },);
        const mapping = mapOverlapped({
          items: ITEMS.slice(
            0,
            3,
          ),
          overlap: 1,
          oneItem: run.oneItem,
          signal: NEVER_STOPPED,
          l: QUIET,
          nameOf: catName,
        },);
        await settle();
        expect(run.started,).toEqual([0,],);

        nonNullishOrThrow(run.gates[0],).resolve('one',);
        await settle();
        expect(run.started,).toEqual([
          0,
          1,
        ],);

        nonNullishOrThrow(run.gates[1],).resolve('two',);
        await settle();
        expect(run.started,).toEqual([
          0,
          1,
          2,
        ],);

        nonNullishOrThrow(run.gates[2],).resolve('three',);
        expect(await mapping,).toEqual([
          'a:one',
          'b:two',
          'c:three',
        ],);
        expect(run.inFlight
          .peak,).toBe(1,);
      },
    },),

    it({
      name: 'keeps no more than the overlap in flight, and starts items in item order',
      fn: async () => {
        const run = gatedRun({ count: 4, },);
        const mapping = mapOverlapped({
          items: ITEMS,
          overlap: 2,
          oneItem: run.oneItem,
          signal: NEVER_STOPPED,
          l: QUIET,
          nameOf: catName,
        },);
        await settle();
        expect(run.started,).toEqual([
          0,
          1,
        ],);
        expect(run.inFlight
          .peak,).toBe(2,);

        nonNullishOrThrow(run.gates[0],).resolve('one',);
        await settle();
        expect(run.started,).toEqual([
          0,
          1,
          2,
        ],);

        nonNullishOrThrow(run.gates[1],).resolve('two',);
        nonNullishOrThrow(run.gates[2],).resolve('three',);
        await settle();
        nonNullishOrThrow(run.gates[3],).resolve('four',);
        expect(await mapping,).toEqual([
          'a:one',
          'b:two',
          'c:three',
          'd:four',
        ],);
        expect(run.inFlight
          .peak,).toBe(2,);
      },
    },),

    it({
      name: 'returns results in item order however the items finished',
      fn: async () => {
        const run = gatedRun({ count: 4, },);
        const mapping = mapOverlapped({
          items: ITEMS,
          overlap: 2,
          oneItem: run.oneItem,
          signal: NEVER_STOPPED,
          l: QUIET,
          nameOf: catName,
        },);
        await settle();

        // The second item finishes first, so the second lane takes the third
        // item while the first item is still waiting.
        nonNullishOrThrow(run.gates[1],).resolve('two',);
        await settle();
        expect(run.started,).toEqual([
          0,
          1,
          2,
        ],);

        nonNullishOrThrow(run.gates[0],).resolve('one',);
        await settle();
        nonNullishOrThrow(run.gates[3],).resolve('four',);
        await settle();
        nonNullishOrThrow(run.gates[2],).resolve('three',);
        expect(await mapping,).toEqual([
          'a:one',
          'b:two',
          'c:three',
          'd:four',
        ],);
      },
    },),

    it({
      name: 'stops starting items after one fails, lets the one in flight finish, and throws that failure',
      fn: async () => {
        const run = gatedRun({ count: 4, },);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS,
            overlap: 2,
            oneItem: run.oneItem,
            signal: NEVER_STOPPED,
            l: QUIET,
            nameOf: catName,
          },),
        },);
        await settle();

        /**
         What the second item throws.
         */
        const fault = new Error('the second item broke',);
        nonNullishOrThrow(run.gates[1],).reject(fault,);
        await settle();
        // Nothing past the two already started is admitted.
        expect(run.started,).toEqual([
          0,
          1,
        ],);

        nonNullishOrThrow(run.gates[0],).resolve('one',);
        expect(await failure,).toBe(fault,);
        expect(run.started,).toEqual([
          0,
          1,
        ],);
        expect(run.inFlight
          .peak,).toBe(2,);
      },
    },),

    it({
      name: 'throws the LOWEST position`s failure when two fail, which is the one a sequential loop would have reached first',
      fn: async () => {
        const run = gatedRun({ count: 4, },);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS,
            overlap: 2,
            oneItem: run.oneItem,
            signal: NEVER_STOPPED,
            l: QUIET,
            nameOf: catName,
          },),
        },);
        await settle();

        /**
         What the second item throws, first in time.
         */
        const later = new Error('the second item broke first',);
        nonNullishOrThrow(run.gates[1],).reject(later,);
        await settle();

        /**
         What the first item throws, second in time.
         */
        const earlier = new Error('the first item broke after it',);
        nonNullishOrThrow(run.gates[0],).reject(earlier,);
        expect(await failure,).toBe(earlier,);
        expect(run.started,).toEqual([
          0,
          1,
        ],);
      },
    },),

    it({
      name: 'THROWS THE EARLIEST POSITION\'S FAILURE when a later position raised a model-dark refusal first, since '
        + 'a model whose own providers are dry costs one item its voice and does not end the run, so it outranks nothing',
      fn: async () => {
        const run = gatedRun({ count: 3, },);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              3,
            ),
            overlap: 3,
            oneItem: run.oneItem,
            signal: NEVER_STOPPED,
            l: QUIET,
            nameOf: catName,
          },),
        },);
        await settle();
        nonNullishOrThrow(run.gates[2],).reject(modelDarkRefusal({ modelId: 'glm-5.3', },),);
        await settle();
        /**
         What the earlier item throws, second in time.
         */
        const earlier = new Error('the first item broke after it',);
        nonNullishOrThrow(run.gates[0],).reject(earlier,);
        nonNullishOrThrow(run.gates[1],).resolve('fine',);
        expect(await failure,).toBe(earlier,);
      },
    },),

    it({
      name: 'THROWS THE EVERY-PROVIDER-DRY REFUSAL although an earlier position failed for another reason, since the '
        + 'dry refusal ends the run and the other failure is one item\'s, and writes the earlier failure it does not throw',
      fn: async () => {
        const run = gatedRun({ count: 3, },);
        /**
         Every line the helper wrote, behind its level.
         */
        const lines: string[] = [];
        /**
         What the router raised for the later item, first in time.
         */
        const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              3,
            ),
            overlap: 3,
            oneItem: run.oneItem,
            signal: NEVER_STOPPED,
            l: levelCapturingLogger({ lines, },),
            nameOf: catName,
          },),
        },);
        await settle();
        nonNullishOrThrow(run.gates[2],).reject(dry,);
        await settle();
        nonNullishOrThrow(run.gates[0],).reject(new Error('the first item broke after it',),);
        nonNullishOrThrow(run.gates[1],).resolve('fine',);
        expect(await failure,).toBe(dry,);
        expect(lines,).toEqual([
          'warn [mapOverlapped] cat a failed (refused by Error), and the failure thrown is cat c\'s, '
            + 'which ends the run',
        ],);
      },
    },),

    it({
      name: 'THROWS A FAILURE THAT CARRIES A NEURALWATT RETURN although an earlier position failed for another reason, '
        + 'since the pass may park on it, and says only that it names a return the run\'s gate may park on',
      fn: async () => {
        const run = gatedRun({ count: 3, },);
        /**
         Every line the helper wrote, behind its level.
         */
        const lines: string[] = [];
        /**
         A bench error standing in for the run package's, which this package cannot import: it names the distance to the
         NeuralWatt return that would lift the bench.
         */
        class BenchWithReturnError extends Error {
          public override readonly name: string = 'BenchWithReturnError';

          public readonly neuralwattReturnsInMs: number = 301_000;
        }
        /**
         What the later item raised, first in time.
         */
        const bench = new BenchWithReturnError('writing bench unreachable',);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              3,
            ),
            overlap: 3,
            oneItem: run.oneItem,
            signal: NEVER_STOPPED,
            l: levelCapturingLogger({ lines, },),
            nameOf: catName,
          },),
        },);
        await settle();
        nonNullishOrThrow(run.gates[2],).reject(bench,);
        await settle();
        nonNullishOrThrow(run.gates[0],).reject(new Error('the first item broke after it',),);
        nonNullishOrThrow(run.gates[1],).resolve('fine',);
        expect(await failure,).toBe(bench,);
        expect(lines,).toEqual([
          'warn [mapOverlapped] cat a failed (refused by Error), and the failure thrown is cat c\'s, '
            + 'which names a NeuralWatt return the run\'s gate may park on',
        ],);
      },
    },),

    it({
      name: 'THROWS THE EVERY-PROVIDER-DRY REFUSAL OVER A BENCH ERROR THAT CARRIES A RETURN in either arrival order: '
        + 'the refusal ends the run and the bench error only may be parked on, so what arrived first does not '
        + 'decide it',
      fn: async () => {
        /**
         A bench error standing in for the run package's, which this package cannot import.
         */
        class BenchWithReturnError extends Error {
          public override readonly name: string = 'BenchWithReturnError';

          public readonly neuralwattReturnsInMs: number = 301_000;
        }
        /**
         What the router raised.
         */
        const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);
        /**
         What the thrown failure is when the bench error arrives first and the refusal second.
         */
        const benchFirst = gatedRun({ count: 3, },);
        const first = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              3,
            ),
            overlap: 3,
            oneItem: benchFirst.oneItem,
            signal: NEVER_STOPPED,
            l: QUIET,
            nameOf: catName,
          },),
        },);
        await settle();
        nonNullishOrThrow(benchFirst.gates[1],).reject(new BenchWithReturnError('writing bench unreachable',),);
        await settle();
        nonNullishOrThrow(benchFirst.gates[2],).reject(dry,);
        await settle();
        nonNullishOrThrow(benchFirst.gates[0],).resolve('fine',);
        expect(await first,).toBe(dry,);

        /**
         The same two failures with the refusal arriving first.
         */
        const dryFirst = gatedRun({ count: 3, },);
        const second = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              3,
            ),
            overlap: 3,
            oneItem: dryFirst.oneItem,
            signal: NEVER_STOPPED,
            l: QUIET,
            nameOf: catName,
          },),
        },);
        await settle();
        nonNullishOrThrow(dryFirst.gates[2],).reject(dry,);
        await settle();
        nonNullishOrThrow(dryFirst.gates[1],).reject(new BenchWithReturnError('writing bench unreachable',),);
        await settle();
        nonNullishOrThrow(dryFirst.gates[0],).resolve('fine',);
        expect(await second,).toBe(dry,);
      },
    },),

    it({
      name: 'admits every item at once when the overlap exceeds the count',
      fn: async () => {
        const run = gatedRun({ count: 3, },);
        const mapping = mapOverlapped({
          items: ITEMS.slice(
            0,
            3,
          ),
          overlap: 5,
          oneItem: run.oneItem,
          signal: NEVER_STOPPED,
          l: QUIET,
          nameOf: catName,
        },);
        await settle();
        expect(run.started,).toEqual([
          0,
          1,
          2,
        ],);
        expect(run.inFlight
          .peak,).toBe(3,);

        nonNullishOrThrow(run.gates[2],).resolve('three',);
        nonNullishOrThrow(run.gates[1],).resolve('two',);
        nonNullishOrThrow(run.gates[0],).resolve('one',);
        expect(await mapping,).toEqual([
          'a:one',
          'b:two',
          'c:three',
        ],);
      },
    },),

    it({
      name: 'REFUSES an overlap below one or fractional before running anything',
      fn: async () => {
        const run = gatedRun({ count: 1, },);
        for (const overlap of [
          0,
          -1,
          1.5,
          Number.NaN,
        ]) {
          // oxlint-disable-next-line no-await-in-loop -- each refusal is checked in turn on the same fixture
          const refusal = await collected({
            run: mapOverlapped({
              items: ITEMS.slice(
                0,
                1,
              ),
              overlap,
              oneItem: run.oneItem,
              signal: NEVER_STOPPED,
              l: QUIET,
              nameOf: catName,
            },),
          },);
          expect(refusal,).toBeInstanceOf(OverlapRefusedError,);
          expect((refusal as Error).message,).toContain(String(overlap,),);
        }
        expect(run.started,).toEqual([],);
      },
    },),

    it({
      name: 'maps no items to no results, running nothing',
      fn: async () => {
        const run = gatedRun({ count: 0, },);
        expect(await mapOverlapped({
          items: [],
          overlap: 3,
          oneItem: run.oneItem,
          signal: NEVER_STOPPED,
          l: QUIET,
          nameOf: catName,
        },),).toEqual([],);
        expect(run.started,).toEqual([],);
      },
    },),

    it({
      name: 'WRITES EACH JOB\'S OWN FAILURE THE CALLER\'S STOP LEFT UNTHROWN, in position order whichever ended '
        + 'first, with the provider\'s words, writes none for a job ending in the caller\'s own reason at any '
        + 'position, and throws that reason',
      fn: async () => {
        const run = gatedRun({ count: 4, },);
        /**
         Every line the helper wrote, behind its level.
         */
        const lines: string[] = [];
        /**
         The caller's steering.
         */
        const steering = new AbortController();
        /**
         Why the caller stopped the work.
         */
        const reason = new Error('the owner called the cats in',);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS,
            overlap: 4,
            oneItem: run.oneItem,
            signal: steering.signal,
            l: levelCapturingLogger({ lines, },),
            nameOf: catName,
          },),
        },);
        await settle();
        steering.abort(reason,);
        nonNullishOrThrow(run.gates[3],).reject(new ProviderHttpError({
          provider: 'synthetic',
          status: 503,
          bodyText: 'the fourth cat is napping in the sun',
        },),);
        await settle();
        nonNullishOrThrow(run.gates[2],).reject(reason,);
        await settle();
        nonNullishOrThrow(run.gates[1],).reject(new ProviderHttpError({
          provider: 'synthetic',
          status: 429,
          bodyText: 'the second cat is out of treats',
        },),);
        await settle();
        nonNullishOrThrow(run.gates[0],).reject(reason,);
        expect(await failure,).toBe(reason,);
        expect(lines,).toEqual([
          'warn [mapOverlapped] cat b failed (refused by ProviderHttpError from synthetic with HTTP 429 (the provider said: "the '
            + 'second cat is out of treats")), and the caller stopped the work',
          'warn [mapOverlapped] cat d failed (refused by ProviderHttpError from synthetic with HTTP 503 (the provider said: "the '
            + 'fourth cat is napping in the sun")), and the caller stopped the work',
        ],);
      },
    },),

    it({
      name: 'WRITES NO LINE FOR THE FAILURE IT THROWS under the caller\'s stop, nor for a later job ending in the '
        + 'caller\'s reason, and writes a job\'s own failure that ended before the stop arrived',
      fn: async () => {
        const run = gatedRun({ count: 3, },);
        /**
         Every line the helper wrote, behind its level.
         */
        const lines: string[] = [];
        /**
         The caller's steering.
         */
        const steering = new AbortController();
        /**
         Why the caller stopped the work.
         */
        const reason = new Error('the owner called the cats in',);
        /**
         What the first item throws, after the stop.
         */
        const firstRefusal = new ProviderHttpError({
          provider: 'synthetic',
          status: 500,
          bodyText: 'the first cat knocked the bowl over',
        },);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              3,
            ),
            overlap: 3,
            oneItem: run.oneItem,
            signal: steering.signal,
            l: levelCapturingLogger({ lines, },),
            nameOf: catName,
          },),
        },);
        await settle();
        nonNullishOrThrow(run.gates[1],).reject(new ProviderHttpError({
          provider: 'synthetic',
          status: 429,
          bodyText: 'the second cat is out of treats',
        },),);
        await settle();
        steering.abort(reason,);
        nonNullishOrThrow(run.gates[2],).reject(reason,);
        await settle();
        nonNullishOrThrow(run.gates[0],).reject(firstRefusal,);
        expect(await failure,).toBe(firstRefusal,);
        expect(lines,).toEqual([
          'warn [mapOverlapped] cat b failed (refused by ProviderHttpError from synthetic with HTTP 429 (the provider said: "the '
            + 'second cat is out of treats")), and the caller stopped the work',
        ],);
      },
    },),

    it({
      name: 'WRITES EACH FAILURE IT DOES NOT THROW WHILE THE CALLER HAS NOT STOPPED, in position order whichever '
        + 'ended first, with the provider\'s words and the item whose failure is thrown, and writes none for the '
        + 'lowest position\'s failure, which it throws',
      fn: async () => {
        const run = gatedRun({ count: 3, },);
        /**
         Every line the helper wrote, behind its level.
         */
        const lines: string[] = [];
        /**
         What the first item throws, last in time.
         */
        const earlier = new Error('the first item broke after the others',);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              3,
            ),
            overlap: 3,
            oneItem: run.oneItem,
            signal: NEVER_STOPPED,
            l: levelCapturingLogger({ lines, },),
            nameOf: catName,
          },),
        },);
        await settle();
        nonNullishOrThrow(run.gates[2],).reject(new ProviderHttpError({
          provider: 'synthetic',
          status: 503,
          bodyText: 'the third cat is napping in the sun',
        },),);
        await settle();
        nonNullishOrThrow(run.gates[1],).reject(new ProviderHttpError({
          provider: 'synthetic',
          status: 429,
          bodyText: 'the second cat is out of treats',
        },),);
        await settle();
        nonNullishOrThrow(run.gates[0],).reject(earlier,);
        expect(await failure,).toBe(earlier,);
        expect(lines,).toEqual([
          'warn [mapOverlapped] cat b failed (refused by ProviderHttpError from synthetic with HTTP 429 (the provider said: "the '
            + 'second cat is out of treats")), and the failure thrown is cat a\'s, which comes before it',
          'warn [mapOverlapped] cat c failed (refused by ProviderHttpError from synthetic with HTTP 503 (the provider said: "the '
            + 'third cat is napping in the sun")), and the failure thrown is cat a\'s, which comes before it',
        ],);
      },
    },),

    it({
      name: 'WRITES A FAILURE THAT IS NO ERROR AT ALL while the caller has not stopped, since only a stopped '
        + 'caller\'s reason is skipped',
      fn: async () => {
        const run = gatedRun({ count: 2, },);
        /**
         Every line the helper wrote, behind its level.
         */
        const lines: string[] = [];
        /**
         What the first item throws.
         */
        const earlier = new Error('the first item broke',);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              2,
            ),
            overlap: 2,
            oneItem: run.oneItem,
            signal: NEVER_STOPPED,
            l: levelCapturingLogger({ lines, },),
            nameOf: catName,
          },),
        },);
        await settle();
        // The reason a signal that never aborted reads, which a failure equal
        // to it must not be mistaken for.
        nonNullishOrThrow(run.gates[1],).reject(NEVER_STOPPED.reason,);
        await settle();
        nonNullishOrThrow(run.gates[0],).reject(earlier,);
        expect(await failure,).toBe(earlier,);
        expect(lines,).toEqual([
          'warn [mapOverlapped] cat b failed (refused by a thrown value that is not an Error), and the failure '
            + 'thrown is cat a\'s, which comes before it',
        ],);
      },
    },),

    it({
      name: 'WRITES ONCE a buy\'s own failure under the caller\'s stop, on the line the twin memo wrote as the buy '
        + 'ended, and adds no line of its own for it',
      fn: async () => {
        const run = gatedRun({ count: 2, },);
        /**
         Every line the memo and the helper wrote, behind its level.
         */
        const lines: string[] = [];
        /**
         Logger both write through.
         */
        const l = levelCapturingLogger({ lines, },);
        /**
         The caller's steering.
         */
        const steering = new AbortController();
        /**
         Why the caller stopped the work.
         */
        const reason = new Error('the owner called the cats in',);
        /**
         Buys of this run by question.
         */
        const memo: TwinMemo<string> = new Map();
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              2,
            ),
            overlap: 2,
            oneItem: async function waitsOrBuys(row,): Promise<string> {
              if (row.position === 0)
                return await run.oneItem(row,);
              /**
               The second item's buy under the memo.
               */
              const asked = await reuseTwinOrBuy({
                key: 'the sunny windowsill',
                memo,
                buy: async function buyTheWindowsill(): Promise<string> {
                  return await run.oneItem(row,);
                },
                persistedOf: function stored(bought: string,): TwinStored<string> {
                  return {
                    kind: 'stored',
                    record: bought,
                  };
                },
                l,
                signal: steering.signal,
              },);
              return (asked.kind === 'reused') ? asked.twin : asked.bought;
            },
            signal: steering.signal,
            l,
            nameOf: catName,
          },),
        },);
        await settle();
        steering.abort(reason,);
        nonNullishOrThrow(run.gates[1],).reject(new ProviderHttpError({
          provider: 'synthetic',
          status: 503,
          bodyText: 'the second cat is napping in the sun',
        },),);
        await settle();
        nonNullishOrThrow(run.gates[0],).reject(reason,);
        expect(await failure,).toBe(reason,);
        expect(lines,).toEqual([
          NAPPING_BUY_LINE,
        ],);
      },
    },),

    it({
      name: 'WRITES ONCE a buy\'s own failure while the caller has not stopped, on the line the twin memo wrote, '
        + 'and adds no line of its own for it',
      fn: async () => {
        const run = gatedRun({ count: 2, },);
        /**
         Every line the memo and the helper wrote, behind its level.
         */
        const lines: string[] = [];
        /**
         Logger both write through.
         */
        const l = levelCapturingLogger({ lines, },);
        /**
         Buys of this run by question.
         */
        const memo: TwinMemo<string> = new Map();
        /**
         What the first item throws.
         */
        const earlier = new Error('the first item broke',);
        const failure = collected({
          run: mapOverlapped({
            items: ITEMS.slice(
              0,
              2,
            ),
            overlap: 2,
            oneItem: async function waitsOrBuys(row,): Promise<string> {
              if (row.position === 0)
                return await run.oneItem(row,);
              /**
               The second item's buy under the memo.
               */
              const asked = await reuseTwinOrBuy({
                key: 'the sunny windowsill',
                memo,
                buy: async function buyTheWindowsill(): Promise<string> {
                  return await run.oneItem(row,);
                },
                persistedOf: function stored(bought: string,): TwinStored<string> {
                  return {
                    kind: 'stored',
                    record: bought,
                  };
                },
                l,
                signal: NEVER_STOPPED,
              },);
              return (asked.kind === 'reused') ? asked.twin : asked.bought;
            },
            signal: NEVER_STOPPED,
            l,
            nameOf: catName,
          },),
        },);
        await settle();
        nonNullishOrThrow(run.gates[1],).reject(new ProviderHttpError({
          provider: 'synthetic',
          status: 503,
          bodyText: 'the second cat is napping in the sun',
        },),);
        await settle();
        nonNullishOrThrow(run.gates[0],).reject(earlier,);
        expect(await failure,).toBe(earlier,);
        expect(lines,).toEqual([
          NAPPING_BUY_LINE,
        ],);
      },
    },),
  ],
},);
