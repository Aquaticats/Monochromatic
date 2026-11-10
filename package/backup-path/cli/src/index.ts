/**
 `backup-path` CLI entrypoint.
 
 Copies one path into `./bak/<timestamp>/` in the working directory.
 
 TODO: deprecate Optique
 
 Importing this module is side-effect free: the program body runs only when this
 file is executed as the entrypoint (gated by `import.meta.main`), so unit tests
 can import {@link backupPath} without starting a backup.
 
 @example
 ```sh
 backup-path ./some/file-or-dir
 ```
 */

// TODO: deprecate Optique
import { argument, } from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';

import { backupPath, } from './backup-path.ts';

export {
  backupFolderName,
  backupPath,
  copyIntoBackup,
} from './backup-path.ts';

if (import.meta.main) {
  /**
   Parsed positional path argument from process.argv.
   */
  const path = runSync(
    argument(string(),),
    {
      programName: 'backup-path',
      help: 'option',
    },
  );

  console.log(`Backing up ${path}`,);
  await backupPath({
    sourcePath: path,
    backupRoot: 'bak',
  },);
}
