/**
 Tests for the retirement rule.

 Every case comes from pi's bundled catalog as measured on 2026-10-02, so the suite
 pins real ids rather than invented ones. See `doc/planning/pi-model-retirement.md`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  compareRecency,
  decideRetirements,
  parseModelId,
  UNORDERED,
  type CatalogEntry,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** API type used by most fixtures. */
const COMPLETIONS_API = 'openai-completions';

/** Second API type, used to prove routes never retire each other. */
const ANTHROPIC_API = 'anthropic-messages';

/** Retirement expectation for one catalog built from a fixture set. */
type RetirementCase = {
  /** Case name shown in test output. */
  name: string;
  /** Catalog entries the rule sees. */
  entries: readonly CatalogEntry[];
  /** Expected retirements, order-insensitive. */
  expected: readonly { readonly retiredId: string; readonly keeperId: string; }[];
};

/**
 Build one catalog entry with the common API type.
 
 @param provider - provider id owning the entry
 
 @param modelId - model id exactly as a catalog carries it
 
 @param api - API type the entry is served under
 
 @returns catalog entry for a fixture set
 */
function entry(
  {
    provider,
    modelId,
    api = COMPLETIONS_API,
  }: {
    readonly provider: string;
    readonly modelId: string;
    readonly api?: string;
  },
): CatalogEntry {
  return { provider, api, modelId, };
}

/**
 Retirement controls taken from the measured catalog, including the three `radius`
 inversions that the alias-wins tiebreak in
 `@monochromatic-dev/agent-harness-shared-model-selection` produced and this rule must not.
 */
const RETIREMENT_CASES: readonly RetirementCase[] = [
  {
    name: 'retires a lower minor version in one family',
    entries: [
      entry({ provider: 'opencode-go', modelId: 'glm-5.2', },),
      entry({ provider: 'opencode-go', modelId: 'glm-5.3', },),
    ],
    expected: [{ retiredId: 'glm-5.2', keeperId: 'glm-5.3', }],
  },
  {
    name: 'retires a base version in favor of its own point release',
    entries: [
      entry({ provider: 'radius', modelId: 'claude-opus-5', },),
      entry({ provider: 'radius', modelId: 'claude-opus-5-5', },),
    ],
    expected: [{ retiredId: 'claude-opus-5', keeperId: 'claude-opus-5-5', }],
  },
  {
    name: 'retires every older point release of one family',
    entries: [
      entry({ provider: 'radius', modelId: 'claude-opus-4-5', },),
      entry({ provider: 'radius', modelId: 'claude-opus-4-8', },),
      entry({ provider: 'radius', modelId: 'claude-opus-5', },),
      entry({ provider: 'radius', modelId: 'claude-opus-5-5', },),
    ],
    expected: [
      { retiredId: 'claude-opus-4-5', keeperId: 'claude-opus-5-5', },
      { retiredId: 'claude-opus-4-8', keeperId: 'claude-opus-5-5', },
      { retiredId: 'claude-opus-5', keeperId: 'claude-opus-5-5', },
    ],
  },
  {
    name: 'retires a glued older version against its newer sibling',
    entries: [
      entry({ provider: 'openrouter', modelId: 'qwen/qwen3.7-flash', },),
      entry({ provider: 'openrouter', modelId: 'qwen/qwen3.8-flash', },),
    ],
    expected: [{ retiredId: 'qwen/qwen3.7-flash', keeperId: 'qwen/qwen3.8-flash', }],
  },
  {
    name: 'retires a pinned snapshot in favor of the rolling alias',
    entries: [
      entry({ provider: 'openrouter', modelId: 'deepseek/deepseek-v4-pro', },),
      entry({ provider: 'openrouter', modelId: 'deepseek/deepseek-v4-pro-0813', },),
    ],
    expected: [
      { retiredId: 'deepseek/deepseek-v4-pro-0813', keeperId: 'deepseek/deepseek-v4-pro', },
    ],
  },
  {
    name: 'retires the earlier of two dated snapshots',
    entries: [
      entry({ provider: 'openrouter', modelId: 'openai/gpt-4o-2024-05-13', },),
      entry({ provider: 'openrouter', modelId: 'openai/gpt-4o-2024-11-20', },),
    ],
    expected: [
      { retiredId: 'openai/gpt-4o-2024-05-13', keeperId: 'openai/gpt-4o-2024-11-20', },
    ],
  },
  {
    name: 'retires a year-month snapshot against its successor',
    entries: [
      entry({ provider: 'openrouter', modelId: 'mistralai/mistral-large-2407', },),
      entry({ provider: 'openrouter', modelId: 'mistralai/mistral-large-2512', },),
    ],
    expected: [
      { retiredId: 'mistralai/mistral-large-2407', keeperId: 'mistralai/mistral-large-2512', },
    ],
  },
  {
    name: 'retires a minor version against a higher minor in the same tier',
    entries: [
      entry({ provider: 'openrouter', modelId: 'deepseek/deepseek-v4-flash', },),
      entry({ provider: 'openrouter', modelId: 'deepseek/deepseek-v4.1-flash', },),
      entry({ provider: 'openrouter', modelId: 'deepseek/deepseek-v4-flash-0731', },),
    ],
    expected: [
      { retiredId: 'deepseek/deepseek-v4-flash', keeperId: 'deepseek/deepseek-v4.1-flash', },
    ],
  },
];

