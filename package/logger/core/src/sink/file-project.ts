import type { stat, } from 'node:fs/promises';
import type {
  dirname,
  join,
} from 'node:path';
import { reportLoggerInternalError, } from '../error-format.ts';

//region Ancestor search
// Locates the project the process belongs to so log files land beside its
// `node_modules` rather than at whatever stray cwd launched the script.

/**
 Sentinel returned by {@link findNodeModulesUp} when no ancestor directory
 contains a `node_modules`. A unique symbol so it never collides with a real
 path string the walk might otherwise return, keeping the result free of a
 banned `string | undefined` union.
 
 @example
 ```ts
 const dir = await findNodeModulesUp({ cwd, stat, dirname, join });
 if (dir === NO_NODE_MODULES_FOUND) {
   // no ancestor project root
 }
 ```
 */
export const NO_NODE_MODULES_FOUND: unique symbol = Symbol('logger:no-node-modules-found',);

/**
 Walks up from `cwd` to find the nearest ancestor directory containing a
 `node_modules` subdirectory, returning that subdirectory's absolute path.
 
 Using find-up rather than cwd-relative placement keeps log directories
 anchored to the project the caller actually belongs to. Without this,
 scripts invoked from build output (e.g. `dist/`) or other stray cwds
 would create `node_modules/.monochromatic/` inside those trees, polluting
 shipped artifacts.
 
 Exported through the `./node` subpath primarily so `file.unit.test.ts`
 can exercise both the hit and miss paths directly with an injected `stat`.
 
 @param cwd - starting directory for the upward search
 
 @param stat - `node:fs/promises` stat, injected so tests drive hit,
 miss, and fault paths deterministically without touching a real tree
 
 @param dirname - `node:path` dirname
 
 @param join - `node:path` join
 
 @param reportError - logger fault reporter injected for deterministic tests
 
 @returns absolute path to the nearest ancestor `node_modules`, or
 {@link NO_NODE_MODULES_FOUND} when no ancestor contains one
 
 @example
 ```ts
 const dir = await findNodeModulesUp({ cwd: process.cwd(), stat, dirname, join });
 ```
 */
export async function findNodeModulesUp(
  {
    cwd,
    stat: statPath,
    dirname: parentOf,
    join: joinPath,
    reportError = reportLoggerInternalError,
  }: {
    readonly cwd: string;
    readonly stat: typeof stat;
    readonly dirname: typeof dirname;
    readonly join: typeof join;
    readonly reportError?: typeof reportLoggerInternalError;
  },
): Promise<string | typeof NO_NODE_MODULES_FOUND> {
  /**
   Directory being tested in this iteration; either resolves to a node_modules or triggers the walk to the parent.
   */
  const candidate = joinPath(
    cwd,
    'node_modules',
  );
  try {
    /**
     Stat result for `candidate`; only directories count as a hit, guarding against a sibling file also named `node_modules`.
     */
    const entry = await statPath(candidate,);
    if (entry.isDirectory())
      return candidate;
  }
  catch (error: unknown) {
    // Missing candidate is expected while walking ancestors; only unexpected stat failures are logger faults.
    if (!(Error.isError(error,)
      && ('code' in error)
      && (error.code === 'ENOENT'))) {
      reportError({
        context: `node_modules candidate ${candidate} unavailable during file sink search`,
        error,
      },);
    }
  }
  /**
   Parent directory used by the next recursive step; equal to `cwd` only at the filesystem root, which terminates the walk.
   */
  const parent = parentOf(cwd,);
  if (parent === cwd)
    return NO_NODE_MODULES_FOUND;
  return await findNodeModulesUp({
    cwd: parent,
    stat: statPath,
    dirname: parentOf,
    join: joinPath,
    reportError,
  },);
}
//endregion Ancestor search
