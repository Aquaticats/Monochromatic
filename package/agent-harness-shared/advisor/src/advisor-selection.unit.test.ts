/**
 Unit tests for the configured initial reviewer tier of Advisor model selection.
 
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
} from '@monochromatic-dev/module-test/ts';

import {
  type AdvisorConfig,
  type AdvisorDefaultRanking,
  type AdvisorModelResolution,
  type AdvisorRankingCandidate,
  type DefaultModelSelection,
  type EffectiveModelScope,
  type RankAdvisorDefaultCandidates,
  type ScopedAdvisorModel,
  applyConfiguredInitialReviewer,
  rankDefaultAdvisorCandidates,
  resolveConfiguredDefaultReviewer,
  scopeIncludingConfiguredReviewer,
  scopeAvoidingCurrentMainModel,
  selectAdvisorModel,
  selectAdvisorRunContext,
} from '../dist/final/node/index.mjs';
import { testAdvisorTranscriptSerialization, } from './transcript-test-fixture.ts';

//region Fixtures

/** Advisor output token budget used by selection tests. */
const ADVISOR_OUTPUT_TOKENS = 32;

/** Advisor input token estimate used by selection tests. */
const ADVISOR_INPUT_TOKENS = 16;

/** Fixture context budget. */
const CONTEXT_WINDOW = 4_096;

/** Fixture maximum output tokens. */
const MAX_TOKENS = 512;

/** Runtime Advisor config fixture without a configured reviewer. */
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
 
 @returns pi model fixture
 
 @example
 ```typescript
 modelFixture({ provider: 'cheap', id: 'reviewer', name: 'Cheap', inputCost: 1, outputCost: 2 });
 ```
 */
function modelFixture(
  {
    provider,
    id,
    name,
    inputCost,
    outputCost,
  }: {
    readonly provider: string;
    readonly id: string;
    readonly name: string;
    readonly inputCost: number;
    readonly outputCost: number;
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
    maxTokens: MAX_TOKENS,
  } satisfies Model<Api>;
}

/** Cheap scoped model fixture. */
const cheapModel = modelFixture({
  provider: 'cheap',
  id: 'reviewer',
  name: 'Reviewer Cheap',
  inputCost: 1,
  outputCost: 2,
},);

