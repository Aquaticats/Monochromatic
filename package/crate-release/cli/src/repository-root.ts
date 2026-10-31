import {
  dirname,
  join,
} from 'node:path';

import { repositoryMarkerFile } from './constants.ts';
import { CrateReleaseError } from './error.ts';

/**
 Find the repository root by walking up from a directory until one holds the repository marker file.

 The walk is bounded by the depth of the path, so the recursion ends at the filesystem root at the latest.

 @param startDirectory - directory to start from, normally the directory of this package

 @param exists - tells whether a path exists, injected so a test needs no real directories

 @param directory - directory under inspection, the start directory on the first call

 @returns the nearest directory, the start directory included, that holds the marker

 @throws CrateReleaseError when no directory up to the filesystem root holds the marker; the message names `--root`
 as the way to name the root directly

 @example
 ```ts
 findRepositoryRoot({ startDirectory: import.meta.dirname, exists: existsSync });
 ```
 */
export function findRepositoryRoot({
  startDirectory,
  exists,
  directory = startDirectory,
}: {
  readonly startDirectory: string;
  readonly exists: (path: string) => boolean;
  readonly directory?: string;
}): string {
  if (exists(join(
    directory,
    repositoryMarkerFile
  )))
    return directory;
  /**
   Parent directory, which is the directory itself at the filesystem root.
   */
  const parent = dirname(directory);
  if (parent === directory)
    throw new CrateReleaseError(
      `No ${repositoryMarkerFile} exists in ${startDirectory} or any directory above it; pass --root <repository root>.`,
    );
  return findRepositoryRoot({
    startDirectory,
    exists,
    directory: parent
  });
}
