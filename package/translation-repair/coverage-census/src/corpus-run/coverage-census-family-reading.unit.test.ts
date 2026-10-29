/**
 Tests for how the census names and sorts sources when it measures a family
 of packages: a source is named from the family directory, as
 `<package>/src/<file>`, and is a runner entry or library source by the file's
 name inside its package, the way the packages' build entry lists name it.
 Paths and names are cat-themed invention.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  censusReportLines,
  type CensusSummary,
  ownSourceOf,
  readBundle,
  sourceKindOf,
  sourceLineAt,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Runner entry sources as the packages' build entry lists name them.
 */
const ENTRY_FILES: ReadonlySet<string> = new Set(['src/corpus-run/purr.ts',],);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: ownSourceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A SOURCE BY ITS PATH INSIDE ITS PACKAGE, whether it is named from the package or from the family '
            + 'directory, and no other source',
          fn: async () => {
            expect(ownSourceOf({ source: 'src/nap.ts', },),).toEqual({ kind: 'own', path: 'src/nap.ts', },);
            expect(ownSourceOf({ source: 'alpha/src/nap.ts', },),).toEqual({ kind: 'own', path: 'src/nap.ts', },);
            expect(ownSourceOf({ source: 'alpha/src/corpus-run/purr.ts', },),).toEqual({ kind: 'own', path: 'src/corpus-run/purr.ts', },);
            expect(ownSourceOf({ source: '../module/whisker/src/index.ts', },),).toEqual({ kind: 'foreign', },);
            expect(ownSourceOf({ source: 'node_modules/whisker/src/index.ts', },),).toEqual({ kind: 'foreign', },);
            expect(ownSourceOf({ source: 'alpha/dist/final/node/nap.mjs', },),).toEqual({ kind: 'foreign', },);
            expect(ownSourceOf({ source: '(unmapped) nap.mjs', },),).toEqual({ kind: 'foreign', },);
          },
        },),
      ],
    },),
    describe({
      name: sourceKindOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A SOURCE NAMED FROM THE FAMILY DIRECTORY AS A RUNNER ENTRY OR LIBRARY SOURCE by its name inside its '
            + 'package, and one outside every package as another package',
          fn: async () => {
            expect(sourceKindOf({ source: 'alpha/src/corpus-run/purr.ts', entryFiles: ENTRY_FILES, },),).toBe('entry file',);
            expect(sourceKindOf({ source: 'alpha/src/nap.ts', entryFiles: ENTRY_FILES, },),).toBe('library source',);
            expect(sourceKindOf({ source: 'src/corpus-run/purr.ts', entryFiles: ENTRY_FILES, },),).toBe('entry file',);
            expect(sourceKindOf({ source: '../module/whisker/src/index.ts', entryFiles: ENTRY_FILES, },),).toBe('other package',);
            expect(sourceKindOf({ source: 'node_modules/whisker/src/index.ts', entryFiles: ENTRY_FILES, },),).toBe('other package',);
          },
        },),
      ],
    },),
    describe({
      name: readBundle.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A BUNDLE NAMED BY ITS PATH FROM THE FAMILY DIRECTORY through the map beside it, naming its sources '
            + 'from the family directory',
          fn: async () => {
            await using family = await scratchDir({ prefix: 'coverage-census-family-reading-', },);
            const distDirectory = join(
              family.path,
              'alpha',
              'dist',
              'final',
              'node',
            );
            await mkdir(
              distDirectory,
              { recursive: true, },
            );
            await writeFile(
              join(
                distDirectory,
                'nap.mjs',
              ),
              'x\ny',
            );
            await writeFile(
              join(
                distDirectory,
                'nap.mjs.map',
              ),
              JSON.stringify({
                version: 3,
                sources: ['../../../src/nap.ts',],
                names: [],
                mappings: ';AAAA',
              },),
            );

            const { lines, sources, text, } = await readBundle({
              distDirectory: family.path,
              packageDirectory: family.path,
              bundle: 'alpha/dist/final/node/nap.mjs',
            },);

            expect(text,).toBe('x\ny',);
            expect(sources,).toEqual(['alpha/src/nap.ts',],);
            expect(sourceLineAt({
              lines,
              offset: 2,
            },),).toEqual({
              kind: 'mapped',
              source: 'alpha/src/nap.ts',
              line: 1,
            },);
          },
        },),
      ],
    },),
    describe({
      name: censusReportLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS THE OUTERMOST UNCALLED FUNCTIONS OF A SOURCE NAMED FROM THE FAMILY DIRECTORY as package source, '
            + 'and still leaves out another package\'s',
          fn: async () => {
            /**
             A census holding one uncalled function in a family source and one outside the family.
             */
            const census: CensusSummary = {
              head: 'abc123def',
              clean: true,
              testFiles: [],
              passes: 2,
              totals: [],
              rows: [],
              invariantThrows: [],
              uncalled: [
                {
                  bundle: 'alpha/dist/final/node/nap.mjs',
                  start: 0,
                  end: 1,
                  name: 'doze',
                  nested: false,
                  at: {
                    kind: 'mapped',
                    source: 'alpha/src/nap.ts',
                    line: 10,
                  },
                },
                {
                  bundle: 'alpha/dist/final/node/nap.mjs',
                  start: 0,
                  end: 1,
                  name: 'twitch',
                  nested: false,
                  at: {
                    kind: 'mapped',
                    source: '../module/whisker/src/index.ts',
                    line: 3,
                  },
                },
              ],
              unloadedBundles: [],
              unloadedSources: [],
              censusPath: '/tmp/census.json',
            };

            expect(censusReportLines({ census, },).filter(function isFunctionLine(line,): boolean {
              return line.startsWith('  ',);
            },),).toEqual(['  alpha/src/nap.ts:10 doze',],);
          },
        },),
      ],
    },),
  ],
},);
