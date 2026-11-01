import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  pageTextBySlice,
  pageTextOf,
  type SliceReplacement,
  slicesInOrder,
  withRewrittenText,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  readTags,
  type TagReading,
} from './tag-attributes.ts';

import {
  applySpanRewrites,
  type SpanRewrite,
} from './span-rewrites.ts';

//region JSX attribute restore
// CLASS NINETY-NINE (XingZ630, 2026-09-23). The original's dotted section
// markers `<DottedNumber n="二"/>` to `<DottedNumber n="七"/>` reached the
// settled page as `n="II"`, `n="III"`, `n="IV"`, `n="5"`, `n="VI"` and
// `n="七"` where the archive writes Roman numerals throughout; XingZ629 had
// shipped `n="五"` in the same series. A tag's string attribute is
// apparatus, not prose: no judge weighs it and every slice is judged
// alone, so the numeral style moves slice to slice. THE PAGE DECIDES ONCE,
// HERE, from the archive: where the page carries the same tags in the same
// order as the archive's text for the slice, every attribute both carry
// takes the archive's value. A slice whose tag sequence differs from the
// archive's, and a slice the archive never carried, are left as the bench
// wrote them.

/**
 One attribute rewrite the pass will apply, from the archive's tag.
 */
type AttributeRewrite = {
  /**
   Offset of the value's first character in the page text.
   */
  readonly start: number;

  /**
   Offset just past the value in the page text.
   */
  readonly end: number;

  /**
   Value the archive carries.
   */
  readonly value: string;

  /**
   Tag as the page wrote it, for the finding.
   */
  readonly before: string;

  /**
   Tag as the archive writes it, for the finding.
   */
  readonly after: string;
};

/**
 Whether two tag sequences name the same tags in the same order.

 @param left - tags of one text

 @param right - tags of another

 @returns True when the names align one to one

 @example
 ```ts
 sameTagSequence({ left: pageTags, right: archiveTags, },); // true
 ```
 */
function sameTagSequence(
  {
    left,
    right,
  }: {
    readonly left: readonly TagReading[];
    readonly right: readonly TagReading[];
  },
): boolean {
  if (left.length !== right.length)
    return false;
  return left.every(function namesAlign(
    tag,
    at,
  ): boolean {
    /**
     Tag at the same position on the other side.
     */
    const other = right[at];
    return (other !== undefined) && (tag.name === other.name);
  },);
}

/**
 Rewrites the archive's attribute values ask of one page tag, in offset
 order.

 @param pageTag - tag as the page wrote it

 @param archiveTag - tag as the archive writes it

 @returns One rewrite per attribute whose values differ and which the
 archive's tag names once

 @example
 ```ts
 const rewrites = attributeRewrites({ pageTag, archiveTag, },);
 ```
 */
function attributeRewrites(
  {
    pageTag,
    archiveTag,
  }: {
    readonly pageTag: TagReading;
    readonly archiveTag: TagReading;
  },
): readonly AttributeRewrite[] {
  return pageTag.attributes
    .flatMap(function differing(attribute,): readonly AttributeRewrite[] {
      /**
       Archive's first attribute of the same name, if it carries one, and
       every further one. GATHERED, NOT THE FIRST ALONE: the archive's tag is
       text a person wrote, and nothing makes an attribute's name unique in
       it. Which of two values a reader of the page sees depends on what
       renders the tag, so neither is the value the archive fixed, and the
       attribute stays as the bench wrote it.
       */
      const [archived, ...others] = archiveTag.attributes
        .filter(function sameName(candidate,): boolean {
          return candidate.name === attribute.name;
        },);
      if ((archived === undefined) || (others.length > 0))
        return [];
      if (archived.value === attribute.value)
        return [];
      return [
        {
          start: attribute.valueStart,
          end: attribute.valueEnd,
          value: archived.value,
          before: pageTag.text,
          after: archiveTag.text,
        },
      ];
    },);
}

/**
 Page text with the rewrites applied, through the applier every page pass
 shares (`span-rewrites.ts`).

 @param text - page text of the slice

 @param rewrites - rewrites in offset order

 @returns Rewritten text

 @example
 ```ts
 const text = applyRewrites({ text, rewrites, },);
 ```
 */
function applyRewrites(
  {
    text,
    rewrites,
  }: {
    readonly text: string;
    readonly rewrites: readonly AttributeRewrite[];
  },
): string {
  return applySpanRewrites({
    text,
    rewrites: rewrites.map(function asSpan(rewrite,): SpanRewrite {
      return {
        start: rewrite.start,
        end: rewrite.end,
        to: rewrite.value,
      };
    },),
  },)
    .text;
}

/**
 Restores every tag attribute the bench rendered to the archive's value,
 slice by slice, where the page carries the archive's tags in order.

 @param slices - prepared pairs, whose archive side fixes the attributes

 @param replacements - what the page would write per slice

 @returns Replacements with the attributes restored, the rewritten rows
 alone, and one finding per rewritten attribute

 @example
 ```ts
 const restored = restoreJsxAttributes({ slices, replacements, },);
 ```
 */
export function restoreJsxAttributes(
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
   One finding per rewritten attribute.
   */
  const findings: string[] = [];
  for (const slice of slicesInOrder({ slices, },)) {
    /**
     Index of this slice.
     */
    const { sliceIndex, } = slice.target;
    if (!replaced.has(sliceIndex,))
      continue;
    /**
     Tags the archive writes for this slice.
     */
    const archiveTags = readTags({ text: slice.target
      .text, },);
    if (archiveTags.length === 0)
      continue;
    /**
     Page text of this slice.
     */
    const text = pageTextOf({
      pageText,
      sliceIndex,
    },);
    /**
     Tags the page wrote for this slice.
     */
    const pageTags = readTags({ text, },);
    if (!sameTagSequence({
      left: pageTags,
      right: archiveTags,
    },))
      continue;
    /**
     Every attribute rewrite the archive asks of this slice.
     */
    const rewrites = pageTags.flatMap(function perTag(
      pageTag,
      at,
    ): readonly AttributeRewrite[] {
      /**
       Archive's tag at the same position.
       */
      const archiveTag = archiveTags[at];
      if (archiveTag === undefined)
        throw new Error(
          'unreachable: a page tag has no archive tag at its position, '
            + 'though sameTagSequence had matched the two sequences by length and by name',
        );
      return attributeRewrites({
        pageTag,
        archiveTag,
      },);
    },);
    if (rewrites.length === 0)
      continue;
    rewritten.set(
      sliceIndex,
      applyRewrites({
        text,
        rewrites,
      },),
    );
    for (const rewrite of rewrites) {
      findings.push(
        `jsx-attribute-restored (slice ${String(sliceIndex,)}: ${rewrite.before} to ${rewrite.after}; `
          + 'a tag attribute is apparatus the archive fixed)',
      );
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

//endregion JSX attribute restore
