import {
  type ArchiveOriginalSpan,
  archiveOriginalReadingOf,
} from './archive-original-note.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type ChunkPair,
  declinedTargetIdsOfPairing,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { BlockPair, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { RepairDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Preparation seal
// What the preparation asks about a seal, per document and per aligned chunk,
// split out of `document-preparation.ts` at its line budget. The rule lives in
// `archive-original-note.ts`; this is how the preparation applies it.

/**
 Spans the archive's notes seal, EMPTY when the caller asked for no seal, the
 archive carries no such note, or the note seals the whole page (which is
 the pass's business: it declines the entry before preparing it).

 @param document - parsed archive

 @param seal - whether the caller asked for the seal at all

 @returns Sealed spans in document order

 @example
 ```ts
 const spans = archiveOriginalSpansOf({ document: targetDocument, seal: true, },);
 ```
 */
export function archiveOriginalSpansOf(
  {
    document,
    seal,
  }: {
    readonly document: RepairDocument;
    readonly seal: boolean;
  },
): readonly ArchiveOriginalSpan[] {
  if (!seal)
    return [];
  /**
   What the archive's notes say.
   */
  const reading = archiveOriginalReadingOf({ document, },);
  return (reading.kind === 'spans') ? reading.spans : [];
}

/**
 Translation blocks of one aligned chunk that a seal covers.

 @param pair - aligned chunk

 @param sealedTargetIds - ids of every sealed block in the archive

 @returns Ids of this chunk's sealed blocks

 @example
 ```ts
 const sealedTargets = chunkSealedTargets({ pair, sealedTargetIds, },);
 ```
 */
export function chunkSealedTargets(
  {
    pair,
    sealedTargetIds,
  }: {
    readonly pair: ChunkPair;
    readonly sealedTargetIds: ReadonlySet<string>;
  },
): ReadonlySet<string> {
  return new Set(
    pair.target
      .nodes
      .map(function toId(node,): string {
        return node.id;
      },)
      .filter(function isSealed(id,): boolean {
        return sealedTargetIds.has(id,);
      },),
  );
}

/**
 Translation blocks the chunk's pairing accounted for nowhere, LESS the
 sealed ones: a sealed block leaves review by rule, not by the pairing's
 silence, and is not handed to the block correction round.

 Derived from the same pairing subdivision was handed, so the assertion and
 the carving cannot drift.

 @param pair - aligned chunk

 @param blockPairing - correspondences the roster agreed for it, absent when
 it agreed none

 @param sealedTargets - this chunk's sealed blocks

 @returns Ids of the declined blocks that are not sealed

 @example
 ```ts
 const declined = declinedLessSealed({ pair, blockPairing, sealedTargets, },);
 ```
 */
export function declinedLessSealed(
  {
    pair,
    blockPairing,
    sealedTargets,
  }: {
    readonly pair: ChunkPair;
    readonly blockPairing?: readonly BlockPair[];
    readonly sealedTargets: ReadonlySet<string>;
  },
): ReadonlySet<string> {
  if (blockPairing === undefined)
    return new Set<string>();
  return new Set(
    [
      ...declinedTargetIdsOfPairing({
        pairs: blockPairing,
        sourceNodes: pair.source
          .nodes,
        targetNodes: pair.target
          .nodes,
      },),
    ]
      .filter(function isNotSealed(id,): boolean {
        return !sealedTargets.has(id,);
      },),
  );
}

/**
 Alignment finding recording what one chunk's seal kept out of review, so the
 decision is legible from the artifact alone.

 @param pairIndex - aligned chunk

 @param sealedTargets - translation blocks sealed

 @param sealedSourceIds - originals sealed with them

 @returns Finding in the alignment channel's wording

 @example
 ```ts
 findings.push(sealedFinding({ pairIndex, sealedTargets, sealedSourceIds, },),);
 ```
 */
export function sealedFinding(
  {
    pairIndex,
    sealedTargets,
    sealedSourceIds,
  }: {
    readonly pairIndex: number;
    readonly sealedTargets: ReadonlySet<string>;
    readonly sealedSourceIds: ReadonlySet<string>;
  },
): string {
  /**
   Sealed block ids, in the order the set holds them.
   */
  const ids = [ ...sealedTargets, ]
    .join(', ',);
  return `alignment archive-original (pair ${String(pairIndex,)}: ${
    String(sealedTargets.size,)
  } translation ${
    wordForCount({
      count: sealedTargets.size,
      one: 'block',
      many: 'blocks',
    },)
  } and ${String(sealedSourceIds.size,)} original ${
    wordForCount({
      count: sealedSourceIds.size,
      one: 'block',
      many: 'blocks',
    },)
  } sealed by the archive's note, `
    + `shipped as the archive has them: ${ids})`;
}

//endregion Preparation seal
