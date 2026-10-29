/**
 Tests the coverage census's mapping from bundle offsets to source lines
 (ledger T8): which map files it refuses, what it keeps of one it reads, line
 starts, a position before the first mapping read as unmapped rather than as
 line 1, sources named from the package whether the map writes them relative
 or as file URLs, a segment naming no source read as unmapped, a stretch split
 into one piece per source its characters map to (ledger M67), and a
 function's line. The maps are written by hand: in the one-source map, bundle
 line 0 has no mapping and lines 1 and 2 map to source lines 1 and 2; the
 two-source map and the sourceless one are described where they are declared.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { lineStartsOf, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  bundleLinesOf,
  mapFunction,
  mapStretch,
  readSourceMap,
  sourceLineAt,
  SourceMapFileError,
} from '../../dist/final/node/index.mjs';

/**
 Package directory the fixture names sources from.
 */
const PACKAGE = '/cattery';

/**
 Build directory beside it.
 */
const DIST = '/cattery/dist/final/node';

/**
 Bundle text: three lines, the first unmapped.
 */
const BUNDLE_TEXT = 'x\ny\nz';

/**
 A map whose bundle line 0 has no segment and whose lines 1 and 2 map to
 source lines 0 and 1 (0-based) of its one source.

 @param source - source as the map writes it

 @returns Map file's text
 */
function mapText({ source, }: { readonly source: string; },): string {
  return JSON.stringify({
    version: 3,
    sources: [source,],
    names: ['nap', 7,],
    mappings: ';AAAA;AACA',
  },);
}

/**
 The fixture bundle's lines, its map naming the given source.

 @param source - source as the map writes it

 @returns What sourceLineAt reads
 */
function linesWith({ source, }: { readonly source: string; },) {
  return bundleLinesOf({
    text: BUNDLE_TEXT,
    map: readSourceMap({
      path: 'nap.mjs.map',
      text: mapText({ source, },),
    },).map,
    mapDirectory: DIST,
    packageDirectory: PACKAGE,
  },);
}

/**
 Bundle text of two modules: an unmapped first line, one line of `nap.ts`,
 then two of `purr.ts`.
 */
const TWO_MODULES_TEXT = 'x\ny\nz\nw';

/**
 The two-module bundle's lines: line 1 maps to `nap.ts` line 0, lines 2 and 3
 to `purr.ts` lines 4 and 5 (0-based).
 */
const TWO_MODULES = bundleLinesOf({
  text: TWO_MODULES_TEXT,
  map: readSourceMap({
    path: 'nap.mjs.map',
    text: JSON.stringify({
      version: 3,
      sources: ['../../../src/nap.ts', '../../../src/purr.ts',],
      mappings: ';AAAA;ACIA;AACA',
    },),
  },).map,
  mapDirectory: DIST,
  packageDirectory: PACKAGE,
},);

/**
 A bundle whose second line holds a mapped character and then one the map
 marks with a segment naming no source, as a bundler may write for code it
 generated. The separator closing the mappings matters: Node 26.10.0 reads a
 one-field segment as sourceless only when a separator follows it, and at the
 end of the string reads it as continuing the previous source.
 */
