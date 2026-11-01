/**
 Characterization tests for Advisor model-selection behavior before shared extraction.
 
 @module
 */

import type {
  Api,
  Model,
} from '@earendil-works/pi-ai';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type AdvisorConfig,
  type AdvisorModelResolution,
  type EffectiveModelScope,
  selectAdvisorModel,
  selectAdvisorRunContext,
} from '../dist/final/node/index.mjs';
import { testAdvisorTranscriptSerialization, } from './transcript-test-fixture.ts';

//region Fixtures

/** Cheap model input token price. */
const CHEAP_INPUT = 1;

/** Cheap model output token price. */
const CHEAP_OUTPUT = 2;

/** Expensive model input token price. */
const EXPENSIVE_INPUT = 4;

/** Expensive model output token price. */
const EXPENSIVE_OUTPUT = 8;

/** Third model input token price. */
const THIRD_INPUT = 3;

/** Third model output token price. */
const THIRD_OUTPUT = 6;

/** Advisor output token budget used by selection tests. */
const ADVISOR_OUTPUT_TOKENS = 32;

/** Advisor input token estimate used by selection tests. */
const ADVISOR_INPUT_TOKENS = 16;

/** Advertised output capacity just below Advisor requirement. */
const INSUFFICIENT_MAX_TOKENS = ADVISOR_OUTPUT_TOKENS - 1;

/** Fixture context budget. */
const CONTEXT_WINDOW = 4_096;

/** Fixture maximum output tokens. */
const MAX_TOKENS = 512;

/** Runtime Advisor config fixture. */
const advisorConfig: AdvisorConfig = {
  hedgingEnabled: false,
  collectionGraceMs: 30_000,
  enabled: true,
  timeoutMs: 1_000,
  maxAdvisorOutputTokens: ADVISOR_OUTPUT_TOKENS,
  includePriorAdvisorResults: true,
  source: {
    globalPath: '/tmp/global.json',
    projectPath: '/tmp/project.json',
    globalLoaded: false,
    projectLoaded: false,
  },
};

/**
 Build a complete pi model fixture.
 
 @param provider - provider slug
 
 @param id - model id
 
 @param name - display name
 
 @param inputCost - input price per million tokens
 
 @param outputCost - output price per million tokens
 
 @param maxOutputTokens - optional advertised output capacity
 
 @returns pi model fixture
 
 @example
 ```typescript
 modelFixture({ provider: 'openai', id: 'gpt-5', name: 'GPT', inputCost: 1, outputCost: 2 });
 ```
 */
function modelFixture(
  {
    provider,
    id,
    name,
    inputCost,
    outputCost,
    maxOutputTokens,
  }: {
    readonly provider: string;
    readonly id: string;
    readonly name: string;
    readonly inputCost: number;
    readonly outputCost: number;
    readonly maxOutputTokens?: number;
  },
): Model<Api> {
  return {
    id,
    name,
    api: 'faux',
    provider,
    baseUrl: 'https://example.invalid',
    reasoning: false,
    input: ['text',],
    cost: {
      input: inputCost,
      output: outputCost,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: CONTEXT_WINDOW,
    maxTokens: maxOutputTokens
      ?? MAX_TOKENS,
  } satisfies Model<Api>;
}

/** Cheap scoped model fixture. */
const cheapModel = modelFixture({
  provider: 'cheap',
  id: 'reviewer',
  name: 'Reviewer Cheap',
  inputCost: CHEAP_INPUT,
  outputCost: CHEAP_OUTPUT,
},);

/** Expensive scoped model fixture. */
const expensiveModel = modelFixture({
  provider: 'expensive',
  id: 'reviewer',
  name: 'Reviewer Expensive',
  inputCost: EXPENSIVE_INPUT,
  outputCost: EXPENSIVE_OUTPUT,
},);

/** High-cost model whose endpoint advertises too little output capacity. */
const insufficientOutputModel = modelFixture({
  provider: 'limited',
  id: 'limited-reviewer',
  name: 'Limited Reviewer',
  inputCost: EXPENSIVE_INPUT,
  outputCost: EXPENSIVE_OUTPUT,
  maxOutputTokens: INSUFFICIENT_MAX_TOKENS,
},);

/** Registry-only model fixture used to distinguish out-of-scope from unknown slugs. */
const thirdModel = modelFixture({
  provider: 'third',
  id: 'reviewer',
  name: 'Reviewer Third',
  inputCost: THIRD_INPUT,
  outputCost: THIRD_OUTPUT,
},);

/** Effective scope fixture with ambiguous bare ids. */
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

/** Model-resolution port fixture exposing scoped plus out-of-scope models. */
const modelRegistry = {
  getAll() {
    return [
      cheapModel,
      expensiveModel,
      thirdModel,
      insufficientOutputModel,
    ];
  },
} as unknown as AdvisorModelResolution;

/**
 Capture a synchronous error from a selection action.
 
 @param action - action expected to throw
 
 @returns caught error value
 
 @example
 ```typescript
 const error = captureError(function fail() { throw new Error('x'); });
 ```
 */
function captureError(
  action: () => unknown,
): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected action to throw',);
}

