/**
 Identity of the host-side code that runs and judges the planted controls.

 A record of a passing run proves that this code, over this table of controls, noticed every removal. The table has its
 own digest, but the code that starts the containers, filters the tests and reads the outcome can change while the
 table and the image stay the same, and a changed filter or judge could stop noticing a removal.

 The record therefore names `plantedHarnessId`, a literal digest of the sources below that is part of the loaded code,
 not a digest read from disk afterwards: a file read after the modules loaded can describe other code than the code that
 executed. The identity is content-derived, so two branches that change the judge differently get different identities
 where a counter would collide. A unit test asserts that the literal equals the digest of the sources, so editing any of
 them without writing the new digest here fails the test, and the test prints the digest to write.
 */
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

/**
 Identity of the runner and judge: the digest of the sources in {@link plantedHarnessFiles} (see
 {@link plantedHarnessDigest}). After changing one of them, replace this literal with the digest the unit test prints.
 */
export const plantedHarnessId = '18ac2c8ab8e1026b295e1df9cda6107c5156ec572013c1730b7399f86153d797';

/**
 Sources, beside this file, that run the controls (`native-planted-controls.ts`: containers, nextest invocation, evidence),
 judge a run (`native-planted-controls-outcome.ts`: which output counts as noticed) and start the processes
 (`native-verification-process.ts`: how commands and containers run and fail). The tables of controls are not listed: the
 record's digest of the controls covers them.
 */
export const plantedHarnessFiles: readonly string[] = [
  'native-planted-controls.ts',
  'native-planted-controls-outcome.ts',
  'native-verification-process.ts',
];

/**
 Digest of the runner and judge source, which the unit test requires to equal {@link plantedHarnessId}.

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
