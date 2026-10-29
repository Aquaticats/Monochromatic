/**
 Tests for the per-run spend ceiling: the resolver, the predicate the
 scheduler asks before each entry, and the line it prints when it stops.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  resolveSpendCeilingUsd,
  SPEND_CEILING_PROVIDERS,
  SPEND_CEILING_USD,
  SPEND_CEILING_VAR,
  spendCeilingNote,
  spendCeilingOverrideNote,
  spendCeilingReached,
  SpendCeilingOverrideError,
} from '../../dist/final/node/index.mjs';

/**
 Built-in the resolver falls back to in these cases.
 */
const FALLBACK = 7;

/**
 Points the process's own ceiling variable at a value for one case, and puts
 it back as the case found it.

 @param says - value the process's variable holds while the case runs

 @returns Disposable restoring the variable, its absence included

 @example
 ```ts
 using exported = ceilingSaying({ says: 'plenty', },);
 ```
 */
function ceilingSaying({ says, }: { readonly says: string; },): Disposable {
  /**
   The variable as this process found it, put back when the case ends.
   */
  const found = process.env[SPEND_CEILING_VAR];
  process.env[SPEND_CEILING_VAR] = says;
  return {
    [Symbol.dispose]: function restoreCeiling(): void {
      if (found === undefined)
        Reflect.deleteProperty(process.env, SPEND_CEILING_VAR,);
      else
        process.env[SPEND_CEILING_VAR] = found;
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: resolveSpendCeilingUsd.name,
      // ONE AT A TIME: a case points the process-wide ceiling variable
      // elsewhere while the read runs (ledger B79).
      concurrency: 1,
      children: [
        it({
          name: 'RETURNS the built-in for an unset or blank override, since neither is an override',
          fn: async () => {
            expect(resolveSpendCeilingUsd({
              fallback: FALLBACK,
              raw: '',
            },),).toBe(FALLBACK,);
            expect(resolveSpendCeilingUsd({
              fallback: FALLBACK,
              raw: '   ',
            },),).toBe(FALLBACK,);
          },
        },),
        it({
          name: 'READS a number of USD, zero included, since zero means start nothing and is how the guard '
            + 'is shown to fire on a live run without spending',
          fn: async () => {
            expect(resolveSpendCeilingUsd({
              fallback: FALLBACK,
              raw: '5.5',
            },),).toBe(5.5,);
            expect(resolveSpendCeilingUsd({
              fallback: FALLBACK,
              raw: '0',
            },),).toBe(0,);
          },
        },),
        it({
          name: 'REFUSES a value that is not a non-negative number, naming the variable and quoting the '
            + 'value, rather than replacing it by the default an operator did not ask for',
          fn: async () => {
            for (const raw of ['plenty', '-1', '5 dollars', 'Infinity',]) {
              /**
               What the resolver raised for this value.
               */
              const refusal = caught(function readUnreadable(): number {
                return resolveSpendCeilingUsd({
                  fallback: FALLBACK,
                  raw,
                },);
              },);
              expect(refusal,).toBeInstanceOf(SpendCeilingOverrideError,);
              expect((refusal as Error).message,).toContain(SPEND_CEILING_VAR,);
              expect((refusal as Error).message,).toContain(JSON.stringify(raw,),);
              expect((refusal as SpendCeilingOverrideError).messageNamesOnly,).toBe(true,);
            }
          },
        },),
        it({
          name: 'REFUSES a value not written as a plain decimal, which `Number` read as dollars nobody typed: a '
            + 'hexadecimal, an exponent, a sign, a space either side and a point missing digits on one side; and a '
            + 'digit run past the range a double holds (ledger B73)',
          fn: async () => {
            /**
             Spellings `Number` reads as an amount that no operator writes as one.
             */
            const spellings = [
              '0x10',
              '1e1',
              '+15',
              ' 15',
              '15 ',
              '15.',
              '.5',
              `1${'0'.repeat(400,)}`,
            ];
            expect(spellings.map(function refusedOf(raw,): boolean {
              return caught(function readSpelling(): number {
                return resolveSpendCeilingUsd({
                  fallback: FALLBACK,
                  raw,
                },);
              },) instanceof SpendCeilingOverrideError;
            },),).toEqual(spellings.map(function refused(): boolean {
              return true;
            },),);
          },
        },),
        it({
          name: 'READS ONLY THE TEXT IT IS HANDED, never the process\'s own variable, whose unreadable value '
            + 'would refuse the read and whose amount would replace the fallback',
          fn: async () => {
            {
              using refused = ceilingSaying({ says: 'plenty', },);
              expect(resolveSpendCeilingUsd({
                fallback: FALLBACK,
                raw: '5.5',
              },),).toBe(5.5,);
            }
            {
              using exported = ceilingSaying({ says: '12.5', },);
              expect(resolveSpendCeilingUsd({
                fallback: FALLBACK,
                raw: '',
              },),).toBe(FALLBACK,);
            }
          },
        },),
      ],
    },),

    describe({
      name: spendCeilingReached.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STOPS at or past the ceiling and not below it, so a ceiling of zero refuses the first entry',
          fn: async () => {
            expect(spendCeilingReached({
              spentUsd: 19.99,
              ceilingUsd: 20,
            },),).toBe(false,);
            expect(spendCeilingReached({
              spentUsd: 20,
              ceilingUsd: 20,
            },),).toBe(true,);
            expect(spendCeilingReached({
              spentUsd: 20.01,
              ceilingUsd: 20,
            },),).toBe(true,);
            expect(spendCeilingReached({
              spentUsd: 0,
              ceilingUsd: 0,
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'SPEND_CEILING_PROVIDERS',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'METERS every provider whose spend no balance bounds, OpenRouter, DeepSeek and Xiaomi, each against '
            + 'the one built-in allowance',
          fn: async () => {
            expect(SPEND_CEILING_PROVIDERS,).toContain('deepseek',);
            expect(SPEND_CEILING_PROVIDERS,).toContain('openrouter',);
            expect(SPEND_CEILING_PROVIDERS,).toContain('xiaomi',);
          },
        },),
      ],
    },),

    describe({
      name: spendCeilingNote.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES both figures, the metered provider, and the dial that raises the allowance',
          fn: async () => {
            /**
             The line the scheduler prints when it stops.
             */
            const note = spendCeilingNote({
              provider: 'xiaomi',
              spentUsd: 20.4,
              ceilingUsd: 20,
            },);
            expect(note.startsWith('SPEND CEILING reached',),).toBe(true,);
            expect(note,).toContain('20.4 of 20 USD',);
            expect(note,).toContain('USD spent on xiaomi this run',);
            expect(note,).toContain(SPEND_CEILING_VAR,);
          },
        },),
      ],
    },),

    describe({
      name: spendCeilingOverrideNote.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS NOTHING UNDER THE BUILT-IN, and names an overridden allowance with the built-in, the metered provider '
            + 'and the dial that set it, so a run never hides which ceiling it ran under (ledger T8)',
          fn: async () => {
            expect(spendCeilingOverrideNote({ ceilingUsd: SPEND_CEILING_USD, },),).toBe('',);
            /**
             The line for a launch that raised the allowance.
             */
            const note = spendCeilingOverrideNote({ ceilingUsd: SPEND_CEILING_USD + FALLBACK, },);
            expect(note.startsWith(`SPEND CEILING OVERRIDDEN by ${SPEND_CEILING_VAR}`,),).toBe(true,);
            expect(note,).toContain(
              `${String(SPEND_CEILING_USD + FALLBACK,)} USD on any one of ${SPEND_CEILING_PROVIDERS.join(', ',)}`,
            );
            expect(note,).toContain(`rather than the built-in ${String(SPEND_CEILING_USD,)}`,);
          },
        },),
      ],
    },),
  ],
},);
