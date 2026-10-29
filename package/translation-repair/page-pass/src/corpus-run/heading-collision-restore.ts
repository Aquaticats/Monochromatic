import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { headingWords, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SliceReplacement, } from '../splice-slices.ts';
import {
  pageTextBySlice,
  pageTextOf,
  slicesInOrder,
  withRewrittenText,
} from './assembly-page-text.ts';

//region Heading collision restore
// CLASS FORTY-FIVE (hulicaijia6, 2026-09-17). The repair lane rendered the
// 相遇 heading as "Meeting", which is the archive's heading for 初识 two
// sections earlier, every slice floor passed it (a slice holds one heading),
// and the publish guard refused the page after 205 minutes for rendering 13
// distinct source headings as 12. The re-attempt resumed the same cached
// texts and would have died the same way.
//
// THE ARCHIVE'S HEADING IS THE FALLBACK. Measured at pin a41fc607 no archive
// renders two distinct source headings identically, so where a page heading
// collides with another section's and differs from the archive's rendering at
// its position, the archive's rendering is restored into that slice's text
// here, at the one place that sees every heading, and recorded as a page
// assembly override the way a trimmed slice is. The publish guard keeps the
// last word for whatever this cannot resolve.

/**
 Node kind the parser gives a heading.
 */
const HEADING_KIND = 'heading';

/**
 One heading as written and as compared.
 */
type HeadingLine = {
  /**
   Heading text with its marks, as the document carries it.
   */
  readonly text: string;

  /**
   Words the distinctness rule compares.
   */
  readonly words: string;
};

/**
 One page heading with the slice that carries it.
 */
type PageHeading = HeadingLine & {
  /**
   Slice whose text the heading sits in.
   */
  readonly sliceIndex: number;
};

/**
 Headings of one document in order.

 @param text - document or slice text

 @returns Heading lines in document order

 @example
 ```ts
 const lines = headingLinesOf({ text: '## 小猫\n\n它睡了。', },);
 ```
 */
function headingLinesOf({ text, }: { readonly text: string; },): readonly HeadingLine[] {
  return parseDocument({ text, },)
    .nodes
    .filter(function isHeading(node,): boolean {
      return node.kind === HEADING_KIND;
    },)
    .map(function toLine(node,): HeadingLine {
      return {
        text: node.text,
        words: headingWords({ text: node.text, },),
      };
    },);
}

/**
 Restores the archive's heading into every slice whose rendered heading
 repeats another section's while the original's headings differ.

 PAIRS HEADINGS BY POSITION ONLY WHERE THE POSITIONS LINE UP: the original
 and the archive carry as many headings as each other, and every slice of the
 page carries as many as its archive text does. Anywhere else the page is left
 as it stands for the publish guard (ledger B83).

 @param sourceText - original document

 @param targetText - archive document the replacements address

 @param slices - prepared pairs defining each slice's archive span

 @param replacements - what the page would write per slice

 @returns Replacements with the restored headings, the restored rows alone,
 and one finding per restoration

 @example
 ```ts
 const restoration = restoreCollidingHeadings({ sourceText, targetText, slices, replacements, },);
 ```
 */
