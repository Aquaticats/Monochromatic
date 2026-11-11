import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  factsToPiAiModelProfile,
  type QwenTokenPlanResolvedModelFacts,
} from '../dist/final/node/index.mjs';

//region Test fixtures

/**
 Wire-compat pins one resolved facts row carries, spelled exactly as the
 shared facts table states them.
 */
const COMPAT_PINS = {
  thinkingFormat: 'qwen',
  supportsDeveloperRole: false,
  supportsStore: false,
  supportsStrictMode: true,
  supportsReasoningEffort: true,
} satisfies QwenTokenPlanResolvedModelFacts['compat'];

/**
 Offered efforts with wire spellings that differ from their level names, so a
 projection cannot pass on level-name fallback.
 */
const OFFERED_EFFORTS = {
  high: 'turbo',
  max: 'ultra',
} satisfies NonNullable<QwenTokenPlanResolvedModelFacts['efforts']>;

/**
 Resolved facts for the reasoning state with selectable efforts: every field
 the projection copies is decided.
 */
const EFFORTS_MODEL_FACTS: QwenTokenPlanResolvedModelFacts = {
  id: 'probe-3-8-27b',
  name: 'Probe 3.8 27B',
  kind: 'chat',
  contextWindow: 262_128,
  maxTokens: 131_072,
  input: ['text', 'image'],
  reasoning: true,
  efforts: OFFERED_EFFORTS,
  compat: COMPAT_PINS,
};

/**
 Resolved facts for the non-reasoning state: the projection must declare
 `reasoningEfforts: false` rather than inherit anything.
 */
const NON_REASONING_MODEL_FACTS: QwenTokenPlanResolvedModelFacts = {
  id: 'probe-non-reasoning',
  name: 'Probe Non Reasoning',
  kind: 'chat',
  contextWindow: 128_000,
  maxTokens: 16_384,
  input: ['text'],
  reasoning: false,
  compat: COMPAT_PINS,
};

/**
 Resolved facts for the reasoning-without-efforts state: no dict means the
 field stays out so the installed catalog decides the capability.
 */
const NO_EFFORTS_MODEL_FACTS: QwenTokenPlanResolvedModelFacts = {
  id: 'probe-reasoning-without-efforts',
  name: 'Probe Reasoning Without Efforts',
  kind: 'chat',
  contextWindow: 1_000_000,
  maxTokens: 65_536,
  input: ['text'],
  reasoning: true,
  compat: COMPAT_PINS,
};

//endregion Test fixtures

await describe({
  name: 'facts to model profile projection',
  children: [
    //region Reasoning states

    it({
      name: 'projects a model with selectable efforts as its declared efforts dict',
      fn: async () => {
        const model = factsToPiAiModelProfile({ facts: EFFORTS_MODEL_FACTS, });
        expect(model.reasoningEfforts).toEqual(OFFERED_EFFORTS);
      },
    },),
    it({
      name: 'projects a non-reasoning model as reasoningEfforts false',
      fn: async () => {
        const model = factsToPiAiModelProfile({ facts: NON_REASONING_MODEL_FACTS, });
        expect(model.reasoningEfforts).toBe(false);
      },
    },),
    it({
      name: 'omits reasoningEfforts for a reasoning model that offers no selectable efforts',
      fn: async () => {
        const model = factsToPiAiModelProfile({ facts: NO_EFFORTS_MODEL_FACTS, });
        expect(Object.hasOwn(
          model,
          'reasoningEfforts'
        ),).toBe(false);
      },
    },),

    //endregion Reasoning states

    //region Field copy

    it({
      name: 'copies identity, capacities, modalities, and the wire-compat pins the facts decided',
      fn: async () => {
        expect(factsToPiAiModelProfile({ facts: EFFORTS_MODEL_FACTS, })).toEqual({
          id: 'probe-3-8-27b',
          name: 'Probe 3.8 27B',
          contextWindow: 262_128,
          maxTokens: 131_072,
          input: ['text', 'image'],
          reasoningEfforts: OFFERED_EFFORTS,
          compat: COMPAT_PINS,
        });
      },
    },),

    //endregion Field copy
  ],
},);
