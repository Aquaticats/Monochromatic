/**
 Release-age and trust checks over a set of locked versions.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import type { RustCatalogSettings, } from '@monochromatic-dev/module-rust-catalog/ts';

import type { LockedVersion, } from './lockfile-diff.ts';
import { l, } from './logger.ts';
import {
  releaseAgeProblems,
  trustProblems,
} from './policy.ts';
import type { Registry, } from './registry.ts';

//region Grouping

/**
 Versions grouped by crate name, so each crate costs one registry lookup.

 @param versions - locked versions

 @returns name to its versions, in first-seen order

 @example
 ```ts
 groupByName([{ name: 'a', version: '1.0.0', }, { name: 'a', version: '2.0.0', },]).get('a'); // ['1.0.0', '2.0.0']
 ```
 */
export function groupByName(versions: readonly LockedVersion[],): ReadonlyMap<string, readonly string[]> {
  /**
   Versions per name.
   */
  const groups = new Map<string, string[]>();
  for (const locked of versions)
    groups.set(locked.name, [...(groups.get(locked.name,) ?? []), locked.version,],);
  return groups;
}

//endregion Grouping

//region Checks

/**
 Outcome of a check pass.

 @example
 ```ts
 const outcome: CheckOutcome = { problems: [], warnings: [], };
 ```
 */
export type CheckOutcome = {
  /**
   Findings that refuse the update or fail the verification.
   */
  readonly problems: readonly string[];
  /**
   Findings reported without refusing (release age with `minimumReleaseAgeStrict = false`).
   */
  readonly warnings: readonly string[];
};

/**
 Checks release age for versions from the sparse index's `pubtime`.

 @param versions - versions to check

 @param settings - catalog settings

 @param registry - registry lookups

 @param now - current time in milliseconds

 @returns problems when `minimumReleaseAgeStrict` is true, otherwise warnings

 @throws RegistryError when the index cannot be read

 @example
 ```ts
 await checkReleaseAge({ versions, settings, registry, now: Date.now(), });
 ```
 */
export async function checkReleaseAge(
  { versions, settings, registry, now, }: {
    readonly versions: readonly LockedVersion[];
    readonly settings: RustCatalogSettings;
    readonly registry: Registry;
    readonly now: number;
  },
): Promise<CheckOutcome> {
  if (settings.minimumReleaseAge === 0)
    return { problems: [], warnings: [], };
  /**
   Findings across every crate.
   */
  const findings = (await Promise.all([...groupByName(versions,),].map(async function checkName([name, nameVersions,],): Promise<readonly string[]> {
    /**
     Publish time per version of this crate.
     */
    const times = await registry.pubtimes(name,);
    return nameVersions.flatMap(function check(version: string,): readonly string[] {
      return releaseAgeProblems({ name, version, pubtime: times.get(version,) ?? '', settings, now, },);
    },);
  },),)).flat();
  return settings.minimumReleaseAgeStrict ? { problems: findings, warnings: [], } : { problems: [], warnings: findings, };
}

/**
 Checks the trust policy, one crates.io history request per crate at most.

 Requests run one at a time so the registry's spacing holds.

 @param versions - versions to check

 @param settings - catalog settings

 @param registry - registry lookups

 @param now - current time in milliseconds

 @returns trust problems

 @throws RegistryError when crates.io cannot be read

 @example
 ```ts
 await checkTrust({ versions, settings, registry, now: Date.now(), });
 ```
 */
export async function checkTrust(
  { versions, settings, registry, now, }: {
    readonly versions: readonly LockedVersion[];
    readonly settings: RustCatalogSettings;
    readonly registry: Registry;
    readonly now: number;
  },
): Promise<readonly string[]> {
  /**
   Logger for progress.
   */
  const log = tagged({ tag: checkTrust.name, l, },);
  if (settings.trustPolicy === 'off')
    return [];
  /**
   Crates to look up.
   */
  const groups = [...groupByName(versions,),];
  log.info(`checking the trust policy for ${String(versions.length,)} versions of ${String(groups.length,)} crates`,);
  /**
   Problems found so far.
   */
  const problems: string[] = [];
  for (const [name, nameVersions,] of groups) {
    /**
     Every release of this crate.
     */
    // oxlint-disable-next-line no-await-in-loop -- crates.io allows one API request per second, so lookups must be sequential.
    const history = await registry.history({ name, versions: nameVersions, },);
    for (const version of nameVersions)
      problems.push(...trustProblems({ name, version, history, settings, now, },),);
  }
  return problems;
}

//endregion Checks
