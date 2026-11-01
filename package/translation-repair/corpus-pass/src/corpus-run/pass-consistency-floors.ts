import {
  type DeclaredNamePair,
  declaredNameRefusalFinding,
  findDroppedDeclaredNames,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { assessSliceAlignment, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import {
  type DisputedWording,
  floorReach,
  requireComparedVerdict,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Pass consistency floors
// Every floor a changed slice must pass before the pass lets it stand, run on
// the bytes that would ship, with the slice's current wording as the
// incumbent: the polish round's convention (`consolidation-polish-round.ts`),
// which measures a rewrite against the wording already approved for the slice
// and not against the archive. Where the archive stands, the two are one text.
//
// THE FLOORS, in order: the ground the floor can read, the deterministic
// publication rule (`validateTranslatedSlice`, which also refuses a lost
// quoted passage, as the floors test pins), and the two guards the translate
// lane puts on a replacement of the archive's wording (`translate-slice.ts`)
// that the rule does not hold: the size the original can account for and the
// declared names the incumbent carries.
//
// MARKDOWN ONLY, because a repeat group holds no front matter
// (`repeat-groups.ts` leaves it out, so no place of a group lies in a front
// matter slice) and so no slice the pass changes has the YAML ground the
// translate lane reads differently.

/**
 What the floors made of a changed slice.

 @example
 ```ts
 const verdict: ConsistencyFloorVerdict = { kind: 'passes', };
 ```
 */
export type ConsistencyFloorVerdict =
  | {
    /**
     Every floor passed.
     */
    readonly kind: 'passes';
  }
    | {
    /**
     A floor refused the slice.
     */
      readonly kind: 'refused';

    /**
     What the floors found, one sentence each.
     */
      readonly findings: readonly string[];
    };

/**
 Runs every floor on a changed slice.

 @param sliceIndex - slice the findings name

 @param sourceText - the slice's original

 @param currentText - the wording the slice carries now, which the candidate
 would replace

 @param candidateText - the slice with the pass's replacements made

 @param lineStructured - whether the line-structure rule governs the slice

 @param declared - name pairs the front matter declares

 @param declaredNames - declared name forms the incumbent must keep

 @param declaredSubjectForms - the declared forms among them that name the subject, which may stand in for one another

 @param disputedWordings - wordings a disputed slice refuses

 @returns That every floor passed, or what the floors found

 @throws {@link FloorGroundDisagreementError} when the publication rule says it
 compared nothing on ground the reach read as comparable

 @example
 ```ts
 const verdict = consistencyFloorVerdict({ sliceIndex: 3, sourceText, currentText, candidateText, lineStructured: false, declared: [], declaredNames: [], disputedWordings: [], },);
 ```
 */
export function consistencyFloorVerdict(
  {
    sliceIndex,
    sourceText,
    currentText,
    candidateText,
    lineStructured,
    declared,
    declaredNames,
    declaredSubjectForms,
    disputedWordings,
  }: {
    readonly sliceIndex: number;
    readonly sourceText: string;
    readonly currentText: string;
    readonly candidateText: string;
    readonly lineStructured: boolean;
    readonly declared: readonly DeclaredNamePair[];
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly disputedWordings: readonly DisputedWording[];
  },
): ConsistencyFloorVerdict {
  /**
   Whether the publication rule can compare anything here.
   */
  const reach = floorReach({
    sourceText,
    pageText: currentText,
  },);
  if (reach.kind === 'blind') {
    return {
      kind: 'refused',
      findings: [`the floor can compare nothing here (${reach.detail})`,],
    };
  }
  /**
   The publication rule's verdict on the candidate, a pass or a refusal since
   the ground was read as comparable.
   */
  const validation = requireComparedVerdict({
    verdict: validateTranslatedSlice({
      sourceText,
      candidateText,
      pageText: currentText,
      lineStructured,
      declared,
      disputedWordings,
    },),
  },);
  if (validation.kind === 'invalid') {
    return {
      kind: 'refused',
      findings: validation.findings,
    };
  }
  /**
   Whether the slice's two sides can be the same passage.
   */
  const alignment = assessSliceAlignment({
    sourceText,
    incumbentText: currentText,
  },);
  if (alignment.kind === 'incumbent-dominates-source') {
    return {
      kind: 'refused',
      findings: [
        'the wording the slice carries is longer than its original can account for '
          + `(${String(alignment.incumbentCodePoints,)} against ${String(alignment.sourceCodePoints,)} code points), `
          + 'so no replacement is made',
      ],
    };
  }
  /**
   Declared names the slice carries now and the candidate does not.
   */
  const dropped = findDroppedDeclaredNames({
    forms: declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { subjectForms: declaredSubjectForms, }),
    baseText: currentText,
    candidateText,
  },);
  if (dropped.length > 0) {
    return {
      kind: 'refused',
      findings: [
        declaredNameRefusalFinding({
          sliceIndex,
          dropped,
        },),
      ],
    };
  }
  return { kind: 'passes', };
}

//endregion Pass consistency floors
