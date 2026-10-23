import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { modelDarkReason, } from '../dist/final/node/index.mjs';

//region Model-dark refusal
// THE REFUSAL THE ROUTER RAISES FOR A MODEL WHOSE OWN PROVIDERS ARE ALL OUT OF BUDGET while another provider reads wet:
// Hyper alone serves the model, Hyper is dry, and every other provider reads wet. It costs one seat its voice and
// leaves the run going, which the layers that turn a failed call into a lost voice must read it as.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The reason is the router's own (`modelDarkReason`), so a layer's case fails if the
// router's words ever stop being a lost seat.

/**
 Providers that serve the dark model: Hyper alone.
 */
const HYPER_ALONE = {
  neuralwatt: false,
  synthetic: false,
  hyper: true,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  openrouter: false,
} as const;

/**
 Providers that read dry: Hyper alone, so every other provider is wet.
 */
const HYPER_DRY = HYPER_ALONE;

/**
 Builds the refusal for one dark model.

 @param modelId - model the call was addressed to

 @returns The lost-seat refusal, whose reason names Hyper dry and every other provider wet

 @example
 ```ts
 throw modelDarkRefusal({ modelId: 'glm-5.3', },);
 ```
 */
export function modelDarkRefusal({ modelId, }: { readonly modelId: string; },): NoProviderForModelError {
  return new NoProviderForModelError({
    modelId,
    reason: modelDarkReason({
      reach: HYPER_ALONE,
      dry: HYPER_DRY,
    },),
  },);
}

//endregion Model-dark refusal
