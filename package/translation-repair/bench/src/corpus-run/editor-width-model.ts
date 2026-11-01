import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Editor width model
// How a width comparison is read, kept apart from the calls that gather it so
// the reading can be tested without spending quota.
//
// The question this serves: does seating more EDITORS produce a
// better repair, holding the judging panel fixed. Two things make that harder
// to read than it looks.
//
// First, most slices will not move at all. The same strong editor tends to win
// both slates, and a width-three winner often survives the wider slate intact.
// A comparison that judged those against themselves would spend calls to learn
// nothing and would dilute whatever signal the moved slices carry, so they are
// classified out before any head-to-head runs.
//
// Second, a winner that changed is not yet a winner that improved. The lane
// disagrees with itself run to run, so the fraction of slices where the
// shipped text moved means nothing until the same arm RUN TWICE says how often
// it moves with no width change at all. That repeat re-produces as well as
// re-judges, because the width arms differ in production and a band covering
// only judge churn would credit ordinary editor variation to the extra seats.

/**
 Fate of one slice under the two widths, named exhaustively so a slice that
 fits none of them is a defect rather than a silent omission.

 `nothing-shipped` covers both slates declining, which says nothing about
 width. `same-text` is the common case and is the answer for that slice:
 widening changed nothing here. Only `differs` earns a head-to-head.
 */
export type WidthComparison = 'nothing-shipped' | 'same-text' | 'differs';

/**
 Which winner the panel preferred when both were put on one slate.

 `position-decided` is a REAL OUTCOME rather than a failure. The two orders
 disagreed, so the slate position decided the ballot rather than the text, and
 counting that as a win for whichever order ran first is how a position bias
 gets reported as a quality difference.
 */
export type HeadToHeadVerdict = 'wide-wins' | 'narrow-wins' | 'position-decided' | 'tied';

/**
 Which half of the sample a run spends.

 The sample is split rather than redrawn so a result near its own null band
 has a second, untouched reading available. That only means anything if the
 second half can actually be run, which is what this selects.
 */
export type WidthDraw = 'a' | 'b';

/**
 Which arm produced a text, so a verdict can name it without carrying it.
 */
export type WidthArm = 'narrow' | 'wide';

/**
 Everything one slice contributed, with no passage text in it.

 TEXTS ARE DELIBERATELY ABSENT. This record is what the report prints and what
 a later session reads, and the corpus is unlicensed, so the comparison keeps
 hashes and lengths rather than the words they stand for.
 */
export type WidthRow = {
  /**
   Entry the slice came from.
   */
  readonly entryId: string;

  /**
   Slice within that entry.
   */
  readonly sliceIndex: number;

  /**
   Accepted issues the editors were given, which is the work available to do.
   */
  readonly acceptedIssues: number;

  /**
   How the two widths compared.
   */
  readonly comparison: WidthComparison;

  /**
   Editors heard at each width, out of those seated. A width that seated six
   and heard three did not run at six.
   */
  readonly heardNarrow: number;

  /**
   {@link WidthRow.heardNarrow} for the wide arm.
   */
  readonly heardWide: number;

  /**
   Whether each arm shipped a repair at all.

   SEPARATE FROM THE COMPARISON because the wide arm fields twice the
   candidates against the same selection minimum, so it can split its own vote
   and settle on the incumbent where the narrow arm settled on a repair. That
   shows up as `differs` exactly like a better rewrite does, and the two are
   opposite answers to the width question: one says widening improved the repair, the
   other says widening suppressed it.
   */
  readonly narrowShipped: boolean;

  /**
   {@link WidthRow.narrowShipped} for the wide arm.
   */
  readonly wideShipped: boolean;

  /**
   Whether the narrow arm, run a second time end to end, shipped the same
   text. This is the null band: a flip here is the lane disagreeing with
   itself, with no width change behind it.
   */
  readonly narrowRepeatAgreed: boolean;

  /**
   Head-to-head reading, absent where the comparison earned none.
   */
  readonly verdict: HeadToHeadVerdict | 'not-run';

  /**
   Ballots that named a candidate in the head-to-head, summed over both
   orders. A verdict carried by two of twelve must be visible as such.
   */
  readonly usableBallots: number;

  /**
   Models that wrote the winning text at each arm, for a roster-level reading
   of who the extra seats actually bought.
   */
  readonly narrowProducers: readonly RosterModelId[];

  /**
   {@link WidthRow.narrowProducers} for the wide arm.
   */
  readonly wideProducers: readonly RosterModelId[];
};

/**
 Reads the two arms into one classification.

 @param narrowText - text the narrow arm shipped, blank when it shipped none

 @param wideText - text the wide arm shipped, blank when it shipped none

 @returns Which of the three cases this slice is

 @example
 ```ts
 const comparison = classifyWidths({ narrowText, wideText, },);
 ```
 */
