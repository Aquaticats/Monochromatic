/**
 Tests the seat reader's refusal against the XingZ6019 shape: NeuralWatt dry because the day's energy was spent, its day
 starting again 48 s after the entry stopped, and a writing bench short of its floor without it.

 The reader waits for no NeuralWatt return (the owner's answer of 2026-10-29, "Never wait, relaunch";
 `run-seats-neuralwatt-never-waits.unit.test.ts` holds the cases that show it refuses at once). Where NeuralWatt is dry
 for a reason that is not the day's energy, the refusal states only the true cause. The budgets are the real ones over
 stub meters and a stub wall clock.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

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
  name: 'the seat reader and NeuralWatt\'s dryness',
  concurrency: 1,
  children: [
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
      name: 'STATES THE TIME OF DAY IN UTC AND THE DISTANCE for a spent day, whatever the distance: 700 ms away '
        + '(scaled from seconds) is refused at once, as 60 ms is',
      fn: async () => {
        // A frozen monotonic clock: the distance stays 700 whatever the host does between the reading and the refusal.
        const { client, reads, } = clientBefore({ msBeforeReturn: 700, now: () => 0, },);
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
        expect((refusal as WritingBenchUnreachableError).message,).toContain('(in 1 s)',);
        expect(reads.count,).toBe(1,);
      },
    },),
  ],
},);
