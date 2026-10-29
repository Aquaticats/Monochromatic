import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  DEFAULT_FIXTURE_PATTERNS,
  isCheckedFile,
  isFixtureModule,
  isTestFile,
} from '../dist/final/node/index.mjs';

/** Directory the sample paths in this file sit under. */
const DIR = '/repo/package/module/x/src';

/**
 Tests one path against the shipped default fixture globs.
 
 @param path - normalized absolute path to classify
 
 @returns whether a default glob covers it
 
 @example
 ```ts
 matchesDefault('/repo/src/fixture.json');
 ```
 */
function matchesDefault(path: string,): boolean {
  return isFixtureModule({
    patterns: DEFAULT_FIXTURE_PATTERNS,
    path,
  },);
}

await describe({
  name: 'checked file classification',
  children: [
    describe({
      name: isTestFile.name,
      children: [
        it({
          name: 'accepts a unit test',
          fn: async () => {
            expect(isTestFile({ path: `${DIR}/parse.unit.test.ts`, },),).toBe(true,);
          },
        },),
        it({
          name: 'accepts a benchmark',
          fn: async () => {
            expect(isTestFile({ path: `${DIR}/parse.bench.ts`, },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects ordinary source',
          fn: async () => {
            expect(isTestFile({ path: `${DIR}/parse.ts`, },),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a file merely containing the word test',
          fn: async () => {
            expect(isTestFile({ path: `${DIR}/test-run.ts`, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isFixtureModule.name,
      children: [
        it({
          name: 'accepts a fixture-prefixed data file of any extension',
          fn: async () => {
            expect(matchesDefault(`${DIR}/fixture.json`,),).toBe(true,);
          },
        },),
        it({
          name: 'accepts a name carrying the fixture infix',
          fn: async () => {
            expect(matchesDefault(`${DIR}/toml-fixture-data.ts`,),).toBe(true,);
          },
        },),
        it({
          name: 'accepts the three literal test support names',
          fn: async () => {
            expect([
              matchesDefault(`${DIR}/test-support.ts`,),
              matchesDefault(`${DIR}/test-setup.ts`,),
              matchesDefault(`${DIR}/test-fixtures.ts`,),
            ],).toEqual([
              true,
              true,
              true,
            ],);
          },
        },),
        it({
          name: 'rejects helper and harness suffixes, which name package behavior',
          fn: async () => {
            expect([
              matchesDefault(`${DIR}/tree-helpers.ts`,),
              matchesDefault(`${DIR}/render-harness.ts`,),
            ],).toEqual([
              false,
              false,
            ],);
          },
        },),
        it({
          name: 'rejects package behavior a test- prefix glob would have exempted',
          fn: async () => {
            expect(matchesDefault('/repo/package/mutation-test/cli/src/container/test-run.ts',),)
              .toBe(false,);
          },
        },),
        it({
          name: 'rejects ordinary source',
          fn: async () => {
            expect(matchesDefault(`${DIR}/parse.ts`,),).toBe(false,);
          },
        },),
        it({
          name: 'rejects everything when the configured list is empty',
          fn: async () => {
            expect(isFixtureModule({
              patterns: [],
              path: `${DIR}/tree-helpers.ts`,
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isCheckedFile.name,
      children: [
        it({
          name: 'checks a test file',
          fn: async () => {
            expect(isCheckedFile({
              patterns: DEFAULT_FIXTURE_PATTERNS,
              path: `${DIR}/parse.unit.test.ts`,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'checks a test support module too, closing the re-export laundering path',
          fn: async () => {
            expect(isCheckedFile({
              patterns: DEFAULT_FIXTURE_PATTERNS,
              path: `${DIR}/test-support.ts`,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'leaves a helper-suffixed module alone, since it is package behavior',
          fn: async () => {
            expect(isCheckedFile({
              patterns: DEFAULT_FIXTURE_PATTERNS,
              path: `${DIR}/tree-helpers.ts`,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'leaves ordinary source alone',
          fn: async () => {
            expect(isCheckedFile({
              patterns: DEFAULT_FIXTURE_PATTERNS,
              path: `${DIR}/parse.ts`,
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
