/**
 The daily update's file handling: path joins, reading a group's files before it runs, finding which changed, and putting
 files back when a step fails.

 @module
 */

import type { DailyUpdateEffects, } from './daily-update-run-types.ts';
import {
  type PathSelector,
  selects,
  type UpdateGroup,
} from './daily-update-plan.ts';

/**
 Joins a repository and a repository-relative path.

 @param repository - absolute repository path

 @param repositoryPath - repository-relative path

 @returns the absolute path

 @example
 ```ts
 absoluteOf({ repository: '/r', repositoryPath: 'a/b', },); // '/r/a/b'
 ```
 */
export function absoluteOf({
  repository,
  repositoryPath,
}: {
  readonly repository: string;
  readonly repositoryPath: string;
},): string {
  return `${repository}/${repositoryPath}`;
}

/**
 The repository-relative part of an absolute path inside the repository.

 @param repository - absolute repository path

 @param absolutePath - absolute path

 @returns the relative path, or the empty string for a path outside the repository

 @example
 ```ts
 relativeOf({ repository: '/r', absolutePath: '/r/a/b', },); // 'a/b'
 ```
 */
export function relativeOf({
  repository,
  absolutePath,
}: {
  readonly repository: string;
  readonly absolutePath: string;
},): string {
  return absolutePath.startsWith(`${repository}/`,) ? absolutePath.slice(repository.length + 1,) : '';
}

/**
 One snapshot entry with named parts, so callbacks destructure properties instead of a positional tuple.
 */
export type SnapshotFile = {
  /**
   Absolute path of the file.
   */
  readonly absolutePath: string;
  /**
   Text the file had when the snapshot was taken.
   */
  readonly text: string;
};

/**
 Names the parts of every snapshot entry.

 @param snapshot - absolute path to original text

 @returns one named pair per entry, in snapshot order

 @example
 ```ts
 snapshotFilesOf(new Map([['/r/a', 'x',],],),); // [{ absolutePath: '/r/a', text: 'x', },]
 ```
 */
export function snapshotFilesOf(snapshot: ReadonlyMap<string, string>,): readonly SnapshotFile[] {
  /**
   Entries in snapshot order.
   */
  const files: SnapshotFile[] = [];
  for (const [absolutePath, text,] of snapshot) {
    files.push({
      absolutePath,
      text,
    },);
  }
  return files;
}

/**
 Puts files back to the text they had before the group began.

 @param effects - the outside world

 @param snapshot - absolute path to original text

 @param selector - which files to put back; absent means every file in the snapshot, external ones included

 @returns the absolute paths whose text had to be written back

 @example
 ```ts
 await restoreFiles({ effects, snapshot, },);
 ```
 */
export async function restoreFiles({
  effects,
  snapshot,
  selector,
}: {
  readonly effects: DailyUpdateEffects;
  readonly snapshot: ReadonlyMap<string, string>;
  readonly selector?: PathSelector;
},): Promise<readonly string[]> {
  /**
   Snapshot files among those to put back.
   */
  const wantedFiles = snapshotFilesOf(snapshot,)
    .filter(function isWanted({ absolutePath, }: SnapshotFile,): boolean {
      return (selector === undefined)
        || selects({
          selector,
          repositoryPath: relativeOf({
            repository: effects.repository,
            absolutePath,
          },),
        },);
    },);
  // Files are independent of each other, so they are checked and written back side by side.
  /**
   Per wanted file: its path when it was written back, empty when it already had its snapshot text.
   */
  const written = await Promise.all(wantedFiles.map(async function putBack({
    absolutePath,
    text,
  }: SnapshotFile,): Promise<readonly string[]> {
    // Reading before writing keeps untouched files untouched, so their modification times stay.
    if (await effects.readText(absolutePath,) === text)
      return [];
    await effects.writeText({
      absolutePath,
      text,
    },);
    return [absolutePath,];
  },),);
  return written.flat();
}

/**
 Reads the text of every file a group may change, before its first command runs.

 @param effects - the outside world

 @param group - the group

 @returns absolute path to text

 @example
 ```ts
 const snapshot = await snapshotGroup({ effects, group, },);
 ```
 */
export async function snapshotGroup({
  effects,
  group,
}: {
  readonly effects: DailyUpdateEffects;
  readonly group: UpdateGroup;
},): Promise<ReadonlyMap<string, string>> {
  /**
   Tracked files the group owns, then the files outside the repository.
   */
  const absolutePaths: string[] = [];
  for (const repositoryPath of await effects.trackedPaths()) {
    if (selects({
      selector: group.owns,
      repositoryPath,
    },)) {
      absolutePaths.push(absoluteOf({
        repository: effects.repository,
        repositoryPath,
      },),);
    }
  }
  absolutePaths.push(...group.externalFiles,);

  // Reading one file does not depend on reading another, so the files are read side by side.
  /**
   Each file's path paired with its text.
   */
  const entries = await Promise.all(absolutePaths.map(async function readOne(absolutePath,): Promise<[
    string,
    string
  ]> {
    return [
      absolutePath,
      await effects.readText(absolutePath,),
    ];
  },),);
  return new Map(entries,);
}

/**
 Lists the owned files whose text now differs from the snapshot.

 @param effects - the outside world

 @param snapshot - texts taken before the group began

 @returns repository-relative paths

 @example
 ```ts
 const changed = await changedOwnedPaths({ effects, snapshot, },);
 ```
 */
export async function changedOwnedPaths({
  effects,
  snapshot,
}: {
  readonly effects: DailyUpdateEffects;
  readonly snapshot: ReadonlyMap<string, string>;
},): Promise<readonly string[]> {
  // Files are independent of each other, so they are read side by side.
  /**
   Per file: its repository-relative path when its text changed, empty for an unchanged or external file.
   */
  const changed = await Promise.all(snapshotFilesOf(snapshot,)
    .map(async function changedPath({
      absolutePath,
      text,
    }: SnapshotFile,): Promise<readonly string[]> {
    /**
     Repository-relative path, empty for an external file.
     */
      const repositoryPath = relativeOf({
        repository: effects.repository,
        absolutePath,
      },);
      if ((repositoryPath !== '') && (await effects.readText(absolutePath,) !== text))
        return [repositoryPath,];
      return [];
    },),);
  return changed.flat();
}
