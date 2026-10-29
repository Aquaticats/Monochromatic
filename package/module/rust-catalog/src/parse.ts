/**
 Parsing and validation of `rust-catalog.toml`.

 Unlike the pnpm workspace reader, which warns and skips, this parser fails
 closed: the catalog gates CI and every crate manifest, so one invalid entry
 throws a {@link RustCatalogError} naming every problem at once.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  parseTomlEdit,
  tomlGetValue,
} from '@monochromatic-dev/module-toml-edit/ts';

import { RustCatalogError, } from './errors.ts';
import {
  isBoundedRequirement,
  isCrateName,
  isFloorRequirement,
} from './syntax.ts';
import type {
  RustCatalogDocument,
  RustCatalogEntry,
  RustCatalogIssue,
  RustCatalogMap,
  RustCatalogSettings,
  RustCatalogTrustPolicy,
} from './types.ts';

//region Shared state

/**
 Module logger; functions re-tag it with their own name.
 */
const logger = tagged({ tag: 'rust-catalog', },);

/**
 Top-level keys the catalog file may declare.
 */
const TOP_LEVEL_KEYS: ReadonlySet<string> = new Set([
  'catalog',
  'catalogs',
  'minimumReleaseAge',
  'minimumReleaseAgeExclude',
  'minimumReleaseAgeExcludePrune',
  'minimumReleaseAgeIgnoreMissingTime',
  'minimumReleaseAgeStrict',
  'trustPolicy',
  'trustPolicyExclude',
  'trustPolicyExcludePrune',
  'trustPolicyIgnoreAfter',
],);

/**
 Entry keys that belong at a crate's use site, with the remedy for each.
 */
const USE_SITE_KEYS: ReadonlyMap<string, string> = new Map([
  [
    'features',
    'move `features` to each crate that needs them; Cargo adds use-site features on top of the inherited entry',
  ],
  [
    'optional',
    'move `optional = true` to the crate that makes the dependency optional',
  ],
  [
    'default-features',
    'move `default-features = false` to the crate that disables default features',
  ],
  [
    'path',
    'depend on internal crates by `path` in the crate manifest; internal crates stay out of the catalog',
  ],
  [
    'branch',
    'pin git entries with `rev = "<commit>"` instead of `branch`',
  ],
  [
    'tag',
    'pin git entries with `rev = "<commit>"` instead of `tag`',
  ],
  [
    'workspace',
    'remove `workspace`; catalog entries are the source the crates inherit from',
  ],
],);

/**
 Keys a table-form entry may declare.
 */
const ENTRY_KEYS: ReadonlySet<string> = new Set([
  'git',
  'package',
  'rev',
  'version',
],);

//endregion Shared state

//region Value helpers

/**
 Whether `value` is a plain string-keyed table.

 @param value - materialized TOML value

 @returns whether it can be read as a table

 @example
 ```ts
 isTable({ version: '>=1', }); // true
 ```
 */
function isTable(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null) && !Array.isArray(value,);
}

/**
 Reads a TOML integer or float as a JavaScript number.

 @param value - materialized TOML value

 @returns number, or `undefined` when the value is not numeric

 @example
 ```ts
 toNumber(1440); // 1440
 ```
 */
function toNumber(value: unknown,): number | undefined {
  if ((typeof value) === 'number')
    return value as number;
  if ((typeof value) === 'bigint')
    return Number(value as bigint,);
  return undefined;
}

//endregion Value helpers

//region Entries

/**
 Validates one entry and reports problems instead of throwing.

 @param location - dotted path used in diagnostics, e.g. `catalog.tokio`

 @param name - dependency name the entry is keyed by

 @param value - materialized TOML value

 @param bounded - whether registry versions must be bounded (named catalogs) instead of `>=` floors

 @param issues - collector the problems are appended to

 @returns validated entry, or `undefined` when it has problems

 @example
 ```ts
 const entry = parseEntry({ location: 'catalog.anyhow', name: 'anyhow', value: '>=1', bounded: false, issues, });
 ```
 */
