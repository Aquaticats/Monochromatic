/**
 Copies the built bins to a folder outside the repository, so the check never dangles when the repository is away:
 `<data home>/monochromatic-dev/repository-link/` gets every built `.mjs` file (the bins and the chunks they import)
 and a link named like each bin. Run `mise run //package/repository-link/check:build` first. The data home is
 `XDG_DATA_HOME`, or `~/.local/share`, or the folder after `--data-home=`. It deletes nothing.

 @module
 */

import {
  chmod,
  copyFile,
  mkdir,
  readdir,
  symlink,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';
import {
  argv,
  env,
} from 'node:process';
import {
  INSTALL_FOLDER,
  INSTALLED_BIN_MODE,
} from './constants.ts';
import { pathExists, } from './path-exists.ts';

/**
 Names of the bins and the built file each one runs.
 */
const BINS: readonly (readonly [
  string,
  string
])[] = [
  [
    'repository-link-check',
    'check.mjs',
  ],
  [
    'repository-link-run',
    'run.mjs',
  ],
  [
    'repository-link-record',
    'record.mjs',
  ],
  [
    'repository-link-launch',
    'launch.mjs',
  ],
];

/**
 The built files of this package.
 */
const builtFolder = new URL(
  '../dist/final/node/',
  import.meta.url,
).pathname;

/**
 The data home to install below.
 */
const dataHome = argv.find(function isDataHome(argument,) {
  return argument.startsWith('--data-home=',);
},)
  ?.slice('--data-home='.length,)
  ?? env.XDG_DATA_HOME
  ?? join(
    homedir(),
    '.local',
    'share',
  );

/**
 The folder the copy goes to.
 */
const target = join(
  dataHome,
  INSTALL_FOLDER,
);

if (!await pathExists(join(
  builtFolder,
  'check.mjs',
),)) {
  throw new Error(`${builtFolder} has no built check.mjs; build the package first`,);
}
await mkdir(
  target,
  { recursive: true, },
);
await Promise.all((await readdir(builtFolder,)).filter(function isModule(name,) {
  return name.endsWith('.mjs',);
},)
  .map(async function copyModule(file,) {
    await copyFile(
      join(
        builtFolder,
        file,
      ),
      join(
        target,
        file,
      ),
    );
    await chmod(
      join(
        target,
        file,
      ),
      INSTALLED_BIN_MODE,
    );
  },),);
await Promise.all(BINS.map(async function linkBin([name, file,],) {
  if (!await pathExists(join(
    target,
    name,
  ),)) {
    await symlink(
      file,
      join(
        target,
        name,
      ),
    );
  }
},),);
console.log(`installed ${String(BINS.length,)} bins to ${target}`,);
