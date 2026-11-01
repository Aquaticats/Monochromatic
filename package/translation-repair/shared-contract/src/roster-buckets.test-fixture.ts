/**
 The roster's Bedrock-only models and its seated single-provider judges,
 which only the seat, catalog and probe tests read (ledger B31). They
 shipped as package source (`model-card-derive.ts`,
 `corpus-run/run-config.ts`), with a Hyper-origin bucket no test needs
 once its row check went, while production seated every bench through the
 hold sets alone; the seating measurements the judge sets carried sit on
 the two models' cards in `model-cards.ts`.

 Fixtures are the roster itself; no corpus content appears here.

 @module
 */

import {
  BEDROCK_SERVED_IDS,
  type BedrockOnlyRosterId,
  holdSet,
  OPENROUTER_SERVED_IDS,
  type OpenRouterOnlyRosterId,
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '../dist/final/node/index.mjs';

/**
 Roster models only Amazon Bedrock serves: by the naming rule, the roster
 ids spelled Bedrock's way, in roster order.

 @example
 ```ts
 const everyone = BEDROCK_ONLY_ROSTER_IDS;
 ```
 */
export const BEDROCK_ONLY_ROSTER_IDS: readonly BedrockOnlyRosterId[] = ROSTER_MODEL_IDS
  .filter(function bedrockSpelled(modelId,): modelId is BedrockOnlyRosterId {
    return (BEDROCK_SERVED_IDS as readonly string[]).includes(modelId,);
  },);

/**
 Roster models only OpenRouter serves: by the naming rule, the roster ids
 spelled OpenRouter's way, in roster order.
 */
const OPENROUTER_ONLY_ROSTER_IDS: readonly OpenRouterOnlyRosterId[] = ROSTER_MODEL_IDS
  .filter(function openRouterSpelled(modelId,): modelId is OpenRouterOnlyRosterId {
    return (OPENROUTER_SERVED_IDS as readonly string[]).includes(modelId,);
  },);

/**
 Roster models whose card still holds them out of the judge seats until a
 fidelity probe measures them.
 */
const JUDGE_UNMEASURED: ReadonlySet<RosterModelId> = holdSet({ hold: 'judge-unmeasured', },);

/**
 Bedrock-only models the judge fidelity probe seated: the Bedrock bucket
 less the models still held unmeasured.

 @example
 ```ts
 const seated = SEATED_BEDROCK_JUDGES;
 ```
 */
export const SEATED_BEDROCK_JUDGES: readonly RosterModelId[] = BEDROCK_ONLY_ROSTER_IDS
  .filter(function seated(modelId,): boolean {
    return !JUDGE_UNMEASURED.has(modelId,);
  },);

/**
 OpenRouter-only models the judge fidelity probe seated: the OpenRouter
 bucket less the models still held unmeasured.

 @example
 ```ts
 const seated = SEATED_OPENROUTER_JUDGES;
 ```
 */
export const SEATED_OPENROUTER_JUDGES: readonly RosterModelId[] = OPENROUTER_ONLY_ROSTER_IDS
  .filter(function seated(modelId,): boolean {
    return !JUDGE_UNMEASURED.has(modelId,);
  },);
