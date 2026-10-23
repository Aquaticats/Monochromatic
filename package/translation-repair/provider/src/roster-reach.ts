import {
  type BedrockServedId,
  HYPER_MODELS,
  type HyperServedId,
  type OpenRouterServedId,
  PROVIDER_ORDER,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { BEDROCK_MODELS, } from './bedrock-catalog.ts';
import {
  deepSeekIdFor,
  deepSeekShowsPictures,
  xiaomiIdFor,
  xiaomiShowsPictures,
} from './direct-reach.ts';
import {
  claudeIdFor,
  claudeShowsPictures,
} from './claude-reach.ts';
import {
  neuralWattIdFor,
  neuralWattShowsPictures,
} from './neuralwatt-reach.ts';
import type { ModelReach, } from './budget-routing.ts';
import {
  OPENROUTER_DROPPED_SEATS,
  OPENROUTER_MODELS,
  OPENROUTER_PICTURES_UNMEASURED,
  OPENROUTER_WITHHELD,
} from './openrouter-catalog.ts';
import {
  SYNTHETIC_MODELS,
  SYNTHETIC_WITHHELD,
  type SyntheticModelInfo,
} from './synthetic-catalog.ts';

//region Roster reach
// WHICH PROVIDERS CAN SERVE ONE ROSTER MODEL, and under what spelling, read off
// every catalog rather than listed by hand.
//
// DERIVED, because a hand-written list goes stale silently. The GLM-4.7-Flash roster
// note makes the same argument about aliases: a roster fact that no build
// checks is a roster fact that changes without anyone noticing.
//
// READING IS NARROWER THAN TALKING. Each provider reports modalities for its
// own serving stack, so a call carrying a picture reaches only providers that
// both serve the roster identity and report image input for it. Asking one
// question for text and images would either send a picture where it cannot be
// read or refuse one that another serving path can accept.
//
// THREE SPELLINGS SINCE 2026-09-03. `hf:moonshotai/Kimi-K3`, `kimi-k3` and
// `moonshotai/kimi-k3` are one seat; the roster keeps the Synthetic spelling
// and each provider's translation lives beside that provider's catalog.
//
// A FOURTH SPELLING SINCE 2026-09-07. Amazon Bedrock spells the shared seats
// `google.gemma-4-26b-a4b` and `openai.gpt-oss-120b`, and adds two Gemma 4
// sizes no other provider serves, which the roster names by that spelling.

/**
 Where one roster model can be reached on Charm Hyper.

 @example
 ```ts
 const served = hyperIdFor({ modelId: 'hf:moonshotai/Kimi-K3', },);
 ```
 */
export type HyperSpelling =
  | {
    /**
     Discriminator marking a model this provider serves.
     */
    readonly served: true;

    /**
     Identifier to send, which differs from the roster's for shared models
     and for a seat that moved to a newer Hyper id.
     */
    readonly id: HyperServedId;
  }
    | {
    /**
     Discriminator marking a model this provider does not serve.
     */
      readonly served: false;
    };

/**
 Where one roster model can be reached on OpenRouter.

 @example
 ```ts
 const served = openRouterIdFor({ modelId: 'hf:moonshotai/Kimi-K3', },);
 ```
 */
export type OpenRouterSpelling =
  | {
    /**
     Discriminator marking a model this provider serves.
     */
    readonly served: true;

    /**
     Identifier to send, always different from the roster's.
     */
    readonly id: OpenRouterServedId;
  }
    | {
    /**
     Discriminator marking a model this provider does not serve.
     */
      readonly served: false;
    };

/**
 Where one roster model can be reached on Amazon Bedrock.

 @example
 ```ts
 const served = bedrockIdFor({ modelId: 'gemma-4-26b-a4b-it', },);
 ```
 */
export type BedrockSpelling =
  | {
    /**
     Discriminator marking a model this provider serves.
     */
    readonly served: true;

    /**
     Identifier to send, which differs from the roster's for shared models.
     */
    readonly id: BedrockServedId;
  }
    | {
    /**
     Discriminator marking a model this provider does not serve.
     */
      readonly served: false;
    };

/**
 How Charm Hyper spells one roster model, where it serves it at all.

 @param modelId - roster model to look up

 @returns Wire identifier, or that this provider does not serve it

 @example
 ```ts
 const spelling = hyperIdFor({ modelId, },);
 ```
 */
export function hyperIdFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): HyperSpelling {
  /**
   Entry on this roster seat's card, whatever its served spelling is: the
   roster id, a Synthetic spelling or an older name of a seat that moved to
   a newer Hyper id.
   */
  const entry = Object
    .values(HYPER_MODELS,)
    .find(function serves(info,): boolean {
      return info.rosterId === modelId;
    },);

  if (entry === undefined)
    return { served: false, };

  return {
    served: true,
    id: entry.id,
  };
}

/**
 How OpenRouter spells one roster model, where it serves it at all.

 @param modelId - roster model to look up

 @returns Wire identifier, or that this provider does not serve it

 @example
 ```ts
 const spelling = openRouterIdFor({ modelId, },);
 ```
 */
