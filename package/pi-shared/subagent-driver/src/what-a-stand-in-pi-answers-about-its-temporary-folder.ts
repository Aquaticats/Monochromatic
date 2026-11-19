/**
 The folder a stand-in Pi's own `os.tmpdir()` names, which is where a tool's whole output would be kept.

 The driver hands a run's Pi the variables that name that folder, so a test asks here what Pi's own process resolved. The path is a
 folder inside a test's disposable run folder and never a value the stand-in was handed: no credential of any environment reaches
 this line.

 @module
 */

import { appendFile, } from 'node:fs/promises';
// oxlint-disable-next-line no-restricted-syntax/no-system-tmpdir -- a stand-in reads the folder its own process was given, which is what a test asks it about; nothing of this package is written there.
import { tmpdir, } from 'node:os';

/**
 Writes the folder this process's `os.tmpdir()` names to the log a test reads.

 @param log - the file the test reads these lines from

 @example
 ```ts
 await logTheTemporaryFolderPiResolved({ log: '/run/fixture.log', },);
 ```
 */
export async function logTheTemporaryFolderPiResolved({ log, }: { readonly log: string },): Promise<void> {
  await appendFile(
    log,
    `temporary-folder ${tmpdir()}\n`,
  );
}
