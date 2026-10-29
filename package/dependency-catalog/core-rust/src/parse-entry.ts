/**
 Validation of catalog tables and their entries.

 @module
 */

import {
  isBoundedRequirement,
  isCrateName,
  isFloorRequirement,
} from './syntax.ts';
import {
  isNonEmptyString,
  isTable,
} from './toml-values.ts';
import type {
  RustCatalogEntry,
  RustCatalogIssue,
  RustCatalogMap,
} from './types.ts';

//region Rules

/**
 Sentinel for an entry rejected with at least one issue.
 */
const REJECTED_ENTRY: unique symbol = Symbol('rust-catalog.toml entry rejected with validation issues',);

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

/**
 Requirement rule for registry versions in the default catalog.
 */
const FLOOR_RULE = {
  accepts: isFloorRequirement,
  remedy: 'write a `>=` floor such as `">=1.0.104"`; bounded versions belong in a named catalog',
} as const;

/**
 Requirement rule for registry versions in named catalogs.
 */
const BOUNDED_RULE = {
  accepts: isBoundedRequirement,
  remedy:
    'named catalogs hold bounded requirements such as `"0.58"`, `"^0.58"`, `"~0.58.1"`, or `"=0.58.0"`; a `>=` floor would be pulled to the newest release',
} as const;

//endregion Rules

//region Entry checks

/**
 Reports table keys that are unknown or belong at the use site.

 @param location - dotted path used in diagnostics

 @param table - entry table

 @param issues - collector the problems are appended to

 @example
 ```ts
 reportForeignKeys({ location: 'catalog.tokio', table: { features: [], }, issues, });
 ```
 */
