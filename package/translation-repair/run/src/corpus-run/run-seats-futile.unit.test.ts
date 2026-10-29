/**
 Tests for the holds the seat reader leaves out because the provider's last return served nothing, and for the line that
 says so. The cases over the real budgets and the real reader are in `run-seats-futile-return.unit.test.ts`; these ask
 the helpers directly with scripted holds and marks, to pin the edges: a client that carries no marks, a reading with no
 hold running, a mark on a provider that is not held, and NeuralWatt, whose hold is always a refusal's.

 Fixtures are cat-themed invention; every hold and mark is scripted and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  leaveOutFutileReturns,
  sayFutileSkips,
} from '../../dist/final/node/index.mjs';

/**
 What the budgets report of a return that served nothing: a 300 s hold, refused again 1.2 s after it ended.
 */
const MARK = {
  holdMs: 300_000,
  refusedAfterMs: 1_200,
};

/**
 Holds the cases script: Synthetic and NeuralWatt held, every other provider free.

 @param synthetic - milliseconds Synthetic is held

 @param neuralwatt - milliseconds NeuralWatt is held

 @returns One hold per provider

 @example
 ```ts
 const running = holdsOf({ synthetic: 240_000, neuralwatt: 0, },);
 ```
 */
function holdsOf(
  {
    synthetic,
    neuralwatt,
  }: {
    readonly synthetic: number;
    readonly neuralwatt: number;
  },
): ProviderRecord<number> {
  return providerRecord({
    of: function holdOf(provider: ProviderName,): number {
      if (provider === 'synthetic')
        return synthetic;
      return (provider === 'neuralwatt') ? neuralwatt : 0;
    },
  },);
}

await describe({
  name: leaveOutFutileReturns.name,
  children: [
    it({
      name: 'LEAVES THE HOLDS AS THEY ARE for a client that carries no marks, and for a reading where no hold runs, '
        + 'without asking the client for its marks',
      fn: async () => {
        const running = holdsOf({
          synthetic: 240_000,
          neuralwatt: 0,
        },);
        /**
         How many times the client was asked for its marks.
         */
        const asked = { count: 0, };
        const idle = holdsOf({
          synthetic: 0,
          neuralwatt: 0,
        },);
        const withoutMarks = leaveOutFutileReturns({
          client: {},
          running,
        },);
        const withNoHold = leaveOutFutileReturns({
          client: {
            futileReturns: function futileReturns() {
              asked.count += 1;
              return new Map([['synthetic', MARK,],],);
            },
          },
          running: idle,
        },);

        expect(withoutMarks.held,).toBe(running,);
        expect(withoutMarks.skipped.size,).toBe(0,);
        expect(withNoHold.held,).toBe(idle,);
        expect(withNoHold.skipped.size,).toBe(0,);
        expect(asked.count,).toBe(0,);
      },
    },),

    it({
      name: 'LEAVES OUT THE HOLD OF A HELD PROVIDER WHOSE LAST RETURN SERVED NOTHING and keeps the others, naming the '
        + 'return and how long the hold would have been waited for; a mark on a provider that is not held leaves '
        + 'nothing out',
      fn: async () => {
        const running = holdsOf({
          synthetic: 240_000,
          neuralwatt: 0,
        },);
        const { held, skipped, } = leaveOutFutileReturns({
          client: {
            futileReturns: function futileReturns() {
              return new Map([
                [
                  'synthetic',
                  MARK,
                ],
                [
                  'hyper',
                  MARK,
                ],
              ],);
            },
          },
          running,
        },);

        expect(held.synthetic,).toBe(0,);
        expect(held.hyper,).toBe(0,);
        expect([...skipped.keys(),],).toEqual(['synthetic',],);
        expect(skipped.get('synthetic',),).toEqual({
          ...MARK,
          holdLeftMs: 240_000,
        },);
      },
    },),

    it({
      name: 'LEAVES OUT NEURALWATT LIKE ANY OTHER PROVIDER when a refusal holds it, since its day start is no hold and '
        + 'every hold the reader sees on it is a refusal\'s',
      fn: async () => {
        const running = holdsOf({
          synthetic: 0,
          neuralwatt: 600_000,
        },);
        const refusal = leaveOutFutileReturns({
          client: {
            futileReturns: function futileReturns() {
              return new Map<ProviderName, typeof MARK>([['neuralwatt', MARK,],],);
            },
          },
          running,
        },);

        expect(refusal.held.neuralwatt,).toBe(0,);
        expect([...refusal.skipped.keys(),],).toEqual(['neuralwatt',],);
      },
    },),

    it({
      name: 'SAYS EACH HOLD LEFT OUT ON A LINE OF ITS OWN with the phase, the provider, the hold that ended, how soon the '
        + 'provider refused again and the hold that is not waited for, and says nothing when none is left out',
      fn: async () => {
        /**
         Lines the helper wrote.
         */
        const lines: string[] = [];
        sayFutileSkips({
          skipped: new Map(),
          phase: 'lane contest',
          l: levelCapturingLogger({ lines, },),
        },);
        const none = lines.length;
        sayFutileSkips({
          skipped: new Map([
            [
              'synthetic',
              {
                ...MARK,
                holdLeftMs: 240_000,
              },
            ],
          ],),
          phase: 'lane contest',
          l: levelCapturingLogger({ lines, },),
        },);

        expect(none,).toBe(0,);
        /**
         The one line, worded as the reading words it.
         */
        const expected = 'warn JUDGE SEATS phase=lane contest not waiting for synthetic: its last return served nothing (its '
          + '300000ms hold ended 1200ms before it refused again, and no call it answered came after), so its hold of '
          + '240000ms is not waited for while a bench is short; it is waited for again once it answers a call';
        expect(lines,).toEqual([expected,],);
      },
    },),
  ],
},);
