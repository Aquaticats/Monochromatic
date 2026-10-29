/**
 Validation of the catalog's pnpm-named supply-chain settings.

 @module
 */

import {
  NOT_A_NUMBER,
  toNumber,
} from './toml-values.ts';
import type {
  RustCatalogIssue,
  RustCatalogSettings,
  RustCatalogTrustPolicy,
} from './types.ts';

//region Readers

/**
 Sentinel for a minute setting the file does not declare (or declares invalidly).
 */
const NO_MINUTES: unique symbol = Symbol('rust-catalog.toml minute setting absent or invalid',);

/**
 pnpm's documented default for `minimumReleaseAge`, in minutes.
 */
const DEFAULT_MINIMUM_RELEASE_AGE = 1_440;

/**
 Reads a list-of-strings setting.

 @param key - setting name

 @param value - materialized TOML value

 @param issues - collector the problems are appended to

 @returns the strings, or an empty list when absent or invalid

 @example
 ```ts
 readStringList({ key: 'trustPolicyExclude', value: ['ring@0.17.14',], issues, });
 ```
 */
function readStringList(
  {
    key,
    value,
    issues,
  }: {
    readonly key: string;
    readonly value: unknown;
    readonly issues: RustCatalogIssue[];
  },
): readonly string[] {
  if (value === undefined)
    return [];
  if (!Array.isArray(value,)) {
    issues.push({
      location: key,
      problem: 'is not a list of strings',
      remedy: `write \`${key} = ["name", "name@1.2.3"]\``,
    },);
    return [];
  }
  /**
   Items that are strings, in order.
   */
  const strings = value.filter(function isString(item: unknown,): item is string {
    return (typeof item) === 'string';
  },);
  if (strings.length !== value.length) {
    issues.push({
      location: key,
      problem: 'is not a list of strings',
      remedy: `write \`${key} = ["name", "name@1.2.3"]\``,
    },);
    return [];
  }
  return strings;
}

/**
 Reads a boolean setting.

 @param key - setting name

 @param value - materialized TOML value

 @param fallback - pnpm's documented default

 @param issues - collector the problems are appended to

 @returns the boolean, or `fallback` when absent or invalid

 @example
 ```ts
 readBoolean({ key: 'catalogPrune', value: true, fallback: false, issues, });
 ```
 */
function readBoolean(
  {
    key,
    value,
    fallback,
    issues,
  }: {
    readonly key: string;
    readonly value: unknown;
    readonly fallback: boolean;
    readonly issues: RustCatalogIssue[];
  },
): boolean {
  if (value === undefined)
    return fallback;
  if ((typeof value) !== 'boolean') {
    issues.push({
      location: key,
      problem: 'is not a boolean',
      remedy: `write \`${key} = true\` or \`${key} = false\``,
    },);
    return fallback;
  }
  return value;
}

/**
 Reads a non-negative whole number of minutes.

 @param key - setting name

 @param value - materialized TOML value

 @param issues - collector the problems are appended to

 @returns the minutes, or {@link NO_MINUTES} when absent or invalid

 @example
 ```ts
 readMinutes({ key: 'minimumReleaseAge', value: 1440, issues, }); // 1440
 ```
 */
function readMinutes(
  {
    key,
    value,
    issues,
  }: {
    readonly key: string;
    readonly value: unknown;
    readonly issues: RustCatalogIssue[];
  },
): number | typeof NO_MINUTES {
  if (value === undefined)
    return NO_MINUTES;
  /**
   Numeric form of the setting.
   */
  const minutes = toNumber(value,);
  if ((minutes === NOT_A_NUMBER) || (!Number.isInteger(minutes,))
    || (minutes < 0)) {
    issues.push({
      location: key,
      problem: 'is not a non-negative whole number of minutes',
      remedy: `write \`${key} = 1440\``,
    },);
    return NO_MINUTES;
  }
  return minutes;
}

/**
 Reads the trust policy mode.

 @param value - materialized TOML value

 @param issues - collector the problems are appended to

 @returns the mode, `off` when absent (pnpm's default)

 @example
 ```ts
 readTrustPolicy({ value: 'no-downgrade', issues, });
 ```
 */
function readTrustPolicy(
  {
    value,
    issues,
  }: {
    readonly value: unknown;
    readonly issues: RustCatalogIssue[];
  },
): RustCatalogTrustPolicy {
  if (value === undefined)
    return 'off';
  if ((value === 'no-downgrade') || (value === 'off'))
    return value;
  issues.push({
    location: 'trustPolicy',
    problem: `${JSON.stringify(value,)} is not a trust policy`,
    remedy: 'write `trustPolicy = "no-downgrade"` or `trustPolicy = "off"`',
  },);
  return 'off';
}

//endregion Readers

//region Settings

/**
 Reads every catalog and supply-chain setting with pnpm's names and defaults.

 `minimumReleaseAgeStrict` defaults to `true` only when `minimumReleaseAge`
 is set explicitly, as pnpm documents.

 @param root - materialized top-level table

 @param issues - collector the problems are appended to

 @returns settings with defaults applied

 @example
 ```ts
 const settings = readSettings({ root: { minimumReleaseAge: 1440, }, issues, });
 ```
 */
export function readSettings(
  {
    root,
    issues,
  }: {
    readonly root: Readonly<Record<string, unknown>>;
    readonly issues: RustCatalogIssue[];
  },
): RustCatalogSettings {
  /**
   Explicit minimum release age.
   */
  const explicitAge = readMinutes({
    key: 'minimumReleaseAge',
    value: root.minimumReleaseAge,
    issues,
  },);
  /**
   Explicit trust-policy age cutoff.
   */
  const ignoreAfter = readMinutes({
    key: 'trustPolicyIgnoreAfter',
    value: root.trustPolicyIgnoreAfter,
    issues,
  },);
  return {
    catalogPrune: readBoolean({
      key: 'catalogPrune',
      value: root.catalogPrune,
      fallback: false,
      issues,
    },),
    minimumReleaseAge: explicitAge === NO_MINUTES ? DEFAULT_MINIMUM_RELEASE_AGE : explicitAge,
    minimumReleaseAgeExclude: readStringList({
      key: 'minimumReleaseAgeExclude',
      value: root.minimumReleaseAgeExclude,
      issues,
    },),
    minimumReleaseAgeExcludePrune: readBoolean({
      key: 'minimumReleaseAgeExcludePrune',
      value: root.minimumReleaseAgeExcludePrune,
      fallback: false,
      issues,
    },),
    minimumReleaseAgeIgnoreMissingTime: readBoolean({
      key: 'minimumReleaseAgeIgnoreMissingTime',
      value: root.minimumReleaseAgeIgnoreMissingTime,
      fallback: true,
      issues,
    },),
    minimumReleaseAgeStrict: readBoolean({
      key: 'minimumReleaseAgeStrict',
      value: root.minimumReleaseAgeStrict,
      fallback: explicitAge !== NO_MINUTES,
      issues,
    },),
    trustPolicy: readTrustPolicy({
      value: root.trustPolicy,
      issues,
    },),
    trustPolicyExclude: readStringList({
      key: 'trustPolicyExclude',
      value: root.trustPolicyExclude,
      issues,
    },),
    trustPolicyExcludePrune: readBoolean({
      key: 'trustPolicyExcludePrune',
      value: root.trustPolicyExcludePrune,
      fallback: false,
      issues,
    },),
    ...(ignoreAfter === NO_MINUTES ? {} : { trustPolicyIgnoreAfter: ignoreAfter, }),
  };
}

//endregion Settings
