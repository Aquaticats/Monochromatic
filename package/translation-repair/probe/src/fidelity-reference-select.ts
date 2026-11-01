import { FidelityReferenceError, } from './fidelity-reference-error.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Reviewed-manifest selection
// Filtering is a metadata operation; it never searches archive prose for new gold references.

/**
 Selects only named reviewed references and refuses ambiguous or empty requests.

 @param specs - caller-owned reviewed manifest

 @param onlyEntryIds - reviewed entry names, or empty for the whole manifest

 @returns Owned reference specifications in manifest order

 @throws {@link FidelityReferenceError} for duplicates, missing references or unknown entry filters

 @example
 ```ts
 const selected = selectReviewedFidelitySpecs({ specs, onlyEntryIds });
 ```
 */
export function selectReviewedFidelitySpecs({
  specs,
  onlyEntryIds,
}: {
  readonly specs: readonly FidelityReferenceSpec[];
  readonly onlyEntryIds: readonly string[];
},): readonly FidelityReferenceSpec[] {
  /**
   Own selectors before any asynchronous file acquisition begins.
   */
  const checked = structuredClone(specs,);
  /**
   Duplicate reference IDs would conflate distinct observations.
   */
  const identities = new Set(checked.map(function identity(spec,): string {
    return spec.id;
  },),);
  if ((checked.length === 0) || (identities.size !== checked.length))
    throw new FidelityReferenceError({
      referenceId: 'manifest',
      operation: 'request',
    },);
  /**
   Entry filters are restricted to the reviewed population.
   */
  const entries = new Set(checked.map(function entry(spec,): string {
    return spec.entryId;
  },),);
  for (const entryId of onlyEntryIds) {
    if (!entries.has(entryId,))
      throw new FidelityReferenceError({
        referenceId: entryId,
        operation: 'request',
      },);
  }
  return checked.filter(function selectedEntry(spec,): boolean {
    return (onlyEntryIds.length === 0) || onlyEntryIds.includes(spec.entryId,);
  },);
}

//endregion Reviewed-manifest selection
