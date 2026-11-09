/**
 Tests that the shared pnpm store is mounted where this checkout's links expect it, read-only, and that the constant the
 mount derives from still matches the workspace setting that writes those links.

 @module
 */

import { readFileSync, } from 'node:fs';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pnpmStoreDirectory,
  pnpmStoreMountArguments,
  STORE_DIRECTORY_FROM_HOME,
} from '../dist/final/node/index.mjs';

/**
 Home directory the cases name, so no case depends on the machine's own.
 */
const HOME = '/home/fixture';

/**
 Store directory the cases expect under {@link HOME}.
 */
const STORE = join(
  HOME,
  STORE_DIRECTORY_FROM_HOME,
);

/**
 Text of the workspace file at the repository root, four folders above this test's package.
 */
const WORKSPACE_TEXT = readFileSync(
  join(
    import.meta.dirname,
    '..',
    '..',
    '..',
    '..',
    'pnpm-workspace.yaml',
  ),
  'utf8',
);

await describe({
  name: pnpmStoreDirectory.name,
  children: [
    it({
      name: 'puts the store under the home directory the setting names',
      fn: async () => {
        expect(pnpmStoreDirectory({ home: HOME, },),).toBe(STORE,);
      },
    },),
    it({
      name: 'defaults to the home directory of the runtime',
      fn: async () => {
        expect(pnpmStoreDirectory(),).toBe(join(
          homedir(),
          STORE_DIRECTORY_FROM_HOME,
        ),);
      },
    },),
  ],
},);

await describe({
  name: STORE_DIRECTORY_FROM_HOME,
  children: [
    it({
      name: 'is the setting pnpm-workspace.yaml writes, relative to the home directory',
      fn: async () => {
        /**
         Setting line of the workspace file, which the mount mirrors because it cannot read the file.
         */
        const setting = WORKSPACE_TEXT
          .split('\n',)
          .find((line,) => line.startsWith('storeDir:',),);
        expect(setting,).toBeDefined();
        expect(setting,).toBe(`storeDir: \${HOME}/${STORE_DIRECTORY_FROM_HOME}`,);
      },
    },),
  ],
},);

await describe({
  name: pnpmStoreMountArguments.name,
  children: [
    it({
      name: 'mounts the store at the path its links name and read-only',
      fn: async () => {
        expect(pnpmStoreMountArguments({ home: HOME, },),).toEqual([
          '--volume',
          `${STORE}:${STORE}:ro`,
        ],);
        expect(pnpmStoreMountArguments(),).toEqual([
          '--volume',
          `${join(
            homedir(),
            STORE_DIRECTORY_FROM_HOME,
          )}:${join(
            homedir(),
            STORE_DIRECTORY_FROM_HOME,
          )}:ro`,
        ],);
      },
    },),
    it({
      name: 'spells source and destination the same, so a relative link lands on the store',
      fn: async () => {
        /**
         Mount argument of the store, the one that is neither the option name nor a read-write or relabelled mount.
         */
        const [, mount,] = pnpmStoreMountArguments({ home: HOME, },);
        /**
         Where the mount reads from, and where the container sees it.
         */
        const [source, destination,] = (mount ?? '').split(':',);
        expect(source,).toBe(destination,);
      },
    },),
    it({
      name: 'never mounts the store writable or relabelled',
      fn: async () => {
        /**
         Options of the mount.
         */
        const options = pnpmStoreMountArguments({ home: HOME, },);
        expect(options,).toContain(`${STORE}:${STORE}:ro`,);
        expect(options.some((option,) => option.endsWith(':rw',),),).toBe(false,);
        expect(options.some((option,) => option.endsWith(',z',) || option.endsWith(',Z',),),).toBe(false,);
      },
    },),
  ],
},);
