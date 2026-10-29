/**
 Tests for the directory the picture reader keeps its scratch under: inside a project beside the nearest
 `node_modules`, outside any project in the user cache, and never the system temporary directory.

 WHAT THESE PIN is where the parent lands for each of the three places the working directory, the cache variable and
 the home directory can put it, that it exists afterwards, and that asking twice is not an error. Every case hands the
 function a throwaway working directory, environment and home, so none reads or writes the real ones.

 Fixtures are cat-themed invention.

 @module
 */

import {
  mkdir,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { scratchParent, } from '../dist/final/node/index.mjs';

/**
 Name of the directory this tool keeps its scratch in, under whichever base the place picks.
 */
const TOOL_DIRECTORY = 'translation-repair-slice-reading';

/**
 Reports whether a path is an existing directory.

 @param path - path to probe

 @returns whether `path` exists and is a directory

 @example
 ```ts
 const there = await isDirectory({ path: '/whiskers/cache', },);
 ```
 */
async function isDirectory({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    return (await stat(path,)).isDirectory();
  }
  catch (error) {
    if (Error.isError(error,) && ('code' in error) && (error.code === 'ENOENT'))
      return false;
    throw error;
  }
}

await describe({
  name: scratchParent.name,
  children: [
    it({
      name: 'LANDS BESIDE THE NEAREST NODE_MODULES inside a project, in `.monochromatic`, and makes it, whatever the '
        + 'cache variable says, since the project is the first place the function looks',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'scratch-parent-project-', },);
        const nodeModules = join(
          root.path,
          'node_modules',
        );
        const cwd = join(
          root.path,
          'litter',
          'tray',
        );
        const cache = join(
          root.path,
          'cache',
        );
        await mkdir(
          nodeModules,
          { recursive: true, },
        );
        await mkdir(
          cwd,
          { recursive: true, },
        );

        const parent = await scratchParent({
          cwd,
          env: { XDG_CACHE_HOME: cache, },
          home: join(
            root.path,
            'home',
          ),
        },);

        expect(parent,).toBe(join(
          nodeModules,
          '.monochromatic',
          TOOL_DIRECTORY,
        ),);
        expect(await isDirectory({ path: parent, },),).toBe(true,);
        expect(await isDirectory({ path: cache, },),).toBe(false,);
      },
    },),

    it({
      name: 'LANDS UNDER THE CACHE VARIABLE outside any project, in `monochromatic-dev`, and makes it, without '
        + 'touching the home directory',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'scratch-parent-cache-', },);
        const cache = join(
          root.path,
          'cache',
        );
        const home = join(
          root.path,
          'home',
        );

        const parent = await scratchParent({
          cwd: root.path,
          env: { XDG_CACHE_HOME: cache, },
          home,
        },);

        expect(parent,).toBe(join(
          cache,
          'monochromatic-dev',
          TOOL_DIRECTORY,
        ),);
        expect(await isDirectory({ path: parent, },),).toBe(true,);
        expect(await isDirectory({ path: home, },),).toBe(false,);
      },
    },),

    it({
      name: 'LANDS UNDER THE HOME\'S `.cache` outside any project when the cache variable is unset, and makes it',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'scratch-parent-home-', },);
        const home = join(
          root.path,
          'home',
        );

        const parent = await scratchParent({
          cwd: root.path,
          env: {},
          home,
        },);

        expect(parent,).toBe(join(
          home,
          '.cache',
          'monochromatic-dev',
          TOOL_DIRECTORY,
        ),);
        expect(await isDirectory({ path: parent, },),).toBe(true,);
      },
    },),

    it({
      name: 'RETURNS THE SAME PATH WHEN ASKED AGAIN and does not fail on the directory it already made, the way every '
        + 'reading after the first asks',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'scratch-parent-twice-', },);
        const place = {
          cwd: root.path,
          env: {
            XDG_CACHE_HOME: join(
              root.path,
              'cache',
            ),
          },
          home: join(
            root.path,
            'home',
          ),
        };

        const first = await scratchParent(place,);
        const second = await scratchParent(place,);

        expect(second,).toBe(first,);
        expect(await isDirectory({ path: second, },),).toBe(true,);
      },
    },),
  ],
},);
