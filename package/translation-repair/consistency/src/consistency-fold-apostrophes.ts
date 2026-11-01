import { joinedMarksOf, } from './consistency-joined-marks.ts';
import { marksPastMarkupOf, } from './consistency-marks-past-markup.ts';
import { proseViewOf, } from './consistency-prose-text.ts';
import { singleMarkAt, } from './consistency-loose-marks.ts';
import {
  NO_PLACE,
  type PageEnglish,
  pageReadingOf,
  positionOnPage,
} from './consistency-page-english.ts';
import {
  type MarkReading,
  markReadingsOf,
  type ProvenApostrophes,
  provenApostrophesOf,
} from './consistency-proven-apostrophes.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Consistency fold apostrophes
// THE SPAN OF A QUOTE'S WORDS TAKES IN THE PROVEN APOSTROPHE BESIDE THEM. The
// locator (`consistency-fold.ts`) finds the words of a quote with every single mark
// at their edges dropped, so the stand it finds holds the words alone. This step
// widens that stand over the one mark that is an apostrophe beyond doubt
// (`consistency-proven-apostrophes.ts`): a trailing one directly after the words'
// last character, a leading one directly before their first. It lives apart from
// the locator so that file stays within its length, and the rule it applies is the
// one `consistency-proven-apostrophes.ts` states, nothing more: a mark is proven
// only when no quotation-like character anywhere in the prose of the PAGE could
// be its partner, in its paragraph, in its slice or in any other slice of the page.
// THE PAGE IS THE CALLER'S TO NAME (`consistency-page-english.ts`): a caller that holds
// the English of every slice hands the page and where the words' English starts in it,
// and a caller that holds one English alone gets that English as the page.

/**
 The proven apostrophes of an English, read over its prose: the stretches the page
 does not read as words (code, the strings of a component, a comment, a link's
 destination) hold no partner, and an English the grammar refuses is read whole.

 @param english - the English of a page, or of the slice alone where no more is known

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: a reference
 in the English whose definition stands in another slice is then plain text, and the delimiters of its emphasis pair as plain
 text does (n173). An English that is the page itself holds its definitions already and needs none given

 @returns The offsets in the English of the proven marks, trailing and leading apart

 @example
 ```ts
 const { trailing, } = provenApostrophesInEnglish({ english: 'The parents’ bowl', },); // trailing holds 11
 ```
 */
export function provenApostrophesInEnglish(
  {
    english,
    definitions = '',
  }: {
    readonly english: string;
    readonly definitions?: string;
  },
): ProvenApostrophes {
  /**
   What the proof reads of the English and proves of it.
   */
  const view = proofViewOf({
    english,
    definitions,
  },);
  return view.proven;
}

/**
 What the proof reads of an English and what it proves of it: the prose, which keeps every offset of the English, and the
 proven apostrophes at those offsets.
 */
export type ProofView = {
  /**
   The prose of the English: as long as it, with the stretches the page does not read as words blanked.
   */
  readonly prose: string;

  /**
   The proven apostrophes of the prose, by offset.
   */
  readonly proven: ProvenApostrophes;

  /**
   The offsets of the marks the proof reads as joined to the word after them across closing markup (`**cat**'s`), which
   read as attached.
   */
  readonly joined: ReadonlySet<number>;

  /**
   The apostrophes the page writes as part of a word, in order, with the character each is written in: a mark a Latin letter
   or an ASCII digit stands beside on both sides, a mark joined to the word after it across closing markup, and a proven one
   at the edge of a word. They are read with the view, once for the page, so the form of a page's apostrophes
   (`consistency-apostrophe-forms.ts`) is a filter over this list and not a walk of the page for each place.
   */
  readonly wordApostrophes: readonly WordApostrophe[];
};

/**
 One apostrophe the page writes as part of a word.
 */
export type WordApostrophe = {
  /**
   Offset of the mark in the prose, which is the offset in the English as written.
   */
  readonly at: number;

  /**
   The character the mark is written in.
   */
  readonly character: string;
};

/**
 How many English texts the proof remembers: the page a pass is working on, and the pages the gain check makes of it.
 */
const REMEMBERED_VIEWS = 4;

/**
 One English a view was read for with the definitions it was read with, and the view.
 */
type RememberedView = {
  /**
   The English the view was read from.
   */
  readonly english: string;

  /**
   The definitions the English was read with, empty for an English read alone.
   */
  readonly definitions: string;

  /**
   What the proof read of it.
   */
  readonly view: ProofView;
};

/**
 The views of the most recent English texts, the newest last, so a page read once for one place is not read again for the
 next place of the same page. A view holds only what is read from the English and the definitions it was read with, and a view is
 found by both, so remembering it changes no answer.
 */
const remembered: RememberedView[] = [];

/**
 What the proof reads of an English before it is shaped into a view: the readings of every apostrophe-like mark beside the
 prose, the proven marks and the joined ones.
 */
