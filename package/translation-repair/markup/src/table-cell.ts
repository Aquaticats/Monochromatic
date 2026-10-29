import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Table cell
// ONE READ OF A CELL IN A TABLE BUILT TO ITS OWN BOUNDS. The alignment tables
// (heading affinity and trust, the heading paths' score tables, the block
// walk's table) hold one row per index and one cell per column, and every read
// stays inside them, so a read always finds its cell; one outside them is a
// broken invariant, which throws rather than reading as a default.

/**
 Cell at one row and column of a table built to hold every cell its reader
 asks for.

 @param table - rows of cells

 @param row - row inside the table

 @param column - column inside the row

 @returns Cell

 @throws Error when the table has no such row or the row no such cell

 @example
 ```ts
 tableCell({ table: [[1, 2,], [3, 4,],], row: 1, column: 0, },); // 3
 ```
 */
export function tableCell<CellT,>(
  {
    table,
    row,
    column,
  }: {
    readonly table: readonly (readonly CellT[])[];
    readonly row: number;
    readonly column: number;
  },
): CellT {
  return nonNullishOrThrow(nonNullishOrThrow(table[row],)[column],);
}

//endregion Table cell
