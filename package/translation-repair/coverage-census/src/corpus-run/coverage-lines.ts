import { SourceMap, } from 'node:module';
import {
  relative,
  resolve,
} from 'node:path';
import { fileURLToPath, } from 'node:url';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { lineStartsOf, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { parseModelJson, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { UncalledFunction, } from './coverage-tally.ts';

//region Coverage lines
// Ledger T8: carries a bundle offset V8 counted back to the source line it was
// built from, through the source map the coverage build writes beside each
// chunk. A map entry names the NEAREST MAPPED POSITION AT OR BEFORE the one
// asked about. Under the minifier a stretch can therefore start a line late,
// on the return where it folds a log call and that return into one statement
// (`src/repair-chunk.ts` on 2026-09-29), so triage reads the bundle text beside
// the source lines. A cold stretch, which can cross modules, is split by
// source in `coverage-pieces.ts` (ledger M67).

/**
 A source map file that does not read as a version 3 map.

 @example
 ```ts
 throw new SourceMapFileError({ path: 'dist/final/node/index.mjs.map', says: 'it has no mappings', },);
 ```
 */
export class SourceMapFileError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names a file of the package's
   own build and a shape this module describes.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Builds the refusal naming the file and what did not read.

   @param path - map file read

   @param says - what in it did not read

   @example
   ```ts
   new SourceMapFileError({ path: 'dist/final/node/index.mjs.map', says: 'it has no mappings', },);
   ```
   */
  constructor({
    path,
    says,
  }: {
    readonly path: string;
    readonly says: string;
  },) {
    super({
      says: `source map ${path} does not read as a version 3 map: ${says}; build with the coverage config, which writes one beside every chunk`,
    },);
    this.name = 'SourceMapFileError';
  }
}

/**
 One bundle's text positions and map, enough to name the source line under
 any offset.

 @example
 ```ts
 const lines = bundleLinesOf({ text, map: readSourceMap({ path, text: mapText, },), mapDirectory, packageDirectory, },);
 ```
 */
export type BundleLines = {
  /**
   Offset of each line's first character.
   */
  readonly lineStarts: readonly number[];

  /**
   The bundle's source map.
   */
  readonly map: SourceMap;

  /**
   Directory the map's source paths are relative to.
   */
  readonly mapDirectory: string;

  /**
   Package directory the named sources are made relative to.
   */
  readonly packageDirectory: string;
};

/**
 The source line under a bundle offset, or that no mapping precedes it.

 @example
 ```ts
 const place: SourceLine = { kind: 'mapped', source: 'src/nap.ts', line: 12, };
 ```
 */
export type SourceLine = {
  readonly kind: 'mapped';

  /**
   Source file, relative to the package directory.
   */
  readonly source: string;

  /**
   Its 1-based line.
   */
  readonly line: number;
} | { readonly kind: 'unmapped'; };

/**
 The source a map entry names, as the map writes it, with the 1-based line;
 or that none is named.

 @example
 ```ts
 const entry: SourceEntry = { kind: 'mapped', written: '../../../src/nap.ts', line: 4, };
 ```
 */
export type SourceEntry = {
  readonly kind: 'mapped';

  /**
   Source as the map writes it, relative to the map or a file URL.
   */
  readonly written: string;

  /**
   Its 1-based line.
   */
  readonly line: number;
} | { readonly kind: 'unmapped'; };

/**
 An uncalled function with the source line it starts on.

 @example
 ```ts
 const mapped: MappedFunction = { ...uncalled, at: place, };
 ```
 */
export type MappedFunction = UncalledFunction & {
  /**
   Source line of its first character.
   */
  readonly at: SourceLine;
};

/**
 The source map format version the bundler writes and Node reads.
 */
const SOURCE_MAP_VERSION = 3;

/**
 Reads a map file into Node's source map reader, keeping the sources' names
 and dropping their contents, which the census never reads.

 @param path - map file read, named in the refusal

 @param text - its contents

 @returns Map, and the sources it names as written in it

 @throws SourceMapFileError where the file is not JSON (a write cut short
 leaves one), or is not a version 3 map with named sources and mappings; the
 refusal never repeats the parser's message, which quotes the text it refused

 @example
 ```ts
 const { map, sources, } = readSourceMap({ path, text, },);
 ```
 */
export function readSourceMap(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): {
  readonly map: SourceMap;
  readonly sources: readonly string[];
} {
  /**
   Parse attempt over the file, its failure taken as data.
   */
  const attempt = parseModelJson({ text, },);
  if (!attempt.parsed)
    throw new SourceMapFileError({
      path,
      says: 'it is not JSON',
    },);
  /**
   The file as JSON.
   */
  const parsed = attempt.value;
  if (!isJsonRecord(parsed,))
    throw new SourceMapFileError({
      path,
      says: 'it is not an object',
    },);
  /**
   The fields a version 3 map carries.
   */
  const {
    version,
    sources,
    names,
    mappings,
    file,
  } = parsed;
  if (version !== SOURCE_MAP_VERSION)
    throw new SourceMapFileError({
      path,
      says: `its version is not ${String(SOURCE_MAP_VERSION,)}`,
    },);
  if ((!isJsonArray(sources,)) || (!sources.every(function isName(source,): source is string {
    return (typeof source) === 'string';
  },)))
    throw new SourceMapFileError({
      path,
      says: 'its sources are not a list of names',
    },);
  if ((typeof mappings) !== 'string')
    throw new SourceMapFileError({
      path,
      says: 'it has no mappings',
    },);
  return {
    map: new SourceMap({
      version,
      file: (typeof file) === 'string' ? file : '',
      sources: [...sources,],
      sourcesContent: [],
      names: isJsonArray(names,)
        ? names.filter(function isName(name,): name is string {
          return (typeof name) === 'string';
        },)
        : [],
      mappings,
      sourceRoot: '',
    },),
    sources,
  };
}

/**
 Reads one bundle's positions and map.

 @param text - bundle text

 @param map - its source map

 @param mapDirectory - directory the map's source paths are relative to

 @param packageDirectory - directory sources are named from

 @returns What `sourceLineAt` needs

 @example
 ```ts
 const lines = bundleLinesOf({ text, map, mapDirectory, packageDirectory, },);
 ```
 */
export function bundleLinesOf(
  {
    text,
    map,
    mapDirectory,
    packageDirectory,
  }: {
    readonly text: string;
    readonly map: SourceMap;
    readonly mapDirectory: string;
    readonly packageDirectory: string;
  },
): BundleLines {
  return {
    lineStarts: lineStartsOf({ text, },),
    map,
    mapDirectory,
    packageDirectory,
  };
}

/**
 The bundle line holding an offset.

 @param lines - bundle's positions and map

 @param offset - bundle offset V8 reported

 @returns Its 0-based line and that line's start

 @throws where the offset is negative, which no line holds and V8 never
 reports

 @example
 ```ts
 const { line, lineStart, } = bundleLineAt({ lines, offset: 120, },);
 ```
 */
export function bundleLineAt(
  {
    lines,
    offset,
  }: {
    readonly lines: BundleLines;
    readonly offset: number;
  },
): {
  readonly line: number;
  readonly lineStart: number;
} {
  /**
   Line holding the offset, 0-based; -1 only for a negative offset, whose
   start the lookup then refuses.
   */
  const line = lines.lineStarts
    .findLastIndex(function startsAtOrBefore(start,): boolean {
      return start <= offset;
    },);
  return {
    line,
    lineStart: nonNullishOrThrow(lines.lineStarts[line],),
  };
}

/**
 The source a map entry names at a position. None is named before the first
 mapping, or under a segment that names no source, which a map may write for
 code the bundler generated: Node then returns an entry whose source is
 undefined, though its declaration types the field as always a string
 (`lib/internal/source_map/source_map.js` in Node 26.10.0 stores such a
 segment as its two generated coordinates alone), so the fields are read as
 unknown.

 @param lines - bundle's positions and map

 @param line - 0-based bundle line

 @param column - 0-based column on it

 @returns Source as written and its 1-based line, or that none is named

 @example
 ```ts
 const entry = entryAt({ lines, line: 3, column: 0, },);
 ```
 */
export function entryAt(
  {
    lines,
    line,
    column,
  }: {
    readonly lines: BundleLines;
    readonly line: number;
    readonly column: number;
  },
): SourceEntry {
  /**
   Source and line of the nearest mapping at or before the position, absent
   where none precedes it.
   */
  const {
    originalSource,
    originalLine,
  }: {
    readonly originalSource?: unknown;
    readonly originalLine?: unknown;
  } = lines.map
    .findEntry(
    line,
    column,
    );
  if (((typeof originalSource) !== 'string') || ((typeof originalLine) !== 'number'))
    return { kind: 'unmapped', };
  return {
    kind: 'mapped',
    written: originalSource,
    line: originalLine + 1,
  };
}

/**
 Names a source as the map writes it from the package directory.

 @param lines - bundle's positions and map

 @param written - source as the map writes it, relative to the map or a file
 URL

 @returns Source file relative to the package

 @example
 ```ts
 const source = packageSourceOf({ lines, written: '../../../src/nap.ts', },);
 ```
 */
export function packageSourceOf(
  {
    lines,
    written,
  }: {
    readonly lines: BundleLines;
    readonly written: string;
  },
): string {
  return relative(
    lines.packageDirectory,
    written.startsWith('file:',)
      ? fileURLToPath(written,)
      : resolve(
        lines.mapDirectory,
        written,
      ),
  );
}

/**
 Names the source line under one bundle offset.

 @param lines - bundle's positions and map

 @param offset - bundle offset V8 reported

 @returns Source file relative to the package and its 1-based line, or that no
 mapping names a source there

 @example
 ```ts
 const place = sourceLineAt({ lines, offset: 120, },);
 ```
 */
export function sourceLineAt(
  {
    lines,
    offset,
  }: {
    readonly lines: BundleLines;
    readonly offset: number;
  },
): SourceLine {
  /**
   The bundle line holding it.
   */
  const {
    line,
    lineStart,
  } = bundleLineAt({
    lines,
    offset,
  },);
  /**
   The source named there.
   */
  const entry = entryAt({
    lines,
    line,
    column: offset - lineStart,
  },);
  if (entry.kind === 'unmapped')
    return entry;
  return {
    kind: 'mapped',
    source: packageSourceOf({
      lines,
      written: entry.written,
    },),
    line: entry.line,
  };
}

/**
 Names the source line an uncalled function starts on.

 @param lines - bundle's positions and map

 @param uncalled - function in that bundle

 @returns Function with its line

 @example
 ```ts
 const mapped = mapFunction({ lines, uncalled, },);
 ```
 */
export function mapFunction(
  {
    lines,
    uncalled,
  }: {
    readonly lines: BundleLines;
    readonly uncalled: UncalledFunction;
  },
): MappedFunction {
  return {
    ...uncalled,
    at: sourceLineAt({
      lines,
      offset: uncalled.start,
    },),
  };
}

//endregion Coverage lines
