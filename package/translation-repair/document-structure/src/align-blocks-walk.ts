import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  estimateExpansion,
  GAP_PENALTY,
  scorePairing,
} from './align-blocks.ts';
import type { DocumentNode, } from './document-node.ts';
import { tableCell, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Block alignment walk
// The monotone alignment itself: a Needleman-Wunsch walk over two block lists,
// scoring candidate partnerships with `scorePairing` and paying `GAP_PENALTY`
// to leave a block unpartnered. Monotone because translations preserve order:
// blocks may be dropped, added, or merged, but never reordered wholesale, so a
// crossing alignment would be evidence of a different failure entirely.
//
// The walk reports skips rather than hiding them. The drift this replaces left
// no trace at all in any artifact, which is why it survived until a human
// graded the output.

/**
 One step of the alignment: a partnered pair, or a block skipped on one side.

 @example
 ```ts
 const step: AlignmentStep = { kind: 'paired', sourceIndex: 7, targetIndex: 6, };
 ```
 */
export type AlignmentStep =
  | {
    /**
     Both sides contributed a block.
     */
    readonly kind: 'paired';

    /**
     Original-side block index.
     */
    readonly sourceIndex: number;

    /**
     Translation-side block index.
     */
    readonly targetIndex: number;
  }
    | {
    /**
     The original carries a block the translation does not.
     */
      readonly kind: 'source-only';

    /**
     Original-side block index.
     */
      readonly sourceIndex: number;

    /**
     Whether this block CONTINUES the pairing of the step before it.

     Set when a translation MERGES several originals into one block, so the
     second and later originals ride along with the rendering that covers
     them. The mirror of the same field on `target-only`.
     */
      readonly continuesPairing?: true;
    }
    | {
    /**
     The translation carries a block the original does not.
     */
      readonly kind: 'target-only';

    /**
     Translation-side block index.
     */
      readonly targetIndex: number;

    /**
     Whether this block CONTINUES the pairing of the step before it, rather
     than standing alone.

     Set only by `blockPairingToSteps`, where one original rendered by several
     translation blocks becomes a `paired` step followed by continuations. The
     grouper must not cut between them: separating a rendering from the
     original it renders puts a passage in front of the critics with no source
     beside it, which is the mispairing this whole path exists to end.

     The deterministic walk never sets it, because a block it skips genuinely
     stands alone.
     */
      readonly continuesPairing?: true;
    };

/**
 Cell of the score table plus the move that produced it.
 */
type Cell = {
  /**
   Best cumulative score reaching this cell.
   */
  readonly score: number;

  /**
   Move taken to reach it, `start` only at the origin.
   */
  readonly move: 'start' | 'pair' | 'skip-source' | 'skip-target';
};

/**
 Builds the score table for the two block lists. Row zero and column zero are
 pure gap runs, so a document whose counterpart is empty aligns as all skips
 rather than failing.

 @param sourceNodes - original blocks in document order

 @param targetNodes - translation blocks in document order

 @returns Filled table with one extra row and column for the empty prefixes
 */
function buildTable(
  {
    sourceNodes,
    targetNodes,
  }: {
    readonly sourceNodes: readonly DocumentNode[];
    readonly targetNodes: readonly DocumentNode[];
  },
): readonly (readonly Cell[])[] {
  /**
   How far THIS translation expands, estimated once over both whole lists.

   Per document rather than per pair, and per pair is impossible anyway: the
   pairing is what the table is deciding. A fixed constant made every correct
   pair look implausible on entries whose translator writes long, which is what
   `doc/audit/the-critics-are-shown-the-wrong-paragraph.md` measured.
   */
  const expansion = estimateExpansion({
    sourceNodes,
    targetNodes,
  },);

  /**
   Mutable table under construction; rows are built in order and never
   revisited once complete.
   */
  const table: Cell[][] = [];
  for (let row = 0; row <= sourceNodes.length; row += 1) {
    /**
     Row being filled.
     */
    const cells: Cell[] = [];
    for (let column = 0; column <= targetNodes.length; column += 1) {
      if ((row === 0) && (column === 0)) {
        cells.push({
          score: 0,
          move: 'start',
        },);
        continue;
      }
      if (row === 0) {
        cells.push({
          score: GAP_PENALTY * column,
          move: 'skip-target',
        },);
        continue;
      }
      if (column === 0) {
        cells.push({
          score: GAP_PENALTY * row,
          move: 'skip-source',
        },);
        continue;
      }

      /**
       Original block this cell considers, present by the loop bounds.
       */
      const sourceNode = nonNullishOrThrow(sourceNodes[row - 1],);

      /**
       Translation block this cell considers, present by the loop bounds.
       */
      const targetNode = nonNullishOrThrow(targetNodes[column - 1],);

      /**
       Score of the cell diagonally before this one, a row already built.
       */
      const { score: diagonal, } = tableCell({
        table,
        row: row - 1,
        column: column - 1,
      },);

      /**
       Score of the cell above this one, a row already built.
       */
      const { score: above, } = tableCell({
        table,
        row: row - 1,
        column,
      },);

      /**
       Score of the cell to the left, built earlier in this row.
       */
      const { score: left, } = nonNullishOrThrow(cells[column - 1],);

      /**
       Score for partnering the two blocks.
       */
      const pairScore = diagonal + scorePairing({
        source: sourceNode,
        target: targetNode,
        expansion,
      },);

      /**
       Score for leaving the original's block unpartnered.
       */
      const skipSourceScore = above + GAP_PENALTY;

      /**
       Score for leaving the translation's block unpartnered.
       */
      const skipTargetScore = left + GAP_PENALTY;

      /**
       Best of the three moves; pairing wins ties so the alignment stays as
       connected as the scores allow.
       */
      const best = Math.max(
        pairScore,
        skipSourceScore,
        skipTargetScore,
      );
      cells.push({
        score: best,
        move: best === pairScore
          ? 'pair'
          : (best === skipSourceScore
            ? 'skip-source'
            : 'skip-target'),
      },);
    }
    table.push(cells,);
  }
  return table;
}

/**
 Move recorded at the traceback's position in a filled table.

 @param table - filled score table

 @param cursor - position inside it

 @returns Move that produced that cell

 @example
 ```ts
 const move = moveAt({ table, cursor: { row: 2, column: 1, }, },);
 ```
 */
function moveAt(
  {
    table,
    cursor,
  }: {
    readonly table: readonly (readonly Cell[])[];
    readonly cursor: {
      readonly row: number;
      readonly column: number;
    };
  },
): Cell['move'] {
  /**
   Cell at the position.
   */
  const { move, } = tableCell({
    table,
    row: cursor.row,
    column: cursor.column,
  },);
  return move;
}

/**
 Aligns two block lists monotonically, skipping rather than forcing a partner
 where no partner fits. Order is preserved on both sides.

 @param sourceNodes - original blocks in document order

 @param targetNodes - translation blocks in document order

 @returns Steps in document order, covering every block on both sides exactly
 once

 @example
 ```ts
 const steps = alignBlocks({ sourceNodes, targetNodes, },);
 ```
 */
export function alignBlocks(
  {
    sourceNodes,
    targetNodes,
  }: {
    readonly sourceNodes: readonly DocumentNode[];
    readonly targetNodes: readonly DocumentNode[];
  },
): readonly AlignmentStep[] {
  /**
   Filled score table.
   */
  const table = buildTable({
    sourceNodes,
    targetNodes,
  },);

  /**
   Steps recovered from the table, built backwards then reversed.
   */
  const reversed: AlignmentStep[] = [];

  /**
   Position in the table, walked backwards from the far corner. A mutable
   record rather than two loose bindings, so the traceback's state is one
   named thing.
   */
  const cursor = {
    row: sourceNodes.length,
    column: targetNodes.length,
  };
  // Row zero's cells skip targets, column zero's skip sources, and every other
  // cell pairs or skips, so each move steps toward the origin, the one cell
  // whose move is `start`.
  for (
    let move = moveAt({
      table,
      cursor,
    },);
    move !== 'start';
    move = moveAt({
      table,
      cursor,
    },)
  ) {
    if (move === 'pair') {
      reversed.push({
        kind: 'paired',
        sourceIndex: cursor.row - 1,
        targetIndex: cursor.column - 1,
      },);
      cursor.row -= 1;
      cursor.column -= 1;
      continue;
    }
    if (move === 'skip-source') {
      reversed.push({
        kind: 'source-only',
        sourceIndex: cursor.row - 1,
      },);
      cursor.row -= 1;
      continue;
    }
    reversed.push({
      kind: 'target-only',
      targetIndex: cursor.column - 1,
    },);
    cursor.column -= 1;
  }
  return reversed.toReversed();
}

//endregion Block alignment walk
