/**
 Safe parsing and discovery for pnpm workspace catalog blocks.
 
 @example
 ```ts
 import {
   flattenCatalogEntries,
   readCatalogFile,
 } from '@monochromatic-dev/dependency-catalog-core-pnpm';
 
 const workspace = await readCatalogFile();
 const entries = flattenCatalogEntries({
   document: workspace.catalogs,
   includeNamedCatalogs: true,
 });
 console.info(entries.length);
 ```
 
 @packageDocumentation
 */

export {
  flattenCatalogEntries,
  parseCatalogFromYaml,
} from './parse.ts';
export {
  readCatalogFile,
} from './read.ts';
export {
  isValidPackageName,
} from './package-name.ts';
export type {
  CatalogDocument,
  CatalogEntry,
  CatalogFile,
  CatalogMap,
  FlattenCatalogEntriesOptions,
  ReadCatalogFileOptions,
} from './types.ts';
