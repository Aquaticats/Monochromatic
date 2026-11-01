import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ProviderChoice, } from './budget-routing.ts';
import type { ChatTextRequest, } from './chat-contract.ts';
import { refuseDarkModel, } from './model-dark-reason.ts';
import { holdsKeepingRequestOff, } from './reask-withheld.ts';

//region Provider router held choice
// A HELD MODEL IS NOT SENT TO THE PROVIDER ITS CARD HOLDS IT OFF, FIRST ASK INCLUDED (ledger B1365, the owner's "Honour
// holds (Recommended)"). The run withholds a `hyper-slow` model from a seat while Hyper is the provider that would
// serve it, but the policy of `budget-routing.ts` is a walk down the order that knows no holds, so a model seated while
// Synthetic was wet and asked after Synthetic ran dry went to Hyper all the same. This reads the policy's choice against
// the same function the run's seating and the re-ask read (`holdsKeepingRequestOff`) and refuses it before the slot is
// taken, the way a model with no provider is refused: one seat lost, which the stage counts unreachable and never
// stops a slice, an entry or the run on.
//
// SPLIT FROM `provider-router.ts` at its line budget, together with the refusal of a model no provider can take, since
// both are the answer to one question: may the call go where the policy chose.

/**
 The provider the policy chose for a request, or the refusal of a lost seat.

 A model no provider can take is refused, and so is a request whose seat the model's card holds off the provider chosen,
 both with the refusal `refuseDarkModel` throws, raised before any call; the router marks it as raised after a call
 where an earlier attempt of the same request was made (`markingAfterExchange`). A request that names no seat, and a
 measurement, are not held on the first ask.

 @param request - request being routed, read for its model and its seat

 @param choice - what the policy decided for it

 @param l - logger the line goes to

 @param warnDue - whether the line is due for this model, asked only when the request is refused, so the warn is written
 once per model per cooldown

 @returns Provider to ask

 @throws What `refuseDarkModel` throws, the no-provider refusal of a lost seat, when the policy found no provider or a
 hold of the model's card that the seat's role honours names the one it chose

 @example
 ```ts
 const provider = providerChosenOrRefused({ request, choice, l, warnDue: darkWarnDue, },);
 ```
 */
export function providerChosenOrRefused(
  {
    request,
    choice,
    l,
    warnDue,
  }: {
    readonly request: ForeignBorrowed<ChatTextRequest>;
    readonly choice: ProviderChoice;
    readonly l: Logger;
    readonly warnDue: (args: { readonly modelId: string; },) => boolean;
  },
): ProviderName {
  // A LOST SEAT, NOT THE RUN'S END: this model alone has no provider, and the log line names the model and how each
  // provider read so a read of the run can tell the two apart.
  if (choice.kind === 'unreachable') {
    refuseDarkModel({
      modelId: request.modelId,
      reason: choice.reason,
      l,
      warn: warnDue({ modelId: request.modelId, },),
    },);
  }
  /**
   Holds of the model's card that keep it off that provider for the seat's role.
   */
  const holds = holdsKeepingRequestOff({
    request,
    provider: choice.kind,
    whenNoSeat: 'serve',
  },);
  if (holds.length === 0)
    return choice.kind;
  /**
   Seat the request names, which a request is held for only when it names one.
   */
  const reaskSeat = nonNullishOrThrow(request.reaskSeat,);
  return refuseDarkModel({
    modelId: request.modelId,
    reason: `${choice.kind} is where the policy sends it, and its card holds it off ${choice.kind} for the `
      + `${reaskSeat.role} role (${holds.join(' and ',)}); the seat is lost and not asked there`,
    l,
    warn: warnDue({ modelId: request.modelId, },),
  },);
}

//endregion Provider router held choice
