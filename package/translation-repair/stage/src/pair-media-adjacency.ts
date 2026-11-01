import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  ContainerSpan,
  DocumentNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type BlockPair,
  readBlockPairing,
} from './pair-blocks-wire.ts';
import { photoReferences, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Media-adjacent block claims
// A target transcript can precede or follow media marker that is its only
// source. When roster pairs same marker across both sides, target-only blocks in
// that bounded gap belong in marker's slice so picture readings reach quality
// stages. Ambiguous gaps remain unclaimed and fail closed later.

/**
 Pairing after structural media claims, with count-only findings.

 @example
 ```ts
 const claimed: MediaAdjacentClaim = { pairs: [], findings: [], };
 ```
 */
export type MediaAdjacentClaim = {
  /**
   Original pairs plus safely claimed target blocks.
   */
  readonly pairs: readonly BlockPair[];

  /**
   One finding per claimed run.
   */
  readonly findings: readonly string[];
};

/**
 Inclusive run of unclaimed target indices.
 */
type TargetRun = {
  /**
   First target index.
   */
  readonly start: number;

  /**
   Last target index.
   */
  readonly end: number;
};

/**
 Sentinel for run with no unambiguous media owner.
 */
const MEDIA_OWNER_UNRESOLVED: unique symbol = Symbol('media owner unresolved');

/**
 Finds contiguous target-index runs no pair claims.

 @param targetCount - target block count

 @param claimed - target indices already paired

 @returns Unclaimed runs in target order

 @example
 ```ts
 const runs = unclaimedRuns({ targetCount: 4, claimed: new Set([0, 3]), });
 ```
 */
function unclaimedRuns(
  {
    targetCount,
    claimed,
  }: {
    readonly targetCount: number;
    readonly claimed: ReadonlySet<number>;
  },
): readonly TargetRun[] {
  /**
   Runs found so far.
   */
  const runs: TargetRun[] = [];
  /**
   Last possible target index.
   */
  const lastTargetIndex = targetCount - 1;
  /**
   Target index under inspection.
   */
  let cursor = 0;
  while (cursor < targetCount) {
    if (claimed.has(cursor,)) {
      cursor += 1;
      continue;
    }
    /**
     First unclaimed target in this run.
     */
    const start = cursor;
    while (cursor < lastTargetIndex) {
      /**
       Candidate next target in this run.
       */
      const next = cursor + 1;
      if (claimed.has(next,))
        break;
      cursor = next;
    }
    runs.push({
      start,
      end: cursor,
    },);
    cursor += 1;
  }
  return runs;
}

/**
 Reports whether paired blocks share at least one named picture asset.

 @param pair - source and target indices

 @param sourceBlocks - source block list

 @param targetBlocks - target block list

 @returns Whether both blocks name same asset

 @example
 ```ts
 const shared = pairSharesMedia({ pair, sourceBlocks, targetBlocks, });
 ```
 */
function pairSharesMedia(
  {
    pair,
    sourceBlocks,
    targetBlocks,
  }: {
    readonly pair: BlockPair;
    readonly sourceBlocks: readonly DocumentNode[];
    readonly targetBlocks: readonly DocumentNode[];
  },
): boolean {
  // A PAIR NAMES BLOCKS OF THESE LISTS: the roster's reads are range-checked
  // against them by `readBlockPairing`, and `prepareBlockPairing` checks a
  // cached record against them through `readBlockPairing` itself, the
  // whole reader a reply goes through, before it claims media over the record,
  // missing on one that does not fit.
  /**
   Source block this pair names.
   */
  const source = nonNullishOrThrow(sourceBlocks[pair.source],);
  /**
   Target block this pair names.
   */
  const target = nonNullishOrThrow(targetBlocks[pair.target],);
  /**
   Source asset names.
   */
  const sourceAssets = new Set(photoReferences({ text: source.text, })
    .map(function toAsset(reference,): string {
      return reference.assetName;
    },),);
  if (sourceAssets.size === 0)
    return false;
  return photoReferences({ text: target.text, })
    .some(function sharesAsset(reference,): boolean {
      return sourceAssets.has(reference.assetName,);
    },);
}

/**
 Reports whether run exactly occupies one details container.

 @param run - unclaimed target run

 @param targetBlocks - parsed target blocks

 @param targetContainers - parsed target containers

 @returns Whether run is explicit archive transcript container

 @example
 ```ts
 const enclosed = isDetailsRun({ run, targetBlocks, targetContainers, });
 ```
 */
function isDetailsRun(
  {
    run,
    targetBlocks,
    targetContainers,
  }: {
    readonly run: TargetRun;
    readonly targetBlocks: readonly DocumentNode[];
    readonly targetContainers: readonly ContainerSpan[];
  },
): boolean {
  // A RUN LIES INSIDE THESE BLOCKS: `unclaimedRuns` walks indices below
  // `targetBlocks.length`.
  /**
   First block in run.
   */
  const first = nonNullishOrThrow(targetBlocks[run.start],);
  /**
   Last block in run.
   */
  const last = nonNullishOrThrow(targetBlocks[run.end],);
  return targetContainers.some(function enclosesRun(container,): boolean {
    return (container.name === 'details')
      && (container.openerStartOffset === first.startOffset)
      && (container.closerEndOffset === last.endOffset);
  },);
}

/**
 Finds one source owning run from media anchors on run boundaries.

 @param run - unclaimed target run

 @param pairs - roster-agreed pairs

 @param mediaPairs - subset sharing media

 @returns Source index when every media boundary names same source

 @example
 ```ts
 const owner = mediaOwner({ run, pairs, mediaPairs, });
 ```
 */
function mediaOwner(
  {
    run,
    pairs,
    mediaPairs,
  }: {
    readonly run: TargetRun;
    readonly pairs: readonly BlockPair[];
    readonly mediaPairs: ReadonlySet<BlockPair>;
  },
): number | typeof MEDIA_OWNER_UNRESOLVED {
  /**
   Claimed pair immediately before run.
   */
  const before = pairs.findLast(function beforeRun(pair,): boolean {
    return pair.target < run.start;
  },);
  /**
   Claimed pair immediately after run.
   */
  const after = pairs.find(function afterRun(pair,): boolean {
    return pair.target > run.end;
  },);
  /**
   Boundary media pairs, absent for non-media boundaries.
   */
  const mediaBoundaries = [
    before,
    after,
  ].filter(function isMedia(pair,): pair is BlockPair {
    return (pair !== undefined) && mediaPairs.has(pair,);
  },);
  /**
   First distinct source the media boundaries name, and any others: one
   owner exactly, or the run stays unclaimed.
   */
  const [owner, ...others] = new Set(mediaBoundaries.map(function toSource(pair,): number {
    return pair.source;
  },),);
  if ((owner === undefined) || (others.length > 0))
    return MEDIA_OWNER_UNRESOLVED;
  return owner;
}

/**
 Claims unpaired target runs next to source-matched media marker.

 @param pairs - roster-agreed block correspondences

 @param sourceBlocks - parsed source blocks

 @param targetBlocks - parsed target blocks

 @param targetContainers - parsed target containers proving transcript boundary

 @returns Pairing widened only across unambiguous media transcript gaps

 @throws BlockPairingError if widened result is not monotone

 @example
 ```ts
 const claim = claimMediaAdjacentTargets({ pairs, sourceBlocks, targetBlocks, targetContainers, });
 ```
 */
export function claimMediaAdjacentTargets(
  {
    pairs,
    sourceBlocks,
    targetBlocks,
    targetContainers,
  }: {
    readonly pairs: readonly BlockPair[];
    readonly sourceBlocks: readonly DocumentNode[];
    readonly targetBlocks: readonly DocumentNode[];
    readonly targetContainers: readonly ContainerSpan[];
  },
): MediaAdjacentClaim {
  /**
   Existing target claims.
   */
  const claimed = new Set(pairs.map(function toTarget(pair,): number {
    return pair.target;
  },),);
  /**
   Existing pairs whose blocks share media.
   */
  const mediaPairs = new Set(pairs.filter(function sharesMedia(pair,): boolean {
    return pairSharesMedia({
      pair,
      sourceBlocks,
      targetBlocks,
    },);
  },),);
  if (mediaPairs.size === 0) {
    return {
      pairs,
      findings: [],
    };
  }

  /**
   Target runs no roster pair claims.
   */
  const runs = unclaimedRuns({
    targetCount: targetBlocks.length,
    claimed,
  },);

  /**
   New pairs contributed by unambiguous media gaps.
   */
  const added = runs.flatMap(function claimRun(run,): readonly BlockPair[] {
    if (!isDetailsRun({
      run,
      targetBlocks,
      targetContainers,
    },))
      return [];
    /**
     Source marker owning this run.
     */
    const source = mediaOwner({
      run,
      pairs,
      mediaPairs,
    },);
    if (source === MEDIA_OWNER_UNRESOLVED)
      return [];
    /**
     Distance from first to last target.
     */
    const inclusiveSpan = run.end - run.start;
    /**
     Count of targets in run.
     */
    const runLength = inclusiveSpan + 1;
    return Array.from(
      { length: runLength, },
      function toPair(
        _unused,
        offset,
      ): BlockPair {
        return {
          source,
          target: run.start + offset,
        };
      },
    );
  },);
  if (added.length === 0) {
    return {
      pairs,
      findings: [],
    };
  }

  /**
   Original and structural pairs in monotone order.
   */
  const widened = [
    ...pairs,
    ...added,
  ].toSorted(function bySourceThenTarget(
    left,
    right,
  ): number {
    /**
     Source ordering, zero when both name same source.
     */
    const sourceOrder = left.source - right.source;
    if (sourceOrder !== 0)
      return sourceOrder;
    return left.target - right.target;
  },);
  /**
   Result re-read through same monotonicity guard model replies use.
   */
  const checked = readBlockPairing({
    value: { pairs: widened, },
    sourceCount: sourceBlocks.length,
    targetCount: targetBlocks.length,
  },);
  return {
    pairs: checked,
    findings: added.map(function toFinding(pair,): string {
      return `media-adjacent source ${String(pair.source,)} claims target ${String(pair.target,)}`;
    },),
  };
}

//endregion Media-adjacent block claims
