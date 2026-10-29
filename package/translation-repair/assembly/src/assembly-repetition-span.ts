import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Assembly repetition span
// GROWING WINDOWS INTO THE SPAN THEY BELONG TO, which is the step
// `assembly-repetition.ts` did not have, at a measured cost.
//
// That file finds repeats by counting fixed-length windows, longest first, and
// suppresses any phrase CONTAINED IN one already reported. That rule handles
// shorter-inside-longer and nothing else. A duplicated passage longer than the
// longest window it considers produces one finding per window position, all of
// the same length, so no one of them contains another and none suppresses any
// other. Measured on `vub171-20260822`'s `Zha_Ke`: one duplicated span of about
// 877 words reported as 866 findings, every one exactly 12 words.
//
// WHY THAT IS WORTH A MODULE RATHER THAN A FILTER. Findings are scorecard-stable
// tokens that get counted and compared across runs. One entry contributing 866
// of the corpus-wide 947 made every aggregate over that token a statement about
// one slice of one entry, so any rate or threshold read off it described the
// reporting shape rather than the corpus.
//
// THE MERGE RULE IS ABOUT OCCURRENCES, NOT ADJACENCY, and that distinction is
// the whole correctness argument. Two windows sitting next to each other in the
// document are not evidence of one repeat: they could be two different repeated
// passages that happen to abut. What makes them one repeat is that the second
// window occurs in exactly the places the first does, each advanced by one
// word. Merging on adjacency alone would join unrelated repeats into a single
// invented span; merging on the occurrence sets cannot, because two unrelated
// repeats do not repeat in the same places.

/**
 Every window of one length, indexed by where it sits and by what it spells.

 BOTH DIRECTIONS ARE NEEDED and neither derives cheaply from the other here.
 Growing a span walks offsets and asks what each spells, while the merge test
 asks where a phrase occurs. Recomputing either side per step would re-slice
 the word list on every window.

 @example
 ```ts
 const index: WindowIndex = indexWindows({ words, length: 12, },);
 ```

 Shared with the repetition finder; not part of the lane contract.

 @internal
 */
export type WindowIndex = {
  /**
   Phrase each window spells, by that window's offset.
   */
  readonly byOffset: readonly string[];

  /**
   Offsets each phrase occupies, ascending.
   */
  readonly byPhrase: ReadonlyMap<string, readonly number[]>;
};

/**
 One repeated passage, grown to its full length.

 @example
 ```ts
 const span: GrownSpan = { phrase: 'the same thing said twice over', count: 2, };
 ```

 Shared with the repetition finder; not part of the lane contract.

 @internal
 */
export type GrownSpan = {
  /**
   Words of the whole span, space-joined as the windows were.
   */
  readonly phrase: string;

  /**
   Times the span occurs in the document it was grown from.
   */
  readonly count: number;

  /**
   Whether an earlier span already accounts for every occurrence of this one.

   REPORTED RATHER THAN ACTED ON, because the two things a span is used for
   part company here. A derivative span must not become a FINDING, since it
   describes a duplication already named. It must still SUPPRESS the shorter
   phrases inside it, because those are equally derivative and the caller's
   containment rule can only suppress against a span it was given. Dropping
   such a span outright was measured to turn one finding into eleven: the
   pieces of it reappeared at every shorter length.
   */
  readonly accountedFor: boolean;
};

/**
 Indexes every window of one length over a word list.

 @param words - document as a word list

 @param length - words per window

 @returns Both indexes over the same windows

 @example
 ```ts
 const index = indexWindows({ words, length: 12, },);
 ```

 Shared with the repetition finder; not part of the lane contract.

 @internal
 */
export function indexWindows(
  {
    words,
    length,
  }: {
    readonly words: readonly string[];
    readonly length: number;
  },
): WindowIndex {
  /**
   Phrase of each window, in offset order.
   */
  const byOffset: string[] = [];

  /**
   Offsets of each phrase, filled in ascending order because the walk is.
   */
  const byPhrase = new Map<string, number[]>();

  for (let at = 0; (at + length) <= words.length; at += 1) {
    /**
     This window as one comparable string.
     */
    const phrase = words
      .slice(
      at,
      at + length,
      )
      .join(' ',);
    byOffset.push(phrase,);

    /**
     Offsets already recorded for it, absent the first time it is seen.
     */
    const seen = byPhrase.get(phrase,);
    if (seen === undefined)
      byPhrase.set(
        phrase,
        [at,],
      );
    else
      seen.push(at,);
  }

  return {
    byOffset,
    byPhrase,
  };
}

