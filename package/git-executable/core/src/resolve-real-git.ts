import { constants, } from 'node:fs';
import { access, } from 'node:fs/promises';
import {
  delimiter,
  resolve,
  win32,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { RealGitNotFoundError, } from './error.ts';
import { commonGitPathsForPlatform, } from './platform-paths.ts';
import { resolveCachedRealGit, } from './resolution-cache.ts';
import {
  GitCandidateFileTypeError,
  GitCandidateInspectionLimitError,
  isGitPolicySelfShim,
} from './self-shim.ts';
import type { ResolveRealGitOptions, } from './types.ts';

//region Resolver state and candidate types

/**
 Logger root for real-Git executable resolution.
 */
const moduleLogger = tagged({ tag: 'git-executable', },);

/**
 Default Windows executable extensions in shell lookup order.
 */
const DEFAULT_WINDOWS_PATH_EXTENSIONS = '.COM;.EXE;.BAT;.CMD';

/**
 Inputs for constructing ordered executable candidate sequence.
 */
type BuildCandidateSequenceOptions = {
  /**
   PATH-like directory sequence.
   */
  readonly pathEnv: string;
  /**
   Runtime platform controlling executable names and path identity.
   */
  readonly platform: NodeJS.Platform;
  /**
   Windows executable extensions in shell lookup order.
   */
  readonly pathExtensions: string;
  /**
   Working directory used for relative PATH entries.
   */
  readonly cwd: string;
  /**
   Common paths promoted when PATH exposes them.
   */
  readonly commonGitPaths: readonly string[];
};

//endregion Resolver state and candidate types

//region Candidate construction

/**
 Produces platform-specific Git executable names in shell lookup order.
 
 @param platform - Runtime platform selecting bare name or PATHEXT variants.
 
 @param pathExtensions - Windows executable extensions in lookup order.
 
 @returns Candidate basenames in lookup order.
 
 @example
 ```ts
 executableNamesForPlatform({ platform: 'linux', pathExtensions: '' });
 // => ['git']
 ```
 */
function executableNamesForPlatform({
  platform,
  pathExtensions,
}: {
  readonly platform: NodeJS.Platform;
  readonly pathExtensions: string;
},): readonly string[] {
  if (platform !== 'win32')
    return ['git',];

  return pathExtensions
    .split(';',)
    .filter(function nonemptyExtension(extension,) {
      return extension.length > 0;
    },)
    .map(function gitExecutableName(extension,) {
      return `git${extension.startsWith('.') ? extension : `.${extension}`}`;
    },);
}

/**
 Produces platform-comparable path identity.
 
 @param candidatePath - Absolute candidate path.
 
 @param platform - Runtime platform controlling path case sensitivity.
 
 @returns Identity suitable for candidate lookup and deduplication.
 
 @example
 ```ts
 candidateIdentity({ candidatePath: 'C:\\Git\\git.EXE', platform: 'win32' });
 // => 'c:\\git\\git.exe'
 ```
 */
function candidateIdentity({
  candidatePath,
  platform,
}: {
  readonly candidatePath: string;
  readonly platform: NodeJS.Platform;
},): string {
  return platform === 'win32'
    ? candidatePath.toLowerCase()
    : candidatePath;
}

/**
 Constructs canonical common-path-first executable candidate sequence.
 
 Every PATH entry becomes an absolute candidate before cache identity is built.
 Preferred common paths are promoted only when PATH exposes matching candidates.
 
 @param pathEnv - PATH-like directory sequence.
 
 @param platform - Runtime platform controlling executable names and path identity.
 
 @param pathExtensions - Windows executable extensions in lookup order.
 
 @param cwd - Working directory used for relative PATH entries.
 
 @param commonGitPaths - Common paths promoted when PATH exposes them.
 
 @returns Deduplicated absolute candidates in resolution order.
 
 @example
 ```ts
 buildCandidateSequence({
   pathEnv: '/custom/bin:/usr/bin',
   platform: 'linux',
   pathExtensions: '',
   cwd: '/workspace',
   commonGitPaths: ['/usr/bin/git'],
 });
 // => ['/usr/bin/git', '/custom/bin/git']
 ```
 */
function buildCandidateSequence({
  pathEnv,
  platform,
  pathExtensions,
  cwd,
  commonGitPaths,
}: BuildCandidateSequenceOptions,): readonly string[] {
  /**
   Platform-specific executable basenames in shell lookup order.
   */
  const executableNames = executableNamesForPlatform({
    platform,
    pathExtensions,
  },);
  /**
   Platform PATH delimiter,
   injectable through platform while filesystem resolution remains host-native.
   */
  const pathDelimiter = platform === 'win32'
    ? win32.delimiter
    : delimiter;
  /**
   Absolute executable candidates derived from PATH directory order.
   */
  const pathCandidates = pathEnv
    .split(pathDelimiter,)
    .flatMap(function candidatesInDirectory(dir,) {
      return executableNames.map(function executableInDirectory(name,) {
        return resolve(
          cwd,
          dir,
          name,
        );
      },);
    },);
  /**
   Exact PATH candidate spelling indexed by platform-comparable identity.
   */
  const pathCandidateByIdentity = new Map(pathCandidates.map(function indexPathCandidate(
    pathCandidate,
  ) {
    return [
      candidateIdentity({
        candidatePath: pathCandidate,
        platform,
      },),
      pathCandidate,
    ];
  },),);
  /**
   Common candidates represented by exact spelling exposed through PATH.
   */
  const exposedCommonGitPaths = commonGitPaths.flatMap(function findExposedCommonPath(
    commonGitPath,
  ) {
    /**
     PATH candidate corresponding to current common path.
     */
    const exposedPath = pathCandidateByIdentity.get(candidateIdentity({
      candidatePath: commonGitPath,
      platform,
    },),);
    return exposedPath === undefined
      ? []
      : [exposedPath,];
  },);

  return [...new Set([
    ...exposedCommonGitPaths,
    ...pathCandidates,
  ],),];
}

//endregion Candidate construction

//region Candidate scanning and cache

/**
 Narrows caught value to Node filesystem error shape.
 
 @param error - Caught value from candidate access or inspection.
 
 @returns Whether value carries Node error code.
 
 @example
 ```ts
 isErrnoException({ code: 'ENOENT' });
 // => true
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error);
}

/**
 Reports expected candidate-unavailable filesystem outcomes.
 
 @param error - Caught candidate failure.
 
 @returns Whether failure means candidate is absent,
 unreachable through a non-directory,
 or unreachable because parent path is not a directory.
 
 @example
 ```ts
 isExpectedCandidateMiss({ code: 'ENOENT' });
 // => true
 ```
 */
function isExpectedCandidateMiss(error: unknown,): boolean {
  if ((error instanceof GitCandidateFileTypeError)
    || (error instanceof GitCandidateInspectionLimitError))
    return true;
  if (!isErrnoException(error,))
    return false;
  return (error.code === 'ENOENT')
    || (error.code === 'ENOTDIR');
}

/**
 Scans ordered candidates and returns first executable outside Git policy wrapper.
 
 @param candidates - Absolute candidates in common-path-first order.
 
 @returns First usable real-Git executable path.
 
 @throws {@link RealGitNotFoundError} when every candidate is unavailable or self-referential.
 
 @example
 ```ts
 await scanCandidateSequence(['/usr/bin/git']);
 ```
 */
async function scanCandidateSequence(candidates: readonly string[],): Promise<string> {
  /**
   Function-tagged logger for candidate scan decisions.
   */
  const rl = tagged({
    tag: scanCandidateSequence.name,
    l: moduleLogger,
  },);
  /**
   Self-referential wrappers rejected before selected executable.
   */
  const skippedSelfShimPaths: string[] = [];

  for (const candidate of candidates) {
    try {
      // oxlint-disable-next-line no-await-in-loop -- first usable preferred candidate ends lookup
      await access(
        candidate,
        constants.X_OK,
      );
      // oxlint-disable-next-line no-await-in-loop -- self-shim classification must preserve candidate priority
      if (await isGitPolicySelfShim(candidate,)) {
        skippedSelfShimPaths.push(candidate,);
        continue;
      }

      rl.debug(
        `resolved real Git at ${candidate} using common-platform-path priority; `
          + `self shims skipped: ${String(skippedSelfShimPaths.length,)}`,
      );
      return candidate;
    }
    catch (error: unknown) {
      if (!isExpectedCandidateMiss(error,)) {
        rl.debug(
          `Git candidate inspection failed at ${candidate}: ${String(error,)}`,
        );
      }
    }
  }

  rl.debug(
    `real Git resolution exhausted PATH candidates; self shims skipped: ${String(skippedSelfShimPaths.length,)}`,
  );
  throw new RealGitNotFoundError({
    candidateCount: candidates.length,
    skippedSelfShimCount: skippedSelfShimPaths.length,
  },);
}

/**
 Resolves native Git executable without re-entering workspace Git policy wrapper.
 
 Common platform locations win only when PATH exposes them.
 Remaining candidates preserve PATH and Windows PATHEXT order.
 Successful equal calls share process-lifetime result;
 rejected lookups are removed so later calls retry.
 
 @param options - Optional environment and disposable-fixture lookup inputs.
 
 @returns Absolute path to selected real-Git executable.
 
 @throws {@link RealGitNotFoundError} when no usable candidate is exposed.
 
 @example
 ```ts
 const gitPath = await resolveRealGit();
 // => '/usr/bin/git'
 ```
 */
export async function resolveRealGit(options: ResolveRealGitOptions = {},): Promise<string> {
  /**
   Runtime platform controlling executable and path semantics.
   */
  const platform = options.platform ?? process.platform;
  /**
   Environment supplying process defaults and platform installation roots.
   */
  const environment = options.environment ?? process.env;
  /**
   Effective PATH-like directory sequence.
   */
  const pathEnv = options
    .pathEnv
    ?? environment
      .PATH
    ?? '';
  /**
   Effective Windows executable extension order.
   */
  const pathExtensions = options.pathExtensions
    ?? environment.PATHEXT
    ?? DEFAULT_WINDOWS_PATH_EXTENSIONS;
  /**
   Working directory used to absolutize PATH entries.
   */
  const cwd = options.cwd ?? process.cwd();
  /**
   Preferred common paths for canonical common-platform-path priority.
   */
  const commonGitPaths = options.commonGitPaths ?? commonGitPathsForPlatform({
    platform,
    environment,
  },);
  /**
   Effective absolute candidate sequence and memoization identity source.
   */
  const candidates = buildCandidateSequence({
    pathEnv,
    platform,
    pathExtensions,
    cwd,
    commonGitPaths,
  },);
  /**
   Stable identity preserving effective candidate order and spelling.
   */
  const cacheKey = JSON.stringify(candidates,);
  return await resolveCachedRealGit({
    cacheKey,
    resolve: async function scanEffectiveCandidates(): Promise<string> {
      return await scanCandidateSequence(candidates,);
    },
  },);
}

//endregion Candidate scanning and cache
