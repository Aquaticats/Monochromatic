/**
 Tests which package owns each runner and the task path that starts it.

 The table is held to the build entries of the family by
 `source-scan/src/corpus-run/command-lines.unit.test.ts`; the cases here hold it to the command-line
 declarations of this package and fix the path's shape.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMMAND_LINES,
  COMMAND_OWNERS,
  taskPathOf,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: taskPathOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE TASK OF THE PACKAGE THAT OWNS THE COMMAND, from the repository root, for a command of '
            + 'each of three packages',
          fn: async () => {
            expect(([
              'meter-report',
              'score-verify',
              'coverage-census',
            ] as const).map((command,) => taskPathOf({ command, },)),).toEqual([
              '//package/translation-repair/run:meter-report',
              '//package/translation-repair/probe:score-verify',
              '//package/translation-repair/coverage-census:coverage-census',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'COMMAND_OWNERS',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES AN OWNER FOR EVERY COMMAND the command-line declarations list and for no other, so a '
            + 'runner cannot be added without the package whose task starts it',
          fn: async () => {
            expect(Object.keys(COMMAND_OWNERS,)
              .toSorted(),).toEqual(Object.keys(COMMAND_LINES,)
                .toSorted(),);
          },
        },),

        it({
          name: 'WRITES EACH OWNER AS A PACKAGE DIRECTORY NAME: lower case, with no separator of a path or a task '
            + 'in it, so a path or a task cannot hide in one',
          fn: async () => {
            expect(Object.values(COMMAND_OWNERS,)
              .filter((owner,) =>
                (owner === '') || (owner !== owner.toLowerCase()) || owner.includes('/',) || owner.includes(':',)
                  || owner.includes(' ',)),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
