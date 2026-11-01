import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type CensusStretch,
  compareSourceThenLine,
  SOURCE_KINDS,
  type SourceKind,
  sourceKindOf,
} from './coverage-census-report.ts';

//region Coverage census invariant throws
// Ledger T8: the invariant throws of a census, which it counts apart from
// the cold stretches. A stretch no test ran that is nothing but throws of a
// broken invariant (`coverage-invariant-throw.ts` reads which) is no code a
// test should reach: no honest input gets there, by the invariant. Counted
// as cold it reads as work left for ever, and the one way to close it is to
// delete the guard, which ledger M112 recorded as a prevention and which
// inverts the package's rule that a broken invariant is loud. So no row and
// no total of cold code holds such a stretch, and the report prints their
// count beside the cold counts and names each by source and line, so a reader
// can audit what the census set aside.

/**
 A stretch that is nothing but invariant throws, as the census records it:
 the stretch, and what it throws.

 @example
 ```ts
 const apart: InvariantThrowStretch = { bundle: 'index.mjs', start: 10, end: 20, name: '', source: 'src/nap.ts', startLine: 4, endLine: 6, thrown: ['Error',], };
 ```
 */
export type InvariantThrowStretch = CensusStretch & {
  /**
   Class each throw of the stretch builds, in order: `Error` for a message
   beginning `unreachable:`, or a class named for a broken invariant.
   */
  readonly thrown: readonly string[];
};

/**
 One invariant throw as the report names it.

 @example
 ```ts
 const row: InvariantThrowRow = { source: 'src/nap.ts', kind: 'library source', startLine: 4, endLine: 6, thrown: ['Error',], };
 ```
 */
export type InvariantThrowRow = Pick<InvariantThrowStretch, 'endLine' | 'source' | 'startLine' | 'thrown'> & {
  /**
   Where its source sits.
   */
  readonly kind: SourceKind;
};

/**
 One row per invariant throw, in the order the report lists them.

 @param invariantThrows - stretches the census placed apart

 @param entryFiles - sources the build names as runner entries

 @returns Rows sorted by source and then line

 @example
 ```ts
 const rows = invariantThrowRowsOf({ invariantThrows, entryFiles, },);
 ```
 */
export function invariantThrowRowsOf(
  {
    invariantThrows,
    entryFiles,
  }: {
    readonly invariantThrows: readonly InvariantThrowStretch[];
    readonly entryFiles: ReadonlySet<string>;
  },
): readonly InvariantThrowRow[] {
  return invariantThrows
    .map(function rowOf({
      source,
      startLine,
      endLine,
      thrown,
    },): InvariantThrowRow {
      return {
        source,
        kind: sourceKindOf({
          source,
          entryFiles,
        },),
        startLine,
        endLine,
        thrown,
      };
    },)
    .toSorted(function bySourceThenLine(
      left,
      right,
    ): number {
      return compareSourceThenLine({
        left,
        right,
      },);
    },);
}

/**
 The report's count of invariant throws, printed beside the cold counts.

 @param rows - invariant throws of the census

 @returns One line: how many stretches, then each kind holding any with its
 count, in the report's order of kinds

 @example
 ```ts
 const line = invariantThrowCountLine({ rows, },); // 'invariant throws, counted apart from the cold stretches: 2 stretches (library source 2)'
 ```
 */
export function invariantThrowCountLine({ rows, }: { readonly rows: readonly InvariantThrowRow[]; },): string {
  /**
   Each kind holding an invariant throw, with how many.
   */
  const byKind = SOURCE_KINDS.flatMap(function countOf(kind,): readonly string[] {
    /**
     Invariant throws in sources of this kind.
     */
    const count = rows.filter(function ofKind(row,): boolean {
      return row.kind === kind;
    },)
      .length;
    return (count === 0) ? [] : [`${kind} ${String(count,)}`,];
  },);
  return `invariant throws, counted apart from the cold stretches: ${String(rows.length,)} ${
    wordForCount({
      count: rows.length,
      one: 'stretch',
      many: 'stretches',
    },)
  }${(byKind.length === 0) ? '' : ` (${byKind.join(', ',)})`}`;
}

/**
 What heads the report's list of invariant throws, saying what the census
 took for one, so a reader checking the list knows the rule it was drawn by.
 */
const LIST_HEADING = 'invariant throws by source and line (each stretch nothing but throws of an Error whose message '
  + 'begins "unreachable:" or of a class whose name ends in InvariantError), with what each throws:';

/**
 The report's list of invariant throws, each by source and line so a reader
 can audit what the census counted apart.

 @param rows - invariant throws of the census, in report order

 @returns The heading, then one line per invariant throw: its source, lines,
 the kind of its source and the classes it throws

 @example
 ```ts
 for (const line of invariantThrowListLines({ rows, },)) console.log(line,);
 ```
 */
export function invariantThrowListLines({ rows, }: { readonly rows: readonly InvariantThrowRow[]; },): readonly string[] {
  return [
    LIST_HEADING,
    ...rows.map(function listed(row,): string {
      return `  ${row.source}:${String(row.startLine,)}-${String(row.endLine,)} (${row.kind}): ${row.thrown
        .join(', ',)}`;
    },),
  ];
}

//endregion Coverage census invariant throws
