import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { MappedFunction, } from './coverage-lines.ts';
import type { MappedStretch, } from './coverage-pieces.ts';

//region Coverage census report
// Ledger T8: what the census says about the unit suite, from the cold
// stretches and uncalled functions mapped to source lines. It splits the
// package's library source from the runner entry files the build names (their
// bodies are command-line mains) and from other workspace packages built into
// the bundles. How a run reads an earlier census, so a batch of tests can
// prove it reached the stretches it claims, is `coverage-census-baseline.ts`.
// A stretch that is nothing but invariant throws is no cold stretch here: the
// census places it apart (`coverage-census-place.ts`), so no row or total
// counts it, and `coverage-census-invariant.ts` lists it.

/**
 Every kind, in the order the report names them.

 THE TYPE IS DERIVED FROM THIS LIST, so a kind cannot be added to the type
 without being placed in the order, and the report that walks this list
 cannot skip it.

 @example
 ```ts
 const [first,] = SOURCE_KINDS; // 'library source'
 ```
 */
export const SOURCE_KINDS = [
  'library source',
  'entry file',
  'other package',
  'unmapped',
] as const;

/**
 Where a source sits for the census.

 @example
 ```ts
 const kind: SourceKind = 'library source';
 ```
 */
export type SourceKind = typeof SOURCE_KINDS[number];

/**
 A cold stretch as the census records it: bundle offsets and the source lines
 they cover.

 @example
 ```ts
 const stretch: CensusStretch = { bundle: 'index.mjs', start: 10, end: 20, name: '', source: 'src/nap.ts', startLine: 4, endLine: 6, };
 ```
 */
export type CensusStretch = {
  /**
   Bundle holding it.
   */
  readonly bundle: string;

  /**
   Its first bundle offset.
   */
  readonly start: number;

  /**
   One past its last.
   */
  readonly end: number;

  /**
   Name of the uncalled function it is exactly, empty for code inside a
   function that ran or an anonymous one.
   */
  readonly name: string;

  /**
   Source file relative to the package, or `(unmapped) <bundle>`.
   */
  readonly source: string;

  /**
   First source line it covers, 0 when unmapped.
   */
  readonly startLine: number;

  /**
   Last source line it covers.
   */
  readonly endLine: number;
};

/**
 Orders two census records by source, in code-point order, and then by first
 line, the order the reports list stretches in.

 @param left - one record's source and first line

 @param right - other's

 @returns Negative where `left` comes first, positive where `right` does,
 zero where the two share a source and a first line

 @example
 ```ts
 const ordered = stretches.toSorted(function bySourceThenLine(left, right,): number {
   return compareSourceThenLine({ left, right, },);
 },);
 ```
 */
export function compareSourceThenLine(
  {
    left,
    right,
  }: {
    readonly left: Pick<CensusStretch, 'source' | 'startLine'>;
    readonly right: Pick<CensusStretch, 'source' | 'startLine'>;
  },
): number {
  return compareCodePoints({
    left: left.source,
    right: right.source,
  },)
    || (left.startLine - right.startLine);
}

/**
 One source file's cold code.

 @example
 ```ts
 const row: SourceRow = { source: 'src/nap.ts', kind: 'library source', stretches: 2, lines: 5, uncalled: 1, };
 ```
 */
export type SourceRow = {
  /**
   Source file relative to the package.
   */
  readonly source: string;

  /**
   Where it sits.
   */
  readonly kind: SourceKind;

  /**
   Cold stretches in it.
   */
  readonly stretches: number;

  /**
   Distinct source lines they cover.
   */
  readonly lines: number;

  /**
   Uncalled functions starting in it.
   */
  readonly uncalled: number;
};

/**
 One kind's totals.

 @example
 ```ts
 const total: KindTotal = { kind: 'library source', files: 1, stretches: 2, lines: 5, uncalled: 1, };
 ```
 */
export type KindTotal = Omit<SourceRow, 'source'> & {
  /**
   Source files of this kind holding cold code.
   */
  readonly files: number;
};

/**
 What opens the source a census gives code no source map places. The code has
 no source, so the census names the bundle carrying it, marked so no source
 path reads the same.
 */
const UNMAPPED_MARK = '(unmapped) ';

/**
 The source a census gives code no source map places.

 @param bundle - bundle carrying it

 @returns `(unmapped) <bundle>`

 @example
 ```ts
 const source = unmappedSourceOf({ bundle: 'nap-AAAA.mjs', },);
 ```
 */
export function unmappedSourceOf({ bundle, }: { readonly bundle: string; },): string {
  return `${UNMAPPED_MARK}${bundle}`;
}

