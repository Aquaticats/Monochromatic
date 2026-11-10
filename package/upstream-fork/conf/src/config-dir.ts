/**
 Per-user config directory resolution.
 
 Replaces upstream `conf`'s `env-paths` dependency with the same three
 platform layouts upstream resolves,
 so a migrating app finds its config file at the identical path. Resolution
 happens per call (not module load) so injected homes take effect,
 which the repository's disposable-home test rule requires.
 The root variables treat an empty value as absent,
 as upstream's `env-paths` does with
 `env.APPDATA || path.join(homedir, 'AppData', 'Roaming')`
 (that `path` is upstream's default `node:path` namespace, not this module's);
 they are spelled as explicit empty-and-unset checks because this repository's
 lint rules refuse a logical OR as a nullish fallback.
 A nullish fallback would keep `''` and make the joined path relative to the
 working directory,
 so a caller whose unit exports `APPDATA=` or `XDG_CONFIG_HOME=` would read a
 different file than upstream `conf` does.
 
 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
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
  const library = join(
    homedir(),
    'Library',
  );
  return join(
    library,
    'Preferences',
    name,
  );
}

/**
 Windows layout: `%APPDATA%\<name>\Config`,
 falling back to `AppData\Roaming`,
 including when `%APPDATA%` is set to an empty string.
 
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
  const configuredAppData = process.env
    .APPDATA;
  /**
   Roaming root to join beneath;
   an empty `%APPDATA%` counts as unset, as it does for upstream `env-paths`,
   whose fallback is a logical OR.
   */
  const appData = (configuredAppData === undefined) || (configuredAppData === '')
    ? join(
      homedir(),
      'AppData',
      'Roaming',
    )
    : configuredAppData;
  return join(
    appData,
    name,
    'Config',
  );
}

/**
 freedesktop.org base-directory layout: `$XDG_CONFIG_HOME/<name>`,
 falling back to `~/.config/<name>`,
 including when `$XDG_CONFIG_HOME` is set to an empty string.
 
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
  const configuredConfigRoot = process.env
    .XDG_CONFIG_HOME;
  /**
   XDG config root to join beneath;
   an empty `$XDG_CONFIG_HOME` counts as unset, as it does for upstream
   `env-paths`, whose fallback is a logical OR.
   */
  const configRoot = (configuredConfigRoot === undefined) || (configuredConfigRoot === '')
    ? join(
      homedir(),
      '.config',
    )
    : configuredConfigRoot;
  return join(
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
