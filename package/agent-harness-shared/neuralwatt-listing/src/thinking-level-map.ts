/**
 Thinking-level vocabulary and its wire expansion: the levels the listing's
 effort spellings map onto, and the map a harness's dispatch seam receives.
 @module
 */

//region Public API

/**
 Thinking levels this vocabulary expresses, in dispatch order: the exact set
 the effort table can name, so a listing spelling outside it skips instead of
 inventing a level.
 */
export const NEURALWATT_THINKING_LEVELS = [
  'off',
  'minimal',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
] as const;

/**
 One thinking level the effort table can name.
 */
export type NeuralWattThinkingLevel =
  (typeof NEURALWATT_THINKING_LEVELS)[number];

/**
 Effort spellings to thinking levels, the exact vocabulary this mapping
 expresses: each level dispatches the same spelling the listing gave, so the
 wire format never invents one. A read yields a level only for spellings the
 table names, so a spelling this vocabulary cannot express skips instead of
 inventing a level.
 */
export const NEURALWATT_EFFORT_LEVEL_BY_SPELLING:
  Readonly<Record<string, NeuralWattThinkingLevel>> = {
    none: 'off',
    minimal: 'minimal',
    low: 'low',
    medium: 'medium',
    high: 'high',
    xhigh: 'xhigh',
    max: 'max',
  };

/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors dsh-llm-pi-ai `PiAiReasoningEfforts = Partial<Record<ModelThinkingLevel, string | null>>` (`lib/types/catalog.d.ts`), where `null` is not absence but the instruction "supported, send nothing", so the value must reach the dispatch seam unchanged */
/**
 Offered efforts one model earns: each declared level carries the listing
 spelling dispatch sends. `null` is legal only for the `off` level, where it
 means "supported, send nothing": naming none is the parameter's absence, and
 a spelling cannot say that. The listing mapping emits spellings only.
 */
export type NeuralWattEffortsDict = {
  readonly off?: string | null;
  readonly minimal?: string | null;
  readonly low?: string | null;
  readonly medium?: string | null;
  readonly high?: string | null;
  readonly xhigh?: string | null;
  readonly max?: string | null;
};
/* oxlint-enable no-restricted-syntax/no-nullish-union */

/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors the pi-ai `thinkingLevelMap` value shape (`@earendil-works/pi-ai` `providers/model-schema.ts`), which distinguishes `null` (unsupported) from an absent key (supported, dispatch nothing), so the expansion must keep both spellings reachable */
/**
 Expanded thinking-level map one dispatch seam receives: every level decided
 explicitly. A declared level carries its wire spelling, a declared `null`
 stays absent ("supported, send nothing"), and every undeclared level pins
 `null` ("unsupported"). Pinning matters because pi-ai's own defaulting is
 asymmetric: an absent key means supported through `high` but unsupported for
 `xhigh` and `max`, and a profile author should not need to know that.
 */
export type NeuralWattThinkingLevelMap = {
  readonly off?: string | null;
  readonly minimal?: string | null;
  readonly low?: string | null;
  readonly medium?: string | null;
  readonly high?: string | null;
  readonly xhigh?: string | null;
  readonly max?: string | null;
};
/* oxlint-enable no-restricted-syntax/no-nullish-union */

/**
 Expand one offered efforts dict into the fully decided thinking-level map:
 declared levels carry their wire spelling, a declared `null` stays absent,
 and undeclared levels pin `null`. Mirrors `resolveModelReasoning`'s map
 building in `dsh-llm-pi-ai/lib/index.js`, so a harness projection and the
 shipped DSH adapter dispatch identically. Dict validity (non-empty spellings,
 a level beyond `off`) is the listing mapping's contract, not this expansion's.

  @param efforts - offered efforts one model earns

  @returns thinking-level map with every level decided

  @example
  ```ts
  const map = expandThinkingLevelMap({ efforts: { high: 'high' } });
  // { off: null, minimal: null, low: null, medium: null, high: 'high',
  //   xhigh: null, max: null }
  ```
  */
export function expandThinkingLevelMap({
  efforts,
}: {
  readonly efforts: NeuralWattEffortsDict;
}): NeuralWattThinkingLevelMap {
  /**
   Thinking-level map built level by level in dispatch order through a
   mutable scratch record, since the exported map's fields are readonly. The
   scratch must carry the `null` instruction itself, not merely omit it.
   */
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- builds the exported map, whose declaration documents the pi-ai `thinkingLevelMap` mirror; the scratch carries `null` (unsupported) distinctly from an absent key (supported, send nothing), exactly as the mirror requires */
  const scratch: Record<string, string | null> = {};
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
  for (const level of NEURALWATT_THINKING_LEVELS) {
    /**
     Wire spelling this level declares, or undefined when the dict is silent
     about it.
     */
    const wire = efforts[level];
    if (wire === undefined) {
      scratch[level] = null;
    } else if (wire !== null) {
      scratch[level] = wire;
    }
  }
  return scratch;
}

//endregion Public API
