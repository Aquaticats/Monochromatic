import { markPastRunOf, } from './consistency-closing-marks.ts';
import { decidingQuorumOf, } from './consistency-deciding-quorum.ts';
import {
  locateWords,
  type SpanSearch,
} from './consistency-fold.ts';
import { singleMarkAt, } from './consistency-loose-marks.ts';
import { carriesOrCutsMarkup, } from './consistency-markup.ts';
import {
  isKeptByMajority,
  keepsByLabelOf,
  majorityChoiceOf,
  markEvidenceOf,
  type QuotesByLabel,
  quotesByLabelOf,
  runOffFinalistsOf,
  usableVoicesOf,
} from './consistency-majority.ts';
import { slotInSpan, } from './consistency-slot-in-span.ts';
import { containedQuoteWordsOf, } from './consistency-span-agreement.ts';
import { unlessOverlapping, } from './consistency-count-overlap.ts';
import { ConsistencyInvariantError, } from './consistency-invariant-error.ts';
import type {
  ConsistencyCount,
  ConsistencyRendering,
  ConsistencyVoice,
  ConsistencyWinner,
} from './consistency-count-types.ts';
import type { PageEnglish, } from './consistency-page-english.ts';
import type { ConsistencyEnglish, } from './consistency-wire.ts';
import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { definitionsMovingSlotOf, } from './consistency-slot-definitions.ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency count
// Counting a repeated wording's ballots, deterministically and from usable
// ballots only (the owner's ruling of 2026-10-11, with the lead's design).
//
// BALLOTS AGREE ON A PLACE WHEN THEIR QUOTES LOCATE THE SAME SPAN OF THE PLACE'S
// OWN ENGLISH (`consistency-fold.ts`): a line break written as a space, a curly
// apostrophe written straight, the closing full stop kept or left off and a run
// of spaces do not make two quotes two texts. THE PLACE IS LOCATED BY THE LONGEST
// OF ITS QUOTES THAT STANDS EXACTLY ONCE in its English and that the quotes of more
// than half of all the usable ballots contain in the fold (`consistency-span-agreement.ts`,
// the lead's ruling of 2026-10-15: quotes of one span of different lengths, the second
// judge's a part of the first's, agree on the shorter, at the risk of leaving out words
// a judge meant to carry); the place then carries the words the English holds there, never a
// judge's string, and the span never begins or ends between the halves of a character reference the parser decodes or of a
// backslash escape (`wholeSpanOf`, `consistency-whole-end.ts`: a quote that stops before the semicolon of `&amp;` or starts at
// its `amp;` names the span with the whole reference, and one that stops before the full stop in `\.` names the span without
// it, the unit being kept or cut as the character it shows would be). A ballot whose quote stands nowhere or twice is one of the
// usable ballots that did not agree. Otherwise the place is unlocated and stays
// as it stands. THE CHOICE WINS ON A STRICT MAJORITY and only with the quorum
// the gather closed on, never fewer than two usable ballots (`decidingQuorumOf`: one judge alone never decides,
// the lead's ruling of 2026-10-25), and only if its own span is located; with no strict
// majority, the labels with the highest count (and, when one label leads alone, the labels
// next behind it) are named as the finalists of a run-off, which the pass asks
// once (`finalists`). A PLACE IS KEPT when a strict majority list it.
//
// WORDS THAT STAND MORE THAN ONCE ARE TOLD APART BY THE MARKS THE BALLOTS COPIED.
// The agreed words are folded and hold no mark, so the counter gives the locator
// the evidence itself (`markEvidenceOf` in `consistency-majority.ts`): a side
// holds when a strict majority of the usable ballots (not of those that agreed
// on the words) quoted the agreed words with a single mark directly beside them
// on that side, and a ballot that copied no mark is no evidence against it. The
// locator uses it only where the words stand more than once, in a plain group
// and in a frame alike.
//
// A SLOT IS NO CONDITION OF A SPAN. A frame's slot words are agreed the same way
// inside the span (`consistency-slot-in-span.ts`), and a place whose slot is not
// agreed (or is all of the span, or stands twice, or cuts markup) is still located,
// only without a slot, which leaves the code splice unavailable for it and the
// writers to carry the slot.
//
// A LOCATED RENDERING RECORDS THE SINGLE MARKS BESIDE ITS SPAN. The span is the
// words and the proven apostrophe beside them, if there is one (`the parents’`,
// `consistency-proven-apostrophes.ts`): that mark is inside the span and is not
// recorded beside it. An apostrophe with a possible partner or a quotation mark the
// English holds directly before or after the span stays outside; the rendering
// keeps it (`markBefore`, `markAfter`) so the splice (`consistency-splice.ts`) can
// refuse to move words between two places that hold different marks beside them,
// which would write a pairing of words and marks neither place wrote. WHETHER A MARK IS
// PROVEN IS ASKED OF THE WHOLE PAGE, not of the place's slice: a place carries the
// English of its page (`page`, `consistency-page-english.ts`), so a closing mark whose
// quotation opened in an earlier slice is no apostrophe of the words.
//
// A MARK ONE RUN OF MARKUP BEYOND THE SPAN IS RECORDED TOO, apart from the mark directly
// beside it. When the span is followed (or preceded) by a run of emphasis characters, a
// backtick, closing brackets or double quotation marks, a link's destination, any unit no reader
// reads as a word (a tag, a footnote label, a comment, a character reference) or horizontal white
// space on its line, and a single mark ends the run (`*kittens*’`, `` `kittens`’ ``), the rendering keeps that mark as
// `markPastAfter` (`markPastBefore` on the leading side), read by `markPastRunOf`
// (`consistency-closing-marks.ts`). It is no part of the comparison of the marks beside the
// words; the splice uses it to refuse a replacement that would bring a proven apostrophe to
// that edge, which would sit next to it as a second apostrophe (`*dogs’*’`). The field is
// absent when no run stands there, and it is not written to the record, which holds only the
// marks directly beside the words.
//
// A SPAN THAT CANNOT BE MOVED WHOLE STAYS WHERE IT IS: one that cuts markup or
// carries a footnote mark (`consistency-markup.ts`), and two places of one slice
// whose words overlap (`consistency-count-overlap.ts`), are left unlocated.
//
// NOTHING HERE WRITES TEXT. The words a replacement carries are words the
// English holds, and the splice (`consistency-splice.ts`) only moves them.

