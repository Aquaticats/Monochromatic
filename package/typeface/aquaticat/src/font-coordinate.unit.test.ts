/**
 Glyph-space coordinates have one owner.

 The SVG-to-glyph transform, `(x - cellX) + xShift` on X and `fontY(y)` on Y, was
 spelled out at five call sites, and the coordinate pair had three spellings, so
 a metrics change meant editing five places and the tuple types could drift
 apart. `toFontCoordinate` and the `Coordinate` type are the single owner, and
 the guard cases below fail when a call site starts writing either one out again.
 */
/* oxlint-disable test-import/require-eventual-artifact -- this package ships src/ (package.json `files`) and builds font binaries rather than a JS artifact, so the modules beside this test are the shipped ones; see doc/planning/oxlint-test-import-eventual-artifact.md, section "Packages that ship source". */
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fontY, } from './build-font-metrics.ts';
import { toFontCoordinate, } from './font-coordinate.ts';

//region Source scanning

/**
 Source text of one module in this package.
 */
type ModuleSource = {
  /**
   File name of the module.
   */
  readonly name: string;
  /**
   Text of the module with every whitespace character removed, so a spelling
   split across lines still matches and the file's formatting cannot hide it.
   */
  readonly text: string;
};

/**
 Reads every non-test module beside this suite.
 
 @returns one entry per module, in directory order
 */
async function moduleSources(): Promise<ModuleSource[]> {
  /**
   File names in this module's directory.
   */
  const names = await readdir(import.meta.dirname,);
  return Promise.all(names
    .filter(function isSourceModule(name,) {
      return name.endsWith('.ts',)
        && (!name.endsWith('.test.ts',))
        && (name !== 'env.d.ts');
    },)
    .map(async function readModule(name,): Promise<ModuleSource> {
      /**
       Text of the module with every whitespace character removed.
       */
      // oxlint-disable-next-line no-restricted-syntax/no-regex -- the scan must ignore whitespace so a spelling split across lines still matches, and no string API removes every whitespace character; one linear pass over a source file of a few kilobytes.
      const text = (await readFile(
        join(
          import.meta.dirname,
          name,
        ),
        'utf8',
      )).replaceAll(/\s+/gu, '',);
      return {
        name,
        text,
      };
    },),);
}

/**
 Names the modules whose text contains a needle.
 
 @param sources - modules to search
 
 @param needle - literal to look for
 
 @returns file names of the matching modules
 */
function modulesContaining({
  sources,
  needle,
}: {
  readonly sources: readonly ModuleSource[];
  readonly needle: string;
},): string[] {
  return sources
    .filter(function containsNeedle(source,) {
      return source.text.includes(needle,);
    },)
    .map(function nameOf(source,) {
      return source.name;
    },);
}

//endregion Source scanning

await describe({
  name: '',
  children: [
    describe({
      name: toFontCoordinate.name,
      children: [
        it({
          name: 'shifts X from strip coordinates into glyph coordinates',
          fn: async () => {
            /**
             Offset of the cell the point belongs to.
             */
            const cellX = 645;
            /**
             Horizontal shift that places the glyph at its side bearing.
             */
            const xShift = 40;
            /**
             Strip X coordinate of the point.
             */
            const x = 700;
            /**
             Font-space X the transform must produce.
             */
            const expectedX = (x - cellX) + xShift;
            /**
             Glyph-space X of the point.
             */
            const [fontX,] = toFontCoordinate({
              x,
              y: 0,
              cellX,
              xShift,
            },);

            expect(fontX,).toBe(expectedX,);
          },
        },),

        it({
          name: 'flips Y around the font baseline',
          fn: async () => {
            /**
             Strip Y coordinate of the point.
             */
            const y = 100;
            /**
             Glyph-space Y of the point.
             */
            const [, fontPointY,] = toFontCoordinate({
              x: 0,
              y,
              cellX: 0,
              xShift: 0,
            },);

            expect(fontPointY,).toBe(fontY(y,),);
          },
        },),
      ],
    },),

    describe({
      name: 'coordinate ownership',
      children: [
        it({
          name: 'spells the glyph-space transform once',
          fn: async () => {
            /**
             Modules holding the X transform spelling.
             */
            const transformSites = modulesContaining({
              sources: await moduleSources(),
              needle: '-cellX)+xShift',
            },);

            expect(transformSites,).toStrictEqual(['font-coordinate.ts',],);
          },
        },),

        it({
          name: 'declares the coordinate pair type once',
          fn: async () => {
            /**
             Modules declaring a coordinate tuple type.
             */
            const coordinateTypes = modulesContaining({
              sources: await moduleSources(),
              needle: '[number,number',
            },);

            expect(coordinateTypes,).toStrictEqual(['font-coordinate.ts',],);
          },
        },),
      ],
    },),
  ],
},);
