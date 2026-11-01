import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { carryArchiveNotes, } from './archive-note-carry.ts';
import { restoreArchiveCasing, } from './archive-casing-restore.ts';
import { restoreArchiveDestinations, } from './archive-destination-restore.ts';
import { restoreArchiveItalicTitles, } from './archive-italic-title-restore.ts';
import { restoreArchiveNameCasing, } from './archive-name-casing.ts';
import { unwrapBlockquoteQuotes, } from './blockquote-quote-unify.ts';
import { canadianizePage, } from './canadian-forms.ts';
import {
  placeHandleGlosses,
  restoreCollidingHeadings,
  restoreContributorNames,
  type SliceReplacement,
  unifyHeadingSeries,
  unifyTitleReferences,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import { restoreJsxAttributes, } from './jsx-attribute-restore.ts';
import { foldReplacementLineEndings, } from './line-ending-fold.ts';
import { restoreListSpread, } from './list-spread-restore.ts';
import { restoreNameGlossLines, } from './name-gloss-restore.ts';
import { correctPinyinPage, } from './pinyin-tone.ts';
import { unifyQuoteStyle, } from './quote-style-unify.ts';

//region Page assembly passes
// THE PASSES THAT REWRITE THE COMPOSED PAGE, in the order they run, split out
// of `page-assembly-guard.ts` so the guard can run them again over the page
// left once it withdraws a lane's row (ledger K5).

/**
 What the passes made of the page.

 @example
 ```ts
 const outcome: PagePassesOutcome = { replacements, restored, findings, };
 ```
 */
export type PagePassesOutcome = {
  /**
   Replacements as the last pass left them, which the footnote guard reads.
   */
  readonly replacements: readonly SliceReplacement[];

  /**
   Rows a pass rewrote, the latest pass's text per slice.
   */
  readonly restored: ReadonlyMap<number, SliceReplacement>;

  /**
   What each pass did, in pass order.
   */
  readonly findings: readonly string[];
};

/**
 Runs every page-assembly pass over one set of replacements.

 @param slices - preparation defining replacement spans

 @param sourceText - the original document

 @param targetText - archive text the replacement spans address

 @param replacements - rows the page would write

 @param archiveOriginalSpans - spans sealed as the English original

 @returns Replacements, rewritten rows and findings

 @example
 ```ts
 const passes = runPagePasses({ slices, sourceText, targetText, replacements, archiveOriginalSpans, },);
 ```
 */
export function runPagePasses(
  {
    slices,
    sourceText,
    targetText,
    replacements,
    archiveOriginalSpans,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
  },
): PagePassesOutcome {
  /**
   Every replacement with a model's Windows line endings folded to LF, before
   any pass reads it (ledger A3).
   */
  const lineEndings = foldReplacementLineEndings({ replacements, },);
  /**
   Headings a lane rewrote into another section's, restored to the archive's
   before the footnote guard reads the page (class forty-five).
   */
  const restoration = restoreCollidingHeadings({
    sourceText,
    targetText,
    slices,
    replacements: lineEndings.replacements,
  },);
  /**
   Contributor names rendered one way across headings and signatures (class
   sixty-seven).
   */
  const names = restoreContributorNames({
    slices,
    replacements: restoration.replacements,
  },);
  /**
   Every romanised handle's literal meaning at its first appearance alone
   (class eighty-eight).
   */
  const glosses = placeHandleGlosses({
    slices,
    replacements: names.replacements,
  },);
  /**
   The original's numbered heading series rendered in one style (class
   sixty-eight).
   */
  const series = unifyHeadingSeries({
    slices,
    replacements: glosses.replacements,
  },);
  /**
   Every tag attribute at the archive's value (class ninety-nine).
   */
  const attributes = restoreJsxAttributes({
    slices,
    replacements: series.replacements,
  },);
  /**
   Every reference to a section title as the heading renders it (class one
   hundred).
   */
  const titles = unifyTitleReferences({
    slices,
    replacements: attributes.replacements,
  },);
  /**
   The archive's gloss line of a name wherever the shipped text carries the
   name unglossed (class one hundred five).
   */
  const glossLines = restoreNameGlossLines({
    slices,
    replacements: titles.replacements,
  },);
  /**
   Every list at the archive's spacing between its items (class one hundred
   seventeen).
   */
  const lists = restoreListSpread({
    slices,
    replacements: glossLines.replacements,
  },);
  /**
   Every word the archive writes only in capitals at the archive's form
   (class one hundred twenty-one).
   */
  const casing = restoreArchiveCasing({
    slices,
    replacements: lists.replacements,
  },);
  /**
   Every multi-word name the archive writes title case at the archive's form
   (class one hundred thirty-six).
   */
  const nameCasing = restoreArchiveNameCasing({
    slices,
    replacements: casing.replacements,
  },);
  /**
   Every date month first and every listed word in its Canadian spelling, on
   every slice the page carries (class one hundred thirty-four).
   */
  const canadian = canadianizePage({
    slices,
    replacements: nameCasing.replacements,
    archiveOriginalSpans,
  },);
  /**
   Every pinyin syllable of a Han and pinyin pair in its character's tone,
   on every slice the page carries (class one hundred thirty-seven).
   */
  const pinyinTones = correctPinyinPage({
    slices,
    replacements: canadian.replacements,
    archiveOriginalSpans,
  },);
  /**
   Every slice's prose quote marks in the page's majority style (class one
   hundred forty-two).
   */
  const quotes = unifyQuoteStyle({
    slices,
    replacements: pinyinTones.replacements,
    archiveOriginalSpans,
  },);
  /**
   Every quoted blockquote paragraph unwrapped where the archive sets its
   blockquotes bare (class one hundred sixty-eight).
   */
  const blockquotes = unwrapBlockquoteQuotes({
    slices,
    replacements: quotes.replacements,
    archiveOriginalSpans,
  },);
  /**
   Every quoted archive title in the archive's italics (class one hundred
   seventy-three).
   */
  const italics = restoreArchiveItalicTitles({
    slices,
    replacements: blockquotes.replacements,
    archiveOriginalSpans,
  },);
  /**
   Every link the archive gave its own destination pointed back at it
   (ledger A4, owner 2026-09-27, "Archive's English").
   */
  const destinations = restoreArchiveDestinations({
    slices,
    replacements: italics.replacements,
    archiveOriginalSpans,
  },);
  /**
   Every note the archive's translators left each other that the page's
   wording lacks, after the slice's last block, so its survival never depends
   on which wording won.
   */
  const notes = carryArchiveNotes({
    slices,
    replacements: destinations.replacements,
    archiveOriginalSpans,
  },);
  /**
   Rows the page-assembly passes rewrote, the latest pass's text per slice.
   */
  const restored = new Map<number, SliceReplacement>([
    ...lineEndings.restored,
    ...restoration.restored,
    ...names.restored,
    ...glosses.restored,
    ...series.restored,
    ...attributes.restored,
    ...titles.restored,
    ...glossLines.restored,
    ...lists.restored,
    ...casing.restored,
    ...nameCasing.restored,
    ...canadian.restored,
    ...pinyinTones.restored,
    ...quotes.restored,
    ...blockquotes.restored,
    ...italics.restored,
    ...destinations.restored,
    ...notes.restored,
  ].map(function bySlice(row,): readonly [
    number,
    SliceReplacement,
  ] {
    return [
      row.sliceIndex,
      row,
    ];
  },),);
  return {
    replacements: notes.replacements,
    restored,
    findings: [
      ...lineEndings.findings,
      ...restoration.findings,
      ...names.findings,
      ...glosses.findings,
      ...series.findings,
      ...attributes.findings,
      ...titles.findings,
      ...glossLines.findings,
      ...lists.findings,
      ...casing.findings,
      ...nameCasing.findings,
      ...canadian.findings,
      ...pinyinTones.findings,
      ...quotes.findings,
      ...blockquotes.findings,
      ...italics.findings,
      ...destinations.findings,
      ...notes.findings,
    ],
  };
}

//endregion Page assembly passes