/**
 One candidate span of a place and where it stands in the English.
 */
type SearchedCandidate = {
  /**
   The candidate's folded words.
   */
  readonly words: string;

  /**
   Where they stand in the English, or how they fail to stand once.
   */
  readonly found: SpanSearch;
};

/**
 A candidate that stands exactly once, with where it stands.
 */
type StandingCandidate = {
  /**
   The candidate's folded words.
   */
  readonly words: string;

  /**
   Offset in the English where the span starts.
   */
  readonly start: number;

  /**
   Offset in the English just past the span.
   */
  readonly end: number;
};

/**
 Locates the span a strict majority of the usable ballots named for one place,
 and the slot inside it that they agreed on.

 @param voices - usable ballots

 @param label - place's label

 @param english - place's English

 @param page - English of the whole page the place belongs to and where the place's English starts in it, which the proof of an apostrophe reads; the place's English alone is the page where absent

 @param framed - whether the group is a frame with a slot

 @param quotesByLabel - quote entries of the usable ballots by label, read once for the group

 @returns Located rendering, carrying the words the English holds, or why it is not

 @example
 ```ts
 const rendering = locatedRendering({ voices, label: 'OCCURRENCE 1', english, framed: false, quotesByLabel, },);
 ```
 */
function locatedRendering(
  {
    voices,
    label,
    english,
    page,
    framed,
    quotesByLabel,
  }: {
    readonly voices: readonly ConsistencyVoice[];
    readonly label: string;
    readonly english: ConsistencyEnglish;
    readonly page?: PageEnglish;
    readonly framed: boolean;
    readonly quotesByLabel: QuotesByLabel;
  },
): ConsistencyRendering {
  if (english.kind === 'absent') {
    return {
      kind: 'unlocated',
      reason: 'no-english',
    };
  }
  /**
   The link and footnote definitions of the page, which a reference in this place's English resolves against whichever slice holds
   the definition; none for a place that carries no page.
   */
  const definitions = (page === undefined) ? '' : pageDefinitionsOf({ pageText: page.text, },);
  /**
   The spans the quotes of more than half of all the usable ballots contain, in the fold, longest first.
   */
  const candidates = containedQuoteWordsOf({
    voices,
    label,
    quotesByLabel,
  },);
  if (candidates.length === 0) {
    return {
      kind: 'unlocated',
      reason: 'no-agreement',
    };
  }
  /**
   The candidates that show a reader something.
   */
  const readable = candidates.filter(function showsSomething(words,): boolean {
    return !rendersAsNothing({ text: words, },);
  },);
  if (readable.length === 0) {
    return {
      kind: 'unlocated',
      reason: 'empty',
    };
  }
  /**
   Where each candidate stands in the English, the single marks a strict majority of the usable ballots copied beside it
   choosing between stands, in the order of the candidates.
   */
  const searched = readable.map(function searchFor(words,): SearchedCandidate {
    return {
      words,
      found: locateWords({
        haystack: english.text,
        words,
        marks: markEvidenceOf({
          voices,
          label,
          words,
          quotesByLabel,
        },),
        ...((page === undefined) ? {} : { page, }),
      },),
    };
  },);
  // THE LONGEST CANDIDATE THAT STANDS EXACTLY ONCE LOCATES THE PLACE; one that stands nowhere or more than once is passed over
  // for the next longest, and when none stands once the place reports how the longest failed.
  /**
   The candidates that stand once, longest first.
   */
  const standingOnce = searched.flatMap(function standsOnce({
    words,
    found,
  }: SearchedCandidate,): readonly StandingCandidate[] {
    return (found.kind === 'once') ? [{
      words,
      start: found.start,
      end: found.end,
    },] : [];
  },);
  /**
   The longest candidate that stands once.
   */
  const [standing,] = standingOnce;
  if (standing === undefined) {
    /**
     How the longest candidate failed to stand once, which the place reports.
     */
    const [longest,] = searched.map(function kindOf({ found, }: SearchedCandidate,): SpanSearch['kind'] {
      return found.kind;
    },);
    return {
      kind: 'unlocated',
      reason: (longest === 'many') ? 'ambiguous' : 'not-found',
    };
  }
  /**
   The folded words that locate the place.
   */
  const { words, } = standing;
  /**
   Where they stand in the English.
   */
  const found = {
    start: standing.start,
    end: standing.end,
  };
  /**
   Where the span ends, which the locator has taken whole where a judge's quote stopped inside a character reference or a
   backslash escape.
   */
  const spanEnd = found.end;
  /**
   The English's own words of the span.
   */
  const span = english.text
    .slice(
    found.start,
    spanEnd,
    );
  if (carriesOrCutsMarkup({ text: span, },)) {
    return {
      kind: 'unlocated',
      reason: 'markup',
    };
  }
  /**
   The single mark directly before the span, as the English holds it, as a list of zero or one; a proven apostrophe the span took in is not beside it.
   */
  const [markBefore,] = singleMarkAt({
    text: english.text,
    at: found.start - 1,
  },);
  /**
   The single mark directly after the span, as the English holds it, as a list of zero or one.
   */
  const [markAfter,] = singleMarkAt({
    text: english.text,
    at: spanEnd,
  },);
  /**
   The single mark past a run of markup or opening characters before the span, none when a run is absent or ends in no mark.
   */
  const [markPastBefore,] = markPastRunOf({
    text: english.text,
    at: found.start,
    side: 'before',
    definitions,
  },);
  /**
   The single mark past a run of markup or closing characters after the span.
   */
  const [markPastAfter,] = markPastRunOf({
    text: english.text,
    at: spanEnd,
    side: 'after',
    definitions,
  },);
  /**
   The span with the marks beside it, which the splice compares between places.
   */
  const bare: Extract<ConsistencyRendering, { readonly kind: 'located'; }> = {
    kind: 'located',
    quote: span,
    core: span,
    start: found.start,
    ...((markBefore === undefined) ? {} : { markBefore, }),
    ...((markAfter === undefined) ? {} : { markAfter, }),
    ...((markPastBefore === undefined) ? {} : { markPastBefore, }),
    ...((markPastAfter === undefined) ? {} : { markPastAfter, }),
  };
  if (!framed)
    return bare;
  /**
   The slot inside the span, none when the ballots did not pin one down.
   */
  const [inSpan,] = slotInSpan({
    voices,
    label,
    words,
    span,
    quotesByLabel,
    definitions,
  },);
  /**
   The definitions of the page that move the slot, none when the page's definitions change nothing; the artifact stores them.
   */
  const moving = definitionsMovingSlotOf({
    voices,
    label,
    words,
    span,
    quotesByLabel,
    definitions,
  },);
  return {
    ...bare,
    ...inSpan,
    ...((moving === '') ? {} : { definitions: moving, }),
  };
}

