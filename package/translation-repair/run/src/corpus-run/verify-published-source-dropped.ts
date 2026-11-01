import { namedSlices, } from '@monochromatic-dev/translation-repair-page-check/ts';

//region Verify published source dropped
// THE LINE `verify-published` PRINTS FOR A PAGE SHIPPED WITHOUT THE ORIGINAL OF SOME SLICES, because no writing of
// those originals makes the grammar accept the page (the defect `page-source-unparseable`, `source-ships-compose.ts`
// in `page-check`). PRINTS THE ENTRY, THE DEFECT NAME AND SLICE INDICES, never a passage.

/**
 Lines the report prints for the slices a page was shipped without.

 @param entryId - person entry the page belongs to

 @param slices - slice indices whose original the page lacks, none for a page carrying every original

 @returns One line naming the entry, the defect and the slices; none for a page carrying every original

 @example
 ```ts
 const lines = sourceDroppedLinesOf({ entryId: 'CatEntry1', slices: [88, 91,], },);
 ```
 */
export function sourceDroppedLinesOf(
  {
    entryId,
    slices,
  }: {
    readonly entryId: string;
    readonly slices: readonly number[];
  },
): readonly string[] {
  if (slices.length === 0)
    return [];
  return [
    `${entryId}: page-source-unparseable, the page ships without the source text of slices ${
      namedSlices({ indices: slices, },)
    }`,
  ];
}

//endregion Verify published source dropped
