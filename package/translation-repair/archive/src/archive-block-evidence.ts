import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { PhrasingContent, } from 'mdast';

import { contributorDeclarationLines, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  maskHtmlComments,
  parseMarkdownBody,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  quoteIsIn,
  referencePageTexts,
} from '@monochromatic-dev/translation-repair-reference/ts';
import { wordStarts, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Archive block evidence

/**
 Minimum Unicode characters an exact source anchor must carry.
 */
export const MINIMUM_SOURCE_QUOTE_CHARACTERS = 4;

/**
 Translation-side apparatus prefixes accepted only with roster agreement.
 */
const EDITORIAL_PREFIXES: readonly string[] = [
  'contributor:',
  'contributors:',
  'contributors for this entry:',
  'credit:',
  'credits:',
  'source:',
  'sources:',
  'translated by',
  'translation by',
  'translation:',
  'translator:',
];

/**
 Characters of a text that show a reader something, by code point.

 @param text - text read

 @returns Code points that are not whitespace, default-ignorable or control

 @example
 ```ts
 shownCodePointCount({ text: '猫\u{200B}在', },); // 2
 ```
 */
export function shownCodePointCount({ text, }: { readonly text: string; },): number {
  /**
   Code points seen so far.
   */
  let shown = 0;
  for (const character of text) {
    if (!rendersAsNothing({ text: character, },))
      shown += 1;
  }
  return shown;
}

/**
 Checks exact source support is substantive and inside expected aligned section.

 READ THROUGH THE EVIDENCE FOLD (`normalizePunctuation`, ledger B24): a
 reviewer quoting the original writes its 「」 as English quotes as often as
 not, and the words are what the anchor asks about. The minimum is counted in
 characters a reader sees, not UTF-16 units, as it says: a zero-width space, a
 filler and a space anchor nothing.

 WHITESPACE IS SET ASIDE ON BOTH SIDES (ledger B715), by `quoteIsIn`, the
 comparison the reference attestation makes of a quote it verifies (Mio22,
 ledger B25). The readers are told to preserve line breaks, so a transcript of
 a chat screenshot wraps mid-sentence where the picture does, and a reviewer
 quoting the sentence whole quotes the words and not the wrap: zheermao101
 (run zheermao20) lost an assessment to a 39-character sentence a raw
 `includes` could not find across "罪恶\n的呢". The trade is the attestation's:
 with spaces gone, Latin words can meet across a boundary the text does not
 have, and the four-character minimum is what bounds that. A quote stitched
 from two pictures still refuses, since the heading and the picture's own
 words stand between two transcripts on the sheet.

 @param sourceContext - expected source section

 @param sourceQuote - provider's exact support claim

 @returns Whether quote is long enough and anchored in expected section

 @example
 ```ts
 isArchiveSourceQuoteAnchored({ sourceContext: '猫在窗边睡觉。', sourceQuote: '窗边睡觉', });
 ```
 */
export function isArchiveSourceQuoteAnchored(
  {
    sourceContext,
    sourceQuote,
  }: {
    readonly sourceContext: string;
    readonly sourceQuote: string;
  },
): boolean {
  /**
   Quote without accidental boundary whitespace.
   */
  const normalizedQuote = sourceQuote.trim();
  return (shownCodePointCount({ text: normalizedQuote, },) >= MINIMUM_SOURCE_QUOTE_CHARACTERS)
    && quoteIsIn({
      quote: normalizedQuote,
      text: sourceContext,
    },);
}

/**
 Checks a retention's support is substantive and stated by one page the
 original cites.

 A CITED PAGE IS THE ORIGINAL'S OWN SOURCE (the owner's decision of
 2026-09-16, ledger B28): an archive block carrying what a linked page states
 is the translator's knowledge, not an unsupported insertion. The quote is
 looked for in one page's text at a time, with the fold and the minimum the
 source anchor uses; the address, the attestation's lines (each quoting the
 archive) and the lookup's failure notes are never support.

 @param referenceContext - reference lines, with any attested lines under them

 @param sourceQuote - provider's exact support claim

 @returns Whether quote is long enough and stated by one cited page

 @throws ReferenceLineHeadError when a reference line has no numbered head

 @example
 ```ts
 isArchiveReferenceQuoteAnchored({ referenceContext: '- reference 1 https://cats.example/a: Mittens naps.', sourceQuote: 'Mittens naps', });
 ```
 */
export function isArchiveReferenceQuoteAnchored(
  {
    referenceContext,
    sourceQuote,
  }: {
    readonly referenceContext: string;
    readonly sourceQuote: string;
  },
): boolean {
  return referencePageTexts({ referenceContext, },)
    .some(function statesQuote(pageText,): boolean {
      return isArchiveSourceQuoteAnchored({
        sourceContext: pageText,
        sourceQuote,
      },);
    },);
}

/**
 Whether a line opens with an apparatus label standing as words of its own,
 read by `wordStarts` with both edges bounded, so "Translation byproducts"
 is no "translation by" label (the B23 family).

 @param line - one line of a block, its comments blanked

 @returns Whether the line, case-folded, opens with a label at word edges

 @example
 ```ts
 isLabelLine({ line: 'Translated by Mittens', },); // true
 ```
 */
function isLabelLine({ line, }: { readonly line: string; },): boolean {
  /**
   Case-folded line, as the fixed labels are written.
   */
  const normalized = line.trim()
    .toLowerCase();
  return EDITORIAL_PREFIXES.some(function opensWith(label,): boolean {
    return wordStarts({
      text: normalized,
      needle: label,
      end: 'word',
    },)
      .includes(0,);
  },);
}

/**
 Whether a line holds pictures and nothing else a reader sees.

 @param line - one line of a block, its comments blanked

 @returns Whether Markdown reads the line as one paragraph of images alone

 @example
 ```ts
 isPictureLine({ line: '![A tabby asleep](tabby.png)', },); // true
 ```
 */
function isPictureLine({ line, }: { readonly line: string; },): boolean {
  /**
   The line as Markdown reads it on its own.
   */
  const [block, ...rest] = parseMarkdownBody({ body: line, },)
    .children;
  if ((block?.type !== 'paragraph') || (rest.length > 0))
    return false;
  /**
   Everything the paragraph holds.
   */
  const { children, } = block;
  /**
   What the paragraph holds besides its pictures.
   */
  const besides = children.filter(function notPicture(child: ForeignBorrowed<PhrasingContent>,): boolean {
    return child.type !== 'image';
  },);
  return (besides.length < children.length)
    && besides.every(function showsNothing(child: ForeignBorrowed<PhrasingContent>,): boolean {
      return (child.type === 'text') && rendersAsNothing({ text: child.value, },);
    },);
}

/**
 Deterministically corroborates narrow translation-side apparatus category.

 EVERY LINE A READER SEES IS APPARATUS (ledger B81). With the block's
 comments blanked, each line it shows is one the contributor reader takes
 as a declaration, one opening with an apparatus label, or one of pictures
 alone; a block of closed comments and nothing else is apparatus too, and
 a comment left open is not. The check once accepted a block when any line
 declared contributors, or when it opened with a label, a picture or a
 comment and ended with a closed one, so prose beside them passed with
 them, and a label's words opening a longer word passed as the label.

 What a label line says after its label is not read: whether it names a
 person or tells of one is the reviewers' to judge.

 @param blockText - exact unclaimed archive block

 @returns Whether every line the block shows is contributor, citation, or
 picture apparatus, or the block is closed comments alone

 @example
 ```ts
 isVerifiableEditorialArchiveBlock({ blockText: 'Translator: Cat Friend', });
 ```
 */
export function isVerifiableEditorialArchiveBlock(
  { blockText, }: { readonly blockText: string; },
): boolean {
  /**
   The block with each comment blanked to spaces and its line breaks kept,
   so a line's index is the same in both.
   */
  const {
    masked,
    regions,
  } = maskHtmlComments({ text: blockText, },);
  // A COMMENT LEFT OPEN runs to the end of the page, so the block vouches
  // for nothing after it.
  if (regions.some(function leftOpen(region,): boolean {
    return !region.terminated;
  },))
    return false;
  /**
   Lines the contributor reader takes as declarations.
   */
  const declaring = new Set(contributorDeclarationLines({ text: masked, },)
    .map(function lineOf(declaration,): number {
      return declaration.line;
    },),);
  /**
   Lines a reader sees, each with its index.
   */
  const shown = [...masked.split('\n',)
    .entries(),].filter(function seen([, line,]: ReadonlyEntry<number, string>,): boolean {
      return !rendersAsNothing({ text: line, },);
    },);
  if (shown.length === 0)
    return regions.length > 0;
  return shown.every(function isApparatus([at, line,]: ReadonlyEntry<number, string>,): boolean {
    return declaring.has(at,)
      || isLabelLine({ line, },)
      || isPictureLine({ line, },);
  },);
}

//endregion Archive block evidence
