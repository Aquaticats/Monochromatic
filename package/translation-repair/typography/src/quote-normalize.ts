import {
  noteRefusedMask,
  textNamed,
} from './typography-mask-refusal.ts';
import { proseMaskReading, } from './typography-prose-mask.ts';

//region Quote normalization
// Models copy quote evidence with ASCII punctuation while the corpus uses
// curly variants (the en editing guide mandates them), so byte-exact
// location rejects real evidence: on Xu_Yushu, most unresolved issues
// failed exactly this way ("father's" vs "father’s"). Normalization maps
// punctuation variants onto one canonical character, strictly one-to-one in
// UTF-16 units, so an offset found in normalized text indexes the original
// text unchanged and anchors keep the document's canonical bytes.
//
// TWO FOLDS, FOR TWO QUESTIONS (ledger B24). The EVIDENCE fold
// (`normalizePunctuation`) asks whether a model's quote is the document's
// words: it also folds the corner brackets, because a model quoting a Chinese
// passage paraphrases them. The TYPOGRAPHY fold (`straightenQuotes`) asks
// whether two renderings are the same wording up to what the typography
// restoration (`restore-typography.ts`) would make of them: curly and straight
// only, since a rendering that kept 「」 and one that wrote English quotes are
// different renderings, one of them wrong. A comparison of a quote against its
// source takes the first; a comparison of two renderings takes the second.
//
// MAPS, NOT PLAIN OBJECTS (ledger B77). Every key here is one UTF-16 unit, and
// no name an object inherits is one unit long, so no lookup could reach one;
// the tables are maps all the same, so every table the package keys by text
// has one shape and `text-keyed-tables.unit.test.ts` needs no exceptions.

/**
 Punctuation variants mapped onto canonical ASCII, one UTF-16 unit each.
 CJK corner brackets join the quote classes:
 models paraphrase 「」 as curly or ASCII quotes when quoting zh sources
 (live: a model closing 「...。」 as ...。”), and every bracket here is a
 single UTF-16 unit, so the length guarantee holds.
 */
const PUNCTUATION_CANON: ReadonlyMap<string, string> = new Map([
  [
    '‘',
    "'",
  ],
  [
    '’',
    "'",
  ],
  [
    '“',
    '"',
  ],
  [
    '”',
    '"',
  ],
  [
    '「',
    '"',
  ],
  [
    '」',
    '"',
  ],
  [
    '『',
    "'",
  ],
  [
    '』',
    "'",
  ],
  [
    '\u00A0',
    ' ',
  ],
],);

/**
 Curly quotation marks mapped onto their straight forms, one UTF-16 unit
 each: the typography fold. The corner brackets stay as they are, since
 keeping them is a rendering choice rather than a typography one.
 */
const TYPOGRAPHY_CANON: ReadonlyMap<string, string> = new Map([
  [
    '‘',
    '\'',
  ],
  [
    '’',
    '\'',
  ],
  [
    '“',
    '"',
  ],
  [
    '”',
    '"',
  ],
],);

/**
 Line-break units a model returns as a plain space when it quotes across a
 soft wrap.
 Deliberately NOT part of `PUNCTUATION_CANON`:
 collapsing these changes which quotes match, so admitting them is a
 behaviour change awaiting a decision, and only the diagnostic path may
 consult this map today.
 */
const LINE_BREAK_CANON: ReadonlyMap<string, string> = new Map([
  [
    '\n',
    ' ',
  ],
  [
    '\r',
    ' ',
  ],
],);

/**
 Rewrites each UTF-16 unit through one canonicalization map.
 Length-preserving by construction, since every mapping replaces one unit
 with one unit, so a position found in the result indexes the input exactly.

 @param text - text whose units canonicalize

 @param map - canonical replacement per unit, absent units left alone

 @returns Same-length text with mapped units replaced

 @example
 ```ts
 canonicalize({ text: 'father’s', map: PUNCTUATION_CANON, },);
 ```
 */
function canonicalize(
  {
    text,
    map,
  }: {
    readonly text: string;
    readonly map: ReadonlyMap<string, string>;
  },
): string {
  /**
   Canonicalized units in input order.
   */
  const units: string[] = [];
  for (
    let index = 0;
    index < text.length;
    index += 1
  ) {
    /**
     Unit at this position.
     */
    const unit = text.charAt(index,);
    units.push(map.get(unit,) ?? unit,);
  }
  return units.join('',);
}

/**
 Normalizes punctuation variants onto canonical characters: the evidence
 fold, under which a model's quote matches the document it quotes.
 Length-preserving by construction:
 every mapping replaces one UTF-16 unit with one UTF-16 unit,
 so offsets in the result index the input exactly.

 @param text - text whose punctuation variants collapse

 @returns Same-length text with canonical punctuation

 @example
 ```ts
 normalizePunctuation({ text: 'father’s shop', },);
 ```
 */
export function normalizePunctuation({ text, }: { readonly text: string; },): string {
  return canonicalize({
    text,
    map: PUNCTUATION_CANON,
  },);
}

