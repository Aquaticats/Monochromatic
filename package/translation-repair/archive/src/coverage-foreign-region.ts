import type { SpanAnchor, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Foreign target region
// MEASURED ON SHI_YUMIAOYA4 (2026-09-17): the coverage round for the death
// paragraphs heard one partial claim quoting the archive's farewell line,
// "girl's last tour ended", which the pairing had assigned to the original's
// own farewell slice. The quote anchored, the claim counted as partial
// coverage, the round split, and the passage shipped as a recorded gap on a
// SETTLED memorial page. English the pairing assigned to another source slice
// renders THAT original; a partial claim pointing at it says nothing about the
// passage asked about, so it is dropped: neither coverage nor a vote for
// absence. A FULL claim inside such a region is kept, since an archive that
// merged two originals into one paragraph carries the second inside the
// first's region, and that is exactly what a full claim there reports.

/**
 Target region the pairing assigned to some source slice, as offsets into the
 target text.

 @example
 ```ts
 const region: TargetRegion = { startOffset: 120, endOffset: 180, };
 ```
 */
export type TargetRegion = {
  /**
   Offset the region opens at.
   */
  readonly startOffset: number;

  /**
   Offset just past the region.
   */
  readonly endOffset: number;
};

/**
 Whether any located anchor overlaps a region paired to another source slice.

 OVERLAP RATHER THAN CONTAINMENT: a quote that starts inside a paired
 rendering and runs past it still points at English that renders a different
 original.

 @param anchors - located spans of one quote, in document order

 @param foreignRegions - target regions paired to other source slices

 @returns Whether the quote touches foreign text

 @example
 ```ts
 const foreign = anchorsInsideForeignRegion({ anchors, foreignRegions, },);
 ```
 */
export function anchorsInsideForeignRegion(
  {
    anchors,
    foreignRegions,
  }: {
    readonly anchors: readonly SpanAnchor[];
    readonly foreignRegions: readonly TargetRegion[];
  },
): boolean {
  return anchors.some(function touchesSome(anchor,): boolean {
    return foreignRegions.some(function overlaps(region,): boolean {
      return (anchor.startOffset < region.endOffset) && (anchor.endOffset > region.startOffset);
    },);
  },);
}

//endregion Foreign target region