export type ProofReading = {
  /**
   The prose of the English.
   */
  readonly prose: string;

  /**
   The proven apostrophes of the prose.
   */
  readonly proven: ProvenApostrophes;

  /**
   The marks read as joined to the word after them across closing markup.
   */
  readonly joined: ReadonlySet<number>;

  /**
   How every apostrophe-like mark reads, by its offset.
   */
  readonly readings: ReadonlyMap<number, MarkReading>;
};

/**
 What the proof reads of an English, read now and remembered by no one.

 @internal

 @param english - the English of a page, or of the slice alone where no more is known

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: a reference
 in the English whose definition stands in another slice is then plain text, and the delimiters of its emphasis pair as plain
 text does (n173). An English that is the page itself holds its definitions already and needs none given

 @returns The prose, the proven apostrophes, the marks read as joined and the reading of each apostrophe-like mark

 @example
 ```ts
 const { joined, } = readProof({ english: 'The *cats*’s bowl', },); // holds 10
 ```
 */
export function readProof(
  {
    english,
    definitions = '',
  }: {
    readonly english: string;
    readonly definitions?: string;
  },
): ProofReading {
  /**
   The prose of the English and the delimiters of its emphasis, from one parse with the definitions.
   */
  const {
    prose,
    emphasis,
  } = proseViewOf({
    english,
    definitions,
  },);
  /**
   The marks after closing markup, read on the English as written, where the closing marks are not yet blanked.
   */
  const joined = joinedMarksOf({
    english,
    emphasis,
    definitions,
  },);
  /**
   The marks past closing markup (the trailing side only), read on the English as written for the same reason.
   */
  const pastMarkup = marksPastMarkupOf({
    english,
    emphasis,
    definitions,
  },);
  return {
    prose,
    proven: provenApostrophesOf({
      text: prose,
      attached: joined,
      pastMarkup,
    },),
    joined,
    readings: markReadingsOf({
      text: prose,
      attached: joined,
      pastMarkup,
    },),
  };
}

/**
 Whether the proof reads a mark as part of a word: attached, joined to the word after it, or proven at the edge of one.

 @internal

 @param reading - how the mark reads

 @param at - offset of the mark in the prose

 @param proven - the proven apostrophes of the prose

 @returns Whether the mark is part of a word

 @example
 ```ts
 const part = isPartOfAWord({ reading: 'trailing', at: 11, proven, },); // true when 11 is proven
 ```
 */
export function isPartOfAWord(
  {
    reading,
    at,
    proven,
  }: {
    readonly reading: MarkReading;
    readonly at: number;
    readonly proven: ProvenApostrophes;
  },
): boolean {
  if ((reading === 'attached') || (reading === 'joint'))
    return true;
  /**
   The proven marks that end a word and those that start one.
   */
  const {
    trailing,
    leading,
  } = proven;
  if (reading === 'trailing')
    return trailing.has(at,);
  if (reading === 'leading')
    return leading.has(at,);
  return false;
}

/**
 What the proof reads of an English, shaped into the view the pass remembers.

 @param english - the English of a page, or of the slice alone where no more is known

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: a reference
 in the English whose definition stands in another slice is then plain text, and the delimiters of its emphasis pair as plain
 text does (n173). An English that is the page itself holds its definitions already and needs none given

 @returns The prose, the proven apostrophes, the marks read as joined and the apostrophes that are part of a word

 @example
 ```ts
 const { joined, } = readProofView({ english: 'The *cats*’s bowl', definitions: '', },); // holds 10
 ```
 */
function readProofView(
  {
    english,
    definitions,
  }: {
    readonly english: string;
    readonly definitions: string;
  },
): ProofView {
  /**
   What the proof reads of the English.
   */
  const {
    prose,
    proven,
    joined,
    readings,
  } = readProof({
    english,
    definitions,
  },);
  return {
    prose,
    proven,
    joined,
    wordApostrophes: [...readings,]
      .filter(function partOfAWord([at, reading,]: ReadonlyEntry<number, MarkReading>,): boolean {
        return isPartOfAWord({
          reading,
          at,
          proven,
        },);
      },)
      .map(function wordApostropheAt([at,]: ReadonlyEntry<number, MarkReading>,): WordApostrophe {
        return {
          at,
          character: prose.charAt(at,),
        };
      },),
  };
}

/**
 The prose of an English and the apostrophes proven in it, read once for each page: asked again for an equal English, which
 a page with many places does, it gives the same view and neither parses nor proves the page again.

 @param english - the English of a page, or of the slice alone where no more is known

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: a reference
 in the English whose definition stands in another slice is then plain text, and the delimiters of its emphasis pair as plain
 text does (n173). An English that is the page itself holds its definitions already and needs none given

 @returns The prose, the proven apostrophes (trailing and leading apart) and the marks read as joined

 @example
 ```ts
 const { prose, proven, } = proofViewOf({ english: 'The parents’ `bowl`', },); // proven.trailing holds 11
 ```
 */