/**
 Whether a census source names code no source map places. Such code has no
 lines, and its name moves with its bundle's, so nothing matched by source and
 line reads it across a rebuild.

 @param source - source as a census records it

 @returns Whether it is `(unmapped) <bundle>`

 @example
 ```ts
 const unmapped = isUnmappedSource({ source: '(unmapped) nap-AAAA.mjs', },); // true
 ```
 */
export function isUnmappedSource({ source, }: { readonly source: string; },): boolean {
  return source.startsWith(UNMAPPED_MARK,);
}

/**
 Records a mapped stretch as one census stretch per piece, each over the
 source lines its characters map to (ledger M67: one record per stretch put
 every source after the first out of the census).

 @param stretch - stretch split into its pieces

 @returns The census records, in bundle order

 @example
 ```ts
 const recorded = censusStretchesOf({ stretch, },);
 ```
 */
export function censusStretchesOf({ stretch, }: { readonly stretch: MappedStretch; },): readonly CensusStretch[] {
  /**
   Uncalled function name, when the stretch is exactly one; it starts in the
   first piece.
   */
  const name = (stretch.shape
    .kind
    === 'function') ? stretch.shape
      .name : '';
  return stretch.pieces
    .map(function recordOf(
      piece,
      index,
    ): CensusStretch {
      /**
       The fields every record carries.
       */
      const placed = {
        bundle: stretch.bundle,
        start: piece.start,
        end: piece.end,
        name: (index === 0) ? name : '',
      };
      if (piece.span
        .kind
        === 'unmapped') {
        return {
          ...placed,
          source: unmappedSourceOf({ bundle: stretch.bundle, },),
          startLine: 0,
          endLine: 0,
        };
      }
      return {
        ...placed,
        source: piece.span
          .source,
        startLine: piece.span
          .firstLine,
        endLine: piece.span
          .lastLine,
      };
    },);
}

/**
 How many misplaced functions the refusal names; the count names the rest.
 */
const MISPLACED_NAMED = 5;

/**
 Refuses a census that places an uncalled function in no cold stretch of its
 own source. An uncalled function is cold throughout, so its first line must
 sit in one; one that does not names cold code the census recorded under
 another source, which is how ledger M67 hid `src/repair-chunk.ts`.

 @param stretches - census stretches

 @param uncalled - uncalled functions with their lines

 @throws StatedRefusalError naming up to five functions left uncovered

 @example
 ```ts
 requirePlacedFunctions({ stretches, uncalled, },);
 ```
 */
export function requirePlacedFunctions(
  {
    stretches,
    uncalled,
  }: {
    readonly stretches: readonly CensusStretch[];
    readonly uncalled: readonly MappedFunction[];
  },
): void {
  /**
   Uncalled functions no stretch of their own source covers.
   */
  const misplaced = uncalled.filter(function uncovered(fn,): boolean {
    /**
     Source and line the function starts on, as the census records them.
     */
    const {
      source,
      line,
    } = (fn.at
      .kind
      === 'mapped')
      ? fn.at
      : {
        source: unmappedSourceOf({ bundle: fn.bundle, },),
        line: 0,
      };
    return !stretches.some(function covers(stretch,): boolean {
      return (stretch.bundle === fn.bundle)
        && (stretch.source === source)
        && (stretch.startLine <= line)
        && (stretch.endLine >= line);
    },);
  },);
  if (misplaced.length === 0)
    return;
  /**
   The first few named by bundle, offset and name.
   */
  const named = misplaced
    .slice(
    0,
    MISPLACED_NAMED,
    )
    .map(function nameOf(fn,): string {
      return `${fn.bundle}:${String(fn.start,)} ${fn.name}`;
    },)
    .join(', ',);
  /**
   Which word names this many functions, for the refusal text.
   */
  const functionsWord = wordForCount({
    count: misplaced.length,
    one: 'function',
    many: 'functions',
  },);
  throw new StatedRefusalError({
    says: `the census places ${String(misplaced.length,)} uncalled ${functionsWord} in no cold stretch of the `
      + `source defining each `
      + `(${named}); an uncalled function is cold throughout, so the mapping is wrong and no report is written`,
  },);
}

/**
 A source as a package names it, or a source outside every package's `src`.

 @example
 ```ts
 const own: OwnSource = { kind: 'own', path: 'src/nap.ts', };
 ```
 */
export type OwnSource =
  | {
    /**
     A file under a package's `src`.
     */
    readonly kind: 'own';

    /**
     Its path inside its package, from `src/`.
     */
    readonly path: string;
  }
    | {
    /**
     A file elsewhere.
     */
      readonly kind: 'foreign';
    };

/**
 Names a source by its path inside its package, where it is one of a package's
 own: `src/<file>` as named from the package, or `<package>/src/<file>` as
 named from the family directory, which the census does where it measures the
 family.

 @param source - source as a census records it

 @returns Its path inside its package, or that it stands outside every
 package's `src`: another package of the repository (`../...`), a dependency,
 a build file or an unmapped bundle

 @example
 ```ts
 ownSourceOf({ source: 'stage/src/nap.ts', },); // { kind: 'own', path: 'src/nap.ts', }
 ```
 */
