import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelReach, } from './budget-routing.ts';
import {
  carriesPicture,
  type ChatTextRequest,
} from './chat-contract.ts';
import {
  type CredentialHolds,
  reachPastCredentialHolds,
} from './credential-hold.ts';
import {
  reachOf,
  visionReachOf,
} from './roster-reach.ts';
import {
  reachPastBoundHolds,
  type StreamBoundHolds,
} from './stream-bound-hold.ts';

//region Reach past holds
// WHICH PROVIDERS MAY TAKE A CALL ONCE EVERY HOLD THE ROUTER KEEPS IS READ:
// the credential holds (a provider, for every model, for the run) and the stream
// bound holds (a provider and a model, for a backoff). Split from
// `provider-router.ts` at its line budget.

/**
 Providers that can serve one call, narrowed to vision where it carries a
 picture.

 @param request - call whose reach is read

 @returns Reach for the policy, or for a re-ask, to decide on

 @example
 ```ts
 const reach = reachOfRequest({ request, },);
 ```
 */
export function reachOfRequest(
  { request, }: { readonly request: ForeignBorrowed<ChatTextRequest>; },
): ModelReach {
  if (carriesPicture({ messages: request.messages, },))
    return visionReachOf({ modelId: request.modelId, },);
  return reachOf({ modelId: request.modelId, },);
}

/**
 Whether no provider is left in a reach.

 @param reach - providers read

 @returns True where every provider is closed

 @example
 ```ts
 const nobody = closedEverywhere({ reach, },);
 ```
 */
function closedEverywhere({ reach, }: { readonly reach: ModelReach; },): boolean {
  return PROVIDER_ORDER.every(function closed(provider: ProviderName,): boolean {
    return !reach[provider];
  },);
}

/**
 Reads which providers may take one call once the credential holds and the
 stream bound holds are taken out, refusing where the holds leave nobody.

 THE CREDENTIAL HOLDS ARE READ FIRST, so a model whose every serving provider
 had its credential refused ends with that reason and not with a stream bound
 hold it never met.

 @param reach - providers that serve the model, narrowed to vision where the
 call carries a picture

 @param modelId - model being routed

 @param credentialHolds - the router's credential holds

 @param boundHolds - the router's stream bound holds

 @returns Providers serving the model that hold nothing against it

 @throws {@link NoProviderForModelError} when every provider serving the
 model has its credential refused, or holds the model out on its stream bound

 @example
 ```ts
 const open = reachPastHoldsOrRefusal({ reach, modelId, credentialHolds, boundHolds, },);
 ```
 */
export function reachPastHoldsOrRefusal(
  {
    reach,
    modelId,
    credentialHolds,
    boundHolds,
  }: {
    readonly reach: ModelReach;
    readonly modelId: string;
    readonly credentialHolds: CredentialHolds;
    readonly boundHolds: StreamBoundHolds;
  },
): ModelReach {
  /**
   Providers that serve the model and have their credential accepted, and the
   serving ones taken out.
   */
  const {
    reach: accepted,
    refused,
  } = reachPastCredentialHolds({
    reach,
    holds: credentialHolds,
  },);
  if ((refused.length > 0) && closedEverywhere({ reach: accepted, },)) {
    // REFUSED WITHOUT A CALL, so a round counts the seat unreachable and
    // neither asks nor waits for it. The seat is withheld only here: no
    // provider that serves it has a credential the run can use.
    throw new NoProviderForModelError({
      modelId,
      reason: `every provider serving this model had its credential refused this run (${refused.join(', ',)})`,
    },);
  }

  /**
   The same, less any holding the model out on its stream bound.
   */
  const {
    reach: open,
    heldMs,
  } = reachPastBoundHolds({
    reach: accepted,
    modelId,
    holds: boundHolds,
  },);
  if ((heldMs > 0) && closedEverywhere({ reach: open, },)) {
    // REFUSED WITHOUT A CALL, so a round counts the seat unreachable and
    // neither asks nor waits for it (class one hundred forty-eight).
    throw new NoProviderForModelError({
      modelId,
      reason: `every provider serving this model ran past its stream bound; held out for another ${String(heldMs,)}ms`,
    },);
  }
  return open;
}

//endregion Reach past holds