function reportForeignKeys(
  {
    location,
    table,
    issues,
  }: {
    readonly location: string;
    readonly table: Readonly<Record<string, unknown>>;
    readonly issues: RustCatalogIssue[];
  },
): void {
  for (const key of Object.keys(table,)) {
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
}

/**
 Validates a table-form entry.

 @param location - dotted path used in diagnostics

 @param table - entry table

 @param rule - requirement rule of the enclosing catalog

 @param issues - collector the problems are appended to

 @returns validated entry, or {@link REJECTED_ENTRY} when the table has problems

 @example
 ```ts
 parseTableEntry({ location: 'catalog.x', table: { version: '>=1', }, rule: FLOOR_RULE, issues, });
 ```
 */
function parseTableEntry(
  {
    location,
    table,
    rule,
    issues,
  }: {
    readonly location: string;
    readonly table: Readonly<Record<string, unknown>>;
    readonly rule: typeof FLOOR_RULE | typeof BOUNDED_RULE;
    readonly issues: RustCatalogIssue[];
  },
): RustCatalogEntry | typeof REJECTED_ENTRY {
  /**
   Issue count before this table, so it is rejected when it adds any.
   */
  const issuesBefore = issues.length;
  reportForeignKeys({
    location,
    table,
    issues,
  },);

  /**
   Optional rename target.
   */
  const packageName = table.package;
  if ((packageName !== undefined) && (!(((typeof packageName) === 'string') && isCrateName(packageName,)))) {
    issues.push({
      location,
      problem: '`package` is not a valid crate name',
      remedy: 'set `package` to the crate name on crates.io that this dependency renames',
    },);
  }
  /**
   Rename fields spread into the result; empty when there is no rename.
   */
  const rename = (typeof packageName) === 'string' ? { package: packageName, } : {};

  /**
   Git repository URL, when the entry is a git pin.
   */
  const {git} = table;
  /**
   Pinned commit of a git entry.
   */
  const {rev} = table;
  /**
   Registry version requirement.
   */
  const {version} = table;

  if (git !== undefined) {
    if (!isNonEmptyString(git,)) {
      issues.push({
        location,
        problem: '`git` is not a non-empty string',
        remedy: 'set `git` to the repository URL',
      },);
    }
    if (!isNonEmptyString(rev,)) {
      issues.push({
        location,
        problem: 'git entry has no `rev`',
        remedy: 'pin the entry with `rev = "<full commit hash>"`',
      },);
    }
    if (version !== undefined) {
      issues.push({
        location,
        problem: 'git entry also declares `version`',
        remedy: 'remove `version` from git entries',
      },);
    }
    if ((issues.length !== issuesBefore) || (!isNonEmptyString(git,))
      || (!isNonEmptyString(rev,)))
      return REJECTED_ENTRY;
    return {
      kind: 'git',
      git,
      rev,
      ...rename,
    };
  }

  if (rev !== undefined) {
    issues.push({
      location,
      problem: 'declares `rev` without `git`',
      remedy: 'add `git` or remove `rev`',
    },);
  }
  if (!(((typeof version) === 'string') && rule.accepts(version,))) {
    issues.push({
      location,
      problem: `version requirement ${JSON.stringify(version,)} is not accepted here`,
      remedy: rule.remedy,
    },);
  }
  if ((issues.length !== issuesBefore) || ((typeof version) !== 'string'))
    return REJECTED_ENTRY;
  return {
    kind: 'registry',
    version,
    ...rename,
  };
}

/**
 Validates one entry, reporting problems instead of throwing.

 @param location - dotted path used in diagnostics, e.g. `catalog.tokio`

 @param name - dependency name the entry is keyed by

 @param value - materialized TOML value

 @param bounded - whether registry versions must be bounded (named catalogs) instead of `>=` floors

 @param issues - collector the problems are appended to

 @returns validated entry, or {@link REJECTED_ENTRY} when it has problems

 @example
 ```ts
 parseEntry({ location: 'catalog.anyhow', name: 'anyhow', value: '>=1', bounded: false, issues, });
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
): RustCatalogEntry | typeof REJECTED_ENTRY {
  /**
   Whether the key is a usable dependency name.
   */
  const validName = isCrateName(name,);
  if (!validName) {
    issues.push({
      location,
      problem: `${JSON.stringify(name,)} is not a valid Cargo dependency name`,
      remedy: 'use 1 to 64 ASCII letters, digits, `-`, or `_`, starting with a letter',
    },);
  }
  /**
   Requirement rule of the enclosing catalog.
   */
  const rule = bounded ? BOUNDED_RULE : FLOOR_RULE;

  if ((typeof value) === 'string') {
    if (!rule.accepts(value,)) {
      issues.push({
        location,
        problem: `version requirement ${JSON.stringify(value,)} is not accepted here`,
        remedy: rule.remedy,
      },);
      return REJECTED_ENTRY;
    }
    return validName
      ? {
        kind: 'registry',
        version: value,
      }
      : REJECTED_ENTRY;
  }

  if (!isTable(value,)) {
    issues.push({
      location,
      problem: 'is neither a version string nor an inline table',
      remedy: 'write `name = ">=1.2.3"` or `name = { version = ">=1.2.3", package = "real-name" }`',
    },);
    return REJECTED_ENTRY;
  }

  /**
   Table entry result, rejected when the name was already invalid.
   */
  const entry = parseTableEntry({
    location,
    table: value,
    rule,
    issues,
  },);
  return validName ? entry : REJECTED_ENTRY;
}

//endregion Entry checks

//region Tables

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
export function parseCatalogTable(
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
    issues.push({
      location,
      problem: 'is not a table',
      remedy: `declare it as \`[${location}]\``,
    },);
    return entries;
  }
  for (const [name, entryValue,] of Object.entries(value,)) {
    /**
     Validated entry, or the rejection sentinel.
     */
    const entry = parseEntry({
      location: `${location}.${name}`,
      name,
      value: entryValue,
      bounded,
      issues,
    },);
    if (entry !== REJECTED_ENTRY)
      entries.set(
        name,
        entry,
      );
  }
  return entries;
}

//endregion Tables
