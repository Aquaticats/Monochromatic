/**
 Adapts the shared pnpm workspace catalog reader to deps-cube's resolved
 entry shape.
 
 The shared reader preserves raw catalog values and supports both default and
 named blocks. deps-cube resolves `npm:` aliases locally because its probes
 need the actual registry package name.
 
 @example
 ```ts
 import { readCatalog } from './catalog.ts';
 const entries = await readCatalog();
 for (const entry of entries) {
   console.info(entry.catalogKey, entry.npmName, entry.range);
 }
 ```
 */

import {
  flattenCatalogEntries,
  isValidPackageName,
  readCatalogFile,
  type CatalogEntry as RawCatalogEntry,
} from '@monochromatic-dev/dependency-catalog-core-pnpm/ts';//region Types

/**
 Single resolved catalog entry from `pnpm-workspace.yaml`.
 */
export type CatalogEntry = {
  /**
   Key as written in the catalog, which may differ from `npmName` for an alias.
   */
  readonly catalogKey: string;
  /**
   Actual npm package name used by registry probes.
   */
  readonly npmName: string;
  /**
   Version range or selector after alias decoding.
   */
  readonly range: string;
  /**
   Name of the named catalog this entry came from, omitted for the default block.
   */
  readonly catalogName?: string;
};

//endregion Types

//region Alias decoding

/**
 Thrown when an `npm:` alias target is not a safe npm package name.
 */
class InvalidCatalogAliasError extends Error {
  /**
   @param catalogValue - raw alias value that was rejected
   
   @param target - extracted alias target
   */
  constructor(
    {
      catalogValue,
      target,
    }: {
      readonly catalogValue: string;
      readonly target: string;
    },
  ) {
    super(`Invalid npm alias target ${JSON.stringify(target,)} in ${JSON.stringify(catalogValue,)}`,);
    this.name = 'InvalidCatalogAliasError';
  }
}

/**
 Decodes a catalog value, handling `npm:<name>@<range>` aliases.
 
 @param key - catalog key
 
 @param value - raw catalog value
 
 @returns actual npm name and range for downstream probes
 
 @example
 ```ts
 decodeAlias({ key: 'local-alias', value: 'npm:aliased-target\@0.8.0' });
 // { npmName: 'aliased-target', range: '0.8.0' }
 decodeAlias({ key: 'preact', value: '^10.26.0' });
 // { npmName: 'preact', range: '^10.26.0' }
 ```
 */
export function decodeAlias(
  {
    key,
    value,
  }: {
    readonly key: string;
    readonly value: string;
  },
): {
  npmName: string;
  range: string;
} {
  if (!value.startsWith('npm:',)) {
    return {
      npmName: key,
      range: value,
    };
  }

  /**
   Alias body after removing the `npm:` marker.
   */
  const remainder = value.slice('npm:'.length,);
  /**
   Final at-sign separating an optional selector from the target package name.
   */
  const atIndex = remainder.lastIndexOf('@',);
  /**
   Alias target package name before the optional selector suffix.
   */
  const npmName = atIndex <= 0
    ? remainder
    : remainder.slice(
      0,
      atIndex,
    );
  if (!isValidPackageName(npmName,)) {
    throw new InvalidCatalogAliasError({
      catalogValue: value,
      target: npmName,
    },);
  }
  return {
    npmName,
    range: atIndex <= 0 ? '*' : remainder.slice(atIndex + 1,),
  };
}

//endregion Alias decoding

//region Public reader

/**
 Locates and parses `pnpm-workspace.yaml`, returning resolved entries from
 the default catalog and every named catalog.
 
 @param startDir - optional starting directory for workspace discovery
 
 @returns resolved catalog entries in default-then-named order
 
 @throws Error when the workspace file is missing or contains no entries
 
 @example
 ```ts
 const entries = await readCatalog();
 console.info(entries.length);
 ```
 */
export async function readCatalog(
  { startDir, }: { readonly startDir?: string; } = {},
): Promise<readonly CatalogEntry[]> {
  /**
   Located workspace file, using the default cwd search when no start directory was supplied.
   */
  const workspace = startDir === undefined
    ? await readCatalogFile()
    : await readCatalogFile({ startDir, },);
  /**
   Raw entries from both default and named blocks before alias decoding.
   */
  const rawEntries = flattenCatalogEntries({
    document: workspace.catalogs,
    includeNamedCatalogs: true,
  },);
  if (rawEntries.length
    === 0) {
    throw new Error(`No catalog or catalogs entries found in ${workspace.path}`,);
  }

  /**
   Resolved entries consumed by deps-cube's registry and install probes.
   */
  return rawEntries.map(function resolveEntry(entry: RawCatalogEntry,): CatalogEntry {
    /**
     Alias-decoded npm package name and selector.
     */
    const {
      npmName,
      range,
    } = decodeAlias({
      key: entry.catalogKey,
      value: entry.catalogValue,
    },);
    return {
      catalogKey: entry.catalogKey,
      npmName,
      range,
      ...(entry.catalogName === undefined ? {} : { catalogName: entry.catalogName, }),
    };
  },);
}

//endregion Public reader
