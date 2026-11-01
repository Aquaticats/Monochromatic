import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  bothHalvesInserted,
  type ChunkPair,
  type ContainerHalf,
  containerHalfPairs,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { InsertionCoverageRow, } from './insertion-coverage-model.ts';

//region Insertion container halves
// THE HALVES OF A CONTAINER ARE ADMITTED TOGETHER. The coverage round reads
// each source-only slice on its own, and on XingZ607 it split on the summary
// of a disclosure block (one voice anchored the poem's title elsewhere on the
// page) while every voice found the block's body absent and the page short of
// it. The body and the closing tag were admitted, the opening half was not,
// and the lane assembled a page with a closing tag and no opening. A container
// is one element: once any of its slices is admitted on its own evidence, the
// page is short of the element, and both halves ship with it.

/**
 Finding prefix an admitted half is recorded under.

 @example
 ```ts
 findings.some((finding) => finding.startsWith(CONTAINER_HALF_ADMITTED_FINDING));
 ```
 */
export const CONTAINER_HALF_ADMITTED_FINDING: string = 'insertion-container-half-admitted';

/**
 One half that follows an admitted slice of its container.
 */
type FollowingHalf = {
  /**
   Half admitted by its container.
   */
  readonly half: ContainerHalf;

  /**
   Slice admitted on its own evidence that carries the half in.
   */
  readonly beside: number;
};

/**
 Admits the unadmitted half of every container one of whose slices is admitted.

 @param slices - prepared slices, whose source names the pairs

 @param positions - positions admitted on their own evidence

 @param unresolvedRows - rows neither admitted nor proven carried

 @returns Positions and unresolved rows after the halves follow, with a finding per half admitted

 @example
 ```ts
 const halves = admitContainerHalves({ slices, positions, unresolvedRows, },);
 ```
 */
export function admitContainerHalves(
  {
    slices,
    positions,
    unresolvedRows,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly positions: ReadonlySet<number>;
    readonly unresolvedRows: readonly InsertionCoverageRow[];
  },
): {
  readonly positions: ReadonlySet<number>;
  readonly unresolvedRows: readonly InsertionCoverageRow[];
  readonly findings: readonly string[];
} {
  /**
   Halves that follow an admitted slice of their container, each beside the
   admitted slice that carries it in.
   */
  const following = containerHalfPairs({ slices, },)
    .filter(function inserted(pair,): boolean {
      return bothHalvesInserted({ pair, },);
    },)
    .flatMap(function unadmittedHalves(pair,): readonly FollowingHalf[] {
      /**
       Position of the opening half.
       */
      const openPosition = pair.open
        .position;
      /**
       Position of the closing half.
       */
      const closePosition = pair.close
        .position;
      /**
       First admitted position inside the container, opening half through
       closing half.
       */
      const admitted = [...positions,]
        .toSorted(function ascending(
          left,
          right,
        ): number {
          return left - right;
        },)
        .find(function inside(position,): boolean {
          return (position >= openPosition) && (position <= closePosition);
        },);
      if (admitted === undefined)
        return [];
      /**
       Prepared slice at the admitted position, whose index the finding
       names: the position lies between two of these slices' own positions.
       */
      const admittedSlice = nonNullishOrThrow(slices[admitted],);
      /**
       Slice index of the admitted position, read off the prepared slices.
       */
      const beside = admittedSlice.target
        .sliceIndex;
      return [
        pair.open,
        pair.close,
      ]
        .filter(function unadmitted(half,): boolean {
          return !positions.has(half.position,);
        },)
        .map(function toEntry(half,): FollowingHalf {
          return {
            half,
            beside,
          };
        },);
    },);
  /**
   Positions the halves add.
   */
  const added = new Set(following.map(function toPosition(entry,): number {
    return entry.half
      .position;
  },),);
  return {
    positions: new Set([
      ...positions,
      ...added,
    ],),
    unresolvedRows: unresolvedRows.filter(function stillUnresolved(row,): boolean {
      return !added.has(row.position,);
    },),
    findings: following.map(function toFinding(entry,): string {
      /**
       Slice the half belongs to.
       */
      const { sliceIndex, } = entry.half;
      return `${CONTAINER_HALF_ADMITTED_FINDING} (slice ${String(sliceIndex,)} beside slice ${
        String(entry.beside,)
      }: one container's halves ship together, and slice ${String(entry.beside,)} is admitted)`;
    },),
  };
}

//endregion Insertion container halves
