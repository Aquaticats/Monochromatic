/**
 Tests that the seat reader does not wait for NeuralWatt's day return.

 THE OWNER'S ANSWER OF 2026-10-29 is "Never wait, relaunch": "Stop at once and relaunch later." The reader used to wait
 for a return that fitted what the pass had left, as it waits for Hyper's stated holds; it now refuses at once, in the
 XingZ6019 shape (NeuralWatt dry because the day's energy was spent, its day starting again 60 ms later, scaled from
 seconds, and a writing bench short of its floor without it) and with every provider dry. The refusal still states the
 time of day in UTC, so the operator knows when to relaunch. The budgets are the real ones over stub meters and a stub
 wall clock, the fake clock stands under every sleep, and no HTTP is made.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  readPhaseSeats,
  WritingBenchUnreachableError,
} from '../../dist/final/node/index.mjs';
import { clientBefore, } from './neuralwatt-seat-client.test-fixture.ts';
import { readOnFakeClock, } from './run-seats-fake-clock.test-fixture.ts';

/**
 Reads the lanes' seats over the NeuralWatt client while a fake clock passes far beyond the return, for a reading that
 must end in a refusal, and reports the refusal with the energy reads counted and the lines logged.

 @param sinon - case's own sandbox, which fakes the timers
 @param syntheticWet - whether Synthetic, the only other configured provider, has budget

 @returns Refusal the reading threw, the energy reads counted and the lines the reading logged

 @example
 ```ts
 const { refusal, reads, } = await refusalWithoutWaiting({ sinon: ctx.sinon, syntheticWet: true, },);
 ```
 */
async function refusalWithoutWaiting(
  {
    sinon,
    syntheticWet,
  }: {
    readonly sinon: DisposableSandbox;
    readonly syntheticWet: boolean;
  },
): Promise<{
  readonly refusal: unknown;
  readonly reads: number;
  readonly lines: readonly string[];
}> {
  return await readOnFakeClock({
    sinon,
    holdMs: 1_000,
    pollMs: 5,
    reading: async function reading({ signal, now, },) {
      const lines: string[] = [];
      const { client, reads, } = clientBefore({ msBeforeReturn: 60, syntheticWet, now, },);
      const refusal = await rejectionOf({
        promise: readPhaseSeats({
          client,
          phase: 'lanes',
          signal,
          l: levelCapturingLogger({ lines, },),
          pollMs: 5,
        },),
      },);
      return {
        refusal,
        reads: reads.count,
        lines,
      };
    },
  },);
}

await describe({
  name: 'the seat reader does not wait for NeuralWatt\'s return',
  concurrency: 1,
  children: [
    it({
      name: 'REFUSES AT ONCE, STATING THE RETURN TIME, WHEN A WRITING BENCH STANDS SHORT WITHOUT NEURALWATT: the return '
        + 'is 60 ms away and fits any bound, and the reader neither waits for it nor reads the meters a second time',
      fn: async ctx => {
        const { refusal, reads, lines, } = await refusalWithoutWaiting({ sinon: ctx.sinon, syntheticWet: true, },);
        expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
        expect((refusal as WritingBenchUnreachableError).message,).toContain('its day starts again at 19:15:08 UTC',);
        expect(reads,).toBe(1,);
        expect(lines.some(function waitLine(line,): boolean {
          return line.includes('waiting ',) && line.includes('for the shortest hold to end',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES WITH THE ROUTER\'S REFUSAL WHEN EVERY PROVIDER IS DRY, with the time of day in its message and no '
        + 'wait: Synthetic spent too and NeuralWatt 60 ms from its day start ends the run on the first reading',
      fn: async ctx => {
        const { refusal, reads, } = await refusalWithoutWaiting({ sinon: ctx.sinon, syntheticWet: false, },);
        expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
        expect((refusal as EveryProviderDryError).message,).toContain('at 19:15:08 UTC',);
        expect(reads,).toBe(1,);
      },
    },),
  ],
},);
