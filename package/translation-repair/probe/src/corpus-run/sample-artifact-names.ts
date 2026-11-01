import { join, } from 'node:path';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type ArtifactFileName,
  artifactFilesIn,
  artifactsDirOf,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';

//region Sample artifact names
// Lists the settled artifacts of a runs directory for the two sampling
// commands, saying so in words when the runs directory holds no artifacts
// directory at all.
//
// A RUNS DIRECTORY NAMED WRONG IS THE ORDINARY MISTAKE here: the variable
// that names it is read from the environment, and a path with no artifacts
// directory under it is a run pointed somewhere else. The listing used to
// raise the filesystem's own error, which the command boundary reports as a
// fault in the command with a stack, though nothing in the command is broken.

/**
 Lists the artifact file names an artifacts directory holds.

 @param runsDir - runs directory a pass wrote into

 @returns Artifact file names, unsorted

 @throws {@link StatedRefusalError} When the runs directory holds no artifacts
 directory, or its artifacts directory cannot be listed; the filesystem's
 reason is named as its bounded code and nothing of the path's own error is
 quoted

 @example
 ```ts
 const names = await listSettledNames({ runsDir, },);
 ```
 */
export async function listSettledNames(
  { runsDir, }: { readonly runsDir: string; },
): Promise<readonly ArtifactFileName[]> {
  /**
   Directory the settled artifacts sit in.
   */
  const artifactsDir = artifactsDirOf({ runsDir, },);

  /**
   What listing it produced.
   */
  const reading = await artifactFilesIn({ dir: artifactsDir, },);
  if (reading.kind === 'read')
    return reading.names;

  if (reading.reason === 'ENOENT') {
    throw new StatedRefusalError({
      says: `there is no artifacts directory at ${
        join(
          runsDir,
          'artifacts',
        )
      }; name the runs directory a pass settled entries into with TRANSLATION_REPAIR_RUNS_DIR`,
    },);
  }
  throw new StatedRefusalError({
    says: `the artifacts directory at ${artifactsDir} could not be listed (${reading.reason}); name the runs `
      + 'directory a pass settled entries into with TRANSLATION_REPAIR_RUNS_DIR',
  },);
}

//endregion Sample artifact names
