/**
 The metadata files of the btrfs test double: how it records a subvolume (a marker for a source, a `.meta` file for a
 snapshot, a received copy), reads it back, and lists every subvolume below a folder the way `btrfs subvolume list -R`
 does.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';
import { pathExists, } from './completion-records.ts';

/**
 Name of the marker file of a source subvolume.
 */
export const SOURCE_MARKER = '.stub-subvolume';

/**
 Name of the metadata file of every other subvolume.
 */
export const META_FILE = '.meta';

/**
 The ID of the first row `subvolume list` prints.
 */
export const LIST_FIRST_ID = 300;

/**
 The key and value pairs of one subvolume's metadata.
 */
export type Meta = Record<string, string>;

/**
 Path of the metadata file of a subvolume.

 @param folder - the subvolume folder
 
 @returns the path

 @example
 ```ts
 import { metaPath, } from './btrfs-test-double-meta.ts';
 
 const result = metaPath(folder,);
 ```
 */
export function metaPath(folder: string,): string {
  return join(
    folder,
    META_FILE,
  );
}

/**
 Reads the metadata of a subvolume, or the implied metadata of a source subvolume.

 @param folder - the subvolume folder
 
 @returns the metadata
 
 @throws Error when the folder is not a subvolume of this double

 @example
 ```ts
 import { readMeta, } from './btrfs-test-double-meta.ts';
 
 const result = await readMeta(folder,);
 ```
 */
export async function readMeta(folder: string,): Promise<Meta> {
  if (await pathExists(metaPath(folder,),)) {
    return Object.fromEntries((await readFile(
      metaPath(folder,),
      'utf8',
    )).split('\n',)
      .filter(function notBlank(line,) {
        return line !== '';
      },)
      .map(function toPair(line,): readonly [
        string,
        string,
      ] {
        /**
         The text before and after the first equals sign.
         */
        const [key = '', value = '',] = line.split(
          '=',
          2,
        );
        return [
          key,
          value,
        ];
      },),);
  }
  /**
   The marker this step searches for.
   */
  const marker = join(
    folder,
    SOURCE_MARKER,
  );
  if (!await pathExists(marker,)) {
    throw new Error(`not a subvolume: ${folder}`,);
  }
  return {
    uuid: (await readFile(
      marker,
      'utf8',
    )).trim(),
    parent: '-',
    received: '-',
    ro: 'false',
  };
}

/**
 Reads the metadata of a snapshot source. A folder with no marker and no metadata file that holds nothing is an empty
 subvolume made by `subvolume create`, which this double does not mark (a marker would be a file for `rsync` to see).

 @param folder - the source folder

 @returns the metadata

 @throws Error when the folder is neither a subvolume of this double nor empty

 @example
 ```ts
 import { readSourceMeta, } from './btrfs-test-double-meta.ts';
 
 const result = await readSourceMeta(folder,);
 ```
 */
export async function readSourceMeta(folder: string,): Promise<Meta> {
  if ((!await pathExists(metaPath(folder,),)) && (!await pathExists(join(
    folder,
    SOURCE_MARKER,
  ),))
    && ((await readdir(folder,)).length === 0)) {
    return {
      uuid: `u-${basename(folder,)}`,
      parent: '-',
      received: '-',
      ro: 'false',
      empty: 'true',
    };
  }
  return await readMeta(folder,);
}

/**
 Writes the metadata of a subvolume.

 @param folder - the subvolume folder
 
 @param meta - the metadata

 @example
 ```ts
 import { writeMeta, } from './btrfs-test-double-meta.ts';
 
 await writeMeta({ folder, meta, },);
 ```
 */
export async function writeMeta(
  {
    folder,
    meta,
  }: {
    readonly folder: string;
    readonly meta: Readonly<Meta>;
  },
): Promise<void> {
  await writeFile(
    metaPath(folder,),
    `${
      Object.entries(meta,)
        .map(function toLine([key, value,]: readonly [
          string,
          string
        ],) {
        return `${key}=${value}`;
      },)
        .join('\n',)
    }\n`,
  );
}

/**
 Lists every subvolume of the double below a folder the way `btrfs subvolume list -R` prints it: one row each with its
 received UUID, UUID and path from the folder.

 @param root - the folder that stands for the filesystem

 @returns the rows, one per line

 @example
 ```ts
 import { listSubvolumes, } from './btrfs-test-double-meta.ts';
 
 const result = await listSubvolumes(root,);
 ```
 */
export async function listSubvolumes(root: string,): Promise<string> {
  /**
   Every file below the folder, as paths from it.
   */
  const entries = await readdir(
    root,
    { recursive: true, },
  );
  /**
   The folders that hold a metadata file.
   */
  const folders = entries.filter(function isMeta(entry,) {
    return basename(entry,) === META_FILE;
  },)
    .map(function folderOf(entry,) {
      return dirname(entry,);
    },)
    .toSorted();
  /**
   One row per subvolume.
   */
  const rows = await Promise.all(folders.map(async function rowOf(
    folder,
    index,
  ) {
    /**
     The metadata of this subvolume.
     */
    const meta = await readMeta(join(
      root,
      folder,
    ),);
    return `ID ${String(LIST_FIRST_ID + index,)} gen 1 top level 5 received_uuid ${meta.received ?? '-'} uuid ${meta.uuid ?? '-'} path ${folder}`;
  },),);
  return rows.join('\n',);
}
