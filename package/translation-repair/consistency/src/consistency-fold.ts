import {
  addressRangesOf,
  standsOutsideAddresses,
} from './consistency-addresses.ts';
import { closingJointsOf, } from './consistency-closing-links.ts';
import {
  edgeFreeRangeOf,
  edgeFreeWordsOf,
} from './consistency-edge-marks.ts';
import { foldedLinksOf, } from './consistency-folded-links.ts';
import { ConsistencyInvariantError, } from './consistency-invariant-error.ts';
import {
  isSingleMark,
  singleMarkAt,
} from './consistency-loose-marks.ts';
import { withProvenApostrophes, } from './consistency-fold-apostrophes.ts';
import type { CodePadding, } from './consistency-markup-ranges.ts';
import type { PageEnglish, } from './consistency-page-english.ts';
import {
  foldWithOffsets,
  type OffsetFoldedText,
  offsetsInFold,
  writtenPointsOf,
} from './consistency-fold-offsets.ts';
import { emphasisDelimitersOf, } from './consistency-prose-text.ts';
import { cutsShownWord, } from './consistency-shown-edge.ts';
import { cutsWord, } from './consistency-word-joints.ts';
import { wholeSpanOf, } from './consistency-whole-end.ts';
import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordStarts, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Consistency fold
// WHERE A JUDGE'S QUOTE STANDS IN A PLACE'S OWN ENGLISH. Judges copy the words
// of an English passage with the line break the page wraps at written as a
// space, a curly apostrophe written straight, the closing full stop kept or
// left off, and two quotes of one passage that differ in only those still name
// one span. This reads a quote and the English in one fold (the evidence fold
// of `quote-normalize.ts`, a run of whitespace as one space, a blank line kept
// apart from a space because it is a paragraph boundary), ignores sentence
// punctuation, quotation marks and brackets that the words do not need, and
// space at the quote's two edges (`consistency-edge-marks.ts`), and answers
// where the quote stands in the English as the English holds it, at the
// boundaries of words (`word-bounds.ts`). The span the English holds is what
// every later step uses, never the judge's string. A STAND is one place where the folded
// words are found in the English, from a start offset to an end offset at word
// boundaries (the `once` of a `SpanSearch` is the one stand); a quote can have no stand,
// one or several, and a stand that a joint of a word cuts (a mark that makes one word of what
// stands on its two sides, `consistency-word-joints.ts`) is no stand.
//
// A WORD HERE IS ALSO JOINED BY ITS OWN MARKS, for this locator alone (the
// glossary's reading in `word-bounds.ts` is unchanged): a hyphen or an
// apostrophe between two letters or digits (a Latin letter with its accents or an ASCII
// digit, `latin-letters.ts`), and a full stop, a comma, a colon or
// a slash between two digits, join them into one word, so `so` is no stand inside
// `so-called`, `t` none inside `don't`, `dog` none inside `dog's`, `2 tall` none
// inside `5'2 tall` and `5 dollars` none inside `0.5 dollars`. An apostrophe with
// nothing joined before it (`'90s`, `'em`, `'Tis`) joins nothing. An apostrophe after
// markup that ends in a letter or a digit, or after a closing bracket or closing curly
// double quotation mark, joins the same way: `the **cat**` is no stand inside
// `the **cat**'s bowl` and a link's text none inside `[the cat](x)'s`. A Greek letter, a
// Han character or an emoji beside a word joins nothing, as before. Which marks join,
// and how emphasis marks beside one are read, is `consistency-word-joints.ts`.
//
// A STAND INSIDE A WEB OR MAIL ADDRESS IS NO STAND. A carry that replaced `cats` in
// `https://cats.example.org/page` would edit the link, so a stand that lies wholly
// inside an address is dropped before the stands are counted: `cats` is found once
// in `Visit [the cats](https://cats.example.org/page) now.`, in the link's text,
// and nowhere in `Visit https://cats.example.org/page now.`. What an address is
// (a run holding `://` from its scheme, a run starting with `www.`, a run holding
// an `@` with a letter or a digit on each side and a full stop after it) is
// `consistency-addresses.ts`. THE LOCATOR SEES TEXT ONLY: its callers hand it the
// English of a slice and none holds a parse of it, so a Markdown link
// destination that is no address (`[cats](cats-page)`) is not read here.
//
// AN EDGE OF THE SPAN NEVER FALLS INSIDE A BACKSLASH ESCAPE OR A CHARACTER REFERENCE. A quote that begins or ends between a
// backslash and the character it escapes, or inside a reference the parser decodes (`amp; cats`, `the cats` and a backslash),
// names the whole unit, which is then kept or cut at the edge as the character it shows would be (`consistency-whole-end.ts`,
// `consistency-edge-units.ts`): a replacement never writes half of one.
//
// THE SPAN IS THE WORDS LOOKED FOR AND THE PROVEN APOSTROPHE BESIDE THEM, no
// other character. The words carry no single mark at their edges, paired with
// anything or not (`consistency-edge-marks.ts`), so `the parents’` and `the parents`
// name the same words. A single mark the English holds just outside where they
// stand is taken into the span only when it is PROVEN an apostrophe: a trailing
// mark that no earlier quotation-like character of the page's prose could have
// opened a quotation for (`the parents’`), a leading mark that no later one could
// close a quotation for (`’round the`), whether or not any judge copied it
// (`consistency-proven-apostrophes.ts`, which states the rule, the corpus counts it
// rests on and the page-wide reading). Such a mark is part of its word, is carried
// in with the chosen words and is removed with the replaced ones. Every other single
// mark stays where the English has it when the span is replaced: the mark that opens
// or closes a quotation, an apostrophe with a possible partner, and any `‘`.
// Telling those apart takes a guess about the English around the words, and a span
// widened over a mark guessed wrong deletes a quotation mark from the page, so the
// guess is not made: a mark with a possible partner is the case this file leaves to
// the refusal. Two places whose words are the same and whose English differs only by
// such a mark hold the same words, and whether a replacement can carry those words
// between them without writing a pairing of words and marks neither place wrote is
// the splice's question (`consistency-splice.ts`), answered from the marks the located
// rendering records beside its span. THE PROOF READS THE WHOLE PAGE: the locator finds the
// words in the English of one slice, and a caller that holds the English of the page hands
// it, with the offset where the slice starts in it (`page`, `consistency-page-english.ts`),
// so a quotation that opens in one slice and closes in another is seen; a caller that hands
// no page gets the slice as the page. The stretches of the page that the Markdown grammar
// does not read as words (code, the strings of a component, comments, link destinations)
// hold no partner (`consistency-prose-text.ts`).
//
// A SINGLE MARK THE QUOTE HELD BESIDE ITS WORDS CHOOSES BETWEEN STANDS, and is
// span content only once a stand is chosen and the mark is a proven apostrophe.
// The choice reads the English as it always did: a proven apostrophe is a single
// mark directly beside the words like any other, so a stand that holds one and a
// stand of the same words that holds none are told apart by the evidence and by
// nothing else. With evidence of a mark on that side the stand with the apostrophe
// is the place, and the span is then widened over it; with none both stands stay
// and the place is ambiguous, the apostrophe choosing nothing.
// When the words stand more than once and there is mark
// evidence for a side (the judges held a single mark directly beside the
// words on it, with no character between the mark and the words), only the
// stands where the English holds a single mark directly beside the words on
// that side are kept, and exactly one left is the place; none or several left
// is ambiguous. Without evidence nothing changes. The caller can give the
// evidence per side (`marks`), which is how a count of many judges' quotes
// reaches the locator (`consistency-majority.ts`, `markEvidenceOf`), since the
// folded words it hands over hold no mark; a caller that hands over a judge's
// own quote and no evidence gets the evidence that quote holds by itself. This
// is what keeps `the cats'` a single place in `Fill the cats' bowl. Then the
// cats nap.`
//
// THE FOLD IS NOT ONE UNIT TO ONE, since a run of whitespace folds to one or two
// units, so the folded English carries a map from each folded unit back to the
// offsets of the original it stands for, and every offset returned indexes the
// English as it was given (the fold itself is `consistency-fold-offsets.ts`).
//
// FOLDING A FOLDED TEXT CHANGES NOTHING. A blank line folds to TWO line breaks,
// which read again are two breaks and so a blank line again, where one break
// stands for a soft wrap and folds to a space; a quote that spans a paragraph
// boundary therefore locates whether its caller folded it first or not.

