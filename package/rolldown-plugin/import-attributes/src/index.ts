/**
 Rolldown plugin that transforms import attributes (`with { type: '...' }`)
 into bundler-compatible module loads.
 
 Rolldown parses import attributes and preserves them in ESM output,
 but does not use them to influence module loading (rolldown#2758).
 This plugin bridges the gap by:
 
 1. Rewriting static import specifiers to encode the attribute as a query parameter
 2. Intercepting dynamic import resolution by scanning the importer's source
 3. Loading matched files with the appropriate transform (e.g. text -> default string export)
 
 Static imports are handled via `transform` (rewriting specifiers before rolldown scans them).
 Dynamic imports require special handling because rolldown's Rust scanner discovers
 dependencies from the original AST before `transform` runs on the importing file.
 
 @example
 ```ts
 // tsdown.node.config.ts
 import { importAttributesPlugin } from '@monochromatic-dev/rolldown-plugin-import-attributes/ts';
 
 export default defineConfig({
   plugins: [importAttributesPlugin()],
 });
 ```
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  dirname,
  resolve,
} from 'node:path';
import type { Plugin, } from 'rolldown';

import { NO_ATTR_TYPE, } from './ast-extract.ts';
import { HANDLERS, } from './handlers.ts';
import {
  ATTR_QUERY_KEY,
  extractAttrType,
  NO_QUERY_ATTR,
  stripAttrQuery,
} from './patterns.ts';
import { scanImporterForAttribute, } from './scan-importer.ts';
import {
  NO_TRANSFORM,
  transformImportAttributes,
} from './transform.ts';

export { importAttributesPlugin, };

//region Plugin factory

/**
 Creates a rolldown plugin that transforms import attributes into bundler-compatible loads.
 Handles all supported attribute types (`text`).
 
 @returns Rolldown plugin instance
 
 @example
 ```ts
 import { importAttributesPlugin } from '\@monochromatic-dev/rolldown-plugin-import-attributes/ts';
 import { defineConfig } from 'tsdown';
 
 export default defineConfig({
   plugins: [importAttributesPlugin()],
 });
 ```
 */
