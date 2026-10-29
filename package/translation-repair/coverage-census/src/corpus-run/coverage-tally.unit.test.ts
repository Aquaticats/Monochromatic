/**
 Tests the coverage census's reading of V8 coverage (ledger T8): which scripts
 a coverage file yields and which it refuses, and the tally's counts over
 several processes. The case the tally exists for comes first: V8 drops a
 nested range whose count equals its parent's, so a block one process lists
 as cold ran in a process that does not list it. Bundles and functions are
 cat-themed invention.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type BundleScript,
  bundleScriptsOf,
  CoverageFileError,
  CoverageTallyError,
  createCoverageTally,
} from '../../dist/final/node/index.mjs';

/**
 Build directory the fixture coverage names.
 */
const PREFIX = 'file:///cattery/dist/final/node/';

/**
 One bundle's script as one process reports it.

 @param bundle - bundle name

 @param functions - each function's name and ranges as `[start, end, count]`, whole extent first

 @returns Script
 */
function scriptOf(
  {
    bundle,
    functions,
  }: {
    readonly bundle: string;
    readonly functions: readonly (readonly [string, readonly (readonly [number, number, number])[]])[];
  },
): BundleScript {
  return {
    bundle,
    functions: functions.map(([functionName, ranges,],) => ({
      functionName,
      ranges: ranges.map(([startOffset, endOffset, count,],) => ({
        startOffset,
        endOffset,
        count,
      })),
    })),
  };
}

/**
 Tallies the given processes, each a list of scripts, over both readings.

 @param processes - each process's scripts

 @returns Painted tally
 */
