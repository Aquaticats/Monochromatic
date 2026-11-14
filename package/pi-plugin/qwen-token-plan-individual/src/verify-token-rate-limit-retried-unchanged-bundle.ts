/**
 The bundle the package produced before this translation existed, built from
 the commit that introduced it, so the proof compares two bundles built by the
 same compiler from the same pi.

 The package's source is copied out of that commit through `git archive` and
 built with this checkout's toolchain;
 the copy borrows the dependency trees the package and the repository root
 already hold,
 so every workspace package it imports resolves to this checkout's source.
 The proof therefore compares this package's own dispatch, and it assumes the
 packages beside it are the same in both runs.

 @module
 */

import {
  readFile,
  stat,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import {
  promisify,
} from 'node:util';
import { execFile, } from 'node:child_process';

//region Constants

/**
 Node's own promisify adapter, typed through its documented namespace:
 the generic `promisify(execFile)` form trips `strict-void-return`
 (`doc/troubleshooting/oxlint-promisify-void-return.md`).
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ =
  promisify;

/**
 Command runner both git and the bundler are called through.
 */
const RUN_COMMAND = promisifyExecFile(execFile,);

//endregion Constants

//region Types

/**
 What building the unchanged bundle needs from the checkout it runs in.
 */
export type UnchangedBundleRequest = {
  /**
   Folder the copy and its archive live in.
   */
  readonly root: string;
  /**
   Repository-relative path of the package directory.
   */
  readonly packagePath: string;
  /**
   Text whose introducing commit names the last unchanged package.
   */
  readonly translationMarker: string;
};

//endregion Types

//region The checkout

/**
 The checkout this proof runs inside,
 found by walking up to the directory that holds the repository's `.git`
 entry.

 The repository's own `git` wrapper refuses to run from anywhere but the
 root, and this proof starts in its package directory.

 @param start - directory to walk up from

 @returns absolute path of the repository root

 @throws naming the directory the walk ended at, which holds no `.git` entry

 @example
 ```typescript
 const repository = await repositoryRootOf({ start: process.cwd() });
 ```
 */
export async function repositoryRootOf({
  start,
}: {
  readonly start: string;
},): Promise<string> {
  try {
    await stat(join(
      start,
      '.git',
    ),);
    return start;
  } catch (error) {
    /**
     Directory above the one just read, or the same directory at the filesystem
     root.
     */
    const above = dirname(start,);
    if (above === start)
      throw new Error(
        `no .git entry was found above ${start}, so this proof cannot find the checkout it runs inside`,
        { cause: error, },
      );
    return await repositoryRootOf({ start: above, },);
  }
}

/**
 The commit whose provider dispatch is the unchanged one,
 found by naming the commit that introduced this translation and stepping one
 back from it.

 @param repository - repository root the search runs in

 @param packagePath - repository-relative path of the package directory

 @param translationMarker - text the introducing commit added

 @returns the commit the unchanged bundle is built from

 @throws naming the search, so a reader can run the proof where that history is
 present

 @example
 ```typescript
 const commit = await unchangedCommitOf({ repository, packagePath, translationMarker });
 ```
 */
async function unchangedCommitOf({
  repository,
  packagePath,
  translationMarker,
}: {
  readonly repository: string;
  readonly packagePath: string;
  readonly translationMarker: string;
},): Promise<string> {
  /**
   Commit that first carried the marker into the package's provider dispatch.
   */
  const introducedIn = (await RUN_COMMAND(
    'git',
    [
      'log',
      '--reverse',
      '--format=%H',
      '-S',
      translationMarker,
      '--',
      `${packagePath}/src/provider.ts`,
    ],
    { cwd: repository, },
  )).stdout
    .split('\n',)
    .find(function firstLine(line,) {
      return line.length > 0;
    },);
  if (introducedIn === undefined)
    throw new Error(
      `no commit introduced "${translationMarker}" into ${packagePath}/src/provider.ts, so the unchanged package cannot be named; run this proof where that history is present`,
    );
  return `${introducedIn}^`;
}

//endregion The checkout

//region Building the bundle

/**
 Build the bundle the package produced before this translation existed.

 @param request - the folder to build in and the package it builds

 @returns absolute path of the unchanged bundle

 @example
 ```typescript
 const bundle = await unchangedBundleOf({ root, packagePath, translationMarker });
 ```
 */
export async function unchangedBundleOf(request: UnchangedBundleRequest,): Promise<string> {
  /**
   Fields of the request this build reads.
   */
  const {
    root,
    packagePath,
    translationMarker,
  } = request;
  /**
   Repository root this proof runs inside.
   */
  const repository = await repositoryRootOf({ start: process.cwd(), },);
  /**
   Commit whose provider dispatch is the unchanged one.
   */
  const commit = await unchangedCommitOf({
    repository,
    packagePath,
    translationMarker,
  },);
  /**
   Folder the package's source at that commit is unpacked into.
   */
  const source = join(
    root,
    'unchanged-source',
  );
  /**
   Archive file the package directory is unpacked from.
   */
  const archivePath = join(
    root,
    'unchanged-package.tar',
  );
  await RUN_COMMAND(
    'mkdir',
    [
      '--parents',
      source,
    ],
  );
  await RUN_COMMAND(
    'git',
    [
      'archive',
      '--format=tar',
      `--output=${archivePath}`,
      commit,
      packagePath,
    ],
    { cwd: repository, },
  );
  await RUN_COMMAND(
    'tar',
    [
      '--extract',
      `--file=${archivePath}`,
      `--directory=${source}`,
    ],
  );
  /**
   Directory the unchanged bundle is built in.
   */
  const packageDirectory = join(
    source,
    packagePath,
  );
  // The copy stands outside the repository, so it borrows the dependency trees
  // the package and the repository root already hold:
  // its own peers and workspace packages, and the tsconfig chain the root
  // hoists.
  await RUN_COMMAND(
    'ln',
    [
      '--symbolic',
      join(
        repository,
        packagePath,
        'node_modules',
      ),
      join(
        packageDirectory,
        'node_modules',
      ),
    ],
  );
  await RUN_COMMAND(
    'ln',
    [
      '--symbolic',
      join(
        repository,
        'node_modules',
      ),
      join(
        source,
        'node_modules',
      ),
    ],
  );
  await RUN_COMMAND(
    'rolldown',
    [
      '--configLoader',
      'native',
      '--config',
      'rolldown.node.config.ts',
    ],
    { cwd: packageDirectory, },
  );
  /**
   Path of the bundle the copy built.
   */
  const bundle = join(
    packageDirectory,
    'dist',
    'final',
    'node',
    'index.mjs',
  );
  /**
   Built bundle, read so the price is known before a run loads it.
   */
  const built = await readFile(
    bundle,
    'utf8',
  );
  // A marker that already reached the commit this copy came from would make
  // the "unchanged" run another run of this translation,
  // and the proof would pass for a reason it does not claim.
  if (built.includes(translationMarker,))
    throw new Error(
      `the bundle built from ${commit} already carries "${translationMarker}", so it is not the package before this translation`,
    );
  return bundle;
}

//endregion Building the bundle
