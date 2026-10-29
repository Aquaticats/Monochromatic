/**
 Release-age and trust-policy decisions for one locked version, as pure functions.

 Release age follows pnpm's `minimumReleaseAge`: a version younger than the
 configured minutes fails unless excluded. The trust policy follows pnpm's
 `trustPolicy: no-downgrade` with heuristic levels from crates.io data:
 Trusted Publishing outranks a token publish by an account that published an
 earlier release, which outranks a first-time publisher. A version fails when
 an earlier-published stable release had a higher level, when its
 trusted-publishing repository differs from earlier trusted releases, or when
 its declared `repository` differs from the previous release.
 See `doc/decision/rust-dependency-catalog.md`.

 @module
 */

import type { RustCatalogSettings, } from '@monochromatic-dev/module-rust-catalog/ts';

import { isExcluded, } from './selectors.ts';

//region Types

/**
 What crates.io records about one published version.

 @example
 ```ts
 const record: VersionRecord = {
   num: '1.0.104', createdAt: '2026-07-18T20:59:37Z', publisherId: '3618', trustpubRepository: '', repository: 'https://github.com/dtolnay/anyhow',
 };
 ```
 */
export type VersionRecord = {
  readonly num: string;
  readonly createdAt: string;
  /**
   Publishing account id; empty for a Trusted Publishing release.
   */
  readonly publisherId: string;
  /**
   Repository of the Trusted Publishing workflow; empty for a token publish.
   */
  readonly trustpubRepository: string;
  /**
   Declared `package.repository`; empty when undeclared.
   */
  readonly repository: string;
};

//endregion Types

//region Release age

/**
 Minutes in one millisecond-based clock unit conversion.
 */
const MS_PER_MINUTE = 60_000;

/**
 Release-age problems for one locked version.

 @param name - crate name

 @param version - locked version

 @param pubtime - publish time from the sparse index, empty when the index has none

 @param settings - catalog settings

 @param now - current time in milliseconds

 @returns problem descriptions, empty when the version passes

 @example
 ```ts
 releaseAgeProblems({ name: 'x', version: '1.0.0', pubtime: '2026-10-29T00:00:00Z', settings, now: Date.now(), });
 ```
 */
export function releaseAgeProblems(
  {
    name,
    version,
    pubtime,
    settings,
    now,
  }: {
    readonly name: string;
    readonly version: string;
    readonly pubtime: string;
    readonly settings: RustCatalogSettings;
    readonly now: number;
  },
): readonly string[] {
  if (isExcluded({
    selectors: settings.minimumReleaseAgeExclude,
    name,
    version,
  },))
    return [];
  if (pubtime === '') {
    return settings.minimumReleaseAgeIgnoreMissingTime
      ? []
      : [`${name}@${version} has no publish time in the crates.io index, and minimumReleaseAgeIgnoreMissingTime is false`,];
  }
  /**
   When the version passes the age gate.
   */
  const mature = Date.parse(pubtime,) + (settings.minimumReleaseAge * MS_PER_MINUTE);
  if (mature <= now)
    return [];
  return [
    `${name}@${version} was published ${pubtime} and passes minimumReleaseAge (${String(settings.minimumReleaseAge,)} minutes) at ${new Date(mature,).toISOString()}; wait until then or add "${name}" to minimumReleaseAgeExclude in rust-catalog.toml`,
  ];
}

//endregion Release age

//region Trust

/**
 Whether a version string is a prerelease.

 @param version - version text

 @returns whether it carries a `-` label

 @example
 ```ts
 isPrerelease('1.0.0-rc.1'); // true
 ```
 */
function isPrerelease(version: string,): boolean {
  return version.split('+',)[0]
    ?.includes('-',)
    === true;
}

/**
 Heuristic trust level of one release given the releases published before it.

 @param record - release to rate

 @param earlier - releases published before it

 @returns 3 for Trusted Publishing, 2 for a known token publisher, 1 for a first-time publisher

 @example
 ```ts
 trustLevel({ record, earlier: [], }); // 1 or 3
 ```
 */
