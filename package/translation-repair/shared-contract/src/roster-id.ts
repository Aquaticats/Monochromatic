//region Roster identity
// WHO IS ON THE ROSTER AND HOW EACH PROVIDER SPELLS THEM, named once.
//
// This file imports nothing on purpose. The card type, the cards and every
// catalog need these names and each other's would be a cycle, so the
// identity lives on its own and the cards describe it.
//
// LISTS RATHER THAN A DERIVATION FROM THE CARDS because declaration emit
// (`isolatedDeclarations`) cannot infer a nested object literal, so the
// literal unions the catalogs key on have to come from `as const` arrays of
// strings. The cards in `model-cards.ts` are a `Record` over the roster
// list, so a name added here without a card, or a card without a name, is a
// type error, and a card naming a served id missing from its provider's
// list is one too.
//
// Roster identities are independent of provider routes. A model is one
// roster entry however many providers can reach it, which is the property
// the adjudication tally depends on: `synthetic-catalog.ts` explains at
// length why one model occupying two seats would silently overstate
// agreement, and a second provider is exactly the new way for that to
// happen.
//
// SHARED MODELS ARE NAMED THE SYNTHETIC WAY, else the Hyper way, else
// Bedrock's, else OpenRouter's, else the decisions endpoint's, in the order
// the providers joined.
// `hf:moonshotai/Kimi-K3` and `kimi-k3` are one model with two spellings,
// and the roster needs one of them; Synthetic was here first, its ids are
// already written into settled artifacts, and `roster-reach.ts` translates
// when a call actually goes to another provider.

/**
 Every model this pipeline may seat, in the order the roster reads them:
 the Synthetic four, the Hyper-origin seats, then the Bedrock-only and
 OpenRouter-only arrivals.

 @example
 ```ts
 const everyone = ROSTER_MODEL_IDS;
 ```
 */
export const ROSTER_MODEL_IDS = [
  'hf:zai-org/GLM-5.3-Flash',
  'hf:Qwen/Qwen3.8-27B',
  'hf:moonshotai/Kimi-K3',
  'hf:openai/gpt-oss-120b',
  'minimax-m3',
  'gemma-4-26b-a4b-it',
  'glm-5.3',
  'deepseek-v4.1-flash',
  'google.gemma-4-e2b',
  'google.gemma-4-31b',
  'inception/mercury-2.5',
  'xiaomi/mimo-v2.6-flash',
  'typesafe/jev-1.13',
  'claude-sonnet-5-5',
  'claude-sonnet-5-5-effort-low',
  'claude-sonnet-5-5-effort-medium',
  'claude-sonnet-5-5-effort-high',
  'claude-haiku-5-5',
  'claude-haiku-5-5-effort-low',
  'claude-haiku-5-5-effort-medium',
  'claude-haiku-5-5-effort-high',
] as const;

/**
 Every model this pipeline may seat, whoever serves it.

 @example
 ```ts
 const modelId: RosterModelId = 'hf:moonshotai/Kimi-K3';
 ```
 */
export type RosterModelId = typeof ROSTER_MODEL_IDS[number];

/**
 Spellings Synthetic serves, which are also those models' roster spellings.

 @example
 ```ts
 const everyone = SYNTHETIC_SERVED_IDS;
 ```
 */
export const SYNTHETIC_SERVED_IDS = [
  'hf:zai-org/GLM-5.3-Flash',
  'hf:Qwen/Qwen3.8-27B',
  'hf:moonshotai/Kimi-K3',
  'hf:openai/gpt-oss-120b',
] as const;

/**
 Roster models Synthetic serves.

 @example
 ```ts
 const modelId: SyntheticServedId = 'hf:zai-org/GLM-5.3-Flash';
 ```
 */
export type SyntheticServedId = typeof SYNTHETIC_SERVED_IDS[number];

/**
 Spellings Charm Hyper serves.

 @example
 ```ts
 const everyone = HYPER_SERVED_IDS;
 ```
 */
export const HYPER_SERVED_IDS = [
  'glm-5.3-flash',
  'qwen3.8-27b',
  'kimi-k3',
  'gpt-oss-120b',
  'minimax-m3',
  'gemma-4-26b-a4b-it',
  'glm-5.3',
  'deepseek-v4.1-flash-1017',
] as const;

/**
 Model identifier Charm Hyper accepts on the wire.

 @example
 ```ts
 const modelId: HyperServedId = 'deepseek-v4.1-flash-1017';
 ```
 */
export type HyperServedId = typeof HYPER_SERVED_IDS[number];

/**
 Spellings OpenRouter serves.

 @example
 ```ts
 const everyone = OPENROUTER_SERVED_IDS;
 ```
 */
export const OPENROUTER_SERVED_IDS = [
  'z-ai/glm-5.3-flash',
  'moonshotai/kimi-k3',
  'openai/gpt-oss-120b',
  'minimax/minimax-m3',
  'google/gemma-4-26b-a4b-it',
  'deepseek/deepseek-v4.1-flash-1017',
  'inception/mercury-2.5',
  'xiaomi/mimo-v2.6-flash',
] as const;

