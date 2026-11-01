/**
 Tests for where one entry's fixed English page goes in the mirrored tree, and for the names `publish-fixed.ts` still
 exports after the move of those names to their own module.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ENGLISH_PAGE_FILE,
  FIXED_TREE_DIR,
  fixedPagePath,
  PEOPLE_DIR,
} from '../../dist/final/node/index.mjs';

await describe({
  name: fixedPagePath.name,
  concurrency: 1,
  children: [
    it({
      name: 'MIRRORS THE CORPUS LAYOUT, people/<id>/page.en.md under the root of the tree it is given',
      fn: async () => {
        expect(fixedPagePath({
          publishDir: join('runs', FIXED_TREE_DIR,),
          entryId: 'CatEntry1',
        },),).toBe(join('runs', 'fixed', 'people', 'CatEntry1', 'page.en.md',),);
      },
    },),

    it({
      name: 'KEEPS THE NAMES OF THE TREE the package exports: the tree directory, the people directory and the page file',
      fn: async () => {
        expect([FIXED_TREE_DIR, PEOPLE_DIR, ENGLISH_PAGE_FILE,],).toEqual(['fixed', 'people', 'page.en.md',],);
      },
    },),
  ],
},);