/** Expensive scoped model fixture. */
const expensiveModel = modelFixture({
  provider: 'expensive',
  id: 'reviewer',
  name: 'Reviewer Expensive',
  inputCost: 4,
  outputCost: 8,
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

/** Model-resolution port fixture. */
const modelRegistry = {
  getAll() {
    return [
      cheapModel,
      expensiveModel,
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

/**
 Build one expected-cost ranking entry for composition tests.
 
 @param slug - canonical scoped model slug
 
 @param expectedCost - expected request cost used for ordering
 
 @returns cost score fixture
 
 @example
 ```typescript
 costScore('cheap/reviewer', 50);
 ```
 */
function costScore(
  slug: string,
  expectedCost: number,
): DefaultModelSelection['ranking'][number] {
  return {
    slug,
    inputTokens: ADVISOR_INPUT_TOKENS,
    maxOutputTokens: ADVISOR_OUTPUT_TOKENS,
    expectedCost,
    inputCost: 1,
    outputCost: 2,
    contextWindow: CONTEXT_WINDOW,
  };
}

/**
 External ranking policy fixture ordering candidates by cheapest input price.
 
 @param options - eligible candidates and requested output budget
 
 @returns reversed-from-cost ranking decision
 
 @example
 ```typescript
 cheapestFirstPolicy({ candidates: [], maxOutputTokens: 32 });
 ```
 */
function cheapestFirstPolicy(
  {
    candidates,
    maxOutputTokens,
  }: {
    readonly candidates: readonly AdvisorRankingCandidate[];
    readonly maxOutputTokens: number;
  },
): AdvisorDefaultRanking {
  return {
    ranked: candidates
      .map(function scopedModel(candidate: AdvisorRankingCandidate,): ScopedAdvisorModel {
        return candidate.scopedModel;
      },)
      .toSorted(function byCheaperInputFirst(
        left: ScopedAdvisorModel,
        right: ScopedAdvisorModel,
      ): number {
        return left.model
          .cost
          .input - right.model
            .cost
            .input;
      },),
    reason: `cheapest input price first for ${String(maxOutputTokens,)} output tokens`,
  };
}

//endregion Fixtures

//region resolveConfiguredDefaultReviewer

await describe({
  name: resolveConfiguredDefaultReviewer.name,
  children: [
    it({
      name: 'resolves canonical slug from the scope',
      fn: async function testResolvesCanonicalSlug() {
        const resolved = resolveConfiguredDefaultReviewer({
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'cheap/reviewer',
          },
          scope,
          modelRegistry,
        },);
        expect(resolved.keyText,).toBe('cheap/reviewer',);
        expect(resolved.entry
          ?.canonicalSlug,).toBe('cheap/reviewer',);
      },
    },),
    it({
      name: 'resolves unique bare id from the scope',
      fn: async function testResolvesUniqueBareId() {
        const singleEntryScope: EffectiveModelScope = {
          source: 'available',
          entries: [{
            model: cheapModel,
            canonicalSlug: 'cheap/reviewer',
          },],
        };
        const resolved = resolveConfiguredDefaultReviewer({
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'reviewer',
          },
          scope: singleEntryScope,
          modelRegistry,
        },);
        expect(resolved.entry
          ?.canonicalSlug,).toBe('cheap/reviewer',);
      },
    },),
    it({
      name: 'resolves unique display name from the scope',
      fn: async function testResolvesUniqueName() {
        const resolved = resolveConfiguredDefaultReviewer({
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'Reviewer Cheap',
          },
          scope,
          modelRegistry,
        },);
        expect(resolved.entry
          ?.canonicalSlug,).toBe('cheap/reviewer',);
      },
    },),
    it({
      name: 'reports nothing for absent or explicitly cleared keys',
      fn: async function testReportsNothingForClearedKeys() {
        const absent = resolveConfiguredDefaultReviewer({
          config: advisorConfig,
          scope,
          modelRegistry,
        },);
        expect(absent,).toEqual({},);
        const cleared = resolveConfiguredDefaultReviewer({
          config: {
            ...advisorConfig,
            defaultReviewerModel: '   ',
          },
          scope,
          modelRegistry,
        },);
        expect(cleared,).toEqual({},);
      },
    },),
    it({
      name: 'reports the key without an entry when it is unresolvable or ambiguous',
      fn: async function testReportsKeyWithoutEntry() {
        const unresolvable = resolveConfiguredDefaultReviewer({
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'missing/reviewer',
          },
          scope,
          modelRegistry,
        },);
        expect(unresolvable,).toEqual({ keyText: 'missing/reviewer', },);
        const ambiguous = resolveConfiguredDefaultReviewer({
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'reviewer',
          },
          scope,
          modelRegistry,
        },);
        expect(ambiguous,).toEqual({ keyText: 'reviewer', },);
      },
    },),
  ],
},);

//endregion resolveConfiguredDefaultReviewer

//region scopeIncludingConfiguredReviewer

