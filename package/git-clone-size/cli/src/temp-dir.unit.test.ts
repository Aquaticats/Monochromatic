/**
 Tests for `makeTempDir`, which gives a clone probe a scratch directory that is never in the system temporary
 directory and removes itself when the enclosing scope ends.

 @module
 */

import {
  existsSync,
  realpathSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { makeTempDir, } from './temp-dir.ts';

await describe({
  name: makeTempDir.name,
  children: [
    it({
      name: 'CREATES THE DIRECTORY OUTSIDE THE SYSTEM TEMPORARY DIRECTORY, because a clone left there fills a '
        + 'shared tmpfs quota that every other command on the machine also writes to',
      fn: async function staysOutOfTheSystemTemporaryDirectory() {
        await using directory = await makeTempDir({ prefix: 'gcs-test-', },);

        expect(existsSync(directory.path,),).toBe(true,);
        expect(realpathSync(directory.path,).startsWith(realpathSync(tmpdir(),),),).toBe(false,);
      },
    },),

    it({
      name: 'CREATES THE DIRECTORY UNDER THE PROJECT node_modules/.monochromatic, the gitignored place every '
        + 'tool of this repository keeps its scratch',
      fn: async function livesUnderTheProjectScratchDirectory() {
        await using directory = await makeTempDir({ prefix: 'gcs-test-', },);

        expect(directory.path.includes('/node_modules/.monochromatic/git-clone-size/',),).toBe(true,);
      },
    },),

    it({
      name: 'REMOVES THE DIRECTORY WHEN THE SCOPE ENDS, success or throw alike',
      fn: async function removesTheDirectoryOnDispose() {
        let path = '';
        {
          await using directory = await makeTempDir({ prefix: 'gcs-test-', },);
          path = directory.path;
          expect(existsSync(path,),).toBe(true,);
        }

        expect(existsSync(path,),).toBe(false,);
      },
    },),
  ],
},);
