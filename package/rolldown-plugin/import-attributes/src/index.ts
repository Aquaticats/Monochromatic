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
import type {
  LoadResult,
  Plugin,
  PluginContext,
  ResolveIdExtraOptions,
  ResolveIdResult,
  TransformResult,
} from 'rolldown';

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

//region Resolve and tag

/**
 Returns the file path of module ID, without any query parameters.
 
 An importer ID can carry a query of its own (`main.ts?variant=1`), and only the
 path before it names a file that exists on disk, so both the record read and the
 directory used for a relative fallback take this form.
 
 @param id - module ID that may carry a query string
 
 @returns path before the first `?`, or the ID itself when it carries none
 
 @example
 ```ts
 filePathOf('/project/src/main.ts?__importattr=text'); // '/project/src/main.ts'
 ```
 */
function filePathOf(id: string,): string {
  return id.split('?',)[0]
    ?? id;
}

/**
 Resolves one specifier and tags the result with the attribute query, so the
 load hook can hand the file to the handler for that attribute type.
 
 Both resolveId entry paths call this: the query-tagged specifier the transform
 already rewrote, and the untagged specifier whose clause the importer scan
 found. Rolldown's own resolver answers first; when it returns `null` a relative
 specifier falls back to the importer-relative absolute path, which is how a
 file rolldown cannot resolve on its own still loads through the handler.
 
 @param context - plugin context whose `resolve` runs rolldown's resolver
 
 @param specifier - specifier to resolve, already stripped of any attribute query
 
 @param importer - importing module; absent when rolldown reports an entry module
 
 @param attrType - attribute type encoded into the resolved ID
 
 @param options - extra options the resolveId hook received; forwarded as a copy
 
 @returns tagged resolution, or `null` when neither the resolver nor the relative fallback applies
 
 @example
 ```ts
 const resolved = await resolveTaggedSpecifier({
   context: this,
   specifier: './schema.sql',
   importer: '/project/src/main.ts',
   attrType: 'text',
   options,
 });
 // resolved.id: '/project/src/schema.sql?__importattr=text'
 ```
 */
async function resolveTaggedSpecifier({
  context,
  specifier,
  importer,
  attrType,
  options,
}: {
  readonly context: PluginContext;
  readonly specifier: string;
  readonly importer?: string;
  readonly attrType: string;
  readonly options: ResolveIdExtraOptions;
},): Promise<ResolveIdResult> {
  /**
   Resolved descriptor from the underlying resolver; `null` triggers the relative-path fallback.
   */
  const resolved = await context.resolve(
    specifier,
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

  /**
   An entry has no directory to resolve against, and an absolute or bare specifier has no relative base either.
   */
  if ((importer === undefined) || (!specifier.startsWith('.',)))
    return null;

  /**
   Importer directory used as the base for the relative-resolution fallback; the attribute query is not part of the path.
   */
  const importerDir = dirname(filePathOf(importer,),);
  /**
   Absolute path produced when the resolver could not locate the target.
   */
  const absolutePath = resolve(
    importerDir,
    specifier,
  );
  return {
    id: `${absolutePath}?${ATTR_QUERY_KEY}=${attrType}`,
    external: false,
  };
}

//endregion Resolve and tag

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
  return {
    name: 'import-attributes',

    /**
     Delegates to {@link transformImportAttributes} to rewrite
     `with { type: '...' }` clauses into query-parameter-tagged specifiers.
     */
    transform(
      code,
      id,
    ): TransformResult {
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
    ): Promise<ResolveIdResult> {
      /**
       Check for query-param-tagged specifiers (from static imports after transform).
       */
      const queryAttrType = extractAttrType(source,);
      if (queryAttrType !== NO_QUERY_ATTR) {
        /**
         Tagged specifier: the transform already decided the type, so only resolution and relative fallback remain.
         */
        return await resolveTaggedSpecifier({
          context: this,
          specifier: stripAttrQuery(source,),
          ...(importer === undefined ? {} : { importer, }),
          attrType: queryAttrType,
          options,
        },);
      }

      /*
       Every remaining case handles an untagged specifier, where only the
       importer's own source can say whether an attribute clause was written.
       */
      if (importer === undefined)
        return null;

      /**
       Attribute type discovered by scanning the importer's AST for this specifier.
       */
      const attrType = await scanImporterForAttribute({
        specifier: source,
        importerPath: filePathOf(importer,),
      },);

      if (attrType === NO_ATTR_TYPE)
        return null;

      return await resolveTaggedSpecifier({
        context: this,
        specifier: source,
        importer,
        attrType,
        options,
      },);
    },

    /**
     Loads modules tagged with `?__importattr=<type>` (skipping past
     {@link NO_QUERY_ATTR}) by reading the file and passing its content
     through the registered {@link HANDLERS} handler.
     */
    async load(id,): Promise<LoadResult> {
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
  staticDeclAttrType,
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