await describe({
  name: scopeIncludingConfiguredReviewer.name,
  children: [
    it({
      name: 'keeps the scope untouched when no configured reviewer exists',
      fn: async function testKeepsScopeWithoutReviewer() {
        const result = scopeIncludingConfiguredReviewer({ scope, },);
        expect(result,).toBe(scope,);
      },
    },),
    it({
      name: 'keeps the scope untouched when the configured reviewer is already present',
      fn: async function testKeepsScopeWhenReviewerPresent() {
        const configuredReviewer = scope.entries[0] as ScopedAdvisorModel;
        const result = scopeIncludingConfiguredReviewer({
          scope,
          configuredReviewer,
        },);
        expect(result,).toBe(scope,);
      },
    },),
    it({
      name: 'appends the configured reviewer when current-main avoidance dropped it',
      fn: async function testAppendsMissingReviewer() {
        const configuredReviewer = scope.entries[1] as ScopedAdvisorModel;
        const withoutReviewer: EffectiveModelScope = {
          source: 'available',
          entries: [{
            model: cheapModel,
            canonicalSlug: 'cheap/reviewer',
          },],
        };
        const result = scopeIncludingConfiguredReviewer({
          scope: withoutReviewer,
          configuredReviewer,
        },);
        expect(
          result.entries
            .map(function mapSlug(entry,) {
              return entry.canonicalSlug;
            },),
        )
          .toEqual([
            'cheap/reviewer',
            'expensive/reviewer',
          ],);
      },
    },),
  ],
},);

//endregion scopeIncludingConfiguredReviewer

//region applyConfiguredInitialReviewer

await describe({
  name: applyConfiguredInitialReviewer.name,
  children: [
    it({
      name: 'promotes the configured reviewer and keeps the remaining ranking order',
      fn: async function testPromotesConfiguredReviewer() {
        const configuredReviewer = scope.entries[0] as ScopedAdvisorModel;
        const defaultSelection: DefaultModelSelection = {
          selected: scope.entries[1] as ScopedAdvisorModel,
          ranking: [
            costScore(
              'expensive/reviewer',
              100,
            ),
            costScore(
              'cheap/reviewer',
              50,
            ),
          ],
          reason: 'highest expected cost: expensive/reviewer',
        };
        const composed = applyConfiguredInitialReviewer({
          defaultSelection,
          resolution: {
            keyText: 'cheap/reviewer',
            entry: configuredReviewer,
          },
        },);
        expect(composed.selected
          .canonicalSlug,).toBe('cheap/reviewer',);
        expect(
          composed.ranking
            .map(function mapSlug(score,) {
              return score.slug;
            },),
        )
          .toEqual([
            'cheap/reviewer',
            'expensive/reviewer',
          ],);
        expect(composed.reason,).toContain('defaultReviewerModel "cheap/reviewer"',);
        expect(composed.reason,).toContain('cheap/reviewer as initial reviewer',);
      },
    },),
    it({
      name: 'names the scoped slug when a resolved entry carries no key text',
      fn: async function testNamesSlugWithoutKeyText() {
        const configuredReviewer = scope.entries[0] as ScopedAdvisorModel;
        const defaultSelection: DefaultModelSelection = {
          selected: configuredReviewer,
          ranking: [costScore(
            'cheap/reviewer',
            50,
          ),],
          reason: 'highest expected cost: cheap/reviewer',
        };
        const composed = applyConfiguredInitialReviewer({
          defaultSelection,
          resolution: { entry: configuredReviewer, },
        },);
        expect(composed.reason,).toContain('defaultReviewerModel "cheap/reviewer"',);
      },
    },),
    it({
      name: 'reports an unresolvable configured key and keeps the default ranking',
      fn: async function testReportsUnresolvableKey() {
        const defaultSelection: DefaultModelSelection = {
          selected: scope.entries[1] as ScopedAdvisorModel,
          ranking: [costScore(
            'expensive/reviewer',
            100,
          ),],
          reason: 'highest expected cost: expensive/reviewer',
        };
        const composed = applyConfiguredInitialReviewer({
          defaultSelection,
          resolution: { keyText: 'missing/reviewer', },
        },);
        expect(composed.selected
          .canonicalSlug,).toBe('expensive/reviewer',);
        expect(composed.reason,).toContain(
          'defaultReviewerModel "missing/reviewer" did not resolve among scoped models',
        );
        expect(
          composed.ranking
            .map(function mapSlug(score,) {
              return score.slug;
            },),
        )
          .toEqual(['expensive/reviewer',],);
      },
    },),
    it({
      name: 'keeps the default selection untouched without a configured key',
      fn: async function testKeepsSelectionWithoutKey() {
        const defaultSelection: DefaultModelSelection = {
          selected: scope.entries[1] as ScopedAdvisorModel,
          ranking: [costScore(
            'expensive/reviewer',
            100,
          ),],
          reason: 'highest expected cost: expensive/reviewer',
        };
        const composed = applyConfiguredInitialReviewer({
          defaultSelection,
          resolution: {},
        },);
        expect(composed,).toBe(defaultSelection,);
      },
    },),
    it({
      name: 'throws when the configured reviewer disappeared from the ranking',
      fn: async function testThrowsWhenRankingEntryMissing() {
        const configuredReviewer = scope.entries[0] as ScopedAdvisorModel;
        const defaultSelection: DefaultModelSelection = {
          selected: scope.entries[1] as ScopedAdvisorModel,
          ranking: [costScore(
            'expensive/reviewer',
            100,
          ),],
          reason: 'highest expected cost: expensive/reviewer',
        };
        const caught = captureError(function composeMissingRankingEntry() {
          return applyConfiguredInitialReviewer({
            defaultSelection,
            resolution: {
              keyText: 'cheap/reviewer',
              entry: configuredReviewer,
            },
          },);
        },);
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain(
          'disappeared from the default ranking',
        );
      },
    },),
  ],
},);

