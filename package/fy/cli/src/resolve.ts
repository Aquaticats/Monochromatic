import { stat, } from 'node:fs/promises';
import { createRequire, } from 'node:module';
import { join, } from 'node:path';

import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/module-fs-path/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Logger root for cli-fy after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'cli-fy', },);

/**
 Sentinel marking "not found here", returned by the resolution helpers below.
 
 A unique `Symbol` is the genuine-sentinel form the no-nullish-union rule allows: it lets a
 helper signal absence without a `string | undefined` return type, and callers test identity
 against it instead of a nullish check.
 */
const NOT_FOUND = Symbol('module specifier resolution candidate not found',);

/**
 Attempts to resolve a bare specifier from a given base directory.
 Returns the resolved path or {@link NOT_FOUND} if resolution fails.
 
 @param specifier - ESM import specifier (e.g. `lodash`, `@scope/pkg/sub`)
 
 @param baseDir - Directory to resolve from
 
 @returns Resolved file URL string, or {@link NOT_FOUND} on failure
 
 @example
 ```ts
 resolveFrom('lodash', '/home/user/project');
 // => 'file:///home/user/project/node_modules/lodash/index.js'
 ```
 */
function resolveFrom(
  {
    specifier,
    baseDir,
  }: {
    readonly specifier: string;
    readonly baseDir: string;
  },
): string | typeof NOT_FOUND {
  /**
   Tagged logger scoped to this function so log lines identify the call site.
   */
  const rl = tagged({
    tag: resolveFrom.name,
    l,
  },);
  rl.info(`trying base ${baseDir}`,);
  try {
    /**
     CommonJS-style `require` anchored at a synthetic file under `baseDir` so Node resolves relative to that directory.
     */
    const require = createRequire(join(
      baseDir,
      'noop.js',
    ),);
    /**
     Absolute path returned by Node's resolver; logged before return for traceability.
     */
    const resolved = require.resolve(specifier,);
    rl.info(`resolved to ${resolved}`,);
    return resolved;
  }
  catch (resolutionError: unknown) {
    rl.info(`not found in ${baseDir}: ${String(resolutionError,)}`,);
    return NOT_FOUND;
  }
}

/**
 Finds the global node_modules directory by checking common global install locations.
 Returns the path or {@link NOT_FOUND} if none found.
 
 @returns Path to global node_modules, or {@link NOT_FOUND}
 
 @example
 ```ts
 findGlobalNodeModules();
 // => '/home/user/.bun/install/global/node_modules'
 ```
 */
async function findGlobalNodeModules(): Promise<string | typeof NOT_FOUND> {
  /**
   Tagged logger scoped to this function so log lines identify the call site.
   */
  const rl = tagged({
    tag: findGlobalNodeModules.name,
    l,
  },);
  /**
   User's home directory; drives the per-user candidates below and is required for them to be meaningful.
   */
  const home = process.env
    .HOME
    ?? process
      .env
      .USERPROFILE;
  if (home === undefined) {
    rl.info('no HOME or USERPROFILE set',);
    return NOT_FOUND;
  }
  /**
   Candidate global node_modules paths, ordered by priority
   */
  const candidates = [
    join(
      home,
      '.bun',
      'install',
      'global',
      'node_modules',
    ),
    join(
      home,
      '.local',
      'lib',
      'node_modules',
    ),
    '/usr/local/lib/node_modules',
    '/usr/lib/node_modules',
  ];
  /**
   Candidate check results in the same order as {@link candidates}; all stats run concurrently while preserving priority for the final match.
   */
  const candidateResults = await Promise.all(
    candidates.map(async function inspectCandidate(candidate,): Promise<string | typeof NOT_FOUND> {
      try {
        /**
         Size of the candidate's `.package-lock.json`; a non-zero size confirms a real global install lives at this path.
         */
        const lockFileSize = (await stat(join(
          candidate,
          '.package-lock.json',
        ),))
          .size;
        if (lockFileSize
          > 0)
          return candidate;
      }
      catch (statError: unknown) {
        rl.debug(`global node_modules candidate rejected: ${candidate}: ${String(statError,)}`,);
      }
      return NOT_FOUND;
    },),
  );
  /**
   First candidate that had a non-empty package-lock marker.
   */
  const globalNodeModules = candidateResults.find(function isFound(candidateResult,) {
    return candidateResult !== NOT_FOUND;
  },);
  if (globalNodeModules !== undefined) {
    rl.info(`found global node_modules at ${globalNodeModules}`,);
    return globalNodeModules;
  }

  rl.info('no global node_modules found',);
  return NOT_FOUND;
}