export function trustLevel({
  record,
  earlier,
}: {
  readonly record: VersionRecord;
  readonly earlier: readonly VersionRecord[]
},): number {
  if (record.trustpubRepository !== '')
    return 3;
  return earlier.some(function samePublisher(previous: VersionRecord,): boolean {
    return (previous.publisherId !== '') && (previous.publisherId === record.publisherId);
  },)
    ? 2
    : 1;
}

/**
 Trust-policy problems for one locked version.

 @param name - crate name

 @param version - locked version

 @param history - every release of the crate crates.io returned

 @param settings - catalog settings

 @param now - current time in milliseconds

 @returns problem descriptions, empty when the version passes or the policy is off

 @example
 ```ts
 trustProblems({ name: 'anyhow', version: '1.0.104', history, settings, now: Date.now(), });
 ```
 */
export function trustProblems(
  {
    name,
    version,
    history,
    settings,
    now,
  }: {
    readonly name: string;
    readonly version: string;
    readonly history: readonly VersionRecord[];
    readonly settings: RustCatalogSettings;
    readonly now: number;
  },
): readonly string[] {
  if ((settings.trustPolicy === 'off') || isExcluded({
    selectors: settings.trustPolicyExclude,
    name,
    version,
  },))
    return [];
  /**
   The release being checked.
   */
  const record = history.find(function isVersion(entry: VersionRecord,): boolean {
    return entry.num === version;
  },);
  if (record === undefined)
    return [`${name}@${version} is not in the crates.io version list, so its trust cannot be checked; add it to trustPolicyExclude if it is expected`,];
  if ((settings.trustPolicyIgnoreAfter !== undefined) && ((Date.parse(record.createdAt,) + (settings.trustPolicyIgnoreAfter * MS_PER_MINUTE)) <= now))
    return [];
  /**
   Releases published before this one, oldest first.
   */
  const earlier = history.filter(function isEarlier(entry: VersionRecord,): boolean {
    return Date.parse(entry.createdAt,) < Date.parse(record.createdAt,);
  },)
    .toSorted(function byDate(
      left: VersionRecord,
      right: VersionRecord,
    ): number {
      return Date.parse(left.createdAt,) - Date.parse(right.createdAt,);
    },);
  /**
   Problems found.
   */
  const problems: string[] = [];
  /**
   This release's level.
   */
  const level = trustLevel({
    record,
    earlier,
  },);
  /**
   Highest level among earlier stable releases (prereleases do not raise the bar for a stable release).
   */
  const best = earlier.reduce(
    function highest(
      top: number,
      entry: VersionRecord,
      index: number,
    ): number {
    // Each entry is rated against the releases before it in `earlier`, so the
    // slice uses the unfiltered position; prereleases are only skipped as candidates.
      if ((!isPrerelease(version,)) && isPrerelease(entry.num,))
        return top;
      return Math.max(
        top,
        trustLevel({
          record: entry,
          earlier: earlier.slice(
            0,
            index,
          ),
        },),
      );
    },
    0,
  );
  if (best > level)
    problems.push(`${name}@${version} has trust level ${String(level,)}, below an earlier release's ${String(best,)} (3 trusted publishing, 2 known publisher, 1 first-time publisher)`,);
  /**
   Repositories earlier trusted releases came from.
   */
  const trustedRepositories = new Set(earlier.map(function toRepo(entry: VersionRecord,): string {
    return entry.trustpubRepository;
  },)
    .filter(function isSet(repository: string,): boolean {
      return repository !== '';
    },),);
  if ((record.trustpubRepository !== '') && (trustedRepositories.size > 0)
    && (!trustedRepositories.has(record.trustpubRepository,)))
    problems.push(`${name}@${version} was trusted-published from ${record.trustpubRepository}, not ${[...trustedRepositories,].join(', ',)}`,);
  /**
   The release just before this one.
   */
  const previous = earlier.at(-1,);
  if ((previous !== undefined) && (previous.repository !== record.repository))
    problems.push(`${name}@${version} declares repository ${JSON.stringify(record.repository,)}, but ${previous.num} declared ${JSON.stringify(previous.repository,)}`,);
  return problems.map(function withRemedy(problem: string,): string {
    return `${problem}; review the release, then add "${name}@${version}" to trustPolicyExclude in rust-catalog.toml if it is legitimate`;
  },);
}

//endregion Trust