function importAttributesPlugin(): Plugin {
  /**
   Cache of importer file sources, keyed by absolute path.
   Prevents re-reading the same file when multiple imports from it
   trigger `resolveId` before `transform` has run.
   */
  const importerSourceCache = new Map<string, string>();

  return {
    name: 'import-attributes',

    /**
     Delegates to {@link transformImportAttributes} to rewrite
     `with { type: '...' }` clauses into query-parameter-tagged specifiers.
     */
    transform(
      code,
      id,
    ) {
      /**
       Transformed module, or {@link NO_TRANSFORM} when no attribute clause was present.
       */
      const result = transformImportAttributes({
        code,
        id,
      },);
      // Rolldown's transform hook signals "no change" with null; convert the NO_TRANSFORM sentinel back at the boundary.
      if (result === NO_TRANSFORM)
        return null;
      return result;
    },

    /**
     Resolves specifiers tagged with `?__importattr=<type>` (from transform rewriting,
     detected past {@link NO_QUERY_ATTR}) or specifiers discovered by rolldown's
     scanner before transform ran (for dynamic imports where the scanner
     processes the original AST).
     
     The options object is read, never written: `this.resolve` receives a fresh
     spread copy with `skipSelf: true`, so a caller can reuse its own object.
     
     For untagged specifiers, scans the importer's source via
     {@link scanImporterForAttribute} to check whether the import had a
     `with { type: '...' }` clause.
     */
    async resolveId(
      source,
      importer,
      options,
    ) {
      /**
       Check for query-param-tagged specifiers (from static imports after transform).
       */
      const queryAttrType = extractAttrType(source,);
      if (queryAttrType !== NO_QUERY_ATTR) {
        /**
         Specifier without the attribute query so the downstream resolver can locate the file.
         */
        const cleanSource = stripAttrQuery(source,);

        /**
         Resolved descriptor from the underlying resolver; `null` triggers the relative-path fallback.
         */
        const resolved = await this.resolve(
          cleanSource,
          importer,
          {
            ...options,
            skipSelf: true,
          },
        );

        if (resolved !== null) {
          return {
            id: `${resolved.id}?${ATTR_QUERY_KEY}=${queryAttrType}`,
            external: false,
          };
        }

        if ((importer !== undefined) && (cleanSource
          .startsWith('.',))) {
          /**
           Importer directory used as the base for resolving the relative specifier.
           */
          const importerDir = dirname(importer.split('?',)[0]
            ?? importer,);
          /**
           Absolute path produced when the resolver could not locate the target.
           */
          const absolutePath = resolve(
            importerDir,
            cleanSource,
          );
          return {
            id: `${absolutePath}?${ATTR_QUERY_KEY}=${queryAttrType}`,
            external: false,
          };
        }

        return null;
      }

      /**
       For untagged specifiers from an importer, scan the importer's source
       to check if this import has an attribute clause.
       This catches dynamic imports that rolldown's scanner discovered
       from the original AST before `transform` could rewrite them.
       */
      if (importer !== undefined) {
        /**
         Importer path stripped of any attribute query; used both as scan target and base directory.
         */
        const cleanImporter = importer.split('?',)[0]
          ?? importer;
        /**
         Attribute type discovered by scanning the importer's AST for this specifier.
         */
        const attrType = await scanImporterForAttribute({
          specifier: source,
          importerPath: cleanImporter,
          importerSourceCache,
        },);

        if (attrType !== NO_ATTR_TYPE) {
          /**
           Resolved descriptor for an untagged dynamic-import specifier; `null` triggers the relative fallback.
           */
          const resolved = await this.resolve(
            source,
            importer,
            {
              ...options,
              skipSelf: true,
            },
          );

          if (resolved !== null) {
            return {
              id: `${resolved.id}?${ATTR_QUERY_KEY}=${attrType}`,
              external: false,
            };
          }

          if (source.startsWith('.',)) {
            /**
             Importer directory used as the base for the relative-resolution fallback.
             */
            const importerDir = dirname(cleanImporter,);
            /**
             Absolute path produced when the resolver returned `null`.
             */
            const absolutePath = resolve(
              importerDir,
              source,
            );
            return {
              id: `${absolutePath}?${ATTR_QUERY_KEY}=${attrType}`,
              external: false,
            };
          }
        }
      }

      return null;
    },

    /**
     Loads modules tagged with `?__importattr=<type>` (skipping past
     {@link NO_QUERY_ATTR}) by reading the file and passing its content
     through the registered {@link HANDLERS} handler.
     */
    async load(id,) {
      /**
       Attribute type encoded in the requested ID; absent IDs are left to other loaders.
       */
      const attrType = extractAttrType(id,);
      if (attrType === NO_QUERY_ATTR)
        return null;

      /**
       Registered transformer for this attribute type; absent types are left to other loaders.
       */
      const handler = HANDLERS[attrType];
      if (handler === undefined)
        return null;

      /**
       File path stripped of the attribute query so the bytes can be read from disk.
       */
      const filePath = stripAttrQuery(id,);
      /**
       Raw file contents fed into the handler.
       */
      const content = await readFile(
        filePath,
        'utf8',
      );
      /**
       Module source produced by the handler, returned as rolldown's loaded module.
       */
      const moduleCode = handler(
        content,
        filePath,
      );

      return { code: moduleCode, };
    },
  };
}

//endregion Plugin factory

//region Built-artifact test surface

/**
 Query-parameter codec for the attribute marker, exported for built-artifact verification.

 @internal
 */
export {
  ATTR_QUERY_KEY,
  extractAttrType,
  NO_QUERY_ATTR,
  stripAttrQuery,
} from './patterns.ts';

/**
 AST attribute-type extraction, exported for built-artifact verification.

 @internal
 */
export {
  extractTypeFromAttributes,
  extractTypeFromOptions,
  getStringLiteralValue,
  NO_ATTR_TYPE,
  NON_STRING_NODE,
} from './ast-extract.ts';

/**
 Attribute handlers, exported for built-artifact verification.

 @internal
 */
export {
  HANDLERS,
  textHandler,
} from './handlers.ts';

/**
 Source-span replacement helpers, exported for built-artifact verification.

 @internal
 */
export {
  collectStaticReplacements,
  skipWithClauseWhitespace,
} from './transform-helpers.ts';

/**
 The transform-hook implementation, exported for built-artifact verification.

 @internal
 */
export {
  NO_TRANSFORM,
  transformImportAttributes,
} from './transform.ts';

/**
 The importer-source scanner, exported for built-artifact verification.

 @internal
 */
export { scanImporterForAttribute, } from './scan-importer.ts';

//endregion Built-artifact test surface
