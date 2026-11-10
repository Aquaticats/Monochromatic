/**
 Unit tests for the region-marker guards, driven by fixtures small enough to read.

 Each control names the threshold it checks against rather than the shipped
 `SUBSTANTIAL_MODULE_LINE_COUNT`, so a change to that policy constant leaves these
 cases meaningful. The positive controls are the two a broken guard would fail to
 distinguish: a long unmarked module must be named, and a long marked one must not.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readProductModules,
  unmarkedSubstantialModules,
  type ModuleText,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Threshold the controls check against, unrelated to the shipped constant so a
 policy change cannot silently turn every case into the same one.
 */
const CONTROL_LINE_COUNT_THRESHOLD = 4;

/**
 One line of a fixture module, chosen so a line count is the only thing that
 differs between the fixtures of a control.
 */
const CONTROL_LINE = '// fixture line';

/**
 Marker a marked fixture carries.
 */
const CONTROL_REGION_MARKER = '//region Fixture section';

/**
 Suffix every test module of this package carries.
 */
const TEST_MODULE_SUFFIX = '.unit.test.ts';

/**
 Folder holding this package's product and test modules, which is this test's own
 folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Builds a fixture module of an exact filler line count.

 @param lineCount - Filler lines the module holds, before any marker line.

 @param marker - Marker line to append, or an empty string for an unmarked module.

 @param path - File name the fixture reports as.

 @returns The fixture module.

 @example
 ```ts
 controlModule({ lineCount: 5, marker: '', path: 'unmarked.ts' });
 ```
 */
function controlModule(
  {
    lineCount,
    marker,
    path,
  }: {
    readonly lineCount: number;
    readonly marker: string;
    readonly path: string;
  },
): ModuleText {
  return {
    path,
    text: `${CONTROL_LINE}\n`.repeat(lineCount,) + marker,
  };
}

//endregion Fixtures

await describe({
  name: unmarkedSubstantialModules.name,
  children: [
    it({
      name: 'names a long module that carries no marker',
      fn: async () => {
        expect(
          unmarkedSubstantialModules({
            lineCountThreshold: CONTROL_LINE_COUNT_THRESHOLD,
            modules: [
              controlModule({
                lineCount: CONTROL_LINE_COUNT_THRESHOLD + 1,
                marker: '',
                path: 'unmarked.ts',
              },),
            ],
          },),
        ).toEqual(['unmarked.ts',],);
      },
    },),
    it({
      name: 'leaves a long module that carries a marker alone',
      fn: async () => {
        expect(
          unmarkedSubstantialModules({
            lineCountThreshold: CONTROL_LINE_COUNT_THRESHOLD,
            modules: [
              controlModule({
                lineCount: CONTROL_LINE_COUNT_THRESHOLD + 1,
                marker: CONTROL_REGION_MARKER,
                path: 'marked.ts',
              },),
            ],
          },),
        ).toEqual([],);
      },
    },),
    it({
      name: 'leaves a module at the threshold alone',
      fn: async () => {
        expect(
          unmarkedSubstantialModules({
            lineCountThreshold: CONTROL_LINE_COUNT_THRESHOLD,
            modules: [
              controlModule({
                lineCount: CONTROL_LINE_COUNT_THRESHOLD,
                marker: '',
                path: 'at-threshold.ts',
              },),
            ],
          },),
        ).toEqual([],);
      },
    },),
    it({
      name: 'keeps the order of the modules it reports',
      fn: async () => {
        expect(
          unmarkedSubstantialModules({
            lineCountThreshold: CONTROL_LINE_COUNT_THRESHOLD,
            modules: [
              controlModule({
                lineCount: CONTROL_LINE_COUNT_THRESHOLD + 2,
                marker: '',
                path: 'first.ts',
              },),
              controlModule({
                lineCount: CONTROL_LINE_COUNT_THRESHOLD + 1,
                marker: '',
                path: 'second.ts',
              },),
            ],
          },),
        ).toEqual(['first.ts', 'second.ts',],);
      },
    },),
  ],
},);

await describe({
  name: readProductModules.name,
  children: [
    it({
      name: 'reads this package\'s product modules and not its tests',
      fn: async () => {
        /**
         Modules of this package's own source folder, read the way a guard reads
         them.
         */
        const modules = await readProductModules({ directory: SOURCE_DIRECTORY, },);
        /**
         File names those modules report as.
         */
        const paths = modules.map(function modulePath(module,): string {
          return module.path;
        },);

        expect(paths,).toContain('index.ts',);
        expect(
          paths.filter(function isTest(path,): boolean {
            return path.endsWith(TEST_MODULE_SUFFIX,);
          },),
        ).toEqual([],);
      },
    },),
  ],
},);
