/**
 The folder shape of the scratch library, as the app's folder list shows it: one entry per top-level folder, with the
 number of audio files it holds, and the folder a check picks when it needs a track pane full of rows.
 
 The checks read the shape from the library they built, so a folder is chosen by the name the app will show rather than
 by the row it happened to sit on, and a library of another shape (a generated one, an external one) needs no edit.
 
 @example
 ```ts
 import { longestFolder, topLevelFolders, } from './library-folders.ts';
 
 const folder = longestFolder({ folders: await topLevelFolders({ library, },), minimumTracks: 8, },);
 ```
 
 @packageDocumentation
 */

import type { Dirent, } from 'node:fs';
import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { CheckFailureError, } from './errors.ts';

/**
 Extensions of the audio files the app indexes. A file with any other extension is not a track.
 */
export const AUDIO_FILE_EXTENSIONS: readonly string[] = [
  'flac',
  'm4a',
  'mp3',
  'oga',
  'ogg',
  'opus',
  'wav',
];

/**
 One top-level folder of the library and how many tracks it holds.
 */
export type FolderTracks = {
  readonly name: string;
  readonly trackCount: number;
};

/**
 The folder a check picked, and the tracks it holds.
 */
export type ChosenFolder = FolderTracks;

/**
 Tells whether a file name ends with an extension of an audio file the app indexes.
 
 @param name - file name, without a directory
 
 @returns whether the app counts the file as a track
 
 @example
 ```ts
 isAudioFile('01 Mid tone.flac',); // true
 ```
 */
export function isAudioFile(name: string,): boolean {
  /**
   The extension of the name, in lower case, without its dot.
   */
  const extension = name
    .slice(name.lastIndexOf('.',) + 1,)
    .toLowerCase();
  return AUDIO_FILE_EXTENSIONS.includes(extension,);
}

/**
 Counts the audio files of a directory and every directory below it.
 
 @param directory - directory to count
 
 @returns the number of audio files under it
 
 @example
 ```ts
 await countAudioFiles('/work/library/Mmm Tones',); // 30
 ```
 */
async function countAudioFiles(directory: string,): Promise<number> {
  /**
   Entries of the directory, in any order.
   */
  const entries = await readdir(
    directory,
    { withFileTypes: true, },
  );
  /**
   One count per entry: a track is one, a directory is what it holds, anything else is nothing.
   */
  const counts = await Promise.all(entries.map(async function countEntry(entry,): Promise<number> {
    if (entry.isDirectory())
      return await countAudioFiles(join(
        directory,
        entry.name,
      ),);
    return (entry.isFile() && isAudioFile(entry.name,)) ? 1 : 0;
  },),);
  return counts.reduce(
    function total(
      soFar,
      count,
    ): number {
      return soFar + count;
    },
    0,
  );
}

/**
 Reads the entries of a directory, naming the path when it cannot be read.
 
 @param directory - directory to read
 
 @returns its entries, in any order
 
 @throws CheckFailureError when the directory cannot be read
 
 @example
 ```ts
 const entries = await readEntries('/work/library',);
 ```
 */
async function readEntries(directory: string,): Promise<readonly Dirent[]> {
  try {
    return await readdir(
      directory,
      { withFileTypes: true, },
    );
  }
  catch (error: unknown) {
    throw new CheckFailureError(
      `the library ${directory} cannot be read: ${caughtValueText(error,)}`,
    );
  }
}

/**
 Lists the top-level folders of a library with the number of tracks each holds.
 
 A track at the library root belongs to no folder: the app shows those under one entry of its own, which no folder of
 the library can be chosen by name.
 
 @param library - library directory
 
 @returns one entry per top-level directory that holds at least one track, in name order
 
 @throws CheckFailureError when the library directory cannot be read
 
 @example
 ```ts
 const folders = await topLevelFolders({ library: '/work/library', },);
 ```
 */
export async function topLevelFolders({ library, }: { readonly library: string; },): Promise<readonly FolderTracks[]> {
  /**
   Entries of the library root, in any order.
   */
  const entries = await readEntries(library,);
  /**
   One count per top-level directory, and nothing for a loose file.
   */
  const counted = await Promise.all(entries.filter(function isDirectory(entry,): boolean {
    return entry.isDirectory();
  },)
    .map(function countFolder(entry,): Promise<FolderTracks> {
      return countAudioFiles(join(
        library,
        entry.name,
      ),)
        .then(function folderOfTracks(trackCount,): FolderTracks {
          return {
            name: entry.name,
            trackCount,
          };
        },);
    },),);
  return counted
    .filter(function holdsTracks(folder,): boolean {
      return folder.trackCount > 0;
    },)
    .toSorted(function byName(
      left,
      right,
    ): number {
      return left.name
        .localeCompare(right.name,);
    },);
}

/**
 Picks the folder that holds the most tracks, which is the one whose track list fills the pane.
 
 @param folders - the top-level folders of the library
 
 @param minimumTracks - tracks the caller needs under the pane, at least
 
 @returns the folder with the most tracks, the first by name when two hold as many
 
 @throws CheckFailureError when the library holds no folder, or none holds as many tracks as the caller needs
 
 @example
 ```ts
 longestFolder({ folders, minimumTracks: 8, },); // { name: 'Mmm Tones', trackCount: 30 }
 ```
 */
export function longestFolder(
  {
    folders,
    minimumTracks,
  }: {
    readonly folders: readonly FolderTracks[];
    readonly minimumTracks: number;
  },
): ChosenFolder {
  if (folders.length === 0)
    throw new CheckFailureError(
      `the library holds no folder, so no folder can fill the track pane with ${String(minimumTracks,)} tracks`,
    );
  /**
   The folder with the most tracks, the first by name when two hold as many.
   */
  const longest = folders.reduce(function moreTracks(
    best,
    folder,
  ): FolderTracks {
    return folder.trackCount > best.trackCount ? folder : best;
  },);
  if (longest.trackCount < minimumTracks)
    throw new CheckFailureError(
      `the fullest folder ${longest.name} holds ${String(longest.trackCount,)} tracks, fewer than the ${String(minimumTracks,)} the click points need`,
    );
  return longest;
}

/**
 How many rows a walk through the folder list may step over to reach a folder.
 
 Every folder is one row, and every group heading the list inserts is one more; a list can hold no more headings than
 folders, because a heading starts a group of at least one name.
 
 @param folders - the top-level folders of the library
 
 @returns the number of steps a walk may take
 
 @example
 ```ts
 walkStepLimit({ folders, },); // 61 for thirty folders
 ```
 */
export function walkStepLimit({ folders, }: { readonly folders: readonly FolderTracks[]; },): number {
  return (folders.length * 2) + 1;
}
