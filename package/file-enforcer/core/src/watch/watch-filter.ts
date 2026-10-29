import { stat, } from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import {
  expandGlob,
  globWatchDirectory,
} from '../io/glob.ts';
import {
  globs,
  reads,
  writes,
  writeTimestamps,
} from '../tracker.ts';
import {
  nearestExistingDirectory,
  trackedGlobStaticDirectoryAffected,
} from './watch-path.ts';

/**
 Possible outcomes when classifying a filesystem event
 */
export type EventKind = 'source' | 'protected' | 'ignore';

/**
 Derives the unique set of parent directories that need chokidar watchers,
 covering every tracked read path, every tracked write path (for protection),
 and the config file itself. Each tracked glob's static root, found via
 {@link globWatchDirectory}, is reduced to its nearest existing ancestor
 with {@link nearestExistingDirectory}.
 
 @param configPath - Absolute path to the config file
 
 @returns Set of absolute directory paths to watch
 
 @example
 ```ts
 const dirs = await watchDirs('/abs/path/to/config.ts');
 ```
 */
export async function watchDirs(configPath: string,): Promise<Set<string>> {
  /**
   All paths that need monitoring: reads, writes, and the config
   */
  const allPaths = [
    ...reads,
    ...writes,
    resolve(configPath,),
  ];
  /**
   Nearest existing directories covering tracked glob static roots.
   */
  const globDirectories = await Promise.all(
    [...globs.keys(),].map(async function toGlobWatchDirectory(pattern,): Promise<string> {
      return await nearestExistingDirectory(globWatchDirectory(pattern,),);
    },),
  );
  return new Set([
    ...allPaths.map(function toDir(filePath,): string {
      return dirname(filePath,);
    },),
    ...globDirectories,
  ],);
}

/**
 Returns whether a path currently matches one of the tracked glob patterns,
 expanded fresh via {@link expandGlob}.
 
 @param absolutePath - Absolute path from filesystem event.
 
 @returns Whether the event path belongs to a tracked glob.
 
 @example
 ```ts
 const matched = await matchesTrackedGlob('/repo/src/new.ts');
 ```
 */
async function matchesTrackedGlob(absolutePath: string,): Promise<boolean> {
  /**
   Per-glob checks for whether current expansion contains the event path.
   */
  const matchResults = await Promise.all(
    [...globs.keys(),].map(async function globContainsPath(pattern,): Promise<boolean> {
      /**
       Current paths matched by this tracked glob.
       */
      const matchedPaths = await expandGlob(pattern,);
      return matchedPaths.some(function pathMatches(candidate,): boolean {
        return resolve(candidate,) === absolutePath;
      },);
    },),
  );
  return matchResults.some(function foundMatch(matched,): boolean {
    return matched;
  },);
}

/**
 Classifies a filesystem event into one of three categories:
 - `source`: a tracked read or the config file changed; normal re-run
 - `protected`: a managed destination was modified externally; re-run + notify
 - `ignore`: unrelated file or our own write echo; skip
 
 For write paths, uses `stat()` to compare the file's mtime against our
 recorded write timestamp. If mtime \> our timestamp, the edit is external.
 Read paths are matched against tracked glob static directories with
 {@link trackedGlobStaticDirectoryAffected} and against live glob expansions
 with {@link matchesTrackedGlob}.
 
 @param filename - Filename from the watch event (relative to watched dir)
 
 @param watchedDir - Absolute path of the directory being watched
 
 @param configPath - Absolute path of the config file
 
 @returns Classification of the event
 
 @example
 ```ts
 const kind = await classifyEvent({
   filename: 'index.ts',
   watchedDir: '/abs/src',
   configPath: '/abs/config.ts',
 });
 ```
 */
export async function classifyEvent(
  {
    filename,
    watchedDir,
    configPath,
  }: {
    readonly filename: string;
    readonly watchedDir: string;
    readonly configPath: string;
  },
): Promise<EventKind> {
  /**
   Absolute path of the changed file
   */
  const absolutePath = resolve(join(
    watchedDir,
    filename,
  ),);

  if (writes.has(absolutePath,)) {
    /**
     Timestamp of our last actual write, if any
     */
    const ourWriteTime = writeTimestamps.get(absolutePath,);
    if (ourWriteTime === undefined) {
      // We registered the dest but never actually wrote (content was unchanged).
      // An fs.watch event for this path must be external.
      return 'protected';
    }
    try {
      /**
       File metadata for mtime comparison
       */
      const fileStat = await stat(absolutePath,);
      // External edit: file was modified after our last write.
      // Floor mtimeMs because stat() returns sub-ms precision (float)
      // while Date.now() returns whole ms (integer), causing false positives
      // when both timestamps fall within the same millisecond.
      if (Math.floor(fileStat.mtimeMs,)
        > ourWriteTime)
        return 'protected';
    }
    catch (statError: unknown) {
      void statError;
      // File may have been deleted externally
      return 'protected';
    }
    // mtime <= our write time: this is our own write echoing through fs.watch
    return 'ignore';
  }

  /**
   Resolved config path for comparison
   */
  const resolvedConfig = resolve(configPath,);

  if (reads.has(absolutePath,)
    || (absolutePath === resolvedConfig)
    || trackedGlobStaticDirectoryAffected(absolutePath,)
    || await matchesTrackedGlob(absolutePath,))
    return 'source';

  return 'ignore';
}

/**
 Backwards-compatible wrapper around {@link classifyEvent} that returns true
 for any actionable event.
 
 @param filename - Filename from the watch event (relative to watched dir)
 
 @param watchedDir - Absolute path of the directory being watched
 
 @param configPath - Absolute path of the config file
 
 @returns Whether this event should trigger a re-run
 
 @example
 ```ts
 const trigger = await shouldTrigger({
   filename: 'index.ts',
   watchedDir: '/abs/src',
   configPath: '/abs/config.ts',
 });
 if (trigger) {
   // re-run config
 }
 ```
 */
export async function shouldTrigger(
  {
    filename,
    watchedDir,
    configPath,
  }: {
    readonly filename: string;
    readonly watchedDir: string;
    readonly configPath: string;
  },
): Promise<boolean> {
  return (await classifyEvent({
    filename,
    watchedDir,
    configPath,
  },)) !== 'ignore';
}
