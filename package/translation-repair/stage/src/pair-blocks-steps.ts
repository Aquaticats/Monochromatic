import type { AlignmentStep, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  assertPairsNameBlocks,
  type BlockPair,
} from './pair-blocks-wire.ts';
import { pairUnpartneredGaps, } from './unpartnered-gap-steps.ts';

//region Block pairing steps
// TRANSLATES A ROSTER'S PAIRING INTO THE VOCABULARY THE GROUPER ALREADY READS,
// so nothing downstream of alignment learns that a model was involved.
//
// The step vocabulary is `paired`, `source-only` and `target-only`, one block
// per step. A pairing may name ONE original rendered by SEVERAL translation
// blocks, which is the correspondence the deterministic walk could not express
// and the whole reason for asking a model. It is carried here as the first
// correspondence `paired` and the rest `target-only`, which puts them in the
// same run as long as the budget holds, exactly as a skipped block already is.
//
// EVERY BLOCK APPEARS EXACTLY ONCE, on the side it belongs to, because the
// grouper measures characters per step and a block counted twice would inflate
// a run past its budget and cut the document somewhere it should not.
//
// A PAIRING THAT NAMES A BLOCK ITS SIDE LACKS IS REFUSED HERE, through
// `assertPairsNameBlocks` (`pair-blocks-wire.ts`), the check `readBlockPairing`
// reads a model's reply through. That reader checks a reply against the blocks
// its sheet numbered, but a pairing also arrives from a settled artifact's
// recipe (`corpus-run/artifact-two-lane-rebuild.ts`), which `parseBlockPairing`
// checks for shape, order and section, never against the blocks of the text it
// is then carved over, and that text's section may hold fewer blocks than the
// pairing names; `rebuildPreparation` reads the refusal as a moved carve.
// Converted unrefused, a pair naming a translation block past the last became
// a step naming no block and steps for every index before it, and a pair
// naming an original past the last vanished while its translation block was
// still counted as claimed, so that block reached no step at all. Every
// pairing passes through here on its way to the grouper, so this is the one
// place that sees them all.

/**
 Converts a pairing into monotone alignment steps covering both sides.

 @param pairs - correspondences the roster agreed on, in document order

 @param sourceCount - original blocks

 @param targetCount - translation blocks

 @returns Steps in document order, each block appearing exactly once

 @example
 ```ts
 const steps = bareBlockPairingSteps({ pairs, sourceCount: 12, targetCount: 16, },);
 ```
 */
