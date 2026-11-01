/**
 The scratch libraries of the states sessions.

 @packageDocumentation
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { waveBytes, } from './library.ts';

/**
 Writes a short tone file, creating its folder.

 @param path - file to write

 @param seconds - length of the tone

 @example
 ```ts
 await writeTone({ path: join(
   root,
   'a.wav',
 ), seconds: 1, },);
 ```
 */
export async function writeTone(
  {
    path,
    seconds,
  }: {
    readonly path: string;
    readonly seconds: number;
  },
): Promise<void> {
  await mkdir(
    join(
      path,
      '..',
    ),
    { recursive: true, },
  );
  await writeFile(
    path,
    waveBytes({
      seconds,
      hertz: 330,
    },),
  );
}

/**
 Length of the two tracks of the first folder of the main library, in seconds. Long enough that the window title can be read
 while the first track still plays, even on a busy machine.
 */
const FOLDER_TRACK_SECONDS = 3;

/**
 Builds the main states library: one folder of two one-second tracks that share a prefix, and three loose files that
 sit in the virtual folders of their letters.

 @param root - scratch directory

 @returns the library directory

 @example
 ```ts
 const library = await buildMainLibrary(context.base,);
 ```
 */
export async function buildMainLibrary(root: string,): Promise<string> {
  /**
   Library directory.
   */
  const library = join(
    root,
    'states-main',
  );
  await Promise.all([
    writeTone({
      path: join(
        library,
        'Aaa Folder',
        'Aaa Folder - One.wav',
      ),
      seconds: FOLDER_TRACK_SECONDS,
    },),
    writeTone({
      path: join(
        library,
        'Aaa Folder',
        'Aaa Folder - Two.wav',
      ),
      seconds: FOLDER_TRACK_SECONDS,
    },),
    writeTone({
      path: join(
        library,
        'Loose 10.wav',
      ),
      seconds: 1,
    },),
    writeTone({
      path: join(
        library,
        'Loose 2.wav',
      ),
      seconds: 1,
    },),
    writeTone({
      path: join(
        library,
        'Tango.wav',
      ),
      seconds: 1,
    },),
  ],);
  return library;
}

/**
 Builds the library with one top-level folder whose tracks differ in their numbers and share a prefix.

 @param root - scratch directory

 @returns the library directory

 @example
 ```ts
 const library = await buildSingleLibrary(context.base,);
 ```
 */
export async function buildSingleLibrary(root: string,): Promise<string> {
  /**
   Library directory.
   */
  const library = join(
    root,
    'states-single',
  );
  await Promise.all([
    'Take 10 Zed.wav',
    'Take 2 Zed.wav',
    'Take 3 Zed.wav',
  ].map(function writeSingleTrack(file,): Promise<void> {
    return writeTone({
      path: join(
        library,
        'Only Artist',
        file,
      ),
      seconds: 1,
    },);
  },),);
  return library;
}

/**
 Name of the folder without audio.
 */
export const EMPTY_LIBRARY_NAME = 'states-empty-library';

/**
 Builds a folder that holds a picture and no audio.

 @param root - scratch directory

 @returns the library directory

 @example
 ```ts
 const library = await buildEmptyLibrary(context.base,);
 ```
 */
export async function buildEmptyLibrary(root: string,): Promise<string> {
  /**
   Library directory.
   */
  const library = join(
    root,
    EMPTY_LIBRARY_NAME,
  );
  await mkdir(
    library,
    { recursive: true, },
  );
  await writeFile(
    join(
      library,
      'cover.jpg',
    ),
    'not audio',
  );
  return library;
}

/**
 Builds the library of the refresh session: two short tracks that the checks then replace, add to and break.

 @param root - scratch directory

 @returns the library directory

 @example
 ```ts
 const library = await buildRefreshLibrary(context.base,);
 ```
 */
export async function buildRefreshLibrary(root: string,): Promise<string> {
  /**
   Library directory.
   */
  const library = join(
    root,
    'states-refresh',
  );
  await Promise.all([
    writeTone({
      path: join(
        library,
        'a.wav',
      ),
      seconds: 1,
    },),
    writeTone({
      path: join(
        library,
        'b.wav',
      ),
      seconds: 1,
    },),
  ],);
  return library;
}