//endregion Fixtures

await describe({
  name: selectAdvisorModel.name,
  children: [
    it({
      name: 'keeps default selection slug and ranking order for fixed fixtures',
      fn: async function testDefaultSelectionShape() {
        const result = selectAdvisorModel({
          scope,
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);

        expect(result.selected.canonicalSlug,).toBe('expensive/reviewer',);
        expect(
          result.defaultSelection?.ranking.map(function mapScore(score,) {
            return score.slug;
          },),
        )
          .toEqual([
            'expensive/reviewer',
            'cheap/reviewer',
          ],);
      },
    },),
    it({
      name: 'skips current main model for default selection when alternate remains',
      fn: async function testDefaultAvoidsCurrentMainModel() {
        const result = selectAdvisorModel({
          scope,
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
          currentMainModel: expensiveModel,
        },);

        expect(result.selected.canonicalSlug,).toBe('cheap/reviewer',);
        expect(
          result.defaultSelection?.ranking.map(function mapScore(score,) {
            return score.slug;
          },),
        )
          .toEqual(['cheap/reviewer',],);
      },
    },),
    it({
      name: 'falls back to current main model when no alternate remains',
      fn: async function testDefaultFallbackToOnlyCurrentMainModel() {
        const singleModelScope: EffectiveModelScope = {
          source: 'available',
          entries: [{
            model: expensiveModel,
            canonicalSlug: 'expensive/reviewer',
          },],
        };
        const result = selectAdvisorModel({
          scope: singleModelScope,
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
          currentMainModel: expensiveModel,
        },);

        expect(result.selected.canonicalSlug,).toBe('expensive/reviewer',);
      },
    },),
    it({
      name: 'honors explicit current main model request',
      fn: async function testExplicitCurrentMainModelRequest() {
        const result = selectAdvisorModel({
          scope,
          requestedSlug: 'expensive/reviewer',
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
          currentMainModel: expensiveModel,
        },);

        expect(result.selected.canonicalSlug,).toBe('expensive/reviewer',);
      },
    },),
    it({
      name: 'selects the higher-cost model regardless of declared output capacity',
      fn: async function testDefaultKeepsInsufficientOutputModel() {
        const mixedCapacityScope: EffectiveModelScope = {
          source: 'available',
          entries: [
            {
              model: cheapModel,
              canonicalSlug: 'cheap/reviewer',
            },
            {
              model: insufficientOutputModel,
              canonicalSlug: 'limited/limited-reviewer',
            },
          ],
        };
        const result = selectAdvisorModel({
          scope: mixedCapacityScope,
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);

        expect(result.selected.canonicalSlug,).toBe('limited/limited-reviewer',);
        expect(
          result.defaultSelection?.ranking.map(function mapScore(score,) {
            return score.slug;
          },),
        )
          .toEqual(['limited/limited-reviewer', 'cheap/reviewer',],);
      },
    },),
    it({
      name: 'accepts model advertising exactly configured output capacity',
      fn: async function testDefaultAcceptsExactOutputCapacity() {
        const exactCapacityModel = modelFixture({
          provider: 'exact',
          id: 'reviewer',
          name: 'Exact Capacity Reviewer',
          inputCost: CHEAP_INPUT,
          outputCost: CHEAP_OUTPUT,
          maxOutputTokens: ADVISOR_OUTPUT_TOKENS,
        },);
        const exactCapacityScope: EffectiveModelScope = {
          source: 'available',
          entries: [{
            model: exactCapacityModel,
            canonicalSlug: 'exact/reviewer',
          },],
        };
        const result = selectAdvisorModel({
          scope: exactCapacityScope,
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);

        expect(result.selected.canonicalSlug,).toBe('exact/reviewer',);
      },
    },),
    it({
      name: 'resolves explicit model below configured output capacity',
      fn: async function testExplicitAcceptsInsufficientOutputModel() {
        const mixedCapacityScope: EffectiveModelScope = {
          source: 'available',
          entries: [
            {
              model: cheapModel,
              canonicalSlug: 'cheap/reviewer',
            },
            {
              model: insufficientOutputModel,
              canonicalSlug: 'limited/limited-reviewer',
            },
          ],
        };
        const result = selectAdvisorModel({
          scope: mixedCapacityScope,
          requestedSlug: 'limited/limited-reviewer',
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);

        expect(result.selected.canonicalSlug,).toBe('limited/limited-reviewer',);
      },
    },),
    it({
      name: 'selects the only scoped model when it advertises less output than the budget',
      fn: async function testDefaultAcceptsInsufficientOutputScope() {
        const insufficientCapacityScope: EffectiveModelScope = {
          source: 'available',
          entries: [{
            model: insufficientOutputModel,
            canonicalSlug: 'limited/limited-reviewer',
          },],
        };
        const result = selectAdvisorModel({
          scope: insufficientCapacityScope,
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);

        expect(result.selected.canonicalSlug,).toBe('limited/limited-reviewer',);
      },
    },),
    it({
      name: 'keeps ambiguous bare-id error shape',
      fn: async function testAmbiguousBareIdErrorShape() {
        const caught = captureError(function selectAmbiguousBareId() {
          return selectAdvisorModel({
            scope,
            requestedSlug: 'reviewer',
            config: advisorConfig,
            estimatedInputTokens: ADVISOR_INPUT_TOKENS,
            modelRegistry,
          },);
        },);

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('ambiguous in scoped models',);
      },
    },),
    it({
      name: 'keeps out-of-scope slug error shape',
      fn: async function testOutOfScopeSlugErrorShape() {
        const caught = captureError(function selectOutOfScopeSlug() {
          return selectAdvisorModel({
            scope,
            requestedSlug: 'third/reviewer',
            config: advisorConfig,
            estimatedInputTokens: ADVISOR_INPUT_TOKENS,
            modelRegistry,
          },);
        },);

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('is not in scoped models',);
      },
    },),
    it({
      name: 'keeps unknown slug error shape',
      fn: async function testUnknownSlugErrorShape() {
        const caught = captureError(function selectUnknownSlug() {
          return selectAdvisorModel({
            scope,
            requestedSlug: 'missing/reviewer',
            config: advisorConfig,
            estimatedInputTokens: ADVISOR_INPUT_TOKENS,
            modelRegistry,
          },);
        },);

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('was not found in scoped models',);
      },
    },),
  ],
},);

await describe({
  name: selectAdvisorRunContext.name,
  children: [
    it({
      name: 'builds default context for non-current model when alternate remains',
      fn: async function testRunContextAvoidsCurrentMainModel() {
        const result = selectAdvisorRunContext({
          branch: [],
          transcript: testAdvisorTranscriptSerialization,
          config: advisorConfig,
          advisorSystemPrompt: 'review carefully',
          scope,
          modelRegistry,
          currentMainModel: expensiveModel,
        },);

        expect(result.selection.selected.canonicalSlug,).toBe('cheap/reviewer',);
      },
    },),
    it({
      name: 'builds context for every scoped model regardless of declared output capacity',
      fn: async function testRunContextKeepsInsufficientOutputModel() {
        const mixedCapacityScope: EffectiveModelScope = {
          source: 'available',
          entries: [
            {
              model: cheapModel,
              canonicalSlug: 'cheap/reviewer',
            },
            {
              model: insufficientOutputModel,
              canonicalSlug: 'limited/limited-reviewer',
            },
          ],
        };
        const result = selectAdvisorRunContext({
          branch: [],
          transcript: testAdvisorTranscriptSerialization,
          config: advisorConfig,
          advisorSystemPrompt: 'review carefully',
          scope: mixedCapacityScope,
          modelRegistry,
        },);

        expect(result.selection.selected.canonicalSlug,).toBe('limited/limited-reviewer',);
        expect(result.selection.defaultSelection?.ranking,).toHaveLength(2,);
      },
    },),
  ],
},);