/**
 Counts the ballots of one group.

 @param voices - every heard ballot with its seat

 @param places - label and English of each place, in page order, with the slice
 it stands in where the caller knows it and the English of the whole page with
 the offset where the place's English starts in it

 @param framed - whether the group is a frame with a slot

 @param quorum - voices the gather closed on, which the usable ballots must reach and never fewer than two (a quorum of
 one, on a seated bench of one or two seats, is raised to two)

 @returns What the usable ballots come to

 @example
 ```ts
 const count = countConsistency({ voices, places, framed: false, quorum: 2, },);
 ```
 */
export function countConsistency(
  {
    voices,
    places,
    framed,
    quorum,
  }: {
    readonly voices: readonly ConsistencyVoice[];
    readonly places: readonly {
      readonly label: string;
      readonly english: ConsistencyEnglish;
      readonly sliceIndex?: number;
      readonly page?: PageEnglish;
    }[];
    readonly framed: boolean;
    readonly quorum: number;
  },
): ConsistencyCount {
  /**
   Labels the sheet showed.
   */
  const labels = places.map(function labelOf(place,): string {
    return place.label;
  },);
  /**
   Ballots that can be counted.
   */
  const usableVoices = usableVoicesOf({
    voices,
    labels,
  },);
  /**
   The quotes of the usable ballots by place, read once for the group so each place is looked up and the ballots are not read
   again for every place.
   */
  const quotesByLabel = quotesByLabelOf({ voices: usableVoices, },);
  /**
   How many usable ballots keep each place, read once for the group for the same reason.
   */
  const keepsByLabel = keepsByLabelOf({ voices: usableVoices, },);
  /**
   Each place's rendering.
   */
  const renderings = unlessOverlapping({
    places,
    renderings: places.map(function renderingOf(place,): ConsistencyRendering {
      return locatedRendering({
        voices: usableVoices,
        label: place.label,
        english: place.english,
        ...((place.page === undefined) ? {} : { page: place.page, }),
        framed,
        quotesByLabel,
      },);
    },),
  },);
  /**
   Whether a strict majority keeps each place.
   */
  const kept = places.map(function isKept(place,): boolean {
    return isKeptByMajority({
      voices: usableVoices,
      label: place.label,
      keepsByLabel,
    },);
  },);
  /**
   The place a strict majority chose, as a list of zero or one.
   */
  const chosen = majorityChoiceOf({ voices: usableVoices, },);
  return {
    usable: usableVoices.length,
    renderings,
    kept,
    winner: winnerOf({
      usable: usableVoices.length,
      quorum,
      chosen,
      labels,
      renderings,
    },),
    finalists: ((usableVoices.length === 0) || (usableVoices.length < decidingQuorumOf({ quorum, },)))
      ? []
      : runOffFinalistsOf({
        voices: usableVoices,
        labels,
      },),
  };
}