export function classifyWidths(
  {
    narrowText,
    wideText,
  }: {
    readonly narrowText: string;
    readonly wideText: string;
  },
): WidthComparison {
  if ((narrowText === '') && (wideText === ''))
    return 'nothing-shipped';

  // One arm shipping and the other not IS a difference, and the interesting
  // kind: the extra seats either found a repair the narrow roster missed or
  // spread the ballots thin enough to decline one it would have made.
  if (narrowText === wideText)
    return 'same-text';

  return 'differs';
}

/**
 Reads two ordered head-to-head rounds into one verdict.

 ORDERS MUST AGREE. Each round is the same pair of texts judged by the same
 panel, differing only in which sat first, so a disagreement is the position
 talking. Reporting the first order alone would launder that into a result.

 @param firstOrderWinner - arm the panel preferred with the narrow text first

 @param secondOrderWinner - arm it preferred with the wide text first

 @returns Verdict for this pair

 @example
 ```ts
 const verdict = readHeadToHead({ firstOrderWinner: 'wide', secondOrderWinner: 'wide', },);
 ```
 */
export function readHeadToHead(
  {
    firstOrderWinner,
    secondOrderWinner,
  }: {
    readonly firstOrderWinner: WidthArm | 'none';
    readonly secondOrderWinner: WidthArm | 'none';
  },
): HeadToHeadVerdict {
  // A panel that declined to rank in either order preferred neither, which is
  // a tie on the evidence rather than a missing measurement.
  if ((firstOrderWinner === 'none') && (secondOrderWinner === 'none'))
    return 'tied';

  if (firstOrderWinner !== secondOrderWinner)
    return 'position-decided';

  if (firstOrderWinner === 'wide')
    return 'wide-wins';

  if (firstOrderWinner === 'narrow')
    return 'narrow-wins';

  // Both orders agree and neither is `none`, which the both-`none` check
  // already handled, so the pair is `wide` or `narrow` and both returned. Reaching here
  // means an arm was added without a reading, and answering `tied` would report
  // that omission as a measurement.
  throw new Error(`unreachable: both orders named ${firstOrderWinner}, which is not an arm`,);
}

/**
 What the whole draw says, in the terms the width question has to answer in.
 */
export type WidthSummary = {
  /**
   Slices that produced a row at all.

   EVERY row, including those where neither arm shipped anything. The band
   and the move count are both read over this same set, which is what makes
   comparing them legitimate; see {@link WidthSummary.churned}.
   */
  readonly slices: number;

  /**
   Slices where neither arm shipped a repair.

   Reported so the reader can see how much of the draw was trivial, without
   removing those slices from the two counts that are compared against each
   other.
   */
  readonly nothingShipped: number;

  /**
   Slices where the shipped text moved when the editors widened.
   */
  readonly moved: number;

  /**
   Slices where the narrow arm run twice shipped different text. This is the
   null band for {@link WidthSummary.moved} and is measured, not assumed.

   COUNTED OVER EVERY ROW, for the same reason the move count is. Restricting
   the band to slices whose arms differed would drop exactly the rows that can
   carry churn but can never carry a move: a slice where both arms shipped
   nothing, yet the narrow repeat shipped something, is churn the band must
   see. Dropping those shrinks the band while leaving the move count alone,
   which tilts every reading toward width mattering.
   */
  readonly churned: number;

  /**
   Slices that moved WITHOUT churning.

   THIS AND ITS PARTNER ARE WHAT DECIDE IT. Both bits are measured on the same
   slice, so slices where they agree carry no information about which happens
   more: a slice that both moved and churned would have changed anyway, and a
   slice that did neither says nothing either way. The answer lives entirely in
   the slices where the two disagree, and comparing the raw totals throws that
   away.
   */
  readonly movedNotChurned: number;

  /**
   Slices that churned without moving, the other half of the paired reading.

   When this is not smaller than {@link WidthSummary.movedNotChurned}, the lane
   changes its own mind at least as readily as doubling the roster changes it,
   and width has not been shown to do anything.
   */
  readonly churnedNotMoved: number;

  /**
   Slices only the narrow arm shipped a repair on, so widening SUPPRESSED a
   repair rather than improving one.
   */
  readonly narrowOnly: number;

  /**
   Slices only the wide arm shipped a repair on.
   */
  readonly wideOnly: number;

  /**
   Head-to-head wins for the wide arm.
   */
  readonly wideWins: number;

  /**
   Head-to-head wins for the narrow arm.
   */
  readonly narrowWins: number;

  /**
   Pairs the slate position decided rather than the text.
   */
  readonly positionDecided: number;

  /**
   Pairs the panel would not separate.
   */
  readonly tied: number;
};