export function proofViewOf(
  {
    english,
    definitions = '',
  }: {
    readonly english: string;
    readonly definitions?: string;
  },
): ProofView {
  /**
   The view read before for an equal English read with equal definitions, none when there is none.
   */
  const known = remembered.find(function isSame(entry,): boolean {
    return (entry.english === english) && (entry.definitions === definitions);
  },);
  if (known !== undefined)
    return known.view;
  /**
   What the proof reads of the English.
   */
  const view = readProofView({
    english,
    definitions,
  },);
  remembered.push({
    english,
    definitions,
    view,
  },);
  if (remembered.length > REMEMBERED_VIEWS)
    remembered.shift();
  return view;
}

/**
 How every apostrophe-like mark of an English reads (attached, joined after closing markup, trailing, leading or free), by
 the reading the proof makes of it: the same prose, the same joints (marks that make one word of the letters on their
 two sides, `consistency-word-joints.ts`), the same neighbour test
 (`continuesLatinWordOrCompatibilityForm`). The English is read now and not
 remembered, since it is a text made for one splice and would push the page the pass is working on out of the memory
 (`proofViewOf`).

 @param english - the English, as one place holds it or as a page

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: a reference
 in the English whose definition stands in another slice is then plain text, and the delimiters of its emphasis pair as plain
 text does (n173). An English that is the page itself holds its definitions already and needs none given

 @returns The reading of each apostrophe-like mark by its offset in the English, in the order the marks stand

 @example
 ```ts
 const readings = markReadingsInEnglish({ english: 'The *cats*’s bowl.', },); // 10 reads as joint
 ```
 */
export function markReadingsInEnglish(
  {
    english,
    definitions = '',
  }: {
    readonly english: string;
    readonly definitions?: string;
  },
): ReadonlyMap<number, MarkReading> {
  /**
   How the proof reads each apostrophe-like mark of the English.
   */
  const { readings, } = readProof({
    english,
    definitions,
  },);
  return readings;
}

/**
 The span of the words, widened over a proven apostrophe directly beside either
 edge: a trailing one directly after the words' last character, a leading one
 directly before their first. No other single mark is taken in, and a mark that
 a possible partner anywhere on the page could pair with is not proven, so it
 stays outside.

 @param haystack - the English as the place holds it

 @param start - offset where the words start

 @param end - exclusive offset where they end

 @param page - the English of the whole page and where the haystack starts in it; the haystack alone is the page where absent

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`) the proof reads the English with, for a caller that
 holds the definitions and not the page: a reader of a recorded artifact, which stores the definitions a place was read with (generation
 19). A reference in the English whose definition stands in another slice is plain text without them, and the delimiters of its emphasis
 pair as plain text does, so a mark after closing emphasis reads as a leading mark where the page reads it as a joint of its word (n173);
 an English that is the page itself holds its definitions already and needs none given

 @returns The offsets of the span, the words and the proven apostrophes beside them

 @throws {@link import('./consistency-invariant-error.ts').ConsistencyInvariantError} when the page is too short to hold the haystack at its offset

 @example
 ```ts
 const span = withProvenApostrophes({ haystack: 'The parents’ bowl', start: 0, end: 11, },); // { start: 0, end: 12 }
 ```
 */
export function withProvenApostrophes(
  {
    haystack,
    start,
    end,
    page,
    definitions = '',
  }: {
    readonly haystack: string;
    readonly start: number;
    readonly end: number;
    readonly page?: PageEnglish;
    readonly definitions?: string;
  },
): {
  readonly start: number;
  readonly end: number;
} {
  // THE ENGLISH IS READ ONLY WHEN A SINGLE MARK STANDS BESIDE THE WORDS, since a
  // span with none beside it has no apostrophe to take in.
  if ((singleMarkAt({
    text: haystack,
    at: start - 1,
  },)
    .length
    === 0) && (singleMarkAt({
      text: haystack,
      at: end,
    },)
      .length
      === 0))
    return {
      start,
      end,
    };
  /**
   The English the proof reads, and where the haystack starts in it.
   */
  const {
    text,
    offset,
    shipped,
  } = pageReadingOf({
    english: haystack,
    ...((page === undefined) ? {} : { page, }),
  },);
  /**
   The proven apostrophes of the page, found by reading its prose once.
   */
  const {
    trailing,
    leading,
  } = provenApostrophesInEnglish({
    english: text,
    definitions,
  },);
  /**
   Where the character before the words stands on the page, none where a page-assembly pass changed the character there.
   */
  const before = positionOnPage({
    written: haystack,
    shipped: shipped ?? haystack,
    offset,
    at: start - 1,
  },);
  /**
   Where the character after the words stands on the page, none where a page-assembly pass changed the character there.
   */
  const after = positionOnPage({
    written: haystack,
    shipped: shipped ?? haystack,
    offset,
    at: end,
  },);
  return {
    start: ((before !== NO_PLACE) && leading.has(before,)) ? (start - 1) : start,
    end: ((after !== NO_PLACE) && trailing.has(after,)) ? (end + 1) : end,
  };
}

//endregion Consistency fold apostrophes
