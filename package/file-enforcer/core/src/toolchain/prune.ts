/**
 `prune.ts`: deletes the superseded versions of the crates this repository publishes, right after the sync
 provisions a new version of one.

 `cargo binstall` replaces an installed binary in place, so the stale copies live in cargo's registry cache
 (`registry/cache/<index>/<crate>-<version>.crate`) and its extracted sources
 (`registry/src/<index>/<crate>-<version>/`), left by earlier builds and installs.
 */

import {
  readdir,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { l, } from '../logger.ts';
import { isErrnoException, } from './fs-probe.ts';
import {
  homeDirFromEnv,
  type PathEnv,
} from './paths.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Ownership

/**
 Name prefix every crate this repository publishes carries
 (`monochromatic-lint`, `monochromatic-git-policy-cli`, ...).
 */
const OWN_CRATE_PREFIX = 'monochromatic-';

/**
 Suffix of a crate archive in the registry cache.
 */
const CRATE_ARCHIVE_SUFFIX = '.crate';

/**
 Whether one plan entry is a crate this repository publishes.

 @param entry - Plan entry to test

 @returns `true` for a `cargo-binstall` entry whose crate name carries the repository prefix

 @example
 ```ts
 isOwnCrate(toolById('cargo:monochromatic-lint',),); // true
 ```
 */
export function isOwnCrate(entry: ToolPlanEntry,): boolean {
  return (entry.route === 'cargo-binstall')
    && (entry.packageId ?? entry.id).startsWith(OWN_CRATE_PREFIX,);
}

//endregion Ownership

//region Version names

/**
 A semantic version, whole: three numeric parts with an optional pre-release or build suffix.
 */
const SEMVER_WHOLE = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.+-]*)?$/u;

/**
 Version a registry entry name carries after its crate name, so `monochromatic-lint-closure-0.1.0` is not
 read as a version of `monochromatic-lint`.

 @param name - Entry name without the archive suffix
 @param crate - Crate name the entry must belong to

 @returns The version text, or `null` when the entry is another crate's

 @example
 ```ts
 versionOfEntry({ name: 'monochromatic-lint-0.1.0', crate: 'monochromatic-lint', },); // '0.1.0'
 ```
 */
export function versionOfEntry(
  {
    name,
    crate,
  }: {
    readonly name: string;
    readonly crate: string;
  },
): string | null {
  if (!name.startsWith(`${crate}-`,))
    return null;
  /**
   Text after the crate name and its separator.
   */
  const rest = name.slice(crate.length + 1,);
  return SEMVER_WHOLE.test(rest,)
    ? rest
    : null;
}

//endregion Version names

//region Prune

/**
 Child names of one directory; a missing directory has none.

 @param dir - Absolute directory to list

 @returns Entry names in lexicographic order
 */
async function childNames(dir: string,): Promise<readonly string[]> {
  try {
    return (await readdir(dir,))
      .toSorted();
  } catch (error) {
    if (!isErrnoException(error,))
      throw error;
    return [];
  }
}

/**
 Deletes every cached archive and extracted source of an own crate except the locked version.

 Runs only for a crate this repository publishes. A deletion that fails is logged and skipped, since a leftover
 cache entry never affects the install that just succeeded.

 @param entry - Plan entry whose superseded versions are deleted
 @param env - Environment consulted for `CARGO_HOME` and `HOME`

 @returns Absolute paths that were deleted, in registry then name order

 @example
 ```ts
 await pruneStaleOwnCrates({ entry, env: process.env, },); // ['/home/me/.cargo/registry/cache/index.crates.io-x/monochromatic-lint-0.1.0.crate']
 ```
 */
export async function pruneStaleOwnCrates(
  {
    entry,
    env,
  }: {
    readonly entry: ToolPlanEntry;
    readonly env: PathEnv;
  },
): Promise<readonly string[]> {
  if (!isOwnCrate(entry,))
    return [];
  /**
   Logger tagged with this prune.
   */
  const log = tagged({
    tag: pruneStaleOwnCrates.name,
    l,
  },);
  /**
   Crate name as the registry spells it.
   */
  const crate = entry.packageId ?? entry.id;
  /**
   Cargo home the install wrote to.
   */
  const cargoHome = (env.CARGO_HOME !== undefined) && (env.CARGO_HOME !== '')
    ? env.CARGO_HOME
    : join(
      homeDirFromEnv({ env, },),
      '.cargo',
    );
  /**
   Absolute paths found to hold a superseded version.
   */
  const stale: string[] = [];
  for (const kind of [
    {
      dir: 'cache',
      suffix: CRATE_ARCHIVE_SUFFIX,
    },
    {
      dir: 'src',
      suffix: '',
    },
  ]) {
    /**
     Registry kind directory holding one subdirectory per registry index.
     */
    const kindDir = join(
      cargoHome,
      'registry',
      kind.dir,
    );
    for (const index of await childNames(kindDir,)) {
      /**
       One registry index directory.
       */
      const indexDir = join(
        kindDir,
        index,
      );
      for (const name of await childNames(indexDir,)) {
        /**
         Version this entry holds of the crate, when it holds one.
         */
        const version = name.endsWith(kind.suffix,)
          ? versionOfEntry({
            name: name.slice(
              0,
              name.length - kind.suffix.length,
            ),
            crate,
          },)
          : null;
        if ((version !== null) && (version !== entry.version))
          stale.push(join(
            indexDir,
            name,
          ),);
      }
    }
  }
  for (const path of stale) {
    try {
      // oxlint-disable-next-line no-await-in-loop -- one deletion at a time so a failure names its path
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
      log.info(`${entry.id}: deleted stale ${path}`,);
    } catch (error) {
      log.warn(`${entry.id}: could not delete ${path}: ${caughtValueText(error,)}`,);
    }
  }
  return stale;
}

//endregion Prune
