import type {
  DeepSeekServedId,
  RosterModelId,
  XiaomiServedId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  DEEPSEEK_MODELS,
  XIAOMI_MODELS,
} from './direct-catalog.ts';

//region Direct reach
// WHERE ONE ROSTER MODEL CAN BE REACHED ON DEEPSEEK'S AND XIAOMI'S OWN APIS:
// DeepSeek V4.1 Flash on DeepSeek and MiMo V2.6 Flash on Xiaomi, one model
// each, read off the catalogs, which are read off the cards. No withholding
// set, as for NeuralWatt: the cards' own holds decide which roles a model
// fills, and this file decides only whether a call already addressed to the
// model can go to the direct route.

/**
 Where one roster model can be reached on DeepSeek.

 @example
 ```ts
 const served = deepSeekIdFor({ modelId: 'deepseek-v4.1-flash', },);
 ```
 */
export type DeepSeekSpelling =
  | {
    /**
     Discriminator marking a model this provider serves.
     */
    readonly served: true;

    /**
     Identifier to send, always `deepseek-flash`.
     */
    readonly id: DeepSeekServedId;
  }
    | {
    /**
     Discriminator marking a model this provider does not serve.
     */
      readonly served: false;
    };

/**
 Where one roster model can be reached on Xiaomi.

 @example
 ```ts
 const served = xiaomiIdFor({ modelId: 'xiaomi/mimo-v2.6-flash', },);
 ```
 */
export type XiaomiSpelling =
  | {
    /**
     Discriminator marking a model this provider serves.
     */
    readonly served: true;

    /**
     Identifier to send, always `mimo-v2.6-flash`.
     */
    readonly id: XiaomiServedId;
  }
    | {
    /**
     Discriminator marking a model this provider does not serve.
     */
      readonly served: false;
    };

/**
 How DeepSeek spells one roster model, where it serves it at all.

 @param modelId - roster model to look up

 @returns Wire identifier, or that this provider does not serve it

 @example
 ```ts
 const spelling = deepSeekIdFor({ modelId, },);
 ```
 */
export function deepSeekIdFor({ modelId, }: { readonly modelId: RosterModelId; },): DeepSeekSpelling {
  /**
   Entry standing in for this roster seat, if this provider has one.
   */
  const entry = Object
    .values(DEEPSEEK_MODELS,)
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
 How Xiaomi spells one roster model, where it serves it at all.

 @param modelId - roster model to look up

 @returns Wire identifier, or that this provider does not serve it

 @example
 ```ts
 const spelling = xiaomiIdFor({ modelId, },);
 ```
 */
export function xiaomiIdFor({ modelId, }: { readonly modelId: RosterModelId; },): XiaomiSpelling {
  /**
   Entry standing in for this roster seat, if this provider has one.
   */
  const entry = Object
    .values(XIAOMI_MODELS,)
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
 Whether DeepSeek will show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether this provider serves it AND its card reports image input

 @example
 ```ts
 const shows = deepSeekShowsPictures({ modelId, },);
 ```
 */
export function deepSeekShowsPictures({ modelId, }: { readonly modelId: RosterModelId; },): boolean {
  /**
   DeepSeek's spelling, which is also whether it serves this model at all.
   */
  const spelling = deepSeekIdFor({ modelId, },);
  if (!spelling.served)
    return false;
  return DEEPSEEK_MODELS[spelling.id]
    .readsImages;
}

/**
 Whether Xiaomi will show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether this provider serves it AND its card reports image input

 @example
 ```ts
 const shows = xiaomiShowsPictures({ modelId, },);
 ```
 */
export function xiaomiShowsPictures({ modelId, }: { readonly modelId: RosterModelId; },): boolean {
  /**
   Xiaomi's spelling, which is also whether it serves this model at all.
   */
  const spelling = xiaomiIdFor({ modelId, },);
  if (!spelling.served)
    return false;
  return XIAOMI_MODELS[spelling.id]
    .readsImages;
}

//endregion Direct reach