const SOURCELESS = bundleLinesOf({
  text: 'x\nyz',
  map: readSourceMap({
    path: 'nap.mjs.map',
    text: JSON.stringify({
      version: 3,
      sources: ['../../../src/nap.ts',],
      mappings: ';AAAA,C;',
    },),
  },).map,
  mapDirectory: DIST,
  packageDirectory: PACKAGE,
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readSourceMap.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a map that is not an object, is not version 3, names sources that are not strings, or has no mappings',
          fn: async () => {
            for (const text of [
              'null',
              JSON.stringify({
                version: 2,
                sources: [],
                mappings: '',
              },),
              JSON.stringify({
                version: 3,
                sources: [1,],
                mappings: '',
              },),
              JSON.stringify({
                version: 3,
                mappings: '',
              },),
              JSON.stringify({
                version: 3,
                sources: [],
              },),
            ]) {
              expect(() => readSourceMap({
                path: 'nap.mjs.map',
                text,
              },),).toThrow(SourceMapFileError,);
            }
          },
        },),
        it({
          name: 'REFUSES A MAP CUT SHORT as its own refusal saying it is not JSON, naming the file and quoting none of '
            + 'its text, where the parser\'s refusal reached the census as a fault in the command',
          fn: async () => {
            const refusal = caught(function readsCutShortMap() {
              readSourceMap({
                path: 'nap.mjs.map',
                text: mapText({ source: '../../../src/nap.ts', },)
                  .slice(
                  0,
                  30,
                  ),
              },);
            },);
            expect(refusal,).toBeInstanceOf(SourceMapFileError,);
            expect(String(refusal,),).toBe(
              'SourceMapFileError: source map nap.mjs.map does not read as a version 3 map: it is not JSON; build '
                + 'with the coverage config, which writes one beside every chunk',
            );
          },
        },),
        it({
          name: 'KEEPS SOURCE NAMES AND DROPS THEIR CONTENTS, with no file named read as empty and names that are not strings dropped',
          fn: async () => {
            const { map, sources, } = readSourceMap({
              path: 'nap.mjs.map',
              text: JSON.stringify({
                ...JSON.parse(mapText({ source: '../../../src/nap.ts', },),),
                sourcesContent: ['the whole source',],
              },),
            },);
            expect(sources,).toEqual(['../../../src/nap.ts',],);
            expect(map.payload.sourcesContent,).toEqual([],);
            expect(map.payload.file,).toBe('',);
            expect(map.payload.names,).toEqual(['nap',],);
          },
        },),
        it({
          name: 'KEEPS THE FILE A MAP NAMES, and reads a map with no names list as naming none',
          fn: async () => {
            const { map, } = readSourceMap({
              path: 'nap.mjs.map',
              text: JSON.stringify({
                version: 3,
                file: 'nap.mjs',
                sources: ['../../../src/nap.ts',],
                mappings: ';AAAA',
              },),
            },);
            expect(map.payload.file,).toBe('nap.mjs',);
            expect(map.payload.names,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: lineStartsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STARTS AT 0 AND AFTER EVERY NEWLINE, a trailing newline opening an empty last line',
          fn: async () => {
            expect(lineStartsOf({ text: 'ab\nc\n', },),).toEqual([0, 3, 5,],);
            expect(lineStartsOf({ text: '', },),).toEqual([0,],);
          },
        },),
      ],
    },),

    describe({
      name: sourceLineAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A POSITION BEFORE THE FIRST MAPPING AS UNMAPPED, and one after it as its 1-based source line named from the package',
          fn: async () => {
            const lines = linesWith({ source: '../../../src/nap.ts', },);
            expect(sourceLineAt({
              lines,
              offset: 0,
            },),).toEqual({ kind: 'unmapped', },);
            expect(sourceLineAt({
              lines,
              offset: BUNDLE_TEXT.indexOf('y',),
            },),).toEqual({
              kind: 'mapped',
              source: 'src/nap.ts',
              line: 1,
            },);
            expect(sourceLineAt({
              lines,
              offset: BUNDLE_TEXT.indexOf('z',),
            },),).toEqual({
              kind: 'mapped',
              source: 'src/nap.ts',
              line: 2,
            },);
          },
        },),
        it({
          name: 'REFUSES A NEGATIVE OFFSET, which no line holds and V8 never reports',
          fn: async () => {
            expect(() => sourceLineAt({
              lines: linesWith({ source: '../../../src/nap.ts', },),
              offset: -1,
            },),).toThrow();
          },
        },),
        it({
          name: 'READS A SEGMENT NAMING NO SOURCE AS UNMAPPED, which Node reports as an entry whose source is undefined',
          fn: async () => {
            expect(sourceLineAt({
              lines: SOURCELESS,
              offset: 3,
            },),).toEqual({ kind: 'unmapped', },);
          },
        },),
        it({
          name: 'NAMES A SOURCE THE MAP WRITES AS A FILE URL from the package too',
          fn: async () => {
            expect(sourceLineAt({
              lines: linesWith({ source: `file://${PACKAGE}/src/purr.ts`, },),
              offset: BUNDLE_TEXT.indexOf('y',),
            },),).toEqual({
              kind: 'mapped',
              source: 'src/purr.ts',
              line: 1,
            },);
          },
        },),
      ],
    },),

    describe({
      name: mapStretch.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPLITS A STRETCH WHERE ITS SOURCE CHANGES, each piece with its own offsets and the lines its characters map to (ledger M67)',
          fn: async () => {
            const stretch = {
              bundle: 'nap.mjs',
              start: TWO_MODULES_TEXT.indexOf('y',),
              end: TWO_MODULES_TEXT.length,
              shape: { kind: 'block', },
            } as const;
            expect(mapStretch({
              lines: TWO_MODULES,
              stretch,
            },),).toEqual({
              ...stretch,
              pieces: [
                {
                  start: TWO_MODULES_TEXT.indexOf('y',),
                  end: TWO_MODULES_TEXT.indexOf('z',),
                  span: {
                    kind: 'mapped',
                    source: 'src/nap.ts',
                    firstLine: 1,
                    lastLine: 1,
                  },
                },
                {
                  start: TWO_MODULES_TEXT.indexOf('z',),
                  end: TWO_MODULES_TEXT.length,
                  span: {
                    kind: 'mapped',
                    source: 'src/purr.ts',
                    firstLine: 5,
                    lastLine: 6,
                  },
                },
              ],
            },);
          },
        },),
        it({
          name: 'KEEPS AN UNMAPPED RUN AS ITS OWN PIECE, before the first mapping or under a segment naming no source',
          fn: async () => {
            expect(mapStretch({
              lines: TWO_MODULES,
              stretch: {
                bundle: 'nap.mjs',
                start: 0,
                end: TWO_MODULES_TEXT.indexOf('y',) + 1,
                shape: { kind: 'block', },
              },
            },).pieces,).toEqual([
              {
                start: 0,
                end: TWO_MODULES_TEXT.indexOf('y',),
                span: { kind: 'unmapped', },
              },
              {
                start: TWO_MODULES_TEXT.indexOf('y',),
                end: TWO_MODULES_TEXT.indexOf('y',) + 1,
                span: {
                  kind: 'mapped',
                  source: 'src/nap.ts',
                  firstLine: 1,
                  lastLine: 1,
                },
              },
            ],);
            expect(mapStretch({
              lines: SOURCELESS,
              stretch: {
                bundle: 'nap.mjs',
                start: 2,
                end: 4,
                shape: { kind: 'block', },
              },
            },).pieces,).toEqual([
              {
                start: 2,
                end: 3,
                span: {
                  kind: 'mapped',
                  source: 'src/nap.ts',
                  firstLine: 1,
                  lastLine: 1,
                },
              },
              {
                start: 3,
                end: 4,
                span: { kind: 'unmapped', },
              },
            ],);
          },
        },),
        it({
          name: 'READS A ONE-CHARACTER STRETCH AS ONE PIECE ON ONE LINE, and refuses an empty one, which holds no character to map',
          fn: async () => {
            const lines = linesWith({ source: '../../../src/nap.ts', },);
            const start = BUNDLE_TEXT.indexOf('z',);
            expect(mapStretch({
              lines,
              stretch: {
                bundle: 'nap.mjs',
                start,
                end: start + 1,
                shape: { kind: 'block', },
              },
            },).pieces,).toEqual([{
              start,
              end: start + 1,
              span: {
                kind: 'mapped',
                source: 'src/nap.ts',
                firstLine: 2,
                lastLine: 2,
              },
            },],);
            expect(() => mapStretch({
              lines,
              stretch: {
                bundle: 'nap.mjs',
                start,
                end: start,
                shape: { kind: 'block', },
              },
            },),).toThrow();
          },
        },),
      ],
    },),

    describe({
      name: mapFunction.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE LINE AN UNCALLED FUNCTION STARTS ON',
          fn: async () => {
            const uncalled = {
              bundle: 'nap.mjs',
              start: BUNDLE_TEXT.indexOf('z',),
              end: BUNDLE_TEXT.length,
              name: 'nap',
              nested: false,
            };
            expect(mapFunction({
              lines: linesWith({ source: '../../../src/nap.ts', },),
              uncalled,
            },),).toEqual({
              ...uncalled,
              at: {
                kind: 'mapped',
                source: 'src/nap.ts',
                line: 2,
              },
            },);
          },
        },),
      ],
    },),
  ],
},);
