import { reachOf, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type ProviderRecord,
  providerRecord,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Writing providers
// WHICH PROVIDERS KEEP A RUN GOING WHILE THEY READ WET: those that serve a model seated in a writing bench. The writing
// benches (`WRITING_BENCHES` in `run-seats-floor.ts`: editors, refiners, translators) are held to a floor of two seats
// a wet provider serves, because a bench below it writes nothing a judge can choose between. A provider that serves
// only seats that judge or read cannot lift a writing bench to its floor, so its wet meter must not hide a fleet that
// can write nothing. Xiaomi, whose one seat is a checker, is that provider in the roster of 2026-10-22.
//
// DERIVED FROM THE ROSTER THROUGH `reachOf`, NOT FROM A LIST OF PROVIDER NAMES, so a roster change that seats a writer
// on a provider counts that provider with no edit here. THE LEAD'S RULING OF 2026-10-22 ON THE REVIEW'S M3, open to the
// owner's veto.

/**
 The providers that serve at least one of the given models.

 @param modelIds - models seated in the writing benches

 @returns Per provider, whether it serves any of them

 @example
 ```ts
 writingProvidersOf({ modelIds: ['xiaomi/mimo-v2.6-flash',], },).xiaomi; // => true
 ```
 */
export function writingProvidersOf(
  { modelIds, }: { readonly modelIds: readonly RosterModelId[]; },
): ProviderRecord<boolean> {
  /**
   Each model's reach.
   */
  const reaches = modelIds.map(function reachOfModel(modelId,): ProviderRecord<boolean> {
    return reachOf({ modelId, },);
  },);
  return providerRecord({
    of: function servesAny(provider,): boolean {
      return reaches.some(function serves(reach,): boolean {
        return reach[provider];
      },);
    },
  },);
}

//endregion Writing providers
