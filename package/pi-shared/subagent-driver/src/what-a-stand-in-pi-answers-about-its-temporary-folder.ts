/**
 The folder a stand-in Pi's own `os.tmpdir()` names, which is where a tool's whole output would be kept, and whether that folder
 was already there when this start began.

 The driver hands a run's Pi the variables that name that folder, so a test asks here what Pi's own process resolved, and whether
 the driver had made the folder before it started Pi. The path is a folder inside a test's disposable run folder and never a value
 the stand-in was handed: no credential of any environment reaches these lines.

 @module
 */

import {
  appendFile,
  lstat,
} from 'node:fs/promises';
// oxlint-disable-next-line no-restricted-syntax/no-system-tmpdir -- a stand-in reads the folder its own process was given, which is what a test asks it about; nothing of this package is written there.
import { tmpdir, } from 'node:os';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The answer for a path that is not there.
 */
const NOT_THERE = 'ENOENT';

/**
 Says whether the folder this process's `os.tmpdir()` names was there when the process began.

 @param folder - the folder to ask about

 @returns `yes` when it is a folder, `no` when it is not there or is something else

 @throws the error the look gave, when it was not a missing path

 @example
 ```ts
 const there = await wasThere({ folder: '/run/temporary-files-pi-wrote', },);
 ```
 */
async function wasThere({ folder, }: { readonly folder: string },): Promise<'yes' | 'no'> {
  try {
    return (await lstat(folder,)).isDirectory() ? 'yes' : 'no';
  }
  catch (error) {
    if (errorCodeOf(error,) === NOT_THERE) {
      return 'no';
    }
    throw error;
  }
}

/**
 Writes the folder this process's `os.tmpdir()` names to the log a test reads, and whether it was there at this start.

 @param log - the file the test reads these lines from

 @example
 ```ts
 await logTheTemporaryFolderPiResolved({ log: '/run/fixture.log', },);
 ```
 */
export async function logTheTemporaryFolderPiResolved({ log, }: { readonly log: string },): Promise<void> {
  /**
   The folder this process's own `os.tmpdir()` resolved.
   */
  const folder = tmpdir();
  await appendFile(
    log,
    `temporary-folder ${folder}\n`,
  );
  await appendFile(
    log,
    `temporary-folder-was-there ${await wasThere({ folder, },)}\n`,
  );
}
