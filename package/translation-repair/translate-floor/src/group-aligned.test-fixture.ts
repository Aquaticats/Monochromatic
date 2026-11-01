/**
 Test-only grouping of an aligned block pair with nothing sealed, through
 the grouper the slicing calls (ledger B30). A wrapper doing this shipped as
 package source, and the tests called it while production always passes the
 archive's seals to `groupNodesSealed`; the tests now read that function.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type {
  AlignmentStep,
  DocumentNode,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  type AlignedRun,
  groupNodesSealed,
} from '../dist/final/node/index.mjs';

/**
 Runs `groupNodesSealed` makes of an aligned pair when the archive's note
 seals nothing.

 @param sourceNodes - original blocks in document order

 @param targetNodes - translation blocks in document order

 @param sourceBudget - original-side character budget per slice

 @param targetBudget - translation-side character budget per slice

 @param steps - roster's pairing as steps, when the case has one

 @returns Runs covering every block on both sides exactly once

 @throws {@link Error} when the grouper reports an original sealed away
 although nothing was sealed

 @example
 ```ts
 const runs = groupWithNothingSealed({ sourceNodes, targetNodes, sourceBudget: 900, targetBudget: 1600, },);
 ```
 */
export function groupWithNothingSealed(
  {
    sourceNodes,
    targetNodes,
    sourceBudget,
    targetBudget,
    steps,
  }: {
    readonly sourceNodes: readonly DocumentNode[];
    readonly targetNodes: readonly DocumentNode[];
    readonly sourceBudget: number;
    readonly targetBudget: number;
    readonly steps?: readonly AlignmentStep[];
  },
): readonly AlignedRun[] {
  /**
   The grouper's answer with an empty seal set.
   */
  const {
    runs,
    sealedSourceIds,
  } = groupNodesSealed({
    sourceNodes,
    targetNodes,
    sourceBudget,
    targetBudget,
    ...((steps === undefined) ? {} : { steps, }),
    sealed: new Set<string>(),
  },);
  if (sealedSourceIds.size > 0)
    throw new Error(`grouping with nothing sealed reported ${String(sealedSourceIds.size,)} sealed originals`,);
  return runs;
}
