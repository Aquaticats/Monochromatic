import { join, } from 'node:path';

import {
  type DirectoryReading,
  namesOfKind,
  readingOf,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Artifact file name
// WHAT NAMES AN ARTIFACT, read one way. A settled entry's artifact is a
// REGULAR FILE named `<entry id>.json` in an artifacts directory. Each reader
// used to read that for itself, filtering names by `.json` and slicing the
// suffix off, and two of them (settled-carve.ts and the rendering audit's run
// sets) skipped the regular-file check the census and the scheduler apply, so
// a directory or a symlink named like an artifact became a settled entry
// there (ledger B64). The suffix is a type as well as a check: a name that
// passed `isArtifactFileName` carries it, and `entryIdOfArtifact` takes only
// such a name, so no reader slices a name that lacks it.

/**
 What ends an artifact's file name.
 */
const ARTIFACT_SUFFIX = '.json';

/**
 Directory under a runs directory holding one artifact per settled entry.
 */
export const ARTIFACTS_DIR = 'artifacts';

/**
 The artifacts directory of one runs directory.

 @param runsDir - runs directory a pass writes into

 @returns Path of its artifacts directory

 @example
 ```ts
 const artifactsDir = artifactsDirOf({ runsDir, },);
 ```
 */
export function artifactsDirOf({ runsDir, }: { readonly runsDir: string; },): string {
  return join(
    runsDir,
    ARTIFACTS_DIR,
  );
}

/**
 An artifact's file name: an entry id and the suffix.

 @example
 ```ts
 const name: ArtifactFileName = 'Mittens.json';
 ```
 */
export type ArtifactFileName = `${string}.json`;

/**
 Whether a file name names an artifact. A plain parameter, since it narrows
 the name itself and serves as a filter callback.

 @param name - file name, without a directory

 @returns Whether it ends in the artifact suffix

 @example
 ```ts
 const names = listed.filter(isArtifactFileName,);
 ```
 */
export function isArtifactFileName(name: string,): name is ArtifactFileName {
  return name.endsWith(ARTIFACT_SUFFIX,);
}

/**
 The entry id an artifact's file name carries, which is the name without its
 suffix. A file called exactly `.json` carries the empty id.

 @param name - artifact file name

 @returns Entry id

 @example
 ```ts
 const entryId = entryIdOfArtifact({ name: 'Mittens.json', },); // 'Mittens'
 ```
 */
export function entryIdOfArtifact({ name, }: { readonly name: ArtifactFileName; },): string {
  return name.slice(
    0,
    -ARTIFACT_SUFFIX.length,
  );
}

/**
 The file name an entry's artifact is written under and read back from.

 @param entryId - corpus entry

 @returns Artifact file name

 @example
 ```ts
 const name = artifactFileNameOf({ entryId: 'Mittens', },); // 'Mittens.json'
 ```
 */
export function artifactFileNameOf({ entryId, }: { readonly entryId: string; },): ArtifactFileName {
  return `${entryId}${ARTIFACT_SUFFIX}`;
}

/**
 Lists the artifacts a directory holds: its REGULAR FILES named like one.

 Directory entries are checked rather than assumed (`namesOfKind`). A
 directory named `backup.json` otherwise reached `readFile` and threw EISDIR
 out of the whole census, and a symlink was followed wherever it pointed,
 which could duplicate another artifact under a second identity or leave the
 directory entirely. Neither is an artifact, and neither should cost more
 than being skipped.

 @param artifactsDir - directory holding one JSON per settled entry

 @returns Artifact file names, unsorted

 @example
 ```ts
 const names = await listArtifactFiles({ artifactsDir, },);
 ```
 */
export async function listArtifactFiles(
  { artifactsDir, }: { readonly artifactsDir: string; },
): Promise<readonly ArtifactFileName[]> {
  return (await namesOfKind({
    dir: artifactsDir,
    kind: 'file',
  },))
    .filter(isArtifactFileName,);
}

/**
 Lists the artifacts a directory holds, reporting an absent or unreadable
 directory rather than raising, for readers that choose between layouts or
 report a missing run as a finding.

 ABSENCE IS A KIND, as it is for any directory listing here: a directory
 holding no artifact and a directory that is not there both leave nothing to
 read, but only one says the caller was pointed somewhere real.

 @param dir - directory that may hold artifacts

 @returns Artifact file names, or why the directory could not be listed

 @example
 ```ts
 const listing = await artifactFilesIn({ dir: artifactsDirOf({ runsDir, },), },);
 ```
 */
export async function artifactFilesIn(
  { dir, }: { readonly dir: string; },
): Promise<DirectoryReading<ArtifactFileName>> {
  return await readingOf({
    list: async function listed(): Promise<readonly ArtifactFileName[]> {
      return await listArtifactFiles({ artifactsDir: dir, },);
    },
  },);
}

//endregion Artifact file name
