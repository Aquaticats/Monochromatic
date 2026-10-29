/**
 `prune.ts`: deletes the superseded versions of the crates this repository publishes, right after the sync
 provisions a new version of one.

 `cargo binstall` replaces an installed binary in place, so the stale copies live in cargo's registry cache
 (`registry/cache/<index>/<crate>-<version>.crate`) and its extracted sources
 (`registry/src/<index>/<crate>-<version>/`), left by earlier `cargo install` runs and builds.
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
 A registry entry name split into its crate and version.
 */
export type CrateEntryName = {
  readonly crate: string;
  readonly version: string;
};

/**
 Splits a registry entry name at the first hyphen whose remainder is a whole semantic version, so a pre-release
 hyphen (`0.5.0-rc.1`) stays in the version and a longer crate name (`monochromatic-lint-closure`) stays whole.

 @param name - Entry name without the archive suffix

 @returns The crate and version, or `null` when no hyphen is followed by a whole version

 @example
 ```ts
 splitEntryName('monochromatic-lint-0.1.0',); // { crate: 'monochromatic-lint', version: '0.1.0' }
 ```
 */
export function splitEntryName(name: string,): CrateEntryName | null {
  for (let at = name.indexOf('-',); at !== -1; at = name.indexOf(
    '-',
    at + 1,
  )) {
    /**
     Text after this hyphen.
     */
    const rest = name.slice(at + 1,);
    if (SEMVER_WHOLE.test(rest,)) {
      return {
        crate: name.slice(
          0,
          at,
        ),
        version: rest,
      };
    }
  }
  return null;
}

/**
 Orders two semantic versions: numeric core first, then a release above its own pre-release.

 @param left - First version
 @param right - Second version

 @returns Negative when `left` is older, positive when newer, zero when equal

 @example
 ```ts
 compareVersions({ left: '0.1.0', right: '0.1.1', },); // -1
 ```
 */
export function compareVersions(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   Numeric core and pre-release text of each side.
   */
  const [leftCore = '', leftPre,] = left.split(
    '-',
    2,
  );
  const [rightCore = '', rightPre,] = right.split(
    '-',
    2,
  );
  /**
   Numeric parts of each core.
   */
  const leftParts = leftCore.split('.',)
    .map(Number,);
  const rightParts = rightCore.split('.',)
    .map(Number,);
  for (let i = 0; i < 3; i++) {
    /**
     Difference of this numeric part.
     */
    const difference = (leftParts[i] ?? 0) - (rightParts[i] ?? 0);
    if (difference !== 0)
      return Math.sign(difference,);
  }
  if (leftPre === rightPre)
    return 0;
  if (leftPre === undefined)
    return 1;
  if (rightPre === undefined)
    return -1;
  return leftPre < rightPre
    ? -1
    : 1;
}

//endregion Version names

//region Prune

/**
 One own-crate entry found in the registry cache.
 */
type RegistryEntry = CrateEntryName & { readonly path: string; };

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
 Lists every own-crate archive and extracted source in a cargo home's registry.

 @param cargoHome - Absolute cargo home

 @returns Entries in registry kind, index, then name order
 */
async function listOwnEntries(cargoHome: string,): Promise<readonly RegistryEntry[]> {
  /**
   Own-crate entries found so far.
   */
  const found: RegistryEntry[] = [];
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
        if (!name.endsWith(kind.suffix,))
          continue;
        /**
         Crate and version this entry name carries.
         */
        const split = splitEntryName(name.slice(
          0,
          name.length - kind.suffix.length,
        ),);
        if ((split !== null) && split.crate.startsWith(OWN_CRATE_PREFIX,)) {
          found.push({
            ...split,
            path: join(
              indexDir,
              name,
            ),
          },);
        }
      }
    }
  }
  return found;
}

/**
 Deletes every superseded cached archive and extracted source of the repository's own crates.

 Runs when the entry just installed is a crate this repository publishes. A plan crate keeps its locked version;
 any other own crate (a library dependency such as `monochromatic-jsonc-edit`) keeps its newest cached version.
 A deletion that fails is logged and skipped, since a leftover cache entry never affects the install that just
 succeeded.

 @param entry - Plan entry that was just installed
 @param tools - Plan the locked versions are read from
 @param env - Environment consulted for `CARGO_HOME` and `HOME`

 @returns Absolute paths that were deleted, in registry then name order

 @example
 ```ts
 await pruneStaleOwnCrates({ entry, tools: ALL_TOOLS, env: process.env, },); // ['/home/me/.cargo/registry/cache/index.crates.io-x/monochromatic-lint-0.1.0.crate']
 ```
 */
export async function pruneStaleOwnCrates(
  {
    entry,
    tools,
    env,
  }: {
    readonly entry: ToolPlanEntry;
    readonly tools: readonly ToolPlanEntry[];
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
   Locked version of each plan crate this repository publishes.
   */
  const locked = new Map<string, string>();
  for (const tool of tools) {
    if (isOwnCrate(tool,))
      locked.set(
        tool.packageId ?? tool.id,
        tool.version,
      );
  }
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
   Every own-crate entry in the registry.
   */
  const entries = await listOwnEntries(cargoHome,);
  /**
   Newest cached version of each crate, kept when the plan does not lock one.
   */
  const newest = new Map<string, string>();
  for (const found of entries) {
    /**
     Newest version recorded so far for this crate.
     */
    const current = newest.get(found.crate,);
    if ((current === undefined) || (compareVersions({
      left: found.version,
      right: current,
    },) > 0))
      newest.set(
        found.crate,
        found.version,
      );
  }
  /**
   Entries holding a version other than the one kept for their crate.
   */
  const stale = entries.filter(
    function isSuperseded(found,): boolean {
      return found.version !== (locked.get(found.crate,) ?? newest.get(found.crate,));
    },
  );
  for (const found of stale) {
    try {
      // oxlint-disable-next-line no-await-in-loop -- one deletion at a time so a failure names its path
      await rm(
        found.path,
        {
          recursive: true,
          force: true,
        },
      );
      log.info(`${entry.id}: deleted stale ${found.path}`,);
    } catch (error) {
      log.warn(`${entry.id}: could not delete ${found.path}: ${caughtValueText(error,)}`,);
    }
  }
  return stale.map(
    function pathOf(found,): string {
      return found.path;
    },
  );
}

//endregion Prune
