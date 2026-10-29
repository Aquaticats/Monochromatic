/**
 Release-age and trust checks over a set of locked versions.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { RustCatalogSettings, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import type { LockedVersion, } from './lockfile-diff.ts';
import { l, } from './logger.ts';
import {
  releaseAgeProblems,
  trustProblems,
  type VersionRecord,
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
    groups.set(
      locked.name,
      [
        ...(groups.get(locked.name,) ?? []),
        locked.version,
      ],
    );
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
  {
    versions,
    settings,
    registry,
    now,
  }: {
    readonly versions: readonly LockedVersion[];
    readonly settings: RustCatalogSettings;
    readonly registry: Registry;
    readonly now: number;
  },
): Promise<CheckOutcome> {
  if (settings.minimumReleaseAge === 0)
    return {
      problems: [],
      warnings: [],
    };
  /**
   Findings across every crate.
   */
  const findings = (await Promise.all([...groupByName(versions,),].map(async function checkName([name, nameVersions,]: readonly [
    string,
    readonly string[],
  ],): Promise<readonly string[]> {
    /**
     Publish time per version of this crate.
     */
    const times = await registry.pubtimes(name,);
    return nameVersions.flatMap(function check(version: string,): readonly string[] {
      return releaseAgeProblems({
        name,
        version,
        pubtime: times.get(version,) ?? '',
        settings,
        now,
      },);
    },);
  },),)).flat();
  return settings.minimumReleaseAgeStrict ? {
    problems: findings,
    warnings: [],
  } : {
    problems: [],
    warnings: findings,
  };
}

/**
 Trust problems of several versions of one crate.

 @param name - crate name

 @param versions - versions to check

 @param history - every release of the crate

 @param settings - catalog settings

 @param now - current time in milliseconds

 @param ownerIds - account ids of the crate's listed individual owners

 @returns each refused version with its findings

 @example
 ```ts
 crateTrustRefusals({ name, versions, history, settings, now, ownerIds: new Set(), });
 ```
 */
function crateTrustRefusals(
  {
    name,
    versions,
    history,
    settings,
    now,
    ownerIds,
  }: {
    readonly name: string;
    readonly versions: readonly string[];
    readonly history: readonly VersionRecord[];
    readonly settings: RustCatalogSettings;
    readonly now: number;
    readonly ownerIds: ReadonlySet<string>;
  },
): readonly TrustRefusal[] {
  return versions.flatMap(function check(version: string,): readonly TrustRefusal[] {
    /**
     Findings for this version.
     */
    const problems = trustProblems({
      name,
      version,
      history,
      settings,
      now,
      ownerIds,
    },);
    return problems.length === 0
      ? []
      : [
        {
          name,
          version,
          problems,
        },
      ];
  },);
}

/**
 One version the trust policy refuses, with every finding against it.

 @example
 ```ts
 const refusal: TrustRefusal = { name: 'dep', version: '1.0.1', problems: ['dep@1.0.1 has trust level 1 ...',], };
 ```
 */
export type TrustRefusal = {
  readonly name: string;
  readonly version: string;
  readonly problems: readonly string[];
};

/**
 Checks the trust policy: one crates.io history request per crate at most,
 plus an owners request for a crate that would otherwise be refused.

 Requests run one at a time so the registry's spacing holds.

 @param versions - versions to check

 @param settings - catalog settings

 @param registry - registry lookups

 @param now - current time in milliseconds

 @returns each refused version with its findings

 @throws RegistryError when crates.io cannot be read

 @example
 ```ts
 await trustRefusals({ versions, settings, registry, now: Date.now(), });
 ```
 */
export async function trustRefusals(
  {
    versions,
    settings,
    registry,
    now,
  }: {
    readonly versions: readonly LockedVersion[];
    readonly settings: RustCatalogSettings;
    readonly registry: Registry;
    readonly now: number;
  },
): Promise<readonly TrustRefusal[]> {
  /**
   Logger for progress.
   */
  const log = tagged({
    tag: trustRefusals.name,
    l,
  },);
  if (settings.trustPolicy === 'off')
    return [];
  /**
   Crates to look up.
   */
  const groups = [...groupByName(versions,),];
  log.info(`checking the trust policy for ${String(versions.length,)} versions of ${String(groups.length,)} crates`,);
  await registry.prefetch(
    new Set(groups.map(function toName([name,]: readonly [
      string,
      readonly string[],
    ],): string {
      return name;
    },),),
  );
  /**
   Refusals found so far.
   */
  const refusals: TrustRefusal[] = [];
  for (const [name, nameVersions,] of groups) {
    /**
     Every release of this crate.
     */
    // oxlint-disable-next-line no-await-in-loop -- crates.io allows one API request per second, so lookups must be sequential.
    const history = await registry.history({
      name,
      versions: nameVersions,
    },);
    /**
     Findings without the owner list; most crates pass here and cost no owners request.
     */
    const withoutOwners = crateTrustRefusals({
      name,
      versions: nameVersions,
      history,
      settings,
      now,
      ownerIds: new Set<string>(),
    },);
    if (withoutOwners.length === 0)
      continue;
    // A listed owner's first release ranks as a known publisher, so a refusal is
    // rechecked against the current owner list before it stands.
    /**
     Account ids of the crate's listed individual owners.
     */
    // oxlint-disable-next-line no-await-in-loop -- crates.io allows one API request per second, so lookups must be sequential.
    const ownerIds = await registry.owners(name,);
    refusals.push(...crateTrustRefusals({
      name,
      versions: nameVersions,
      history,
      settings,
      now,
      ownerIds,
    },),);
  }
  return refusals;
}

/**
 Trust findings as messages, for callers that report rather than act on refusals.

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
  {
    versions,
    settings,
    registry,
    now,
  }: {
    readonly versions: readonly LockedVersion[];
    readonly settings: RustCatalogSettings;
    readonly registry: Registry;
    readonly now: number;
  },
): Promise<readonly string[]> {
  return (await trustRefusals({
    versions,
    settings,
    registry,
    now,
  },)).flatMap(function messages(refusal: TrustRefusal,): readonly string[] {
    return refusal.problems;
  },);
}

//endregion Checks