/**
 Wraps {@link findRootCached} to swallow discovery errors and emit a single
 diagnostic log instead. Returns the cached root or {@link NOT_FOUND} when the helper throws
 (i.e. when no `mise.toml` with `[monorepo]` exists above CWD).
 
 @returns Cached monorepo root, or {@link NOT_FOUND} outside a monorepo
 
 @example
 ```ts
 const root = await tryFindMiseMonorepoRoot();
 // => '/home/user/Monochromatic' (or NOT_FOUND outside a workspace)
 ```
 */
async function tryFindMiseMonorepoRoot(): Promise<string | typeof NOT_FOUND> {
  /**
   Tagged logger scoped to this helper so the "no monorepo root" log identifies the call site.
   */
  const rl = tagged({
    tag: tryFindMiseMonorepoRoot.name,
    l,
  },);
  try {
    return await findRootCached({ marker: MISE_MONOREPO, },);
  }
  catch (discoveryError: unknown) {
    rl.info(`no monorepo root found: ${String(discoveryError,)}`,);
    return NOT_FOUND;
  }
}

/**
 Resolves an ESM specifier by searching CWD node_modules, monorepo root node_modules,
 and global node_modules in that order.
 
 @param specifier - Bare import specifier to resolve
 
 @returns Resolved file path
 
 @throws When the specifier cannot be resolved from any location
 
 @example
 ```ts
 await resolveSpecifier({ specifier: 'lodash' });
 // => '/home/user/project/node_modules/lodash/lodash.js'
 ```
 */
export async function resolveSpecifier(
  { specifier, }: { readonly specifier: string; },
): Promise<string> {
  /**
   Tagged logger scoped to this function so log lines identify the call site.
   */
  const rl = tagged({
    tag: resolveSpecifier.name,
    l,
  },);
  /**
   Process working directory; first resolution candidate, also reused in the not-found error message.
   */
  const cwd = process.cwd();

  //region CWD resolution
  rl.info(`resolving "${specifier}" from CWD: ${cwd}`,);
  /**
   Result of the CWD resolution attempt; returned eagerly when resolved so monorepo and global lookups are skipped.
   */
  const fromCwd = resolveFrom({
    specifier,
    baseDir: cwd,
  },);
  if (fromCwd !== NOT_FOUND)
    return fromCwd;
  //endregion CWD resolution

  //region Monorepo root resolution
  /**
   Cached monorepo root populated by {@link findRootCached}.
   
   Stays {@link NOT_FOUND} outside a monorepo or when discovery throws; reused below to
   render the diagnostic line for the not-found error.
   */
  const monorepoRoot = await tryFindMiseMonorepoRoot();
  if ((monorepoRoot !== NOT_FOUND) && (monorepoRoot !== cwd)) {
    rl.info(`trying monorepo root: ${monorepoRoot}`,);
    /**
     Result of resolution anchored at the monorepo root; tried only when the root differs from CWD.
     */
    const fromMonorepo = resolveFrom({
      specifier,
      baseDir: monorepoRoot,
    },);
    if (fromMonorepo !== NOT_FOUND)
      return fromMonorepo;
  }
  //endregion Monorepo root resolution

  //region Global resolution
  /**
   Detected global `node_modules` path; final fallback when project-local resolution fails.
   */
  const globalDir = await findGlobalNodeModules();
  if (globalDir !== NOT_FOUND) {
    rl.info(`trying global: ${globalDir}`,);
    /**
     Result of resolution anchored one level above the global `node_modules` so Node's `require` discovers packages installed there.
     */
    const fromGlobal = resolveFrom({
      specifier,
      baseDir: join(
        globalDir,
        '..',
      ),
    },);
    if (fromGlobal !== NOT_FOUND)
      return fromGlobal;
  }
  //endregion Global resolution

  /**
   Diagnostic line included in the thrown error only when a monorepo root was discovered.
   */
  const monorepoLine = monorepoRoot !== NOT_FOUND
    ? `  - Monorepo root: ${monorepoRoot}\n`
    : '';
  /**
   Diagnostic line included in the thrown error only when a global `node_modules` was detected.
   */
  const globalLine = globalDir !== NOT_FOUND ? `  - Global: ${globalDir}\n` : '';
  throw new Error(
    `Cannot resolve "${specifier}" from any of:\n  - CWD: ${cwd}\n${monorepoLine}${globalLine}Install the package first (e.g. pnpm add <package>)`,
  );
}