function bareBlockPairingSteps(
  {
    pairs,
    sourceCount,
    targetCount,
  }: {
    readonly pairs: readonly BlockPair[];
    readonly sourceCount: number;
    readonly targetCount: number;
  },
): readonly AlignmentStep[] {
  /**
   Translation blocks each original is paired with, each once: a pair given
   twice names its translation block once, since every block appears once.
   */
  const targetsBySource = new Map<number, Set<number>>();
  for (const pair of pairs) {
    /**
     Targets recorded for this original so far.
     */
    const already = targetsBySource.get(pair.source,) ?? new Set<number>();
    already.add(pair.target,);
    targetsBySource.set(
      pair.source,
      already,
    );
  }

  /**
   Translation blocks some original claims.
   */
  const claimedTargets = new Set(pairs.map(function toTarget(pair,): number {
    return pair.target;
  },),);

  /**
   Steps in document order.
   */
  const steps: AlignmentStep[] = [];

  /**
   Translation blocks already emitted, so unpaired ones land in order.
   */
  let emittedTargets = 0;

  /**
   Emits every unclaimed translation block strictly before a boundary.

   @param before - first translation index NOT to emit

   @example
   ```ts
   emitUnclaimedTargetsBefore(3,);
   ```
   */
  function emitUnclaimedTargetsBefore(before: number,): void {
    while (emittedTargets < before) {
      if (!claimedTargets.has(emittedTargets,))
        steps.push({
          kind: 'target-only',
          targetIndex: emittedTargets,
        },);
      emittedTargets += 1;
    }
  }

  /**
   Translation blocks already carried by an earlier original.

   A translation that MERGES several originals into one block names that block
   against each of them. The first original pairs with it; the rest ride along
   as continuations, so their text reaches the same slice without the
   translation block being counted again.
   */
  const carriedTargets = new Set<number>();
  for (let source = 0; source < sourceCount; source += 1) {
    /**
     Translation blocks this original renders as, in order.
     */
    const targets = [ ...(targetsBySource.get(source,) ?? []), ]
      .toSorted(function ascending(
        left,
        right,
      ): number {
        return left - right;
      },);
    if (targets.length === 0) {
      steps.push({
        kind: 'source-only',
        sourceIndex: source,
      },);
      continue;
    }

    /**
     Whether an earlier original already claimed one of this original's
     renderings, which makes this original part of a merge however many
     further renderings it also has.

     Testing ANY rather than EVERY is what keeps a merge that then splits from
     losing its original: such an original's first rendering is carried, so a
     first-rendering-wins test never places it and the block leaves the
     document. That reached production once, deleting a closing message.
     */
    const anyCarried = targets.some(function isCarried(target,): boolean {
      return carriedTargets.has(target,);
    },);
    if (anyCarried) {
      steps.push({
        kind: 'source-only',
        sourceIndex: source,
        continuesPairing: true,
      },);
      for (const target of targets) {
        if (carriedTargets.has(target,))
          continue;
        emitUnclaimedTargetsBefore(target,);
        steps.push({
          // A FURTHER RENDERING of an original that already rides along. It
          // carries its own text and belongs in the same run as the merge.
          kind: 'target-only',
          targetIndex: target,
          continuesPairing: true,
        },);
        carriedTargets.add(target,);
        emittedTargets = target + 1;
      }
      continue;
    }
    // NONE OF THESE IS CARRIED YET: no earlier original claimed any of them
    // (`anyCarried`), and each appears once in the list.
    for (const [at, target,] of targets.entries()) {
      emitUnclaimedTargetsBefore(target,);
      steps.push((at === 0)
        ? {
          kind: 'paired',
          sourceIndex: source,
          targetIndex: target,
        }
        : {
          // THE SECOND AND LATER RENDERINGS of one original. They carry their
          // own text and must not re-count the original's characters, and they
          // must not be cut away from the original they render.
          kind: 'target-only',
          targetIndex: target,
          continuesPairing: true,
        },);
      carriedTargets.add(target,);
      emittedTargets = target + 1;
    }
  }
  emitUnclaimedTargetsBefore(targetCount,);
  return steps;
}

/**
 Converts a pairing into monotone alignment steps covering both sides, with
 every interior gap the roster left unplaced on both sides read as one merge
 (class one hundred twelve, mikaela15), so the grouper, the anchor reader and
 the decline reader all see such a gap the same way.

 @param pairs - correspondences the roster agreed on, in document order

 @param sourceCount - original blocks

 @param targetCount - translation blocks

 @returns Steps in document order, each block appearing exactly once

 @throws BlockPairingError when a pair names a block its side does not
 carry, which a stored pairing carved over another text can

 @example
 ```ts
 const steps = blockPairingToSteps({ pairs, sourceCount: 12, targetCount: 16, },);
 ```
 */
export function blockPairingToSteps(
  {
    pairs,
    sourceCount,
    targetCount,
  }: {
    readonly pairs: readonly BlockPair[];
    readonly sourceCount: number;
    readonly targetCount: number;
  },
): readonly AlignmentStep[] {
  assertPairsNameBlocks({
    pairs,
    sourceCount,
    targetCount,
  },);
  return pairUnpartneredGaps({
    steps: bareBlockPairingSteps({
      pairs,
      sourceCount,
      targetCount,
    },),
  },);
}

//endregion Block pairing steps
