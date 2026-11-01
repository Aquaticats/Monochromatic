import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  ChatTextRequest,
  ExcludedProviders,
} from './chat-contract.ts';

//region Provider exclusion
// THE PROVIDERS A RE-ASK MUST STAY OFF (ledger B1410). A request hinted off one provider that is answered by another and
// then re-asked was re-asked with the hint replaced by the provider just tried, so the provider the caller excluded could
// answer it. The re-ask carries what the request carried and adds the provider just tried.

/**
 The providers a re-ask of a request must stay off: those the request carried, then the provider that just answered.

 @param request - request that was answered, whose hint is kept

 @param provider - provider that answered it

 @returns Request's providers and then this one, each once

 @example
 ```ts
 const excluded = excludingAlso({ request, provider: 'hyper', },);
 ```
 */
export function excludingAlso(
  {
    request,
    provider,
  }: {
    readonly request: ForeignBorrowed<ChatTextRequest>;
    readonly provider: ProviderName;
  },
): ExcludedProviders {
  /**
   Providers the request already excludes, when it carries a hint.
   */
  const { otherThan: carried, } = request;
  if (carried === undefined)
    return [provider,];
  return carried.includes(provider,) ? carried : [
    ...carried,
    provider,
  ];
}

//endregion Provider exclusion
