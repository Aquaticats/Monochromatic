import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { markedAfterExchange, } from '@monochromatic-dev/translation-repair-markup/ts';

import type { ModelReach, } from './budget-routing.ts';
import type { ChatTextRequest, } from './chat-contract.ts';
import {
  type CredentialHolds,
  reachPastCredentialHolds,
} from './credential-hold.ts';
import {
  reachOfRequest,
  reachPastHoldsOrRefusal,
} from './reach-past-holds.ts';
import type { StreamBoundHolds, } from './stream-bound-hold.ts';

//region Provider router reach
// THE TWO READINGS OF WHICH PROVIDERS MAY TAKE A CALL, split from `provider-router.ts` at its line cap when the router
// came to mark a refusal raised after an exchange (B1230). Both read the holds the router keeps and nothing else.

/**
 The two readings of the providers that may take a call.

 @example
 ```ts
 const { reachFor, reachPastBoundHoldsOrRefusal, } = createReachReaders({ credentialHolds, boundHolds, },);
 ```
 */
export type ReachReaders = {
  /**
   Providers that can serve one call, narrowed to vision where it carries a picture, less any whose credential was
   refused.
   */
  readonly reachFor: (args: { readonly request: ForeignBorrowed<ChatTextRequest>; },) => ModelReach;

  /**
   Providers that may take one call once the credential holds and the stream bound holds are taken out, refusing where
   the holds leave nobody: asked before each attempt, and after the last one a call has, where `afterExchange` marks
   its refusal as raised after a call (B1230).
   */
  readonly reachPastBoundHoldsOrRefusal: (
    args: {
      readonly request: ForeignBorrowed<ChatTextRequest>;
      readonly afterExchange?: boolean;
    },
  ) => ModelReach;
};

/**
 Builds the two readings over the holds a router keeps.

 @param credentialHolds - providers whose credential was refused this run

 @param boundHolds - providers held out for one model after a call ran past its stream bound

 @returns Two readings

 @example
 ```ts
 const { reachFor, } = createReachReaders({ credentialHolds, boundHolds, },);
 ```
 */
export function createReachReaders(
  {
    credentialHolds,
    boundHolds,
  }: {
    readonly credentialHolds: CredentialHolds;
    readonly boundHolds: StreamBoundHolds;
  },
): ReachReaders {
  return {
    reachFor: function reachFor({ request, },): ModelReach {
      /**
       Reach of the call less the providers whose credential was refused.
       */
      const { reach, } = reachPastCredentialHolds({
        reach: reachOfRequest({ request, },),
        holds: credentialHolds,
      },);
      return reach;
    },
    reachPastBoundHoldsOrRefusal: function reachPastBoundHoldsOrRefusal(
      {
        request,
        afterExchange = false,
      },
    ): ModelReach {
      try {
        return reachPastHoldsOrRefusal({
          reach: reachOfRequest({ request, },),
          modelId: request.modelId,
          credentialHolds,
          boundHolds,
        },);
      }
      catch (error) {
        throw afterExchange ? markedAfterExchange({ error, },) : error;
      }
    },
  };
}

//endregion Provider router reach
