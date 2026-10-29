import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type CarriedContainer,
  type ContainerReading,
  readCarriedContainers,
} from './insertion-container-blocks.ts';
import type { InsertionCoverageRow, } from './insertion-coverage-model.ts';

//region Insertion container deficit
// A CONTAINER THE ARCHIVE CARRIES SHORT OF BLOCKS. The whole-page budget
// corroborates an absent verdict by the page's size, and on a page whose
// translated part runs long it has nothing to spend: XingZ611 (2026-09-19)
// read the memorial disclosure block's third paragraph absent by a majority
// and refused it on an interior budget of zero, so it shipped as a recorded
// gap. The archive's block is a rendering of another revision of that
// memorial, three paragraphs where the original writes four, and the block's
// own shape is the deterministic second signal the budget could not give:
// inside one container the original has more blocks than the archive
// renders, so at least that many source blocks have no rendering of their
// own. An absent verdict inside such a container is admitted while the
// deficit lasts, in document order. A container the archive carries with as
// many blocks as the original admits nothing here: a merged rendering is
// what the roster's carried verdicts are for.
// THE DEFICIT DECIDES A SPLIT TOO (class sixty-nine, XingZ618, 2026-09-19):
// the rule first admitted absent verdicts alone, and the same third
// paragraph split on its next run, one voice anchoring a claim on an
// archive sentence the pairing never assigned, and stayed unresolved. A
// minority anchored claim is no majority, and a block the archive
// measurably lacks is the second signal; where a majority found the
// passage carried, the row stays out. The shape of class sixty in the tail.
//
// THE BLOCKS ARE THE PARSE'S (ledger B68), counted in
// `insertion-container-blocks.ts`, and a passage costs the blocks the
// container counted for its slice, so the deficit and what it is spent on
// are one measure.

/**
 Finding prefix an admitted passage is recorded under.

 @example
 ```ts
 findings.some((finding) => finding.startsWith(CONTAINER_DEFICIT_ADMITTED_FINDING));
 ```
 */
export const CONTAINER_DEFICIT_ADMITTED_FINDING: string = 'insertion-container-deficit-admitted';

/**
 Finding prefix a split verdict the deficit decided is recorded under.

 @example
 ```ts
 findings.some((finding) => finding.startsWith(SPLIT_IN_CONTAINER_DEFICIT_FINDING));
 ```
 */
export const SPLIT_IN_CONTAINER_DEFICIT_FINDING: string = 'insertion-split-in-container-deficit';

/**
 Finding prefix a container left uncounted is recorded under, because a
 half of one side would not parse (ledger B68).

 @example
 ```ts
 findings.some((finding) => finding.startsWith(CONTAINER_DEFICIT_UNREAD_FINDING));
 ```
 */
export const CONTAINER_DEFICIT_UNREAD_FINDING: string = 'insertion-container-deficit-unread';

/**
 Whether no majority found the passage carried: an absent majority, or a
 split (class sixty-nine; class forty-eight's unanchored split among them).

 @param row - unresolved coverage row

 @returns True when the verdict is absent or split

 @example
 ```ts
 const undecided = unresolvedRows.filter(noMajorityCarried);
 ```
 */
function noMajorityCarried(row: InsertionCoverageRow,): boolean {
  return (row.verdictKind === 'absent') || (row.verdictKind === 'split');
}

/**
 Finding naming a split the deficit decided, when the row is one with an
 anchored claim; an unanchored split is read as absent already (class
 forty-eight) and needs no second line.

 @param row - row the deficit admitted

 @returns One finding, or none

 @example
 ```ts
 const lines = splitFinding({ row, },);
 ```
 */
function splitFinding({ row, }: { readonly row: InsertionCoverageRow; },): readonly string[] {
  /**
   Whether any voice anchored a claim of coverage.
   */
  const anchored = (row.anchoredFull > 0) || (row.anchoredPartial > 0);
  if ((row.verdictKind !== 'split') || (!anchored))
    return [];
  return [
    `${SPLIT_IN_CONTAINER_DEFICIT_FINDING} (slice ${String(row.sliceIndex,)}, full ${String(row.anchoredFull,)}, partial ${
      String(row.anchoredPartial,)
    }, absent ${String(row.absentCount,)} of ${String(row.asked,)} asked; no majority, the block deficit decides)`,
  ];
}

/**
 Finding naming a container left uncounted and the slice the parse refused.

 @param reading - container as read, uncounted

 @returns One finding

 @example
 ```ts
 const line = unreadFinding({ reading, },);
 ```
 */