/**
 Decides the group's winner from the facts the ballots gave.

 @param usable - usable ballots

 @param quorum - voices the gather closed on, raised to two where it is one

 @param chosen - label a strict majority chose, as a list of zero or one

 @param labels - labels of the places, in order

 @param renderings - each place's rendering, in order

 @returns Winner, or why there is none

 @throws {@link ConsistencyInvariantError} when the chosen label is none of the
 sheet's labels, which a usable ballot cannot choose

 @example
 ```ts
 const winner = winnerOf({ usable: 3, quorum: 2, chosen: ['OCCURRENCE 1',], labels, renderings, },);
 ```
 */
export function winnerOf(
  {
    usable,
    quorum,
    chosen,
    labels,
    renderings,
  }: {
    readonly usable: number;
    readonly quorum: number;
    readonly chosen: readonly string[];
    readonly labels: readonly string[];
    readonly renderings: readonly ConsistencyRendering[];
  },
): ConsistencyWinner {
  // A quorum of one (a seated bench of one or two seats) is raised to two: one judge alone never decides.
  if ((usable === 0) || (usable < decidingQuorumOf({ quorum, },))) {
    return {
      kind: 'none',
      reason: 'no-quorum',
    };
  }
  /**
   The label a strict majority chose, if any.
   */
  const [label,] = chosen;
  if (label === undefined) {
    return {
      kind: 'none',
      reason: 'no-majority',
    };
  }
  /**
   Position of the chosen label among the places.
   */
  const position = labels.indexOf(label,);
  if (position === (-1)) {
    throw new ConsistencyInvariantError({ invariant:
      `the label ${label} a majority chose is no label of the sheet, and a usable ballot chooses a label the sheet showed`, },);
  }
  if (renderings[position]
    ?.kind
    !== 'located') {
    return {
      kind: 'none',
      reason: 'chosen-unlocated',
    };
  }
  return {
    kind: 'chosen',
    label,
  };
}

//endregion Consistency count