//endregion applyConfiguredInitialReviewer

//region Selection integration

await describe({
  name: selectAdvisorModel.name,
  children: [
    it({
      name: 'selects the configured reviewer as initial model instead of the highest-cost one',
      fn: async function testSelectsConfiguredReviewer() {
        const result = selectAdvisorModel({
          scope,
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'cheap/reviewer',
          },
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);
        expect(result.selected
          .canonicalSlug,).toBe('cheap/reviewer',);
        expect(result.requestedSlug,).toBe(undefined,);
        expect(
          result.defaultSelection
            ?.ranking
            .map(function mapSlug(score,) {
              return score.slug;
            },),
        )
          .toEqual([
            'cheap/reviewer',
            'expensive/reviewer',
          ],);
      },
    },),
    it({
      name: 'keeps the configured reviewer as initial model even when it is the current main model',
      fn: async function testConfiguredReviewerWinsOverCurrentMainAvoidance() {
        const result = selectAdvisorModel({
          scope,
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'expensive/reviewer',
          },
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
          currentMainModel: expensiveModel,
        },);
        expect(result.selected
          .canonicalSlug,).toBe('expensive/reviewer',);
        expect(
          result.defaultSelection
            ?.ranking
            .map(function mapSlug(score,) {
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
      name: 'keeps explicit requests exact and above the configured reviewer',
      fn: async function testExplicitRequestWinsOverConfiguredReviewer() {
        const result = selectAdvisorModel({
          scope,
          requestedSlug: 'expensive/reviewer',
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'cheap/reviewer',
          },
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);
        expect(result.selected
          .canonicalSlug,).toBe('expensive/reviewer',);
        expect(result.requestedSlug,).toBe('expensive/reviewer',);
        expect(result.defaultSelection,).toBe(undefined,);
      },
    },),
    it({
      name: 'falls through to the cost ranking when the configured reviewer is unresolvable',
      fn: async function testFallsThroughToCostRanking() {
        const result = selectAdvisorModel({
          scope,
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'missing/reviewer',
          },
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);
        expect(result.selected
          .canonicalSlug,).toBe('expensive/reviewer',);
        expect(result.defaultSelection
          ?.reason,).toContain('did not resolve among scoped models',);
      },
    },),
    it({
      name: 'honors a capacity-limited configured reviewer',
      fn: async function testHonorsCapacityLimitedConfiguredReviewer() {
        /**
         High-cost model whose endpoint advertises less output than the budget.
         */
        const limitedModel = {
          ...modelFixture({
            provider: 'limited',
            id: 'limited-reviewer',
            name: 'Limited Reviewer',
            inputCost: 9,
            outputCost: 9,
          },),
          maxTokens: ADVISOR_OUTPUT_TOKENS - 1,
        } satisfies Model<Api>;
        const mixedCapacityScope: EffectiveModelScope = {
          source: 'available',
          entries: [
            {
              model: cheapModel,
              canonicalSlug: 'cheap/reviewer',
            },
            {
              model: limitedModel,
              canonicalSlug: 'limited/limited-reviewer',
            },
          ],
        };
        const result = selectAdvisorModel({
          scope: mixedCapacityScope,
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'limited/limited-reviewer',
          },
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
        },);
        expect(result.selected
          .canonicalSlug,).toBe('limited/limited-reviewer',);
        expect(result.defaultSelection
          ?.reason,).toContain('configured defaultReviewerModel "limited/limited-reviewer" selects limited/limited-reviewer',);
      },
    },),
    it({
      name: 'ranks default candidates with the supplied policy instead of cost ranking',
      fn: async function testPolicyRanksDefaultCandidates() {
        const result = selectAdvisorModel({
          scope,
          config: advisorConfig,
          estimatedInputTokens: ADVISOR_INPUT_TOKENS,
          modelRegistry,
          rankDefaultCandidates: cheapestFirstPolicy,
        },);
        expect(result.selected
          .canonicalSlug,).toBe('cheap/reviewer',);
        expect(
          result.defaultSelection
            ?.ranking
            .map(function mapSlug(score,) {
              return score.slug;
            },),
        )
          .toEqual([
            'cheap/reviewer',
            'expensive/reviewer',
          ],);
        expect(result.defaultSelection
          ?.reason,).toContain('cheapest input price first',);
      },
    },),
  ],
},);

await describe({
  name: selectAdvisorRunContext.name,
  children: [
    it({
      name: 'builds the initial context for the configured reviewer and leads the fallback order with it',
      fn: async function testRunContextLeadsWithConfiguredReviewer() {
        const result = selectAdvisorRunContext({
          branch: [],
          transcript: testAdvisorTranscriptSerialization,
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'cheap/reviewer',
          },
          advisorSystemPrompt: 'review carefully',
          scope,
          modelRegistry,
          currentMainModel: cheapModel,
        },);
        expect(result.selection.selected
          .canonicalSlug,).toBe('cheap/reviewer',);
        expect(
          result.candidates
            ?.map(function mapCandidate(candidate,) {
              return candidate.scopedModel
                .canonicalSlug;
            },),
        )
          .toEqual([
            'cheap/reviewer',
            'expensive/reviewer',
          ],);
        expect(result.advisorContext
          .maxContextChars,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'keeps the cost-ranked initial context when the configured reviewer is unresolvable',
      fn: async function testRunContextFallsThrough() {
        const result = selectAdvisorRunContext({
          branch: [],
          transcript: testAdvisorTranscriptSerialization,
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'missing/reviewer',
          },
          advisorSystemPrompt: 'review carefully',
          scope,
          modelRegistry,
        },);
        expect(result.selection.selected
          .canonicalSlug,).toBe('expensive/reviewer',);
        expect(
          result.candidates
            ?.map(function mapCandidate(candidate,) {
              return candidate.scopedModel
                .canonicalSlug;
            },),
        )
          .toEqual([
            'expensive/reviewer',
            'cheap/reviewer',
          ],);
      },
    },),
    it({
      name: 'leads the fallback order with the supplied policy ranking',
      fn: async function testRunContextPolicyOrder() {
        const result = selectAdvisorRunContext({
          branch: [],
          transcript: testAdvisorTranscriptSerialization,
          config: advisorConfig,
          advisorSystemPrompt: 'review carefully',
          scope,
          modelRegistry,
          rankDefaultCandidates: cheapestFirstPolicy,
        },);
        expect(result.selection.selected
          .canonicalSlug,).toBe('cheap/reviewer',);
        expect(
          result.candidates
            ?.map(function mapCandidate(candidate,) {
              return candidate.scopedModel
                .canonicalSlug;
            },),
        )
          .toEqual([
            'cheap/reviewer',
            'expensive/reviewer',
          ],);
      },
    },),
    it({
      name: 'promotes the configured reviewer above the policy ranking',
      fn: async function testRunContextConfiguredReviewerOverPolicy() {
        const result = selectAdvisorRunContext({
          branch: [],
          transcript: testAdvisorTranscriptSerialization,
          config: {
            ...advisorConfig,
            defaultReviewerModel: 'expensive/reviewer',
          },
          advisorSystemPrompt: 'review carefully',
          scope,
          modelRegistry,
          rankDefaultCandidates: cheapestFirstPolicy,
        },);
        expect(result.selection.selected
          .canonicalSlug,).toBe('expensive/reviewer',);
        expect(
          result.candidates
            ?.map(function mapCandidate(candidate,) {
              return candidate.scopedModel
                .canonicalSlug;
            },),
        )
          .toEqual([
            'expensive/reviewer',
            'cheap/reviewer',
          ],);
        expect(result.selection.defaultSelection
          ?.reason,).toContain('cheapest input price first',);
        expect(result.selection.defaultSelection
          ?.reason,).toContain('defaultReviewerModel "expensive/reviewer"',);
      },
    },),
  ],
},);

//endregion Selection integration

//region rankDefaultAdvisorCandidates

/** Input token estimates keyed by canonical slug for ranking seam tests. */
const rankingEstimates = new Map<string, number>([
  [
    'cheap/reviewer',
    ADVISOR_INPUT_TOKENS,
  ],
  [
    'expensive/reviewer',
    ADVISOR_INPUT_TOKENS,
  ],
],);

await describe({
  name: rankDefaultAdvisorCandidates.name,
  children: [
    it({
      name: 'keeps the expected-cost ranking when no policy is supplied',
      fn: async function testCostRankingWithoutPolicy() {
        const selection = rankDefaultAdvisorCandidates({
          scope,
          estimatedInputTokensBySlug: rankingEstimates,
          maxOutputTokens: ADVISOR_OUTPUT_TOKENS,
        },);
        expect(selection.selected
          .canonicalSlug,).toBe('expensive/reviewer',);
        expect(
          selection.ranking
            .map(function mapSlug(score,) {
              return score.slug;
            },),
        )
          .toEqual([
            'expensive/reviewer',
            'cheap/reviewer',
          ],);
        expect(selection.reason,).toContain('highest expected cost',);
      },
    },),
    it({
      name: 'uses the policy order and forwards candidates plus output budget',
      fn: async function testPolicyOrderAndInputs() {
        const selection = rankDefaultAdvisorCandidates({
          scope,
          estimatedInputTokensBySlug: rankingEstimates,
          maxOutputTokens: ADVISOR_OUTPUT_TOKENS,
          rankDefaultCandidates: function observingPolicy({
            candidates,
            maxOutputTokens,
          }: Parameters<RankAdvisorDefaultCandidates>[0],): AdvisorDefaultRanking {
            /**
             Policy reason recording every forwarded candidate, estimate, and budget.
             */
            const observed = candidates
              .map(function mapObservedCandidate(candidate: AdvisorRankingCandidate,): string {
                return `${candidate.scopedModel
                  .canonicalSlug}=${String(candidate.estimatedInputTokens,)}`;
              },)
              .join(' ',);
            return {
              ranked: cheapestFirstPolicy({
                candidates,
                maxOutputTokens,
              },)
                .ranked,
              reason: `observed ${observed} budget ${String(maxOutputTokens,)}`,
            };
          },
        },);
        expect(selection.reason,).toContain(
          'observed cheap/reviewer=16 expensive/reviewer=16',
        );
        expect(selection.reason,).toContain('budget 32',);
        expect(selection.selected
          .canonicalSlug,).toBe('cheap/reviewer',);
      },
    },),
    it({
      name: 'throws when the policy drops a candidate',
      fn: async function testThrowsWhenPolicyDropsCandidate() {
        const caught = captureError(function dropCandidate() {
          return rankDefaultAdvisorCandidates({
            scope,
            estimatedInputTokensBySlug: rankingEstimates,
            maxOutputTokens: ADVISOR_OUTPUT_TOKENS,
            rankDefaultCandidates: function droppingPolicy(): AdvisorDefaultRanking {
              return {
                ranked: [scope.entries[0] as ScopedAdvisorModel,],
                reason: 'drops the other candidate',
              };
            },
          },);
        },);
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain(
          'returned 1 ranked candidates for 2 eligible candidates',
        );
      },
    },),
    it({
      name: 'throws when the policy duplicates a candidate',
      fn: async function testThrowsWhenPolicyDuplicatesCandidate() {
        const firstEntry = scope.entries[0] as ScopedAdvisorModel;
        const caught = captureError(function duplicateCandidate() {
          return rankDefaultAdvisorCandidates({
            scope,
            estimatedInputTokensBySlug: rankingEstimates,
            maxOutputTokens: ADVISOR_OUTPUT_TOKENS,
            rankDefaultCandidates: function duplicatingPolicy(): AdvisorDefaultRanking {
              return {
                ranked: [
                  firstEntry,
                  firstEntry,
                ],
                reason: 'duplicates the first candidate',
              };
            },
          },);
        },);
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain(
          'must return each eligible candidate exactly once; missing or duplicated: cheap/reviewer',
        );
      },
    },),
    it({
      name: 'throws when the policy returns an unknown candidate',
      fn: async function testThrowsWhenPolicyReturnsUnknownCandidate() {
        /**
         Scoped entry the policy invented, absent from the eligible scope.
         */
        const inventedEntry: ScopedAdvisorModel = {
          model: cheapModel,
          canonicalSlug: 'invented/reviewer',
        };
        const caught = captureError(function inventCandidate() {
          return rankDefaultAdvisorCandidates({
            scope,
            estimatedInputTokensBySlug: rankingEstimates,
            maxOutputTokens: ADVISOR_OUTPUT_TOKENS,
            rankDefaultCandidates: function inventingPolicy(): AdvisorDefaultRanking {
              return {
                ranked: [
                  scope.entries[0] as ScopedAdvisorModel,
                  inventedEntry,
                ],
                reason: 'invents a candidate',
              };
            },
          },);
        },);
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain(
          'must return each eligible candidate exactly once; missing or duplicated: expensive/reviewer',
        );
      },
    },),
    it({
      name: 'throws when the policy returns nothing for an empty scope',
      fn: async function testThrowsWhenPolicyReturnsNothing() {
        const emptyScope: EffectiveModelScope = {
          source: 'available',
          entries: [],
        };
        const caught = captureError(function emptyRanking() {
          return rankDefaultAdvisorCandidates({
            scope: emptyScope,
            estimatedInputTokensBySlug: new Map<string, number>(),
            maxOutputTokens: ADVISOR_OUTPUT_TOKENS,
            rankDefaultCandidates: function emptyPolicy(): AdvisorDefaultRanking {
              return {
                ranked: [],
                reason: 'no candidates',
              };
            },
          },);
        },);
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain(
          'default ranking policy returned no candidates',
        );
      },
    },),
  ],
},);

//endregion rankDefaultAdvisorCandidates

//region scopeAvoidingCurrentMainModel

await describe({
  name: scopeAvoidingCurrentMainModel.name,
  children: [
    it({
      name: 'drops the same model on another route when the display name matches',
      fn: async function testDropsSameModelDifferentRoute() {
        /** Same model id and display name under two provider routes. */
        const routeAModel = modelFixture({
          provider: 'route-a',
          id: 'mimo-v2.6-pro',
          name: 'MiMo-V2.6-Pro',
          inputCost: 1,
          outputCost: 2,
        },);
        const routeBModel = modelFixture({
          provider: 'route-b',
          id: 'mimo-v2.6-pro',
          name: 'MiMo-V2.6-Pro',
          inputCost: 1,
          outputCost: 2,
        },);
        /** Independent model kept in scope. */
        const otherModel = modelFixture({
          provider: 'route-b',
          id: 'deepseek-v4-flash',
          name: 'DeepSeek V4 Flash',
          inputCost: 1,
          outputCost: 2,
        },);
        const result = scopeAvoidingCurrentMainModel({
          scope: {
            source: 'available',
            entries: [
              {
                model: routeAModel,
                canonicalSlug: 'route-a/mimo-v2.6-pro',
              },
              {
                model: routeBModel,
                canonicalSlug: 'route-b/mimo-v2.6-pro',
              },
              {
                model: otherModel,
                canonicalSlug: 'route-b/deepseek-v4-flash',
              },
            ],
          },
          currentMainModel: {
            provider: 'route-b',
            id: 'mimo-v2.6-pro',
            name: 'MiMo-V2.6-Pro',
          },
        },);

        expect(
          result.entries
            .map(function mapSlug(entry,) {
              return entry.canonicalSlug;
            },),
        )
          .toEqual(['route-b/deepseek-v4-flash',],);
      },
    },),
    it({
      name: 'keeps same-id entries whose display names differ',
      fn: async function testKeepsSameIdDifferentName() {
        const result = scopeAvoidingCurrentMainModel({
          scope,
          currentMainModel: {
            provider: 'expensive',
            id: 'reviewer',
            name: 'Reviewer Expensive',
          },
        },);

        expect(
          result.entries
            .map(function mapSlug(entry,) {
              return entry.canonicalSlug;
            },),
        )
          .toEqual(['cheap/reviewer',],);
      },
    },),
    it({
      name: 'keeps the scope when every entry is the current model',
      fn: async function testKeepsScopeOfOnlyCurrentModel() {
        /** Same model id and display name under two provider routes. */
        const routeAModel = modelFixture({
          provider: 'route-a',
          id: 'mimo-v2.6-pro',
          name: 'MiMo-V2.6-Pro',
          inputCost: 1,
          outputCost: 2,
        },);
        const routeBModel = modelFixture({
          provider: 'route-b',
          id: 'mimo-v2.6-pro',
          name: 'MiMo-V2.6-Pro',
          inputCost: 1,
          outputCost: 2,
        },);
        const result = scopeAvoidingCurrentMainModel({
          scope: {
            source: 'available',
            entries: [
              {
                model: routeAModel,
                canonicalSlug: 'route-a/mimo-v2.6-pro',
              },
              {
                model: routeBModel,
                canonicalSlug: 'route-b/mimo-v2.6-pro',
              },
            ],
          },
          currentMainModel: {
            provider: 'route-b',
            id: 'mimo-v2.6-pro',
            name: 'MiMo-V2.6-Pro',
          },
        },);

        expect(result.entries
          .length,).toBe(2,);
      },
    },),
    it({
      name: 'keeps the scope untouched when the current model is absent',
      fn: async function testKeepsScopeWhenCurrentAbsent() {
        const result = scopeAvoidingCurrentMainModel({
          scope: {
            source: 'available',
            entries: [{
              model: cheapModel,
              canonicalSlug: 'cheap/reviewer',
            },],
          },
          currentMainModel: {
            provider: 'route-b',
            id: 'mimo-v2.6-pro',
            name: 'MiMo-V2.6-Pro',
          },
        },);

        expect(result.entries
          .length,).toBe(1,);
      },
    },),
  ],
},);

//endregion scopeAvoidingCurrentMainModel
