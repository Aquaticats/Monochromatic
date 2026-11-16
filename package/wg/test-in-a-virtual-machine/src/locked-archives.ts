/**
 The two toolchains a guest installs, taken from the version the repository
 already locks.

 The lockfile supplies the archive URL and its SHA-256 for this platform, so a
 guest runs the same Node and the same pnpm every other task in the checkout
 uses. An archive is downloaded once, checked, and kept beside this package's
 cache so a later run downloads nothing.

 @module
 */

import {
  createHash,
} from 'node:crypto';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { basename, join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  CACHE_DIRECTORY,
  ensureCacheDirectory,
} from './cache-directory.ts';
import { fileExists, } from './file-presence.ts';

/**
 The platform key the lock uses for this machine's architecture.
 */
const LOCK_PLATFORM_KEY = 'linux-x64';

/**
 Path of the repository's toolchain lock.
 */
const LOCK_PATH = fileURLToPath(new URL(
  '../../../../toolchain.lock.json',
  import.meta.url,
),);

/**
 One archive the lock names.
 */
export type LockedArchive = {
  /**
   Tool name as the lock keys it.
   */
  readonly tool: string;

  /**
   Version the lock records.
   */
  readonly version: string;

  /**
   Archive URL for this machine's platform.
   */
  readonly url: string;

  /**
   SHA-256 the archive must have.
   */
  readonly sha256: string;
};

/**
 Reports whether a value is a JSON object with named fields.

 @param value - value to inspect

 @returns whether the value is such an object
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null) && (!Array.isArray(value,));
}

/**
 Reads the lock and answers one tool's archive for this platform.

 @param tool - key the lock uses, such as `node` or `pnpm`

 @returns URL and checksum of that tool's archive

 @throws when the lock has no archive for this platform

 @example
 ```ts
 await lockedArchive({ tool: 'node' });
 ```
 */
export async function lockedArchive({ tool, }: { readonly tool: string; },): Promise<LockedArchive> {
  /**
   Parsed lockfile.
   */
  const lock: unknown = JSON.parse(await readFile(
    LOCK_PATH,
    'utf8',
  ),);
  if (!isRecord(lock,) || (!isRecord(lock['tools']))) {
    throw new Error('toolchain.lock.json has no tools table.',);
  }
  /**
   Entry the lock keeps for the tool.
   */
  const entry: unknown = lock['tools'][tool];
  if (!isRecord(entry,)) {
    throw new Error(`toolchain.lock.json has no entry for ${tool}.`,);
  }
  /**
   Archive records the lock keeps, one per platform.
   */
  const artifacts: unknown = entry['artifacts'];
  if (!isRecord(artifacts,)) {
    throw new Error(`toolchain.lock.json names no archives for ${tool}.`,);
  }
  /**
   The one archive this platform runs.
   */
  const archive: unknown = artifacts[LOCK_PLATFORM_KEY];
  if (!isRecord(archive,) || ((typeof archive['url']) !== 'string') || ((typeof archive['sha256']) !== 'string')) {
    throw new Error(`toolchain.lock.json has an incomplete ${LOCK_PLATFORM_KEY} archive for ${tool}.`,);
  }
  /**
   Version the entry records.
   */
  const version: unknown = entry['version'];
  return {
    sha256: archive['sha256'],
    tool,
    url: archive['url'],
    version: typeof version === 'string' ? version : 'unknown',
  };
}

/**
 Downloads an archive unless it is cached, and answers its path with its
 checksum verified.

 @param archive - the locked archive to have on disk

 @returns host path of the verified archive

 @throws when the checksum does not match what the lock records

 @example
 ```ts
 await haveArchive({ archive: await lockedArchive({ tool: 'node' }) });
 ```
 */
export async function haveArchive({ archive, }: { readonly archive: LockedArchive; },): Promise<string> {
  await ensureCacheDirectory();
  /**
   Cached copy of the archive.
   */
  const path = join(
    CACHE_DIRECTORY,
    `${archive.tool}-${basename(new URL(archive.url,).pathname,)}`,
  );
  if (!(await fileExists(path,))) {
    /**
     The archive as the network answered.
     */
    const response = await fetch(archive.url,);
    if (!response.ok) {
      throw new Error(`could not download ${archive.url}: ${String(response.status,)} ${response.statusText}`,);
    }
    await writeFile(
      path,
      Buffer.from(await response.arrayBuffer(),),
    );
  }
  /**
   What the cached bytes hash to.
   */
  const digest = createHash('sha256',)
    .update(await readFile(path,),)
    .digest('hex',);
  if (digest !== archive.sha256) {
    throw new Error(`the ${archive.tool} archive at ${path} hashes to ${digest}, and toolchain.lock.json records ${archive.sha256}.`,);
  }
  return path;
}
