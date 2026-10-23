/**
 Holds the run's Claude seating to what the owner asked (C6 of the review of the Claude route): "Sonnet edits, Haiku
 wide", with the effort cards left to the bench. The constants are literals typed as roster ids, so a later edit could
 add an effort card, drop the Haiku checker place or seat two cards of one model together without a test failing;
 these cases fail on each.

 THE POSITIVE CONTROLS COME WITH THE NEGATIVE ONES: a view with Claude wet seats Haiku on the checker bench, and the
 same view with Claude dry does not, so the absence is read against a case shown able to move.

 NOTHING HERE REACHES A PROVIDER: the views are invented budget readings.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { assertOneWireModelPerBench, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import {
  judgeSeatsFor,
  RUN_BENCHES,
  RUN_CHECKER_ORDER,
  RUN_MODELS,
} from '../../dist/final/node/index.mjs';

/**
 Every provider wet.
 */
const ALL_WET: BudgetView = {
  neuralwatt: false,
  synthetic: false,
  hyper: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: false,
};

/**
 The reading in which no wet provider serves the second measured checker (Mimo is served by Xiaomi and OpenRouter),
 so the bench needs a fourth place, with Claude wet.
 */
const MIMO_UNSERVED_CLAUDE_WET: BudgetView = {
  ...ALL_WET,
  xiaomi: true,
  openrouter: true,
};

/**
 The same reading with Claude dry.
 */
const MIMO_UNSERVED_CLAUDE_DRY: BudgetView = {
  ...MIMO_UNSERVED_CLAUDE_WET,
  anthropic: true,
};

await describe({
  name: 'the run\'s Claude seating',
  children: [
    it({
      name: 'SEATS SONNET 5.5 AS AN EDITOR, directly ahead of glm-5.3, and no effort card of it or of Haiku',
      fn: async () => {
        const editors = RUN_MODELS.editorModelIds;
        expect(editors,).toContain('claude-sonnet-5-5',);
        expect(editors.indexOf('claude-sonnet-5-5',),).toBe(editors.indexOf('glm-5.3',) - 1,);
        expect(editors.filter(function isHaiku(modelId,): boolean {
          return modelId.startsWith('claude-haiku',);
        },),).toEqual([],);
      },
    },),

    it({
      name: 'PUTS HAIKU 5.5 FOURTH IN THE CHECKER ORDER, behind the three the benchmark ranks, so it is not on the '
        + 'static bench of three',
      fn: async () => {
        expect(RUN_CHECKER_ORDER.indexOf('claude-haiku-5-5',),).toBe(3,);
        expect(RUN_MODELS.checkerModelIds,).not.toContain('claude-haiku-5-5',);
      },
    },),

    it({
      name: 'FILLS A CHECKER PLACE WITH HAIKU WHEN A MEASURED CHECKER\'S PROVIDERS ARE DRY AND CLAUDE IS WET, and '
        + 'with the next measured seat when Claude is dry too (the positive control for the absence)',
      fn: async () => {
        expect(judgeSeatsFor({ dry: ALL_WET, },).repairModels.checkerModelIds,).not.toContain('claude-haiku-5-5',);
        expect(judgeSeatsFor({ dry: MIMO_UNSERVED_CLAUDE_WET, },).repairModels.checkerModelIds,).toContain('claude-haiku-5-5',);
        expect(judgeSeatsFor({ dry: MIMO_UNSERVED_CLAUDE_DRY, },).repairModels.checkerModelIds,)
          .not.toContain('claude-haiku-5-5',);
      },
    },),

    it({
      name: 'LISTS NO EFFORT CARD IN ANY HAND-WRITTEN LIST: the effort cards are for the bench, and no list of the run '
        + 'names one',
      fn: async () => {
        /**
         Every list the run writes down by hand or derives from the roster.
         */
        const lists = [
          ...Object.values(RUN_MODELS,).filter(function isList(member,): boolean {
            return Array.isArray(member,);
          },),
          RUN_CHECKER_ORDER,
          ...RUN_BENCHES.map(function listOf({ modelIds, },): readonly string[] {
            return modelIds;
          },),
        ] as readonly (readonly string[])[];
        expect(lists.flat().filter(function isEffortCard(modelId,): boolean {
          return modelId.includes('-effort-',);
        },),).toEqual([],);
      },
    },),

    it({
      name: 'HOLDS ONE WIRE MODEL PER BENCH on every list the run seats, and names every list of RUN_MODELS among them, '
        + 'so a list added later is checked at load',
      fn: async () => {
        for (const { bench, modelIds, } of RUN_BENCHES)
          assertOneWireModelPerBench({ bench, modelIds, },);
        /**
         The benches the load-time check reads, as lists.
         */
        const checked = RUN_BENCHES.map(function listOf({ modelIds, },): readonly string[] {
          return modelIds;
        },);
        for (const [name, modelIds,] of Object.entries(RUN_MODELS,)) {
          if (Array.isArray(modelIds,)) {
            expect(checked.some(function isSame(list,): boolean {
              return list === modelIds;
            },),).toBe(true,);
            expect(name,).not.toBe('',);
          }
        }
        expect(checked.includes(RUN_CHECKER_ORDER,),).toBe(true,);
      },
    },),
  ],
},);
