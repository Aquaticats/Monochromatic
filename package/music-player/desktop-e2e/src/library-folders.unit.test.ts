/**
 Tests for reading the folder shape of the scratch library: the top-level folders and how many tracks each holds, the
 folder a check that needs a full track pane picks, and the bound of a walk through the folder list.
 
 @module
 */

import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CheckFailureError, } from './errors.ts';
import {
  longestFolder,
  topLevelFolders,
  walkStepLimit,
} from './library-folders.ts';

/**
 Creates a file and any missing directory above it.
 
 @param path - file to create
 
 @param text - content, empty when absent
 
 @example
 ```ts
 await touch('/work/library/Mmm Tones/01 Mid tone.wav',);
 ```
 */
async function touch(path: string, text = '',): Promise<void> {
  await mkdir(
    join(
      path,
      '..',
    ),
    { recursive: true, },
  );
  await writeFile(
    path,
    text,
  );
}

await describe({
  name: topLevelFolders.name,
  children: [
    it({
      name: 'counts the audio files of each top-level folder, with a subfolder and a non-audio file counted once each way',
      fn: async ({ expect, },) => {
        await using scratch = await mkdtempDisposable(join(scratchRoot(), 'folders-',),);
        const library = join(scratch.path, 'library',);
        await touch(join(library, 'Aaa Tones', '01 Alpha tone.wav',),);
        await touch(join(library, 'Mmm Tones', '01 Mid tone.wav',),);
        await touch(join(library, 'Mmm Tones', '02 Mid tone.mp3',),);
        await touch(join(library, 'Mmm Tones', 'Disc', '03 Mid tone.flac',),);
        await touch(join(library, 'Mmm Tones', 'notes.txt',),);
        await touch(join(library, 'Loose at the root.wav',),);
        expect(await topLevelFolders({ library, },),).toStrictEqual([
          {
            name: 'Aaa Tones',
            trackCount: 1,
          },
          {
            name: 'Mmm Tones',
            trackCount: 3,
          },
        ],);
      },
    },),
    it({
      name: 'reads a library with no folder as no folder at all, and a missing library names the path',
      fn: async ({ expect, },) => {
        await using scratch = await mkdtempDisposable(join(scratchRoot(), 'folders-',),);
        const empty = join(scratch.path, 'library',);
        await mkdir(
          empty,
          { recursive: true, },
        );
        await touch(join(empty, 'Loose at the root.wav',),);
        expect(await topLevelFolders({ library: empty, },),).toStrictEqual([],);
        let caught: unknown;
        try {
          await topLevelFolders({ library: join(scratch.path, 'gone',), },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain(join(scratch.path, 'gone',),);
      },
    },),
  ],
},);

await describe({
  name: longestFolder.name,
  children: [
    it({
      name: 'picks the folder with the most tracks, and the first name when two folders hold as many',
      fn: async ({ expect, },) => {
        expect(longestFolder({
          folders: [
            {
              name: 'Aaa Tones',
              trackCount: 4,
            },
            {
              name: 'Mmm Tones',
              trackCount: 30,
            },
            {
              name: 'Nnn Tones',
              trackCount: 30,
            },
          ],
          minimumTracks: 8,
        },),).toStrictEqual({
          name: 'Mmm Tones',
          trackCount: 30,
        },);
      },
    },),
    it({
      name: 'fails when no folder holds the tracks the click points need, naming the largest folder and the minimum',
      fn: async ({ expect, },) => {
        let caught: unknown;
        try {
          longestFolder({
            folders: [
              {
                name: 'Aaa Tones',
                trackCount: 3,
              },
              {
                name: 'Mmm Tones',
                trackCount: 5,
              },
            ],
            minimumTracks: 8,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(CheckFailureError,);
        expect((caught as Error).message,).toContain('Mmm Tones',);
        expect((caught as Error).message,).toContain('5 tracks',);
        expect((caught as Error).message,).toContain('8',);
      },
    },),
    it({
      name: 'fails when the library holds no folder at all',
      fn: async ({ expect, },) => {
        let caught: unknown;
        try {
          longestFolder({
            folders: [],
            minimumTracks: 8,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(CheckFailureError,);
        expect((caught as Error).message,).toContain('no folder',);
      },
    },),
  ],
},);

await describe({
  name: walkStepLimit.name,
  children: [
    it({
      name: 'allows a row per folder and a row per group heading, so a walk can never stop early on a real library',
      fn: async ({ expect, },) => {
        expect(walkStepLimit({
          folders: [
            { name: 'A', trackCount: 1, },
            { name: 'B', trackCount: 1, },
          ],
        },),).toBe(5,);
        expect(walkStepLimit({ folders: [], },),).toBe(1,);
      },
    },),
  ],
},);