/**
 Where a quote stands in an English.

 @example
 ```ts
 const found: SpanSearch = { kind: 'once', start: 4, end: 11, };
 ```
 */
export type SpanSearch =
  | {
    /**
     The quote's words stand nowhere in the English.
     */
    readonly kind: 'none';
  }
    | {
    /**
     The quote's words stand exactly once.
     */
      readonly kind: 'once';

    /**
     Offset in the English where the span starts.
     */
      readonly start: number;

    /**
     Exclusive offset in the English where the span ends.
     */
      readonly end: number;
    }
    | {
    /**
     The quote's words stand more than once, overlaps included.
     */
      readonly kind: 'many';
    };

/**
 The words of a quote in the fold, without the sentence punctuation, space and
 invisible characters at its two edges, without any single mark at an edge
 (paired with something inside the words or not), and without a double
 quotation mark or a bracket at an edge that pairs with nothing inside the
 words or encloses all of them. A backslash escape and a character reference the
 parser decodes are each cut or kept whole, as the character they show would be.
 Reading the result again gives the result.

 @param text - quote as a judge gave it, or the folded words of one

 @returns The folded words, empty when the quote shows a reader nothing

 @example
 ```ts
 const words = foldedWordsOf({ text: '“So she was a girl, wasn’t she?”', },); // "So she was a girl, wasn't she"
 ```
 */
