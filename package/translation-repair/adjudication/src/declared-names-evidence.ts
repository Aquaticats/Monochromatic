import type { SelectEvidence, } from './candidate-select-wire.ts';
import { DECLARED_NAMES_HEADING, } from './declared-identity-rule.ts';

//region Declared names evidence
// THE DECLARED NAMES AS ONE SELECTION SLATE READS THEM (ledger B28). A slate
// carries the house rules, which point to a DECLARED NAMES block, but not the
// declared-identity rules the review sheets carry, so the entry's label states
// the one rule a slate needs. The repair slates wrote this entry inline; the
// refine slates had none although their refiner reads the names, and the
// archive correction slate gave the bare heading under criteria that never
// mention names. All three now read this one entry.

/**
 Label the declared names carry on a selection slate: the heading the house
 rules name, and what a slate does with a name it lists.
 */
export const DECLARED_NAMES_EVIDENCE_LABEL: string = `${DECLARED_NAMES_HEADING} from the documents' own front matter: `
  + 'a declared name or handle used to refer to its person or place is correct, never an addition or a wrong term';

/**
 The declared names as one labelled slate entry, none when the page declares
 nothing.

 @param identityContext - declared names and handles from both front matters

 @returns One entry, or none

 @example
 ```ts
 const evidence = [...declaredNamesEvidence({ identityContext, },), ...citedReferenceEvidence({ referenceContext, },),];
 ```
 */
export function declaredNamesEvidence(
  { identityContext, }: { readonly identityContext?: string; },
): readonly SelectEvidence[] {
  return ((identityContext === undefined) || (identityContext === ''))
    ? []
    : [{
      label: DECLARED_NAMES_EVIDENCE_LABEL,
      text: identityContext,
    },];
}

//endregion Declared names evidence
