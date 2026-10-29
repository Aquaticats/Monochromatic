import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { buildGrid, } from './align-headings-grid.ts';
import {
  type OptimalPaths,
  scanOptimalPaths,
} from './align-headings-optimal.ts';
import { tableCell, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Forced heading alignment
// An aligner that can REFUSE. The shipped scorer cannot: a pairing scores
// `diagonal + headingAffinity`, which is never below zero, while a gap costs
// `GAP_PENALTY` per side, so pairing two headings that share nothing scores 0
// against -0.70 for leaving both unpaired and the maximum always prefers the
// unsupported pairing. Every gap it emits is the forced surplus of a length
// difference, never a judgement that two headings do not correspond.
//
// This one scores LEXICOGRAPHICALLY: trusted anchors first, then fewest gaps,
// then soft affinity. And it emits a pairing only when that pairing lies on
// EVERY optimal path, which is what makes "I cannot tell" expressible. Anything
// else comes back ambiguous, and an ambiguous section gets no critic work, per
// `doc/decision/translation-repair-unpairable-section.md`.
//
// Measured against production over 92 entries: 90 align identically, XingZ60
// keeps 12 of its 13 pairs and loses only the wrong one, and XIEPT2 refuses all
// 8 rather than pairing Chinese prose against bare English headings.

/**
 Why a heading ended up with no partner.

 `forced-gap` means no optimal path pairs it at all, so the other side simply
 has nothing for it. `ambiguous` means several optimal pairings exist and the
 aligner declines to guess, which is the outcome the shipped scorer cannot
 produce.

 `roster-unpaired` comes from somewhere else entirely: a model was shown both
 documents and did not name this section in its pairing. It is kept separate
 from the other two because those describe a scorer's table and this describes
 a reading, and on this corpus the two disagree about nearly every section.
 */
export type UnpairedReason =
  | 'forced-gap'
    | 'ambiguous'
    | 'roster-unpaired';

/**
 Where an untranslated section's rendering may be written, or why it may not.

 THREE ANSWERS RATHER THAN A NULLABLE INDEX, because "we may not insert this"
 has two causes wanting opposite remedies, and collapsing them loses the one
 that matters. A section that MAY PAIR is one an optimal alignment can match
 against existing translation, so inserting it risks writing the page's own
 content in twice. A section with SEVERAL BOUNDARIES is genuinely missing but
 could go in more than one place, so inserting it risks putting real content
 in the wrong section. The first is a duplication, the second a misfiling.
 */
export type InsertionAnchor =
  | {
    /**
     No optimal alignment pairs this section, and every optimal alignment
     skips it at the same place.
     */
    readonly kind: 'proven';

    /**
     Target unit the insertion goes before. Equal to the target length when
     the section belongs after everything the translation carries.
     */
    readonly beforeTargetIndex: number;
  }
    | {
    /**
     Some optimal alignment pairs this section with existing translation, so
     whatever it says may already be on the page.
     */
      readonly kind: 'may-pair';
    }
    | {
    /**
     Nothing pairs it, but optimal alignments disagree about where it sits.
     */
      readonly kind: 'several-boundaries';

    /**
     Places it could go, in ascending order, so a report can say how wide the
     disagreement is rather than only that there was one.
     */
      readonly boundaries: readonly number[];
    };

/**
 One decision about one heading.

 @example
 ```ts
 const step: ForcedAlignStep = { kind: 'paired', sourceIndex: 0, targetIndex: 0, affinity: 1, };
 ```
 */
export type ForcedAlignStep =
  | {
    /**
     Both sides correspond on every optimal path.
     */
    readonly kind: 'paired';

    /**
     Source unit index.
     */
    readonly sourceIndex: number;

    /**
     Target unit index.
     */
    readonly targetIndex: number;

    /**
     Affinity of the pairing.
     */
    readonly affinity: number;
  }
    | {
    /**
     Original carries a section the translation does not, or the aligner
     refuses to say which one it is.
     */
      readonly kind: 'source-only';

    /**
     Source unit index.
     */
      readonly sourceIndex: number;

    /**
     Whether nothing could pair, or too much could.
     */
      readonly reason: UnpairedReason;

    /**
     Where a translation of this section could be inserted, when that place is
     proven.
     */
      readonly anchor: InsertionAnchor;
    }
    | {
    /**
     Translation carries a section the original does not, or the aligner
     refuses to say which one it is.
     */
      readonly kind: 'target-only';

    /**
     Target unit index.
     */
      readonly targetIndex: number;

    /**
     Whether nothing could pair, or too much could.
     */
      readonly reason: UnpairedReason;
    };

/**
 Decides where an unpaired source section's rendering could be written.

 @param partners - target units this section pairs with on some optimal path

 @param gapColumns - target columns it is skipped at on some optimal path

 @returns Proven place, or which kind of uncertainty forbids one

 @example
 ```ts
 const anchor = anchorFor({ partners, gapColumns, },);
 ```
 */
function anchorFor(
  {
    partners,
    gapColumns,
  }: {
    readonly partners: ReadonlySet<number>;
    readonly gapColumns: ReadonlySet<number>;
  },
): InsertionAnchor {
  if (partners.size > 0)
    return { kind: 'may-pair', };

  /**
   Places this section could sit, in document order.
   */
  const boundaries = [...gapColumns,]
    .toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },);

  /**
   The single place, when the optimal alignments agree on one.
   */
  const [only,] = boundaries;
  if ((boundaries.length === 1) && (only !== undefined))
    return {
      kind: 'proven',
      beforeTargetIndex: only,
    };

  return {
    kind: 'several-boundaries',
    boundaries,
  };
}

