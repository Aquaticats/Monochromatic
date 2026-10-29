/**
 Tests for the census's placement of a painted tally on source lines
 (ledger T8): a cold block in a mapped bundle placed on its source line, a
 function no process called placed on the line it starts, the sources the
 loaded bundles carry, and the sources only an unloaded bundle carries;
 a cold stretch that is nothing but an invariant throw placed apart from the
 cold stretches; a bundle with no map read only where code must be placed
 in it, and refused by name there. Each case builds a disposable package
 with a build directory, its maps and a coverage directory. Names are
 cat-themed invention.

 @module
 */

import {
  mkdir,
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bundleMapsOf,
  invariantThrowRowsOf,
  kindTotalsOf,
  placeTally,
  readMappedBundles,
  sourceRowsOf,
  tallyCoverage,
} from '../../dist/final/node/index.mjs';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 A bundle's text: one line, then a second line of 120 characters, which a
 map of `;AAAA` places wholly on its source's first line.
 */
const BUNDLE_TEXT = `x\n${'y'.repeat(120,)}`;

/**
 One function a coverage file lists: its name and its ranges as
 `[start, end, count]`, whole extent first.
 */
type FixtureFunction = {
  readonly name: string;
  readonly ranges: readonly (readonly [number, number, number])[];
};

/**
 One bundle a fixture build holds.
 */
type FixtureBundle = {
  /**
   Its file name.
   */
  readonly bundle: string;

  /**
   The source its map names, absent for a bundle written without a map.
   */
  readonly source?: string;

  /**
   Its text, absent for a bundle holding `BUNDLE_TEXT`.
   */
  readonly text?: string;
};

/**
 A disposable package with a build directory holding the bundles given,
 each with a map to its source when it names one, the sources' files, and
 an empty coverage directory.

 @param bundles - bundles the build holds

 @returns Package, its build directory and coverage directory, and the
 build's URL prefix
 */
async function fixturePackage(
  { bundles, }: { readonly bundles: readonly FixtureBundle[]; },
) {
  /**
   Built package: setup's own fields, plus the directory's path and
   disposer, wired by `scratchDirWith`.
   */
  const built = await scratchDirWith({
    prefix: 'translation-repair-place-test-',
    setup: async function seeded({ path, },): Promise<{
      readonly distDirectory: string;
      readonly coverageDirectory: string;
      readonly prefix: string;
    }> {
      /**
       Its build directory.
       */
      const distDirectory = join(
        path,
        'dist',
        'final',
        'node',
      );
      /**
       Where the coverage files go.
       */
      const coverageDirectory = join(
        path,
        'coverage',
      );
      await mkdir(
        distDirectory,
        { recursive: true, },
      );
      await mkdir(
        coverageDirectory,
        { recursive: true, },
      );
      await mkdir(
        join(
          path,
          'src',
        ),
        { recursive: true, },
      );
      await Promise.all(bundles.map(async function writtenBundle({ bundle, source, text, },): Promise<void> {
        await writeFile(
          join(
            distDirectory,
            bundle,
          ),
          text ?? BUNDLE_TEXT,
        );
        if (source === undefined)
          return;
        await Promise.all([
          writeFile(
            join(
              distDirectory,
              `${bundle}.map`,
            ),
            JSON.stringify({
              version: 3,
              sources: [`../../../${source}`,],
              names: [],
              mappings: ';AAAA',
            },),
          ),
          writeFile(
            join(
              path,
              source,
            ),
            'doze\nyawn\nstretch',
          ),
        ],);
      },),);
      return {
        distDirectory,
        coverageDirectory,
        prefix: `${pathToFileURL(distDirectory,).href}/`,
      };
    },
  },);
  return {
    /**
     The package directory alone, the shape every call site already binds
     with `await using`.
     */
    directory: {
      path: built.path,
      [Symbol.asyncDispose]: built[Symbol.asyncDispose],
    },
    distDirectory: built.distDirectory,
    coverageDirectory: built.coverageDirectory,
    prefix: built.prefix,
  };
}