export function foldedWordsOf({ text, }: { readonly text: string; },): string {
  return edgeFreeWordsOf(writtenPointsOf({ folded: foldWithOffsets({ text, },), },),)
    .join('',);
}

/**
 On which sides a single mark stood directly beside some words: before them,
 after them, or both.

 @example
 ```ts
 const marks: MarksBeside = { before: false, after: true, };
 ```
 */
export type MarksBeside = {
  /**
   Whether a single mark stood directly before the words.
   */
  readonly before: boolean;

  /**
   Whether a single mark stood directly after the words.
   */
  readonly after: boolean;
};

/**
 A quote read for the words it names and the single marks it held directly
 beside them.
 */
type ReadQuote = {
  /**
   The folded words, without the marks and punctuation at the quote's edges.
   */
  readonly needle: string;

  /**
   Whether the quote held a single mark directly before the words: the
   character just before them, as the judge wrote it, is one.
   */
  readonly heldBefore: boolean;

  /**
   Whether the quote held a single mark directly after the words.
   */
  readonly heldAfter: boolean;
};

/**
 Reads a quote for its words and for whether a single mark stood directly
 beside them on either side.

 @param text - quote as a judge gave it, or the folded words of one

 @returns The words and the two sides a single mark stood directly beside

 @example
 ```ts
 const quote = readQuote({ text: 'the cats\'', },); // { needle: 'the cats', heldBefore: false, heldAfter: true }
 ```
 */
function readQuote({ text, }: { readonly text: string; },): ReadQuote {
  /**
   The quote in the fold with the character each unit stood for.
   */
  const folded = foldWithOffsets({ text, },);
  /**
   The folded quote, one whole character each, and the characters as the judge wrote them.
   */
  const {
    points,
    originals,
  } = writtenPointsOf({ folded, },);
  /**
   Where the words stand among the characters.
   */
  const {
    from,
    to,
  } = edgeFreeRangeOf({
    points,
    originals,
  },);
  /**
   The words.
   */
  const needle = points.slice(
    from,
    to,
  )
    .join('',);
  /**
   The folded units before the words.
   */
  const before = points.slice(
    0,
    from,
  )
    .join('',);
  /**
   Offset in the folded units where the words start, which differs from `from` after a character beyond the first plane.
   */
  const startUnit = before.length;
  return {
    needle,
    heldBefore: isSingleMark({ character: folded.originals[startUnit - 1] ?? '', },),
    heldAfter: isSingleMark({ character: folded.originals[startUnit + needle.length] ?? '', },),
  };
}

/**
 On which sides a judge's quote held a single mark directly beside its words:
 the character just outside the words, as the judge wrote it, is one of the
 four single marks. A space or a line break between the mark and the words is
 a character between, and a double quotation mark is no single mark.

 @param text - quote as a judge gave it

 @returns Whether a single mark stood directly before the words and directly after them

 @example
 ```ts
 const marks = marksHeldBesideWords({ text: 'the cats\'', },); // { before: false, after: true }
 ```
 */
export function marksHeldBesideWords({ text, }: { readonly text: string; },): MarksBeside {
  /**
   The quote read for its words and the marks beside them.
   */
  const {
    heldBefore,
    heldAfter,
  } = readQuote({ text, },);
  return {
    before: heldBefore,
    after: heldAfter,
  };
}