/**
 Aligns two heading sequences, emitting a pairing only when it is forced.

 A pairing is forced when it lies on EVERY optimal path. Anything else is
 reported unpaired with `ambiguous`, which is the outcome that lets a caller
 skip a section rather than guess at it.

 Exported so the walk can be read directly on corpus pages, which is the only
 way to find out which refusal real pages actually produce.

 @internal

 @param sourceHeadings - original-side unit labels in document order

 @param targetHeadings - translation-side unit labels in document order

 @returns One step per source unit, then the unpaired target units

 @example
 ```ts
 const steps = alignHeadingsForced({ sourceHeadings, targetHeadings, },);
 ```
 */
export function alignHeadingsForced(
  {
    sourceHeadings,
    targetHeadings,
  }: {
    readonly sourceHeadings: readonly string[];
    readonly targetHeadings: readonly string[];
  },
): readonly ForcedAlignStep[] {
  /**
   Source length.
   */
  const rows = sourceHeadings.length;

  /**
   Target length.
   */
  const columns = targetHeadings.length;

  /**
   Affinity and trust over every pairing.
   */
  const grid = buildGrid({
    sourceHeadings,
    targetHeadings,
  },);

  /**
   What every optimal alignment does with each unit.
   */
  const paths: OptimalPaths = scanOptimalPaths({
    sourceHeadings,
    targetHeadings,
  },);

  /**
   Target units each source unit pairs with on SOME optimal path.
   */
  const {
    partnersOfSource,
    partnersOfTarget,
    sourceGapColumns,
    targetCanGap,
  } = paths;

  /**
   Decisions for every source unit, then the target units left over.
   */
  const steps: ForcedAlignStep[] = [];

  /**
   Target units claimed by a forced pairing.
   */
  const claimed = new Set<number>();

  // The scan holds one partner set and one gap set per source unit, and one
  // partner set and one gap flag per target unit, so every read of them by a
  // unit's index finds its entry.
  for (let row = 0; row < rows; row += 1) {
    /**
     Targets this source unit could pair with optimally.
     */
    const partners = nonNullishOrThrow(partnersOfSource[row],);

    /**
     Target columns at which this source unit could go unpaired optimally.
     */
    const gapColumns = nonNullishOrThrow(sourceGapColumns[row],);

    /**
     The single partner, when there is exactly one and no gap competes.
     */
    const only = ((partners.size === 1) && (gapColumns.size === 0))
      ? [...partners,][0]
      : undefined;

    if (only !== undefined) {
      /**
       How many sources the single partner could pair with optimally.
       */
      const { size: partnerSources, } = nonNullishOrThrow(partnersOfTarget[only],);
      if ((partnerSources === 1) && (!nonNullishOrThrow(targetCanGap[only],))) {
        steps.push({
          kind: 'paired',
          sourceIndex: row,
          targetIndex: only,
          affinity: tableCell({
            table: grid.affinity,
            row,
            column: only,
          },),
        },);
        claimed.add(only,);
        continue;
      }
    }

    steps.push({
      kind: 'source-only',
      sourceIndex: row,
      reason: (partners.size === 0) ? 'forced-gap' : 'ambiguous',
      anchor: anchorFor({
        partners,
        gapColumns,
      },),
    },);
  }

  for (let column = 0; column < columns; column += 1) {
    if (claimed.has(column,))
      continue;
    /**
     How many sources this target could pair with optimally.
     */
    const { size: partnerSources, } = nonNullishOrThrow(partnersOfTarget[column],);
    steps.push({
      kind: 'target-only',
      targetIndex: column,
      reason: (partnerSources === 0) ? 'forced-gap' : 'ambiguous',
    },);
  }

  return steps;
}

//endregion Forced heading alignment