export function restoreCollidingHeadings(
  {
    sourceText,
    targetText,
    slices,
    replacements,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Nothing to restore.
   */
  const unchanged = {
    replacements,
    restored: [],
    findings: [],
  };
  /**
   Original's headings by position.
   */
  const source = headingLinesOf({ text: sourceText, },);
  /**
   Archive's headings by position.
   */
  const archive = headingLinesOf({ text: targetText, },);
  if (source.length !== archive.length)
    return unchanged;
  /**
   Text the page carries per slice: a lane's replacement, else the archive's.
   */
  const pageText = pageTextBySlice({
    slices,
    replacements,
  },);
  /**
   Each slice's page headings, beside how many headings its archive text
   carries.
   */
  const bySlice = slicesInOrder({ slices, },)
    .map(function headingsOf(slice,): {
      readonly headings: readonly PageHeading[];
      readonly archived: number;
    } {
      /**
       Index of this slice.
       */
      const { sliceIndex, } = slice.target;
      /**
       Archive text of this slice.
       */
      const incumbent = slice.target
        .text;
      /**
       What the page carries for this slice.
       */
      const text = pageTextOf({
        pageText,
        sliceIndex,
      },);
      return {
        headings: headingLinesOf({ text, },)
          .map(function withSlice(line,): PageHeading {
            return {
              ...line,
              sliceIndex,
            };
          },),
        archived: headingLinesOf({ text: incumbent, },)
          .length,
      };
    },);
  // POSITIONS PAIR ONLY WHERE EVERY SLICE KEEPS ITS HEADING COUNT (ledger
  // B83). A slice that drops a heading and another that gains one leave the
  // page's total where the original's is, while every heading between them
  // sits one position off; pairing those positions once compared a section
  // the page never replaced with its neighbour's archive heading and shipped
  // that section as empty text. Where any slice moved its count, nothing here
  // can say which heading answers which, and the publish guard keeps the last
  // word.
  if (bySlice.some(function movedCount(slice,): boolean {
    /**
     Headings the page carries in this slice.
     */
    const { headings, } = slice;
    return headings.length !== slice.archived;
  },))
    return unchanged;
  /**
   Page headings in document order, each with its slice.
   */
  const page: readonly PageHeading[] = bySlice.flatMap(function headingsOf(slice,): readonly PageHeading[] {
    return slice.headings;
  },);
  if (page.length !== source.length)
    return unchanged;
  /**
   Compared words of the original's headings by position.
   */
  const sourceWords = source.map(function wordsOf(line,): string {
    return line.words;
  },);
  /**
   Compared words of the archive's headings by position.
   */
  const archiveWords = archive.map(function wordsOf(line,): string {
    return line.words;
  },);
  /**
   Positions whose page heading repeats another section's heading while the
   original's differ, and whose rendering is not the archive's own.
   */
  const colliding = page
    .map(function collidesAt(
      heading,
      index,
    ): number {
      /**
       Whether another section reads the same while the original differs.
       */
      const collides = page.some(function otherSectionSameWords(
        other,
        otherIndex,
      ): boolean {
        return (otherIndex !== index)
          && (other.words === heading.words)
          && (sourceWords[otherIndex] !== sourceWords[index]);
      },);
      return (collides && (heading.words !== archiveWords[index])) ? index : NOT_COLLIDING;
    },)
    .filter(function isPosition(index,): boolean {
      return index !== NOT_COLLIDING;
    },);
  if (colliding.length === 0)
    return unchanged;
  /**
   Restored text per slice, built one collision at a time so two headings in
   one slice both restore.
   */
  const restoredText = new Map<number, string>();
  /**
   Finding per restoration.
   */
  const findings: string[] = [];
  for (const index of colliding) {
    /**
     Heading being restored, which exists because every colliding position is
     an index into the page's headings.
     */
    const heading = nonNullishOrThrow(page[index],);
    /**
     Archive heading at the same position, which exists because the archive
     carries as many headings as the page.
     */
    const replacement = nonNullishOrThrow(archive[index],);
    // ONLY A SLICE THE PAGE REPLACES CAN COLLIDE. With every slice keeping its
    // heading count, a slice the page leaves alone carries the archive's own
    // headings at the archive's own positions, so it never differs from the
    // archive there (ledger B83).
    /**
     Slice text as it stands after earlier restorations.
     */
    const current = restoredText.get(heading.sliceIndex,)
      ?? pageTextOf({
        pageText,
        sliceIndex: heading.sliceIndex,
      },);
    restoredText.set(
      heading.sliceIndex,
      current.replace(
        heading.text,
        function literal(): string {
          return replacement.text;
        },
      ),
    );
    findings.push(
      `slice ${String(heading.sliceIndex,)}: heading "${heading.words}" repeats another section's heading `
        + `where the original's differ; the archive's "${replacement.words}" restored (class forty-five)`,
    );
  }
  return {
    ...withRewrittenText({
      replacements,
      rewritten: restoredText,
    },),
    findings,
  };
}

/**
 Position marker for a heading that collides with none.
 */
const NOT_COLLIDING = -1;

//endregion Heading collision restore
