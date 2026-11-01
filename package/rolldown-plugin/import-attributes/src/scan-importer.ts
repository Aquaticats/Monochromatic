/**
 Scans importer source files to discover import attribute types
 for dynamic imports that rolldown's Rust scanner processes
 before `transform` runs.
 
 Uses AST parsing via rolldown's `parseSync` instead of regex
 for robust, formatting-independent matching.
 
 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  type ESTree,
  parseSync,
  Visitor,
} from 'rolldown/utils';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  extractTypeFromAttributes,
  extractTypeFromOptions,
  getStringLiteralValue,
  NO_ATTR_TYPE,
} from './ast-extract.ts';

/**
 Scans an importer file's source code to find the import attribute type
 associated with a given specifier.
 
 This is used for dynamic imports where rolldown's Rust scanner discovers
 dependencies before the `transform` hook can rewrite them. The matched
 declaration's attribute type is read with {@link extractTypeFromAttributes}
 (static forms) or {@link extractTypeFromOptions} (dynamic `import()`).
 
 @param specifier - import specifier to look for (e.g. `./sample.sql`)
 
 @param importerPath - absolute path to the importing file
 
 @param importerSourceCache - cache to avoid re-reading the same file
 
 @returns attribute type string if found and supported, {@link NO_ATTR_TYPE} otherwise
 
 @example
 ```ts
 const cache = new Map<string, string>();
 const attrType = await scanImporterForAttribute({
   specifier: './sample.sql',
   importerPath: '/project/src/main.ts',
   importerSourceCache: cache,
 },);
 // 'text' if main.ts contains: import x from './sample.sql' with { type: 'text' }
 ```
 */
export async function scanImporterForAttribute({
  specifier,
  importerPath,
  importerSourceCache,
}: {
  readonly specifier: string;
  readonly importerPath: string;
  importerSourceCache: Map<string, string>;
},): Promise<string | typeof NO_ATTR_TYPE> {
  /**
   Importer source text; lazily read from disk on cache miss and stored back.
   */
  let source = importerSourceCache.get(importerPath,);
  if (source === undefined) {
    try {
      source = await readFile(
        importerPath,
        'utf8',
      );
      importerSourceCache.set(
        importerPath,
        source,
      );
    }
    catch (error: unknown) {
      // Best-effort scan: an unreadable or virtual importer (e.g. a plugin-generated
      // module whose id is not a real filesystem path) has no source to scan, so
      // report no attribute rather than failing the build; surface the cause for diagnosis.
      console.warn(
        `import-attributes: could not read importer ${importerPath} while scanning for import attributes; treating as no attribute type.`,
        error,
      );
      return NO_ATTR_TYPE;
    }
  }

  if (!source.includes(specifier,))
    return NO_ATTR_TYPE;

  /**
   Parsed AST root produced by rolldown's parser.
   */
  const result = parseSync(
    importerPath,
    source,
  );
  /**
   Mutable accumulator written by the visitor when a matching specifier is encountered.
   */
  let found: string | typeof NO_ATTR_TYPE = NO_ATTR_TYPE;

  /**
   AST visitor that records the attribute type on the first matching specifier.
   */
  const visitor = new Visitor({
    ImportDeclaration(node: ForeignBorrowed<ESTree.ImportDeclaration>,): void {
      if (found !== NO_ATTR_TYPE)
        return;
      if ((node.source
        .value
        !== specifier) || (node.attributes
          .length
          === 0))
        return;
      found = extractTypeFromAttributes(node.attributes,);
    },

    ExportNamedDeclaration(node: ForeignBorrowed<ESTree.ExportNamedDeclaration>,): void {
      if (found !== NO_ATTR_TYPE)
        return;
      if ((node.source
        === null)
        || (node.source
          .value
          !== specifier)
        || (node.attributes
          .length
          === 0))
        {
        return;
      }
      found = extractTypeFromAttributes(node.attributes,);
    },

    ExportAllDeclaration(node: ForeignBorrowed<ESTree.ExportAllDeclaration>,): void {
      if (found !== NO_ATTR_TYPE)
        return;
      if ((node.source
        .value
        !== specifier) || (node.attributes
          .length
          === 0))
        return;
      found = extractTypeFromAttributes(node.attributes,);
    },

    ImportExpression(node: ForeignBorrowed<ESTree.ImportExpression>,): void {
      if (found !== NO_ATTR_TYPE)
        return;
      if (node.options
        === null)
        return;
      /**
       Literal specifier text; computed sources are skipped.
       */
      const sourceValue = getStringLiteralValue(node.source,);
      if (sourceValue !== specifier)
        return;
      found = extractTypeFromOptions(node.options,);
    },
  },);

  visitor.visit(result.program,);
  return found;
}
