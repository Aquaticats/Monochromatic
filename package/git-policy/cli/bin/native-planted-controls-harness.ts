/**
 Identity of the host-side code that runs and judges the planted controls.

 A record of a passing run proves that this code, over this table of controls, noticed every removal. The table has its
 own digest, but the code that starts the containers, filters the tests and reads the outcome can change while the
 table and the image stay the same, and a changed filter or judge could stop noticing a removal. The record therefore
 holds a digest of the source of that code, and a record made by other source is not proof.
 */
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

/**
 Sources, beside this file, that run the controls (`native-planted-controls.ts`: containers, nextest invocation, evidence)
 and judge a run (`native-planted-controls-outcome.ts`: which output counts as noticed).
 */
export const plantedHarnessFiles: readonly string[] = [
  'native-planted-controls.ts',
  'native-planted-controls-outcome.ts',
];

/**
 Digest of the runner and judge source.

 @param files - file names, relative to `directory`; defaults to {@link plantedHarnessFiles}

 @param directory - directory that holds them; defaults to this file's own directory

 @returns the hex SHA-256 over each file's name and bytes, in the order given

 @throws Error when a file cannot be read, because a record must never be judged against code that was not seen

 @example
 ```ts
 await plantedHarnessDigest({});
 ```
 */
export async function plantedHarnessDigest({
  files = plantedHarnessFiles,
  directory = import.meta.dirname,
}: {
  readonly files?: readonly string[];
  readonly directory?: string;
}): Promise<string> {
  /**
   Each file's name with the digest of its bytes.
   */
  const entries = await Promise.all(files.map(async function digestOf(file) {
    return [
      file,
      createHash('sha256')
        .update(await readFile(join(
          directory,
          file,
        )))
        .digest('hex'),
    ];
  }));
  return createHash('sha256')
    .update(JSON.stringify(entries))
    .digest('hex');
}
