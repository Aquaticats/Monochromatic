/**
 New-instance launcher: given the focused window's identity, decide the command
 that starts another independent instance of that app and run it.
 
 The generic path is `kstart --application <desktopFileName>`, but that D-Bus
 activates apps declaring `DBusActivatable`, which for a single-instance app
 just opens a window in the existing process. Apps needing different behavior
 are listed in {@link NEW_WINDOW_OVERRIDES}, keyed by lowercased resource
 class:
   - Browsers only re-focus on relaunch, so they need an explicit `--new-window`.
   - Ghostty must be a fresh PROCESS, not a window in the running
     single-instance server, so it is launched with `--gtk-single-instance=false`
     (the CLI flag overrides whatever the running ghostty was started with).
 
 @module
 */

import { execFile } from 'node:child_process';
import { once } from 'node:events';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { NoAppIdentityError } from './errors.ts';

/**
 Lowercased resource class to the exact command (and args) that launches a new,
 independent instance. First element is the executable, the rest are its args.
 
 @example
 ```ts
 NEW_WINDOW_OVERRIDES.firefox // ['firefox', '--new-window']
 ```
 */
export const NEW_WINDOW_OVERRIDES: Record<string, readonly [
  string,
  ...string[]
]> = {
  'com.mitchellh.ghostty': [
    'ghostty',
    '--gtk-single-instance=false'
  ],
  ghostty: [
    'ghostty',
    '--gtk-single-instance=false'
  ],
  librewolf: [
    'librewolf',
    '--new-window'
  ],
  firefox: [
    'firefox',
    '--new-window'
  ],
  'firefox-esr': [
    'firefox-esr',
    '--new-window'
  ],
  chrome: [
    'google-chrome',
    '--new-window'
  ],
  'google-chrome': [
    'google-chrome',
    '--new-window'
  ],
  chromium: [
    'chromium',
    '--new-window'
  ],
  'chromium-browser': [
    'chromium',
    '--new-window'
  ],
  brave: [
    'brave-browser',
    '--new-window'
  ],
  'brave-browser': [
    'brave-browser',
    '--new-window'
  ],
  vivaldi: [
    'vivaldi',
    '--new-window'
  ],
  'vivaldi-stable': [
    'vivaldi',
    '--new-window'
  ],
  opera: [
    'opera',
    '--new-window'
  ],
  edge: [
    'microsoft-edge',
    '--new-window'
  ],
  'microsoft-edge': [
    'microsoft-edge',
    '--new-window'
  ],
};

/**
 A resolved launch command.
 
 @example
 ```ts
 const chosen: LaunchCommand = { cmd: 'kstart', args: ['--application', 'org.kde.konsole'] };
 ```
 */
export type LaunchCommand = {
  /**
   Executable to run.
   */
  readonly cmd: string;
  /**
   Arguments passed to the executable.
   */
  readonly args: readonly string[];
};

/**
 Choose the command that launches a new instance of the focused app, preferring
 an explicit override, then the desktop file, then the bare resource class.
 
 @param desktopFileName - Focused window's `desktopFileName`, may be empty
 
 @param resourceClass - Focused window's `resourceClass`, may be empty
 
 @returns Command that launches a new instance
 
 @throws {@link NoAppIdentityError} when neither a desktop file nor a class was provided
 
 @example
 ```ts
 selectLaunchCommand({ desktopFileName: '', resourceClass: 'ghostty' });
 ```
 */
export function selectLaunchCommand({
  desktopFileName,
  resourceClass
}: {
  readonly desktopFileName: string;
  readonly resourceClass: string;
}): LaunchCommand {
  /**
   Override lookup key.
   */
  const cls = resourceClass.toLowerCase();
  /**
   Explicit command for apps that need special new-instance handling.
   */
  const override = NEW_WINDOW_OVERRIDES[cls];
  if (override) {
    return {
      cmd: override[0],
      args: override.slice(1)
    };
  }
  if (desktopFileName) {
    return {
      cmd: 'kstart',
      args: [
        '--application',
        desktopFileName
      ]
    };
  }
  if (resourceClass) {
    return {
      cmd: 'kstart',
      args: [resourceClass]
    };
  }
  throw new NoAppIdentityError();
}

/**
 Launch a new instance of the app identified by the focused window, logging
 (never throwing) so a failed launch cannot crash the daemon.
 
 @param desktopFileName - Focused window's `desktopFileName`, may be empty
 
 @param resourceClass - Focused window's `resourceClass`, may be empty
 
 @example
 ```ts
 launchNewInstance({ desktopFileName: '', resourceClass: 'ghostty' });
 ```
 */
export async function launchNewInstance({
  desktopFileName,
  resourceClass
}: {
  readonly desktopFileName: string;
  readonly resourceClass: string;
}): Promise<void> {
  try {
    /**
     Resolved command; throws when no app identity was provided.
     */
    const chosen = selectLaunchCommand({
      desktopFileName,
      resourceClass
    });
    console.log(`[key-helper] launching new instance: ${chosen.cmd} ${chosen.args
      .join(' ')}`);
    /**
     Spawned process, awaited via its `close` event.
     */
    const child = execFile(
      chosen.cmd,
      [...chosen.args]
    );
    await once(
      child,
      'close'
    );
  } catch (error) {
    /**
     Best-effort message extracted from a thrown value of unknown type.
     */
    const message = caughtValueText(error,);
    console.error(`[key-helper] launch failed: ${message}`);
  }
}