/**
 The offsets in an English of the stretch a stand of the needle covers.

 @param folded - the English in the fold, with the character each unit stood for

 @param at - where the stand starts in the folded English

 @param length - length of the needle

 @returns Where the stand starts in the English as it was given and the exclusive offset where it ends

 @throws {@link ConsistencyInvariantError} when a position of the stand has no
 offsets in the original, since every folded unit stands for a stretch of it

 @example
 ```ts
 const { start, end, } = standSpan({ folded: foldWithOffsets({ text: 'The cat', },), at: 4, length: 3, },); // { start: 4, end: 7 }
 ```
 */
function standSpan(
  {
    folded,
    at,
    length,
  }: {
    readonly folded: OffsetFoldedText;
    readonly at: number;
    readonly length: number;
  },
): {
  readonly start: number;
  readonly end: number;
} {
  /**
   Offset in the English where the first folded unit's stretch starts.
   */
  const start = folded.starts[at];
  /**
   Exclusive offset in the English where the last folded unit's stretch ends.
   */
  const end = folded.ends[(at + length) - 1];
  if ((start === undefined) || (end === undefined)) {
    throw new ConsistencyInvariantError({ invariant:
      `a folded position ${String(at,)} of the English has no offsets in it, and every folded unit stands for a stretch of it`, },);
  }
  return {
    start,
    end,
  };
}

/**
 The stands of a needle that hold a single mark directly beside the words on
 each side the quote held one on. A quote that held none on either side keeps
 every stand.

 @param haystack - the English as the place holds it

 @param folded - the English in the fold

 @param stands - start offsets of the stands in the folded English

 @param needle - the folded words looked for

 @param quote - the quote the stands were looked for by

 @returns The stands that fit, in the order they were given

 @throws {@link ConsistencyInvariantError} when a stand has no offsets in the original

 @example
 ```ts
 const kept = standsBesideMarks({ haystack, folded, stands, needle, quote: readQuote({ text: 'the cats\'', },), },);
 ```
 */
function standsBesideMarks(
  {
    haystack,
    folded,
    stands,
    needle,
    quote,
  }: {
    readonly haystack: string;
    readonly folded: OffsetFoldedText;
    readonly stands: readonly number[];
    readonly needle: string;
    readonly quote: ReadQuote;
  },
): readonly number[] {
  return stands.filter(function holdsTheMarks(at,): boolean {
    /**
     Where the stand lies in the English.
     */
    const {
      start,
      end,
    } = standSpan({
      folded,
      at,
      length: needle.length,
    },);
    /**
     The single mark directly before the stand, as a list of zero or one.
     */
    const marksBefore = singleMarkAt({
      text: haystack,
      at: start - 1,
    },);
    /**
     The single mark directly after it.
     */
    const marksAfter = singleMarkAt({
      text: haystack,
      at: end,
    },);
    return ((!quote.heldBefore) || (marksBefore.length > 0)) && ((!quote.heldAfter) || (marksAfter.length > 0));
  },);
}

/**
 Every start of a needle in a folded English where it stands at the boundaries
 of words and neither of its edges is a joint of a longer word.

 @param haystack - the English as it was given, which the delimiters of its emphasis are read from by the parse

 @param folded - the English in the fold, with the character each unit stood for

 @param needle - folded words looked for

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for an English read alone: a reference with an empty label is
 read as a link where a definition of it stands in the English or here, as the page shows it

 @returns Start offsets in the folded English, in ascending order

 @example
 ```ts
 const text = 'the so-called cat';
 const starts = apartStartsOf({ haystack: text, folded: foldWithOffsets({ text, },), needle: 'so', definitions: '', },); // []
 ```
 */
