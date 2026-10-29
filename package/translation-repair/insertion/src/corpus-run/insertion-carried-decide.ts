import {
  nonemptyOrThrow,
  nonNullishOrThrow,
} from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { CarriedInsertion, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { AnchorTarget, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  abutting,
  type AnchorHolder,
  anchorRegion,
  type RegionAnchoring,
} from './insertion-carried-anchor.ts';
import {
  type PairedNeighbour,
  pairedNeighbours,
} from './insertion-carried-neighbours.ts';
import { shiftReceiver, } from './insertion-carried-shift.ts';

//region Carried insertion fold decision
// WHICH NEIGHBOUR CARRIES A PASSAGE. The rule is in the fold's own region
// comment (insertion-carried-fold.ts): every evidence block placed in a
// paired slice next to the carried one, the carrier the neighbour holding the
// larger share of the quoted text (the earlier on a tie), the two sources
// abutting across blank space alone. Every stand-aside names its reason.
// Where the carrier's span renders the passage alone, the decision is a
// shift (insertion-carried-shift.ts, class one hundred seventy-nine).
//
// A DECISION CARRIES THE SLICES IT DECIDED ON: the carrier and receiver as
// the neighbour search found them, paired, and the carried slice as read.
// The fold then looks nothing up again, and no lookup that cannot miss is
// guarded (T8 batch 14).

/**
 One carried passage's fold decision.
 */
export type FoldDecision =
  | {
    readonly kind: 'fold';

    /**
     Paired neighbour whose source widens over the passage.
     */
    readonly carrier: PairedNeighbour;

    /**
     The carried slice as prepared.
     */
    readonly carried: ChunkPair;
  }
    | {
    /**
     Fold where the carrier's archive span renders the passage alone, so the
     carrier's own source moves to its far neighbour (class one hundred
     seventy-nine).
     */
      readonly kind: 'shift';

    /**
     Paired neighbour that takes the passage's source.
     */
      readonly carrier: PairedNeighbour;

    /**
     Paired slice on the carrier's far side that takes the carrier's own
     source.
     */
      readonly receiver: PairedNeighbour;

    /**
     The carried slice as prepared.
     */
      readonly carried: ChunkPair;
    }
    | {
      readonly kind: 'aside';

    /**
     Why the passage stays carried.
     */
      readonly reason: string;
    };

/**
 A region the anchoring placed.
 */
type PlacedRegion = Extract<RegionAnchoring, { readonly anchored: true; }>;

/**
 A region the anchoring could not place, with why.
 */
type UnplacedRegion = Extract<RegionAnchoring, { readonly anchored: false; }>;

/**
 One neighbour's share of the anchored text.
 */
type NeighbourShare = {
  /**
   The neighbour.
   */
  readonly neighbour: PairedNeighbour;

  /**
   Code points of the anchored text its span holds.
   */
  readonly codePoints: number;
};

/**
 The neighbour holding the larger share of the anchored text, the earlier on
 a tie.

 @param holders - every block's holder with its share

 @param neighbours - paired slices next to the carried one, earlier first,
 which hold every holder's block

 @returns Carrier

 @example
 ```ts
 const carrier = carrierAmong({ holders, neighbours, },);
 ```
 */
function carrierAmong(
  {
    holders,
    neighbours,
  }: {
    readonly holders: readonly AnchorHolder[];
    readonly neighbours: readonly PairedNeighbour[];
  },
): PairedNeighbour {
  /**
   Each neighbour's share of the quoted text.
   */
  const shares = neighbours.map(function shareOf(neighbour,): NeighbourShare {
    return {
      neighbour,
      codePoints: holders
        .filter(function held(holder,): boolean {
          return holder.position === neighbour.position;
        },)
        .reduce(function sum(
          total,
          holder,
        ): number {
          return total + holder.codePoints;
        },
        0,
        ),
    };
  },);
  /**
   The largest share, the earlier neighbour kept on a tie.
   */
  const largest = shares.reduce(function larger(
    best,
    next,
  ): NeighbourShare {
    return (next.codePoints > best.codePoints) ? next : best;
  },);
  return largest.neighbour;
}

/**
 Decides whether one carried passage folds into a neighbour.

 @param slices - prepared slices as they stand

 @param sourceText - whole original

 @param target - archive parsed for anchoring

 @param candidate - carried passage under decision

 @returns Fold into the carrier, a shift, or standing aside with the reason

 @example
 ```ts
 const decision = decideFold({ slices, sourceText, target, candidate, },);
 ```
 */
export function decideFold(
  {
    slices,
    sourceText,
    target,
    candidate,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly target: AnchorTarget;
    readonly candidate: CarriedInsertion;
  },
): FoldDecision {
  // A CARRIED VERDICT RESTS ON EVIDENCE. `judgeCoverage` calls a passage
  // carried only on a majority of full votes, and its evidence is those
  // votes' matched regions, so a carried passage always names one.
  /**
   Distinct regions the voices anchored.
   */
  const regions = nonemptyOrThrow([...new Set(candidate.evidence,),],);
  /**
   Where each region sits, block by block.
   */
  const anchorings = regions.map(function anchorOne(region,): RegionAnchoring {
    return anchorRegion({
      slices,
      target,
      region,
    },);
  },);
  /**
   The first region the anchoring could not place, if any.
   */
  const unplaced = anchorings.find(function isUnplaced(anchoring,): anchoring is UnplacedRegion {
    return !anchoring.anchored;
  },);
  if (unplaced !== undefined) {
    return {
      kind: 'aside',
      reason: `evidence ${unplaced.reason}`,
    };
  }
  /**
   Every block's holder across every region, all of which are placed.
   */
  const holders = anchorings
    .filter(function isPlaced(anchoring,): anchoring is PlacedRegion {
      return anchoring.anchored;
    },)
    .flatMap(function holdersOf(anchoring,): readonly AnchorHolder[] {
      return anchoring.holders;
    },);
  /**
   Nearest paired slices on either side, earlier first; carried passages in
   a row (mikaela14) are looked through.
   */
  const neighbours = pairedNeighbours({
    slices,
    position: candidate.position,
  },);
  /**
   A block held by a slice that is not a neighbour, if any.
   */
  const stray = holders.find(function isStray(holder,): boolean {
    return !neighbours.some(function holds(neighbour,): boolean {
      return neighbour.position === holder.position;
    },);
  },);
  if (stray !== undefined) {
    return {
      kind: 'aside',
      reason: `evidence sits in slice at position ${String(stray.position,)}, not a neighbour`,
    };
  }
  /**
   The neighbour holding most of the evidence.
   */
  const carrier = carrierAmong({
    holders,
    neighbours,
  },);
  /**
   The carried slice as prepared: the recorded insertion, since the rows are
   built from the insertion slices at these positions, the admission's
   positions index this preparation, and a fold rewrites only a carrier's
   source, keeping every slice where it stands.
   */
  const carried = nonNullishOrThrow(slices[candidate.position],);
  /**
   Whether the two sources abut across blank space alone.
   */
  const touching = abutting({
    sourceText,
    one: carrier.slice
      .source,
    other: carried.source,
  },);
  if (!touching) {
    /**
     Stable index the carrier reports under.
     */
    const carrierIndex = carrier.slice
      .target
      .sliceIndex;
    return {
      kind: 'aside',
      reason: `the original writes more than blank space between the carried source and slice ${String(carrierIndex,)}`,
    };
  }
  /**
   The far neighbour taking the carrier's own source, where the carrier's
   span renders the passage alone.
   */
  const [receiver,] = shiftReceiver({
    slices,
    sourceText,
    target,
    holders,
    carrier,
    carriedPosition: candidate.position,
  },);
  if (receiver !== undefined) {
    return {
      kind: 'shift',
      carrier,
      receiver,
      carried,
    };
  }
  return {
    kind: 'fold',
    carrier,
    carried,
  };
}

//endregion Carried insertion fold decision
