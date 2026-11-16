/**
 The checkout a guest receives.

 A test needs the sources it imports, the packages' manifests so `pnpm` can
 install the workspace exactly as the lockfile describes, and the built bundles
 the integration tests drive. Build output, dependency folders and Git history
 are left out: the guest installs and builds what it needs from the lockfile,
 and nothing there reads history.

 @module
 */

import { spawn, } from 'node:child_process';
import { glob, } from 'node:fs/promises';
import { dirname, join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  CACHE_DIRECTORY,
  ensureCacheDirectory,
} from './cache-directory.ts';

/**
 Repository root this package lives in.
 */
const REPOSITORY_ROOT = fileURLToPath(new URL(
  '../../../..',
  import.meta.url,
),);

/**
 Files at the repository root that describe the workspace and its lockfile.
 */
const WORKSPACE_ROOT_FILES: readonly string[] = [
  'package.json',
  'pnpm-lock.yaml',
  'pnpm-workspace.yaml',
  '.pnpmfile.mjs',
  '.pnpmfile.policies.json',
];

/**
 Files one package contributes whatever its sources are.
 */
const PACKAGE_MANIFEST_FILES: readonly string[] = ['package.json',];

/**
 Directories whose contents the guest needs beyond sources and manifests.
 */
const EXTRA_DIRECTORIES: readonly string[] = [
  'package/wg/cli-quicker/dist/final/node',
  'package/wg/cli-quicker-exempt/Cargo.toml',
  'package/wg/cli-quicker-exempt/Cargo.lock',
];

/**
 Lists the paths the guest receives, relative to the repository root.

 @returns every path to pack, in the order it is written to the archive

 @example
 ```ts
 await payloadPaths();
 ```
 */
export async function payloadPaths(): Promise<readonly string[]> {
  /**
   Accumulated paths.
   */
  const paths: string[] = [...WORKSPACE_ROOT_FILES,];
  for await (const manifest of glob(
    'package/*/*/package.json',
    { cwd: REPOSITORY_ROOT, },
  )) {
    paths.push(manifest,);
  }
  for await (const sources of glob(
    'package/*/*/src',
    { cwd: REPOSITORY_ROOT, },
  )) {
    paths.push(sources,);
  }
  if (!paths.includes('package/wg/cli-quicker/package.json',)) {
    throw new Error(`no workspace package was found below ${REPOSITORY_ROOT}; the payload would hold nothing to install.`,);
  }
  paths.push(...EXTRA_DIRECTORIES,);
  return paths.toSorted();
}

/**
 Writes the archive a guest unpacks.

 @param destination - host path of the archive to write

 @returns the archive's path

 @throws when `tar` fails

 @example
 ```ts
 await writePayload({ destination: '/work/payload.tar.gz' });
 ```
 */
export async function writePayload({ destination, }: { readonly destination: string; },): Promise<string> {
  /**
   Every path to pack, one per line, so no shell re-splits a name with spaces.
   */
  const listing = `${(await payloadPaths()).join('\n',)}\n`;
  await ensureCacheDirectory();
  /**
   `tar` reading the list from its own standard input.
   */
  const tar = spawn(
    'tar',
    [
      '--create',
      '--gzip',
      '--file',
      destination,
      '--directory',
      REPOSITORY_ROOT,
      '--verbatim-files-from',
      '--files-from',
      '-',
    ],
    {
      cwd: REPOSITORY_ROOT,
      stdio: [
        'pipe',
        'ignore',
        'pipe',
      ],
    },
  );
  /**
   Anything tar said about the files it packed.
   */
  let complaint = '';
  tar.stderr.setEncoding('utf8',);
  tar.stderr.on(
    'data',
    function keep(text: string,): void {
      complaint += text;
    },
  );
  tar.stdin.end(listing,);
  /**
   Exit status of the tar call.
   */
  const status = await new Promise<number>(function waitForExit(
    resolve,
  ): void {
    tar.on(
      'close',
      function closed(code,): void {
        resolve(code ?? 1,);
      },
    );
  },);
  if (status !== 0) {
    throw new Error(`tar could not write ${destination}: ${complaint.trim()}`,);
  }
  return destination;
}

/**
 Path of the payload archive this package keeps between runs.

 Kept beside the downloaded toolchain archives, because a build recreates it and
 nothing depends on it existing beforehand.

 @returns host path of the archive
 */
export const PAYLOAD_PATH: string = join(
  CACHE_DIRECTORY,
  'payload.tar.gz',
);
