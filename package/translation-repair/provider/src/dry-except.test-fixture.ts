/**
 A meter reading in which only some providers are not dry, for the cases about the give-up decision.

 Fixtures are cat-themed invention; no meter is read.

 @module
 */

import {
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';

/**
 What a meter reading finds when only the named providers read wet.

 @param wet - providers that read wet

 @returns Dryness per provider, dry for every provider not named

 @example
 ```ts
 const dry = dryExcept({ wet: ['neuralwatt',], },);
 ```
 */
export function dryExcept({ wet, }: { readonly wet: readonly ProviderName[]; },): ProviderRecord<boolean> {
  return providerRecord({
    of: function isDry(provider,): boolean {
      return !wet.includes(provider,);
    },
  },);
}
