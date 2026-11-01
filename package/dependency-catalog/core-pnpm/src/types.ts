/**
 Public data types for parsed `pnpm-workspace.yaml` catalog blocks.
 
 @module
 */

//region Catalog shapes

/**
 Prototype-safe mapping from package name to its raw catalog value.
 
 @example
 ```ts
 const catalog: CatalogMap = { oxlint: '>=1.71.0', };
 ```
 */
export type CatalogMap = Readonly<Record<string, string>>;

/**
 Parsed default and named catalog blocks.
 
 @example
 ```ts
 const document: CatalogDocument = {
   defaultCatalog: { oxlint: '>=1.71.0', },
   namedCatalogs: { legacy: { react: '^18.0.0', }, },
 };
 ```
 */
export type CatalogDocument = {
  /**
   Entries from the default `catalog:` block.
   */
  readonly defaultCatalog: CatalogMap;
  /**
   Entries grouped by name from the `catalogs:` block.
   */
  readonly namedCatalogs: Readonly<Record<string, CatalogMap>>;
};

/**
 One raw catalog entry before consumer-specific alias handling.
 
 @example
 ```ts
 const entry: CatalogEntry = {
   catalogKey: 'zod',
   catalogValue: 'npm:\@jsr/zod__zod@>=4.1.8',
 };
 ```
 */
export type CatalogEntry = {
  /**
   Package name as written in the catalog map.
   */
  readonly catalogKey: string;
  /**
   Full scalar value as written in YAML, including any `npm:` alias.
   */
  readonly catalogValue: string;
  /**
   Named catalog containing the entry, omitted for the default catalog.
   */
  readonly catalogName?: string;
};

//endregion Catalog shapes

//region File reading

/**
 Parsed workspace file returned by {@link readCatalogFile}.
 
 @example
 ```ts
 const result = await readCatalogFile();
 console.info(result.path, result.content.length);
 ```
 */
export type CatalogFile = {
  /**
   Absolute path to the located `pnpm-workspace.yaml`.
   */
  readonly path: string;
  /**
   Original UTF-8 file content, retained for formatting-preserving writers.
   */
  readonly content: string;
  /**
   Parsed catalog blocks from {@link content}.
   */
  readonly catalogs: CatalogDocument;
};

/**
 Options for locating `pnpm-workspace.yaml`.
 
 @example
 ```ts
 const result = await readCatalogFile({ startDir: '/workspace/packages/app', });
 ```
 */
export type ReadCatalogFileOptions = {
  /**
   Directory from which the upward search starts; defaults to the process cwd.
   */
  readonly startDir?: string;
};

/**
 Options for flattening parsed catalog blocks into raw entries.
 
 @example
 ```ts
 const entries = flattenCatalogEntries({
   document,
   includeNamedCatalogs: true,
 });
 ```
 */
export type FlattenCatalogEntriesOptions = {
  /**
   Parsed catalog document to flatten.
   */
  readonly document: CatalogDocument;
  /**
   Whether named `catalogs:` entries should follow default entries.
   */
  readonly includeNamedCatalogs?: boolean;
};

//endregion File reading