function parseEntry(
  {
    location,
    name,
    value,
    bounded,
    issues,
  }: {
    readonly location: string;
    readonly name: string;
    readonly value: unknown;
    readonly bounded: boolean;
    readonly issues: RustCatalogIssue[];
  },
): RustCatalogEntry | undefined {
  /**
   Issue count before this entry, so the entry is rejected when it adds any.
   */
  const issuesBefore = issues.length;
  if (!isCrateName(name,)) {
    issues.push({
      location,
      problem: `${JSON.stringify(name,)} is not a valid Cargo dependency name`,
      remedy: 'use 1 to 64 ASCII letters, digits, `-`, or `_`, starting with a letter',
    },);
  }

  /**
   Requirement rule for registry versions in this catalog.
   */
  const versionRule = bounded
    ? {
      accepts: isBoundedRequirement,
      remedy:
        'named catalogs hold bounded requirements such as `"0.58"`, `"^0.58"`, `"~0.58.1"`, or `"=0.58.0"`; a `>=` floor would be pulled to the newest release',
    }
    : {
      accepts: isFloorRequirement,
      remedy: 'write a `>=` floor such as `">=1.0.104"`; bounded versions belong in a named catalog',
    };

  if ((typeof value) === 'string') {
    if (!versionRule.accepts(value as string,)) {
      issues.push({
        location,
        problem: `version requirement ${JSON.stringify(value,)} is not accepted here`,
        remedy: versionRule.remedy,
      },);
      return undefined;
    }
    return issues.length === issuesBefore
      ? { kind: 'registry', version: value as string, }
      : undefined;
  }

  if (!isTable(value,)) {
    issues.push({
      location,
      problem: 'is neither a version string nor an inline table',
      remedy: 'write `name = ">=1.2.3"` or `name = { version = ">=1.2.3", package = "real-name" }`',
    },);
    return undefined;
  }

  for (const key of Object.keys(value,)) {
    /**
     Remedy when the key belongs at the use site instead.
     */
    const useSiteRemedy = USE_SITE_KEYS.get(key,);
    if (useSiteRemedy !== undefined) {
      issues.push({
        location,
        problem: `declares \`${key}\``,
        remedy: useSiteRemedy,
      },);
    } else if (!ENTRY_KEYS.has(key,)) {
      issues.push({
        location,
        problem: `declares unknown key \`${key}\``,
        remedy: 'use only `version`, `package`, `git`, and `rev`',
      },);
    }
  }

  /**
   Optional rename target, validated like a crate name.
   */
  const packageName = value.package;
  if ((packageName !== undefined) && (((typeof packageName) !== 'string') || !isCrateName(packageName as string,))) {
    issues.push({
      location,
      problem: '`package` is not a valid crate name',
      remedy: 'set `package` to the crate name on crates.io that this dependency renames',
    },);
  }

  if (value.git !== undefined) {
    if (((typeof value.git) !== 'string') || ((value.git as string).length === 0)) {
      issues.push({ location, problem: '`git` is not a non-empty string', remedy: 'set `git` to the repository URL', },);
    }
    if (((typeof value.rev) !== 'string') || ((value.rev as string).length === 0)) {
      issues.push({
        location,
        problem: 'git entry has no `rev`',
        remedy: 'pin the entry with `rev = "<full commit hash>"`',
      },);
    }
    if (value.version !== undefined) {
      issues.push({ location, problem: 'git entry also declares `version`', remedy: 'remove `version` from git entries', },);
    }
    if (issues.length !== issuesBefore)
      return undefined;
    return {
      kind: 'git',
      git: value.git as string,
      rev: value.rev as string,
      ...(packageName === undefined ? {} : { package: packageName as string, }),
    };
  }

  if (value.rev !== undefined) {
    issues.push({ location, problem: 'declares `rev` without `git`', remedy: 'add `git` or remove `rev`', },);
  }
  if (((typeof value.version) !== 'string') || !versionRule.accepts(value.version as string,)) {
    issues.push({
      location,
      problem: `version requirement ${JSON.stringify(value.version,)} is not accepted here`,
      remedy: versionRule.remedy,
    },);
  }
  if (issues.length !== issuesBefore)
    return undefined;
  return {
    kind: 'registry',
    version: value.version as string,
    ...(packageName === undefined ? {} : { package: packageName as string, }),
  };
}