function tallied({ processes, }: { readonly processes: readonly (readonly BundleScript[])[]; },) {
  const tally = createCoverageTally();
  for (const scripts of processes)
    tally.noteBoundaries({ scripts, },);
  for (const scripts of processes)
    tally.paint({ scripts, },);
  return tally;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: bundleScriptsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'YIELDS ONLY SCRIPTS UNDER THE BUILD DIRECTORY, named from it, and never reads the others',
          fn: async () => {
            const text = JSON.stringify({
              result: [
                {
                  url: `${PREFIX}nap.mjs`,
                  functions: [{
                    functionName: 'nap',
                    ranges: [{
                      startOffset: 0,
                      endOffset: 9,
                      count: 1,
                    },],
                  },],
                },
                {
                  url: 'node:internal/purr',
                  functions: 'not read',
                },
              ],
            },);
            expect(bundleScriptsOf({
              path: 'coverage-1.json',
              text,
              bundleUrlPrefix: PREFIX,
            },),).toEqual([{
              bundle: 'nap.mjs',
              functions: [{
                functionName: 'nap',
                ranges: [{
                  startOffset: 0,
                  endOffset: 9,
                  count: 1,
                },],
              },],
            },],);
          },
        },),
        it({
          name: 'REFUSES a file that is not an object, has no result list, or holds a script that is not an object or has no url',
          fn: async () => {
            for (const text of [
              'null',
              JSON.stringify({ result: 'none', },),
              JSON.stringify({ result: [3,], },),
              JSON.stringify({ result: [{ functions: [], },], },),
            ]) {
              expect(() => bundleScriptsOf({
                path: 'coverage-1.json',
                text,
                bundleUrlPrefix: PREFIX,
              },),).toThrow(CoverageFileError,);
            }
          },
        },),
        it({
          name: 'REFUSES A COVERAGE FILE CUT SHORT as its own refusal saying it is not JSON, naming the file and '
            + 'quoting none of its text, where the parser\'s refusal reached the census as a fault in the command',
          fn: async () => {
            const refusal = caught(function readsCutShortCoverage() {
              bundleScriptsOf({
                path: 'coverage-1.json',
                text: JSON.stringify({ result: [{
                  url: `${PREFIX}nap.mjs`,
                  functions: [],
                },], },)
                  .slice(
                  0,
                  35,
                  ),
                bundleUrlPrefix: PREFIX,
              },);
            },);
            expect(refusal,).toBeInstanceOf(CoverageFileError,);
            expect(String(refusal,),).toBe(
              'CoverageFileError: coverage file coverage-1.json does not read as V8 writes one: it is not JSON',
            );
          },
        },),
        it({
          name: 'REFUSES a bundle script whose functions are not a list, or hold one that is not an object, without a name, without an extent, or with a range that is not three numbers',
          fn: async () => {
            for (const functions of [
              'none',
              [3,],
              [{ ranges: [{
                startOffset: 0,
                endOffset: 9,
                count: 1,
              },], },],
              [{
                functionName: 'nap',
                ranges: [],
              },],
              [{
                functionName: 'nap',
                ranges: [{
                  startOffset: '0',
                  endOffset: 9,
                  count: 1,
                },],
              },],
            ]) {
              expect(() => bundleScriptsOf({
                path: 'coverage-1.json',
                text: JSON.stringify({ result: [{
                  url: `${PREFIX}nap.mjs`,
                  functions,
                },], },),
                bundleUrlPrefix: PREFIX,
              },),).toThrow(CoverageFileError,);
            }
          },
        },),
      ],
    },),

    describe({
      name: createCoverageTally.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS A BLOCK A PROCESS DOES NOT LIST AT ITS PARENT\'S COUNT: cold where one process lists it, ran where V8 merged it into its parent',
          fn: async () => {
            const listedCold = scriptOf({
              bundle: 'nap.mjs',
              functions: [
                ['', [[0, 100, 1,],],],
                ['nap', [[10, 90, 2,], [40, 60, 0,],],],
              ],
            },);
            const merged = scriptOf({
              bundle: 'nap.mjs',
              functions: [
                ['', [[0, 100, 1,],],],
                ['nap', [[10, 90, 3,],],],
              ],
            },);
            expect(tallied({ processes: [[listedCold,], [merged,],], },).coldStretches(),).toEqual([],);
            expect(tallied({ processes: [[listedCold,],], },).coldStretches(),).toEqual([{
              bundle: 'nap.mjs',
              start: 40,
              end: 60,
              shape: { kind: 'block', },
            },],);
          },
        },),
        it({
          name: 'NAMES A WHOLE FUNCTION NO PROCESS CALLED, a function opening its chunk included, apart from the chunk\'s top-level script at the same offset',
          fn: async () => {
            const tally = tallied({
              processes: [[scriptOf({
                bundle: 'purr.mjs',
                functions: [
                  ['', [[0, 100, 1,],],],
                  ['purr', [[0, 50, 0,],],],
                ],
              },),],],
            },);
            expect(tally.coldStretches(),).toEqual([{
              bundle: 'purr.mjs',
              start: 0,
              end: 50,
              shape: {
                kind: 'function',
                name: 'purr',
              },
            },],);
            expect(tally.uncalledFunctions(),).toEqual([{
              bundle: 'purr.mjs',
              start: 0,
              end: 50,
              name: 'purr',
              nested: false,
            },],);
          },
        },),
        it({
          name: 'MERGES A COLD FUNCTION WITH THE COLD CODE AROUND IT into one stretch, and marks an uncalled function nested in another but not a sibling or one in another bundle',
          fn: async () => {
            const tally = tallied({
              processes: [[
                scriptOf({
                  bundle: 'nap.mjs',
                  functions: [
                    ['', [[0, 100, 1,],],],
                    ['stretch', [[10, 90, 0,],],],
                    ['yawn', [[20, 30, 0,],],],
                    ['blink', [[95, 99, 0,],],],
                  ],
                },),
                scriptOf({
                  bundle: 'purr.mjs',
                  functions: [
                    ['', [[0, 100, 1,],],],
                    ['knead', [[20, 30, 0,],],],
                  ],
                },),
              ],],
            },);
            expect(tally.coldStretches(),).toEqual([
              {
                bundle: 'nap.mjs',
                start: 10,
                end: 90,
                shape: {
                  kind: 'function',
                  name: 'stretch',
                },
              },
              {
                bundle: 'nap.mjs',
                start: 95,
                end: 99,
                shape: {
                  kind: 'function',
                  name: 'blink',
                },
              },
              {
                bundle: 'purr.mjs',
                start: 20,
                end: 30,
                shape: {
                  kind: 'function',
                  name: 'knead',
                },
              },
            ],);
            expect(tally.uncalledFunctions().map((fn,) => [fn.bundle, fn.name, fn.nested,]),).toEqual([
              ['nap.mjs', 'stretch', false,],
              ['nap.mjs', 'yawn', true,],
              ['nap.mjs', 'blink', false,],
              ['purr.mjs', 'knead', false,],
            ],);
          },
        },),
        it({
          name: 'ENDS A COLD RUN AT THE BUNDLE\'S LAST BOUNDARY when nothing after it ran, and orders uncalled functions sharing a start by the longer first',
          fn: async () => {
            expect(tallied({
              processes: [[scriptOf({
                bundle: 'nap.mjs',
                functions: [['doze', [[0, 10, 0,],],],],
              },),],],
            },).coldStretches(),).toEqual([{
              bundle: 'nap.mjs',
              start: 0,
              end: 10,
              shape: {
                kind: 'function',
                name: 'doze',
              },
            },],);
            expect(tallied({
              processes: [[scriptOf({
                bundle: 'nap.mjs',
                functions: [
                  ['', [[0, 100, 1,],],],
                  ['snooze', [[10, 30, 0,],],],
                  ['doze', [[10, 60, 0,],],],
                ],
              },),],],
            },).uncalledFunctions().map((fn,) => [fn.name, fn.nested,]),).toEqual([
              ['doze', false,],
              ['snooze', true,],
            ],);
          },
        },),
        it({
          name: 'LEAVES CODE NO PROCESS HAD A RANGE OVER, which is no code V8 counted',
          fn: async () => {
            expect(tallied({
              processes: [[scriptOf({
                bundle: 'nap.mjs',
                functions: [
                  ['doze', [[0, 10, 1,],],],
                  ['snooze', [[20, 30, 1,],],],
                ],
              },),],],
            },).coldStretches(),).toEqual([],);
          },
        },),
        it({
          name: 'LISTS THE BUNDLES SOME PROCESS LOADED, sorted',
          fn: async () => {
            expect(tallied({
              processes: [
                [scriptOf({
                  bundle: 'purr.mjs',
                  functions: [['', [[0, 9, 1,],],],],
                },),],
                [scriptOf({
                  bundle: 'nap.mjs',
                  functions: [['', [[0, 9, 1,],],],],
                },),],
              ],
            },).loadedBundles(),).toEqual(['nap.mjs', 'purr.mjs',],);
          },
        },),
        it({
          name: 'REFUSES a boundary noted after painting began, and a bundle, offset or extentless function the first reading never saw',
          fn: async () => {
            const nap = scriptOf({
              bundle: 'nap.mjs',
              functions: [['', [[0, 100, 1,],],],],
            },);
            const painted = createCoverageTally();
            painted.noteBoundaries({ scripts: [nap,], },);
            painted.paint({ scripts: [nap,], },);
            expect(() => painted.noteBoundaries({ scripts: [nap,], },),).toThrow(CoverageTallyError,);
            expect(() => painted.paint({
              scripts: [scriptOf({
                bundle: 'purr.mjs',
                functions: [['', [[0, 100, 1,],],],],
              },),],
            },),).toThrow(CoverageTallyError,);
            expect(() => painted.paint({
              scripts: [scriptOf({
                bundle: 'nap.mjs',
                functions: [['', [[0, 99, 1,],],],],
              },),],
            },),).toThrow(CoverageTallyError,);
            expect(() => painted.paint({
              scripts: [{
                bundle: 'nap.mjs',
                functions: [{
                  functionName: 'nap',
                  ranges: [],
                },],
              },],
            },),).toThrow(CoverageTallyError,);
          },
        },),
      ],
    },),
  ],
},);