export function ownSourceOf({ source, }: { readonly source: string; },): OwnSource {
  if (source.startsWith('src/',))
    return {
      kind: 'own',
      path: source,
    };
  /**
   Where the package's name ends, if the source starts with one.
   */
  const slash = source.indexOf('/',);
  if (slash <= 0)
    return { kind: 'foreign', };
  /**
   What stands before the first slash.
   */
  const first = source.slice(
    0,
    slash,
  );
  if (first.startsWith('.',) || first.startsWith('(',))
    return { kind: 'foreign', };
  /**
   What stands after the first segment.
   */
  const rest = source.slice(slash + 1,);
  return rest.startsWith('src/',)
    ? {
      kind: 'own',
      path: rest,
    }
    : { kind: 'foreign', };
}

/**
 Says where a source sits.

 @param source - source relative to the package or to the family directory, or `(unmapped) <bundle>`

 @param entryFiles - sources the build names as runner entries

 @returns Its kind

 @example
 ```ts
 sourceKindOf({ source: 'src/nap.ts', entryFiles: new Set(), },); // 'library source'
 ```
 */
export function sourceKindOf(
  {
    source,
    entryFiles,
  }: {
    readonly source: string;
    readonly entryFiles: ReadonlySet<string>;
  },
): SourceKind {
  if (isUnmappedSource({ source, },))
    return 'unmapped';
  /**
   Its path inside its package.
   */
  const own = ownSourceOf({ source, },);
  if (own.kind === 'foreign')
    return 'other package';
  return entryFiles.has(own.path,) ? 'entry file' : 'library source';
}

/**
 One row per source holding cold code, most cold lines first.

 @param stretches - census stretches

 @param uncalled - uncalled functions with their lines

 @param entryFiles - sources the build names as runner entries

 @returns Rows, ties by source name

 @example
 ```ts
 const rows = sourceRowsOf({ stretches, uncalled, entryFiles, },);
 ```
 */
export function sourceRowsOf(
  {
    stretches,
    uncalled,
    entryFiles,
  }: {
    readonly stretches: readonly CensusStretch[];
    readonly uncalled: readonly MappedFunction[];
    readonly entryFiles: ReadonlySet<string>;
  },
): readonly SourceRow[] {
  /**
   Every source holding a stretch.
   */
  const sources = [...new Set(stretches.map(function sourceOf(stretch,): string {
    return stretch.source;
  },),),];
  return sources
    .map(function rowOf(source,): SourceRow {
      /**
       Its stretches.
       */
      const own = stretches.filter(function inSource(stretch,): boolean {
        return stretch.source === source;
      },);
      /**
       Distinct lines they cover.
       */
      const lines = new Set(own.flatMap(function linesOf(stretch,): readonly number[] {
        return Array.from(
          { length: (stretch.endLine - stretch.startLine) + 1, },
          function lineAt(
            _unused,
            index,
          ): number {
            return stretch.startLine + index;
          },
        );
      },),);
      return {
        source,
        kind: sourceKindOf({
          source,
          entryFiles,
        },),
        stretches: own.length,
        lines: lines.size,
        uncalled: uncalled.filter(function startsIn(fn,): boolean {
          return (fn.at
            .kind
            === 'mapped') && (fn.at
              .source
              === source);
        },)
          .length,
      };
    },)
    .toSorted(function mostLinesFirst(
      left,
      right,
    ): number {
      return (right.lines - left.lines)
        || compareCodePoints({
          left: left.source,
          right: right.source,
        },);
    },);
}

/**
 Totals per kind, in a fixed order.

 @param rows - source rows

 @returns One total per kind holding any row

 @example
 ```ts
 const totals = kindTotalsOf({ rows, },);
 ```
 */
export function kindTotalsOf({ rows, }: { readonly rows: readonly SourceRow[]; },): readonly KindTotal[] {
  return SOURCE_KINDS.flatMap(function totalOf(kind,): readonly KindTotal[] {
    /**
     Rows of this kind.
     */
    const own = rows.filter(function ofKind(row,): boolean {
      return row.kind === kind;
    },);
    if (own.length === 0)
      return [];
    return [{
      kind,
      files: own.length,
      stretches: own.reduce(
        function sum(
          total,
          row,
        ): number {
          return total + row.stretches;
        },
        0,
      ),
      lines: own.reduce(
        function sum(
          total,
          row,
        ): number {
          return total + row.lines;
        },
        0,
      ),
      uncalled: own.reduce(
        function sum(
          total,
          row,
        ): number {
          return total + row.uncalled;
        },
        0,
      ),
    },];
  },);
}

//endregion Coverage census report
