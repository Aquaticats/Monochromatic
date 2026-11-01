import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { artifactFilesIn, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';

//region Score artifacts directory
// What the `score-*` runners that read a run's settled artifacts check before
// they read any: that the directory is there.

/**
 Refuses a runs directory whose artifacts directory cannot be listed.

 A directory that is not there says the run was pointed somewhere wrong, which
 is an answer, where the listing's own failure reaches the operator as a fault
 in the command with a stack and no path. A directory that is there and holds
 nothing is left to the pool, which states that refusal itself.

 @param artifactsDir - directory the pass writes entries into

 @throws {@link StatedRefusalError} when the directory is absent, is not a
 directory or cannot be listed, naming the path and the filesystem reason

 @example
 ```ts
 await requireArtifactsDir({ artifactsDir: artifactsDirOf({ runsDir, },), },);
 ```
 */
export async function requireArtifactsDir({ artifactsDir, }: { readonly artifactsDir: string; },): Promise<void> {
  /**
   What listing the directory found.
   */
  const listing = await artifactFilesIn({ dir: artifactsDir, },);
  if (listing.kind === 'read')
    return;

  throw new StatedRefusalError({
    says: `cannot list ${artifactsDir} (${listing.reason}): name a runs directory that holds an `
      + '`artifacts` directory with TRANSLATION_REPAIR_RUNS_DIR',
  },);
}

//endregion Score artifacts directory
