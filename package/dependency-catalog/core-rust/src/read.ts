/**
 Reading `rust-catalog.toml` from disk.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { parseRustCatalog, } from './parse.ts';
import type { RustCatalogDocument, } from './types.ts';

/**
 File name of the catalog at the repository root.
 */
export const RUST_CATALOG_FILENAME = 'rust-catalog.toml';

/**
 Reads and validates the catalog in `rootDir`.

 @param rootDir - directory holding `rust-catalog.toml`, usually the repository root

 @returns the exact source text and the validated document; tools that rewrite the file keep `source` for
 comment-preserving edits

 @throws RustCatalogError when the catalog is invalid

 @example
 ```ts
 const { document, } = await readRustCatalog({ rootDir: process.cwd(), });
 ```
 */
export async function readRustCatalog(
  { rootDir, }: { readonly rootDir: string; },
): Promise<{
  readonly source: string;
  readonly sourcePath: string;
  readonly document: RustCatalogDocument
}> {
  /**
   Absolute path of the catalog file.
   */
  const sourcePath = join(
    rootDir,
    RUST_CATALOG_FILENAME,
  );
  /**
   Exact file text.
   */
  const source = await readFile(
    sourcePath,
    'utf8',
  );
  return {
    source,
    sourcePath,
    document: parseRustCatalog({
      source,
      sourcePath,
    },),
  };
}