function unreadFinding(
  { reading, }: { readonly reading: Extract<ContainerReading, { readonly kind: 'unread'; }>; },
): string {
  /**
   Halves of the container.
   */
  const {
    open,
    close,
  } = reading.pair;
  /**
   Whose slice would not parse.
   */
  const whose = (reading.side === 'source') ? 'the original\'s' : 'the archive\'s';
  return `${CONTAINER_DEFICIT_UNREAD_FINDING} (${open.name} of slices ${String(open.sliceIndex,)} to ${
    String(close.sliceIndex,)
  }: ${whose} slice ${String(reading.sliceIndex,)} could not be read: ${reading.detail})`;
}

/**
 Admits passages no majority found carried inside one carried container
 while its block deficit lasts, in document order.

 @param container - carried container with its block counts

 @param unresolvedRows - rows neither admitted nor proven carried

 @returns Findings per admitted position

 @example
 ```ts
 const admitted = spendDeficit({ container, unresolvedRows, },);
 ```
 */
function spendDeficit(
  {
    container,
    unresolvedRows,
  }: {
    readonly container: CarriedContainer;
    readonly unresolvedRows: readonly InsertionCoverageRow[];
  },
): ReadonlyMap<number, readonly string[]> {
  /**
   Halves of the container.
   */
  const {
    open,
    close,
  } = container.pair;
  /**
   Blocks the original writes inside the container, by position.
   */
  const { sourceByPosition, } = container;
  /**
   Blocks the archive is short of, spent as rows are admitted.
   */
  let deficit = container.sourceBlocks - container.targetBlocks;
  /**
   Positions admitted here, with their findings.
   */
  const admitted = new Map<number, readonly string[]>();
  /**
   Rows no majority found carried inside this container, in document order.
   */
  const inside = unresolvedRows
    .filter(noMajorityCarried,)
    .filter(function within(row,): boolean {
      return (row.position >= open.position) && (row.position <= close.position);
    },)
    .toSorted(function byPosition(
      left,
      right,
    ): number {
      return left.position - right.position;
    },);
  for (const row of inside) {
    /**
     Blocks this row's slice writes, as the container counted them: the row
     lies between the halves, so the container read its slice.
     */
    const blocks = nonNullishOrThrow(sourceByPosition.get(row.position,),);
    /**
     Whether the deficit has room for this passage.
     */
    const affordable = (blocks > 0) && (blocks <= deficit);
    if (!affordable)
      continue;
    deficit -= blocks;
    admitted.set(
      row.position,
      [
        ...splitFinding({ row, },),
        `${CONTAINER_DEFICIT_ADMITTED_FINDING} (slice ${String(row.sliceIndex,)} inside ${open.name} of slices ${
          String(open.sliceIndex,)
        } to ${String(close.sliceIndex,)}: the original writes ${String(container.sourceBlocks,)} ${
          wordForCount({
            count: container.sourceBlocks,
            one: 'block',
            many: 'blocks',
          },)
        } there, `
          + `the archive ${String(container.targetBlocks,)})`,
      ],
    );
  }
  return admitted;
}

/**
 Admits passages no majority found carried inside a container the archive
 carries with fewer blocks than the original writes there, while the
 deficit lasts.

 @param slices - prepared slices, whose source names the containers

 @param positions - positions admitted on their own evidence

 @param unresolvedRows - rows neither admitted nor proven carried

 @returns Positions and unresolved rows after the deficit is spent, with a
 finding per passage admitted and per container left uncounted

 @example
 ```ts
 const deficit = admitContainerDeficit({ slices, positions, unresolvedRows, },);
 ```
 */
export function admitContainerDeficit(
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
   Positions admitted over every counted container, with their findings.
   */
  const admitted = new Map<number, readonly string[]>();
  /**
   One finding per container left uncounted.
   */
  const unread: string[] = [];
  for (const reading of readCarriedContainers({ slices, },)) {
    if (reading.kind === 'unread') {
      unread.push(unreadFinding({ reading, },),);
      continue;
    }
    for (
      const [
        position,
        findings,
      ] of spendDeficit({
        container: reading.container,
        unresolvedRows,
      },)
    )
      admitted.set(
        position,
        findings,
      );
  }
  return {
    positions: new Set([
      ...positions,
      ...admitted.keys(),
    ],),
    unresolvedRows: unresolvedRows.filter(function stillUnresolved(row,): boolean {
      return !admitted.has(row.position,);
    },),
    findings: [
      ...[...admitted.values(),].flat(),
      ...unread,
    ],
  };
}

//endregion Insertion container deficit
