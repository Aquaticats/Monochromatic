import { constants, } from 'node:fs';
import {
  access,
  stat,
} from 'node:fs/promises';
import {
  basename,
  delimiter,
  dirname,
  isAbsolute,
  join,
  resolve,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger for Rust companion discovery.
 */
const l = tagged({ tag: 'application-exemption-command', },);

/**
 Default Rust companion executable name.
 */
const DEFAULT_COMMAND = 'wg-quicker-exempt';

/**
 Package-directory name containing TypeScript CLI.
 */
const CLI_QUICKER_DIRECTORY = 'cli-quicker';

/**
 Product-category directory name shared by wg packages.
 */
const WG_DIRECTORY = 'wg';

/**
 Package-directory name containing Rust companion.
 */
const CLI_QUICKER_EXEMPT_DIRECTORY = 'cli-quicker-exempt';

/**
 Sentinel for absent executable or package root.
 */
const COMMAND_NOT_FOUND: unique symbol = Symbol('wg-quicker exemption command not found',);

/**
 Reports whether candidate is executable regular file.
 
 @param path - Absolute executable candidate.
 
 @returns Whether path is regular and executable.
 
 @example
 ```ts
 await isExecutableFile({ path: '/usr/bin/true' });
 ```
 */
async function isExecutableFile({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    /**
     Metadata plus execute access checked concurrently.
     */
    const [metadata,] = await Promise.all([
      stat(path,),
      access(
        path,
        constants.X_OK,
      ),
    ],);
    return metadata.isFile();
  }
  catch (error) {
    l.debug(`candidate unavailable ${path}: ${String(error,)}`,);
    return false;
  }
}

/**
 Lists the candidates for a bare executable name along a search path.

 Only absolute directories count.
 An empty segment means the current directory and a relative one is resolved against it,
 and the current directory is not a root-controlled location.

 @param command - Bare executable name.

 @param pathText - Delimited search path.

 @returns Absolute candidates in search order.

 @example
 ```ts
 pathCandidates({ command: 'wg-quicker-exempt', pathText: '/usr/bin:/bin' });
 ```
 */
function pathCandidates(
  {
    command,
    pathText,
  }: {
    readonly command: string;
    readonly pathText: string;
  },
): readonly string[] {
  return pathText
    .split(delimiter,)
    .filter(function absolutePath(directory,): boolean {
      return isAbsolute(directory,);
    },)
    .map(function commandPath(directory,): string {
      return join(
        directory,
        command,
      );
    },);
}

/**
 Resolves first executable candidate while checking independent paths concurrently.
 
 @param candidates - Ordered absolute executable candidates.
 
 @returns First executable in source order or absence sentinel.
 
 @example
 ```ts
 await firstExecutable({ candidates: ['/usr/bin/true'] });
 ```
 */
async function firstExecutable(
  { candidates, }: { readonly candidates: readonly string[]; },
): Promise<string | typeof COMMAND_NOT_FOUND> {
  /**
   Isolated array sharing only immutable primitive strings.
   */
  const isolatedCandidates = [
    ...candidates,
  ];
  /**
   Candidate or absence for every independent filesystem probe.
   */
  const results = await Promise.all(isolatedCandidates.map(async function checkCandidate(candidate,) {
    return await isExecutableFile({ path: candidate, },)
      ? candidate
      : COMMAND_NOT_FOUND;
  },),);
  /**
   First successful candidate retaining declared order.
   */
  const result = results.find(function found(value,): boolean {
    return (typeof value) === 'string';
  },);
  return result ?? COMMAND_NOT_FOUND;
}

/**
 Reports validated package root candidate.
 
 @param candidate - Directory expected to be `package/wg/cli-quicker`.
 
 @returns Candidate or absence sentinel.
 
 @example
 ```ts
 packageRootCandidate('/repo/package/wg/cli-quicker');
 ```
 */
function packageRootCandidate(
  candidate: string,
): string | typeof COMMAND_NOT_FOUND {
  if (basename(candidate,) !== CLI_QUICKER_DIRECTORY)
    return COMMAND_NOT_FOUND;
  if (basename(dirname(candidate,),) !== WG_DIRECTORY)
    return COMMAND_NOT_FOUND;
  return candidate;
}

/**
 Resolves package root from source or bundled script layout.
 
 @returns `package/wg/cli-quicker` path or absence sentinel.
 
 @example
 ```ts
 currentPackageRoot();
 ```
 */
function currentPackageRoot(): string | typeof COMMAND_NOT_FOUND {
  /**
   Module directory for source or current Rolldown entry bundle.
   */
  const scriptDirectory = import.meta.dirname;
  /**
   Source layout candidate from `src/index.ts`.
   */
  const sourceCandidate = packageRootCandidate(resolve(
    scriptDirectory,
    '..',
  ),);
  if ((typeof sourceCandidate) === 'string')
    return sourceCandidate;
  /**
   Bundle layout candidate from `dist/final/node/index.mjs`.
   */
  return packageRootCandidate(resolve(
    scriptDirectory,
    '..',
    '..',
    '..',
  ),);
}

/**
 Lists the repository sibling release and debug builds.

 @returns Release first,
 then debug for development before the first optimized build;
 empty when this code does not run from the repository layout.

 @example
 ```ts
 workspaceCandidates();
 ```
 */
function workspaceCandidates(): readonly string[] {
  /**
   Current TypeScript package root when running from repository layout.
   */
  const packageRoot = currentPackageRoot();
  if ((typeof packageRoot) === 'symbol')
    return [];
  return [
    join(
      dirname(packageRoot,),
      CLI_QUICKER_EXEMPT_DIRECTORY,
      'target',
      'release',
      DEFAULT_COMMAND,
    ),
    join(
      dirname(packageRoot,),
      CLI_QUICKER_EXEMPT_DIRECTORY,
      'target',
      'debug',
      DEFAULT_COMMAND,
    ),
  ];
}

/**
 Outcome of looking for the Rust companion.
 */
export type ApplicationExemptionCommand =
  | {
    /**
     The companion was found.
     */
    readonly found: true;

    /**
     Exact executable path.
     */
    readonly path: string;
  }
    | {
    /**
     The companion was not found.
     */
      readonly found: false;

    /**
     Every location that was looked at.
     */
      readonly searched: readonly string[];
    };

/**
 Finds the Rust companion in root-controlled locations only:
 the repository sibling release or debug build,
 then the secure path.

 The caller's own `PATH` is never searched,
 because whoever can set it before `sudo` would choose which program runs as root in place of the process watch,
 which is a switch for a policy that has none.
 Under `sudo` the `PATH` here is its `secure_path`.

 @returns The exact executable path,
 or every location searched when there is none.

 @example
 ```ts
 await resolveApplicationExemptionCommand();
 ```
 */
export async function resolveApplicationExemptionCommand(): Promise<ApplicationExemptionCommand> {
  /**
   Repository siblings paired with the current TypeScript package.
   */
  const workspace = workspaceCandidates();
  /**
   Directories of the secure path.
   */
  const installed = pathCandidates({
    command: DEFAULT_COMMAND,
    pathText: process.env
      .PATH
      ?? '',
  },);
  /**
   First executable in search order.
   */
  const found = await firstExecutable({ candidates: [
    ...workspace,
    ...installed,
  ], },);
  if ((typeof found) === 'string')
    return {
      found: true,
      path: found,
    };
  return {
    found: false,
    searched: [
      ...workspace,
      ...installed,
    ],
  };
}
