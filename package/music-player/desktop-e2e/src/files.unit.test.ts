/**
 Tests for the small file-system probes.
 
 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ensureDirectory,
  pathExists,
  plantFile,
} from './files.ts';

await describe({
  name: pathExists.name,
  children: [
    it({
      name: 'is true for an existing directory and false for a missing path',
      fn: async ({ expect, }) => {
        /**
         Scratch directory of the test.
         */
        const dir = await mkdtemp(join(scratchRoot(), 'e2e-files-',),);
        expect(await pathExists(dir,),).toBe(true,);
        expect(
          await pathExists(join(dir, 'missing',),),
        ).toBe(false,);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
  ],
},);

await describe({
  name: ensureDirectory.name,
  children: [
    it({
      name: 'creates missing parents and accepts an existing directory',
      fn: async ({ expect, }) => {
        /**
         Scratch directory of the test.
         */
        const dir = await mkdtemp(join(scratchRoot(), 'e2e-files-',),);
        await ensureDirectory(join(dir, 'a', 'b',),);
        await ensureDirectory(join(dir, 'a', 'b',),);
        expect(
          await pathExists(join(dir, 'a', 'b',),),
        ).toBe(true,);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
  ],
},);

await describe({
  name: plantFile.name,
  children: [
    it({
      name: 'writes the file for the length of the scope and removes it when the scope ends',
      fn: async ({ expect, }) => {
        /**
         Scratch directory of the test.
         */
        const dir = await mkdtemp(join(scratchRoot(), 'e2e-files-',),);

        /**
         Where the planted file goes.
         */
        const path = join(dir, 'planted.txt',);
        {
          await using planted = await plantFile({ path, content: 'inside', settleAfterRemovalMs: 0, },);
          expect(planted.path,).toBe(path,);
          expect(await readFile(path, 'utf8',),).toBe('inside',);
        }
        expect(await pathExists(path,),).toBe(false,);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'accepts a file that the scope already removed',
      fn: async ({ expect, }) => {
        /**
         Scratch directory of the test.
         */
        const dir = await mkdtemp(join(scratchRoot(), 'e2e-files-',),);

        /**
         Where the planted file goes.
         */
        const path = join(dir, 'planted.txt',);
        {
          await using _planted = await plantFile({ path, content: new Uint8Array([1, 2,],), settleAfterRemovalMs: 0, },);
          await rm(path,);
        }
        expect(await pathExists(path,),).toBe(false,);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
  ],
},);
