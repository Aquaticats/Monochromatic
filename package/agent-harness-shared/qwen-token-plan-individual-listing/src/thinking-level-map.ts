/**
 Thinking-level vocabulary and its wire expansion: the levels the facts table's
 effort dicts map onto, and the map a harness's dispatch seam receives. The
 expansion reproduces pi-ai's builtin `thinkingLevelMap` value for value
 (`src/pi-ai-builtin-catalog.unit.test.ts` pins it against the checked-in
 fixture). @module
 */

//region Public API

/**
 Thinking levels this vocabulary expresses, in dispatch order: the exact set
 the efforts dict can name, and the exact key set pi-ai's `thinkingLevelMap`
 carries, so a harness projection never invents a level.
 */
export const QWEN_TOKEN_PLAN_THINKING_LEVELS = [
  'off',
  'minimal',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
] as const;

/**
 One thinking level the efforts dict can name.
 */
export type QwenTokenPlanThinkingLevel =
  (typeof QWEN_TOKEN_PLAN_THINKING_LEVELS)[number];

/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors dsh-llm-pi-ai `PiAiReasoningEfforts = Partial<Record<ModelThinkingLevel, string | null>>` (`lib/types/catalog.d.ts`), where `null` is not absence but the instruction "level unsupported", so the value must reach the dispatch seam unchanged */
/**
 Efforts one model offers: each declared level carries the wire spelling
 dispatch sends. `null` is legal only as the declaration "supported, send
 nothing"; naming none is the property's absence. The facts table carries
 spellings only (`doc/planning/qwen-token-plan-individual-provider.md` section
 "Model facts table").
 */
export type QwenTokenPlanEffortsDict = {
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
export type QwenTokenPlanThinkingLevelMap = {
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
 Expand one efforts dict into the fully decided thinking-level map: declared
 levels carry their wire spelling, a declared `null` stays absent, and
 undeclared levels pin `null`. For every facts row the pi-ai builtin catalog
 also carries, this expansion equals the builtin `thinkingLevelMap` value for
 value, so an overlapping model dispatches identically to the builtin.

  @param efforts - efforts one model offers

  @returns thinking-level map with every level decided

  @example
  ```ts
  const map = expandQwenTokenPlanThinkingLevelMap({
    efforts: { high: 'high' },
  });
  // { off: null, minimal: null, low: null, medium: null, high: 'high',
  //   xhigh: null, max: null }
  ```
  */
export function expandQwenTokenPlanThinkingLevelMap({
  efforts,
}: {
  readonly efforts: QwenTokenPlanEffortsDict;
}): QwenTokenPlanThinkingLevelMap {
  /**
   Thinking-level map built level by level in dispatch order through a mutable
   scratch record, since the exported map's fields are readonly. The scratch
   must carry the `null` instruction itself, not merely omit it.
   */
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- builds the exported map, whose declaration documents the pi-ai `thinkingLevelMap` mirror; the scratch carries `null` (unsupported) distinctly from an absent key (supported, send nothing), exactly as the mirror requires */
  const scratch: Record<string, string | null> = {};
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
  for (const level of QWEN_TOKEN_PLAN_THINKING_LEVELS) {
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