/**
 Straightens curly quotation marks: the typography fold, under which two
 renderings are the same wording when they differ only in what the
 typography restoration would change.
 Length-preserving like {@link normalizePunctuation}.

 @param text - rendering whose curly quotes straighten

 @returns Same-length text with straight quotes

 @example
 ```ts
 straightenQuotes({ text: 'the cat’s “nap”', },); // 'the cat\'s "nap"'
 ```
 */
export function straightenQuotes({ text, }: { readonly text: string; },): string {
  return canonicalize({
    text,
    map: TYPOGRAPHY_CANON,
  },);
}

/**
 Straightens the curly quotation marks of a rendering's prose only: the
 typography fold read where the typography restoration reads. A quote that is
 no prose text of the page parse (in a link destination or title, an address,
 code, math, an expression or a tag) is code or markup, which the restoration
 never touches (`typography-prose-mask.ts`), so two renderings apart there are
 two texts. For a whole slice, which may carry code; a title or a single phrase
 has none, and takes {@link straightenQuotes}.
 Length-preserving like {@link straightenQuotes}.

 @param text - rendering whose prose quotes straighten

 @returns Same-length text with straight quotes in its prose

 @example
 ```ts
 straightenProseQuotes({ text: 'the cat’s `“nap”`', },); // 'the cat\'s `“nap”`'
 ```
 */
export function straightenProseQuotes({ text, }: { readonly text: string; },): string {
  /**
   Which units are prose rather than code or markup, and whether the grammar refused the text.
   */
  const {
    mask,
    refused,
  } = proseMaskReading({ text, },);
  if (refused) {
    noteRefusedMask({
      reader: 'straightenProseQuotes',
      name: textNamed({ text, },),
    },);
  }
  /**
   Text with every quote straightened, the same length as the input.
   */
  const straight = straightenQuotes({ text, },);
  /**
   Units in input order, each taken straightened where it is prose.
   */
  const units: string[] = [];
  for (
    let index = 0;
    index < text.length;
    index += 1
  )
    units.push(((mask[index] === true) ? straight : text).charAt(index,),);
  return units.join('',);
}

/**
 Collapses every line break onto a plain space.
 Shares the length guarantee of `normalizePunctuation`, so a position found
 in the result still indexes the input.

 FOR DISPLAY, NOT FOR MATCHING: this flattens a paragraph break as readily as
 a soft wrap, which is right for a one-line diagnostic and wrong for deciding
 whether a quote occurs. Matching uses {@link collapseSoftLineBreaks}.

 @param text - text whose line breaks collapse

 @returns Same-length text reading line breaks as spaces

 @example
 ```ts
 collapseLineBreaks({ text: 'her\nshop', },);
 ```
 */
export function collapseLineBreaks({ text, }: { readonly text: string; },): string {
  return canonicalize({
    text,
    map: LINE_BREAK_CANON,
  },);
}

/**
 Whether one position holds a line-break unit.

 @param text - text being scanned

 @param index - position to read, which may sit outside the text

 @returns Whether that position holds a line break

 @example
 ```ts
 const breaks = isLineBreakAt({ text: 'a\nb', index: 1, },);
 ```
 */
function isLineBreakAt(
  {
    text,
    index,
  }: {
    readonly text: string;
    readonly index: number;
  },
): boolean {
  return LINE_BREAK_CANON.has(text.charAt(index,),);
}

/**
 Collapses only SOLE line breaks onto plain spaces, leaving a run of them as
 it stands.

 WHY A RUN IS LEFT ALONE: a lone break inside a paragraph is a soft wrap, and
 a model quoting across it writes a space, so the two forms mean the same text.
 A run of breaks is a STRUCTURAL boundary. Collapsing those too made a blank
 line into two spaces, so a quote carrying two spaces matched straight across
 a paragraph boundary the document keeps: safe only for as long as every model
 joined lines with exactly one space, which is not a property this pipeline can
 assume of its inputs. A run now matches nothing but itself.

 WHAT IT STILL DOES NOT PROTECT: boundaries a single line break represents,
 inside fenced code, between list items, and between table rows, plus a
 Markdown hard break, whose two trailing spaces plus a wrap read as three
 spaces. Those need the parse rather than the characters, and stay unprotected
 here.

 Length-preserving like everything here, so offsets still index the input.

 @param text - text whose soft wraps collapse

 @returns Same-length text reading sole line breaks as spaces

 @example
 ```ts
 collapseSoftLineBreaks({ text: 'her\nshop', },);
 ```
 */
export function collapseSoftLineBreaks({ text, }: { readonly text: string; },): string {
  /**
   Units in input order, each either collapsed or kept.
   */
  const units: string[] = [];
  for (
    let index = 0;
    index < text.length;
    index += 1
  ) {
    /**
     Unit at this position.
     */
    const unit = text.charAt(index,);

    /**
     Whether this break stands alone between non-break neighbours.
     */
    const sole = LINE_BREAK_CANON.has(unit,)
      && (!isLineBreakAt({
        text,
        index: index - 1,
      },))
      && (!isLineBreakAt({
        text,
        index: index + 1,
      },));
    units.push(sole ? ' ' : unit,);
  }
  return units.join('',);
}

//endregion Quote normalization
