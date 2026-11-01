/**
 Disposable scratch copy of the files pnpm needs to resolve a workspace:
 root manifests, lockfile, pnpmfile hooks, and every project `package.json`.

 A copy (not a git worktree) keeps uncommitted manifest edits and lets the
 loose-mode resolution rewrite files freely without touching the real
 checkout. Resolution with `--lockfile-only` reads nothing else.

 @module
 */

import {
  copyFile,
  mkdir,
  mkdtemp,
  readdir,
  rm,
} from 'node:fs/promises';
import {
  dirname,
  join,
  relative,
  sep,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger for scratch workspace lifecycle.
 */
const l = tagged({ tag: 'deps-update/scratch', },);

//region Types

/**
 Scratch workspace that removes itself when its `await using` scope ends.
 */
export type ScratchWorkspace = AsyncDisposable & {
  /**
   Absolute path to the scratch workspace root.
   */
  readonly dir: string;
};

//endregion Types

//region Errors

/**
 Thrown when a project directory lies outside the workspace root.
 */
export class ProjectOutsideRootError extends Error {
  /**
   Builds the error naming both paths.

   @param root - workspace root

   @param projectDir - offending project directory

   @example
   ```ts
   throw new ProjectOutsideRootError({ root: '/repo', projectDir: '/elsewhere' });
   ```
   */
  constructor({
    root,
    projectDir,
  }: {
    readonly root: string;
    readonly projectDir: string;
  },) {
    super(`Workspace project ${projectDir} is outside workspace root ${root}`,);
    this.name = 'ProjectOutsideRootError';
  }
}

//endregion Errors

//region Copy

/**
 Root files pnpm reads during resolution, besides pnpmfile variants.
 */
const ROOT_FILES = [
  'package.json',
  'pnpm-workspace.yaml',
  'pnpm-lock.yaml',
  '.npmrc',
] as const;

/**
 Filename prefix shared by `.pnpmfile.cjs`, `.pnpmfile.mjs`, and their data files.
 */
const PNPMFILE_PREFIX = '.pnpmfile';

/**
 Copies one file, skipping it when the source does not exist.

 @param from - source path

 @param to - destination path; parent directories are created

 @returns whether the file existed and was copied

 @example
 ```ts
 await copyIfPresent({ from: '/repo/.npmrc', to: '/scratch/.npmrc' });
 ```
 */
async function copyIfPresent({
  from,
  to,
}: {
  readonly from: string;
  readonly to: string;
},): Promise<boolean> {
  await mkdir(
    dirname(to,),
    { recursive: true, },
  );
  try {
    await copyFile(
      from,
      to,
    );
    return true;
  }
  catch (error) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT')) {
      l.debug(`skip absent ${from}`,);
      return false;
    }
    throw error;
  }
}

/**
 Creates a scratch workspace mirroring the resolution inputs of `root`.

 @param root - absolute workspace root

 @param projectDirs - absolute directories of every workspace project, root included or not

 @returns disposable scratch workspace; disposal deletes it

 @throws ProjectOutsideRootError when a project directory is not under `root`

 @example
 ```ts
 await using scratch = await createScratchWorkspace({ root: '/repo', projectDirs: ['/repo/package/a/b'] });
 ```
 */
export async function createScratchWorkspace({
  root,
  projectDirs,
}: {
  readonly root: string;
  readonly projectDirs: readonly string[];
},): Promise<ScratchWorkspace> {
  /**
   Logger tagged with this function.
   */
  const cl = tagged({
    tag: createScratchWorkspace.name,
    l,
  },);
  /**
   Gitignored per-tool scratch parent inside the workspace being resolved, never the system temporary directory
   whose backing device is unknowable.
   */
  const scratchParent = join(
    root,
    'node_modules',
    '.monochromatic',
    'deps-update',
  );
  await mkdir(
    scratchParent,
    { recursive: true, },
  );
  /**
   Fresh private directory for this run.
   */
  const dir = await mkdtemp(join(
    scratchParent,
    'deps-update-',
  ),);
  cl.debug(`scratch workspace ${dir}`,);
  /**
   Disposal handle, created before copying so a failed copy still cleans up.
   */
  const scratch: ScratchWorkspace = {
    dir,
    [Symbol.asyncDispose]: async function removeScratch(): Promise<void> {
      cl.debug(`removing ${dir}`,);
      await rm(
        dir,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
  /**
   Removes the scratch directory if copying throws; `move()` disarms it on success.
   */
  await using cleanupOnFailure = new AsyncDisposableStack();
  cleanupOnFailure.use(scratch,);
  /**
   Root-level pnpmfile variants and their companion data files.
   */
  const pnpmfiles = (await readdir(root,)).filter(function isPnpmfile(entry,): boolean {
    return entry.startsWith(PNPMFILE_PREFIX,);
  },);
  /**
   Project manifest paths relative to `root`.
   */
  const manifests = projectDirs.map(function toManifest(projectDir,): string {
    /**
     Project location relative to root; empty for the root itself.
     */
    const rel = relative(
      root,
      projectDir,
    );
    if ((rel === '..') || rel.startsWith(`..${sep}`,))
      throw new ProjectOutsideRootError({
        root,
        projectDir,
      },);
    return join(
      rel,
      'package.json',
    );
  },);
  /**
   Every relative path to mirror, deduplicated (root manifest appears twice).
   */
  const paths = [...new Set([
    ...ROOT_FILES,
    ...pnpmfiles,
    ...manifests,
  ],),];
  await Promise.all(paths.map(async function mirror(path,): Promise<void> {
    await copyIfPresent({
      from: join(
        root,
        path,
      ),
      to: join(
        dir,
        path,
      ),
    },);
  },),);
  cl.debug(`copied up to ${String(paths.length,)} files`,);
  cleanupOnFailure.move();
  return scratch;
}

//endregion Copy
