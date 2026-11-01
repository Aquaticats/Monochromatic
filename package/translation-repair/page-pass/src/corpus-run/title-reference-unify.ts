import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  rendersAsNothing,
  straightenQuotes,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { isHanOnly, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { SliceReplacement, } from '../splice-slices.ts';
import { withoutHtmlComments, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  pageTextBySlice,
  pageTextOf,
  slicesInOrder,
  withRewrittenText,
} from './assembly-page-text.ts';
import { headingTitles, } from './heading-title-lines.ts';
import { locateTitleRendering, } from './title-reference-locate.ts';
import {
  type RenderedHeading,
  rewriteRenderings,
} from './title-reference-rewrite.ts';
import { bracketsTitle, } from './title-reference-scope.ts';

//region Title reference unify
// CLASS ONE HUNDRED (XingZ630, 2026-09-23). The original heads a section
// 笼中之鸟 and credits the song as 《[笼中之鸟](…)》; it heads another
// 零重祈愿, credits it as 《零重祈愿》 and points a footnote at 「零重祈愿」篇.
// The settled page headed "Bird in a Cage" over the link "The Caged Bird"
// and "Zero-Layer Prayer" over the footnote's "Zero-Degree Prayer";
// XingZ629 had headed "The Bird in the Cage" over the link "Bird in a
// Cage". Every slice is judged alone and no judge sees the heading beside
// the reference. THE PAGE DECIDES ONCE, HERE, where every heading is in
// view: a reference to a section title takes the heading's rendering,
// found in the referencing slice by the link's destination, by every Han
// gloss after the English, by title brackets or by quotes; a slice that
// offers two bracketed or quoted spans is reported, not guessed at.

/**
 Opening parenthesis of a gloss.
 */
const GLOSS_OPEN = '(';

/**
 Closing parenthesis of a gloss.
 */
const GLOSS_CLOSE = ')';

/**
 Rendering with a trailing Han gloss of the title stripped.

 @param rendering - heading title as the page renders it

 @param title - Han title

 @returns Rendering without ` (title)` at its end

 @example
 ```ts
 withoutTitleGloss({ rendering: 'Cat (猫)', title: '猫', },); // 'Cat'
 ```
 */
function withoutTitleGloss(
  {
    rendering,
    title,
  }: {
    readonly rendering: string;
    readonly title: string;
  },
): string {
  /**
   Gloss the rendering may end with.
   */
  const gloss = `${GLOSS_OPEN}${title}${GLOSS_CLOSE}`;
  if (!rendering.endsWith(gloss,))
    return rendering;
  return rendering.slice(
    0,
    rendering.length - gloss.length,
  )
    .trim();
}

/**
 Every Han section heading the page renders in English, by title, where
 the page carries as many heading lines as the original for the slice; a
 title two headings share is kept where they render it alike and dropped as
 ambiguous where they do not.

 @param slices - prepared pairs in order

 @param pageText - page text per slice

 @returns Rendered headings by original title

 @example
 ```ts
 const headings = renderedHeadings({ slices, pageText, },);
 ```
 */
function renderedHeadings(
  {
    slices,
    pageText,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly pageText: ReadonlyMap<number, string>;
  },
): ReadonlyMap<string, RenderedHeading> {
  /**
   Headings read so far.
   */
  const headings = new Map<string, RenderedHeading>();
  /**
   Titles two headings render differently.
   */
  const shared = new Set<string>();
  for (const slice of slices) {
    /**
     Index of this slice.
     */
    const { sliceIndex, } = slice.target;
    /**
     Titles the original heads here.
     */
    const titles = headingTitles({ text: slice.source
      .text, },);
    /**
     Page text of this slice, which the page text holds for every slice.
     */
    const text = pageTextOf({
      pageText,
      sliceIndex,
    },);
    /**
     Titles the page heads here.
     */
    const rendered = headingTitles({ text, },);
    if (titles.length !== rendered.length)
      continue;
    titles.forEach(function pairWith(
      title,
      at,
    ): void {
      if (!isHanOnly({ text: title, },))
        return;
      /**
       Rendering without its gloss, the page heading as many lines here as
       the original.
       */
      const rendering = withoutTitleGloss({
        rendering: nonNullishOrThrow(rendered[at],),
        title,
      },);
      // A RENDERING THAT SHOWS A READER NOTHING IS NONE (ledger B40): the page
      // text is what a lane wrote, and asked of an empty-string comparison a
      // heading of one zero-width space or Hangul filler was taken for the
      // title's English and written over every reference to it.
      if (rendersAsNothing({ text: rendering, },))
        return;
      // A rendering still in Han, the title itself among them, is no English
      // to unify with.
      if (isHanOnly({ text: rendering, },))
        return;
      /**
       Heading already read under the same title, if any.
       */
      const read = headings.get(title,);
      if (read === undefined) {
        headings.set(
          title,
          {
            sliceIndexes: [sliceIndex,],
            title,
            rendering,
          },
        );
        return;
      }
      // A TITLE TWO HEADINGS SHARE (quality call, T8 batch 10). Rendered
      // alike, apostrophe style aside as the typography fold reads it (ledger
      // B24), a reference to either takes that one rendering; rendered apart,
      // which one it takes cannot be read.
      if (straightenQuotes({ text: read.rendering, },) !== straightenQuotes({ text: rendering, },)) {
        shared.add(title,);
        return;
      }
      headings.set(
        title,
        {
          ...read,
          sliceIndexes: [
            ...read.sliceIndexes,
            sliceIndex,
          ],
        },
      );
    },);
  }
  for (const title of shared)
    headings.delete(title,);
  return headings;
}

/**
 Unifies every reference to a section title with the heading's rendering,
 across the replaced slices.

 @param slices - prepared pairs, whose original names the headings and the
 references

 @param replacements - what the page would write per slice

 @returns Replacements with the references unified, the rewritten rows
 alone, and one finding per reference read

 @example
 ```ts
 const unified = unifyTitleReferences({ slices, replacements, },);
 ```
 */
export function unifyTitleReferences(
  {
    slices,
    replacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Page text per slice.
   */
  const pageText = pageTextBySlice({
    slices,
    replacements,
  },);
  /**
   Slices in order.
   */
  const ordered = slicesInOrder({ slices, },);
  /**
   Section headings the page renders in English.
   */
  const headings = renderedHeadings({
    slices: ordered,
    pageText,
  },);
  /**
   Slices a replacement covers.
   */
  const replaced = new Set(replacements.map(function indexOf(replacement,): number {
    return replacement.sliceIndex;
  },),);
  /**
   Page text per slice after the rewrites.
   */
  const rewritten = new Map<number, string>();
  /**
   One finding per reference read.
   */
  const findings: string[] = [];
  for (const slice of ordered) {
    /**
     Index of this slice.
     */
    const { sliceIndex, } = slice.target;
    if (!replaced.has(sliceIndex,))
      continue;
    /**
     Original text of this slice, comments cut.
     */
    const sourceText = withoutHtmlComments({ text: slice.source
      .text, },);
    for (const heading of headings.values()) {
      /**
       Slices whose text carries the heading.
       */
      const { sliceIndexes, } = heading;
      if (sliceIndexes.includes(sliceIndex,))
        continue;
      if (!bracketsTitle({
        text: sourceText,
        title: heading.title,
      },))
        continue;
      /**
       Page text of this slice as it stands after earlier rewrites.
       */
      const text = rewritten.get(sliceIndex,)
        ?? pageTextOf({
          pageText,
          sliceIndex,
        },);
      /**
       Where this slice renders the title.
       */
      const located = locateTitleRendering({
        sourceText,
        pageText: text,
        title: heading.title,
        rendering: heading.rendering,
      },);
      /**
       Heading or headings whose rendering the reference takes.
       */
      const headingSlices = `the ${
        wordForCount({
          count: sliceIndexes.length,
          one: 'heading of slice',
          many: 'headings of slices',
        },)
      } ${sliceIndexes.join(', ',)}`;
      /**
       Whose rendering the reference takes, for the findings.
       */
      const whose = `「${heading.title}」 rendered by ${headingSlices} as "${heading.rendering}"`;
      if (located.kind === 'none') {
        findings.push(`title-reference-unplaced (slice ${String(sliceIndex,)}: ${whose}, no linked, glossed, bracketed or quoted span found)`,);
        continue;
      }
      if (located.kind === 'ambiguous') {
        findings.push(`title-reference-ambiguous (slice ${String(sliceIndex,)}: ${whose}, but the slice offers more than one span to read)`,);
        continue;
      }
      /**
       Rewrites of the located renderings.
       */
      const rewrite = rewriteRenderings({
        text,
        renderings: located.renderings,
        heading,
        sliceIndex,
        whose,
      },);
      if (rewrite.text !== text) {
        rewritten.set(
          sliceIndex,
          rewrite.text,
        );
      }
      findings.push(...rewrite.findings,);
    }
  }
  /**
   Replacements with the rewritten slices folded in.
   */
  const folded = withRewrittenText({
    replacements,
    rewritten,
  },);
  return {
    ...folded,
    findings,
  };
}

//endregion Title reference unify
