/**
 Tests for the marks of a return that served nothing, asked directly with the numbers a refusal arrives with: the hold
 it replaces, when it arrived, when the provider last answered a call and when the entry began. The cases the budgets
 reach through real refusals are in `provider-budget-futile-return.unit.test.ts`; these pin the edges of the rule, a
 refusal at the instant a hold ends and a call served at that instant, and the hold that held for no time.

 The logger is one that keeps its lines, so nothing is printed.

 Fixtures are cat-themed invention; no meter is read and no provider is asked.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { createFutileReturns, } from '../dist/final/node/index.mjs';

/**
 Hold the cases replace: set for 300 s and ended at instant 400,000.
 */
const EARLIER = {
  holdMs: 300_000,
  endsAt: 400_000,
};

/**
 Marks a refusal with the inputs given and the rest of a refusal a long way from any edge.

 @param earlier - hold the refusal replaces

 @param at - when the refusal arrived

 @param servedAt - when the provider last answered a call

 @param entryStartedAt - when the entry began

 @returns Marks after the refusal, and the lines written

 @example
 ```ts
 const { marks, } = refuse({ earlier: EARLIER, at: 401_000, servedAt: -Infinity, entryStartedAt: 0, },);
 ```
 */
function refuse(
  {
    earlier,
    at,
    servedAt,
    entryStartedAt,
  }: {
    readonly earlier: { readonly holdMs: number; readonly endsAt: number; };
    readonly at: number;
    readonly servedAt: number;
    readonly entryStartedAt: number;
  },
): {
  readonly marks: ReadonlyMap<string, { readonly holdMs: number; readonly refusedAfterMs: number; }>;
  readonly lines: readonly string[];
} {
  /**
   Lines the marks wrote.
   */
  const lines: string[] = [];
  /**
   The marks under test.
   */
  const futile = createFutileReturns({
    l: levelCapturingLogger({ lines, },),
    entryWords: function entryWords(): string {
      return 'entry CatEntry';
    },
  },);
  futile.note({
    provider: 'synthetic',
    earlier,
    at,
    servedAt,
    entryStartedAt,
  },);
  return {
    marks: futile.current(),
    lines,
  };
}

await describe({
  name: createFutileReturns.name,
  children: [
    it({
      name: 'MARKS A REFUSAL THAT ARRIVES AT THE INSTANT THE HOLD ENDS: the hold has run out, and the refusal is the first '
        + 'answer after it',
      fn: async () => {
        const { marks, lines, } = refuse({
          earlier: EARLIER,
          at: 400_000,
          servedAt: Number.NEGATIVE_INFINITY,
          entryStartedAt: 0,
        },);

        expect(marks.get('synthetic',),).toEqual({
          holdMs: 300_000,
          refusedAfterMs: 0,
        },);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('entry CatEntry does not wait for synthetic\'s holds to end',);
      },
    },),

    it({
      name: 'MARKS NOTHING FOR A REFUSAL INSIDE THE HOLD, one instant before it ends',
      fn: async () => {
        const { marks, lines, } = refuse({
          earlier: EARLIER,
          at: 399_999,
          servedAt: Number.NEGATIVE_INFINITY,
          entryStartedAt: 0,
        },);

        expect(marks.size,).toBe(0,);
        expect(lines,).toEqual([],);
      },
    },),

    it({
      name: 'MARKS NOTHING WHEN A CALL WAS SERVED AT THE INSTANT THE HOLD ENDED, and marks it when the call was served one '
        + 'instant before',
      fn: async () => {
        expect(
          refuse({
            earlier: EARLIER,
            at: 401_000,
            servedAt: 400_000,
            entryStartedAt: 0,
          },).marks.size,
        ).toBe(0,);
        expect(
          refuse({
            earlier: EARLIER,
            at: 401_000,
            servedAt: 399_999,
            entryStartedAt: 0,
          },).marks.size,
        ).toBe(1,);
      },
    },),

    it({
      name: 'MARKS NOTHING FOR A HOLD THAT HELD FOR NO TIME: a refusal on a wet meter with no other wet provider sets one, '
        + 'and a refusal after it is not a provider coming back',
      fn: async () => {
        const { marks, lines, } = refuse({
          earlier: {
            holdMs: 0,
            endsAt: 400_000,
          },
          at: 401_000,
          servedAt: Number.NEGATIVE_INFINITY,
          entryStartedAt: 0,
        },);

        expect(marks.size,).toBe(0,);
        expect(lines,).toEqual([],);
      },
    },),

    it({
      name: 'MARKS NOTHING FOR A HOLD THAT RAN OUT BEFORE THE ENTRY BEGAN, and marks one that ended at the instant it began',
      fn: async () => {
        expect(
          refuse({
            earlier: EARLIER,
            at: 401_000,
            servedAt: Number.NEGATIVE_INFINITY,
            entryStartedAt: 400_001,
          },).marks.size,
        ).toBe(0,);
        expect(
          refuse({
            earlier: EARLIER,
            at: 401_000,
            servedAt: Number.NEGATIVE_INFINITY,
            entryStartedAt: 400_000,
          },).marks.size,
        ).toBe(1,);
      },
    },),

    it({
      name: 'FORGETS ONE PROVIDER WHEN IT SERVES AND EVERY PROVIDER WHEN THE NEXT ENTRY BEGINS',
      fn: async () => {
        /**
         The marks under test.
         */
        const futile = createFutileReturns({
          l: levelCapturingLogger({ lines: [], },),
          entryWords: function entryWords(): string {
            return 'the run';
          },
        },);
        for (const provider of [
          'synthetic',
          'hyper',
        ] as const) {
          futile.note({
            provider,
            earlier: EARLIER,
            at: 401_000,
            servedAt: Number.NEGATIVE_INFINITY,
            entryStartedAt: 0,
          },);
        }
        expect([...futile.current().keys(),],).toEqual([
          'synthetic',
          'hyper',
        ],);

        futile.served({ provider: 'synthetic', },);
        expect([...futile.current().keys(),],).toEqual(['hyper',],);

        futile.forgetAll();
        expect(futile.current().size,).toBe(0,);
      },
    },),
  ],
},);