/**
 Writes one process's coverage file.

 @param coverageDirectory - where it goes

 @param prefix - build directory URL with a trailing slash

 @param scripts - each bundle the process loaded, with its functions
 */
async function writeCoverage(
  {
    coverageDirectory,
    prefix,
    scripts,
  }: {
    readonly coverageDirectory: string;
    readonly prefix: string;
    readonly scripts: readonly (readonly [string, readonly FixtureFunction[]])[];
  },
): Promise<void> {
  await writeFile(
    join(
      coverageDirectory,
      'coverage-1.json',
    ),
    JSON.stringify({
      result: scripts.map(([bundle, functions,],) => ({
        url: `${prefix}${bundle}`,
        functions: functions.map(({ name, ranges, },) => ({
          functionName: name,
          ranges: ranges.map(([startOffset, endOffset, count,],) => ({
            startOffset,
            endOffset,
            count,
          })),
        })),
      })),
    },),
  );
}

/**
 A guard of the nap bundle: one throw of an `Error` whose message begins
 `unreachable:`.
 */
const NAP_GUARD = 'throw new Error("unreachable: a lap is always given");';

/**
 A block of the nap bundle holding a call beside its invariant throw.
 */
const NAP_MIXED = '{ hiss(); throw new Error("unreachable: no lap is that cold"); }';

/**
 A function the nap bundle's third throw builds its message with.
 */
const NAP_CALLBACK = '(toy) => toy.name';

/**
 A throw of the nap bundle whose message holds `NAP_CALLBACK`.
 */
const NAP_HOLDING = `throw new Error(\`unreachable: \${toys.map(${NAP_CALLBACK}).join()} holds no toy\`);`;

/**
 The nap bundle's text: one line, then a function on a second line, which a
 map of `;AAAA` places wholly on its source's first line.
 */
const NAP_TEXT = `x\nfunction nap(lap, toys) { if (lap === void 0) ${NAP_GUARD} if (lap < 0) ${NAP_MIXED} `
  + `if (toys.length === 0) ${NAP_HOLDING} return lap; }`;

/**
 The purr bundle's only guard: one throw of a class whose name ends in
 `InvariantError`.
 */
const PURR_GUARD = 'throw new PurrInvariantError({ cat });';

/**
 The purr bundle's text, laid out as `NAP_TEXT` is.
 */
const PURR_TEXT = `x\nfunction purr(cat) { if (cat === void 0) ${PURR_GUARD} return cat; }`;

/**
 Where a piece of a bundle's text sits, as a coverage range that never ran.

 @param text - bundle text

 @param piece - piece of it, written once there

 @returns Its range as `[start, end, 0]`
 */
function coldRange(
  {
    text,
    piece,
  }: {
    readonly text: string;
    readonly piece: string;
  },
): readonly [number, number, number] {
  /**
   Offset of the piece's first character.
   */
  const start = text.indexOf(piece,);
  return [
    start,
    start + piece.length,
    0,
  ];
}

/**
 A loaded bundle's script whose top level ran and holds nothing else.
 */
const LINKAGE_ONLY: readonly FixtureFunction[] = [{
  name: '',
  ranges: [[0, BUNDLE_TEXT.length, 1,],],
},];

