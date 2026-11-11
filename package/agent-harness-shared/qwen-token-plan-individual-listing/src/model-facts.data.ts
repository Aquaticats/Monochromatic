/**
 Curated facts table for the Qwen Token Plan individual route: one row per id
 the table knows, grouped by provenance. Membership comes from the live
 listing (`doc/planning/qwen-token-plan-individual-provider.md` section "Model
 facts table"); every chat row reproduces the source it cites byte for byte,
 so the override changes membership, not facts. @module
 */

import {
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  type QwenTokenPlanModelFacts,
} from './model-facts.ts';

//region Public API

/**
 Facts rows carried at launch, in the order
 `doc/planning/qwen-token-plan-individual-provider.md` section "Model facts
 table" lists them, followed by the seven media kind rows that make the skip
 rule data driven. Provenance per group:

 - builtin rows: pi-ai 1.5.2
   `dist/providers/data/qwen-token-plan-individual.json`
   (`package/pi-plugin/neuralwatt/node_modules/@earendil-works/pi-ai`),
   extracted 2026-11-11 and pinned in
   `src/test-fixtures/fixture.pi-ai-qwen-token-plan-individual-catalog.ts`;
 - sibling catalog rows: pi-ai 1.5.2 `dist/providers/data/qwen-token-plan.json`,
   which also supplies their display names; its 384,000 max output wins over
   the research report's 393,216 (the recorded catalog-row-wins decision);
 - documented row: `qwen4-max-preview`, from
   `doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md` question 2
   (1M context, 393,216 max output shared with thinking, efforts
   `low`/`medium`/`xhigh`), with no catalog row anywhere;
 - media kind rows: kind labels from
   `doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md` question 3
   and display names from the models.dev `alibaba-token-plan` slice where it
   carries the id (suggested names marked below otherwise). They never serve,
   carry no chat input, and hold no token numbers, so nothing guesses from
   them.
 */
export const qwenTokenPlanIndividualFacts: readonly QwenTokenPlanModelFacts[] = [
  //region pi-ai builtin catalog rows

  {
    id: 'qwen3.8-max',
    name: 'Qwen3.8 Max',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    input: [
      'text',
      'image',
    ],
    reasoning: true,
    efforts: {
      low: 'low',
      medium: 'medium',
      xhigh: 'xhigh',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },
  {
    id: 'qwen3.8-flash',
    name: 'Qwen3.8 Flash',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    input: [
      'text',
      'image',
    ],
    reasoning: true,
    efforts: {
      low: 'low',
      medium: 'medium',
      xhigh: 'xhigh',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },
  {
    id: 'qwen3.7-max',
    name: 'Qwen3.7 Max',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    input: [
      'text',
    ],
    reasoning: true,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    id: 'qwen3.7-plus',
    name: 'Qwen3.7 Plus',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 65_536,
    input: [
      'text',
      'image',
    ],
    reasoning: true,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    id: 'qwen3.6-flash',
    name: 'Qwen3.6 Flash',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 65_536,
    input: [
      'text',
      'image',
    ],
    reasoning: true,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    id: 'glm-5.2',
    name: 'GLM-5.2',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    input: [
      'text',
    ],
    reasoning: true,
    efforts: {
      high: 'high',
      max: 'max',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },
  {
    id: 'glm-5.3',
    name: 'GLM-5.3',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    input: [
      'text',
    ],
    reasoning: true,
    efforts: {
      low: 'low',
      high: 'high',
      max: 'max',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },
  {
    id: 'deepseek-v4-flash-0731',
    name: 'DeepSeek V4 Flash 0731',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 384_000,
    input: [
      'text',
    ],
    reasoning: true,
    efforts: {
      high: 'high',
      max: 'max',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },
  {
    id: 'deepseek-v4-pro',
    name: 'DeepSeek V4 Pro',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 384_000,
    input: [
      'text',
    ],
    reasoning: true,
    efforts: {
      high: 'high',
      max: 'max',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },
  {
    id: 'deepseek-v4-pro-0813',
    name: 'DeepSeek V4 Pro 0813',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 384_000,
    input: [
      'text',
    ],
    reasoning: true,
    efforts: {
      high: 'high',
      max: 'max',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },

  //endregion pi-ai builtin catalog rows

  //region pi-ai sibling catalog rows

  {
    id: 'deepseek-v4.1-flash',
    name: 'DeepSeek V4.1 Flash',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 384_000,
    input: [
      'text',
      'image',
    ],
    reasoning: true,
    efforts: {
      low: 'low',
      high: 'high',
      max: 'max',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },
  {
    id: 'deepseek-v4.1-flash-1017',
    name: 'DeepSeek V4.1 Flash 1017',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 384_000,
    input: [
      'text',
      'image',
    ],
    reasoning: true,
    efforts: {
      low: 'low',
      high: 'high',
      max: 'max',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },

  //endregion pi-ai sibling catalog rows

  //region Documented row with no catalog entry

  {
    // Display name chosen to match the builtin naming style ("Qwen3.8 Max");
    // no catalog row names this id anywhere.
    id: 'qwen4-max-preview',
    name: 'Qwen4 Max Preview',
    kind: 'chat',
    contextWindow: 1_000_000,
    maxTokens: 393_216,
    input: [
      'text',
      'image',
    ],
    reasoning: true,
    efforts: {
      low: 'low',
      medium: 'medium',
      xhigh: 'xhigh',
    },
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  },

  //endregion Documented row with no catalog entry

  //region Media kind rows (never served; drive the skip rule)

  {
    id: 'wan2.7-image',
    name: 'Wan2.7 Image',
    kind: 'image',
    input: [],
    reasoning: false,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    id: 'wan2.7-image-pro',
    name: 'Wan2.7 Image Pro',
    kind: 'image',
    input: [],
    reasoning: false,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    // Suggested name: the models.dev slice carries no row for this id.
    id: 'wan3.0-video',
    name: 'Wan3.0 Video',
    kind: 'video',
    input: [],
    reasoning: false,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    id: 'qwen-image-3.0-pro',
    name: 'Qwen-Image-3.0-Pro',
    kind: 'image',
    input: [],
    reasoning: false,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    // Suggested name from the models.dev row for `qwen-audio-3.0-asr-flash`.
    id: 'qwen-audio-3.0-tts-plus',
    name: 'Qwen-Audio-3.0-TTS Plus',
    kind: 'tts',
    input: [],
    reasoning: false,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    id: 'qwen-audio-3.0-asr-flash',
    name: 'Qwen-Audio-3.0-ASR Flash',
    kind: 'asr',
    input: [],
    reasoning: false,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },
  {
    // Suggested name from the models.dev row for `qwen-audio-3.0-asr-flash`.
    id: 'qwen-audio-3.0-realtime-plus',
    name: 'Qwen-Audio-3.0-Realtime Plus',
    kind: 'realtime',
    input: [],
    reasoning: false,
    compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  },

  //endregion Media kind rows (never served; drive the skip rule)
];

//endregion Public API
