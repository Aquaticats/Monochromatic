import { readFile, } from 'node:fs/promises';
import { caughtErrorHasCode, } from './error.ts';
import { hashContent, } from './staleness-hash.ts';
import {
  fileStampListsMatch,
  readFileStamps,
} from './staleness-stamps.ts';
import {
  ABSENT_FILE_STAMPS,
  type DestinationStamp,
} from './staleness-types.ts';

/**
 Returns whether current destination metadata and content hashes match recorded stamps:
 compares filesystem metadata via {@link readFileStamps} and {@link fileStampListsMatch},
 then content via {@link hashContent}.
 
 @param recordedStamps - Destination stamps persisted in manifest.
 
 @returns Whether every destination is unchanged.
 
 @throws When destination content cannot be read for reasons other than absence
 (distinguished from absence via {@link caughtErrorHasCode}).
 
 @example
 ```ts
 const fresh = await destinationStampListsMatch({ recordedStamps });
 ```
 */
export async function destinationStampListsMatch(
  {
    recordedStamps,
  }: {
    readonly recordedStamps: readonly DestinationStamp[];
  },
): Promise<boolean> {
  /**
   Current filesystem metadata for every recorded destination.
   */
  const currentStamps = await readFileStamps(recordedStamps.map(function destinationPath(stamp,): string {
    return stamp.path;
  },),);
  if (currentStamps === ABSENT_FILE_STAMPS)
    return false;
  if (!fileStampListsMatch({
    currentStamps,
    recordedStamps,
  },))
    return false;

  /**
   Content hash comparison results for every recorded destination.
   */
  const hashMatches = await Promise.all(
    recordedStamps.map(async function destinationHashMatches(stamp,): Promise<boolean> {
      try {
        return hashContent(await readFile(
          stamp.path,
          'utf8',
        ),) === stamp.hash;
      }
      catch (hashReadError: unknown) {
        if (caughtErrorHasCode({
          error: hashReadError,
          code: 'ENOENT',
        },))
          return false;

        throw hashReadError;
      }
    },),
  );
  return hashMatches.every(function hashMatched(hashMatch,): boolean {
    return hashMatch;
  },);
}
