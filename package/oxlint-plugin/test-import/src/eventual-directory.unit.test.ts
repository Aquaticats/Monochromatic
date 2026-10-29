import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { eventualDirectories, } from '../dist/final/node/index.mjs';

/** Package root every case in this file derives directories against. */
const ROOT = '/repo/package/module/x';

/**
 Computes eventual directories for declared targets, sorted for stable comparison.
 
 @param shippingTargets - specifiers a manifest declared as entries
 
 @returns sorted normalized directories
 
 @example
 ```ts
 directoriesOf(['./dist/final/node/index.mjs']);
 ```
 */
function directoriesOf(shippingTargets: readonly string[],): readonly string[] {
  return eventualDirectories({
    packageRoot: ROOT,
    shippingTargets,
  },)
    .toSorted();
}

await describe({
  name: eventualDirectories.name,
  children: [
    it({
      name: 'always includes the default artifact root, even with no entries declared',
      fn: async () => {
        expect(directoriesOf([],),).toEqual([`${ROOT}/dist/final`,],);
      },
    },),
    it({
      name: 'adds the directory holding a declared entry',
      fn: async () => {
        expect(directoriesOf(['./dist/final/node/index.mjs',],),).toEqual([
          `${ROOT}/dist/final`,
          `${ROOT}/dist/final/node`,
        ],);
      },
    },),
    it({
      name: 'adds an entry directory outside the default root',
      fn: async () => {
        expect(directoriesOf(['dist/app/main.mjs',],),).toEqual([
          `${ROOT}/dist/app`,
          `${ROOT}/dist/final`,
        ],);
      },
    },),
    it({
      name: 'never counts the bare dist root, so intermediate output stays rejected',
      fn: async () => {
        expect(directoriesOf(['./dist/Face-Regular.otf',],),).toEqual([`${ROOT}/dist/final`,],);
      },
    },),
    it({
      name: 'discards entries pointing into source',
      fn: async () => {
        expect(directoriesOf([
          './src/index.ts',
          './src/cli.ts',
        ],),).toEqual([`${ROOT}/dist/final`,],);
      },
    },),
    it({
      name: 'never counts the package root itself',
      fn: async () => {
        expect(directoriesOf(['./index.mjs',],),).toEqual([`${ROOT}/dist/final`,],);
      },
    },),
    it({
      name: 'deduplicates entries resolving to one directory',
      fn: async () => {
        expect(directoriesOf([
          './dist/final/node/index.d.mts',
          './dist/final/node/index.mjs',
          './dist/final/node/extra.mjs',
        ],),).toEqual([
          `${ROOT}/dist/final`,
          `${ROOT}/dist/final/node`,
        ],);
      },
    },),
  ],
},);
