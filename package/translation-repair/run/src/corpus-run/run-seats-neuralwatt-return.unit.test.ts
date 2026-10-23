/**
 Tests the seat reader against the XingZ6019 shape: NeuralWatt dry because the day's energy was spent, its day starting
 again 48 s after the entry stopped, and a writing bench short of its floor without it.

 The reader waits for a return the budgets name when it lies inside the bound its client carries (what the pass has
 left), at any distance for a client that carries none, and then seats the NeuralWatt writers; where no return is named,
 or it lies past the bound, it refuses and the refusal states only the true cause. The budgets are the real ones over
 stub meters and a stub wall clock, so the return reaches the reader the way a run's does (a hold), and the
 times are scaled to milliseconds.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  readPhaseSeats,
  WritingBenchUnreachableError,
} from '../../dist/final/node/index.mjs';
import {
  clientBefore,
  SIGNAL,
  SPENT_DAY,
} from './neuralwatt-seat-client.test-fixture.ts';

await describe({
  name: 'the seat reader and NeuralWatt\'s return',
  concurrency: 1,
  children: [
    it({
      name: 'WAITS FOR NEURALWATT\'S DAY TO START AGAIN when the lanes\' writing benches stand short without it and the '
        + 'return is near, then seats the NeuralWatt writers, logging the wait with its reason and its end',
      fn: async () => {
        const lines: string[] = [];
        const { client, reads, } = clientBefore({ msBeforeReturn: 60, },);
        const { unreachableWriters, } = await readPhaseSeats({
          client,
          phase: 'lanes',
          signal: SIGNAL,
          l: levelCapturingLogger({ lines, },),
          pollMs: 5,
        },);
        expect(unreachableWriters,).toEqual([],);
        expect(reads.count,).toBe(2,);
        expect(lines.some(function waitLine(line,): boolean {
          return line.startsWith('warn JUDGE SEATS phase=lanes short of quorum',)
            && line.includes('waiting ',) && (!line.includes('waiting 0ms',))
            && line.includes('neuralwatt, whose day starts again at 19:15:08 UTC',);
        },),).toBe(true,);
        expect(lines.some(function endLine(line,): boolean {
          return line.startsWith('info JUDGE SEATS phase=lanes waited ',) && (!line.startsWith('info JUDGE SEATS phase=lanes waited 0ms',))
            && line.includes('for neuralwatt, whose day starts again at 19:15:08 UTC',);
        },),).toBe(true,);
        expect(lines.some(function seatsLine(line,): boolean {
          return line.startsWith('info JUDGE SEATS phase=lanes neuralwatt=wet',) && line.includes('waited=',)
            && (!line.includes('waited=0ms',));
        },),).toBe(true,);
      },
    },),

    it({
      name: 'WAITS FOR A RETURN PAST THE 300 s A ROUTER CALL WAITS for a client that names no pass bound, the way '
        + 'Hyper\'s stated holds are waited for: the budgets name the return as a hold at any distance for a caller '
        + 'that names no bound, and the NeuralWatt writers are seated once it has passed',
      fn: async () => {
        const lines: string[] = [];
        const { client, reads, } = clientBefore({ msBeforeReturn: 80, },);
        const { unreachableWriters, } = await readPhaseSeats({
          client,
          phase: 'lanes',
          signal: SIGNAL,
          l: levelCapturingLogger({ lines, },),
          pollMs: 5,
        },);
        expect(unreachableWriters,).toEqual([],);
        expect(reads.count,).toBe(2,);
        expect(lines.some(function waitLine(line,): boolean {
          return line.startsWith('warn JUDGE SEATS phase=lanes short of quorum',)
            && line.includes('neuralwatt, whose day starts again at 19:15:08 UTC',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'STATES THE KEY\'S OWN TIME FOR A BLOCKED KEY and not a day start: NeuralWatt dry because the key\'s '
        + 'allowance is blocked comes back when the block lifts, so the refusal names that and no return it '
        + 'does not have',
      fn: async () => {
        const lines: string[] = [];
        const { client, reads, } = clientBefore({
          msBeforeReturn: 48_000,
          spent: { ...SPENT_DAY, remainingWh: 400, keyBlockedUntil: '2026-10-23T05:00:00.000Z', },
        },);
        const refusal = await rejectionOf({
          promise: readPhaseSeats({
            client,
            phase: 'lanes',
            signal: SIGNAL,
            l: levelCapturingLogger({ lines, },),
            pollMs: 5,
          },),
        },);
        expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
        expect((refusal as WritingBenchUnreachableError).message,).toContain(
          'NeuralWatt reads dry because its key allowance is blocked until 2026-10-23T05:00:00.000Z',
        );
        expect((refusal as WritingBenchUnreachableError).message,).not.toContain('day starts again',);
        expect(reads.count,).toBe(1,);
      },
    },),

    it({
      name: 'WAITS FOR NEURALWATT WITH EVERY PROVIDER DRY: Synthetic spent too, the return 60 ms away, and the reading '
        + 'waits for the hold and seats the NeuralWatt writers instead of ending the run',
      fn: async () => {
        const lines: string[] = [];
        const { client, reads, } = clientBefore({ msBeforeReturn: 60, syntheticWet: false, },);
        const { unreachableWriters, } = await readPhaseSeats({
          client,
          phase: 'lanes',
          signal: SIGNAL,
          l: levelCapturingLogger({ lines, },),
          pollMs: 5,
        },);
        expect(unreachableWriters,).toEqual([],);
        expect(reads.count,).toBe(2,);
      },
    },),

    it({
      name: 'STATES THAT THE ACCOUNT STILL SHOWS THE DAY SPENT when the reader waited for the return and the account '
        + 'has not yet shown the new day, instead of waiting a whole day more for a return that has passed',
      fn: async () => {
        const lines: string[] = [];
        const { client, reads, } = clientBefore({ msBeforeReturn: 60, syntheticWet: false, lagging: true, },);
        const refusal = await rejectionOf({
          promise: readPhaseSeats({
            client,
            phase: 'lanes',
            signal: SIGNAL,
            l: levelCapturingLogger({ lines, },),
            pollMs: 5,
          },),
        },);
        expect(reads.count,).toBe(2,);
        expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
        expect((refusal as EveryProviderDryError).message,).toContain('still reads the day spent',);
        expect((refusal as EveryProviderDryError).neuralwattReturnsInMs,).toBeUndefined();
      },
    },),

    it({
      name: 'WAITS FOR A RETURN INSIDE WHAT THE PASS HAS LEFT, past 300 s too: a return 301 away with 600 left (scaled '
        + 'from seconds to milliseconds so the case sleeps for real) is waited for and the NeuralWatt '
        + 'writers are seated',
      fn: async () => {
        const { client, reads, } = clientBefore({ msBeforeReturn: 301, passLeftMs: 600, },);
        const { unreachableWriters, } = await readPhaseSeats({
          client,
          phase: 'lanes',
          signal: SIGNAL,
          l: levelCapturingLogger({ lines: [], },),
          pollMs: 5,
        },);
        expect(unreachableWriters,).toEqual([],);
        expect(reads.count,).toBe(2,);
      },
    },),

    it({
      name: 'REFUSES AT ONCE, STATING THE RETURN TIME, for a return past what the pass has left: 700 away with 600 '
        + 'left (scaled from seconds to milliseconds) is not waited for, since the pass would end first, and the '
        + 'refusal names the time of day in UTC and says the pass has less left',
      fn: async () => {
        // A frozen monotonic clock: the distance stays 700 whatever the host does between the reading and the hold.
        const { client, reads, } = clientBefore({ msBeforeReturn: 700, passLeftMs: 600, now: () => 0, },);
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
        expect((refusal as WritingBenchUnreachableError).message,).toContain('its day starts again at 19:15:08 UTC',);
        expect((refusal as WritingBenchUnreachableError).message,).toContain('this pass has left',);
        expect(reads.count,).toBe(1,);
      },
    },),
  ],
},);
