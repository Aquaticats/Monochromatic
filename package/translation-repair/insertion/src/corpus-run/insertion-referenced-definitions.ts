import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type FootnoteMention,
  footnoteMentions,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { InsertionCoverageRow, } from './insertion-coverage-model.ts';

//region Insertion referenced definitions
// A DEFINITION SHIPS WITH ITS MARKER. The whole-page shortfall budget is
// spent in document order, and footnote definitions stand last on a page.
// On XingZ608 the budget ran out 80 code points before the definitions
// (10,714 of 10,794 spent on the passages before them), the definitions
// were refused, and the translate assembly then withdrew every carrier of a
// marker with no definition, one of them holding the page's only rendering
// of a source link. A marker the page will carry with no definition under
// its label is the page's own graph saying the definition is absent: the
// same kind of independent signal as a missing destination, which the
// admission already reads past the budget.

/**
 Finding prefix an admitted definition is recorded under.

 @example
 ```ts
 findings.some((finding) => finding.startsWith(DEFINITION_ADMITTED_FINDING));
 ```
 */
export const DEFINITION_ADMITTED_FINDING: string = 'insertion-definition-admitted';

/**
 Who references a label: a slice by index, or the standing page.
 */
type Referrer =
  | {
    readonly kind: 'slice';
    readonly sliceIndex: number;
  }
    | { readonly kind: 'page'; };

/**
 An unresolved row that defines a label the shipping page references and
 does not define, with that label and who references it.
 */
type FollowingDefinition = {
  /**
   Row admitted with its marker.
   */
  readonly row: InsertionCoverageRow;

  /**
   Label it defines, with its convention.
   */
  readonly label: string;

  /**
   Who references the label.
   */
  readonly referrer: Referrer;
};

/**
 Labels a text mentions in one role: convention and identifier together,
 which is what a definition must match.

 @param text - slice source or the standing page

 @param role - `reference` or `definition`

 @returns Labels in that role, in the order the text first mentions each

 @example
 ```ts
 const defined = labelsOf({ text, role: 'definition', },);
 ```
 */
function labelsOf(
  {
    text,
    role,
  }: {
    readonly text: string;
    readonly role: FootnoteMention['role'];
  },
): readonly string[] {
  return [
    ...new Set(footnoteMentions({ text, },)
      .filter(function inRole(mention,): boolean {
        return mention.role === role;
      },)
      .map(function toLabel(mention,): string {
        return `${mention.convention} ${mention.identifier}`;
      },),),
  ];
}

/**
 Names a referrer for a finding.

 @param referrer - who references the label

 @returns Clause naming the slice or the page

 @example
 ```ts
 const clause = describeReferrer({ referrer: { kind: 'page', }, },);
 ```
 */
function describeReferrer({ referrer, }: { readonly referrer: Referrer; },): string {
  if (referrer.kind === 'page')
    return 'the page';
  return `slice ${String(referrer.sliceIndex,)}`;
}

/**
 Admits every unresolved definition whose label the shipping page references and does not define.

 @param slices - prepared slices, whose source the admitted positions are read off

 @param positions - positions admitted on their own evidence

 @param unresolvedRows - rows neither admitted nor proven carried

 @param targetText - standing page, whose markers and definitions count

 @returns Positions and unresolved rows after the definitions follow, with a finding per definition admitted

 @example
 ```ts
 const definitions = admitReferencedDefinitions({ slices, positions, unresolvedRows, targetText, },);
 ```
 */
export function admitReferencedDefinitions(
  {
    slices,
    positions,
    unresolvedRows,
    targetText,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly positions: ReadonlySet<number>;
    readonly unresolvedRows: readonly InsertionCoverageRow[];
    readonly targetText: string;
  },
): {
  readonly positions: ReadonlySet<number>;
  readonly unresolvedRows: readonly InsertionCoverageRow[];
  readonly findings: readonly string[];
} {
  /**
   Slices admitted on their own evidence, in position order.
   */
  const admittedSlices = [...positions,]
    .toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },)
    .map(function toSlice(position,): ChunkPair {
      // Admitted positions index these slices: every admission reads them
      // off the same preparation.
      return nonNullishOrThrow(slices[position],);
    },);
  /**
   Who references each label the shipping page will carry, the page first,
   then admitted slices in order, first referrer kept.
   */
  const referrers = new Map<string, Referrer>();
  for (
    const label of labelsOf({
      text: targetText,
      role: 'reference',
    },)
  ) {
    if (!referrers.has(label,))
      referrers.set(
        label,
        { kind: 'page', },
      );
  }
  for (const slice of admittedSlices) {
    /**
     Index the slice's finding names it by.
     */
    const { sliceIndex, } = slice.target;
    for (
      const label of labelsOf({
        text: slice.source
          .text,
        role: 'reference',
      },)
    ) {
      if (!referrers.has(label,))
        referrers.set(
          label,
          {
            kind: 'slice',
            sliceIndex,
          },
        );
    }
  }
  /**
   Labels the shipping page already defines.
   */
  const defined = new Set([
    targetText,
    ...admittedSlices.map(function toSource(slice,): string {
      return slice.source
        .text;
    },),
  ].flatMap(function definitionsIn(text,): readonly string[] {
    return labelsOf({
      text,
      role: 'definition',
    },);
  },),);
  /**
   Unresolved rows defining a referenced, undefined label, each with that label and its referrer.
   */
  const following = unresolvedRows.flatMap(function definitionRows(row,): readonly FollowingDefinition[] {
    /**
     Labels this row defines that the page references and lacks, each with
     its referrer, read in one lookup; the first is the one admitted.
     */
    const wanted = labelsOf({
      text: row.sourceText,
      role: 'definition',
    },)
      .flatMap(function referencedAndMissing(label,): readonly FollowingDefinition[] {
        /**
         Who references the label, absent where nobody does.
         */
        const referrer = referrers.get(label,);
        return ((referrer === undefined) || defined.has(label,))
          ? []
          : [{
            row,
            label,
            referrer,
          },];
      },);
    return wanted.slice(
      0,
      1,
    );
  },);
  /**
   Positions the definitions add.
   */
  const added = new Set(following.map(function toPosition(entry,): number {
    return entry.row
      .position;
  },),);
  return {
    positions: new Set([
      ...positions,
      ...added,
    ],),
    unresolvedRows: unresolvedRows.filter(function stillUnresolved(row,): boolean {
      return !added.has(row.position,);
    },),
    findings: following.map(function toFinding(entry,): string {
      /**
       Label with its convention, as the scanner keys it.
       */
      const { label, } = entry;
      /**
       Identifier without its convention, which is how a page reader names it.
       */
      const identifier = label.slice(label.indexOf(' ',) + 1,);
      /**
       Slice the definition belongs to.
       */
      const { sliceIndex, } = entry.row;
      return `${DEFINITION_ADMITTED_FINDING} (slice ${String(sliceIndex,)} defines ${identifier}, `
        + `referenced by ${describeReferrer({ referrer: entry.referrer, },)}: a marker's definition ships with it, `
        + 'and the page defines nothing under that label)';
    },),
  };
}

//endregion Insertion referenced definitions
