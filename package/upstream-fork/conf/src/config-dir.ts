/**
 Per-user config directory resolution.
 
 Replaces upstream `conf`'s `env-paths` dependency with the same three
 platform layouts upstream resolves,
 so a migrating app finds its config file at the identical path. Resolution
 happens per call (not module load) so injected homes take effect,
 which the repository's disposable-home test rule requires.
 
 @module
 */

import { homedir, } from 'node:os';
import path from 'node:path';
import process from 'node:process';

//region Platform layouts

/**
 macOS layout: `~/Library/Preferences/<name>`.
 
 @param name - App name including any suffix.
 
 @returns Absolute config directory for a macOS user.
 
 @example
 ```ts
 macosConfigDirectory('foo-nodejs'); // => '/Users/me/Library/Preferences/foo-nodejs'
 ```
 */
function macosConfigDirectory(name: string,): string {
  /**
   Per-user Library directory macOS keeps preferences in.
   */
  const library = path.join(
    homedir(),
    'Library',
  );
  return path.join(
    library,
    'Preferences',
    name,
  );
}

/**
 Windows layout: `%APPDATA%\<name>\Config`,
 falling back to `AppData\Roaming`.
 
 @param name - App name including any suffix.
 
 @returns Absolute config directory for a Windows user.
 
 @example
 ```ts
 windowsConfigDirectory('foo-nodejs'); // => 'C:\\Users\\me\\AppData\\Roaming\\foo-nodejs\\Config'
 ```
 */
function windowsConfigDirectory(name: string,): string {
  /**
   Roaming application-data root Windows stores per-app config under.
   */
  const appData = process.env
    .APPDATA
    ?? path.join(
      homedir(),
      'AppData',
      'Roaming',
    );
  return path.join(
    appData,
    name,
    'Config',
  );
}

/**
 freedesktop.org base-directory layout: `$XDG_CONFIG_HOME/<name>`,
 falling back to `~/.config/<name>`.
 
 @param name - App name including any suffix.
 
 @returns Absolute config directory for a Linux or BSD user.
 
 @example
 ```ts
 xdgConfigDirectory('foo-nodejs'); // => '/home/me/.config/foo-nodejs'
 ```
 */
function xdgConfigDirectory(name: string,): string {
  /**
   XDG config root shared by freedesktop.org platforms.
   */
  const configRoot = process.env
    .XDG_CONFIG_HOME
    ?? path.join(
      homedir(),
      '.config',
    );
  return path.join(
    configRoot,
    name,
  );
}

//endregion Platform layouts

//region Resolution

/**
 Resolves the per-user config directory for one app name,
 with the same suffix handling and platform layouts as upstream `conf`'s
 `env-paths` dependency.
 
 @param projectName - App name, typically package.json's `name`.
 
 @param projectSuffix - Suffix appended as `-<suffix>` to avoid clashing with
 native apps; `''` appends nothing.
 
 @returns Absolute path of the per-user config directory.
 
 @example
 ```ts
 configDirectory({
   projectName: 'foo',
   projectSuffix: 'nodejs',
 }); // => '/home/me/.config/foo-nodejs'
 ```
 */
export function configDirectory({
  projectName,
  projectSuffix,
}: {
  readonly projectName: string;
  readonly projectSuffix: string;
},): string {
  /**
   App directory name after suffixing;
   `''` suffix leaves the bare project name.
   */
  const suffixedName = projectSuffix === '' ? projectName : `${projectName}-${projectSuffix}`;
  if (process.platform === 'darwin')
    return macosConfigDirectory(suffixedName,);
  if (process.platform === 'win32')
    return windowsConfigDirectory(suffixedName,);
  return xdgConfigDirectory(suffixedName,);
}

//endregion Resolution
