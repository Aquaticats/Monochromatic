import {
  addWatchedPaths,
  setWriteTimestamp,
  trackDest,
} from '../tracker.ts';
import { destinationStampListsMatch, } from './staleness-destination-match.ts';
import {
  fileStampListsMatch,
  globStampsMatch,
  readFileStamps,
} from './staleness-stamps.ts';
import {
  ABSENT_FILE_STAMPS,
  type DestinationStamp,
  type FileStamp,
  type GlobStamp,
} from './staleness-types.ts';

/**
 Decides whether recorded source, glob, and destination stamps still match disk:
 checks source files via {@link readFileStamps} and {@link fileStampListsMatch},
 destinations via {@link destinationStampListsMatch}, and globs via {@link globStampsMatch}.
 Shared by the single-entry and whole-manifest freshness checks so the
 definition of "fresh" lives in one place.
 
 @param sourceFiles - Recorded source file stamps.
 
 @param sourceGlobs - Recorded glob path-set stamps.
 
 @param destinationFiles - Recorded destination stamps with content hashes.
 
 @returns Whether every recorded stamp still matches current filesystem state.
 
 @throws When stat or destination reads fail for reasons other than absence.
 
 @example
 ```ts
 const fresh = await stampsAreFresh({ sourceFiles, sourceGlobs, destinationFiles });
 ```
 */
export async function stampsAreFresh(
  {
    sourceFiles,
    sourceGlobs,
    destinationFiles,
  }: {
    readonly sourceFiles: readonly FileStamp[];
    readonly sourceGlobs: readonly GlobStamp[];
    readonly destinationFiles: readonly DestinationStamp[];
  },
): Promise<boolean> {
  /**
   Current metadata for the recorded source paths.
   */
  const currentSources = await readFileStamps(sourceFiles.map(function sourcePath(sourceFile,): string {
    return sourceFile.path;
  },),);
  if (currentSources === ABSENT_FILE_STAMPS)
    return false;
  if (!fileStampListsMatch({
    currentStamps: currentSources,
    recordedStamps: sourceFiles,
  }))
    return false;
  if (!await destinationStampListsMatch({ recordedStamps: destinationFiles, }))
    return false;

  return await globStampsMatch(sourceGlobs,);
}

/**
 Restores source and destination paths of a fresh skip into the global tracker
 via {@link addWatchedPaths} and {@link trackDest} so watch mode keeps observing
 them even though the builder was skipped, recording each destination's
 timestamp with {@link setWriteTimestamp}.
 
 @param sourceFiles - Fresh source stamps.
 
 @param destinationFiles - Fresh destination stamps.
 
 @example
 ```ts
 registerFreshPaths({ sourceFiles, destinationFiles });
 ```
 */
export function registerFreshPaths(
  {
    sourceFiles,
    destinationFiles,
  }: {
    readonly sourceFiles: readonly FileStamp[];
    readonly destinationFiles: readonly DestinationStamp[];
  },
): void {
  addWatchedPaths(sourceFiles.map(function sourcePath(sourceFile,): string {
    return sourceFile.path;
  },),);
  destinationFiles.forEach(function registerDestination(destinationFile,): void {
    trackDest(destinationFile.path,);
    setWriteTimestamp({
      filePath: destinationFile.path,
      timestamp: Math.floor(destinationFile.mtimeMs,),
    },);
  },);
}