export function openRouterIdFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): OpenRouterSpelling {
  /**
   Entry standing in for this roster seat, if this provider has one.
   */
  const entry = Object
    .values(OPENROUTER_MODELS,)
    .find(function serves(info,): boolean {
      return info.sharedWith === modelId;
    },);

  if (entry === undefined)
    return { served: false, };

  return {
    served: true,
    id: entry.id,
  };
}

/**
 How Amazon Bedrock spells one roster model, where it serves it at all.

 @param modelId - roster model to look up

 @returns Wire identifier, or that this provider does not serve it

 @example
 ```ts
 const spelling = bedrockIdFor({ modelId, },);
 ```
 */
export function bedrockIdFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): BedrockSpelling {
  /**
   Entry standing in for this roster seat, if this provider has one.
   */
  const entry = Object
    .values(BEDROCK_MODELS,)
    .find(function serves(info,): boolean {
      return info.sharedWith === modelId;
    },);

  if (entry === undefined)
    return { served: false, };

  return {
    served: true,
    id: entry.id,
  };
}

/**
 What Synthetic knows about one roster model, where it serves it at all.

 @example
 ```ts
 const entry: SyntheticEntry = { served: true, info, };
 ```
 */
export type SyntheticEntry =
  | {
    /**
     Discriminator marking a model this provider serves.
     */
    readonly served: true;

    /**
     Catalog entry, carrying the facts routing and budgeting read.
     */
    readonly info: SyntheticModelInfo;
  }
    | {
    /**
     Discriminator marking a model this provider does not serve.
     */
      readonly served: false;
    };

/**
 What Synthetic's catalog says about one roster model.

 FOUND RATHER THAN INDEXED, mirroring {@link hyperIdFor}. Indexing the record
 with a roster id needs an assertion that the id is one of its keys, which is
 the claim this function exists to check.

 @param modelId - roster model to look up

 @returns Catalog entry, or that this provider does not serve it

 @example
 ```ts
 const entry = syntheticEntryFor({ modelId, },);
 ```
 */
export function syntheticEntryFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): SyntheticEntry {
  /**
   Entry naming this model, if this provider has one.
   */
  const info = Object
    .values(SYNTHETIC_MODELS,)
    .find(function serves(candidate,): boolean {
      return candidate.id === modelId;
    },);

  if (info === undefined)
    return { served: false, };

  return {
    served: true,
    info,
  };
}

/**
 Which providers can take a text call for one roster model.

 @param modelId - roster model to route

 @returns Reach for the budget router to decide on

 @example
 ```ts
 const reach = reachOf({ modelId, },);
 ```
 */
export function reachOf(
  { modelId, }: { readonly modelId: RosterModelId; },
): ModelReach {
  /**
   Synthetic's entry, which is also whether it serves this model at all.
   */
  const synthetic = syntheticEntryFor({ modelId, },);

  /**
   Hyper's spelling, which is also whether it serves this model at all.
   */
  const hyper = hyperIdFor({ modelId, },);

  /**
   OpenRouter's spelling, which is also whether it serves this model at all.
   */
  const openrouter = openRouterIdFor({ modelId, },);

  /**
   Bedrock's spelling, which is also whether it serves this model at all.
   */
  const bedrock = bedrockIdFor({ modelId, },);

  return {
    // SERVED, with no withholding: the free energy is not a purchase, and the
    // cards' own holds decide which roles a model fills (`neuralwatt-reach.ts`).
    neuralwatt: neuralWattIdFor({ modelId, },)
      .served,
    // SERVED AND ROUTED. A seat withheld from Synthetic on measured latency
    // (class one hundred eighteen, 2026-09-24) is one this provider does not
    // serve as far as the router is concerned (`SYNTHETIC_WITHHELD`).
    synthetic: synthetic.served && (!SYNTHETIC_WITHHELD.has(modelId,)),
    hyper: hyper.served,
    bedrock: bedrock.served,
    // SERVED, with no withholding, and for ONE MODEL EACH since 2026-10-22: the
    // owner's keys there are granted for DeepSeek V4.1 Flash and MiMo V2.6
    // Flash alone (`direct-reach.ts`), and the cards' own holds decide which
    // roles those models fill.
    deepseek: deepSeekIdFor({ modelId, },)
      .served,
    xiaomi: xiaomiIdFor({ modelId, },)
      .served,
    // SERVED, with no withholding, for the EIGHT CLAUDE CARDS ONLY since 2026-10-22: the owner's key is granted for
    // Sonnet 5.5 and Haiku 5.5 alone (`claude-reach.ts`), and the cards' own holds decide which roles they fill.
    anthropic: claudeIdFor({ modelId, },)
      .served,
    // SERVED AND BOUGHT. A seat the owner withheld from OpenRouter on cost
    // (2026-09-03) is one this provider does not serve as far as the router
    // is concerned, since 2026-09-09 (`OPENROUTER_WITHHELD`); so is a seat the
    // owner dropped from it (`OPENROUTER_DROPPED_SEATS`, ledger P11), which
    // held only while its card carried no OpenRouter block.
    openrouter: openrouter.served
      && (!OPENROUTER_WITHHELD.has(modelId,))
      && (!OPENROUTER_DROPPED_SEATS.has(modelId,)),
  };
}