/**
 Whether one window's occurrences are another's advanced by exactly one word.

 This is the test that makes a merge safe. It holds only when the two windows
 are consecutive pieces of the SAME repeated passage: every place the first
 occurs, the second occurs one word later, and nowhere else.

 @param earlier - occurrences of the window on the left

 @param later - occurrences of the window one word to its right

 @returns Whether the two are one passage rather than two

 @example
 ```ts
 const together = advancesByOne({ earlier: [3, 40,], later: [4, 41,], },);
 ```
 */
function advancesByOne(
  {
    earlier,
    later,
  }: {
    readonly earlier: readonly number[];
    readonly later: readonly number[];
  },
): boolean {
  return (earlier.length === later.length)
    && earlier.every(function nextTo(
      position,
      index,
    ): boolean {
      return later[index] === (position + 1);
    },);
}

/**
 Whether reported ranges already hold every occurrence of one passage.

 @param covered - ranges an earlier span accounted for, merged and disjoint

 @param occurrences - where this passage sits

 @param wordCount - words it spans

 @returns Whether it describes a duplication already named

 @internal

 @example
 ```ts
 const derivative = accountedForBy({ covered, occurrences: [8, 27,], wordCount: 22, },);
 ```
 */
function accountedForBy(
  {
    covered,
    occurrences,
    wordCount,
  }: {
    readonly covered: readonly WordRegion[];
    readonly occurrences: readonly number[];
    readonly wordCount: number;
  },
): boolean {
  return occurrences.every(function inside(position,): boolean {
    return covered.some(function holds(region,): boolean {
      return (region.start <= position)
        && ((position + wordCount) <= region.end);
    },);
  },);
}

/**
 Every place the window at one offset sits.

 AN INDEX COVERS EVERY OFFSET IT IS ASKED FOR here: it is built by
 {@link indexWindows} over the same words and length the walk reads, which
 indexes every offset up to the last a window fits at, and the walk never
 passes that offset. A miss is a caller handing in another list's index,
 never a window that occurs nowhere.

 @param index - windows of one length over the walked words

 @param offset - window offset, at most the last a window fits at

 @returns Offsets the same phrase occupies, ascending

 @example
 ```ts
 const places = occurrencesAt({ index, offset: 0, },);
 ```
 */
function occurrencesAt(
  {
    index,
    offset,
  }: {
    readonly index: WindowIndex;
    readonly offset: number;
  },
): readonly number[] {
  /**
   What the window at that offset spells.
   */
  const phrase = nonNullishOrThrow(index.byOffset[offset],);
  /**
   Offsets that phrase occupies.
   */
  const places = index.byPhrase
    .get(phrase,);
  return nonNullishOrThrow(places,);
}

/**
 Grows admitted windows into the maximal passages they belong to.

 REPORTS NOTHING ALREADY ACCOUNTED FOR, which is one rule doing two jobs and
 is the generalisation of the containment rule a caller applies to shorter
 phrases. A span whose every occurrence sits inside text an already-reported
 span covers is not a second fact about the document.

 It catches the span reached again at its own second occurrence, which would
 otherwise be emitted once per occurrence. It also catches the artifact a
 passage said three or more times produces: in `P P P` the join between two
 copies occurs twice, so the tail of one copy followed by the head of the next
 is itself a repeat, and reporting it beside `P said three times` describes
 one duplication as two. Measured: without this the triple case reported two
 findings rather than one.

 A RUN THAT BREAKS EARLY IS NOT AN ERROR. Where a window occurs somewhere the
 rest of its passage does not, the occurrence sets stop advancing together and
 the span ends there. That is two facts rather than one, and reporting them as
 two is right: the shorter phrase really does occur more often than the longer
 one containing it.

 @param words - document as a word list

 @param length - words per window, which is the shortest span this can emit

 @param index - windows of that length over those words

 @param admitted - offsets whose window is worth reporting on its own

 @returns Maximal spans, in the order their first occurrence appears, each
 saying whether an earlier one already accounts for it

 @example
 ```ts
 const spans = grownSpans({ words, length: 12, index, admitted, },);
 ```

 Shared with the repetition finder; not part of the lane contract.

 @internal
 */