/**
 Keep controls: pairs the rule must abstain on, each one a measured false positive
 of a more aggressive ordering.
 */
const KEEP_CASES: readonly RetirementCase[] = [
  {
    name: 'keeps tier variants of the same version',
    entries: [
      entry({ provider: 'opencode-go', modelId: 'glm-5.3', },),
      entry({ provider: 'opencode-go', modelId: 'glm-5.3-flash', },),
    ],
    expected: [],
  },
  {
    name: 'keeps parameter sizes of the same version',
    entries: [
      entry({ provider: 'openrouter', modelId: 'meta-llama/llama-3.1-8b-instruct', },),
      entry({ provider: 'openrouter', modelId: 'meta-llama/llama-3.1-70b-instruct', },),
    ],
    expected: [],
  },
  {
    name: 'keeps a versionless rolling alias beside a versioned sibling',
    entries: [
      entry({ provider: 'openrouter', modelId: 'openai/gpt-4o', },),
      entry({ provider: 'openrouter', modelId: 'openai/gpt-4o-2024-11-20', },),
    ],
    expected: [],
  },
  {
    name: 'keeps a snapshot whose date meets a minor version component',
    entries: [
      entry({ provider: 'openrouter', modelId: 'deepseek/deepseek-v4-flash-0731', },),
      entry({ provider: 'openrouter', modelId: 'deepseek/deepseek-v4.1-flash', },),
    ],
    expected: [],
  },
  {
    name: 'keeps both sides of a month-day pair against an eight digit date',
    entries: [
      entry({ provider: 'openrouter', modelId: 'qwen/qwen3.5-plus-02-15', },),
      entry({ provider: 'openrouter', modelId: 'qwen/qwen3.5-plus-20260420', },),
    ],
    expected: [],
  },
  {
    name: 'keeps the same id served under two APIs',
    entries: [
      entry({
        provider: 'openrouter',
        modelId: 'google/gemini-3-pro-image',
        api: COMPLETIONS_API,
      },),
      entry({
        provider: 'openrouter',
        modelId: 'google/gemini-3-pro-image',
        api: ANTHROPIC_API,
      },),
    ],
    expected: [],
  },
  {
    name: 'keeps the same family across two providers',
    entries: [
      entry({ provider: 'hyper', modelId: 'glm-5.2', },),
      entry({ provider: 'opencode-go', modelId: 'glm-5.3', },),
    ],
    expected: [],
  },
  {
    name: 'keeps routing targets with no version evidence',
    entries: [
      entry({ provider: 'openrouter', modelId: 'openrouter/auto', },),
      entry({ provider: 'synthetic', modelId: 'syn:large:text', },),
    ],
    expected: [],
  },
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: decideRetirements.name,
      children: RETIREMENT_CASES.map(function createRetirementCase(testCase,) {
        return it({
          name: testCase.name,
          fn: async function runRetirementCase() {
            /**
             Rule outcome for the fixture catalog.
             */
            const decision = decideRetirements({ entries: testCase.entries, },);
            expect(decision.retirements.map(function toPair(retirement,) {
              return { retiredId: retirement.retiredId, keeperId: retirement.keeperId, };
            },),).toEqual([...testCase.expected],);
          },
        },);
      },),
    },),
    describe({
      name: `${decideRetirements.name} abstentions`,
      children: KEEP_CASES.map(function createKeepCase(testCase,) {
        return it({
          name: testCase.name,
          fn: async function runKeepCase() {
            /**
             Rule outcome for the fixture catalog.
             */
            const decision = decideRetirements({ entries: testCase.entries, },);
            expect(decision.retirements,).toEqual([],);
          },
        },);
      },),
    },),
    describe({
      name: 'decision invariants',
      children: [
        it({
          name: 'counts a repeated catalog row as a duplicate identity',
          fn: async function runDuplicateIdentity() {
            /**
             Rule outcome over a catalog repeating one row.
             */
            const decision = decideRetirements({
              entries: [
                entry({ provider: 'radius', modelId: 'claude-opus-5', },),
                entry({ provider: 'radius', modelId: 'claude-opus-5', },),
              ],
            },);
            expect(decision.retirements,).toEqual([],);
            expect(decision.abstentions.duplicateIdentity,).toBe(1,);
          },
        },),
        it({
          name: 'counts versionless entries it protects',
          fn: async function runVersionlessTally() {
            /**
             Rule outcome over a family mixing one versionless alias with versions.
             */
            const decision = decideRetirements({
              entries: [
                entry({ provider: 'openrouter', modelId: 'openai/gpt-4o', },),
                entry({ provider: 'openrouter', modelId: 'openai/gpt-4o-2024-05-13', },),
                entry({ provider: 'openrouter', modelId: 'openai/gpt-4o-2024-11-20', },),
              ],
            },);
            expect(decision.abstentions.versionlessProtected,).toBe(1,);
            expect(decision.retirements.map(function toId(retirement,) {
              return retirement.retiredId;
            },),).toEqual(['openai/gpt-4o-2024-05-13'],);
          },
        },),
        it({
          name: 'counts an unordered pair it declines',
          fn: async function runUnorderedTally() {
            /**
             Rule outcome over the measured month-day against eight-digit pair.
             */
            const decision = decideRetirements({
              entries: [
                entry({ provider: 'openrouter', modelId: 'qwen/qwen3.5-plus-02-15', },),
                entry({ provider: 'openrouter', modelId: 'qwen/qwen3.5-plus-20260420', },),
              ],
            },);
            expect(decision.retirements,).toEqual([],);
            expect(
              (decision.abstentions.unorderedPair + decision.abstentions.keeperAmbiguity) > 0,
            ).toBe(true,);
          },
        },),
        it({
          name: 'never reports a keeper older than the entry it retires',
          fn: async function runKeeperRecencyInvariant() {
            /**
             Every fixture catalog concatenated, so the invariant is checked across all cases.
             */
            const allEntries = [...RETIREMENT_CASES, ...KEEP_CASES].flatMap(
              function collectEntries(testCase,) {
                return [...testCase.entries];
              },
            );
            /**
             Rule outcome over the concatenated catalog.
             */
            const decision = decideRetirements({ entries: allEntries, },);
            for (const retirement of decision.retirements) {
              /**
               Recency of the keeper against the retired entry, which must favor the keeper.
               */
              const outcome = compareRecency({
                left: parseModelId(retirement.keeperId,),
                right: parseModelId(retirement.retiredId,),
              },);
              expect(outcome,).toBe('left',);
            }
          },
        },),
      ],
    },),
    describe({
      name: compareRecency.name,
      children: [
        it({
          name: 'orders a higher component as newer',
          fn: async function runComponentOrder() {
            expect(compareRecency({
              left: parseModelId('glm-5.2',),
              right: parseModelId('glm-5.3',),
            },),).toBe('right',);
          },
        },),
        it({
          name: 'leaves different name shapes unordered',
          fn: async function runNameShapeMismatch() {
            expect(compareRecency({
              left: parseModelId('glm-5.2',),
              right: parseModelId('gpt-6-luna',),
            },),).toBe(UNORDERED,);
          },
        },),
        it({
          name: 'leaves a versionless parse unordered',
          fn: async function runVersionlessOrder() {
            expect(compareRecency({
              left: parseModelId('openai/gpt-4o',),
              right: parseModelId('openai/gpt-4o-2024-11-20',),
            },),).toBe(UNORDERED,);
          },
        },),
        it({
          name: 'prefers the rolling alias over a dated snapshot of equal version',
          fn: async function runAliasPreference() {
            expect(compareRecency({
              left: parseModelId('deepseek-v4-pro',),
              right: parseModelId('deepseek-v4-pro-0813',),
            },),).toBe('left',);
          },
        },),
        it({
          name: 'prefers a point release over its base version',
          fn: async function runPointReleasePreference() {
            expect(compareRecency({
              left: parseModelId('claude-opus-5',),
              right: parseModelId('claude-opus-5-5',),
            },),).toBe('right',);
          },
        },),
        it({
          name: 'leaves a mixed date and version continuation unordered',
          fn: async function runMixedContinuation() {
            expect(compareRecency({
              left: parseModelId('mistral-medium-3',),
              right: parseModelId('mistral-medium-3-5-2512',),
            },),).toBe(UNORDERED,);
          },
        },),
      ],
    },),
  ],
},);
