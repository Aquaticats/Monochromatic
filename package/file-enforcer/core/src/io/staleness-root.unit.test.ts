import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createMemoryRootFilesystem,
  findNodeModulesRoot,
} from '../../dist/final/node/index.mjs';

//region Cases

/**
 One discovery case over an in-memory tree.
 */
type RootCase = {
  /**
   Case name shown in the runner.
   */
  readonly name: string;
  /**
   Directories present in the tree.
   */
  readonly directories: readonly string[];
  /**
   Regular files present in the tree, path to content.
   */
  readonly files?: Readonly<Record<string, string>>;
  /**
   Directory the walk starts from.
   */
  readonly startDirectory: string;
  /**
   Directory the walk must answer with.
   */
  readonly expected: string;
};

/**
 Discovery cases: the nearest owner wins, the start directory counts, a
 file named `node_modules` is skipped, and no owner falls back to the start.
 */
const CASES: readonly RootCase[] = [
  {
    name: 'returns the nearest ancestor owning node_modules',
    directories: ['/w/node_modules', '/w/packages/app',],
    startDirectory: '/w/packages/app',
    expected: '/w',
  },
  {
    name: 'returns the start directory when it owns node_modules itself',
    directories: ['/w/node_modules',],
    startDirectory: '/w',
    expected: '/w',
  },
  {
    name: 'prefers a nearer node_modules over an outer one',
    directories: ['/w/node_modules', '/w/packages/app/node_modules', '/w/packages/app/src',],
    startDirectory: '/w/packages/app/src',
    expected: '/w/packages/app',
  },
  {
    name: 'skips a regular file named node_modules',
    directories: ['/w/node_modules', '/w/packages/app',],
    files: { '/w/packages/app/node_modules': '', },
    startDirectory: '/w/packages/app',
    expected: '/w',
  },
  {
    name: 'falls back to the start directory when no ancestor owns node_modules',
    directories: ['/w/packages/app',],
    startDirectory: '/w/packages/app',
    expected: '/w/packages/app',
  },
];

//endregion Cases

await describe({
  name: findNodeModulesRoot.name,
  children: CASES.map(function rootCase(row,) {
    return it({
      name: row.name,
      fn: async function runRootCase(): Promise<void> {
        /**
         In-memory filesystem holding the case's tree.
         */
        const fs = createMemoryRootFilesystem(
          row.files === undefined
            ? { directories: row.directories, }
            : {
              directories: row.directories,
              files: row.files,
            },
        );
        expect(await findNodeModulesRoot({
          fs,
          startDirectory: row.startDirectory,
        },),).toBe(row.expected,);
      },
    },);
  },),
},);
