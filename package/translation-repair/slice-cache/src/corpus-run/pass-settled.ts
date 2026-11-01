import {
  entryIdOfArtifact,
  listArtifactFiles,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';

//region Pass settled
// What the SCHEDULER counts as already done, read the same way the census reads
// it.
//
// These two answers had drifted apart, and the drift was invisible in both
// directions. The scheduler took any `*.json` NAME, so a directory or a symlink
// called `Mittens.json` marked Mittens settled and the entry was never run
// again; the census skips non-regular files, so no guard ever reported it. The
// entry silently ceased to exist, exactly as an unplaceable artifact does,
// without even an unplaceable artifact to name.
//
// Split from `corpus-pass.ts` at its line cap, and this is the seam worth
// splitting on: the scheduler asks a different question from every rate-
// producing reader, sees every generation on purpose, and now shares one
// listing with the reader that judges those files.

/**
 Entry ids this directory already carries an artifact for.

 Unfiltered by generation ON PURPOSE. A scheduler that skipped entries settled
 by another pipeline would re-run them into the same directory and mix
 generations, which is the failure the resume guard refuses outright.

 @param artifactsDir - directory holding one JSON per settled entry

 @returns Ids already settled, whatever produced them

 @internal

 @example
 ```ts
 const done = await artifactBackedIds({ artifactsDir, },);
 ```
 */
export async function artifactBackedIds(
  { artifactsDir, }: { readonly artifactsDir: string; },
): Promise<ReadonlySet<string>> {
  return new Set(
    (await listArtifactFiles({ artifactsDir, },))
      .map(function toId(name,): string {
        return entryIdOfArtifact({ name, },);
      },),
  );
}

//endregion Pass settled