/**
 Validates one catalog table into an ordered map.

 @param location - dotted path of the table, e.g. `catalogs.windows-legacy`

 @param value - materialized TOML value of the table

 @param bounded - whether registry versions must be bounded

 @param issues - collector the problems are appended to

 @returns validated entries in source order

 @example
 ```ts
 const catalog = parseCatalogTable({ location: 'catalog', value: { anyhow: '>=1', }, bounded: false, issues, });
 ```
 */
function parseCatalogTable(
  {
    location,
    value,
    bounded,
    issues,
  }: {
    readonly location: string;
    readonly value: unknown;
    readonly bounded: boolean;
    readonly issues: RustCatalogIssue[];
  },
): RustCatalogMap {
  /**
   Entries accepted so far, in source order.
   */
  const entries = new Map<string, RustCatalogEntry>();
  if (value === undefined)
    return entries;
  if (!isTable(value,)) {
    issues.push({ location, problem: 'is not a table', remedy: `declare it as \`[${location}]\``, },);
    return entries;
  }
  for (const [name, entryValue,] of Object.entries(value,)) {
    /**
     Validated entry, absent when it had problems.
     */
    const entry = parseEntry({
      location: `${location}.${name}`,
      name,
      value: entryValue,
      bounded,
      issues,
    },);
    if (entry !== undefined)
      entries.set(name, entry,);
  }
  return entries;
}

//endregion Entries

//region Settings

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
  if (!Array.isArray(value,) || !value.every(function isString(item: unknown,): boolean {
    return (typeof item) === 'string';
  },)) {
    issues.push({ location: key, problem: 'is not a list of strings', remedy: `write \`${key} = ["name", "name@1.2.3"]\``, },);
    return [];
  }
  return value as readonly string[];
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
 readBoolean({ key: 'trustPolicyExcludePrune', value: true, fallback: false, issues, });
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
    issues.push({ location: key, problem: 'is not a boolean', remedy: `write \`${key} = true\` or \`${key} = false\``, },);
    return fallback;
  }
  return value as boolean;
}

/**
 Reads a non-negative minute count.

 @param key - setting name

 @param value - materialized TOML value

 @param issues - collector the problems are appended to

 @returns the minutes, or `undefined` when absent or invalid

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
): number | undefined {
  if (value === undefined)
    return undefined;
  /**
   Numeric form of the setting.
   */
  const minutes = toNumber(value,);
  if ((minutes === undefined) || !Number.isInteger(minutes,) || (minutes < 0)) {
    issues.push({ location: key, problem: 'is not a non-negative whole number of minutes', remedy: `write \`${key} = 1440\``, },);
    return undefined;
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

/**
 pnpm's documented default for `minimumReleaseAge`, in minutes.
 */
const DEFAULT_MINIMUM_RELEASE_AGE = 1440;

/**
 Reads every supply-chain setting with pnpm's names and defaults.

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
function readSettings(
  {
    root,
    issues,
  }: {
    readonly root: Readonly<Record<string, unknown>>;
    readonly issues: RustCatalogIssue[];
  },
): RustCatalogSettings {
  /**
   Explicit minimum release age, absent when the file leaves it to the default.
   */
  const explicitAge = readMinutes({ key: 'minimumReleaseAge', value: root.minimumReleaseAge, issues, },);
  /**
   Explicit trust-policy age cutoff.
   */
  const ignoreAfter = readMinutes({ key: 'trustPolicyIgnoreAfter', value: root.trustPolicyIgnoreAfter, issues, },);
  return {
    minimumReleaseAge: explicitAge ?? DEFAULT_MINIMUM_RELEASE_AGE,
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
      fallback: explicitAge !== undefined,
      issues,
    },),
    trustPolicy: readTrustPolicy({ value: root.trustPolicy, issues, },),
    trustPolicyExclude: readStringList({ key: 'trustPolicyExclude', value: root.trustPolicyExclude, issues, },),
    trustPolicyExcludePrune: readBoolean({
      key: 'trustPolicyExcludePrune',
      value: root.trustPolicyExcludePrune,
      fallback: false,
      issues,
    },),
    ...(ignoreAfter === undefined ? {} : { trustPolicyIgnoreAfter: ignoreAfter, }),
  };
}

