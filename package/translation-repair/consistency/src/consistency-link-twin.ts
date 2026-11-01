import type { FoldedLinks, } from './consistency-folded-links.ts';

//region Consistency link twin
// THE TEXT A READER SEES OF AN ENGLISH WITH LINKS IN IT, for the reading of closing marks (n169). The reading of
// `consistency-closing-marks.ts` runs over a folded English one unit at a time and asks what closes before an apostrophe. A link
// shows a reader its label and nothing of its brackets, destination or title (`consistency-link-label.ts`), so the reading is made
// over the twin of the English: the same text with each link that shows nothing taken out (`kit[](u)'kit` reads as `kit'kit`) and each
// link that shows closing marks and nothing else written as one closing square bracket (`kit[\]](u)'kit` and `kit[”](u)'kit` read as
// `kit]'kit`), and each link with a label that ends in an open mark written as its label (`[kit!](u)'s` reads as `kit!'s`, whose
// apostrophe no closing bracket precedes), and the offsets the twin finds are carried back to the English. A run of closing marks reaches the apostrophe through
// a run of closing marks of any length, so one closing bracket stands for a label of several without changing any answer.
//
// THE OFFSET OF A STAND'S END IN THE ENGLISH IS EITHER SIDE OF A LINK THAT SHOWS NOTHING, since the two are one place to a reader. A
// stand never ends inside a link, so the offsets carried back are the offset of each unit the twin kept and the offset of each link
// taken out directly before it.

/**
 The text a reader sees of a folded English, with the offsets to carry an answer about it back to the English.
 */
export type LinkTwin = {
  /**
   The folded English with each link that shows nothing taken out, each link of closing marks written as `]` and each link with a
   label that ends in an open mark written as its label.
   */
  readonly folded: string;

  /**
   The character the English held before the fold at each unit of the twin.
   */
  readonly originals: readonly string[];

  /**
   The offsets in the twin of the units that are the closing delimiter of an emphasis, strong or strikethrough stretch.
   */
  readonly closing: ReadonlySet<number>;

  /**
   The offset in the English of each unit of the twin, one more for the end of the text.
   */
  readonly starts: readonly number[];

  /**
   For an offset of the twin, the offsets in the English where the links taken out directly before it start.
   */
  readonly removedBefore: ReadonlyMap<number, readonly number[]>;
};

/**
 The closing square bracket a link of closing marks is written as in the twin.
 */
const CLOSING_STAND_IN = ']';

/**
 The twin of a folded English: what a reader sees of it, link by link.

 @param folded - the English in the fold

 @param originals - the character the English held before the fold at each UTF-16 unit of it

 @param closing - the offsets in the fold of the units that are the closing delimiter of an emphasis, strong or strikethrough stretch

 @param links - the links the parse holds, at the offsets of the fold

 @returns The twin and the offsets that carry an answer about it back

 @example
 ```ts
 const twin = linkTwinOf({ folded: 'kit[](u)\'kit', originals: Array.from('kit[](u)\'kit',), closing: new Set(), links, },); // twin.folded is "kit'kit"
 ```
 */
export function linkTwinOf(
  {
    folded,
    originals,
    closing,
    links,
  }: {
    readonly folded: string;
    readonly originals: readonly string[];
    readonly closing: ReadonlySet<number>;
    readonly links: FoldedLinks;
  },
): LinkTwin {
  /**
   The links that show nothing and the links of closing marks, by their first unit.
   */
  const {
    emptyEnds,
    closingSpans,
    openLabelStretches,
  } = links;
  /**
   The units of the twin, mutated only inside this function.
   */
  const units: string[] = [];
  /**
   The characters the units held before the fold, mutated only inside this function.
   */
  const twinOriginals: string[] = [];
  /**
   The closing delimiters of emphasis at the offsets of the twin, mutated only inside this function.
   */
  const twinClosing = new Set<number>();
  /**
   The offset in the English of each unit of the twin, mutated only inside this function.
   */
  const starts: number[] = [];
  /**
   The links taken out directly before the unit the reading has reached, mutated only inside this function.
   */
  const pending: number[] = [];
  /**
   The links taken out before each offset of the twin, mutated only inside this function.
   */
  const removedBefore = new Map<number, readonly number[]>();
  /**
   Keeps the links taken out so far as the ones directly before the next unit of the twin.
   */
  function settlePending(): void {
    if (pending.length === 0)
      return;
    removedBefore.set(
      units.length,
      [...pending,],
    );
    pending.length = 0;
  }
  /**
   The offset the reading has reached, mutated only inside this function.
   */
  const reading = { at: 0, };
  while (reading.at < folded.length) {
    /**
     The offset after the link that shows nothing, or the stretch of a link with a label that ends in an open mark (its opening
     bracket or its resource), that starts at the offset reached, absent where none does.
     */
    const emptyEnd = emptyEnds.get(reading.at,) ?? openLabelStretches.get(reading.at,);
    if (emptyEnd !== undefined) {
      pending.push(reading.at,);
      reading.at = emptyEnd;
      continue;
    }
    /**
     The offset after the link of closing marks that starts at the offset reached, absent where none does.
     */
    const closingEnd = closingSpans.get(reading.at,);
    settlePending();
    if (closingEnd !== undefined) {
      units.push(CLOSING_STAND_IN,);
      twinOriginals.push(CLOSING_STAND_IN,);
      starts.push(reading.at,);
      reading.at = closingEnd;
      continue;
    }
    if (closing.has(reading.at,))
      twinClosing.add(units.length,);
    units.push(folded.charAt(reading.at,),);
    twinOriginals.push(originals[reading.at] ?? '',);
    starts.push(reading.at,);
    reading.at += 1;
  }
  settlePending();
  starts.push(folded.length,);
  return {
    folded: units.join('',),
    originals: twinOriginals,
    closing: twinClosing,
    starts,
    removedBefore,
  };
}

/**
 The offsets of the English that stand for the offsets an answer about the twin names.

 @param twin - the twin of the English

 @param found - offsets of the twin

 @returns The offset in the English of each unit of the twin named, and the start of each link taken out directly before it

 @example
 ```ts
 const carried = originalOffsetsOf({ twin, found: new Set([3,],), },); // holds 3
 ```
 */
export function originalOffsetsOf(
  {
    twin,
    found,
  }: {
    readonly twin: LinkTwin;
    readonly found: ReadonlySet<number>;
  },
): ReadonlySet<number> {
  /**
   The offset in the English of each unit of the twin, and the links taken out before each offset of it.
   */
  const {
    starts,
    removedBefore,
  } = twin;
  /**
   The offsets of the English, mutated only inside this function.
   */
  const carried = new Set<number>();
  for (const offset of found) {
    /**
     The offset in the English of the unit of the twin at the offset.
     */
    const start = starts[offset];
    if (start !== undefined)
      carried.add(start,);
    /**
     The starts of the links taken out directly before the unit at the offset.
     */
    const taken = removedBefore.get(offset,) ?? [];
    for (const removed of taken)
      carried.add(removed,);
  }
  return carried;
}

//endregion Consistency link twin
