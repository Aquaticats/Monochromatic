/**
 Parsing and validation of `rust-catalog.toml`, the repository's pnpm-style Rust dependency catalog.

 @example
 ```ts
 import { readRustCatalog, } from '@monochromatic-dev/module-rust-catalog';

 const { document, } = await readRustCatalog({ rootDir: process.cwd(), });
 console.info(document.catalog.get('anyhow'));
 ```

 @packageDocumentation
 */

export { RustCatalogError, } from './errors.ts';
export { parseRustCatalog, } from './parse.ts';
export {
  readRustCatalog,
  RUST_CATALOG_FILENAME,
} from './read.ts';
export {
  floorVersion,
  isBoundedRequirement,
  isCrateName,
  isFloorRequirement,
  isVersion,
} from './syntax.ts';
export type {
  RustCatalogDocument,
  RustCatalogEntry,
  RustCatalogGitEntry,
  RustCatalogIssue,
  RustCatalogMap,
  RustCatalogRegistryEntry,
  RustCatalogSettings,
  RustCatalogTrustPolicy,
} from './types.ts';
