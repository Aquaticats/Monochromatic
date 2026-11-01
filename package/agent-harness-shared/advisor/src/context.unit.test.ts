/**
 Unit tests for Advisor context serialization.
 
 @module
 */

import type {
  Api,
  Model,
} from '@earendil-works/pi-ai';
import type { SessionEntry, } from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type AdvisorConfig,
  ADVISOR_MESSAGE_TYPE,
  buildAdvisorContext,
  DEFAULT_CONFIG,
  maxContextCharsForAdvisorModel,
  truncateContext,
} from '../dist/final/node/index.mjs';
import { testAdvisorTranscriptSerialization, } from './transcript-test-fixture.ts';

//region Fixtures

/** Context truncation budget for tests. */
const TRUNCATION_BUDGET = 5;

/** Stable timestamp for fixture entries. */
const TIMESTAMP = '2026-05-15T00:00:00.000Z';

/** Small model context window used in budget tests. */
const SMALL_CONTEXT_WINDOW = 5_000;

/** Large model context window used in budget tests. */
const LARGE_CONTEXT_WINDOW = 10_000;

/** Fixture max output tokens. */
const MAX_TOKENS = 1_000;

/** Focused Advisor question fixture. */
const FOCUS_QUESTION = 'Which assumption is weakest?';

/** Output token budget used in context-budget tests. */
const OUTPUT_TOKEN_BUDGET = 100;

/** Configured context cap used in context-budget tests. */
const CONFIGURED_CONTEXT_CAP = 10;

/** Advisor config fixture with prior Advisor results omitted. */
const omitPriorAdvisorConfig: AdvisorConfig = {
  ...DEFAULT_CONFIG,
  includePriorAdvisorResults: false,
  source: {
    globalPath: '/home/test/.pi/agent/extensions/pi-advisor.json',
    projectPath: '/repo/.pi/extensions/pi-advisor.json',
    globalLoaded: false,
    projectLoaded: false,
  },
};

/** Advisor config fixture with no configured context cap. */
const dynamicBudgetConfig: AdvisorConfig = {
  ...omitPriorAdvisorConfig,
  maxAdvisorOutputTokens: OUTPUT_TOKEN_BUDGET,
};

/** Advisor config fixture with configured context cap. */
const cappedBudgetConfig: AdvisorConfig = {
  ...dynamicBudgetConfig,
  maxContextChars: CONFIGURED_CONTEXT_CAP,
};

/** Advisor custom message from a previous manual `/advisor` run. */
const priorAdvisorMessage: SessionEntry = {
  type: 'custom_message',
  id: 'advisor-message',
  parentId: null,
  timestamp: TIMESTAMP,
  customType: ADVISOR_MESSAGE_TYPE,
  content: 'prior advisor text',
  display: true,
};

/** Non-Advisor custom message that should remain visible. */
const otherCustomMessage: SessionEntry = {
  type: 'custom_message',
  id: 'other-message',
  parentId: 'advisor-message',
  timestamp: TIMESTAMP,
  customType: 'other-extension',
  content: 'other extension text',
  display: true,
};

/**
 Build fixture Advisor model.
 
 @param contextWindow - token context window
 
 @returns fixture model
 */
function fixtureModel(
  contextWindow: number,
): Model<Api> {
  return {
    id: 'reviewer',
    name: 'Reviewer',
    api: 'faux',
    provider: 'faux-provider',
    baseUrl: 'https://example.invalid',
    reasoning: false,
    input: ['text',],
    cost: {
      input: 1,
      output: 1,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow,
    maxTokens: MAX_TOKENS,
  } satisfies Model<Api>;
}

//endregion Fixtures

await describe({
  name: buildAdvisorContext.name,
  children: [
    it({
      name: 'omits prior Advisor custom messages when configured',
      fn: async () => {
        const context = buildAdvisorContext({
          branch: [
            priorAdvisorMessage,
            otherCustomMessage,
          ],
          transcript: testAdvisorTranscriptSerialization,
          config: omitPriorAdvisorConfig,
          advisorSystemPrompt: 'review carefully',
        },);
        expect(context.text,).not.toContain('prior advisor text',);
        expect(context.text,).toContain('other extension text',);
      },
    },),
    it({
      name: 'counts focused question in input token estimate',
      fn: async () => {
        const contextWithoutQuestion = buildAdvisorContext({
          branch: [otherCustomMessage,],
          transcript: testAdvisorTranscriptSerialization,
          config: omitPriorAdvisorConfig,
          advisorSystemPrompt: 'review carefully',
        },);
        const contextWithQuestion = buildAdvisorContext({
          branch: [otherCustomMessage,],
          transcript: testAdvisorTranscriptSerialization,
          config: omitPriorAdvisorConfig,
          advisorSystemPrompt: 'review carefully',
          question: FOCUS_QUESTION,
        },);
        expect(contextWithQuestion.estimatedInputTokens,).toBeGreaterThan(
          contextWithoutQuestion.estimatedInputTokens,
        );
      },
    },),
  ],
},);

await describe({
  name: truncateContext.name,
  children: [
    it({
      name: 'preserves head and tail when truncating',
      fn: async () => {
        const result = truncateContext({
          text: 'abcdefghij',
          maxChars: TRUNCATION_BUDGET,
        },);
        expect(result.truncated,).toBe(true,);
        expect(result.text,).toContain(
          'advisor: middle of serialized conversation omitted',
        );
      },
    },),
  ],
},);

await describe({
  name: maxContextCharsForAdvisorModel.name,
  children: [
    it({
      name: 'derives a larger budget for a larger model context window',
      fn: async () => {
        const smallBudget = maxContextCharsForAdvisorModel({
          transcript: testAdvisorTranscriptSerialization,
          config: dynamicBudgetConfig,
          model: fixtureModel(SMALL_CONTEXT_WINDOW,),
          advisorSystemPrompt: 'review carefully',
        },);
        const largeBudget = maxContextCharsForAdvisorModel({
          transcript: testAdvisorTranscriptSerialization,
          config: dynamicBudgetConfig,
          model: fixtureModel(LARGE_CONTEXT_WINDOW,),
          advisorSystemPrompt: 'review carefully',
        },);
        expect(largeBudget,).toBeGreaterThan(smallBudget,);
      },
    },),
    it({
      name: 'honors configured context cap below model-derived budget',
      fn: async () => {
        const budget = maxContextCharsForAdvisorModel({
          transcript: testAdvisorTranscriptSerialization,
          config: cappedBudgetConfig,
          model: fixtureModel(LARGE_CONTEXT_WINDOW,),
          advisorSystemPrompt: 'review carefully',
        },);
        expect(budget,).toBe(CONFIGURED_CONTEXT_CAP,);
      },
    },),
    it({
      name: 'reserves context budget for focused question',
      fn: async () => {
        const budgetWithoutQuestion = maxContextCharsForAdvisorModel({
          transcript: testAdvisorTranscriptSerialization,
          config: dynamicBudgetConfig,
          model: fixtureModel(LARGE_CONTEXT_WINDOW,),
          advisorSystemPrompt: 'review carefully',
        },);
        const budgetWithQuestion = maxContextCharsForAdvisorModel({
          transcript: testAdvisorTranscriptSerialization,
          config: dynamicBudgetConfig,
          model: fixtureModel(LARGE_CONTEXT_WINDOW,),
          advisorSystemPrompt: 'review carefully',
          question: FOCUS_QUESTION,
        },);
        expect(budgetWithQuestion,).toBeLessThan(budgetWithoutQuestion,);
      },
    },),
  ],
},);
