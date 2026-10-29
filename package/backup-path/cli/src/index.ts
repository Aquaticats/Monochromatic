// TODO: deprecate Optique
import { argument, } from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';
import {
  cp,
  mkdir,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

/**
 TODO: deprecate Optique
 Parsed positional path argument from process.argv.
 
 @example
 ```sh
 backup-path ./some/file-or-dir
 ```
 */
const path = runSync(
  argument(string(),),
  {
    programName: 'backup-path',
    help: 'option',
  },
);

console.log(`Backing up ${path}`,);
/**
 Current ISO timestamp with colons removed, used as the backup subdirectory name
 */
const now = new Date().toISOString()
  .replaceAll(
  ':',
  '',
  );
await mkdir(join(
  'bak',
  now,
),);
await cp(
  path,
  join(
    'bak',
    now,
    basename(path,),
  ),
  {
    recursive: true,
    errorOnExist: true,
    preserveTimestamps: true,
  },
);
