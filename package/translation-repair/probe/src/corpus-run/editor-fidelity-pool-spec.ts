import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { DamageAttempt, } from '@monochromatic-dev/translation-repair-repair/ts';
import type { EditorDamageKind, } from '../editor-fidelity-trials.ts';
import type { FidelityReferenceSpec, } from '../fidelity-reference-model.ts';
import type { Section, } from './editor-fidelity-pool-sections.ts';

//region Editor fidelity pool specification
// THE SPECIFICATION OF ONE DRAWN DAMAGE: ids, ranges and hashes of the section, its donor and the damaged twin, and no
// passage of either page, since the corpus is unlicensed and the manifest commits only what can be checked against it.

/**
 Builds the specification a drawn damage is committed as.

 @param entryId - the entry

 @param section - the section the damage is in

 @param kind - the damage

 @param damage - the twin built

 @param donor - the section the donor sentence came from

 @param corpusSha - commit the pages were read at

 @param reviewedOn - date of the freeze

 @returns The specification, holding the one damage

 @example
 ```ts
 const spec = specForDamage({ entryId, section, kind, damage, donor, corpusSha, reviewedOn, },);
 ```
 */
export function specForDamage(
  {
    entryId,
    section,
    kind,
    damage,
    donor,
    corpusSha,
    reviewedOn,
  }: {
    readonly entryId: string;
    readonly section: Section;
    readonly kind: EditorDamageKind;
    readonly damage: Extract<DamageAttempt, { readonly kind: 'damaged'; }>;
    readonly donor: Section;
    readonly corpusSha: string;
    readonly reviewedOn: string;
  },
): FidelityReferenceSpec {
  return {
    id: `${entryId}-s${String(section.index,)}`,
    entryId,
    corpusSha,
    source: {
      ...section.source,
      hash: hashContent({ content: section.sourceRaw, },),
    },
    archive: {
      ...section.archive,
      hash: hashContent({ content: section.archiveRaw, },),
    },
    referenceHash: hashContent({ content: section.referenceText, },),
    referenceChars: section.referenceText
      .length,
    edits: [],
    donor: {
      ...donor.archive,
      hash: hashContent({ content: donor.archiveRaw, },),
    },
    damages: [
      {
        kind,
        hash: hashContent({ content: damage.damagedText, },),
        changedChars: damage.changedChars,
      },
    ],
    reviewedOn,
  };
}

/**
 Joins the two specifications of one section into one that holds both damages; at most one insertion and one alteration
 are drawn, so only the two can share a section.

 @param specs - the specifications drawn, the insertion's first

 @returns One specification when both were drawn in the same section, else the specifications as they were

 @example
 ```ts
 const joined = joinedBySection({ specs, },);
 ```
 */
export function joinedBySection({ specs, }: { readonly specs: readonly FidelityReferenceSpec[]; },): readonly FidelityReferenceSpec[] {
  /**
   The two specifications, when there are two.
   */
  const [first, second,] = specs;
  if ((first === undefined) || (second === undefined)
    || (first.id !== second.id))
    return specs;
  return [
    {
      ...first,
      damages: [
        ...first.damages,
        ...second.damages,
      ],
    },
  ];
}

//endregion Editor fidelity pool specification
