import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type SliceReplacement,
  slicesInOrder,
} from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Page slice rewrite
// The loop the page-assembly passes that read every slice share (class one
// hundred thirty-four's Canadian forms, class one hundred thirty-seven's
// pinyin tones): each slice is read as the page will carry it, the lane's
// row where one replaced it and the archive's text where none did, and a
// rewrite that changes it adds or swaps the slice's row. The front matter is
// published as the archive has it and a span sealed as the English original
// is carried byte for byte, so both stand aside.

/**
 What one text rewrite returns: the text and each change as "from" to "to".
 */
export type TextRewrite = {
  readonly text: string;
  readonly changed: readonly string[];
};

/**
 Whether a slice's archive span lies inside any span sealed as the English
 original.

 @param slice - prepared pair

 @param archiveOriginalSpans - sealed spans

 @returns Whether the slice overlaps a sealed span

 @example
 ```ts
 sealed({ slice, archiveOriginalSpans: [], },); // false
 ```
 */
function sealed(
  {
    slice,
    archiveOriginalSpans,
  }: {
    readonly slice: ChunkPair;
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
  },
): boolean {
  return archiveOriginalSpans.some(function overlaps(span,): boolean {
    return (span.startOffset
      < slice.target
        .endOffset) && (span.endOffset
          > slice.target
            .startOffset);
  },);
}

/**
 Applies one text rewrite to every slice the page carries, the slices no
 lane replaced included.

 @param slices - prepared pairs, whose archive text stands where no row replaces it

 @param replacements - what the page would write per slice

 @param archiveOriginalSpans - spans sealed as the English original

 @param rewrite - rewrite of one slice's text, given the slice's original and
 the archive's text for it

 @param findingName - name each finding opens with

 @returns Replacements with the rewrite applied (a row added for an archive
 slice it changed), the rewritten rows alone, and one finding per slice
 changed

 @example
 ```ts
 const page = rewriteEverySlice({ slices, replacements, archiveOriginalSpans: [], rewrite, findingName: 'form-rewritten', },);
 ```
 */
export function rewriteEverySlice(
  {
    slices,
    replacements,
    archiveOriginalSpans,
    rewrite,
    findingName,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
    readonly rewrite: (input: {
      readonly text: string;
      readonly source: string;
      readonly archive: string;
    },) => TextRewrite;
    readonly findingName: string;
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Replacement text by slice.
   */
  const replaced = new Map(replacements.map(function toEntry(row,): readonly [
    number,
    string,
  ] {
    return [
      row.sliceIndex,
      row.replacementText,
    ];
  },),);
  /**
   Rows the rewrite changed, with the finding each carries.
   */
  const changedRows = slicesInOrder({ slices, },)
    .filter(function eligible(slice,): boolean {
      return (slice.syntax !== 'front-matter') && (!sealed({
        slice,
        archiveOriginalSpans,
      },));
    },)
    .flatMap(function rewriteSlice(slice,): readonly {
      readonly row: SliceReplacement;
      readonly finding: string;
    }[] {
      /**
       Text the page carries at this slice.
       */
      const text = replaced.get(slice.target
        .sliceIndex,)
        ?? slice.target
          .text;
      /**
       That text rewritten.
       */
      const rewritten = rewrite({
        text,
        source: slice.source
          .text,
        archive: slice.target
          .text,
      },);
      if (rewritten.changed
        .length
        === 0)
        return [];
      return [{
        row: {
          sliceIndex: slice.target
            .sliceIndex,
          replacementText: rewritten.text,
        },
        finding: `${findingName} (slice ${String(slice.target
          .sliceIndex,)}: ${rewritten.changed
            .join(', ',)})`,
      },];
    },);
  /**
   Rewritten row by slice.
   */
  const rewrittenRows = new Map(changedRows.map(function toEntry({ row, },): readonly [
    number,
    SliceReplacement,
  ] {
    return [
      row.sliceIndex,
      row,
    ];
  },),);
  return {
    replacements: [
      ...replacements.map(function swap(row,): SliceReplacement {
        return rewrittenRows.get(row.sliceIndex,) ?? row;
      },),
      ...changedRows
        .filter(function added({ row, },): boolean {
          return !replaced.has(row.sliceIndex,);
        },)
        .map(function rowOf({ row, },): SliceReplacement {
          return row;
        },),
    ],
    restored: changedRows.map(function rowOf({ row, },): SliceReplacement {
      return row;
    },),
    findings: changedRows.map(function findingOf({ finding, },): string {
      return finding;
    },),
  };
}
//endregion Page slice rewrite
