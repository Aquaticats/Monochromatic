import type { Dirent, } from 'node:fs';
import {
  readdir,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import {
  UNMEASURED,
  type Measured,
} from './measure.ts';
import { spawnResult, } from './spawn.ts';

/**
 Reads a directory's entries, reporting any read failure as
 {@link UNMEASURED} so the caller distinguishes a genuinely empty directory
 (zero entries) from one it could not enumerate.
 
 @param dir - directory to read
 
 @returns directory entries, or {@link UNMEASURED} when the read failed
 
 @example
 ```ts
 const entries = await readEntries({ dir: '/tmp/clone/objects' });
 ```
 */
async function readEntries({ dir, }: { readonly dir: string; },): Promise<readonly Dirent[] | typeof UNMEASURED> {
  /**
   Tagged logger naming directory read attempts.
   */
  const rl = tagged({
    tag: readEntries.name,
    l: logger,
  },);
  try {
    return await readdir(
      dir,
      { withFileTypes: true, },
    );
  }
  catch (error: unknown) {
    rl.debug(`directory read failed: ${String(error,)}`,);
    return UNMEASURED;
  }
}

/**
 Recursively sums the byte size of every regular file under a directory,
 using an explicit work-stack rather than recursion. A genuinely empty
 directory sums to 0; a root that cannot be enumerated yields
 {@link UNMEASURED} so an unreadable store is never mistaken for an empty one.
 A subdirectory that vanishes mid-walk (a concurrently-changing store) is
 skipped rather than failing the whole measurement.
 
 @param path - directory to measure
 
 @returns total bytes of all regular files beneath `path`, or
   {@link UNMEASURED} when the root could not be read
 
 @example
 ```ts
 const bytes = await dirSize({ path: '/tmp/clone/objects' });
 ```
 */
export async function dirSize({ path, }: { readonly path: string; },): Promise<Measured> {
  /**
   Mutable accumulator; a const binding with a mutated field keeps the
   function-root-let lint satisfied.
   */
  const acc = { bytes: 0, };
  /**
   Root listing; a hard read failure here means the whole store is
   unmeasurable, as opposed to a legitimately empty store that lists as zero
   entries.
   */
  const root = await readEntries({ dir: path, },);
  if (root === UNMEASURED)
    return UNMEASURED;
  /**
   Pending directories as (dir, pre-read entries) pairs; a side-effecting
   cursor walked via `pop`, each directory read exactly once.
   */
  const stack: {
    readonly dir: string;
    readonly entries: readonly Dirent[]
  }[] = [
    {
      dir: path,
      entries: root,
    },
  ];
  /* oxlint-disable eslint/no-await-in-loop -- a directory tree is walked with an explicit work-stack; each `readdir`/`stat` depends on a directory popped from the stack, so the level-by-level descent is inherently sequential. A git object store under a temp clone holds only a handful of pack files, so the serial walk is cheap. */
  while (stack.length > 0) {
    /**
     Next directory and its already-read entries; non-nullish given the length
     guard above.
     */
    const {
      dir,
      entries,
    } = nonNullishOrThrow(stack.pop(),);
    for (const entry of entries) {
      /**
       Absolute path of this entry.
       */
      const full = join(
        dir,
        entry.name,
      );
      if (entry.isDirectory()) {
        /**
         Subdirectory listing; a transient mid-walk read failure is swallowed
         because one vanished subdirectory under a concurrently-changing store
         is not a whole-measurement failure.
         */
        const sub = await readEntries({ dir: full, },);
        if (sub !== UNMEASURED)
          stack.push({
            dir: full,
            entries: sub,
          },);
        continue;
      }
      if (entry.isFile()) {
        /**
         File metadata for the byte size.
         */
        const info = await stat(full,);
        acc.bytes += info.size;
      }
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return acc.bytes;
}

/**
 Resolves a repository's object-store directory via git, then measures it.
 Targets `objects/` specifically, not the whole git dir, so the near-constant
 hooks/config/description offset does not distort small-repo ratios.
 
 @param repoPath - repository working directory or git dir
 
 @returns total bytes of the object store, or {@link UNMEASURED} when git
   cannot resolve the store or it cannot be read
 
 @example
 ```ts
 const bytes = await objectsDirSize({ repoPath: '/tmp/clone' });
 ```
 */
export async function objectsDirSize({ repoPath, }: { readonly repoPath: string; },): Promise<Measured> {
  /**
   Absolute objects path as git resolves it, honoring bare vs non-bare and
   any `GIT_OBJECT_DIRECTORY` override.
   */
  const {
    stdout,
    exitCode,
  } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      repoPath,
      'rev-parse',
      '--git-path',
      'objects',
    ],
  },);
  if ((exitCode !== 0) || (stdout === ''))
    return UNMEASURED;
  /**
   `--git-path` yields a path relative to the repo unless absolute; join
   against the repo so a relative `objects` resolves correctly.
   */
  const objectsPath = stdout.startsWith('/',) ? stdout : join(
    repoPath,
    stdout,
  );
  return await dirSize({ path: objectsPath, },);
}
