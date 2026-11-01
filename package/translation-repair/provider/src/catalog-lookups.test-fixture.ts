/**
 Test-only lookups over the provider catalogs and the model cards (ledger
 B30). They shipped as package source while only tests called them: the
 roster-reach test proves every roster id is some provider's row through
 the label lookups, and the catalog tests read the Hyper counterpart split
 and single cards. Production reads the tables and `ROSTER_CARDS` directly.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type BedrockServedId,
  HYPER_MODELS,
  type HyperServedId,
  MODEL_CARDS,
  NO_SYNTHETIC_COUNTERPART,
  type OpenRouterServedId,
  type RosterCard,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  BEDROCK_MODELS,
  OPENROUTER_MODELS,
} from '../dist/final/node/index.mjs';

/**
 Whether a label is one of Bedrock's spellings.

 @param label - model label as a log line or a caller wrote it

 @returns Whether `BEDROCK_MODELS` has a row under it

 @example
 ```ts
 const served = bedrockServesLabel('anthropic.claude-sonnet-5',);
 ```
 */
export function bedrockServesLabel(label: string,): label is BedrockServedId {
  return Object.hasOwn(
    BEDROCK_MODELS,
    label,
  );
}

/**
 Whether OpenRouter's catalog carries a label under that exact spelling.

 A LABEL, NOT A ROSTER ID: the roster never names a model the OpenRouter
 way, so this answers only whether a spelling is one of its rows.

 @param label - spelling being looked up

 @returns Whether `OPENROUTER_MODELS` has a row under it

 @example
 ```ts
 const served = openRouterServesLabel('moonshotai/kimi-k3',);
 ```
 */
export function openRouterServesLabel(label: string,): label is OpenRouterServedId {
  return Object.hasOwn(
    OPENROUTER_MODELS,
    label,
  );
}

/**
 Whether Charm Hyper's catalog carries a label under that exact spelling.

 A LABEL, NOT A ROSTER ID: the roster names shared models by their Synthetic
 spelling and reaches Hyper for them through `hyperIdFor`, so this answers
 only whether the given spelling is a Hyper row, which for the roster means
 the Hyper-only labels.

 @param label - spelling being looked up

 @returns Whether `HYPER_MODELS` has a row under it

 @example
 ```ts
 const served = hyperServesLabel('minimax-m3',);
 ```
 */
export function hyperServesLabel(label: string,): label is HyperServedId {
  return Object.hasOwn(
    HYPER_MODELS,
    label,
  );
}

/**
 Hyper models that stand in for one Synthetic serves.

 @returns Their identifiers, in catalog order

 @example
 ```ts
 const shared = hyperModelsWithSyntheticCounterparts();
 ```
 */
export function hyperModelsWithSyntheticCounterparts(): readonly HyperServedId[] {
  return Object
    .values(HYPER_MODELS,)
    .filter(function shared(info,): boolean {
      return info.sharedWith !== NO_SYNTHETIC_COUNTERPART;
    },)
    .map(function toId(info,): HyperServedId {
      return info.id;
    },);
}

/**
 Hyper models without a Synthetic counterpart. Other provider reach is
 decided by `reachOf`, not by this split; an OpenRouter route may still
 exist for the same model.

 @returns Their identifiers, in catalog order

 @example
 ```ts
 const withoutSynthetic = hyperModelsWithoutSyntheticCounterparts();
 ```
 */
export function hyperModelsWithoutSyntheticCounterparts(): readonly HyperServedId[] {
  return Object
    .values(HYPER_MODELS,)
    .filter(function alone(info,): boolean {
      return info.sharedWith === NO_SYNTHETIC_COUNTERPART;
    },)
    .map(function toId(info,): HyperServedId {
      return info.id;
    },);
}

/**
 Card of one roster model.

 @param modelId - roster model to look up

 @returns Its card beside its id

 @example
 ```ts
 const card = cardOf({ modelId: 'minimax-m3', },);
 ```
 */
export function cardOf(
  { modelId, }: { readonly modelId: RosterModelId; },
): RosterCard {
  return {
    id: modelId,
    ...MODEL_CARDS[modelId],
  };
}
