/**
 Where a tool call's path really points: every `..` segment applied to the folder above it, and every symbolic link followed, in
 the order the operating system reads them.

 A path a tool call names need not exist yet: `write` makes the file, and an `edit` of a missing file is refused by the tool
 itself. So the deepest folder that does exist is resolved through the file system, and the names below it are appended as they
 were given, which is what the operating system would do: a name under a folder that is not there cannot be a symbolic link.

 The order matters and is the reason `path.resolve` is not used on the whole path: `link/../x` is the folder above whatever
 `link` names, which `path.resolve` would read as this folder's own `x`.

 @module
 */

import { realpath, } from 'node:fs/promises';
import {
  dirname,
  isAbsolute,
  join,
  parse,
  relative,
  sep,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

//region Which answers mean a path is not there

/**
 The codes the operating system gives a path whose folder, or whose own name, is not there.
 */
const MISSING_PATH_CODES: ReadonlySet<string> = new Set([
  'ENOENT',
  'ENOTDIR',
],);

//endregion Which answers mean a path is not there

//region Resolution

/**
 Resolves one path as the operating system would, following symbolic links and applying `..` to the folder above whatever the
 segments before it resolved to.

 @param path - the path to resolve, absolute or relative to `base`

 @param base - the folder a relative path is read from, which is the session's working folder

 @returns the absolute path with every symbolic link followed and every `..` applied

 @example
 ```ts
 const real = await resolveWithSymbolicLinks({ path: 'link/../x', base: '/repo/pi-a', });
 ```
 */
export async function resolveWithSymbolicLinks({
  path,
  base,
}: {
  readonly path: string;
  readonly base?: string;
},): Promise<string> {
  /**
   The folder a relative path is read from, which is this process's own when no caller names one.
   */
  const baseFolder = base ?? process.cwd();
  /**
   The path as the shell would see it before any link is followed: a relative path built onto its folder.
   */
  const whole = isAbsolute(path,) ? path : `${baseFolder}${sep}${path}`;
  /**
   The root every absolute path starts at, which is `/` on this machine and a drive on Windows.
   */
  const { root, } = parse(whole,);
  /**
   The segments to walk, from the root down, with the doubled and empty ones skipped as the file system skips them.
   */
  const segments = whole.slice(root.length,)
    .split(sep,);
  /**
   Where the walk has arrived: the root, with each segment applied in turn.
   */
  let resolved = root;
  /* oxlint-disable eslint/no-await-in-loop -- each segment is resolved against what the segment before it resolved to, so one
     symbolic link decides what the next segment means; see doc/lint/sequential-awaits.md. */
  for (const segment of segments) {
    if ((segment === '') || (segment === '.')) {
      continue;
    }
    if (segment === '..') {
      resolved = dirname(resolved,);
      continue;
    }
    resolved = await resolvedOrItself(join(
      resolved,
      segment,
    ),);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return resolved;
}

/**
 Reads a path through the file system, answering the path itself when nothing is there.

 @param path - the path to read

 @returns what the link or the file system says this path is, or the path when it is not there

 @throws when the file system refuses the read for a reason other than the path being absent, which no answer of this module may
 guess at

 @example
 ```ts
 const real = await resolvedOrItself('/repo/pi-a/link',);
 ```
 */
async function resolvedOrItself(path: string,): Promise<string> {
  try {
    return await realpath(path,);
  }
  catch (error) {
    /**
     What the operating system answered.
     */
    const code = errorCodeOf(error,);
    if (MISSING_PATH_CODES.has(code,)) {
      return path;
    }
    throw error;
  }
}

//endregion Resolution

//region Containment

/**
 Says whether a path lies inside a folder, which it does not when it is the folder itself.

 @param folder - the folder a path would lie in

 @param path - the path to place, already resolved the same way as the folder

 @returns whether the path is below the folder

 @example
 ```ts
 const inside = isInside({ folder: '/repo', path: '/repo/package/a.ts', });
 ```
 */
export function isInside({
  folder,
  path,
}: {
  readonly folder: string;
  readonly path: string;
},): boolean {
  /**
   Where the path stands with respect to the folder.
   */
  const where = relative(
    folder,
    path,
  );
  return (where !== '')
    && (where !== '..')
    && (!where.startsWith(`..${sep}`,))
    && (!isAbsolute(where,));
}

//endregion Containment
