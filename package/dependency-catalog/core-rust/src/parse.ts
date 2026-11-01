/**
 Parsing and validation of `rust-catalog.toml`.

 Unlike the pnpm workspace reader, which warns and skips, this parser fails
 closed: the catalog gates CI and every crate manifest, so one invalid entry
 throws a {@link RustCatalogError} naming every problem at once.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  parseTomlEdit,
  tomlGetValue,
} from '@monochromatic-dev/toml-edit-core/ts';

import { RustCatalogError, } from './errors.ts';
import { parseCatalogTable, } from './parse-entry.ts';
import { readSettings, } from './parse-settings.ts';
import { isTable, } from './toml-values.ts';
import type {
  RustCatalogDocument,
  RustCatalogIssue,
  RustCatalogMap,
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
  'catalogPrune',
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

//endregion Shared state

//region Named catalogs

/**
 Validates the `[catalogs.<name>]` tables.

 @param value - materialized `catalogs` value

 @param issues - collector the problems are appended to

 @returns named catalogs in source order

 @example
 ```ts
 parseNamedCatalogs({ value: { legacy: { windows: '0.58', }, }, issues, });
 ```
 */
function parseNamedCatalogs(
  {
    value,
    issues,
  }: {
    readonly value: unknown;
    readonly issues: RustCatalogIssue[];
  },
): ReadonlyMap<string, RustCatalogMap> {
  /**
   Named catalogs accepted so far.
   */
  const catalogs = new Map<string, RustCatalogMap>();
  if (value === undefined)
    return catalogs;
  if (!isTable(value,)) {
    issues.push({
      location: 'catalogs',
      problem: 'is not a table',
      remedy: 'declare named catalogs as `[catalogs.<name>]`',
    },);
    return catalogs;
  }
  for (const [catalogName, table,] of Object.entries(value,)) {
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
      parseCatalogTable({
        location: `catalogs.${catalogName}`,
        value: table,
        bounded: true,
        issues,
      },),
    );
  }
  return catalogs;
}

//endregion Named catalogs

//region Public parser

/**
 Parses and validates `rust-catalog.toml` text.

 @param source - raw file text

 @param sourcePath - path used in diagnostics

 @returns validated catalogs and settings

 @throws RustCatalogError when any entry or setting is invalid, listing every problem

 @throws TomlEditError when the text is not valid TOML

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
  const log = tagged({
    tag: parseRustCatalog.name,
    l: logger,
  },);
  log.debug(`parsing ${sourcePath}`,);

  /**
   Whole document materialized as plain values.
   */
  const root = tomlGetValue({
    edit: parseTomlEdit({ source, },),
    path: [],
  },);
  if (!isTable(root,)) {
    throw new RustCatalogError({
      sourcePath,
      issues: [{
        location: '(file)',
        problem: 'is not a TOML table',
        remedy: 'start from `[catalog]`',
      },],
    },);
  }

  /**
   Problems found across the whole file.
   */
  const issues: RustCatalogIssue[] = [];
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
  const catalog = parseCatalogTable({
    location: 'catalog',
    value: root.catalog,
    bounded: false,
    issues,
  },);
  /**
   Named catalogs of bounded requirements.
   */
  const catalogs = parseNamedCatalogs({
    value: root.catalogs,
    issues,
  },);
  /**
   Supply-chain settings with defaults applied.
   */
  const settings = readSettings({
    root,
    issues,
  },);
  if (issues.length > 0) {
    log.error(`${sourcePath}: ${String(issues.length,)} problem(s)`,);
    throw new RustCatalogError({
      sourcePath,
      issues,
    },);
  }

  log.debug(`${sourcePath}: ${String(catalog.size,)} default entries, ${String(catalogs.size,)} named catalogs`,);
  return {
    catalog,
    catalogs,
    settings,
  };
}

//endregion Public parser
