import {
  apostropheMarksOf,
  withApostropheForm,
} from './consistency-loose-marks.ts';
import { proofViewOf, } from './consistency-fold-apostrophes.ts';
import {
  NO_PLACE,
  type PageEnglish,
  pageReadingOf,
  positionOnPage,
} from './consistency-page-english.ts';

//region Consistency apostrophe forms
// THE FORM AN APOSTROPHE ARRIVES IN. An apostrophe that comes with the chosen words
// (one inside a word, like `dog's`, or a proven one at an edge, like `dogs’`) goes
// into a place that writes its apostrophes in a form of its own, and a page that
// mixes the straight and the curly form reads as careless. So the apostrophes that
// arrive are written in the receiving place's form: the form the apostrophes of the
// receiving words share when they all share one; otherwise the form the apostrophes
// of the page's prose outside the span share when they all share one; otherwise as the
// chosen place wrote them. THE PAGE IS THE ONE THE PROOF READS, every slice of it
// (`consistency-page-english.ts`), through the same prose view and the same proof
// (`consistency-fold-apostrophes.ts`), so a slice that holds no apostrophe of its own takes
// the form the rest of its page writes; the chosen words that arrive are left out of that
// reading, since their apostrophes are the ones being written. THE APOSTROPHES A PAGE WRITES, for the form, are the ones
// the proof reads as part of a word: a mark with a Latin letter (with its accent) or an ASCII digit on both sides, a
// proven one at the edge of a word, and a mark the proof reads as joined to the word after it across closing markup
// (`*Mio*’s`, `[Mio](url)’s`, `(Mio)’s`, `"Mio"’s`), which the prose view cannot show since it has blanked the markup.
// The straight apostrophe, the closing curly mark and the
// modifier letter apostrophe (U+02BC) are three forms. Nothing is guessed: a page of
// mixed forms has no form to follow, and the words keep the chosen place's. Only the
// apostrophes of words are rewritten here; the loose marks of the words are written in
// the form of the place they go to by `withLooseMarksOf` (`consistency-loose-marks.ts`).
// What a reader of the record compares is unaffected: the words it reads fold the three
// forms to one (`quote-normalize.ts`), and the plan kind it recomputes from the
// recorded words depends on the receiving words' own apostrophes alone.

/**
 The one form a list of apostrophes shares.

 @param marks - apostrophes as written, in order

 @returns The shared form as a list of one, none for an empty list and for marks of more than one form

 @example
 ```ts
 const [form,] = sharedFormOf({ marks: ['’', '’',], },); // '’'
 ```
 */
function sharedFormOf({ marks, }: { readonly marks: readonly string[]; },): readonly string[] {
  /**
   The first mark, whose form the others must share.
   */
  const [first,] = marks;
  return ((first !== undefined) && marks.every(function isFirstForm(mark,): boolean {
    return mark === first;
  },))
    ? [first,]
    : [];
}

/**
 One stretch of the page, by the offsets of the page's own English.
 */
export type PageStretch = {
  /**
   Offset where the stretch starts.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends.
   */
  readonly end: number;
};

/**
 The one form the apostrophes of a page's prose outside a span share: the marks a
 Latin letter or an ASCII digit attaches on both sides, the marks the proof reads as joined to the
 word after them across closing markup (`*Mio*’s`), and the proven ones
 (`consistency-proven-apostrophes.ts`), whatever the stretches the page does not read
 as words hold. The page is the one the proof reads (`consistency-page-english.ts`),
 every slice of it, so a slice that holds no apostrophe outside the span takes the form
 the rest of its page writes. The apostrophes of a page are listed once for the page
 (`wordApostrophes` of `proofViewOf`, remembered for an equal English), so asking for each place
 of a page filters that list and neither parses the page nor walks it again.

 @param english - the English of the slice as the place holds it

 @param start - offset in the slice where the span starts

 @param end - exclusive offset in the slice where it ends

 @param page - the English of the whole page and where the slice starts in it; the slice alone is the page where absent

 @param apart - stretches of the page, by its own offsets, that are not read: the chosen words that arrive in the span, whose
 apostrophes are the ones being written in the page's form and say nothing of it; none by default

 @returns The shared form as a list of one, none where the page holds no apostrophe outside the span or holds more than one form

 @throws {@link import('./consistency-invariant-error.ts').ConsistencyInvariantError} when the page is too short to hold the slice at its offset

 @example
 ```ts
 const form = pageApostropheFormOf({ english: 'Fill the family bowl. The cat’s dish.', start: 5, end: 15, },); // ['’']
 ```
 */
