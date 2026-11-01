import { join, } from 'node:path';

import {
  type ArtifactFileName,
  artifactFilesIn,
  ARTIFACTS_DIR,
  artifactsDirOf,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';

//region Editor standing read listing
// WHICH FILES `editor-standing-read` READS AS ARTIFACTS under one directory the
// command line named, or the runs directory when it named none.
//
// TAKES A RUN DIRECTORY OR AN ARTIFACTS DIRECTORY, because both get typed. A
// run directory holds `artifacts/`; naming that directory itself should work
// rather than report an empty archive.
//
// RETURNS WHAT IT HAS TO SAY rather than printing it, since the directories are
// listed side by side: the procedure prints every line in the order the
// directories were named once all are listed.

/**
 What listing one named directory found, and what it has to say on stderr.

 @example
 ```ts
 const listing: StandingListing = { paths: ['/cattery/artifacts/Mittens.json',], said: [], };
 ```
 */
export type StandingListing = {
  /**
   Full paths of every artifact file found, empty where none is.
   */
  readonly paths: readonly string[];

  /**
   Lines for stderr, one naming why neither layout could be listed, else none.
   */
  readonly said: readonly string[];
};

/**
 Turns one directory's artifact names into paths.

 @param dir - directory the names came from

 @param names - artifact files the directory holds

 @returns Full paths of the artifact files

 @example
 ```ts
 const paths = artifactsAmong({ dir, names, },);
 ```
 */
function artifactsAmong(
  {
    dir,
    names,
  }: {
    readonly dir: string;
    readonly names: readonly ArtifactFileName[];
  },
): readonly string[] {
  return names.map(function toPath(name,): string {
    return join(
      dir,
      name,
    );
  },);
}

/**
 Lists artifact file paths under one named directory.

 REGULAR FILES ONLY, as the census and the scheduler list them. A directory
 named like an artifact used to be read as JSON and reported refused, and a
 symlink was read as one more artifact (ledger B64).

 @param path - run directory, or its artifacts directory

 @returns Full paths of every artifact file found, empty where none is, with
 a line for stderr naming why neither layout could be listed

 @example
 ```ts
 const listing = await standingArtifactPaths({ path, },);
 ```
 */
export async function standingArtifactPaths(
  { path, }: { readonly path: string; },
): Promise<StandingListing> {
  /**
   Nested layout, which is what a pass writes.
   */
  const nested = artifactsDirOf({ runsDir: path, },);

  /**
   What that subdirectory held, if it is there at all.
   */
  const under = await artifactFilesIn({ dir: nested, },);

  if (under.kind === 'read')
    return {
      paths: artifactsAmong({
        dir: nested,
        names: under.names,
      },),
      said: [],
    };

  /**
   Flat layout, reached only when the nested one is absent.
   */
  const flat = await artifactFilesIn({ dir: path, },);

  if (flat.kind === 'read')
    return {
      paths: artifactsAmong({
        dir: path,
        names: flat.names,
      },),
      said: [],
    };

  return {
    paths: [],
    said: [
      `editor-standing-read: no artifacts under ${path} (${under.reason} on `
        + `${ARTIFACTS_DIR}/, ${flat.reason} on the directory itself)`,
    ],
  };
}

//endregion Editor standing read listing