//endregion Settings

//region Public parser

/**
 Parses and validates `rust-catalog.toml` text.

 @param source - raw file text

 @param sourcePath - path used in diagnostics

 @returns validated catalogs and settings

 @throws {RustCatalogError} when any entry or setting is invalid, listing every problem

 @throws {TomlEditError} when the text is not valid TOML

 @example
 ```ts
 const document = parseRustCatalog({
   source: '[catalog]\nanyhow = ">=1.0.104"\n',
   sourcePath: 'rust-catalog.toml',
 });
 document.catalog.get('anyhow'); // { kind: 'registry', version: '>=1.0.104' }
 ```
 */
export function parseRustCatalog(
  {
    source,
    sourcePath,
  }: {
    readonly source: string;
    readonly sourcePath: string;
  },
): RustCatalogDocument {
  /**
   Logger tagged for this call.
   */
  const log = tagged({ tag: parseRustCatalog.name, l: logger, },);
  log.debug(`parsing ${sourcePath}`,);

  /**
   Whole document materialized as plain values.
   */
  const root = tomlGetValue({ edit: parseTomlEdit({ source, },), path: [], },);
  /**
   Problems found across the whole file.
   */
  const issues: RustCatalogIssue[] = [];
  if (!isTable(root,))
    throw new RustCatalogError({ sourcePath, issues: [{ location: '(file)', problem: 'is not a TOML table', remedy: 'start from `[catalog]`', },], },);

  for (const key of Object.keys(root,)) {
    if (!TOP_LEVEL_KEYS.has(key,)) {
      issues.push({
        location: key,
        problem: 'is not a catalog table or a supported setting',
        remedy: `remove it; supported keys are ${[...TOP_LEVEL_KEYS,].join(', ',)}`,
      },);
    }
  }

  /**
   Default catalog of `>=` floors and git pins.
   */
  const catalog = parseCatalogTable({ location: 'catalog', value: root.catalog, bounded: false, issues, },);
  /**
   Named catalogs of bounded requirements.
   */
  const catalogs = new Map<string, RustCatalogMap>();
  if (root.catalogs !== undefined) {
    if (isTable(root.catalogs,)) {
      for (const [catalogName, value,] of Object.entries(root.catalogs,)) {
        if (catalogName === 'default') {
          issues.push({
            location: 'catalogs.default',
            problem: 'repeats the default catalog',
            remedy: 'put default entries under `[catalog]`',
          },);
          continue;
        }
        catalogs.set(
          catalogName,
          parseCatalogTable({ location: `catalogs.${catalogName}`, value, bounded: true, issues, },),
        );
      }
    } else {
      issues.push({ location: 'catalogs', problem: 'is not a table', remedy: 'declare named catalogs as `[catalogs.<name>]`', },);
    }
  }

  /**
   Supply-chain settings with defaults applied.
   */
  const settings = readSettings({ root, issues, },);
  if (issues.length > 0) {
    log.error(`${sourcePath}: ${String(issues.length,)} problem(s)`,);
    throw new RustCatalogError({ sourcePath, issues, },);
  }

  log.debug(`${sourcePath}: ${String(catalog.size,)} default entries, ${String(catalogs.size,)} named catalogs`,);
  return { catalog, catalogs, settings, };
}

//endregion Public parser