function apartStartsOf(
  {
    haystack,
    folded,
    needle,
    definitions,
  }: {
    readonly haystack: string;
    readonly folded: OffsetFoldedText;
    readonly needle: string;
    readonly definitions: string;
  },
): readonly number[] {
  /**
   The stands at the boundaries of words, whether or not they are part of a longer word.
   */
  const starts = wordStarts({
    text: folded.folded,
    needle,
    end: 'word',
  },);
  if (starts.length === 0)
    return starts;
  /**
   The delimiters of the English's emphasis by the parse, at the offsets of the English as it was given.
   */
  const delimiters = emphasisDelimitersOf({
    english: haystack,
    definitions,
  },);
  /**
   The links of the English whose label shows nothing, closing marks or marks, at the offsets of the fold, none where no closing
   square bracket stands in the fold.
   */
  const links = foldedLinksOf({
    english: haystack,
    folded,
    definitions,
  },);
  /**
   Where a stand may end inside a word made with an apostrophe after closing marks, read once for the whole English.
   */
  const joints = closingJointsOf({
    folded: folded.folded,
    originals: folded.originals,
    closing: offsetsInFold({
      offsets: delimiters.closing,
      folded,
    },),
    links,
  },);
  /**
   Every delimiter of emphasis, at the offsets of the fold.
   */
  const emphasis = offsetsInFold({
    offsets: delimiters.offsets,
    folded,
  },);
  /**
   The stands at the boundaries of words that are no part of a longer word.
   */
  const apart = starts.filter(function standsApart(at,): boolean {
    return !cutsWord({
      folded: folded.folded,
      originals: folded.originals,
      joints,
      emphasis,
      links,
      at,
      end: at + needle.length,
    },);
  },);
  if (apart.length === 0)
    return apart;
  return standsOutsideAddresses({
    stands: apart,
    length: needle.length,
    addresses: addressRangesOf({ text: folded.folded, },),
  },);
}

/**
 Where a quote's words stand in an English, once, never or more than once, at
 the boundaries of words: a Latin letter or digit at the first or last
 character of the words may not have another beside it in the English as it is written (and, once the final edges are known,
as a reader sees it: below), so `so`
 is not found inside `also`, and an edge of the words that is a joint of a
 longer word (a hyphen or an apostrophe between two letters or digits, or after
 markup that ends in a letter or a digit; a full stop, a comma, a colon or a
 slash between two digits) is no boundary, so `so` is not found inside
 `so-called`. Han text needs no boundary.

 The span is the words and the proven apostrophe directly beside them, if any:
 a trailing apostrophe after their last character that no earlier quotation-like
 character of the page's prose could have opened a quotation for (`the parents’`),
 a leading one before their first that no later one could close a quotation for
 (`’round the`), whether or not the quote copied it (`consistency-proven-apostrophes.ts`).
 Every other single mark the English holds just outside the words, quotation mark
 or apostrophe, is never inside the span. When the words stand more than once
 and there is mark evidence for a side, only the stands with a single mark
 directly beside them on that side are kept, a proven apostrophe being one such
 mark and no better than another; the span is widened over a proven apostrophe
 only once the one stand is chosen. An edge of the stand that falls inside a backslash escape or a character reference the parser
 decodes is taken to the edge of that unit and cut again as the character it shows would be (`wholeSpanOf`), before any apostrophe
 is read. The word test is then run again on those final edges, reading the text a reader sees (`cutsShownWord`,
 `consistency-shown-edge.ts`: a reference or an escape is its shown character, a default-ignorable character is nothing, an
 ampersand between two word characters joins them like a hyphen), and an edge that cuts a shown word names no span (`none`);
 the count of stands (`standsOf`) is not changed by it. The evidence is the one the caller gives, or,
 when it gives none, what the quote handed over held directly beside its words by
 itself.

 @param haystack - the English as the place holds it

 @param words - quote as a judge gave it, or the folded words of one

 @param marks - on which sides the evidence holds, for a caller whose words hold no mark because many judges' quotes were folded into them; the quote's own marks are the evidence when absent

 @param apostrophes - `proven` to take the proven apostrophes beside the words into the span, `outside` to leave the span at the words alone, as the slot of a frame is found inside a span that already holds its apostrophes

 @param page - the English of the whole page the haystack belongs to and where the haystack starts in it, which the proof of an apostrophe reads; the haystack alone is the page where absent

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`) in place of those read from `page`, for a caller that holds the definitions and not the page: a reader of a recorded artifact, which stores the definitions a place was read with (from generation 18, for a slot); read from `page` where absent, and handed to the proof of the apostrophe beside the words as well as to the word test

 @param codePadding - what pads the content of inline code when the word test reads the text a reader sees: a space and a line ending
 as CommonMark reads it, or a literal space alone as a reader of an artifact written before line endings padded code reads it; a
 space and a line ending where absent

 @returns That the words stand nowhere, the offsets of the one span they name
 in the English, or that they stand more than once and no mark evidence
 chooses one of the stands (a stand inside a longer word is no stand);
 nowhere for a quote that shows a reader nothing

 @throws {@link ConsistencyInvariantError} when a found position has no offsets
 in the original, since every folded unit stands for a stretch of it

 @example
 ```ts
 const found = locateWords({ haystack: 'The cat naps.', words: 'cat naps.', },); // { kind: 'once', start: 4, end: 12 }
 ```
 */