export function pageApostropheFormOf(
  {
    english,
    start,
    end,
    page,
    apart = [],
  }: {
    readonly english: string;
    readonly start: number;
    readonly end: number;
    readonly page?: PageEnglish;
    readonly apart?: readonly PageStretch[];
  },
): readonly string[] {
  /**
   The English the proof reads, and where the slice starts in it.
   */
  const {
    text,
    offset,
    shipped,
  } = pageReadingOf({
    english,
    ...((page === undefined) ? {} : { page, }),
  },);
  /**
   Where the span starts on the page, none where a page-assembly pass changed the character there.
   */
  const spanStart = positionOnPage({
    written: english,
    shipped: shipped ?? english,
    offset,
    at: start,
  },);
  /**
   Where the span's last character stands on the page, none where a page-assembly pass changed it.
   */
  const spanLast = positionOnPage({
    written: english,
    shipped: shipped ?? english,
    offset,
    at: end - 1,
  },);
  /**
   The whole slice on the page, which stands in for a span that has no place there: it reads fewer apostrophes as the page's
   own and can only leave no form shared.
   */
  const wholeSlice: PageStretch = {
    start: offset,
    end: offset + (shipped ?? english).length,
  };
  /**
   Where the span stands on the page.

   @returns The span's own stretch, else the whole slice
   */
  function spanOnPage(): PageStretch {
    if (spanStart === NO_PLACE)
      return wholeSlice;
    if (end <= start) {
      return {
        start: spanStart,
        end: spanStart,
      };
    }
    if (spanLast === NO_PLACE)
      return wholeSlice;
    return {
      start: spanStart,
      end: spanLast + 1,
    };
  }
  /**
   The apostrophes the page writes as part of a word, read once for the page (`proofViewOf` gives the one view for an equal
   English), so asking for each place of a page filters a short list and does not walk the page.
   */
  const { wordApostrophes, } = proofViewOf({ english: text, },);
  /**
   The stretches not read: the span in the page, and the stretches named.
   */
  const unread: readonly PageStretch[] = [
    spanOnPage(),
    ...apart,
  ];
  return sharedFormOf({
    marks: wordApostrophes
      .filter(function standsOutside({ at, },): boolean {
        return !unread.some(function holdsMark(stretch,): boolean {
          return (at >= stretch.start) && (at < stretch.end);
        },);
      },)
      .map(function characterOf({ character, },): string {
        return character;
      },),
  },);
}

/**
 The words that go in with their apostrophes written in the receiving place's form.

 @param text - the words that go in, with their loose marks already written

 @param receiving - the receiving words

 @param page - the form the page's prose outside the receiving words shares, as a list of zero or one, asked only when the receiving words share no form; absent for a caller that holds no English

 @returns The words with every apostrophe of a word in the form the receiving words share, else the page's, else as written

 @example
 ```ts
 const written = inReceivingForm({ text: 'the dogs\'', receiving: 'the kittens’', },); // 'the dogs’'
 ```
 */
export function inReceivingForm(
  {
    text,
    receiving,
    page,
  }: {
    readonly text: string;
    readonly receiving: string;
    readonly page?: () => readonly string[];
  },
): string {
  if (apostropheMarksOf({ text, },)
    .length
    === 0)
    return text;
  /**
   The form the receiving words share, none when they hold no apostrophe of a word or hold more than one form.
   */
  const own = sharedFormOf({ marks: apostropheMarksOf({ text: receiving, },), },);
  /**
   The form to write in, the receiving words' first and the page's otherwise.
   */
  const [form,] = (own.length > 0) ? own : sharedFormOf({ marks: (page === undefined) ? [] : page(), },);
  return (form === undefined) ? text : withApostropheForm({
    text,
    form,
  },);
}

//endregion Consistency apostrophe forms
