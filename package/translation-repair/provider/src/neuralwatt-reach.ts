import type {
  NeuralWattServedId,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { NEURALWATT_MODELS, } from './neuralwatt-catalog.ts';

//region NeuralWatt reach
// WHERE ONE ROSTER MODEL CAN BE REACHED ON NEURALWATT: the two models the
// owner's free daily energy covers, `deepseek-v4.1-flash` and `glm-5.3-flash`
// (`GET https://api.neuralwatt.com/v1/models`, public, read 2026-10-20). Read
// off the catalog, which is read off the cards, so a third model gets here by
// being given a `neuralwatt` side on its card and by nothing else.
//
// NO WITHHOLDING SET, unlike Synthetic and OpenRouter: a seat is held out of
// a provider there by a measured latency or a cost ruling, and neither applies
// to energy that is free until the day's allowance is spent. The cards' own
// holds (`GLM_5_3_FLASH` unseated as judge on 2026-09-02, DeepSeek V4.1 Flash
// `reader-unmeasured`) decide which roles a model may fill; this file decides
// only whether a call already addressed to the model can go to NeuralWatt.

/**
 Where one roster model can be reached on NeuralWatt.

 @example
 ```ts
 const served = neuralWattIdFor({ modelId: 'deepseek-v4.1-flash', },);
 ```
 */
export type NeuralWattSpelling =
  | {
    /**
     Discriminator marking a model this provider serves.
     */
    readonly served: true;

    /**
     Identifier to send, which differs from the roster's for shared models.
     */
    readonly id: NeuralWattServedId;
  }
    | {
    /**
     Discriminator marking a model this provider does not serve.
     */
      readonly served: false;
    };

/**
 How NeuralWatt spells one roster model, where it serves it at all.

 @param modelId - roster model to look up

 @returns Wire identifier, or that this provider does not serve it

 @example
 ```ts
 const spelling = neuralWattIdFor({ modelId, },);
 ```
 */
export function neuralWattIdFor({ modelId, }: { readonly modelId: RosterModelId; },): NeuralWattSpelling {
  /**
   Entry standing in for this roster seat, if this provider has one.
   */
  const entry = Object
    .values(NEURALWATT_MODELS,)
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
 Whether NeuralWatt will show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether this provider serves it AND its listing reports image input

 @example
 ```ts
 const shows = neuralWattShowsPictures({ modelId, },);
 ```
 */
export function neuralWattShowsPictures({ modelId, }: { readonly modelId: RosterModelId; },): boolean {
  /**
   NeuralWatt's spelling, which is also whether it serves this model at all.
   */
  const spelling = neuralWattIdFor({ modelId, },);
  if (!spelling.served)
    return false;
  return NEURALWATT_MODELS[spelling.id]
    .readsImages;
}

//endregion NeuralWatt reach
