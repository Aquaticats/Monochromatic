/**
 Content-based parsing for pnpm workspace catalog blocks.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { parse as parseYaml, } from 'yaml';

import {
  isValidPackageName,
} from './package-name.ts';
import type {
  CatalogDocument,
  CatalogEntry,
  CatalogMap,
  FlattenCatalogEntriesOptions,
} from './types.ts';

//region Parser state

/**
 Logger used for rejected catalog entries and malformed optional blocks.
 */
const logger = tagged({ tag: 'pnpm-workspace-catalog', },);

/**
 Reports whether `value` is a non-array object suitable for YAML mappings.
 
 @param value - parsed YAML value to inspect
 
 @returns whether `value` can be read as a string-keyed mapping
 
 @example
 ```ts
 isRecord({ catalog: {}, }); // true
 isRecord([]); // false
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Creates a map with no inherited properties.
 
 @returns empty null-prototype string map
 
 @example
 ```ts
 const map = createCatalogMap();
 map.oxlint = '>=1.71.0';
 ```
 */
function createCatalogMap(): Record<string, string> {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Object.create(null) is the deliberate null-prototype catalog container
  return Object.create(null,) as Record<string, string>;
}

/**
 Creates a null-prototype map for named catalog blocks.
 
 @returns empty null-prototype named-catalog map
 
 @example
 ```ts
 const catalogs = createNamedCatalogMap();
 catalogs.legacy = { react: '^18.0.0', };
 ```
 */
function createNamedCatalogMap(): Record<string, CatalogMap> {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Object.create(null) is the deliberate null-prototype named-catalog container
  return Object.create(null,) as Record<string, CatalogMap>;
}

/**
 Logs why one catalog mapping was omitted from the safe result.
 
 @param blockName - catalog block label
 
 @param key - rejected catalog key
 
 @param reason - reason for rejection
 
 @example
 ```ts
 warnRejectedEntry({ blockName: 'catalog', key: '__proto__', reason: 'invalid package name', });
 ```
 */
function warnRejectedEntry(
  {
    blockName,
    key,
    reason,
  }: {
    readonly blockName: string;
    readonly key: string;
    readonly reason: string;
  },
): void {
  logger.warn(
    `Rejected ${blockName} entry ${JSON.stringify(key,)}: ${reason}; skipping.`,
  );
}

/**
 Logs why a catalog block could not be read as a YAML mapping.
 
 @param blockName - block label used in the diagnostic
 
 @example
 ```ts
 warnMalformedBlock('catalogs');
 ```
 */
function warnMalformedBlock(blockName: string,): void {
  logger.warn(
    `Ignoring ${blockName}: expected a YAML mapping.`,
  );
}

//endregion Parser state

//region Block parsing

/**
 Copies one YAML catalog mapping into a validated null-prototype map.
 
 @param blockName - YAML block label
 
 @param value - parsed YAML block value
 
 @returns validated catalog map
 
 @mutates value - `Object.entries` may invoke own-property getters or proxy enumeration traps
 on parsed block
 
 @example
 ```ts
 const catalog = parseBlock({ blockName: 'catalog', value: { oxlint: '>=1.71.0', }, });
 ```
 */
function parseBlock(
  {
    blockName,
    value,
  }: {
    readonly blockName: string;
    readonly value: unknown;
  },
): CatalogMap {
  /**
   Safe result map populated only after key and value checks.
   */
  const result = createCatalogMap();

  if ((value === undefined) || (value === null))
    return result;
  if (!isRecord(value,)) {
    warnMalformedBlock(blockName,);
    return result;
  }

  /**
   YAML entries copied into the validated result map.
   */
  const entries = Object.entries(value,);
  entries.forEach(function copyEntry([key, rawValue,],): void {
    if (!isValidPackageName(key,)) {
      warnRejectedEntry({
        blockName,
        key,
        reason: 'not a valid npm package name',
      },);
      return;
    }
    if ((typeof rawValue) !== 'string') {
      warnRejectedEntry({
        blockName,
        key,
        reason: 'catalog values must be strings',
      },);
      return;
    }
    result[key] = rawValue;
  },);

  return result;
}

