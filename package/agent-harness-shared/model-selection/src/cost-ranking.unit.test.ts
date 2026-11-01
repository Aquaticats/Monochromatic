/**
 Unit tests for cost ranking helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type EffectiveModelScope,
  buildCostRanking,
  compareCostScores,
  scoreModel,
  selectDefaultModel,
  selectDefaultModelFromContextEstimates,
} from '../dist/final/node/index.mjs';
import {
  CHEAP_INPUT,
  CHEAP_OUTPUT,
  EXPENSIVE_INPUT,
  EXPENSIVE_OUTPUT,
  fixtureModel,
} from './test-fixtures.ts';

//region Fixtures

/** Cheap model fixture. */
const cheapModel = fixtureModel({
  provider: 'cheap',
  id: 'reviewer',
  inputCost: CHEAP_INPUT,
  outputCost: CHEAP_OUTPUT,
},);

/** Expensive model fixture. */
const expensiveModel = fixtureModel({
  provider: 'expensive',
  id: 'reviewer',
  inputCost: EXPENSIVE_INPUT,
  outputCost: EXPENSIVE_OUTPUT,
},);

/** Effective scope fixture. */
const scope: EffectiveModelScope = {
  source: 'available',
  entries: [
    {
      model: cheapModel,
      canonicalSlug: 'cheap/reviewer',
    },
    {
      model: expensiveModel,
      canonicalSlug: 'expensive/reviewer',
    },
  ],
};

/** Input token estimate fixture. */
const INPUT_TOKENS = 20;

/** Output token budget fixture. */
const OUTPUT_TOKENS = 10;

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: scoreModel.name,
      children: [
        it({
          name: 'computes expected cost score',
          fn: async function testScoreModel() {
            const [entry,] = scope.entries;
            if (entry === undefined)
              throw new Error('missing fixture entry',);
            const score = scoreModel({
              entry,
              estimatedInputTokens: INPUT_TOKENS,
              maxOutputTokens: OUTPUT_TOKENS,
            },);
            expect(score.expectedCost,).toBe(
              (INPUT_TOKENS * CHEAP_INPUT) + (OUTPUT_TOKENS * CHEAP_OUTPUT),
            );
          },
        },),
      ],
    },),
    describe({
      name: compareCostScores.name,
      children: [
        it({
          name: 'uses slug as final tie-breaker',
          fn: async function testCompareCostScores() {
            expect(compareCostScores({
              left: {
                slug: 'a/model',
                inputTokens: 1,
                maxOutputTokens: 1,
                expectedCost: 1,
                inputCost: 1,
                outputCost: 1,
                contextWindow: 1,
              },
              right: {
                slug: 'b/model',
                inputTokens: 1,
                maxOutputTokens: 1,
                expectedCost: 1,
                inputCost: 1,
                outputCost: 1,
                contextWindow: 1,
              },
            },),)
              .toBeLessThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: buildCostRanking.name,
      children: [
        it({
          name: 'sorts highest expected cost first',
          fn: async function testBuildCostRanking() {
            const ranking = buildCostRanking({
              scope,
              estimatedInputTokensBySlug: new Map([
                ['cheap/reviewer', INPUT_TOKENS,],
                ['expensive/reviewer', INPUT_TOKENS,],
              ],),
              maxOutputTokens: OUTPUT_TOKENS,
              errorPrefix: 'advisor',
            },);
            expect(ranking[0]?.slug,).toBe('expensive/reviewer',);
          },
        },),
      ],
    },),
    describe({
      name: selectDefaultModel.name,
      children: [
        it({
          name: 'selects highest expected-cost model',
          fn: async function testSelectDefaultModel() {
            const result = selectDefaultModel({
              scope,
              estimatedInputTokens: INPUT_TOKENS,
              maxOutputTokens: OUTPUT_TOKENS,
            },);
            expect(result.selected.canonicalSlug,).toBe('expensive/reviewer',);
          },
        },),
      ],
    },),
    describe({
      name: selectDefaultModelFromContextEstimates.name,
      children: [
        it({
          name: 'scores each model with its own input-token estimate',
          fn: async function testPerModelEstimates() {
            const result = selectDefaultModelFromContextEstimates({
              scope,
              estimatedInputTokensBySlug: new Map([
                ['cheap/reviewer', 1_000,],
                ['expensive/reviewer', 0,],
              ],),
              maxOutputTokens: OUTPUT_TOKENS,
            },);
            expect(result.selected.canonicalSlug,).toBe('cheap/reviewer',);
          },
        },),
      ],
    },),
  ],
},);