/**
 Whether Synthetic will show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether this provider serves it AND reports vision for it

 @example
 ```ts
 const shows = syntheticShowsPictures({ modelId, },);
 ```
 */
function syntheticShowsPictures(
  { modelId, }: { readonly modelId: RosterModelId; },
): boolean {
  /**
   Synthetic's entry, which is also whether it serves this model at all.
   */
  const entry = syntheticEntryFor({ modelId, },);

  if ((!entry.served) || SYNTHETIC_WITHHELD.has(modelId,))
    return false;

  /**
   What that provider reports about this model's modalities.
   */
  const { readsImages: shows, } = entry.info;

  return shows;
}

/**
 Whether Charm Hyper will show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether this provider serves it AND reports vision for it

 @example
 ```ts
 const shows = hyperShowsPictures({ modelId, },);
 ```
 */
function hyperShowsPictures(
  { modelId, }: { readonly modelId: RosterModelId; },
): boolean {
  /**
   Hyper's spelling, which is also whether it serves this model at all.
   */
  const spelling = hyperIdFor({ modelId, },);

  if (!spelling.served)
    return false;

  /**
   What that provider reports about this model's modalities.
   */
  const { readsImages: shows, } = HYPER_MODELS[spelling.id];

  return shows;
}

/**
 Whether OpenRouter will show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether this provider serves it AND the listing reports vision for it
 AND no hold keeps pictures off this route until they are measured

 @example
 ```ts
 const shows = openRouterShowsPictures({ modelId, },);
 ```
 */
function openRouterShowsPictures(
  { modelId, }: { readonly modelId: RosterModelId; },
): boolean {
  /**
   OpenRouter's spelling, which is also whether it serves this model at all.
   */
  const spelling = openRouterIdFor({ modelId, },);

  if ((!spelling.served) || OPENROUTER_WITHHELD.has(modelId,)
    || OPENROUTER_DROPPED_SEATS.has(modelId,))
    return false;

  // THE LISTING SAYS IMAGE, NO TRANSCRIPTION IS MEASURED HERE: a picture is
  // not sent until the hold is lifted (`openrouter-pictures-unmeasured`).
  if (OPENROUTER_PICTURES_UNMEASURED.has(modelId,))
    return false;

  /**
   What the listing reports about this model's modalities.
   */
  const { readsImages: shows, } = OPENROUTER_MODELS[spelling.id];

  return shows;
}

/**
 Whether Amazon Bedrock will show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether this provider serves it AND its card reports image input

 @example
 ```ts
 const shows = bedrockShowsPictures({ modelId, },);
 ```
 */
function bedrockShowsPictures(
  { modelId, }: { readonly modelId: RosterModelId; },
): boolean {
  /**
   Bedrock's spelling, which is also whether it serves this model at all.
   */
  const spelling = bedrockIdFor({ modelId, },);

  if (!spelling.served)
    return false;

  /**
   What the model card reports about this model's modalities.
   */
  const { readsImages: shows, } = BEDROCK_MODELS[spelling.id];

  return shows;
}

/**
 Which providers can take a call carrying a picture for one roster model.

 NARROWER THAN {@link reachOf} AND DERIVED PER PROVIDER. A later catalog
 change can alter one serving stack's image support without altering another.

 @param modelId - roster model to route

 @returns Reach restricted to providers that report vision for it

 @example
 ```ts
 const reach = visionReachOf({ modelId, },);
 ```
 */
export function visionReachOf(
  { modelId, }: { readonly modelId: RosterModelId; },
): ModelReach {
  return {
    neuralwatt: neuralWattShowsPictures({ modelId, },),
    synthetic: syntheticShowsPictures({ modelId, },),
    hyper: hyperShowsPictures({ modelId, },),
    bedrock: bedrockShowsPictures({ modelId, },),
    deepseek: deepSeekShowsPictures({ modelId, },),
    xiaomi: xiaomiShowsPictures({ modelId, },),
    anthropic: claudeShowsPictures({ modelId, },),
    openrouter: openRouterShowsPictures({ modelId, },),
  };
}

/**
 Whether any provider can show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether a picture reaches it anywhere

 @example
 ```ts
 const reads = readsImages({ modelId, },);
 ```
 */
export function readsImages(
  { modelId, }: { readonly modelId: RosterModelId; },
): boolean {
  /**
   Providers that report vision for this model.
   */
  const reach = visionReachOf({ modelId, },);

  return PROVIDER_ORDER.some(function shows(provider,): boolean {
    return reach[provider];
  },);
}

//endregion Roster reach