export function locateWords(
  {
    haystack,
    words,
    marks,
    apostrophes = 'proven',
    page,
    definitions: given,
    codePadding,
  }: {
    readonly haystack: string;
    readonly words: string;
    readonly marks?: MarksBeside;
    readonly apostrophes?: 'proven' | 'outside';
    readonly page?: PageEnglish;
    readonly definitions?: string;
    readonly codePadding?: CodePadding;
  },
): SpanSearch {
  /**
   The words to look for, and the single marks the quote held directly beside them.
   */
  const read = readQuote({ text: words, },);
  /**
   The quote with the evidence the caller gave in place of the marks it held itself.
   */
  const quote: ReadQuote = (marks === undefined)
    ? read
    : {
      ...read,
      heldBefore: marks.before,
      heldAfter: marks.after,
    };
  /**
   The words to look for.
   */
  const { needle, } = quote;
  if (needle === '')
    return { kind: 'none', };
  /**
   The English in the fold with its offsets.
   */
  const folded = foldWithOffsets({ text: haystack, },);
  /**
   The link and footnote definitions of the page, which a reference in the English resolves against whichever slice holds the
   definition: the ones the caller holds, else those of the page; none for an English read alone.
   */
  const definitions = given ?? ((page === undefined) ? '' : pageDefinitionsOf({ pageText: page.text, },));
  /**
   Where the words stand in the folded English, at word boundaries and at no joint of a longer word, overlaps included.
   */
  const stands = apartStartsOf({
    haystack,
    folded,
    needle,
    definitions,
  },);
  if (stands.length === 0)
    return { kind: 'none', };
  /**
   The stands that remain once a single mark the quote held beside the words chooses among several.
   */
  const [at, ...others] = (stands.length > 1)
    ? standsBesideMarks({
      haystack,
      folded,
      stands,
      needle,
      quote,
    },)
    : stands;
  if ((at === undefined) || (others.length > 0))
    return { kind: 'many', };
  /**
   Where the one stand lies in the English, the words alone, taken whole where an edge of the stand falls inside a backslash escape or
   a character reference and then cut at the edge as the character that unit shows would be.
   */
  const bare = wholeSpanOf({
    text: haystack,
    ...standSpan({
      folded,
      at,
      length: needle.length,
    },),
  },);
  // The word test again on the edges the span has now, reading the text a reader sees (n156, W2): an edge that cuts a shown word
  // names no span.
  if (cutsShownWord({
    text: haystack,
    start: bare.start,
    end: bare.end,
    needle,
    definitions,
    ...((codePadding === undefined) ? {} : { codePadding, }),
  },))
    return { kind: 'none', };
  return {
    kind: 'once',
    ...((apostrophes === 'proven')
      ? withProvenApostrophes({
        haystack,
        ...bare,
        ...((page === undefined) ? {} : { page, }),
        ...((given === undefined) ? {} : { definitions: given, }),
      },)
      : bare),
  };
}

/**
 How many times a quote's words stand in an English, at word boundaries and at
 no joint of a longer word, overlaps included, read in the same fold as
 {@link locateWords}. It counts every stand, whatever single marks stand beside
 it, and so does not choose among them as {@link locateWords} does.

 @param haystack - the English as the place holds it

 @param words - quote as a judge gave it, or the folded words of one

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: a reference
 in the English resolves against them, whichever slice holds the definition (n173)

 @returns The number of stands; none for a quote that shows a reader nothing

 @example
 ```ts
 const stands = standsOf({ haystack: 'The cat naps. The cat naps.', words: 'The cat naps', },); // 2
 ```
 */
export function standsOf(
  {
    haystack,
    words,
    definitions = '',
  }: {
    readonly haystack: string;
    readonly words: string;
    readonly definitions?: string;
  },
): number {
  return apartStartsOf({
    haystack,
    folded: foldWithOffsets({ text: haystack, },),
    needle: foldedWordsOf({ text: words, },),
    definitions,
  },)
    .length;
}

//endregion Consistency fold
