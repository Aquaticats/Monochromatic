/**
 Tests what the seat reader says and does when the return it was told of lies past what the pass has
 left, and the bound itself.

 The reader refuses at once for such a return, and the words it refuses with state the return as past the pass in every
 place it speaks: the error line of a phase that goes on, the warning of a bench short of quorum, and the run-ending
 refusal of a reading where every provider is dry, which also carries the distance for the pass to read. A bound of zero
 or below (the pass or the entry's cap already spent) waits for no return at all.

 Every case here that names a return past the bound sleeps nothing, and reads the budgets on a frozen monotonic clock.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup';
import {
  rejectionOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  holdsWithinPass,
  readPhaseSeats,
  WritingBenchUnreachableError,
} from '../../dist/final/node/index.mjs';
import {
  clientBefore,
  SIGNAL,
} from './neuralwatt-seat-client.test-fixture.ts';

/**
 The clock the budgets count a return on, frozen so no host stall moves a distance between its reading and its hold.
 */
const FROZEN = (): number => 0;

await describe({
  name: 'the seat reader and a return past what the pass has left',
  concurrency: 1,
  children: [
    it({
      name: 'SAYS A RETURN PAST THE PASS IS NAMED in the error line of a phase that goes on without its writers, and '
        + 'does not say that no return was named: 700 away with 600 left',
      fn: async () => {
        const lines: string[] = [];
        const { client, reads, } = clientBefore({
          msBeforeReturn: 700,
          passLeftMs: 600,
          syntheticWet: false,
          now: FROZEN,
        },);
        const { unreachableWriters, } = await readPhaseSeats({
          client,
          phase: 'page consistency',
          signal: SIGNAL,
          l: levelCapturingLogger({ lines, },),
          pollMs: 5,
        },);
        expect(unreachableWriters.length,).toBeGreaterThan(0,);
        expect(reads.count,).toBe(1,);
        /**
         The error line the reading wrote.
         */
        const errorLine = lines.find(function isError(line,): boolean {
          return line.startsWith('error JUDGE SEATS',);
        },);
        expect(errorLine,).toContain('this pass has left',);
        expect(errorLine,).not.toContain('no provider has named its return',);
        expect(errorLine,).toContain('no provider has named a return the pass can wait for',);
      },
    },),

    it({
      name: 'SAYS A RETURN PAST THE PASS IS NAMED in the warning of a bench short of quorum that stops no entry (the '
        + 'readers at the pictures, every provider dry), instead of saying no provider named one',
      fn: async () => {
        const lines: string[] = [];
        const { client, } = clientBefore({ msBeforeReturn: 700, passLeftMs: 600, syntheticWet: false, now: FROZEN, },);
        await readPhaseSeats({
          client,
          phase: 'pictures',
          signal: SIGNAL,
          l: levelCapturingLogger({ lines, },),
          pollMs: 5,
        },);
        /**
         The warning the reading wrote.
         */
        const warning = lines.find(function isShort(line,): boolean {
          return line.startsWith('warn JUDGE SEATS phase=pictures short of quorum',);
        },);
        expect(warning,).toContain('this pass has left',);
        expect(warning,).not.toContain('no provider has named its return',);
      },
    },),

    it({
      name: 'ENDS THE RUN WITH THE DISTANCE AND THE PASS SENTENCE when every provider is dry and the return lies past '
        + 'the pass: the refusal carries the milliseconds to the return for the pass to read, states the time of day, '
        + 'says the pass has less left, and prints no NeuralWatt hold it did not wait for',
      fn: async () => {
        const { client, reads, } = clientBefore({
          msBeforeReturn: 700,
          passLeftMs: 600,
          syntheticWet: false,
          now: FROZEN,
        },);
        const before = monotonicMs();
        const refusal = await rejectionOf({
          promise: readPhaseSeats({
            client,
            phase: 'lanes',
            signal: SIGNAL,
            l: levelCapturingLogger({ lines: [], },),
            pollMs: 5,
          },),
        },);
        const after = monotonicMs();
        expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
        // THE READ THAT NAMED THE RETURN IS STAMPED ON THE REFUSAL TOO, so its park waits from the read as the bench
        // error's does.
        const readAt = (refusal as EveryProviderDryError).neuralwattReadAtMs;
        expect(readAt,).toBeGreaterThanOrEqual(before,);
        expect(readAt,).toBeLessThanOrEqual(after,);
        expect((refusal as EveryProviderDryError).neuralwattReturnsInMs,).toBe(700,);
        expect((refusal as EveryProviderDryError).message,).toContain('19:15:08 UTC',);
        expect((refusal as EveryProviderDryError).message,).toContain('this pass has left',);
        expect((refusal as EveryProviderDryError).message,).not.toContain('neuralwatt 700ms',);
        expect(reads.count,).toBe(1,);
      },
    },),

    it({
      name: 'NAMES THE ENTRY WHEN ITS CAP IS THE SMALLER BOUND: a return an hour away with 20 min left of the entry\'s '
        + 'cap says "the entry has left" in the run-ending refusal and in the bench error, and "this pass has left" '
        + 'when the pass\'s own bound is the smaller',
      fn: async () => {
        /**
         The refusal of a reading over a return an hour away under a 20 min bound owned by the given side.
         */
        async function refusalUnder(
          { boundOwner, syntheticWet, }: { readonly boundOwner: 'entry' | 'pass'; readonly syntheticWet: boolean; },
        ): Promise<Error> {
          const { client, } = clientBefore({
            msBeforeReturn: 3_600_000,
            passLeftMs: 1_200_000,
            boundOwner,
            syntheticWet,
            now: FROZEN,
          },);
          return await rejectionOf({
            promise: readPhaseSeats({
              client,
              phase: 'lanes',
              signal: SIGNAL,
              l: levelCapturingLogger({ lines: [], },),
              pollMs: 5,
            },),
          },) as Error;
        }
        /**
         The run-ending refusal and the bench error under the entry's bound.
         */
        const dryEntry = await refusalUnder({ boundOwner: 'entry', syntheticWet: false, },);
        const benchEntry = await refusalUnder({ boundOwner: 'entry', syntheticWet: true, },);
        expect(dryEntry.message,).toContain('past the 20 min the entry has left',);
        expect(dryEntry.message,).not.toContain('this pass has left',);
        expect(benchEntry.message,).toContain('the entry has left',);
        expect(benchEntry.message,).not.toContain('this pass has left',);
        /**
         The same refusal under the pass's own bound.
         */
        const dryPass = await refusalUnder({ boundOwner: 'pass', syntheticWet: false, },);
        expect(dryPass.message,).toContain('past the 20 min this pass has left',);
        expect(dryPass.message,).not.toContain('the entry has left',);
      },
    },),

    it({
      name: 'CARRIES THE DISTANCE ON THE BENCH ERROR WHEN NEURALWATT\'S RETURN LIFTS THE WRITING BENCH TO ITS FLOOR: '
        + 'with Synthetic the only wet provider the lanes\' editors, refiners and translators are below the floor, '
        + 'NeuralWatt back reaches it, and the error carries the milliseconds, the time of day and the spent day, '
        + 'for the pass to park on',
      fn: async () => {
        const { client, } = clientBefore({ msBeforeReturn: 700, passLeftMs: 600, now: FROZEN, },);
        const before = monotonicMs();
        const refusal = await rejectionOf({
          promise: readPhaseSeats({
            client,
            phase: 'lanes',
            signal: SIGNAL,
            l: levelCapturingLogger({ lines: [], },),
            pollMs: 5,
          },),
        },);
        const after = monotonicMs();
        expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
        // THE READ THAT NAMED THE RETURN IS STAMPED, so a park measures the wait from it and not from the trip.
        const readAt = (refusal as WritingBenchUnreachableError).neuralwattReadAtMs;
        expect(readAt,).toBeGreaterThanOrEqual(before,);
        expect(readAt,).toBeLessThanOrEqual(after,);
        expect((refusal as WritingBenchUnreachableError).neuralwattReturnsInMs,).toBe(700,);
        expect((refusal as WritingBenchUnreachableError).neuralwattReturnsAt,).toBe('19:15:08 UTC',);
        expect((refusal as WritingBenchUnreachableError).neuralwattDryness?.kind,).toBe('spent-day',);
      },
    },),

    it({
      name: 'CARRIES NO DISTANCE ON THE BENCH ERROR WHEN THE ACCOUNT NAMES NO RETURN (the control): the reader waited '
        + '60 ms for the return, the account still shows the day spent, and no further return is named',
      fn: async () => {
        // The case sleeps for real: the reader waits out the 60 ms and reads the account again.
        const { client, } = clientBefore({ msBeforeReturn: 60, lagging: true, },);
        const refusal = await rejectionOf({
          promise: readPhaseSeats({
            client,
            phase: 'lanes',
            signal: SIGNAL,
            l: levelCapturingLogger({ lines: [], },),
            pollMs: 5,
          },),
        },);
        expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
        expect((refusal as WritingBenchUnreachableError).neuralwattReturnsInMs,).toBeUndefined();
        expect((refusal as WritingBenchUnreachableError).neuralwattReturnsAt,).toBeUndefined();
      },
    },),

    it({
      name: 'WAITS FOR NO RETURN WHEN THE BOUND IS ZERO OR BELOW, the pass or the entry\'s cap already spent: a return '
        + '5 away with a bound of -1 refuses at once and the sentence reads 0 s, never a negative number',
      fn: async () => {
        const { client, reads, } = clientBefore({ msBeforeReturn: 5, passLeftMs: -1, now: FROZEN, },);
        const refusal = await rejectionOf({
          promise: readPhaseSeats({
            client,
            phase: 'lanes',
            signal: SIGNAL,
            l: levelCapturingLogger({ lines: [], },),
            pollMs: 5,
          },),
        },);
        expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
        expect((refusal as WritingBenchUnreachableError).message,).toContain('past the 0 s this pass has left',);
        expect(reads.count,).toBe(1,);
      },
    },),

    describe({
      name: holdsWithinPass.name,
      concurrency: 1,
      children: [
        it({
          name: 'HANDS THE CLIENT\'S BOUND TO THE HOLDS, and no bound where the client carries none',
          fn: async () => {
            /**
             The bounds the holds were asked for.
             */
            const asked: number[] = [];
            const holds = {
              neuralwatt: 0,
              synthetic: 0,
              bedrock: 0,
              deepseek: 0,
              xiaomi: 0,
              anthropic: 0,
              hyper: 0,
              openrouter: 0,
            };
            /**
             Records the bound a reading asked for.

             @param args - what the reading handed over

             @returns No hold
             */
            function providerHolds(args?: { readonly returnBoundMs?: number; },): typeof holds {
              asked.push(args?.returnBoundMs ?? Number.NaN,);
              return holds;
            }
            holdsWithinPass({ client: { providerHolds, passLeftMs: () => 600, }, },);
            holdsWithinPass({ client: { providerHolds, }, },);
            expect(asked,).toEqual([600, Number.POSITIVE_INFINITY,],);
          },
        },),
      ],
    },),
  ],
},);