/**
 Creates the empty parsed document used when no catalog blocks are declared.
 
 @returns empty safe catalog document
 
 @example
 ```ts
 const document = createEmptyCatalogDocument();
 expect(document.defaultCatalog).toEqual({});
 ```
 */
function createEmptyCatalogDocument(): CatalogDocument {
  return {
    defaultCatalog: createCatalogMap(),
    namedCatalogs: createNamedCatalogMap(),
  };
}

//endregion Block parsing

//region Public parser

/**
 Parses default and named catalog blocks from raw `pnpm-workspace.yaml` text.
 
 YAML syntax errors are propagated by the `yaml` library. Missing or empty
 catalog blocks produce empty safe maps. Invalid package keys and non-string
 values are warned about and skipped without discarding valid siblings.
 
 @param content - raw workspace YAML text
 
 @returns parsed default and named catalog blocks
 
 @example
 ```ts
 const document = parseCatalogFromYaml([
   'catalog:',
   "  'oxlint': '>=1.71.0'",
 ].join('\n',),);
 console.info(document.defaultCatalog.oxlint);
 ```
 */
export function parseCatalogFromYaml(content: string,): CatalogDocument {
  /**
   YAML document narrowed at runtime before catalog fields are read.
   */
  const parsed: unknown = parseYaml(content,);
  if (!isRecord(parsed,)) {
    if ((parsed !== null) && (parsed !== undefined))
      warnMalformedBlock('workspace document',);
    return createEmptyCatalogDocument();
  }

  /**
   Validated default catalog mapping.
   */
  const defaultCatalog = parseBlock({
    blockName: 'catalog',
    value: parsed.catalog,
  },);
  /**
   Safe outer map for named catalog mappings.
   */
  const namedCatalogs = createNamedCatalogMap();

  if ((parsed.catalogs === undefined) || (parsed.catalogs === null)) {
    return {
      defaultCatalog,
      namedCatalogs,
    };
  }
  if (!isRecord(parsed.catalogs,)) {
    warnMalformedBlock('catalogs',);
    return {
      defaultCatalog,
      namedCatalogs,
    };
  }

  /**
   Named YAML blocks copied into the validated outer map.
   */
  const namedEntries = Object.entries(parsed.catalogs,);
  for (const [name, value,] of namedEntries) {
    namedCatalogs[name] = parseBlock({
      blockName: `catalogs.${name}`,
      value,
    },);
  }

  return {
    defaultCatalog,
    namedCatalogs,
  };
}

/**
 Flattens parsed catalog maps into raw entries while preserving map order.
 
 Default entries are always returned first. Named entries are included only
 when `includeNamedCatalogs` is true, so default-only callers opt into the
 broader result explicitly.
 
 @param document - parsed catalog document
 
 @param includeNamedCatalogs - whether named blocks should be included
 
 @returns raw catalog entries in deterministic default-then-named order
 
 on catalog maps
 
 @example
 ```ts
 const entries = flattenCatalogEntries({
   document,
   includeNamedCatalogs: true,
 });
 ```
 */
export function flattenCatalogEntries(
  {
    document,
    includeNamedCatalogs = false,
  }: ForeignBorrowed<FlattenCatalogEntriesOptions>,
): readonly CatalogEntry[] {
  /**
   Default catalog entries without a catalog-name marker.
   */
  const defaultEntries = Object.entries(document.defaultCatalog,)
    .map(function toDefaultEntry([catalogKey, catalogValue,],): CatalogEntry {
      return {
        catalogKey,
        catalogValue,
      };
    },);
  if (!includeNamedCatalogs)
    return defaultEntries;

  /**
   Named catalog entries appended after the default block.
   */
  const namedEntries: CatalogEntry[] = [];
  for (const [catalogName, catalog,] of Object.entries(document.namedCatalogs,)) {
    for (const [catalogKey, catalogValue,] of Object.entries(catalog,)) {
      namedEntries.push({
        catalogKey,
        catalogName,
        catalogValue,
      },);
    }
  }
  return [
    ...defaultEntries,
    ...namedEntries,
  ];
}

//endregion Public parser