/**
 Model identifier OpenRouter accepts on the wire.

 @example
 ```ts
 const modelId: OpenRouterServedId = 'moonshotai/kimi-k3';
 ```
 */
export type OpenRouterServedId = typeof OPENROUTER_SERVED_IDS[number];

/**
 Spellings Amazon Bedrock serves.

 @example
 ```ts
 const everyone = BEDROCK_SERVED_IDS;
 ```
 */
export const BEDROCK_SERVED_IDS = [
  'openai.gpt-oss-120b',
  'google.gemma-4-26b-a4b',
  'google.gemma-4-e2b',
  'google.gemma-4-31b',
] as const;

/**
 Model identifier Amazon Bedrock accepts on the wire.

 @example
 ```ts
 const modelId: BedrockServedId = 'google.gemma-4-31b';
 ```
 */
export type BedrockServedId = typeof BEDROCK_SERVED_IDS[number];

/**
 Spellings NeuralWatt serves for this pipeline: the two standard ids the
 owner's free daily energy covers (`GET https://api.neuralwatt.com/v1/models`,
 public, fetched 2026-10-20).

 THE STANDARD LANE ONLY. `deepseek-v4.1-flash-flex` and `glm-5.3-flash-flex`
 are billed at 0.65 of the standard price and "may be held under load";
 `deepseek-v4.1-flash-speed` is not covered by the free energy; `nw-flash` is
 an alias that names no model. The free energy covers a covered model's
 standard and flex lanes and not its speed lane
 (`https://docs.neuralwatt.com/billing/daily-free-energy`), and the flex lane
 is held server-side during peak, which a pipeline waiting on a round window
 cannot use.

 @example
 ```ts
 const everyone = NEURALWATT_SERVED_IDS;
 ```
 */
export const NEURALWATT_SERVED_IDS = [
  'deepseek-v4.1-flash',
  'glm-5.3-flash',
] as const;

/**
 Model identifier NeuralWatt accepts on the wire.

 @example
 ```ts
 const modelId: NeuralWattServedId = 'glm-5.3-flash';
 ```
 */
export type NeuralWattServedId = typeof NEURALWATT_SERVED_IDS[number];

/**
 The one spelling DeepSeek's own API serves for this pipeline: `deepseek-flash`,
 which DeepSeek's documentation says "is now served by DeepSeek-V4.1-Flash-1017"
 (`https://api-docs.deepseek.com/quick_start/pricing`, read 2026-10-22).

 A SINGLE LITERAL ON PURPOSE: THE OWNER'S LOCK ON THE KEY, in the type. The
 owner, 2026-10-22: "You can also use the DEEPSEEK_API_KEY as you wish, but you
 may only use deepseek v4.1 flash 1017 with that key." The key is not
 model-scoped (`GET /models` also lists `deepseek-v4-pro`), so the lock is in our
 code: a second spelling here is a type error in every file that reads the
 served id, and `direct-cards.unit.test.ts` fails on a second card side or
 another id. There is no dated 1017 id; the 1017 claim rests on DeepSeek's
 documentation.

 @example
 ```ts
 const everyone = DEEPSEEK_SERVED_IDS;
 ```
 */
export const DEEPSEEK_SERVED_IDS = [
  'deepseek-flash',
] as const;

/**
 Model identifier DeepSeek's own API accepts on the wire: the one the key is
 granted for.

 @example
 ```ts
 const modelId: DeepSeekServedId = 'deepseek-flash';
 ```
 */
export type DeepSeekServedId = typeof DEEPSEEK_SERVED_IDS[number];

/**
 The one spelling Xiaomi's own API serves for this pipeline: `mimo-v2.6-flash`.

 A SINGLE LITERAL ON PURPOSE: THE OWNER'S LOCK ON THE KEY, in the type. The
 owner, 2026-10-22: "I've added TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY in main
 worktree's json file. ... You may only use v2.6-flash with this key." The key
 is not model-scoped (`GET /v1/models` lists `mimo-v2.6-pro`, the speech models
 and others), so the lock is in our code, as for DeepSeek.

 @example
 ```ts
 const everyone = XIAOMI_SERVED_IDS;
 ```
 */
export const XIAOMI_SERVED_IDS = [
  'mimo-v2.6-flash',
] as const;

/**
 Model identifier Xiaomi's own API accepts on the wire: the one the key is
 granted for.

 @example
 ```ts
 const modelId: XiaomiServedId = 'mimo-v2.6-flash';
 ```
 */
export type XiaomiServedId = typeof XIAOMI_SERVED_IDS[number];

/**
 The only two model ids a request on the owner's Claude key may carry: Sonnet 5.5 and Haiku 5.5.

 A TWO-LITERAL LIST ON PURPOSE: THE OWNER'S LOCK ON THE KEY, in the type. The owner, 2026-10-22: "I've added
 TRANSLATION_REPAIR_CLAUDE_API_KEY . You may use latest Haiku and latest Sonnet with this key. This key would have
 $200 every month so be careful what you use." The key is not model-scoped (it lists Opus, Fable and older models),
 so the lock is in our code: a third id here is a type error in every file that reads a wire model, and
 `claude-cards.unit.test.ts` fails on another one. The ids are the pricing page's (`claude-sonnet-5-5`,
 `claude-haiku-5-5`, read 2026-10-22); neither takes a date suffix.

 @example
 ```ts
 const everyone = ANTHROPIC_WIRE_MODELS;
 ```
 */
