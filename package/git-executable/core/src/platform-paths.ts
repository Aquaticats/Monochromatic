import { win32, } from 'node:path';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Common platform paths

/**
 Common Unix Git locations in repository preference order.
 */
const COMMON_UNIX_GIT_PATHS: readonly string[] = [
  '/usr/bin/git',
  '/usr/local/bin/git',
];

/**
 Common macOS Git locations after standard Unix locations.
 */
const COMMON_MACOS_GIT_PATHS: readonly string[] = [
  ...COMMON_UNIX_GIT_PATHS,
  '/opt/homebrew/bin/git',
  '/opt/local/bin/git',
];

/**
 Fallback Program Files root when Windows omits environment metadata.
 */
const DEFAULT_WINDOWS_PROGRAM_FILES = String.raw`C:\Program Files`;

/**
 Returns common absolute Git paths for runtime platform.
 
 @param platform - Runtime platform selecting conventional installation locations.
 
 @param environment - Environment containing Windows installation roots.
 
 @returns Common Git executable paths in preferred lookup order.
 
 @example
 ```ts
 commonGitPathsForPlatform({ platform: 'linux' });
 // => ['/usr/bin/git', '/usr/local/bin/git']
 ```
 */
export function commonGitPathsForPlatform({
  platform,
  environment = process.env,
}: {
  readonly platform: NodeJS.Platform;
  readonly environment?: ForeignBorrowed<NodeJS.ProcessEnv>;
},): readonly string[] {
  if (platform === 'win32') {
    /**
     Program Files roots that can contain machine-wide Git installations.
     */
    const programFilesRoots = new Set([
      environment.ProgramFiles,
      environment.ProgramW6432,
      environment['ProgramFiles(x86)'],
      DEFAULT_WINDOWS_PROGRAM_FILES,
    ].filter(function definedRoot(root,): root is string {
      return root !== undefined;
    },),);
    /**
     Local application root that can contain per-user Git installation.
     */
    const localGitRoots = environment.LOCALAPPDATA === undefined
      ? []
      : [win32.join(
        environment.LOCALAPPDATA,
        'Programs',
      ),];

    return [
      ...programFilesRoots,
      ...localGitRoots,
    ].flatMap(function commonWindowsGitPaths(root,) {
      return [
        win32.join(
          root,
          'Git',
          'cmd',
          'git.exe',
        ),
        win32.join(
          root,
          'Git',
          'bin',
          'git.exe',
        ),
      ];
    },);
  }

  if (platform === 'darwin')
    return COMMON_MACOS_GIT_PATHS;

  return COMMON_UNIX_GIT_PATHS;
}

//endregion Common platform paths
