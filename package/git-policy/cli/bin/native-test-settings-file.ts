/**
 The name of the repository's own settings file, as the gate image receives it.
 */
import { existsSync, } from 'node:fs';
import { resolve, } from 'node:path';

/**
 The settings file name of the current program.
 */
const CURRENT_SETTINGS_FILE = 'monochromatic-git-policy-cli.config.jsonc';

/**
 The settings file name an earlier release of the program read, which this repository keeps until the toolchain lock carries a
 release with the new name (`doc/planning/git-policy-cli-program-rename.md`).
 */
const EARLIER_SETTINGS_FILE = 'cli-git.config.jsonc';

/**
 Find the repository's settings file the way the wrapper does: the current name when it exists, else the earlier name.

 @returns the file name, relative to the repository root, that the gate image copies to its `/work`

 @example
 ```ts
 repositorySettingsFileName(); // => 'cli-git.config.jsonc' until the file is renamed
 ```
 */
export function repositorySettingsFileName(): string {
  if (!existsSync(resolve('../../../', CURRENT_SETTINGS_FILE,),) && existsSync(resolve('../../../', EARLIER_SETTINGS_FILE,),))
    return EARLIER_SETTINGS_FILE;
  return CURRENT_SETTINGS_FILE;
}