export const ANTHROPIC_WIRE_MODELS = [
  'claude-sonnet-5-5',
  'claude-haiku-5-5',
] as const;

/**
 A model id that may go in the `model` member of a request on the owner's Claude key.

 @example
 ```ts
 const model: AnthropicWireModel = 'claude-haiku-5-5';
 ```
 */
export type AnthropicWireModel = typeof ANTHROPIC_WIRE_MODELS[number];

/**
 The efforts the roster carries a card for, each sent as `output_config.effort`.

 THREE, NOT FIVE. The API takes `low` to `max` on both models, and the owner declined to pick a level ("Why don't you
 bench?"), so the route ships the three levels the bench is asked to separate. A card that sends no effort exists
 beside them for each model (the model's own default: Sonnet `high`, Haiku `medium`).

 @example
 ```ts
 const everyone = ANTHROPIC_EFFORTS;
 ```
 */
export const ANTHROPIC_EFFORTS = [
  'low',
  'medium',
  'high',
] as const;

/**
 An effort the roster carries a card for.

 @example
 ```ts
 const effort: AnthropicEffort = 'medium';
 ```
 */
export type AnthropicEffort = typeof ANTHROPIC_EFFORTS[number];

/**
 Spellings the Claude route serves: one per model with no effort and one per model and effort.

 SEVERAL SPELLINGS, ONE WIRE MODEL, because a roster identity is one seat and the effort bench seats each level on its
 own: an effort variant is a distinct card the bench can seat. The spelling is not what goes in the `model` member of
 the request (`ANTHROPIC_WIRE_MODELS` is); it names the card, and the card names the wire model and the effort.

 @example
 ```ts
 const everyone = ANTHROPIC_SERVED_IDS;
 ```
 */
export const ANTHROPIC_SERVED_IDS = [
  'claude-sonnet-5-5',
  'claude-sonnet-5-5-effort-low',
  'claude-sonnet-5-5-effort-medium',
  'claude-sonnet-5-5-effort-high',
  'claude-haiku-5-5',
  'claude-haiku-5-5-effort-low',
  'claude-haiku-5-5-effort-medium',
  'claude-haiku-5-5-effort-high',
] as const;

/**
 A card spelling of the Claude route.

 @example
 ```ts
 const modelId: AnthropicServedId = 'claude-haiku-5-5-effort-low';
 ```
 */
export type AnthropicServedId = typeof ANTHROPIC_SERVED_IDS[number];

/**
 Roster identities introduced through Charm Hyper without a Synthetic
 spelling: by the naming rule, the roster ids that are Hyper spellings.
 The historical bucket name is not exclusive reach: OpenRouter serves
 several too.

 `deepseek-v4.1-flash` is no longer in it: Hyper serves that seat as
 `deepseek-v4.1-flash-1017` since 2026-10-20, and the roster id kept its
 name (`HyperModelInfo.rosterId` ties the two).

 @example
 ```ts
 const modelId: HyperOriginRosterId = 'minimax-m3';
 ```
 */
export type HyperOriginRosterId = Extract<RosterModelId, HyperServedId>;

/**
 Roster models only Amazon Bedrock serves: by the naming rule, the roster
 ids that are Bedrock spellings.

 @example
 ```ts
 const modelId: BedrockOnlyRosterId = 'google.gemma-4-31b';
 ```
 */
export type BedrockOnlyRosterId = Extract<RosterModelId, BedrockServedId>;

/**
 Roster models only OpenRouter serves: by the naming rule, the roster ids
 that are OpenRouter spellings.

 @example
 ```ts
 const modelId: OpenRouterOnlyRosterId = 'inception/mercury-2.5';
 ```
 */
export type OpenRouterOnlyRosterId = Extract<RosterModelId, OpenRouterServedId>;

/**
 Model identifier OpenRouter's decisions endpoint accepts on the wire:
 typed-decision models that take no chat completion (`decisions-client.ts`),
 spelled apart from the chat spellings so no chat catalog, cap table or reach
 ever names them. Jev 1.13 joined on 2026-09-18 on the owner's approval.

 A UNION WITHOUT A LIST (ledger B31). No production code iterates the
 decisions spellings, so the list this type was once derived from was read
 by tests alone; a decisions model joins this union and gets a card, whose
 `decisions.id` the type checker holds to it.

 @example
 ```ts
 const modelId: OpenRouterDecisionId = 'typesafe/jev-1.13';
 ```
 */
export type OpenRouterDecisionId = 'typesafe/jev-1.13';

/**
 Roster models only the decisions endpoint serves: by the naming rule, the
 roster ids that are decisions spellings.

 @example
 ```ts
 const modelId: DecisionOnlyRosterId = 'typesafe/jev-1.13';
 ```
 */
export type DecisionOnlyRosterId = Extract<RosterModelId, OpenRouterDecisionId>;

//endregion Roster identity
