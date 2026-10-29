/**
 Runtime image identity and build.
 
 The image bakes the whole repository with a frozen pnpm install; its
 tag derives from the lockfile and Containerfile contents so dependency
 or runtime changes rebuild while unrelated edits reuse the cache.
 
 @example
 ```ts
 const image = await ensureRuntimeImage({ repoRoot, skipImageBuild: false });
 ```
 */

import type { Dirent, } from 'node:fs';
import { createHash, } from 'node:crypto';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Module logger for host-side image management.
 */
const l = tagged({ tag: 'mutation-test', },);

/**
 Local image repository name for the mutation runtime.
 */
const IMAGE_REPOSITORY = 'localhost/mutation-test-runtime';

/**
 Containerfile path relative to the repo root.
 */
const CONTAINERFILE = 'package/mutation-test/cli/runtime/Containerfile';

/**
 Build-context ignore file path relative to the repo root.
 
 Building from the repo root without it sweeps node_modules, .git, and
 dist into the context (multi-GB layer commit, measured 36 minute
 upload before failing); the ignore list mirrors the work-tree rsync
 excludes.
 */
const CONTAINER_IGNOREFILE = 'package/mutation-test/cli/runtime/containerignore';

/**
 Tag hash length keeping references short yet distinct.
 */
const TAG_HEX_LENGTH = 12;

/**
 Recursively lists files under one directory, sorted for stable hashes.
 
 @param dir - Absolute directory.
 
 @returns Sorted absolute file paths.
 
 @example
 ```ts
 await listFilesSorted('/repo/package/mutation-test/cli/src');
 ```
 */
async function listFilesSorted(dir: string,): Promise<readonly string[]> {
  /**
   Directory entries at this level.
   */
  const entries = await readdir(
    dir,
    { withFileTypes: true, },
  );
  /**
   Files from this level and below.
   */
  const nested = await Promise.all(entries.map(
    async function collect(entry: ForeignBorrowed<Dirent>,): Promise<readonly string[]> {
      /**
       Absolute path of this entry.
       */
      const absolute = join(
        dir,
        entry.name,
      );
      return entry.isDirectory() ? await listFilesSorted(absolute,) : [absolute,];
    },
  ),);
  return nested.flat()
    .toSorted();
}

/**
 Computes the image reference for the current repo state.
 
 The hash covers dependency identity (lockfile), build recipe
 (Containerfile plus ignore list), and this package's own sources,
 because containers execute the baked copy of that source; without the
 source hash a container-side change would silently reuse stale images.
 
 @param options - Repository root.
 
 @returns Image reference including content-derived tag.
 
 @example
 ```ts
 await imageReference({ repoRoot });
 // 'localhost/mutation-test-runtime:3fc9...'
 ```
 */
export async function imageReference(options: {
  readonly repoRoot: string;
},): Promise<string> {
  /**
   Content hash over inputs that must trigger a rebuild.
   */
  const hash = createHash('sha256',);
  /**
   Source directories inside the container's execution closure; host
   orchestration changes must not force image rebuilds.
   */
  const containerSourceDirs = await Promise.all([
    'package/mutation-test/cli/src/container',
    'package/mutation-test/cli/src/engine',
  ].map(async function list(dir,): Promise<readonly string[]> {
    return await listFilesSorted(join(
      options.repoRoot,
      dir,
    ),);
  },),);
  /**
   This package's container-executed source files, baked into the image.
   */
  const sourceFiles = [
    ...containerSourceDirs.flat(),
    join(
      options.repoRoot,
      'package/mutation-test/cli/src/shard-schema.ts',
    ),
    join(
      options.repoRoot,
      'package/mutation-test/cli/src/mounts.ts',
    ),
    join(
      options.repoRoot,
      'package/mutation-test/cli/src/is-record.ts',
    ),
  ]
    .filter(function keepRuntime(file,): boolean {
      return !file.endsWith('.test.ts',);
    },)
    .toSorted();
  /**
   Rebuild-triggering input files, read concurrently.
   */
  const inputs = await Promise.all([
    join(
      options.repoRoot,
      'pnpm-lock.yaml',
    ),
    join(
      options.repoRoot,
      CONTAINERFILE,
    ),
    join(
      options.repoRoot,
      CONTAINER_IGNOREFILE,
    ),
    ...sourceFiles,
  ].map(async function readInput(input,): Promise<Buffer> {
    return await readFile(input,);
  },),);

  for (const content of inputs)
    hash.update(content,);

  return `${IMAGE_REPOSITORY}:${hash.digest('hex',)
    .slice(
      0,
      TAG_HEX_LENGTH,
    )}`;
}

/**
 Returns whether an image reference exists locally.
 
 @param reference - Image reference to probe.
 
 @returns Whether podman knows the image.
 
 @example
 ```ts
 await imageExists('localhost/mutation-test-runtime:abc');
 ```
 */
async function imageExists(reference: string,): Promise<boolean> {
  try {
    await spawn(
      'podman',
      [
        'image',
        'exists',
        reference,
      ],
    );
    return true;
  }
  catch (error) {
    tagged({
      tag: imageExists.name,
      l,
    },)
      .debug(`image probe negative for ${reference}: ${String(error,)}`,);
    return false;
  }
}

/**
 Ensures the runtime image exists, building it when needed.
 
 @param options - Repository root and build-skip toggle.
 
 @returns Usable image reference.
 
 @throws Error when skipping the build but no image exists.
 
 @example
 ```ts
 const image = await ensureRuntimeImage({ repoRoot, skipImageBuild: false });
 ```
 */
export async function ensureRuntimeImage(options: {
  readonly repoRoot: string;
  readonly skipImageBuild: boolean;
},): Promise<string> {
  /**
   Logger scoped to image resolution.
   */
  const rl = tagged({
    tag: ensureRuntimeImage.name,
    l,
  },);
  /**
   Content-derived image reference for the current repo state.
   */
  const reference = await imageReference({ repoRoot: options.repoRoot, },);

  if (await imageExists(reference,)) {
    rl.info(`reusing image ${reference}`,);
    return reference;
  }

  if (options.skipImageBuild)
    throw new Error(`image ${reference} missing and --skip-image-build was set`,);

  rl.info(`building image ${reference}`,);
  await spawn(
    'podman',
    [
      'build',
      '--file',
      join(
        options.repoRoot,
        CONTAINERFILE,
      ),
      '--ignorefile',
      join(
        options.repoRoot,
        CONTAINER_IGNOREFILE,
      ),
      '--tag',
      reference,
      options.repoRoot,
    ],
    {
      stdout: 'inherit',
      stderr: 'inherit',
    },
  );
  return reference;
}
