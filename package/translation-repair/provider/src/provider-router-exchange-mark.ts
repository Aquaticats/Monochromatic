import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { markedAfterExchange, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { NOBODY_REFUSED, } from './budget-hold-wait.ts';
import type { ChatTextRequest, } from './chat-contract.ts';

//region Provider router exchange mark
// WHETHER THE ROUTER'S NO-PROVIDER REFUSAL CAME AFTER A CALL (B1230). The router raises `NoProviderForModelError` before
// any call (a hold, a missing credential, no budget) and, once calls of the same request were cut at the stream bound or
// refused, after them. A re-ask gives a refused model's claim back only for the first kind, since the second kind
// asked it; this wrapper marks the second where the router knows a call was made.

/**
 Chooses the provider for a call, marking a no-provider refusal it raises when a call of the same request was made.

 @param exchanged - whether a call of this request was made before the decision

 @param choose - the router's decision of which provider takes the call

 @param request - call being routed

 @param refused - provider that has just refused us, or nobody

 @returns Provider to ask

 @throws What the decision raises, a no-provider refusal marked as raised after an exchange when one was made

 @example
 ```ts
 const provider = await markingAfterExchange({ exchanged: attempt > 0, choose, request, refused, },);
 ```
 */
export async function markingAfterExchange(
  {
    exchanged,
    choose,
    request,
    refused,
  }: {
    readonly exchanged: boolean;
    readonly choose: (args: {
      readonly request: ForeignBorrowed<ChatTextRequest>;
      readonly refused: ProviderName | typeof NOBODY_REFUSED;
    },) => Promise<ProviderName>;
    readonly request: ForeignBorrowed<ChatTextRequest>;
    readonly refused: ProviderName | typeof NOBODY_REFUSED;
  },
): Promise<ProviderName> {
  try {
    return await choose({
      request,
      refused,
    },);
  }
  catch (error) {
    throw exchanged ? markedAfterExchange({ error, },) : error;
  }
}

//endregion Provider router exchange mark