await describe({
  name: placeTally.name,
  children: [
    it({
      name: 'PLACES a cold block and an uncalled function on their source lines, names the loaded bundles\' '
        + 'sources, and counts the sources only an unloaded bundle carries, reading nothing of a loaded '
        + 'bundle with no map and no cold code',
      fn: async () => {
        const {
          directory,
          distDirectory,
          coverageDirectory,
          prefix,
        } = await fixturePackage({
          bundles: [
            {
              bundle: 'nap.mjs',
              source: 'src/nap.ts',
            },
            {
              bundle: 'purr.mjs',
              source: 'src/purr.ts',
            },
            { bundle: 'index.mjs', },
          ],
        },);
        await using _removed = directory;
        await writeCoverage({
          coverageDirectory,
          prefix,
          scripts: [
            ['nap.mjs', [
              {
                name: '',
                ranges: [[0, BUNDLE_TEXT.length, 1,],],
              },
              {
                name: 'nap',
                ranges: [[10, 90, 2,], [40, 60, 0,],],
              },
              {
                name: 'knead',
                ranges: [[95, 110, 0,],],
              },
            ],],
            ['index.mjs', LINKAGE_ONLY,],
          ],
        },);
        const placed = await placeTally({
          packageDirectory: directory.path,
          distDirectory,
          bundleMaps: bundleMapsOf({
            built: await readdir(distDirectory,),
            distDirectory,
          },),
          tally: await tallyCoverage({
            coverageDirectory,
            bundleUrlPrefix: prefix,
          },),
          entryFiles: new Set(),
        },);
        expect(placed.stretches.map(function where({ bundle, start, end, source, startLine, endLine, },) {
          return {
            bundle,
            start,
            end,
            source,
            startLine,
            endLine,
          };
        },),).toEqual([
          {
            bundle: 'nap.mjs',
            start: 40,
            end: 60,
            source: 'src/nap.ts',
            startLine: 1,
            endLine: 1,
          },
          {
            bundle: 'nap.mjs',
            start: 95,
            end: 110,
            source: 'src/nap.ts',
            startLine: 1,
            endLine: 1,
          },
        ],);
        expect(placed.uncalled.map(function named({ name, at, },) {
          return {
            name,
            at,
          };
        },),).toEqual([{
          name: 'knead',
          at: {
            kind: 'mapped',
            source: 'src/nap.ts',
            line: 1,
          },
        },],);
        expect([...placed.loadedSources,],).toEqual(['src/nap.ts',],);
        expect(placed.unloadedBundles,).toEqual(['purr.mjs',],);
        expect(placed.unloadedSources,).toEqual([{
          source: 'src/purr.ts',
          kind: 'library source',
          lines: 3,
        },],);
      },
    },),
    it({
      name: 'PLACES A COLD STRETCH THAT IS NOTHING BUT AN INVARIANT THROW APART FROM THE COLD STRETCHES, naming what '
        + 'it throws, so the rows and totals built from the cold stretches leave it out and a source holding no '
        + 'other cold code has no row; KEEPS COLD a block holding a call beside its throw, and a throw holding a '
        + 'function no process called',
      fn: async () => {
        const {
          directory,
          distDirectory,
          coverageDirectory,
          prefix,
        } = await fixturePackage({
          bundles: [
            {
              bundle: 'nap.mjs',
              source: 'src/nap.ts',
              text: NAP_TEXT,
            },
            {
              bundle: 'purr.mjs',
              source: 'src/purr.ts',
              text: PURR_TEXT,
            },
          ],
        },);
        await using _removed = directory;
        await writeCoverage({
          coverageDirectory,
          prefix,
          scripts: [
            ['nap.mjs', [
              {
                name: '',
                ranges: [[0, NAP_TEXT.length, 1,],],
              },
              {
                name: 'nap',
                ranges: [
                  [2, NAP_TEXT.length, 2,],
                  coldRange({
                    text: NAP_TEXT,
                    piece: NAP_GUARD,
                  },),
                  coldRange({
                    text: NAP_TEXT,
                    piece: NAP_MIXED,
                  },),
                  coldRange({
                    text: NAP_TEXT,
                    piece: NAP_HOLDING,
                  },),
                ],
              },
              {
                name: '',
                ranges: [coldRange({
                  text: NAP_TEXT,
                  piece: NAP_CALLBACK,
                },),],
              },
            ],],
            ['purr.mjs', [
              {
                name: '',
                ranges: [[0, PURR_TEXT.length, 1,],],
              },
              {
                name: 'purr',
                ranges: [
                  [2, PURR_TEXT.length, 1,],
                  coldRange({
                    text: PURR_TEXT,
                    piece: PURR_GUARD,
                  },),
                ],
              },
            ],],
          ],
        },);
        const placed = await placeTally({
          packageDirectory: directory.path,
          distDirectory,
          bundleMaps: bundleMapsOf({
            built: await readdir(distDirectory,),
            distDirectory,
          },),
          tally: await tallyCoverage({
            coverageDirectory,
            bundleUrlPrefix: prefix,
          },),
          entryFiles: new Set(),
        },);
        /**
         A stretch of the nap bundle as the census records it.

         @param piece - piece of the bundle's text the stretch covers

         @returns Its record
         */
        function napStretch({ piece, }: { readonly piece: string; },) {
          const [start, end,] = coldRange({
            text: NAP_TEXT,
            piece,
          },);
          return {
            bundle: 'nap.mjs',
            start,
            end,
            name: '',
            source: 'src/nap.ts',
            startLine: 1,
            endLine: 1,
          };
        }
        /**
         Where the purr bundle's guard sits.
         */
        const [purrStart, purrEnd,] = coldRange({
          text: PURR_TEXT,
          piece: PURR_GUARD,
        },);
        expect(placed.invariantThrows,).toEqual([
          {
            ...napStretch({ piece: NAP_GUARD, },),
            thrown: ['Error',],
          },
          {
            bundle: 'purr.mjs',
            start: purrStart,
            end: purrEnd,
            name: '',
            source: 'src/purr.ts',
            startLine: 1,
            endLine: 1,
            thrown: ['PurrInvariantError',],
          },
        ],);
        expect(placed.stretches,).toEqual([
          napStretch({ piece: NAP_MIXED, },),
          napStretch({ piece: NAP_HOLDING, },),
        ],);
        /**
         The rows the census builds from the cold stretches.
         */
        const rows = sourceRowsOf({
          stretches: placed.stretches,
          uncalled: placed.uncalled,
          entryFiles: new Set(),
        },);
        expect(rows,).toEqual([{
          source: 'src/nap.ts',
          kind: 'library source',
          stretches: 2,
          lines: 1,
          uncalled: 1,
        },],);
        expect(kindTotalsOf({ rows, },),).toEqual([{
          kind: 'library source',
          files: 1,
          stretches: 2,
          lines: 1,
          uncalled: 1,
        },],);
        expect(invariantThrowRowsOf({
          invariantThrows: placed.invariantThrows,
          entryFiles: new Set(),
        },),).toEqual([
          {
            source: 'src/nap.ts',
            kind: 'library source',
            startLine: 1,
            endLine: 1,
            thrown: ['Error',],
          },
          {
            source: 'src/purr.ts',
            kind: 'library source',
            startLine: 1,
            endLine: 1,
            thrown: ['PurrInvariantError',],
          },
        ],);
      },
    },),
    it({
      name: 'REFUSES cold code in a loaded bundle with no map, naming the bundle',
      fn: async () => {
        const {
          directory,
          distDirectory,
          coverageDirectory,
          prefix,
        } = await fixturePackage({
          bundles: [
            {
              bundle: 'nap.mjs',
              source: 'src/nap.ts',
            },
            { bundle: 'index.mjs', },
          ],
        },);
        await using _removed = directory;
        await writeCoverage({
          coverageDirectory,
          prefix,
          scripts: [
            ['nap.mjs', LINKAGE_ONLY,],
            ['index.mjs', [{
              name: '',
              ranges: [[0, BUNDLE_TEXT.length, 1,], [20, 30, 0,],],
            },],],
          ],
        },);
        await expect(placeTally({
          packageDirectory: directory.path,
          distDirectory,
          bundleMaps: bundleMapsOf({
            built: await readdir(distDirectory,),
            distDirectory,
          },),
          tally: await tallyCoverage({
            coverageDirectory,
            bundleUrlPrefix: prefix,
          },),
          entryFiles: new Set(),
        },),).rejects.toThrow('coverage finds code no test ran in index.mjs',);
      },
    },),
    it({
      name: 'REFUSES a bundle no test loaded that has no map, naming the bundle',
      fn: async () => {
        const {
          directory,
          distDirectory,
          coverageDirectory,
          prefix,
        } = await fixturePackage({
          bundles: [
            {
              bundle: 'nap.mjs',
              source: 'src/nap.ts',
            },
            { bundle: 'litter.mjs', },
          ],
        },);
        await using _removed = directory;
        await writeCoverage({
          coverageDirectory,
          prefix,
          scripts: [['nap.mjs', LINKAGE_ONLY,],],
        },);
        await expect(placeTally({
          packageDirectory: directory.path,
          distDirectory,
          bundleMaps: bundleMapsOf({
            built: await readdir(distDirectory,),
            distDirectory,
          },),
          tally: await tallyCoverage({
            coverageDirectory,
            bundleUrlPrefix: prefix,
          },),
          entryFiles: new Set(),
        },),).rejects.toThrow('no test loaded litter.mjs',);
      },
    },),
    it({
      name: 'REFUSES coverage naming a bundle the build does not hold',
      fn: async () => {
        const {
          directory,
          distDirectory,
          coverageDirectory,
          prefix,
        } = await fixturePackage({
          bundles: [{
            bundle: 'nap.mjs',
            source: 'src/nap.ts',
          },],
        },);
        await using _removed = directory;
        await writeCoverage({
          coverageDirectory,
          prefix,
          scripts: [
            ['nap.mjs', LINKAGE_ONLY,],
            ['ghost.mjs', [{
              name: '',
              ranges: [[0, 50, 1,], [20, 30, 0,],],
            },],],
          ],
        },);
        await expect(placeTally({
          packageDirectory: directory.path,
          distDirectory,
          bundleMaps: bundleMapsOf({
            built: await readdir(distDirectory,),
            distDirectory,
          },),
          tally: await tallyCoverage({
            coverageDirectory,
            bundleUrlPrefix: prefix,
          },),
          entryFiles: new Set(),
        },),).rejects.toThrow('coverage names ghost.mjs',);
      },
    },),
    it({
      name: 'READS EVERY MAPPED BUNDLE by name in the order given, and refuses with the first bundle\'s read when two '
        + 'cannot be read and the second bundle\'s read is refused first',
      fn: async () => {
        // Compared as a list of entries: the matcher compares two Maps with
        // their entries sorted, which would pass a reading in any order. The
        // bundles are listed out of name order so a reading that sorted them
        // fails too.
        expect([
          ...await readMappedBundles({
            mapped: ['purr.mjs', 'nap.mjs',],
            readOne: async function lengthOf({ bundle, },): Promise<number> {
              return bundle.length;
            },
          },),
        ],).toEqual([
          ['purr.mjs', 'purr.mjs'.length,],
          ['nap.mjs', 'nap.mjs'.length,],
        ],);
        /**
         The two refusals the bundle reads end in.
         */
        const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
        /**
         What the reading refused with.
         */
        const refusal = await rejectionOf({
          promise: readMappedBundles({
            mapped: ['nap.mjs', 'purr.mjs',],
            readOne: async function refusesSecondFirst({ bundle, },): Promise<number> {
              return await ((bundle === 'nap.mjs')
                ? refuseAfterThat(new Error('the nap bundle cannot be read',),)
                : refuseAtOnce(new Error('the purr bundle cannot be read',),));
            },
          },),
        },);
        expect(String(refusal,),).toBe('Error: the nap bundle cannot be read',);
      },
    },),
  ],
},);
