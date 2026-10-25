/**
 Tests for the cost ceiling the editor fidelity probe prints before any call: what one ask can cost at most, from the
 card's prices and the completion cap, and how it adds up over the trials and the cards. The amount the probe is held
 to is the spend guard's, tested apart.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { CLAUDE_COMPLETION_CAP, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  ceilingLine,
  editorCeilings,
  MAX_ASKS_PER_TRIAL,
} from '../../dist/final/node/index.mjs';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';
import {
  HAIKU_CARD,
  HIGH_CARD,
  LOW_CARD,
} from '../editor-fidelity.test-fixture.ts';

await describe({
  name: '',
  children: [
    describe({
      name: editorCeilings.name,
      children: [
        it({
          name: 'PRICES THE MOST ONE ASK CAN COST: the sheet at one input token a character, and the whole completion cap '
            + 'at the output price, summed over the trials, thinking counted in the cap',
          fn: async () => {
            expect(CLAUDE_COMPLETION_CAP,).toBe(32_000,);
            const [ceiling,] = editorCeilings({
              cardIds: [LOW_CARD,],
              sheetChars: [1_000, 3_000,],
            },);
        // Sonnet 5.5: 2 USD in and 10 out per million tokens.
            expect(ceiling?.cardId,).toBe(LOW_CARD,);
            expect(ceiling?.trials,).toBe(2,);
            expect(ceiling?.firstPassUsd,).toBeCloseTo(0.322 + 0.326, 10,);
          },
        },),

        it({
          name: 'ADDS THE GATHER\'S RETRIES as a worst case beside the first pass: a reply nothing can read is asked '
            + 'again in each retry round and once more in a recovery round',
          fn: async () => {
            expect(MAX_ASKS_PER_TRIAL,).toBe(5,);
            const [ceiling,] = editorCeilings({
              cardIds: [LOW_CARD,],
              sheetChars: [1_000,],
            },);
            expect(ceiling?.worstCaseUsd,).toBeCloseTo(0.322 * MAX_ASKS_PER_TRIAL, 10,);
          },
        },),

        it({
          name: 'PRICES EACH CARD BY ITS OWN MODEL: the same sheet costs the Haiku card a fraction of the Sonnet card, '
            + 'and a card of one wire model is priced the same at every effort',
          fn: async () => {
            const ceilings = editorCeilings({
              cardIds: [LOW_CARD, HIGH_CARD, HAIKU_CARD,],
              sheetChars: [1_000,],
            },);
            expect(ceilings.map(function idOf(ceiling,): string {
              return ceiling.cardId;
            },),).toStrictEqual([LOW_CARD, HIGH_CARD, HAIKU_CARD,],);
            expect(ceilings[0]?.firstPassUsd,).toBeCloseTo(0.322, 10,);
            expect(ceilings[1]?.firstPassUsd,).toBeCloseTo(0.322, 10,);
        // Haiku 5.5: 0.10 USD in and 0.50 out per million tokens.
            expect(ceilings[2]?.firstPassUsd,).toBeCloseTo(0.0001 + 0.016, 10,);
          },
        },),

        it({
          name: 'USES THE LONG-PROMPT TIER once a sheet passes the tier\'s limit, which prices the whole call',
          fn: async () => {
            const [ceiling,] = editorCeilings({
              cardIds: [HAIKU_CARD,],
              sheetChars: [100_001,],
            },);
        // Haiku 5.5 past 100,000 prompt tokens: 0.50 USD in and 2.50 out per million tokens.
            expect(ceiling?.firstPassUsd,).toBeCloseTo(((100_001 * 0.5) / 1e6) + ((32_000 * 2.5) / 1e6), 10,);
          },
        },),

        it({
          name: 'REFUSES A CARD WITH NO PRICE, rather than pricing it at zero: only the Claude cards carry a table, so a '
            + 'ceiling for any other card cannot be printed',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function pricesStranger(): void {
                editorCeilings({
                  cardIds: ['hf:zai-org/GLM-5.3-Flash',],
                  sheetChars: [1_000,],
                },);
              },
            },),).toBe(
              'hf:zai-org/GLM-5.3-Flash has no price table in claude-card-sides.ts, so no ceiling can be printed and the '
                + 'editor fidelity probe will not start; it benches the Claude cards',
            );
          },
        },),
      ],
    },),

    describe({
      name: ceilingLine.name,
      children: [
        it({
          name: 'SAYS THE CARD, THE TRIALS, BOTH FIGURES AND WHAT THEY ARE BUILT FROM, so the line can be checked by hand',
          fn: async () => {
            const [ceiling,] = editorCeilings({
              cardIds: [LOW_CARD,],
              sheetChars: [1_000, 3_000,],
            },);
            expect(ceilingLine({ ceiling: nonNullishOrThrow(ceiling,), },),).toBe(
              `ceiling ${LOW_CARD}: 2 units, one ask each, at most 0.6480 USD; with 5 asks a trial at most 3.2400 USD; `
                + 'priced at 2 USD in and 10 USD out per million tokens, 32000 output tokens an ask, one input token '
                + 'to the character of the sheet',
            );
          },
        },),
      ],
    },),
  ],
},);
