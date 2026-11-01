/**
 Model-id and capability policy for choosing pi thinking defaults.
 
 @module
 */

import { getModelIdLeaf, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';

export { getModelIdLeaf, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';

//region Thinking default constants

/**
 Thinking levels this extension writes as defaults.
 */
export type ThinkingDefaultLevel = 'high' | 'xhigh';

/**
 Thinking level requested for GPT-shaped model ids and for any model that
 supports `xhigh`.
 */
const XHIGH_THINKING_DEFAULT: ThinkingDefaultLevel = 'xhigh';

/**
 Fallback thinking level used when a model does not support `xhigh`.
 */
const HIGH_THINKING_DEFAULT: ThinkingDefaultLevel = 'high';

//endregion Thinking default constants

//region Model shapes

/**
 Fragment of pi's `ThinkingLevelMap` consulted by the xhigh availability check.
 
 Pi's `@earendil-works/pi-ai` types each entry as `string` (the provider value
 sent for that level) or `null` (the level is hidden). The `xhigh` entry is
 `string | null` in the catalog, so a structural mirror that accepts real
 models cannot avoid the nullish union.
 */
export type ThinkingLevelMapFragment = {
  /**
   Provider value for `xhigh`, or `null` when the model hides the level.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors `@earendil-works/pi-ai`'s `ThinkingLevelMap`, whose `xhigh` entry is `string | null` (`null` marks the level hidden); real models carry `null`, so the mirror cannot avoid the union
  readonly xhigh?: string | null;
};

/**
 Minimal model shape needed by the thinking policy.
 
 `reasoning` and `thinkingLevelMap` are optional so GPT-shaped ids, whose
 default depends on the id leaf alone, can be exercised without fabricating
 capability fields.
 */
export type ModelWithId = {
  /**
   Model identifier as passed through pi.
   */
  readonly id: string;
  /**
   Whether the model emits reasoning content, as registered by its provider.
   */
  readonly reasoning?: boolean;
  /**
   Provider-specific mapping of pi thinking levels; only the `xhigh` entry is
   consulted here.
   */
  readonly thinkingLevelMap?: ThinkingLevelMapFragment;
};

//endregion Model shapes

//region Model id helpers

/**
 Detects whether a model id is GPT-shaped.
 
 A model is GPT-shaped when the final slash-delimited segment, taken via
 {@link getModelIdLeaf}, starts with `gpt-` after lowercasing. Other
 provider separators are intentionally not special-cased.
 
 @param modelId - model id to inspect
 
 @returns whether the model id leaf starts with `gpt-`
 
 @example
 ```typescript
 isGptModelId({ modelId: 'openai/gpt-5.5' }); // true
 ```
 */
export function isGptModelId(
  {
    modelId,
  }: {
    readonly modelId: string;
  },
): boolean {
  /**
   Lowercased final segment used for case-insensitive GPT detection.
   */
  const normalizedLeaf = getModelIdLeaf({ modelId, },)
    .toLowerCase();
  return normalizedLeaf.startsWith('gpt-',);
}

/**
 Detects whether a model supports the `xhigh` thinking level.
 
 Pi treats `xhigh` as opt-in: unlike the other thinking levels, where a
 missing `thinkingLevelMap` entry falls back to provider defaults, `xhigh`
 is supported only when a model declares `reasoning` and maps `xhigh` to a
 non-null value. This mirrors pi's `getSupportedThinkingLevels` so the level
 requested here is the level pi retains after clamping, which keeps the
 active level stable across repeated model events instead of being
 downgraded and re-requested each time.
 
 @param model - model with optional reasoning and thinking-level map fields
 
 @returns whether pi would retain `xhigh` for the model
 
 @example
 ```typescript
 isXhighAvailable({ model: { id: 'synthetic/hf:zai-org/GLM-5.2', reasoning: true, thinkingLevelMap: { xhigh: 'max' } } }); // true
 ```
 */
export function isXhighAvailable(
  {
    model,
  }: {
    readonly model: ModelWithId;
  },
): boolean {
  return (model.reasoning === true) && ((typeof model.thinkingLevelMap
    ?.xhigh) === 'string');
}

/**
 Returns the desired thinking default for a model, delegating the GPT check
 to {@link isGptModelId} and the `xhigh` capability check to
 {@link isXhighAvailable}.
 
 GPT-shaped ids get `xhigh`. Other ids get `xhigh` when the model supports it
 and `high` otherwise, so models without an `xhigh` mapping keep the lighter
 `high` default instead of being clamped down.
 
 @param model - model with an id field from pi
 
 @returns thinking level target for that model
 
 @example
 ```typescript
 getThinkingDefaultForModel({ model: { id: 'gpt-5.5' } }); // 'xhigh'
 ```
 */
export function getThinkingDefaultForModel(
  {
    model,
  }: {
    readonly model: ModelWithId;
  },
): ThinkingDefaultLevel {
  if (isGptModelId({ modelId: model.id, },))
    return XHIGH_THINKING_DEFAULT;
  return isXhighAvailable({ model, },)
    ? XHIGH_THINKING_DEFAULT
    : HIGH_THINKING_DEFAULT;
}

//endregion Model id helpers

export {
  XHIGH_THINKING_DEFAULT,
  HIGH_THINKING_DEFAULT,
};
