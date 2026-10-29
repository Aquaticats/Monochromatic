import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  bundleLineAt,
  type BundleLines,
  entryAt,
  packageSourceOf,
  type SourceEntry,
} from './coverage-lines.ts';
import type { ColdStretch, } from './coverage-tally.ts';

//region Coverage pieces
// Ledger T8: splits a cold stretch by the source each of its characters maps
// to, since a bundle chunk concatenates modules and one cold run can cross
// several. Every character is mapped, not only the first and last: V8's cold
// runs are merged across function and module boundaries, and nothing but the
// map says where one module ends.
//
// WHY EVERY CHARACTER (ledger M67). The census once mapped a stretch's two
// ends and recorded the whole under the first source; a subset census on
// 2026-09-29 put a stretch of 9,569 bundle characters, `repairChunk` among
// them, at one line of `src/select-candidate.ts`, and gave
// `src/repair-chunk.ts` no stretch at all. A subset run maps about 1.4 million
// cold characters, each one binary search in Node's map.

/**
 The source lines one piece of a cold stretch covers: the first and last
 lines its characters map to in one source, or that no source is named there.

 @example
 ```ts
 const span: SourceSpan = { kind: 'mapped', source: 'src/nap.ts', firstLine: 4, lastLine: 7, };
 ```
 */
export type SourceSpan = {
  readonly kind: 'mapped';

  /**
   Source file, relative to the package directory.
   */
  readonly source: string;

  /**
   Lowest 1-based line a character of the piece maps to.
   */
  readonly firstLine: number;

  /**
   Highest.
   */
  readonly lastLine: number;
} | { readonly kind: 'unmapped'; };

/**
 One run of a cold stretch whose characters all map to one source, or all to
 none.

 @example
 ```ts
 const piece: StretchPiece = { start: 10, end: 20, span, };
 ```
 */
export type StretchPiece = {
  /**
   Its first bundle offset.
   */
  readonly start: number;

  /**
   One past its last.
   */
  readonly end: number;

  /**
   The source lines it covers.
   */
  readonly span: SourceSpan;
};

/**
 A cold stretch split into its pieces, one per run of characters in one
 source, in bundle order.

 @example
 ```ts
 const mapped: MappedStretch = { ...stretch, pieces: [piece,], };
 ```
 */
export type MappedStretch = ColdStretch & {
  /**
   Its pieces, never none, since a stretch holds a character.
   */
  readonly pieces: readonly [
    StretchPiece,
    ...StretchPiece[]
  ];
};

/**
 Where a run being walked maps: its source as the map writes it with the
 lines seen so far, or no source.
 */
type RunPlace = {
  readonly kind: 'mapped';
  readonly written: string;
  readonly firstLine: number;
  readonly lastLine: number;
} | { readonly kind: 'unmapped'; };

/**
 A run being walked.
 */
type Run = {
  readonly start: number;
  readonly end: number;
  readonly place: RunPlace;
};

/**
 Every character of a stretch with the source named at it, in bundle order.

 @param lines - bundle's positions and map

 @param stretch - stretch in that bundle

 @returns Each character's offset and the entry naming its source

 @example
 ```ts
 for (const { offset, entry, } of charactersOf({ lines, stretch, },)) console.log(offset, entry.kind,);
 ```
 */
function* charactersOf(
  {
    lines,
    stretch,
  }: {
    readonly lines: BundleLines;
    readonly stretch: ColdStretch;
  },
): Generator<{
  readonly offset: number;
  readonly entry: SourceEntry;
}> {
  /**
   Bundle line of its first character.
   */
  const { line: firstLine, } = bundleLineAt({
    lines,
    offset: stretch.start,
  },);
  /**
   Bundle line of its last.
   */
  const { line: lastLine, } = bundleLineAt({
    lines,
    offset: stretch.end - 1,
  },);
  for (let line = firstLine; line <= lastLine; line += 1) {
    /**
     Where this bundle line starts.
     */
    const lineStart = nonNullishOrThrow(lines.lineStarts[line],);
    /**
     One past the stretch's last character on this line: the next line's
     start, which a line before the last always has, or the stretch's end.
     */
    const upTo = (line === lastLine) ? stretch.end : nonNullishOrThrow(lines.lineStarts[line + 1],);
    for (let offset = Math.max(
      stretch.start,
      lineStart,
    ); offset < upTo; offset += 1) {
      yield {
        offset,
        entry: entryAt({
          lines,
          line,
          column: offset - lineStart,
        },),
      };
    }
  }
}