export function grownSpans(
  {
    words,
    length,
    index,
    admitted,
  }: {
    readonly words: readonly string[];
    readonly length: number;
    readonly index: WindowIndex;
    readonly admitted: ReadonlySet<number>;
  },
): readonly GrownSpan[] {
  /**
   Spans closed so far.
   */
  const spans: GrownSpan[] = [];

  /**
   Word ranges already accounted for by an emitted span, as half-open
   intervals over the word list.
   */
  let covered: readonly WordRegion[] = [];

  /**
   Last offset a window of this length can start at.
   */
  const last = words.length - length;

  /**
   Window the walk is looking at, advanced past each run it closes.
   */
  let at = 0;
  while (at <= last) {
    if (!admitted.has(at,)) {
      at += 1;
      continue;
    }

    /**
     Last window of this run, extended while the next one belongs to the same
     passage.
     */
    let end = at;
    while ((end < last)
      && admitted.has(end + 1,)
      && advancesByOne({
        earlier: occurrencesAt({
          index,
          offset: end,
        },),
        later: occurrencesAt({
          index,
          offset: end + 1,
        },),
      },))
      end += 1;

    /**
     Whole passage this run spells.
     */
    const phrase = words
      .slice(
      at,
      end + length,
      )
      .join(' ',);

    /**
     Every place this passage sits, which the merge test has kept equal to
     the places its first window sits.
     */
    const occurrences = occurrencesAt({
      index,
      offset: at,
    },);

    /**
     Words the passage spans.
     */
    const wordCount = (end + length) - at;

    /**
     Whether an already-reported span accounts for every occurrence of this
     one, in which case this is not a second fact about the document.
     */
    const accountedFor = accountedForBy({
      covered,
      occurrences,
      wordCount,
    },);
    spans.push({
      phrase,
      count: occurrences.length,
      accountedFor,
    },);
    if (!accountedFor) {
      covered = mergeRegions({
        regions: [
          ...covered,
          ...occurrences.map(function toRegion(position,): WordRegion {
            return {
              start: position,
              end: position + wordCount,
            };
          },),
        ],
      },);
    }
    at = end + 1;
  }

  return spans;
}

/**
 Half-open range of words, `start` inclusive and `end` exclusive.

 @example
 ```ts
 const region: WordRegion = { start: 0, end: 19, };
 ```
 */
type WordRegion = {
  /**
   First word of the range.
   */
  readonly start: number;

  /**
   Word after the last, so an empty range has `start` equal to `end`.
   */
  readonly end: number;
};

/**
 Merges ranges into the smallest disjoint set covering the same words.

 WITHOUT THIS THE COVERAGE TEST IS WRONG IN THE CASE IT EXISTS FOR. A passage
 said three times covers three ranges that meet end to start, and the junction
 artifact spans the seam between two of them. Asked whether any ONE range
 holds it, the answer is no; asked whether the ranges TOGETHER hold it, the
 answer is yes, and yes is correct. Measured: without merging, the triple case
 still reported two findings.

 Touching ranges join, since `[0, 19)` and `[19, 38)` leave no word between
 them.

 @param regions - ranges in any order, possibly overlapping

 @returns Ranges ascending by start, none touching another

 @example
 ```ts
 const union = mergeRegions({ regions: [{ start: 19, end: 38, }, { start: 0, end: 19, },], },);
 ```
 */
function mergeRegions(
  { regions, }: { readonly regions: readonly WordRegion[]; },
): readonly WordRegion[] {
  /**
   Ranges kept so far, each starting after the previous one ends.
   */
  const joined: WordRegion[] = [];
  for (
    const region of regions.toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },)
  ) {
    /**
     Range this one either extends or follows.
     */
    const previous = joined.at(-1,);
    if ((previous === undefined) || (region.start > previous.end)) {
      joined.push(region,);
      continue;
    }
    joined[joined.length - 1] = {
      start: previous.start,
      end: Math.max(
        previous.end,
        region.end,
      ),
    };
  }
  return joined;
}

/**
 Counts occurrences of one passage in a word list.

 WORD-WISE RATHER THAN OVER THE TEXT, so the count means the same thing as the
 window counts it is compared against: whitespace between words differs
 between the archive and the assembled document, and a text search would miss
 a passage that is only rewrapped.

 @param words - document as a word list

 @param phrase - passage, space-joined as {@link indexWindows} joins

 @returns Times that passage occurs

 @example
 ```ts
 const times = countSpan({ words: archiveWords, phrase, },);
 ```

 Shared with the repetition finder; not part of the lane contract.

 @internal
 */
export function countSpan(
  {
    words,
    phrase,
  }: {
    readonly words: readonly string[];
    readonly phrase: string;
  },
): number {
  /**
   Passage back as the words it was joined from.
   */
  const span = phrase.split(' ',);

  /**
   Offsets where the whole passage sits.
   */
  const at: number[] = [];
  for (let start = 0; (start + span.length) <= words.length; start += 1) {
    if (span.every(function matches(
      word,
      offset,
    ): boolean {
      return words[start + offset] === word;
    },))
      at.push(start,);
  }
  return at.length;
}

//endregion Assembly repetition span
