/**
 Model facts vocabulary for the Qwen Token Plan individual route: the kind and
 wire-compat pins one model earns, the facts row shape the curated table
 carries, and the resolved shape a harness projection copies straight. Pi's
 schema defaults live here too, as the last fallback of the mapping.
 @module
 */

import type { QwenTokenPlanEffortsDict, } from './thinking-level-map.ts';

//region Public API

/**
 Model kind one listing id earns: chat and router ids answer chat
 completions, the remaining kinds are the dedicated media endpoints that never
 serve as chat (`doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md`
 question 3).
 */
export type QwenTokenPlanModelKind =
  'chat' | 'router' | 'image' | 'video' | 'tts' | 'asr' | 'realtime';

/**
 Id name markers that classify an id this facts table does not know as media:
 the marker words the Token Plan capability labels use, so a media id arriving
 between refreshes is skipped rather than served as chat. Matched against the
 lowercased id as whole hyphen-delimited tokens.
 */
export const QWEN_TOKEN_PLAN_MEDIA_NAME_MARKERS = [
  'image',
  'video',
  'audio',
  'tts',
  'asr',
  'realtime',
] as const;

/**
 Input modality one model earns: text always, image when the facts row marks
 the model vision-capable.
 */
export type QwenTokenPlanModelInput = 'text' | 'image';

/**
 Wire-compat pins copied verbatim by both harness projections: the builtin
 catalog's own pins, measured on all 10 rows it carries
 (`doc/planning/qwen-token-plan-individual-provider.md` section "Settled
 decisions"), so an overlapping model dispatches identically to the builtin.
 `supportsReasoningEffort` is true exactly when the model offers efforts. The
 key order matches the builtin rows' `compat` blocks byte for byte.
 */
export type QwenTokenPlanCompatPins = {
  readonly thinkingFormat: 'qwen';
  readonly supportsDeveloperRole: false;
  readonly supportsStore: false;
  readonly supportsStrictMode: true;
  readonly supportsReasoningEffort: boolean;
};

/**
 Wire-compat pins for a model that offers efforts, keyed exactly as the
 builtin catalog rows carry them so a projection copy stays byte-identical.
 */
export const QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS: QwenTokenPlanCompatPins = {
  supportsStrictMode: true,
  thinkingFormat: 'qwen',
  supportsDeveloperRole: false,
  supportsStore: false,
  supportsReasoningEffort: true,
};

/**
 Wire-compat pins for a model with no selectable efforts: identical to the
 builtin's pins for `qwen3.6-flash`, `qwen3.7-max`, and `qwen3.7-plus`, and the
 base every sibling-walked or default-served id derives its own pins from.
 */
export const QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS: QwenTokenPlanCompatPins = {
  supportsStrictMode: true,
  thinkingFormat: 'qwen',
  supportsDeveloperRole: false,
  supportsStore: false,
  supportsReasoningEffort: false,
};

/**
 One curated facts row: identity, the kind that decides serviceability, and
 the capabilities the table documents for the id. Optional fields stay absent
 rather than guessed, so the sibling walk can tell "unknown" from "known".
 */
export type QwenTokenPlanModelFacts = {
  readonly id: string;
  readonly name: string;
  readonly kind: QwenTokenPlanModelKind;
  readonly contextWindow?: number;
  readonly maxTokens?: number;
  readonly input: readonly QwenTokenPlanModelInput[];
  readonly reasoning: boolean;
  readonly efforts?: QwenTokenPlanEffortsDict;
  readonly compat: QwenTokenPlanCompatPins;
};

/**
 Facts with every fallback already applied: `contextWindow`, `maxTokens`,
 `input`, `efforts`, and `name` all decided, so each harness projection is a
 straight copy. `efforts` stays absent exactly for the two states that offer no
 selector (non-reasoning, and reasoning with no selectable efforts), which
 `reasoning` tells apart.
 */
export type QwenTokenPlanResolvedModelFacts = {
  readonly id: string;
  readonly name: string;
  readonly kind: QwenTokenPlanModelKind;
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly input: readonly QwenTokenPlanModelInput[];
  readonly reasoning: boolean;
  readonly efforts?: QwenTokenPlanEffortsDict;
  readonly compat: QwenTokenPlanCompatPins;
};

/**
 Context window an id with no family falls back to: pi's schema default,
 owner-settled beside the value in
 `doc/planning/qwen-token-plan-individual-provider.md` sections "Settled
 decisions" (facts priority) and "Model facts table" (the `auto` row).
 */
export const QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW = 128_000;

/**
 Max output an id with no family falls back to: pi's schema default,
 owner-settled beside the value in
 `doc/planning/qwen-token-plan-individual-provider.md` sections "Settled
 decisions" (facts priority) and "Model facts table" (the `auto` row).
 */
export const QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS = 16_384;

//endregion Public API