/**
 Whether a character's entry continues a run: the same source as the map
 writes it, or none after none.

 @param place - where the run maps

 @param entry - character's entry

 @returns Whether the two name one source

 @example
 ```ts
 continuesRun({ place: run.place, entry, },);
 ```
 */
function continuesRun(
  {
    place,
    entry,
  }: {
    readonly place: RunPlace;
    readonly entry: SourceEntry;
  },
): boolean {
  if (place.kind === 'unmapped')
    return entry.kind === 'unmapped';
  return (entry.kind === 'mapped') && (entry.written === place.written);
}

/**
 Where a run maps once a character it continues is added: a mapped run's
 lines widened to the character's.

 @param place - where the run maps

 @param entry - entry of a character that continues it

 @returns Place with the character's line counted

 @example
 ```ts
 const widened = widenedPlace({ place: run.place, entry, },);
 ```
 */
function widenedPlace(
  {
    place,
    entry,
  }: {
    readonly place: RunPlace;
    readonly entry: SourceEntry;
  },
): RunPlace {
  if ((place.kind === 'unmapped') || (entry.kind === 'unmapped'))
    return place;
  return {
    ...place,
    firstLine: Math.min(
      place.firstLine,
      entry.line,
    ),
    lastLine: Math.max(
      place.lastLine,
      entry.line,
    ),
  };
}

/**
 Where a run opened by a character maps.

 @param entry - character's entry

 @returns Its source and line as the run's lines, or no source

 @example
 ```ts
 const place = openedPlace({ entry, },);
 ```
 */
function openedPlace({ entry, }: { readonly entry: SourceEntry; },): RunPlace {
  if (entry.kind === 'unmapped')
    return entry;
  return {
    kind: 'mapped',
    written: entry.written,
    firstLine: entry.line,
    lastLine: entry.line,
  };
}

/**
 Splits a cold stretch into runs of characters mapping to one source each,
 recording the lines each run covers.

 @param lines - bundle's positions and map

 @param stretch - stretch in that bundle

 @returns Stretch with its pieces in bundle order

 @throws where the stretch holds no character, which the tally never yields

 @example
 ```ts
 const mapped = mapStretch({ lines, stretch, },);
 ```
 */
export function mapStretch(
  {
    lines,
    stretch,
  }: {
    readonly lines: BundleLines;
    readonly stretch: ColdStretch;
  },
): MappedStretch {
  /**
   Runs so far, the last still growing.
   */
  const runs: Run[] = [];
  for (const {
    offset,
    entry,
  } of charactersOf({
    lines,
    stretch,
  },)) {
    /**
     The run growing, absent before the first character.
     */
    const open = runs.at(-1,);
    if ((open !== undefined) && continuesRun({
      place: open.place,
      entry,
    },)) {
      runs[runs.length - 1] = {
        start: open.start,
        end: offset + 1,
        place: widenedPlace({
          place: open.place,
          entry,
        },),
      };
    }
    else {
      runs.push({
        start: offset,
        end: offset + 1,
        place: openedPlace({ entry, },),
      },);
    }
  }
  /**
   Each run as a piece, the map's source name read from the package.
   */
  const [first, ...rest] = runs.map(function pieceOf(run,): StretchPiece {
    return {
      start: run.start,
      end: run.end,
      span: (run.place
        .kind
        === 'unmapped')
        ? run.place
        : {
          kind: 'mapped',
          source: packageSourceOf({
            lines,
            written: run.place
              .written,
          },),
          firstLine: run.place
            .firstLine,
          lastLine: run.place
            .lastLine,
        },
    };
  },);
  return {
    ...stretch,
    pieces: [
      nonNullishOrThrow(first,),
      ...rest,
    ],
  };
}

//endregion Coverage pieces
