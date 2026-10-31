import { constants, } from 'node:fs';
import {
  access,
  stat,
} from 'node:fs/promises';
import {
  basename,
  delimiter,
  dirname,
  join,
  resolve,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { BypassRouteError, } from './errors.ts';

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
 Resolves bare executable through selected path text.
 
 Empty path segments are skipped so privileged discovery never treats cwd as executable source.
 
 @param command - Bare executable name.
 
 @param pathText - Delimited search path.
 
 @returns Exact executable path or absence sentinel.
 
 @example
 ```ts
 await findInPath({ command: 'wg-quicker-exempt', pathText: '/usr/bin:/bin' });
 ```
 */
function findInPath(
  {
    command,
    pathText,
  }: {
    readonly command: string;
    readonly pathText: string;
  },
): Promise<string | typeof COMMAND_NOT_FOUND> {
  /**
   Absolute candidates preserving declared search order.
   */
  const candidates = pathText
    .split(delimiter,)
    .filter(function nonemptyPath(directory,): boolean {
      return directory !== '';
    },)
    .map(function commandPath(directory,): string {
      return resolve(
        directory,
        command,
      );
    },);
  return firstExecutable({ candidates, },);
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
 Resolves repository sibling release or debug executable.
 
 @returns Exact executable path or absence sentinel.
 
 @example
 ```ts
 await findWorkspaceCompanion();
 ```
 */
async function findWorkspaceCompanion(): Promise<string | typeof COMMAND_NOT_FOUND> {
  /**
   Current TypeScript package root when running from repository layout.
   */
  const packageRoot = currentPackageRoot();
  if ((typeof packageRoot) === 'symbol')
    return COMMAND_NOT_FOUND;
  /**
   Release first,
   then debug for development before first optimized build.
   */
  const candidates = [
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
  return await firstExecutable({ candidates, },);
}

/**
 Resolves the workspace
 or installed Rust companion to exact executable path.
 
 @returns Executable path immune to sudo secure-path reset.
 
 @throws {@link BypassRouteError} when no executable candidate exists.
 
 @example
 ```ts
 await resolveApplicationExemptionCommand();
 ```
 */
export async function resolveApplicationExemptionCommand(): Promise<string> {
  /**
   Secure path and captured caller path.
   No environment variable names the executable,
   because a name that could be set before `sudo` would be a switch for the process watch.
   */
  const {
    PATH: securePathValue,
    WG_QUICKER_CALLER_PATH: callerPathValue,
  } = process.env;
  /**
   Current root secure path searched for root-owned installation.
   */
  const securePath = securePathValue ?? '';
  /**
   Caller path restored under internal name,
   never assigned to privileged PATH.
   */
  const callerPath = callerPathValue ?? '';
  /**
   Repository sibling paired with current TypeScript package.
   */
  const workspace = await findWorkspaceCompanion();
  if ((typeof workspace) === 'string')
    return workspace;
  /**
   Root-owned installation from sudo secure path.
   */
  const installed = await findInPath({
    command: DEFAULT_COMMAND,
    pathText: securePath,
  },);
  if ((typeof installed) === 'string')
    return installed;
  /**
   Caller installation captured before secure path replacement.
   */
  const callerInstalled = await findInPath({
    command: DEFAULT_COMMAND,
    pathText: callerPath,
  },);
  if ((typeof callerInstalled) === 'string')
    return callerInstalled;
  throw new BypassRouteError(
    'wg-quicker-exempt is unavailable. Build it with `mise run //package/wg/cli-quicker-exempt:build`.',
  );
}
