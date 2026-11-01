import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatTextRequest,
  ReaskSeat,
} from '../dist/final/node/index.mjs';

//region Seated request
// A REQUEST FOR A MODEL THAT NAMES THE SEAT IT ASKS FOR, OR NONE, as the router reads it to decide whether the model's
// card holds it off a provider.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The tests of the question (`reask-withheld.unit.test.ts`) and of the router's refusal
// of its answer (`provider-router-held.unit.test.ts`) build the same request, so it is built here once.

/**
 Request for a model, naming the seat when one is given, as a router reads it.

 @param modelId - model asked

 @param reaskSeat - seat the request names, when it names one

 @returns Request carrying nothing the hold reads besides the model and the seat

 @example
 ```ts
 const reaskSeat: ReaskSeat = { role: 'editor', bench: [], reask: 'other-model', };
 const request = requestFor({ modelId: 'hf:Qwen/Qwen3.8-27B', reaskSeat, },);
 ```
 */
export function requestFor(
  {
    modelId,
    reaskSeat,
  }: {
    readonly modelId: RosterModelId;
    readonly reaskSeat?: ReaskSeat;
  },
): ChatTextRequest {
  return {
    modelId,
    messages: [
      {
        role: 'user',
        content: '这只猫睡在哪里？',
      },
    ],
    signal: new AbortController().signal,
    ...((reaskSeat === undefined) ? {} : { reaskSeat, }),
  };
}

//endregion Seated request
