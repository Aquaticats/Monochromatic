/**
 The name of the repository's own settings file, as the gate image receives it.
 */
import { access, } from 'node:fs/promises';
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
 Report whether a file exists.

 @param path - the file to look for

 @returns whether it exists

 @throws The error of the look when it is not "no such file", so a permission problem is not read as absence
 */
async function fileExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error: unknown) {
    if (!Error.isError(error,))
      throw error;
    if (!('code' in error))
      throw error;
    if (error.code !== 'ENOENT')
      throw error;
    return false;
  }
}

/**
 Find the repository's settings file the way the wrapper does: the current name when it exists, else the earlier name.

 @returns the file name, relative to the repository root, that the gate image copies to its `/work`

 @example
 ```ts
 await repositorySettingsFileName(); // => 'cli-git.config.jsonc' until the file is renamed
 ```
 */
export async function repositorySettingsFileName(): Promise<string> {
  if (await fileExists(resolve(
    '../../../',
    CURRENT_SETTINGS_FILE,
  ),))
    return CURRENT_SETTINGS_FILE;
  if (await fileExists(resolve(
    '../../../',
    EARLIER_SETTINGS_FILE,
  ),))
    return EARLIER_SETTINGS_FILE;
  return CURRENT_SETTINGS_FILE;
}