/**
 Whether widening moved this slice's shipped text.

 @param row - one slice's comparison

 @returns Whether the two arms shipped different text

 @example
 ```ts
 const moved = movedText(row,);
 ```
 */
function movedText(row: WidthRow,): boolean {
  return row.comparison === 'differs';
}

/**
 Whether widening changed this slice when the lane would not have.

 @param row - one slice's comparison

 @returns Whether it moved and did not churn

 @example
 ```ts
 const attributable = movedWithoutChurning(row,);
 ```
 */
function movedWithoutChurning(row: WidthRow,): boolean {
  return movedText(row,) && (!churnedOnRepeat(row,));
}

/**
 Whether the lane changed this slice on its own without widening changing it.

 @param row - one slice's comparison

 @returns Whether it churned and did not move

 @example
 ```ts
 const noise = churnedWithoutMoving(row,);
 ```
 */
function churnedWithoutMoving(row: WidthRow,): boolean {
  return churnedOnRepeat(row,) && (!movedText(row,));
}

/**
 Whether the narrow arm shipped a repair the wide arm did not.

 @param row - one slice's comparison

 @returns Whether widening cost this slice its repair

 @example
 ```ts
 const suppressed = onlyNarrowShipped(row,);
 ```
 */
function onlyNarrowShipped(row: WidthRow,): boolean {
  return row.narrowShipped && (!row.wideShipped);
}

/**
 Whether the wide arm shipped a repair the narrow arm did not.

 @param row - one slice's comparison

 @returns Whether widening bought this slice a repair

 @example
 ```ts
 const gained = onlyWideShipped(row,);
 ```
 */
function onlyWideShipped(row: WidthRow,): boolean {
  return row.wideShipped && (!row.narrowShipped);
}

/**
 Whether neither arm shipped a repair on this slice.

 @param row - one slice's comparison

 @returns Whether both arms left the translation as it stood

 @example
 ```ts
 const trivial = shippedNothing(row,);
 ```
 */
function shippedNothing(row: WidthRow,): boolean {
  return row.comparison === 'nothing-shipped';
}

/**
 Whether the narrow arm run twice disagreed with itself.

 @param row - one slice's comparison

 @returns Whether the repeat shipped different text

 @example
 ```ts
 const churned = churnedOnRepeat(row,);
 ```
 */
function churnedOnRepeat(row: WidthRow,): boolean {
  return !row.narrowRepeatAgreed;
}

/**
 Counts rows a predicate holds for.

 @param rows - rows to count over

 @param holds - predicate naming what is being counted

 @returns How many held

 @example
 ```ts
 const moved = countWhere({ rows, holds: movedText, },);
 ```
 */
function countWhere(
  {
    rows,
    holds,
  }: {
    readonly rows: readonly WidthRow[];
    readonly holds: (row: WidthRow) => boolean;
  },
): number {
  return rows
    .filter(holds,)
    .length;
}

/**
 Counts rows carrying one verdict.

 @param rows - rows to count over

 @param verdict - verdict being counted

 @returns How many carried it

 @example
 ```ts
 const wins = countVerdict({ rows, verdict: 'wide-wins', },);
 ```
 */
function countVerdict(
  {
    rows,
    verdict,
  }: {
    readonly rows: readonly WidthRow[];
    readonly verdict: HeadToHeadVerdict;
  },
): number {
  return rows
    .filter(function carries(row,) {
      return row.verdict === verdict;
    },)
    .length;
}

/**
 Counts the rows into the summary the decision reads.

 @param rows - every slice the draw produced

 @returns Counts, with no rate computed: a rate over a handful of slices
 invites reading three of seven as a percentage

 @example
 ```ts
 const summary = summarizeWidths({ rows, },);
 ```
 */
export function summarizeWidths(
  { rows, }: { readonly rows: readonly WidthRow[]; },
): WidthSummary {
  return {
    slices: rows.length,
    nothingShipped: countWhere({
      rows,
      holds: shippedNothing,
    },),
    moved: countWhere({
      rows,
      holds: movedText,
    },),
    churned: countWhere({
      rows,
      holds: churnedOnRepeat,
    },),
    movedNotChurned: countWhere({
      rows,
      holds: movedWithoutChurning,
    },),
    churnedNotMoved: countWhere({
      rows,
      holds: churnedWithoutMoving,
    },),
    narrowOnly: countWhere({
      rows,
      holds: onlyNarrowShipped,
    },),
    wideOnly: countWhere({
      rows,
      holds: onlyWideShipped,
    },),
    wideWins: countVerdict({
      rows,
      verdict: 'wide-wins',
    },),
    narrowWins: countVerdict({
      rows,
      verdict: 'narrow-wins',
    },),
    positionDecided: countVerdict({
      rows,
      verdict: 'position-decided',
    },),
    tied: countVerdict({
      rows,
      verdict: 'tied',
    },),
  };
}

//endregion Editor width model
