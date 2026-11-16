/**
 The path of a program a build wrote, in a folder this review owns: the review's worktree or its own folder, after every symbolic
 link in the existing part of the path is resolved.

 A program from the main checkout, or from a build folder tied to no worktree, is not confirmed: nothing this check reads ties it to
 the reviewed commit. That limit is written in the package's doc.

 @module
 */

import {
  realpath,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  isAbsolute,
  join,
  resolve,
} from 'node:path';
import type { ReviewCommandRun, } from './review-statement-shape.ts';

/**
 The path segments that mark a folder a build writes into: a Cargo target directory's `target`, its `debug` or `release` output (a
 target directory at its own root), and the package's `dist` output.
 */
const BUILD_OUTPUT_SEGMENTS: ReadonlySet<string> = new Set([
  'target',
  'dist',
  'debug',
  'release',
]);

/**
 No built program at a path, as a symbol so that no text can stand for it.
 */
export const NOT_A_BUILT_PROGRAM: unique symbol = Symbol('path holds no build output segment',);

/**
 One stated command, compared with the session call it names.
 */
export type StatedCommandCheck = {
  /**
   The command the statement lists.
   */
  readonly stated: ReviewCommandRun;

  /**
   The index of the session call it matches, or nothing when the session records no such command.
   */
  readonly recordedAt?: number;
};

/**
 What comparing a statement with a session found.
 */
export type CommandComparison = {
  /**
   One line per claim that does not fit the session.
   */
  readonly problems: readonly string[];

  /**
   One line per claim the session cannot confirm either way.
   */
  readonly uncompared: readonly string[];

  /**
   The built programs the statement names and the session confirms, each once.
   */
  readonly builtPrograms: readonly string[];

  /**
   Whether a command the session records ran a build tool, in its own program position or in a script it runs.
   */
  readonly buildCommandRan: boolean;

  /**
   The commands the session records that ran a build tool, for the verdict's line.
   */
  readonly buildTaskCommands: readonly string[];
};

/**
 Reads one path as an absolute one, resolving a relative path against the folder the session started in.

 @param path - the path as written

 @param workingFolder - the folder the session started in, or empty text when it records none

 @returns the path, absolute when the folder is known

 @example
 ```ts
 const path = absolutePathIn({ path: './target/debug/wrapper', workingFolder: '/worktree' });
 ```
 */
function absolutePathIn({
  path,
  workingFolder,
}: {
  readonly path: string;
  readonly workingFolder: string;
},): string {
  if (isAbsolute(path,) || (workingFolder === '')) {
    return path;
  }
  return join(
    workingFolder,
    path,
  );
}

/**
 Reads a path with every symbolic link in its existing part resolved, so two names of one folder compare equal.

 A path whose parent is gone, as a finished review's worktree is, is resolved to its deepest existing ancestor and the rest appended.

 @param path - the path to read

 @returns the path with its existing part resolved

 @example
 ```ts
 const canonical = await canonicalPathOf({ path: '/home/user/temp/agent/a' });
 ```
 */
async function canonicalPathOf({ path, }: { readonly path: string },): Promise<string> {
  /**
   The path as written, made absolute.
   */
  const absolute = resolve(path,);
  /**
   The path with every existing ancestor's links resolved.
   */
  return await realPathOf({
    absolute,
    rest: [],
  },);
}

/**
 Resolves the deepest existing ancestor of a path and joins the rest below it.

 @param absolute - the path, made absolute

 @param rest - the segments below the ancestor this step tries

 @returns the path with every existing link resolved

 @example
 ```ts
 const real = await realPathOf({ absolute: '/w/target/debug/x', rest: [] });
 ```
 */
async function realPathOf({
  absolute,
  rest,
}: {
  readonly absolute: string;
  readonly rest: readonly string[];
},): Promise<string> {
  try {
    /**
     The ancestor with its links resolved.
     */
    const real = await realpath(absolute,);
    return (rest.length === 0) ? real : join(
      real,
      ...rest.toReversed(),
    );
  }
  catch (error) {
    /**
     The ancestor above this one.
     */
    const parent = dirname(absolute,);
    if (parent === absolute) {
      return absolute;
    }
    return await realPathOf({
      absolute: parent,
      rest: [
        ...rest,
        basename(absolute,),
      ],
    },);
  }
}

/**
 Says whether a path lies under one of the roots, after every existing link in both is resolved.

 @param path - the path to place

 @param roots - the folders a built program may lie under

 @returns whether the path is under one of them

 @example
 ```ts
 const under = await underOneOfTheRoots({ path: '/w/target/debug/x', roots: ['/w'] });
 ```
 */
async function underOneOfTheRoots({
  path,
  roots,
}: {
  readonly path: string;
  readonly roots: readonly string[];
},): Promise<boolean> {
  /**
   The path with its existing part resolved.
   */
  const canonical = await canonicalPathOf({ path, },);
  /**
   The roots that name a folder, each resolved once.
   */
  const named = roots.filter(function namesAFolder(root,) {
    return root !== '';
  },);
  /**
   The roots with their existing parts resolved, all at once.
   */
  const canonicalRoots = await Promise.all(named.map(async function resolvedRoot(root,) {
    return await canonicalPathOf({ path: root, },);
  },),);
  return canonicalRoots.some(function holdsIt(canonicalRoot,) {
    return (canonical === canonicalRoot) || canonical.startsWith(`${canonicalRoot}/`,);
  });
}

/**
 Says whether a path is a program a build wrote in a folder this review owns.

 The path must hold a build output segment and lie under the review's worktree or its own folder, after every symbolic link in the
 existing part is resolved. A program from the main checkout or from a build folder tied to no worktree is not confirmed, because
 nothing this check reads ties it to the reviewed commit.

 @param path - the path as written, absolute or relative to the session's folder

 @param workingFolder - the folder the session started in

 @param reviewFolder - the review's own folder

 @returns the absolute path, or {@link NOT_A_BUILT_PROGRAM}

 @example
 ```ts
 const built = await builtProgramPathIn({ path: '/w/package/a/target/debug/a', workingFolder: '/w', reviewFolder: '/reviews/a' });
 ```
 */
export async function builtProgramPathIn({
  path,
  workingFolder,
  reviewFolder,
}: {
  readonly path: string;
  readonly workingFolder: string;
  readonly reviewFolder: string;
},): Promise<string | typeof NOT_A_BUILT_PROGRAM> {
  /**
   The path, resolved against the session's folder when it is relative.
   */
  const absolute = absolutePathIn({
    path,
    workingFolder,
  },);
  if (!absolute.split('/',)
    .some(function isABuildSegment(segment,) {
      return BUILD_OUTPUT_SEGMENTS.has(segment,);
    },)) {
    return NOT_A_BUILT_PROGRAM;
  }
  if (!(await underOneOfTheRoots({
    path: absolute,
    roots: [
      workingFolder,
      reviewFolder,
    ],
  },))) {
    return NOT_A_BUILT_PROGRAM;
  }
  return absolute;
}
